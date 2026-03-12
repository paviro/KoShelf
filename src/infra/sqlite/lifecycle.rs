use std::path::{Path, PathBuf};

pub const LIBRARY_DB_FILENAME: &str = "library.sqlite";

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct RuntimeDataPathOptions {
    pub data_dir: Option<PathBuf>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuntimeDataLifecycle {
    Ephemeral,
    Persistent,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuntimeDataPolicySource {
    CliDataDir,
    AutoEphemeral,
}

impl RuntimeDataPolicySource {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::CliDataDir => "cli_data_dir",
            Self::AutoEphemeral => "auto_ephemeral",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeDataPolicy {
    pub lifecycle: RuntimeDataLifecycle,
    pub source: RuntimeDataPolicySource,
    pub data_dir: Option<PathBuf>,
}

impl RuntimeDataPolicy {
    pub fn persistent_data_dir(&self) -> Option<&Path> {
        self.data_dir.as_deref()
    }

    pub fn is_persistent(&self) -> bool {
        matches!(self.lifecycle, RuntimeDataLifecycle::Persistent)
    }

    pub fn library_db_path(&self) -> Option<PathBuf> {
        self.persistent_data_dir()
            .map(|data_dir| data_dir.join(LIBRARY_DB_FILENAME))
    }
}

pub fn resolve_runtime_data_policy(cli: &RuntimeDataPathOptions) -> RuntimeDataPolicy {
    non_empty_path(cli.data_dir.as_ref())
        .map(|data_dir| persistent_policy(data_dir, RuntimeDataPolicySource::CliDataDir))
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
        data_dir: Some(path),
    }
}

fn ephemeral_policy() -> RuntimeDataPolicy {
    RuntimeDataPolicy {
        lifecycle: RuntimeDataLifecycle::Ephemeral,
        source: RuntimeDataPolicySource::AutoEphemeral,
        data_dir: None,
    }
}

#[cfg(test)]
mod tests {
    use super::{
        LIBRARY_DB_FILENAME, RuntimeDataLifecycle, RuntimeDataPathOptions, RuntimeDataPolicySource,
        resolve_runtime_data_policy,
    };
    use std::path::PathBuf;

    fn options(data_dir: Option<&str>) -> RuntimeDataPathOptions {
        RuntimeDataPathOptions {
            data_dir: data_dir.map(PathBuf::from),
        }
    }

    #[test]
    fn cli_data_dir_enables_persistent_policy() {
        let cli = options(Some("/cli/data"));

        let policy = resolve_runtime_data_policy(&cli);

        assert_eq!(policy.lifecycle, RuntimeDataLifecycle::Persistent);
        assert_eq!(policy.source, RuntimeDataPolicySource::CliDataDir);
        assert_eq!(policy.data_dir, Some(PathBuf::from("/cli/data")));
    }

    #[test]
    fn unresolved_paths_fall_back_to_ephemeral_mode() {
        let cli = options(None);

        let policy = resolve_runtime_data_policy(&cli);

        assert_eq!(policy.lifecycle, RuntimeDataLifecycle::Ephemeral);
        assert_eq!(policy.source, RuntimeDataPolicySource::AutoEphemeral);
        assert_eq!(policy.data_dir, None);
    }

    #[test]
    fn empty_paths_are_treated_as_unset() {
        let cli = options(Some(""));

        let policy = resolve_runtime_data_policy(&cli);

        assert_eq!(policy.lifecycle, RuntimeDataLifecycle::Ephemeral);
        assert_eq!(policy.source, RuntimeDataPolicySource::AutoEphemeral);
        assert_eq!(policy.data_dir, None);
    }

    #[test]
    fn library_db_path_is_derived_from_data_dir() {
        let cli = options(Some("/runtime/data"));

        let policy = resolve_runtime_data_policy(&cli);

        assert_eq!(
            policy.library_db_path(),
            Some(PathBuf::from(format!(
                "/runtime/data/{LIBRARY_DB_FILENAME}"
            )))
        );
    }
}
