use crate::prompts::{PromptConfig, PromptRegistry};
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Deserialize, Serialize, Default)]
pub struct Config {
    #[serde(default)]
    pub prompts: PromptConfig,

    #[serde(default)]
    pub behavior: BehaviorConfig,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct BehaviorConfig {
    #[serde(default = "default_confirm_install")]
    pub confirm_cursor_agent_install: bool,

    #[serde(default = "default_verbose")]
    pub verbose: bool,
}

impl Default for BehaviorConfig {
    fn default() -> Self {
        Self {
            confirm_cursor_agent_install: default_confirm_install(),
            verbose: default_verbose(),
        }
    }
}

fn default_confirm_install() -> bool {
    true
}
fn default_verbose() -> bool {
    false
}

impl Config {
    /// Load configuration from the standard config paths
    pub fn load() -> Result<Self> {
        // Try loading in this order:
        // 1. .git-ai.yaml in current directory (repo-specific)
        // 2. ~/.config/git-ai/config.yaml (user-specific)
        // 3. Default configuration

        if let Ok(config) = Self::load_from_path(&PathBuf::from(".git-ai.yaml")) {
            return Ok(config);
        }

        if let Some(user_config_path) = Self::user_config_path() {
            if let Ok(config) = Self::load_from_path(&user_config_path) {
                return Ok(config);
            }
        }

        Ok(Self::default())
    }

    /// Load configuration from a specific path
    pub fn load_from_path(path: &PathBuf) -> Result<Self> {
        if !path.exists() {
            anyhow::bail!("Config file does not exist: {}", path.display());
        }

        let content = std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read config file: {}", path.display()))?;

        let config: Config = serde_yaml::from_str(&content)
            .with_context(|| format!("Failed to parse config file: {}", path.display()))?;

        Ok(config)
    }

    /// Get the user configuration path
    pub fn user_config_path() -> Option<PathBuf> {
        if let Some(config_dir) = dirs::config_dir() {
            let git_ai_config = config_dir.join("git-ai").join("config.yaml");
            Some(git_ai_config)
        } else {
            // Fallback to home directory
            dirs::home_dir()
                .map(|home_dir| home_dir.join(".config").join("git-ai").join("config.yaml"))
        }
    }

    /// Create a sample configuration file
    pub fn create_sample_config() -> Result<String> {
        let sample = Config {
            prompts: PromptConfig {
                commit: Some("Generate a concise commit message for all changes.".to_string()),
                pr: Some("Generate a comprehensive PR description.".to_string()),
                merge: Some(
                    "Generate a merge summary and conflict resolution guidance.".to_string(),
                ),
            },
            behavior: BehaviorConfig {
                confirm_cursor_agent_install: true,
                verbose: false,
            },
        };

        serde_yaml::to_string(&sample).context("Failed to serialize sample configuration")
    }

    /// Get the prompt registry with configuration overrides applied
    pub fn get_prompts(&self) -> PromptRegistry {
        let default_registry = PromptRegistry::default();
        default_registry.with_overrides(&self.prompts)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert!(config.behavior.confirm_cursor_agent_install);
        assert!(!config.behavior.verbose);
    }

    #[test]
    fn test_sample_config_generation() {
        let sample = Config::create_sample_config().unwrap();
        assert!(sample.contains("prompts:"));
        assert!(sample.contains("behavior:"));
        assert!(sample.contains("confirm_cursor_agent_install"));
    }

    #[test]
    fn test_config_loading_from_path() {
        let temp_dir = tempdir().unwrap();
        let config_path = temp_dir.path().join("test_config.yaml");

        let test_config = r#"
behavior:
  verbose: true

prompts:
  commit: "Custom commit prompt"
"#;

        fs::write(&config_path, test_config).unwrap();

        let config = Config::load_from_path(&config_path).unwrap();
        assert!(config.behavior.verbose);
        assert_eq!(
            config.prompts.commit.as_deref(),
            Some("Custom commit prompt")
        );
    }

    #[test]
    fn test_prompt_fallbacks() {
        let config = Config::default();
        let prompts = config.get_prompts();

        // Should use default prompts when none are configured
        assert!(prompts.commit.contains("commit"));
        assert!(prompts.pr.contains("pull request"));
        assert!(prompts.merge.contains("merge"));
    }

    #[test]
    fn test_prompt_overrides() {
        let mut config = Config::default();
        config.prompts.commit = Some("Custom commit prompt".to_string());

        let prompts = config.get_prompts();
        assert_eq!(prompts.commit, "Custom commit prompt");
        // Other prompts should use defaults
        assert!(prompts.pr.contains("pull request"));
        assert!(prompts.merge.contains("merge"));
    }
}
