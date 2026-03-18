use std::path::{Path, PathBuf};

pub const LIBRARY_DB_FILENAME: &str = "library.sqlite";
pub const KOSHELF_DB_FILENAME: &str = "koshelf.sqlite";

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct RuntimeDataPathOptions {
    pub data_path: Option<PathBuf>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuntimeDataLifecycle {
    Ephemeral,
    Persistent,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuntimeDataPolicySource {
    DataPath,
    AutoEphemeral,
}

impl RuntimeDataPolicySource {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::DataPath => "data_path",
            Self::AutoEphemeral => "auto_ephemeral",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeDataPolicy {
    pub lifecycle: RuntimeDataLifecycle,
    pub source: RuntimeDataPolicySource,
    /// User-provided persistent data path (only set for persistent policies).
    pub data_path: Option<PathBuf>,
    /// Resolved directory for runtime data files (currently: library DB).
    /// Always set once `set_resolved_data_dir` is called — either the
    /// user-provided persistent path or the ephemeral temp directory.
    resolved_data_dir: Option<PathBuf>,
}

impl RuntimeDataPolicy {
    /// The user-provided persistent data directory, if any.
    pub fn persistent_data_dir(&self) -> Option<&Path> {
        if self.is_persistent() {
            self.data_path.as_deref()
        } else {
            None
        }
    }

    pub fn is_persistent(&self) -> bool {
        matches!(self.lifecycle, RuntimeDataLifecycle::Persistent)
    }

    /// Set the resolved runtime data directory.
    ///
    /// For persistent policies this should equal `data_dir`; for ephemeral
    /// policies it points to the shared temp directory.
    pub fn set_resolved_data_dir(&mut self, path: PathBuf) {
        self.resolved_data_dir = Some(path);
    }

    /// The resolved directory where runtime data files reside.
    pub fn resolved_data_dir(&self) -> Option<&Path> {
        self.resolved_data_dir.as_deref()
    }

    /// Path to the library SQLite cache inside the resolved data directory.
    pub fn library_db_path(&self) -> Option<PathBuf> {
        self.resolved_data_dir
            .as_deref()
            .map(|dir| dir.join(LIBRARY_DB_FILENAME))
    }

    /// Path to the KoShelf application DB inside the resolved data directory.
    pub fn koshelf_db_path(&self) -> Option<PathBuf> {
        self.resolved_data_dir
            .as_deref()
            .map(|dir| dir.join(KOSHELF_DB_FILENAME))
    }
}

pub fn resolve_runtime_data_policy(cli: &RuntimeDataPathOptions) -> RuntimeDataPolicy {
    non_empty_path(cli.data_path.as_ref())
        .map(|data_path| persistent_policy(data_path, RuntimeDataPolicySource::DataPath))
        .unwrap_or_else(ephemeral_policy)
}

fn non_empty_path(path: Option<&PathBuf>) -> Option<PathBuf> {
    path.filter(|candidate| !candidate.as_os_str().is_empty())
        .cloned()
}

fn persistent_policy(path: PathBuf, source: RuntimeDataPolicySource) -> RuntimeDataPolicy {
    RuntimeDataPolicy {
        lifecycle: RuntimeDataLifecycle::Persistent,
        source,
        resolved_data_dir: Some(path.clone()),
        data_path: Some(path),
    }
}

fn ephemeral_policy() -> RuntimeDataPolicy {
    RuntimeDataPolicy {
        lifecycle: RuntimeDataLifecycle::Ephemeral,
        source: RuntimeDataPolicySource::AutoEphemeral,
        data_path: None,
        resolved_data_dir: None,
    }
}

#[cfg(test)]
mod tests {
    use super::{
        KOSHELF_DB_FILENAME, LIBRARY_DB_FILENAME, RuntimeDataLifecycle, RuntimeDataPathOptions,
        RuntimeDataPolicySource, resolve_runtime_data_policy,
    };
    use std::path::PathBuf;

    fn options(data_path: Option<&str>) -> RuntimeDataPathOptions {
        RuntimeDataPathOptions {
            data_path: data_path.map(PathBuf::from),
        }
    }

    #[test]
    fn cli_data_path_enables_persistent_policy() {
        let cli = options(Some("/cli/data"));

        let policy = resolve_runtime_data_policy(&cli);

        assert_eq!(policy.lifecycle, RuntimeDataLifecycle::Persistent);
        assert_eq!(policy.source, RuntimeDataPolicySource::DataPath);
        assert_eq!(policy.data_path, Some(PathBuf::from("/cli/data")));
        assert_eq!(
            policy.persistent_data_dir(),
            Some(std::path::Path::new("/cli/data"))
        );
    }

    #[test]
    fn unresolved_paths_fall_back_to_ephemeral_mode() {
        let cli = options(None);

        let policy = resolve_runtime_data_policy(&cli);

        assert_eq!(policy.lifecycle, RuntimeDataLifecycle::Ephemeral);
        assert_eq!(policy.source, RuntimeDataPolicySource::AutoEphemeral);
        assert_eq!(policy.data_path, None);
        assert_eq!(policy.persistent_data_dir(), None);
    }

    #[test]
    fn empty_paths_are_treated_as_unset() {
        let cli = options(Some(""));

        let policy = resolve_runtime_data_policy(&cli);

        assert_eq!(policy.lifecycle, RuntimeDataLifecycle::Ephemeral);
        assert_eq!(policy.source, RuntimeDataPolicySource::AutoEphemeral);
        assert_eq!(policy.data_path, None);
    }

    #[test]
    fn library_db_path_is_derived_from_resolved_data_dir() {
        let cli = options(Some("/runtime/data"));

        let policy = resolve_runtime_data_policy(&cli);

        assert_eq!(
            policy.library_db_path(),
            Some(PathBuf::from(format!(
                "/runtime/data/{LIBRARY_DB_FILENAME}"
            )))
        );
    }

    #[test]
    fn ephemeral_policy_resolves_library_db_after_setting_data_dir() {
        let cli = options(None);
        let mut policy = resolve_runtime_data_policy(&cli);

        assert!(policy.library_db_path().is_none());

        policy.set_resolved_data_dir(PathBuf::from("/tmp/koshelf-abc"));

        assert_eq!(
            policy.library_db_path(),
            Some(PathBuf::from(format!(
                "/tmp/koshelf-abc/{LIBRARY_DB_FILENAME}"
            )))
        );
        // persistent_data_dir still returns None for ephemeral policies
        assert_eq!(policy.persistent_data_dir(), None);
    }

    #[test]
    fn koshelf_db_path_is_derived_from_resolved_data_dir() {
        let cli = options(Some("/runtime/data"));

        let policy = resolve_runtime_data_policy(&cli);

        assert_eq!(
            policy.koshelf_db_path(),
            Some(PathBuf::from(format!(
                "/runtime/data/{KOSHELF_DB_FILENAME}"
            )))
        );
    }
}
