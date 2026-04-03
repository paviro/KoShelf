use std::fs;
use std::io;
use std::path::Path;

pub(crate) fn env_flag(name: &str) -> bool {
    matches!(
        std::env::var(name).as_deref(),
        Ok("1") | Ok("true") | Ok("TRUE") | Ok("yes") | Ok("YES") | Ok("on") | Ok("ON")
    )
}

pub(crate) fn rerun_if_changed_recursive(dir: &Path) {
    if !dir.exists() {
        return;
    }

    // Watch the directory entry too, so add/remove/rename can trigger rebuilds.
    if let Some(p) = dir.to_str() {
        println!("cargo:rerun-if-changed={}", p);
    }

    let entries = match fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return,
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            rerun_if_changed_recursive(&path);
        } else if path.is_file() {
            // Best-effort: only print paths Cargo can parse nicely.
            if let Some(p) = path.to_str() {
                println!("cargo:rerun-if-changed={}", p);
            }
        }
    }
}

pub(crate) fn write_if_changed(path: &Path, bytes: &[u8]) -> io::Result<bool> {
    match fs::read(path) {
        Ok(existing) if existing == bytes => Ok(false),
        _ => {
            fs::write(path, bytes)?;
            Ok(true)
        }
    }
}

/// Returns the most recent modification time among the given paths, or None if
/// none of them exist.
pub(crate) fn newest_mtime(paths: &[&Path]) -> Option<std::time::SystemTime> {
    paths
        .iter()
        .filter_map(|p| fs::metadata(p).and_then(|m| m.modified()).ok())
        .max()
}
