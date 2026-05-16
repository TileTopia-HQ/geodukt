//! Caching layer — cache intermediate pipeline results to disk.

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use crate::incremental::hash_bytes;

/// Cache for intermediate pipeline results.
pub struct ResultCache {
    cache_dir: PathBuf,
    /// Map from cache key to whether the cached result is still valid.
    validity: HashMap<String, bool>,
}

impl ResultCache {
    pub fn new(cache_dir: &Path) -> Self {
        fs::create_dir_all(cache_dir).ok();
        Self {
            cache_dir: cache_dir.to_path_buf(),
            validity: HashMap::new(),
        }
    }

    /// Generate a cache key based on node name and input hash.
    pub fn cache_key(&self, node_name: &str, input_hash: &str) -> String {
        let combined = format!("{node_name}:{input_hash}");
        hash_bytes(combined.as_bytes())
    }

    /// Check if a cached result exists and is valid.
    pub fn has_valid_cache(&self, key: &str) -> bool {
        self.cache_path(key).exists()
    }

    /// Read cached data.
    pub fn read_cache(&self, key: &str) -> Option<Vec<u8>> {
        fs::read(self.cache_path(key)).ok()
    }

    /// Write data to cache.
    pub fn write_cache(&self, key: &str, data: &[u8]) -> std::io::Result<()> {
        fs::write(self.cache_path(key), data)
    }

    /// Invalidate a cache entry.
    pub fn invalidate(&mut self, key: &str) {
        fs::remove_file(self.cache_path(key)).ok();
        self.validity.insert(key.to_string(), false);
    }

    /// Clear entire cache.
    pub fn clear(&self) -> std::io::Result<()> {
        if self.cache_dir.exists() {
            fs::remove_dir_all(&self.cache_dir)?;
            fs::create_dir_all(&self.cache_dir)?;
        }
        Ok(())
    }

    /// Get the path for a cache entry.
    fn cache_path(&self, key: &str) -> PathBuf {
        self.cache_dir.join(format!("{key}.cache"))
    }

    /// List all cache entries.
    pub fn entries(&self) -> Vec<String> {
        fs::read_dir(&self.cache_dir)
            .into_iter()
            .flatten()
            .filter_map(|e| e.ok())
            .filter_map(|e| {
                let name = e.file_name().to_str()?.to_string();
                name.strip_suffix(".cache").map(|s| s.to_string())
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_cache_roundtrip() {
        let dir = TempDir::new().unwrap();
        let cache = ResultCache::new(dir.path());

        let key = cache.cache_key("transform_a", "abc123");
        assert!(!cache.has_valid_cache(&key));

        cache.write_cache(&key, b"cached data").unwrap();
        assert!(cache.has_valid_cache(&key));

        let data = cache.read_cache(&key).unwrap();
        assert_eq!(data, b"cached data");
    }

    #[test]
    fn test_cache_clear() {
        let dir = TempDir::new().unwrap();
        let cache = ResultCache::new(dir.path());
        cache.write_cache("test", b"data").unwrap();
        assert!(cache.has_valid_cache("test"));
        cache.clear().unwrap();
        assert!(!cache.has_valid_cache("test"));
    }
}
