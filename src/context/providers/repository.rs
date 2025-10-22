use crate::context::{ContextData, ContextProvider, ContextType, RepositoryContext};
use crate::config::Config;
use anyhow::{Context, Result};
use ignore::WalkBuilder;
use std::collections::HashMap;
use std::path::PathBuf;
use std::process::Command;

pub struct RepositoryContextProvider;

impl RepositoryContextProvider {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl ContextProvider for RepositoryContextProvider {
    async fn gather_context(&self, _repo_path: &Path) -> Result<ContextData> {
        // This is a hardcoded context based on the git-ai project documentation
        // In a real implementation, this would dynamically parse repository docs
        let context = RepositoryContext {
            project: json!({
                "name": "git-ai",
                "description": "AI-assisted git workflow with cursor-agent integration",
                "primary_language": "Rust",
                "keywords": ["git", "ai", "cli", "cursor-agent", "workflow", "automation"],
                "usage": {
                    "commands": [
                        "git ai commit",
                        "git ai pr",
                        "git ai merge <branch>",
                        "git ai init",
                        "git ai ignore add <language>",
                        "git ai config"
                    ],
                    "examples": [
                        "git ai commit -m \"Focus on security improvements\"",
                        "git ai pr --dry-run",
                        "git ai init --language python --name myproject"
                    ],
                    "configuration": [
                        ".git-ai.yaml",
                        "~/.config/git-ai/config.yaml",
                        "--dry-run",
                        "--verbose",
                        "--no-confirm"
                    ]
                },
                "development": {
                    "setup_steps": [
                        "cargo build --release",
                        "cp target/release/git-ai ~/.local/bin/",
                        "nix profile install github:mattstruble/git-ai"
                    ],
                    "dependencies": ["clap", "async-trait", "reqwest", "serde", "cursor-agent"],
                    "testing": ["cargo test", "nix build"]
                },
                "breaking_changes": {
                    "rules": [
                        "Adheres to Semantic Versioning (semver.org)",
                        "Breaking changes documented in CHANGELOG.md",
                        "BREAKING prefix used for incompatible changes"
                    ],
                    "indicators": [
                        "BREAKING:",
                        "Major version increment",
                        "Changed flag behavior",
                        "API incompatibility"
                    ]
                },
                "conventions": {
                    "commit_style": "Conventional commits with cocogitto automation, format: type(scope): description",
                    "branching": "feat/* for features, origin/feat/context example in docs",
                    "release_process": "Cocogitto hooks, automated changelog generation, GitHub Actions release workflow",
                    "docs_reference": "README.md, CHANGELOG.md, LICENSE"
                },
                "misc": {
                    "notable_designs": [
                        "Command-based architecture",
                        "Git plugin integration",
                        "Cursor-agent dependency for AI",
                        "Context providers system"
                    ],
                    "important_files": [
                        "src/commands/*.rs",
                        "src/cursor_agent.rs",
                        "src/context/providers/*.rs",
                        "config/default_config.yaml"
                    ],
                    "security_notes": [
                        "User confirmation required for cursor-agent installation",
                        "Script validation and checksum verification",
                        "30 second timeout for HTTP requests"
                    ]
                }
            }),
        };

        Ok(ContextData::Repository(context))
    }

    fn name(&self) -> &str {
        "repository"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[tokio::test]
    async fn test_repository_context_provider() {
        let provider = RepositoryContextProvider::new();
        let path = PathBuf::from("/tmp");
        let context = provider.gather_context(&path).await.unwrap();

        match context {
            ContextData::Repository(repo_context) => {
                assert_eq!(repo_context.project["name"], "git-ai");
                assert_eq!(repo_context.project["primary_language"], "Rust");
            }
            _ => panic!("Expected RepositoryContext"),
        }
    }
}
