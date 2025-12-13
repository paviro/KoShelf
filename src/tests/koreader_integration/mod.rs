//! Integration tests that validate KoShelf against KoReader's reference implementation.
//!
//! These tests clone the KoReader repository and use its test data and Lua code
//! to verify that our Rust implementations produce identical results.

use serde::Deserialize;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::OnceLock;
use tempfile::TempDir;

/// Global test directory containing the cloned KoReader repository.
/// Uses OnceLock to ensure we only clone once per test run.
static KOREADER_DIR: OnceLock<KoReaderTestDir> = OnceLock::new();
static KOREADER_REF: OnceLock<String> = OnceLock::new();

const KOREADER_RELEASE_API: &str = "https://api.github.com/repos/koreader/koreader/releases/latest";
const DEFAULT_KOREADER_FALLBACK_REF: &str = "master";

/// Wrapper for the KoReader test directory that handles cleanup.
struct KoReaderTestDir {
    /// Temporary directory holding the clone (None if using existing clone)
    _temp_dir: Option<TempDir>,
    /// Path to the KoReader repository root
    path: PathBuf,
}

impl KoReaderTestDir {
    fn path(&self) -> &Path {
        &self.path
    }
}

#[derive(Debug, Deserialize)]
struct GithubRelease {
    tag_name: String,
}

/// Get the path to the KoReader repository for testing.
///
/// This function will:
/// 1. Check if KOREADER_TEST_PATH env var is set (use existing local clone)
/// 2. Otherwise, clone a shallow copy of KoReader into a temp directory
///
/// The clone is shared across all tests in a single test run.
fn get_koreader_dir() -> &'static KoReaderTestDir {
    KOREADER_DIR.get_or_init(|| {
        // Check for existing local clone via environment variable
        if let Ok(path) = std::env::var("KOREADER_TEST_PATH") {
            let path = PathBuf::from(path);
            if path.exists() && path.join("frontend").exists() {
                eprintln!("Using existing KoReader at: {}", path.display());
                return KoReaderTestDir {
                    _temp_dir: None,
                    path,
                };
            }
        }

        let git_ref = resolve_koreader_ref().to_string();

        // Clone KoReader into a temporary directory
        let temp_dir = TempDir::new().expect("Failed to create temp directory for KoReader");
        let koreader_path = temp_dir.path().join("koreader");

        eprintln!("Cloning KoReader source (ref '{}')...", git_ref);
        let status = Command::new("git")
            .args([
                "clone",
                "--depth=1",
                "--single-branch",
                "--branch",
                git_ref.as_str(),
                "https://github.com/koreader/koreader.git",
                koreader_path.to_str().unwrap(),
            ])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .expect("Failed to execute git clone");

        if !status.success() {
            panic!("Failed to clone KoReader repository");
        }

        // Also need to get the test submodule
        eprintln!("Fetching KoReader test data...");
        let status = Command::new("git")
            .args(["submodule", "update", "--init", "--depth=1", "test"])
            .current_dir(&koreader_path)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .expect("Failed to initialize test submodule");

        if !status.success() {
            eprintln!("Warning: Failed to initialize test submodule (tests may be limited)");
        }

        // Create the spec/front symlink structure that KoReader's Makefile normally creates:
        // - spec/front -> spec (so spec/front/unit resolves to spec/unit)
        // - spec/unit/data -> ../../test (test data files)
        //
        // This allows paths like "spec/front/unit/data/tall.pdf" to resolve correctly
        #[cfg(unix)]
        {
            use std::os::unix::fs::symlink;

            // Create spec/front -> spec (pointing to current directory relatively)
            let spec_front = koreader_path.join("spec/front");
            if !spec_front.exists() {
                // Use "." to make spec/front point to spec directory itself
                let _ = symlink(Path::new("."), &spec_front);
            }

            // Create spec/unit/data -> ../../test
            let data_link = koreader_path.join("spec/unit/data");
            if !data_link.exists() {
                let _ = symlink(Path::new("../../test"), &data_link);
            }
        }
        #[cfg(windows)]
        {
            use std::os::windows::fs::symlink_dir;

            let spec_front = koreader_path.join("spec/front");
            if !spec_front.exists() {
                let _ = symlink_dir(&koreader_path.join("spec"), &spec_front);
            }

            let data_link = koreader_path.join("spec/unit/data");
            if !data_link.exists() {
                let _ = symlink_dir(&koreader_path.join("test"), &data_link);
            }
        }

        KoReaderTestDir {
            _temp_dir: Some(temp_dir),
            path: koreader_path,
        }
    })
}

/// Determine which KoReader ref to use for compatibility tests.
///
/// Priority:
/// 1. KOREADER_TEST_REF env var (explicit override)
/// 2. Latest GitHub release tag
/// 3. Fallback to "master" if release lookup fails
fn resolve_koreader_ref() -> &'static str {
    KOREADER_REF.get_or_init(|| {
        if let Ok(ref_override) = std::env::var("KOREADER_TEST_REF") {
            eprintln!(
                "Using KoReader ref from KOREADER_TEST_REF: {}",
                ref_override
            );
            return ref_override;
        }

        match fetch_latest_release_tag() {
            Ok(tag) => {
                eprintln!("Using latest KoReader release tag: {}", tag);
                tag
            }
            Err(err) => {
                eprintln!(
                    "Warning: failed to fetch latest KoReader release tag ({}). Falling back to '{}'.",
                    err, DEFAULT_KOREADER_FALLBACK_REF
                );
                DEFAULT_KOREADER_FALLBACK_REF.to_string()
            }
        }
    })
}

fn fetch_latest_release_tag() -> Result<String, String> {
    let response = ureq::get(KOREADER_RELEASE_API)
        .call()
        .map_err(|e| format!("GitHub API request failed: {}", e))?;

    let payload = response
        .into_body()
        .read_to_string()
        .map_err(|e| format!("Failed to read release payload: {}", e))?;
    let release: GithubRelease =
        serde_json::from_str(&payload).map_err(|e| format!("Invalid release payload: {}", e))?;

    if release.tag_name.trim().is_empty() {
        Err("Release tag was empty".to_string())
    } else {
        Ok(release.tag_name)
    }
}

mod full_integration_tests;
mod partial_md5_tests;
mod sidecar_path_tests;
mod statistics_db_tests;
