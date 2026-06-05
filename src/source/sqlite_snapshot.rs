use anyhow::{Context, Result};
use std::fs;
use std::path::{Path, PathBuf};

/// Return the main SQLite database path and its WAL/SHM companion paths.
pub(crate) fn sqlite_snapshot_paths(db_path: &Path) -> [PathBuf; 3] {
    [
        db_path.to_path_buf(),
        sqlite_companion_path(db_path, "wal"),
        sqlite_companion_path(db_path, "shm"),
    ]
}

/// Check whether `path` is the SQLite database itself or one of its WAL/SHM companions.
pub(crate) fn is_sqlite_db_or_companion(path: &Path, db_path: &Path) -> bool {
    sqlite_snapshot_paths(db_path)
        .iter()
        .any(|candidate| path == candidate)
}

/// Copy a SQLite database and any present WAL/SHM companions to a temporary DB path.
pub(crate) fn copy_sqlite_snapshot(source_db: &Path, temp_db: &Path) -> Result<()> {
    fs::copy(source_db, temp_db).with_context(|| {
        format!(
            "Failed to copy SQLite database from {:?} to {:?}",
            source_db, temp_db
        )
    })?;

    for suffix in ["wal", "shm"] {
        let source = sqlite_companion_path(source_db, suffix);
        if !source.exists() {
            continue;
        }

        let destination = sqlite_companion_path(temp_db, suffix);
        fs::copy(&source, &destination).with_context(|| {
            format!(
                "Failed to copy SQLite companion from {:?} to {:?}",
                source, destination
            )
        })?;
    }

    Ok(())
}

fn sqlite_companion_path(db_path: &Path, suffix: &str) -> PathBuf {
    let Some(filename) = db_path.file_name() else {
        return db_path.with_extension(suffix);
    };

    db_path.with_file_name(format!("{}-{}", filename.to_string_lossy(), suffix))
}

#[cfg(test)]
mod tests {
    use super::{copy_sqlite_snapshot, is_sqlite_db_or_companion, sqlite_snapshot_paths};
    use std::fs;

    #[test]
    fn identifies_sqlite_main_wal_and_shm_paths() {
        let db = std::path::Path::new("/tmp/statistics.sqlite3");
        let paths = sqlite_snapshot_paths(db);

        assert_eq!(paths[0], db);
        assert_eq!(
            paths[1],
            std::path::Path::new("/tmp/statistics.sqlite3-wal")
        );
        assert_eq!(
            paths[2],
            std::path::Path::new("/tmp/statistics.sqlite3-shm")
        );

        assert!(is_sqlite_db_or_companion(db, db));
        assert!(is_sqlite_db_or_companion(&paths[1], db));
        assert!(is_sqlite_db_or_companion(&paths[2], db));
        assert!(!is_sqlite_db_or_companion(
            std::path::Path::new("/tmp/other.sqlite3"),
            db
        ));
    }

    #[test]
    fn copies_main_db_and_existing_companions() {
        let source_dir = tempfile::tempdir().expect("source temp dir");
        let temp_dir = tempfile::tempdir().expect("destination temp dir");
        let source_db = source_dir.path().join("KoboReader.sqlite");
        let temp_db = temp_dir.path().join("snapshot.sqlite");

        fs::write(&source_db, b"main").expect("source main");
        fs::write(source_dir.path().join("KoboReader.sqlite-wal"), b"wal").expect("source wal");
        fs::write(source_dir.path().join("KoboReader.sqlite-shm"), b"shm").expect("source shm");

        copy_sqlite_snapshot(&source_db, &temp_db).expect("copy snapshot");

        assert_eq!(fs::read(&temp_db).expect("temp main"), b"main");
        assert_eq!(
            fs::read(temp_dir.path().join("snapshot.sqlite-wal")).expect("temp wal"),
            b"wal"
        );
        assert_eq!(
            fs::read(temp_dir.path().join("snapshot.sqlite-shm")).expect("temp shm"),
            b"shm"
        );
    }
}
