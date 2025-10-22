use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// File hash map for tracking file changes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileHashMap {
    pub hashes: HashMap<String, String>,
    pub last_updated: chrono::DateTime<chrono::Utc>,
}

/// File hash tracker for efficient change detection
pub struct FileHashTracker {
    hash_file: PathBuf,
}

impl FileHashTracker {
    pub fn new(cache_dir: PathBuf) -> Result<Self> {
        let hash_file = cache_dir.join("file_hashes.json");
        Ok(Self { hash_file })
    }

    /// Load existing file hashes
    pub async fn load_hashes(&self) -> Result<FileHashMap> {
        if !self.hash_file.exists() {
            return Ok(FileHashMap {
                hashes: HashMap::new(),
                last_updated: chrono::Utc::now(),
            });
        }

        let content = tokio::fs::read_to_string(&self.hash_file)
            .await
            .context("Failed to read file hashes")?;

        let hash_map: FileHashMap =
            serde_json::from_str(&content).context("Failed to parse file hashes JSON")?;

        Ok(hash_map)
    }

    /// Save file hashes to disk
    pub async fn save_hashes(&self, hash_map: &FileHashMap) -> Result<()> {
        let content =
            serde_json::to_string_pretty(hash_map).context("Failed to serialize file hashes")?;

        tokio::fs::write(&self.hash_file, content)
            .await
            .context("Failed to write file hashes")?;

        Ok(())
    }

    /// Calculate hash for a single file
    pub async fn calculate_file_hash(path: &PathBuf) -> Result<String> {
        let content = tokio::fs::read(path)
            .await
            .context("Failed to read file for hashing")?;

        let hash = md5::compute(&content);
        Ok(format!("{:x}", hash))
    }

    /// Check if any files have changed
    pub async fn files_changed(&self, file_paths: &[PathBuf]) -> Result<bool> {
        let hash_map = self.load_hashes().await?;

        for path in file_paths {
            if !path.exists() {
                continue;
            }

            let path_str = path.display().to_string();
            let current_hash = Self::calculate_file_hash(path).await?;

            if let Some(stored_hash) = hash_map.hashes.get(&path_str) {
                if stored_hash != &current_hash {
                    return Ok(true);
                }
            } else {
                // New file
                return Ok(true);
            }
        }

        Ok(false)
    }

    /// Update hashes for given files
    pub async fn update_file_hashes(&self, file_paths: &[PathBuf]) -> Result<()> {
        let mut hash_map = self.load_hashes().await?;

        for path in file_paths {
            if path.exists() {
                let path_str = path.display().to_string();
                let hash = Self::calculate_file_hash(path).await?;
                hash_map.hashes.insert(path_str, hash);
            }
        }

        hash_map.last_updated = chrono::Utc::now();
        self.save_hashes(&hash_map).await?;

        Ok(())
    }

    /// Clean up hashes for files that no longer exist
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
}
