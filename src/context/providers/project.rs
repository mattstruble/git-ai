use crate::context::{
    ContextData, ContextProvider, ContextType, DirectoryTree, EntryType, ProjectContext, TreeEntry,
};
use anyhow::{Context, Result};
use ignore::WalkBuilder;
use std::collections::HashMap;
use std::path::Path;
use std::process::Command;

/// Project context provider
pub struct ProjectContextProvider;

impl ProjectContextProvider {
    pub fn new() -> Self {
        Self
    }

    /// Build directory tree respecting gitignore
    async fn build_directory_tree(&self) -> Result<DirectoryTree> {
        let root = std::env::current_dir()
            .context("Failed to get current directory")?
            .to_string_lossy()
            .to_string();

        let mut entries = Vec::new();

        // Use ignore crate to respect .gitignore
        let walker = WalkBuilder::new(&root)
            .hidden(false) // Include hidden files/dirs
            .ignore(true)  // Respect .ignore files
            .git_ignore(true) // Respect .gitignore
            .git_exclude(true) // Respect .git/info/exclude
            .build();

        for result in walker {
            match result {
                Ok(entry) => {
                    let path = entry.path();

                    // Skip the root directory itself
                    if path.to_string_lossy() == root {
                        continue;
                    }

                    let relative_path = path
                        .strip_prefix(&root)
                        .unwrap_or(path)
                        .to_string_lossy()
                        .to_string();

                    let name = path
                        .file_name()
                        .unwrap_or_default()
                        .to_string_lossy()
                        .to_string();

                    let metadata = entry.metadata().ok();
                    let size = metadata.as_ref().and_then(|m| {
                        if m.is_file() {
                            Some(m.len())
                        } else {
                            None
                        }
                    });

                    let modified = metadata.and_then(|m| {
                        m.modified().ok().map(|t| chrono::DateTime::<chrono::Utc>::from(t))
                    });

                    let entry_type = if entry.file_type().map(|ft| ft.is_dir()).unwrap_or(false) {
                        EntryType::Directory
                    } else {
                        EntryType::File
                    };

                    entries.push(TreeEntry {
                        path: relative_path,
                        name,
                        entry_type,
                        size,
                        modified,
                    });
                }
                Err(_) => continue, // Skip entries we can't read
            }
        }

        // Sort entries for consistent output
        entries.sort_by(|a, b| a.path.cmp(&b.path));

        Ok(DirectoryTree { root, entries })
    }

    /// Count files by extension
    fn count_files_by_extension(&self, tree: &DirectoryTree) -> HashMap<String, u32> {
        let mut counts = HashMap::new();

        for entry in &tree.entries {
            if matches!(entry.entry_type, EntryType::File) {
                let extension = Path::new(&entry.path)
                    .extension()
                    .and_then(|ext| ext.to_str())
                    .unwrap_or("no_extension")
                    .to_lowercase();

                *counts.entry(extension).or_insert(0) += 1;
            }
        }

        counts
    }

    /// Get recently changed files from git
    async fn get_recently_changed_files(&self) -> Result<Vec<String>> {
        let output = Command::new("git")
            .args([
                "diff",
                "--name-only",
                "HEAD~10", // Last 10 commits
            ])
            .output()
            .context("Failed to execute git diff for recently changed files")?;

        if !output.status.success() {
            // If git diff fails, try a different approach
            let output = Command::new("git")
                .args([
                    "log",
                    "--name-only",
                    "--pretty=format:",
                    "-10", // Last 10 commits
                ])
                .output()
                .context("Failed to execute git log for recently changed files")?;

            if !output.status.success() {
                return Ok(Vec::new()); // Return empty if git commands fail
            }
        }

        let output_str = String::from_utf8(output.stdout).unwrap_or_default();
        let mut files: Vec<String> = output_str
            .lines()
            .filter(|line| !line.is_empty())
            .map(|line| line.trim().to_string())
            .collect();

        // Remove duplicates and sort
        files.sort();
        files.dedup();

        // Limit to reasonable number
        files.truncate(50);

        Ok(files)
    }

    /// Calculate total size of tracked files
    fn calculate_total_size(&self, tree: &DirectoryTree) -> u64 {
        tree.entries
            .iter()
            .filter_map(|entry| entry.size)
            .sum()
    }

    /// Count total files
    fn count_total_files(&self, tree: &DirectoryTree) -> u32 {
        tree.entries
            .iter()
            .filter(|entry| matches!(entry.entry_type, EntryType::File))
            .count() as u32
    }
}

#[async_trait::async_trait]
impl ContextProvider for ProjectContextProvider {
    async fn gather(&self) -> Result<ContextData> {
        let directory_tree = self.build_directory_tree().await?;
        let file_counts = self.count_files_by_extension(&directory_tree);
        let recently_changed_files = self.get_recently_changed_files().await.unwrap_or_default();
        let total_size = self.calculate_total_size(&directory_tree);
        let total_files = self.count_total_files(&directory_tree);

        let project_context = ProjectContext {
            directory_tree,
            file_counts,
            recently_changed_files,
            total_files,
            total_size,
        };

        Ok(ContextData::Project(project_context))
    }

    fn context_type(&self) -> ContextType {
        ContextType::Project
    }

    async fn should_refresh(&self, cached_data: &ContextData) -> Result<bool> {
        // Project context can be cached longer since directory structure
        // changes less frequently than git state
        if let ContextData::Project(_) = cached_data {
            // Let the cache system handle expiry based on time
            Ok(false)
        } else {
            Ok(true)
        }
    }
}
