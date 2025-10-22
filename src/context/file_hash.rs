use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tokio::fs as async_fs;

/// Tracks file hashes for cache invalidation
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FileHashMap {
    pub hashes: HashMap<String, String>,
    pub last_updated: chrono::DateTime<chrono::Utc>,
}

/// Manages file hash tracking for cache invalidation
pub struct FileHashTracker {
    hash_file: PathBuf,
}

impl FileHashTracker {
    /// Create a new file hash tracker
    pub fn new(cache_dir: PathBuf) -> Result<Self> {
        let hash_file = cache_dir.join("file_hashes.json");

        // Ensure cache directory exists
        if !cache_dir.exists() {
            std::fs::create_dir_all(&cache_dir).context("Failed to create cache directory")?;
        }

        Ok(Self { hash_file })
    }

    /// Load existing file hashes from disk
    pub async fn load_hashes(&self) -> Result<FileHashMap> {
        if !self.hash_file.exists() {
            return Ok(FileHashMap::default());
        }

        let contents = async_fs::read_to_string(&self.hash_file)
            .await
            .context("Failed to read file hashes")?;

        let hash_map: FileHashMap =
            serde_json::from_str(&contents).context("Failed to parse file hashes")?;

        Ok(hash_map)
    }

    /// Save file hashes to disk
    pub async fn save_hashes(&self, hash_map: &FileHashMap) -> Result<()> {
        let contents =
            serde_json::to_string_pretty(hash_map).context("Failed to serialize file hashes")?;

        async_fs::write(&self.hash_file, contents)
            .await
            .context("Failed to write file hashes")?;

        Ok(())
    }

    /// Calculate SHA256 hash of a file
    pub async fn calculate_file_hash<P: AsRef<Path>>(file_path: P) -> Result<String> {
        let contents = async_fs::read(file_path.as_ref())
            .await
            .context("Failed to read file for hashing")?;

        // Use a simple hash for now - could upgrade to SHA256 later if needed
        let hash = format!("{:x}", md5::compute(&contents));
        Ok(hash)
    }

    /// Check if files have changed since last cache
    pub async fn files_changed(&self, file_paths: &[PathBuf]) -> Result<bool> {
        let current_hashes = self.load_hashes().await?;

        for file_path in file_paths {
            let path_str = file_path.to_string_lossy().to_string();

            // File doesn't exist anymore - consider it changed
            if !file_path.exists() {
                if current_hashes.hashes.contains_key(&path_str) {
                    return Ok(true);
                }
                continue;
            }

            // Calculate current hash
            let current_hash = Self::calculate_file_hash(file_path).await?;

            // Compare with stored hash
            match current_hashes.hashes.get(&path_str) {
                Some(stored_hash) => {
                    if &current_hash != stored_hash {
                        return Ok(true);
                    }
                }
                None => {
                    // New file - consider it changed
                    return Ok(true);
                }
            }
        }

        Ok(false)
    }

    /// Update file hashes for a set of files
    pub async fn update_file_hashes(&self, file_paths: &[PathBuf]) -> Result<()> {
        let mut hash_map = self.load_hashes().await?;

        for file_path in file_paths {
            if file_path.exists() {
                let path_str = file_path.to_string_lossy().to_string();
                let hash = Self::calculate_file_hash(file_path).await?;
                hash_map.hashes.insert(path_str, hash);
            }
        }

        hash_map.last_updated = chrono::Utc::now();
        self.save_hashes(&hash_map).await?;

        Ok(())
    }

    /// Remove file hashes for files that no longer exist
    #[allow(dead_code)]
    pub async fn cleanup_missing_files(&self) -> Result<()> {
        let mut hash_map = self.load_hashes().await?;
        let mut to_remove = Vec::new();

        for path_str in hash_map.hashes.keys() {
            let path = PathBuf::from(path_str);
            if !path.exists() {
                to_remove.push(path_str.clone());
            }
        }

        for path_str in to_remove {
            hash_map.hashes.remove(&path_str);
        }

        if !hash_map.hashes.is_empty() {
            hash_map.last_updated = chrono::Utc::now();
            self.save_hashes(&hash_map).await?;
        }

        Ok(())
    }

    /// Get hash file path (for testing/debugging)
    #[allow(dead_code)]
    pub fn hash_file_path(&self) -> &Path {
        &self.hash_file
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use tokio::fs;

    #[tokio::test]
    async fn test_file_hash_tracker() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let cache_dir = temp_dir.path().join("cache");
        let tracker = FileHashTracker::new(cache_dir)?;

        // Create a test file
        let test_file = temp_dir.path().join("test.txt");
        fs::write(&test_file, "hello world").await?;

        // Initial state - file should be considered changed
        let changed = tracker.files_changed(&[test_file.clone()]).await?;
        assert!(changed);

        // Update hashes
        tracker.update_file_hashes(&[test_file.clone()]).await?;

        // Now file should not be considered changed
        let changed = tracker.files_changed(&[test_file.clone()]).await?;
        assert!(!changed);

        // Modify file
        fs::write(&test_file, "hello world modified").await?;

        // File should be considered changed again
        let changed = tracker.files_changed(&[test_file.clone()]).await?;
        assert!(changed);

        Ok(())
    }
}
