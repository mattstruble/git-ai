use crate::context::{ContextData, ContextType};
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::process::Command;
use tokio::fs;

/// Cache metadata for invalidation
#[derive(Debug, Clone, Serialize, Deserialize)]
struct CacheEntry {
    data: ContextData,
    git_commit_hash: Option<String>,
    working_tree_hash: Option<String>,
    created_at: chrono::DateTime<chrono::Utc>,
    expires_at: Option<chrono::DateTime<chrono::Utc>>,
}

/// Context cache manager with git hash-based invalidation
pub struct ContextCache {
    cache_dir: PathBuf,
}

impl ContextCache {
    /// Create a new context cache
    pub fn new() -> Result<Self> {
        let cache_dir = Self::get_cache_dir()?;

        // Ensure cache directory exists
        if !cache_dir.exists() {
            std::fs::create_dir_all(&cache_dir).with_context(|| {
                format!("Failed to create cache directory: {}", cache_dir.display())
            })?;
        }

        Ok(Self { cache_dir })
    }

    /// Get cached context data if valid
    pub async fn get(&self, context_type: ContextType) -> Result<Option<ContextData>> {
        let cache_file = self.cache_file_path(context_type);

        if !cache_file.exists() {
            return Ok(None);
        }

        // Read and deserialize cache entry
        let content = fs::read_to_string(&cache_file)
            .await
            .with_context(|| format!("Failed to read cache file: {}", cache_file.display()))?;

        let entry: CacheEntry = serde_json::from_str(&content)
            .with_context(|| format!("Failed to parse cache file: {}", cache_file.display()))?;

        // Check if cache entry is still valid
        if self.is_cache_valid(&entry).await? {
            Ok(Some(entry.data))
        } else {
            // Cache is invalid, remove the file
            let _ = fs::remove_file(&cache_file).await;
            Ok(None)
        }
    }

    /// Store context data in cache
    pub async fn store(&self, context_type: ContextType, data: &ContextData) -> Result<()> {
        let cache_file = self.cache_file_path(context_type);
        let git_hashes = self.get_git_hashes().await?;

        let entry = CacheEntry {
            data: data.clone(),
            git_commit_hash: git_hashes.0,
            working_tree_hash: git_hashes.1,
            created_at: chrono::Utc::now(),
            expires_at: self.get_expiry_time(context_type),
        };

        let content =
            serde_json::to_string_pretty(&entry).context("Failed to serialize cache entry")?;

        fs::write(&cache_file, content)
            .await
            .with_context(|| format!("Failed to write cache file: {}", cache_file.display()))?;

        Ok(())
    }

    /// Clear cache for a specific context type
    #[allow(dead_code)]
    pub async fn clear_type(&self, context_type: ContextType) -> Result<()> {
        let cache_file = self.cache_file_path(context_type);
        if cache_file.exists() {
            fs::remove_file(&cache_file).await.with_context(|| {
                format!("Failed to remove cache file: {}", cache_file.display())
            })?;
        }
        Ok(())
    }

    /// Clear all cached data
    #[allow(dead_code)]
    pub async fn clear_all(&self) -> Result<()> {
        if self.cache_dir.exists() {
            let mut dir = fs::read_dir(&self.cache_dir).await.with_context(|| {
                format!(
                    "Failed to read cache directory: {}",
                    self.cache_dir.display()
                )
            })?;

            while let Some(entry) = dir.next_entry().await? {
                if entry.file_type().await?.is_file() {
                    fs::remove_file(entry.path()).await?;
                }
            }
        }
        Ok(())
    }

    /// Get cache statistics
    #[allow(dead_code)]
    pub async fn get_stats(&self) -> Result<CacheStats> {
        let mut total_files = 0;
        let mut total_size = 0;
        let mut oldest_entry = None;
        let mut newest_entry = None;

        if self.cache_dir.exists() {
            let mut dir = fs::read_dir(&self.cache_dir).await?;

            while let Some(entry) = dir.next_entry().await? {
                if entry.file_type().await?.is_file() {
                    total_files += 1;
                    let metadata = entry.metadata().await?;
                    total_size += metadata.len();

                    if let Ok(modified) = metadata.modified() {
                        let modified_dt = chrono::DateTime::<chrono::Utc>::from(modified);

                        if oldest_entry.is_none() || modified_dt < oldest_entry.unwrap() {
                            oldest_entry = Some(modified_dt);
                        }
                        if newest_entry.is_none() || modified_dt > newest_entry.unwrap() {
                            newest_entry = Some(modified_dt);
                        }
                    }
                }
            }
        }

        Ok(CacheStats {
            total_files,
            total_size,
            oldest_entry,
            newest_entry,
        })
    }

    /// Get the cache directory path
    fn get_cache_dir() -> Result<PathBuf> {
        // First try to find .git directory
        let git_dir = Self::find_git_dir().unwrap_or_else(|| PathBuf::from(".git"));

        Ok(git_dir.join("git-ai").join("context-cache"))
    }

    /// Find the .git directory by walking up the directory tree
    fn find_git_dir() -> Option<PathBuf> {
        let mut current = std::env::current_dir().ok()?;

        loop {
            let git_dir = current.join(".git");
            if git_dir.exists() {
                return Some(git_dir);
            }

            if !current.pop() {
                break;
            }
        }

        None
    }

    /// Get cache file path for a context type
    fn cache_file_path(&self, context_type: ContextType) -> PathBuf {
        let filename = format!("{:?}.json", context_type).to_lowercase();
        self.cache_dir.join(filename)
    }

    /// Check if cache entry is still valid
    async fn is_cache_valid(&self, entry: &CacheEntry) -> Result<bool> {
        // Check expiry time
        if let Some(expires_at) = entry.expires_at {
            if chrono::Utc::now() > expires_at {
                return Ok(false);
            }
        }

        // Check git hashes
        let current_hashes = self.get_git_hashes().await?;

        // If commit hash changed, invalidate
        if entry.git_commit_hash != current_hashes.0 {
            return Ok(false);
        }

        // If working tree hash changed, invalidate
        if entry.working_tree_hash != current_hashes.1 {
            return Ok(false);
        }

        Ok(true)
    }

    /// Get current git commit hash and working tree hash
    async fn get_git_hashes(&self) -> Result<(Option<String>, Option<String>)> {
        // Get commit hash
        let commit_hash = Command::new("git")
            .args(["rev-parse", "HEAD"])
            .output()
            .ok()
            .and_then(|output| {
                if output.status.success() {
                    String::from_utf8(output.stdout)
                        .ok()
                        .map(|s| s.trim().to_string())
                } else {
                    None
                }
            });

        // Get working tree hash (based on index and working directory state)
        let working_tree_hash = Command::new("git")
            .args(["status", "--porcelain=v1"])
            .output()
            .ok()
            .and_then(|output| {
                if output.status.success() {
                    let status_output = String::from_utf8(output.stdout).ok()?;
                    // Create a simple hash of the status output
                    Some(format!("{:x}", md5::compute(status_output.as_bytes())))
                } else {
                    None
                }
            });

        Ok((commit_hash, working_tree_hash))
    }

    /// Get expiry time for different context types
    fn get_expiry_time(&self, context_type: ContextType) -> Option<chrono::DateTime<chrono::Utc>> {
        let duration = match context_type {
            ContextType::Git => chrono::Duration::minutes(5), // Git context expires quickly
            ContextType::Project => chrono::Duration::hours(1), // Project structure is more stable
            ContextType::Agent => chrono::Duration::hours(24), // Agent config rarely changes
            ContextType::Interaction => return None, // Interaction context doesn't expire (always fresh)
        };

        Some(chrono::Utc::now() + duration)
    }
}

/// Cache statistics
#[derive(Debug)]
#[allow(dead_code)]
pub struct CacheStats {
    pub total_files: u32,
    pub total_size: u64,
    pub oldest_entry: Option<chrono::DateTime<chrono::Utc>>,
    pub newest_entry: Option<chrono::DateTime<chrono::Utc>>,
}
