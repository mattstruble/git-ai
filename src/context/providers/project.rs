use crate::context::{ContextData, ContextProvider, ContextType, ProjectContext};
use anyhow::{Context, Result};
use ignore::WalkBuilder;
use std::collections::HashMap;
use std::path::PathBuf;
use std::process::Command;

/// Project context provider
pub struct ProjectContextProvider;

impl ProjectContextProvider {
    pub fn new() -> Self {
        Self
    }

    /// Build directory tree structure as a formatted string
    async fn build_directory_tree(&self) -> Result<String> {
        self.get_directory_tree(".").await
    }

    /// Get directory tree structure as a formatted string with depth limit of 3
    async fn get_directory_tree(&self, root: &str) -> Result<String> {
        // Build tree structure with depth tracking
        let mut tree_map: HashMap<String, Vec<String>> = HashMap::new();
        let max_depth = 3;

        // Use ignore crate to respect .gitignore and exclude .git directory
        let walker = WalkBuilder::new(root)
            .hidden(false) // Include hidden files/dirs
            .ignore(true) // Respect .ignore files
            .git_ignore(true) // Respect .gitignore
            .git_exclude(true) // Respect .git/info/exclude
            .git_global(false) // Don't use global git config
            .max_depth(Some(max_depth))
            .build();

        for result in walker {
            match result {
                Ok(entry) => {
                    let path = entry.path();

                    // Skip the root directory itself
                    if path.to_string_lossy() == root {
                        continue;
                    }

                    // Explicitly skip .git directory and its contents
                    if path.file_name().and_then(|n| n.to_str()) == Some(".git") {
                        continue;
                    }

                    // Skip any path that contains .git directory
                    if path.components().any(|c| c.as_os_str() == ".git") {
                        continue;
                    }

                    let relative_path = path
                        .strip_prefix(root)
                        .unwrap_or(path)
                        .to_string_lossy()
                        .to_string();

                    // Calculate depth
                    let depth = relative_path.matches('/').count();
                    if depth >= max_depth {
                        continue;
                    }

                    // Get parent directory
                    let parent = if let Some(parent_path) = path.parent() {
                        parent_path
                            .strip_prefix(root)
                            .unwrap_or(parent_path)
                            .to_string_lossy()
                            .to_string()
                    } else {
                        String::new()
                    };

                    let name = path
                        .file_name()
                        .unwrap_or_default()
                        .to_string_lossy()
                        .to_string();

                    // Add trailing slash for directories
                    let display_name = if entry.file_type().map(|ft| ft.is_dir()).unwrap_or(false) {
                        format!("{}/", name)
                    } else {
                        name
                    };

                    tree_map
                        .entry(parent)
                        .or_insert_with(Vec::new)
                        .push(display_name);
                }
                Err(_) => {
                    // Skip entries we can't read
                    continue;
                }
            }
        }

        // Generate tree string
        self.build_tree_string(&tree_map, "", 0, max_depth)
    }

    /// Build a tree-like string representation
    fn build_tree_string(
        &self,
        tree_map: &HashMap<String, Vec<String>>,
        current_path: &str,
        depth: usize,
        max_depth: usize,
    ) -> Result<String> {
        let mut result = String::new();

        if depth >= max_depth {
            return Ok(result);
        }

        if let Some(children) = tree_map.get(current_path) {
            let mut sorted_children = children.clone();
            sorted_children.sort();

            for (index, child) in sorted_children.iter().enumerate() {
                let is_last = index == sorted_children.len() - 1;
                let prefix = if depth == 0 {
                    (if is_last { "└── " } else { "├── " }).to_string()
                } else {
                    let mut prefix = String::new();
                    for _ in 0..depth {
                        prefix.push_str("│   ");
                    }
                    prefix.push_str(if is_last { "└── " } else { "├── " });
                    prefix
                };

                result.push_str(&format!("{}{}\n", prefix, child));

                // Recursively build subtree for directories
                if child.ends_with('/') {
                    let child_name = child.trim_end_matches('/');
                    let child_path = if current_path.is_empty() {
                        child_name.to_string()
                    } else {
                        format!("{}/{}", current_path, child_name)
                    };

                    let subtree =
                        self.build_tree_string(tree_map, &child_path, depth + 1, max_depth)?;
                    result.push_str(&subtree);
                }
            }
        }

        Ok(result)
    }

    fn count_files_by_extension(&self, root: &str) -> Result<HashMap<String, u32>> {
        let mut counts = HashMap::new();

        // Walk through files to count extensions
        let walker = WalkBuilder::new(root)
            .hidden(false)
            .ignore(true)
            .git_ignore(true)
            .git_exclude(true)
            .git_global(false)
            .max_depth(Some(10)) // Reasonable depth limit
            .build();

        for result in walker {
            if let Ok(entry) = result {
                let path = entry.path();

                // Skip .git directory
                if path.components().any(|c| c.as_os_str() == ".git") {
                    continue;
                }

                if entry.file_type().map(|ft| ft.is_file()).unwrap_or(false) {
                    if let Some(extension) = path.extension().and_then(|e| e.to_str()) {
                        *counts.entry(extension.to_lowercase()).or_insert(0) += 1;
                    }
                }
            }
        }

        Ok(counts)
    }

    /// Get recently changed files using git log
    async fn get_recently_changed_files(&self) -> Result<Vec<String>> {
        let output = Command::new("git")
            .args([
                "log",
                "--pretty=format:",
                "--name-only",
                "--since=7 days ago",
                "-100", // Limit to 100 commits
            ])
            .output()
            .context("Failed to execute git log")?;

        if !output.status.success() {
            return Ok(Vec::new());
        }

        let files: Vec<String> = String::from_utf8(output.stdout)?
            .lines()
            .filter(|line| !line.is_empty())
            .map(|line| line.to_string())
            .collect();

        // Remove duplicates and take top 20
        let mut unique_files: Vec<String> = files
            .into_iter()
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect();
        unique_files.sort();
        unique_files.truncate(20);

        Ok(unique_files)
    }

    fn calculate_total_size(&self, root: &str) -> Result<u64> {
        let mut total_size = 0u64;

        let walker = WalkBuilder::new(root)
            .hidden(false)
            .ignore(true)
            .git_ignore(true)
            .git_exclude(true)
            .git_global(false)
            .max_depth(Some(10))
            .build();

        for result in walker {
            if let Ok(entry) = result {
                let path = entry.path();

                // Skip .git directory
                if path.components().any(|c| c.as_os_str() == ".git") {
                    continue;
                }

                if entry.file_type().map(|ft| ft.is_file()).unwrap_or(false) {
                    if let Ok(metadata) = entry.metadata() {
                        total_size += metadata.len();
                    }
                }
            }
        }

        Ok(total_size)
    }

    fn count_total_files(&self, root: &str) -> Result<u32> {
        let mut count = 0u32;

        let walker = WalkBuilder::new(root)
            .hidden(false)
            .ignore(true)
            .git_ignore(true)
            .git_exclude(true)
            .git_global(false)
            .max_depth(Some(10))
            .build();

        for result in walker {
            if let Ok(entry) = result {
                let path = entry.path();

                // Skip .git directory
                if path.components().any(|c| c.as_os_str() == ".git") {
                    continue;
                }

                if entry.file_type().map(|ft| ft.is_file()).unwrap_or(false) {
                    count += 1;
                }
            }
        }

        Ok(count)
    }

    /// Get contents of dependency and configuration files
    async fn get_dependency_file_contents(&self) -> Result<HashMap<String, String>> {
        let mut file_contents = HashMap::new();

        // Get the same files we track for dependencies, but now read their contents
        let dependency_files = self.get_file_dependencies();

        for file_path in dependency_files {
            if let Ok(content) = tokio::fs::read_to_string(&file_path).await {
                // Limit file size to prevent huge contexts (max 50KB per file)
                if content.len() <= 50_000 {
                    let file_name = file_path
                        .file_name()
                        .unwrap_or_default()
                        .to_string_lossy()
                        .to_string();
                    file_contents.insert(file_name, content);
                }
            }
        }

        Ok(file_contents)
    }
}

#[async_trait::async_trait]
impl ContextProvider for ProjectContextProvider {
    async fn gather(&self) -> Result<ContextData> {
        let directory_tree = self.build_directory_tree().await?;
        let dependency_files = self.get_dependency_file_contents().await?;
        let file_counts = self.count_files_by_extension(".")?;
        let recently_changed_files = self.get_recently_changed_files().await.unwrap_or_default();
        let total_size = self.calculate_total_size(".")?;
        let total_files = self.count_total_files(".")?;

        let project_context = ProjectContext {
            directory_tree,
            dependency_files,
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

    fn get_file_dependencies(&self) -> Vec<PathBuf> {
        // Project context depends on key project files and directory structure
        let mut files = Vec::new();

        // Focus only on files that directly affect code dependencies and compilation
        let exact_files = [
            // Rust
            "Cargo.toml",
            "Cargo.lock",
            // Node.js/JavaScript/TypeScript
            "package.json",
            "package-lock.json",
            "yarn.lock",
            "pnpm-lock.yaml",
            "tsconfig.json",
            "jsconfig.json",
            // Python
            "pyproject.toml",
            "setup.py",
            "setup.cfg",
            "requirements.txt",
            "requirements-dev.txt",
            "Pipfile",
            "Pipfile.lock",
            "poetry.lock",
            // Go
            "go.mod",
            "go.sum",
            "go.work",
            "go.work.sum",
            // Java/Kotlin/Scala
            "pom.xml",
            "build.gradle",
            "build.gradle.kts",
            "settings.gradle",
            "gradle.properties",
            "build.sbt",
            // C#/.NET
            "Directory.Build.props",
            "Directory.Build.targets",
            "nuget.config",
            "global.json",
            // C/C++
            "CMakeLists.txt",
            "conanfile.txt",
            "conanfile.py",
            "vcpkg.json",
            "meson.build",
            "xmake.lua",
            // Ruby
            "Gemfile",
            "Gemfile.lock",
            // PHP
            "composer.json",
            "composer.lock",
            // Swift
            "Package.swift",
            "Podfile",
            "Podfile.lock",
            // Dart/Flutter
            "pubspec.yaml",
            "pubspec.lock",
            // Elixir
            "mix.exs",
            "mix.lock",
            // Erlang
            "rebar.config",
            "rebar.lock",
            // Haskell
            "cabal.project",
            "stack.yaml",
            "package.yaml",
            // OCaml
            "dune-project",
            "opam",
            // Clojure
            "project.clj",
            "deps.edn",
            // R
            "DESCRIPTION",
            "NAMESPACE",
            "renv.lock",
            // Julia
            "Project.toml",
            "Manifest.toml",
            // Zig
            "build.zig",
            "build.zig.zon",
            // Nix
            "flake.nix",
            "flake.lock",
            "default.nix",
            "shell.nix",
            // Core build files
            "Makefile",
            "makefile",
            "justfile",
        ];

        // Add exact file matches
        for file in &exact_files {
            let path = PathBuf::from(file);
            if path.exists() {
                files.push(path);
            }
        }

        // Use glob patterns for files that commonly have variable names and affect dependencies/compilation
        self.add_glob_matches(&mut files, "*.sln"); // .NET solutions
        self.add_glob_matches(&mut files, "*.csproj"); // C# projects
        self.add_glob_matches(&mut files, "*.fsproj"); // F# projects
        self.add_glob_matches(&mut files, "*.vbproj"); // VB.NET projects
        self.add_glob_matches(&mut files, "*.gemspec"); // Ruby gems
        self.add_glob_matches(&mut files, "*.opam"); // OCaml packages

        files
    }
}

impl ProjectContextProvider {
    /// Add files matching a glob pattern to the files list
    fn add_glob_matches(&self, files: &mut Vec<PathBuf>, pattern: &str) {
        if let Ok(entries) = std::fs::read_dir(".") {
            for entry in entries.flatten() {
                if let Some(name) = entry.file_name().to_str() {
                    if self.matches_glob(name, pattern) {
                        files.push(entry.path());
                    }
                }
            }
        }
    }

    /// Simple glob matching for basic patterns like *.ext
    fn matches_glob(&self, name: &str, pattern: &str) -> bool {
        if pattern.starts_with("*.") {
            let ext = &pattern[2..];
            name.ends_with(&format!(".{}", ext))
        } else if pattern.ends_with(".*") {
            let prefix = &pattern[..pattern.len() - 2];
            name.starts_with(prefix)
        } else {
            name == pattern
        }
    }
}
