use crate::config::RepositoryConfig;
use crate::context::{ContextData, ContextProvider, ContextType, RepositoryContext};
use anyhow::Result;
use ignore::WalkBuilder;
use std::collections::HashMap;
use std::path::PathBuf;
use std::process::Command;

/// Repository structure context provider that analyzes file system structure
pub struct RepositoryContextProvider {
    config: RepositoryConfig,
}

impl RepositoryContextProvider {
    pub fn new(config: RepositoryConfig) -> Self {
        Self { config }
    }

    /// Build directory tree structure (limited to depth 3)
    async fn build_directory_tree(&self) -> Result<String> {
        let output = Command::new("tree")
            .args(["-L", "3", "-a", "-I", ".git"])
            .output();

        match output {
            Ok(output) if output.status.success() => {
                Ok(String::from_utf8_lossy(&output.stdout).to_string())
            }
            _ => {
                // Fallback to manual tree building if tree command not available
                self.build_manual_tree().await
            }
        }
    }

    /// Fallback manual tree building
    async fn build_manual_tree(&self) -> Result<String> {
        let mut tree = String::new();
        let walker = WalkBuilder::new(".")
            .max_depth(Some(3))
            .hidden(false)
            .add_custom_ignore_filename(".gitignore")
            .build();

        let mut paths: Vec<PathBuf> = Vec::new();
        for entry in walker.flatten() {
            let path = entry.path();
            if path.starts_with("./.git") {
                continue;
            }
            paths.push(path.to_path_buf());
        }

        paths.sort();

        tree.push_str(".\n");
        for path in paths.iter().take(50) {
            // Limit output
            let depth = path.components().count().saturating_sub(1);
            let indent = "  ".repeat(depth);

            if path.is_dir() {
                tree.push_str(&format!(
                    "{}├── {}/\n",
                    indent,
                    path.file_name().unwrap_or_default().to_string_lossy()
                ));
            } else {
                tree.push_str(&format!(
                    "{}├── {}\n",
                    indent,
                    path.file_name().unwrap_or_default().to_string_lossy()
                ));
            }
        }

        if paths.len() > 50 {
            tree.push_str(&format!(
                "... and {} more files/directories\n",
                paths.len() - 50
            ));
        }

        Ok(tree)
    }

    /// Get dependency files with their content
    async fn get_dependency_files(&self) -> Result<HashMap<String, String>> {
        let dependency_patterns = self.get_dependency_patterns();
        let mut dependency_files = HashMap::new();

        // Use WalkBuilder to find files matching patterns
        let walker = WalkBuilder::new(".")
            .hidden(false)
            .add_custom_ignore_filename(".gitignore")
            .build();

        for entry in walker.flatten() {
            let path = entry.path();

            // Skip .git directory
            if path.starts_with("./.git") {
                continue;
            }

            if path.is_file() {
                let file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");

                let path_str = path.to_string_lossy();

                // Check if file matches any of our dependency patterns
                let matches_pattern = dependency_patterns.iter().any(|pattern| {
                    // Simple pattern matching - can be enhanced later
                    if pattern.contains('*') {
                        // Handle wildcard patterns
                        let pattern_parts: Vec<&str> = pattern.split('*').collect();
                        if pattern_parts.len() == 2 {
                            path_str.starts_with(pattern_parts[0])
                                && path_str.ends_with(pattern_parts[1])
                        } else {
                            file_name == pattern || path_str.ends_with(pattern)
                        }
                    } else {
                        file_name == pattern || path_str.ends_with(pattern)
                    }
                });

                if matches_pattern {
                    if let Ok(content) = tokio::fs::read_to_string(&path).await {
                        // Limit file size to 50KB
                        if content.len() <= 50_000 {
                            dependency_files.insert(path.display().to_string(), content);
                        }
                    }
                }
            }
        }

        Ok(dependency_files)
    }

    /// Get dependency file patterns from config
    fn get_dependency_patterns(&self) -> Vec<String> {
        let mut patterns = Vec::new();
        let dep_config = &self.config.dependency_files;

        // Add patterns from each category if configured
        if let Some(package_managers) = &dep_config.package_managers {
            patterns.extend(package_managers.clone());
        }
        if let Some(build_files) = &dep_config.build_files {
            patterns.extend(build_files.clone());
        }
        if let Some(config_files) = &dep_config.config_files {
            patterns.extend(config_files.clone());
        }
        if let Some(additional) = &dep_config.additional_patterns {
            patterns.extend(additional.clone());
        }

        // Remove duplicates
        patterns.sort();
        patterns.dedup();

        patterns
    }

    /// Count files by extension
    async fn count_files_by_extension(&self) -> Result<HashMap<String, u32>> {
        let mut counts = HashMap::new();

        let walker = WalkBuilder::new(".")
            .hidden(false)
            .add_custom_ignore_filename(".gitignore")
            .build();

        for entry in walker.flatten() {
            let path = entry.path();

            // Skip .git directory
            if path.starts_with("./.git") {
                continue;
            }

            if path.is_file() {
                let extension = path
                    .extension()
                    .and_then(|ext| ext.to_str())
                    .unwrap_or("no_extension")
                    .to_string();

                *counts.entry(extension).or_insert(0) += 1;
            }
        }

        Ok(counts)
    }

    /// Get recently changed files (within last 7 days)
    async fn get_recently_changed_files(&self) -> Result<Vec<String>> {
        let output = Command::new("git")
            .args([
                "log",
                "--since=7.days.ago",
                "--name-only",
                "--pretty=format:",
            ])
            .output();

        match output {
            Ok(output) if output.status.success() => {
                let files: Vec<String> = String::from_utf8_lossy(&output.stdout)
                    .lines()
                    .filter(|line| !line.is_empty())
                    .map(|line| line.to_string())
                    .collect::<std::collections::HashSet<_>>() // Remove duplicates
                    .into_iter()
                    .take(20) // Limit to 20 files
                    .collect();

                Ok(files)
            }
            _ => Ok(vec![]), // Fallback to empty if git command fails
        }
    }

    /// Count total files and calculate total size
    async fn get_repository_stats(&self) -> Result<(u32, u64)> {
        let mut total_files = 0u32;
        let mut total_size = 0u64;

        let walker = WalkBuilder::new(".")
            .hidden(false)
            .add_custom_ignore_filename(".gitignore")
            .build();

        for entry in walker.flatten() {
            let path = entry.path();

            // Skip .git directory
            if path.starts_with("./.git") {
                continue;
            }

            if path.is_file() {
                total_files += 1;
                if let Ok(metadata) = path.metadata() {
                    total_size += metadata.len();
                }
            }
        }

        Ok((total_files, total_size))
    }
}

#[async_trait::async_trait]
impl ContextProvider for RepositoryContextProvider {
    async fn gather(&self) -> Result<ContextData> {
        let directory_tree = self
            .build_directory_tree()
            .await
            .unwrap_or_else(|_| "Unable to generate directory tree".to_string());
        let dependency_files = self.get_dependency_files().await.unwrap_or_default();
        let file_counts = self.count_files_by_extension().await.unwrap_or_default();
        let recently_changed_files = self.get_recently_changed_files().await.unwrap_or_default();
        let (total_files, total_size) = self.get_repository_stats().await.unwrap_or((0, 0));

        let context = RepositoryContext {
            directory_tree,
            dependency_files,
            file_counts,
            recently_changed_files,
            total_files,
            total_size,
        };

        Ok(ContextData::Repository(context))
    }

    fn context_type(&self) -> ContextType {
        ContextType::Repository
    }

    async fn should_refresh(&self, _cached_data: &ContextData) -> Result<bool> {
        // Repository structure context should refresh when files are added/removed/modified
        // For now, we'll rely on git-based invalidation and TTL
        Ok(false)
    }

    fn get_file_dependencies(&self) -> Vec<PathBuf> {
        // Repository context depends on the overall file structure
        // For now, return empty - it will use git hash-based invalidation
        vec![]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;

    #[tokio::test]
    async fn test_repository_context_provider() {
        let config = Config::default();
        let provider = RepositoryContextProvider::new(config.repository);
        let context = provider.gather().await.unwrap();

        match context {
            ContextData::Repository(repo_context) => {
                assert!(!repo_context.directory_tree.is_empty());
                assert!(repo_context.total_files > 0);
            }
            _ => panic!("Expected RepositoryContext"),
        }
    }

    #[tokio::test]
    async fn test_file_counting() {
        let config = Config::default();
        let provider = RepositoryContextProvider::new(config.repository);
        let counts = provider.count_files_by_extension().await.unwrap();

        // Should have some Rust files in this project
        assert!(counts.get("rs").unwrap_or(&0) > &0);
    }
}
