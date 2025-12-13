//! Cache manifest generation for PWA smart caching.
//!
//! Generates a cache-manifest.json file that maps URL paths to content hashes,
//! allowing the service worker to only re-cache files that have actually changed.

use anyhow::Result;
use serde::Serialize;
use std::collections::HashMap;
use std::fs;
use std::io::Write;
use std::path::Path;
use std::sync::Mutex;

/// Cache manifest containing version and file hashes.
#[derive(Serialize)]
pub struct CacheManifest {
    /// Build timestamp/version
    pub version: String,
    /// Map of URL paths to content hashes
    pub files: HashMap<String, String>,
}

/// Thread-safe wrapper for building the cache manifest during site generation.
pub struct CacheManifestBuilder {
    version: String,
    files: Mutex<HashMap<String, String>>,
}

impl CacheManifestBuilder {
    /// Create a new cache manifest builder with the given version.
    pub fn new(version: String) -> Self {
        Self {
            version,
            files: Mutex::new(HashMap::new()),
        }
    }

    /// Compute a short hash of the content (first 8 chars of hex-encoded MD5).
    pub fn compute_hash(content: &[u8]) -> String {
        let digest = md5::compute(content);
        format!("{:x}", digest)[..8].to_string()
    }

    /// Register a file in the manifest.
    ///
    /// - `url_path`: The URL path (e.g., "/books/my-book/")
    /// - `content`: The file content to hash
    pub fn register(&self, url_path: &str, content: &[u8]) {
        let hash = Self::compute_hash(content);
        let mut files = self.files.lock().unwrap();
        files.insert(url_path.to_string(), hash);
    }

    /// Register a file by its filesystem path, computing the URL path from the output directory.
    ///
    /// - `file_path`: Absolute path to the file
    /// - `output_dir`: The output directory root
    /// - `content`: The file content to hash
    pub fn register_file<P: AsRef<Path>, Q: AsRef<Path>>(
        &self,
        file_path: P,
        output_dir: Q,
        content: &[u8],
    ) {
        let file_path = file_path.as_ref();
        let output_dir = output_dir.as_ref();

        if let Ok(relative) = file_path.strip_prefix(output_dir) {
            let mut url_path = String::from("/");
            url_path.push_str(&relative.to_string_lossy().replace('\\', "/"));

            if url_path.ends_with("/index.html") {
                url_path = url_path.strip_suffix("index.html").unwrap().to_string();
            }

            self.register(&url_path, content);
        }
    }

    /// Write the cache manifest to disk.
    pub fn write<P: AsRef<Path>>(&self, output_path: P) -> Result<()> {
        let files = self.files.lock().unwrap();
        let manifest = CacheManifest {
            version: self.version.clone(),
            files: files.clone(),
        };

        let json = serde_json::to_string(&manifest)?;
        let mut file = fs::File::create(output_path)?;
        file.write_all(json.as_bytes())?;

        Ok(())
    }
}
