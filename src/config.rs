use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Deserialize, Serialize, Default)]
pub struct Config {
    #[serde(default)]
    pub behavior: BehaviorConfig,

    #[serde(default)]
    pub commands: CommandConfigs,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct BehaviorConfig {
    #[serde(default = "default_verbose")]
    pub verbose: bool,
}

impl Default for BehaviorConfig {
    fn default() -> Self {
        Self {
            verbose: default_verbose(),
        }
    }
}

fn default_verbose() -> bool {
    false
}

/// Configuration for individual commands
#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct CommandConfigs {
    #[serde(default)]
    pub commit: CommitConfig,

    #[serde(default)]
    pub pr: PrConfig,

    #[serde(default)]
    pub merge: MergeConfig,

    #[serde(default)]
    pub init: InitConfig,

    #[serde(default)]
    pub ignore: IgnoreConfig,
}

/// Configuration for commit command
#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct CommitConfig {
    pub prompt: Option<String>,
    pub no_confirm: Option<bool>,
}

/// Configuration for PR command
#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct PrConfig {
    pub prompt: Option<String>,
    pub no_confirm: Option<bool>,
}

/// Configuration for merge command
#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct MergeConfig {
    pub prompt: Option<String>,
    pub no_confirm: Option<bool>,
}

/// Configuration for init command
#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct InitConfig {
    pub prompt: Option<String>,
    pub no_confirm: Option<bool>,
}

/// Configuration for ignore command
#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct IgnoreConfig {
    pub prompt: Option<String>,
    pub no_confirm: Option<bool>,
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
            behavior: BehaviorConfig { verbose: false },
            commands: CommandConfigs {
                commit: CommitConfig {
                    prompt: Some(
                        "Custom commit prompt (optional - overrides built-in prompt)".to_string(),
                    ),
                    no_confirm: Some(false),
                },
                pr: PrConfig {
                    prompt: Some(
                        "Custom PR prompt (optional - overrides built-in prompt)".to_string(),
                    ),
                    no_confirm: Some(false),
                },
                merge: MergeConfig {
                    prompt: Some(
                        "Custom merge prompt (optional - overrides built-in prompt)".to_string(),
                    ),
                    no_confirm: Some(false),
                },
                init: InitConfig {
                    prompt: Some(
                        "Custom init prompt (optional - overrides built-in prompt)".to_string(),
                    ),
                    no_confirm: Some(false),
                },
                ignore: IgnoreConfig {
                    prompt: Some(
                        "Custom ignore prompt (optional - overrides built-in prompt)".to_string(),
                    ),
                    no_confirm: Some(false),
                },
            },
        };

        serde_yaml::to_string(&sample).context("Failed to serialize sample configuration")
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
        assert!(!config.behavior.verbose);
    }

    #[test]
    fn test_sample_config_generation() {
        let sample = Config::create_sample_config().unwrap();
        assert!(sample.contains("commands:"));
        assert!(sample.contains("behavior:"));
        assert!(sample.contains("commands:"));
        assert!(sample.contains("verbose"));
    }

    #[test]
    fn test_config_loading_from_path() {
        let temp_dir = tempdir().unwrap();
        let config_path = temp_dir.path().join("test_config.yaml");

        let test_config = r#"
behavior:
  verbose: true

commands:
  commit:
    prompt: "Custom commit prompt"
    no_confirm: true
"#;

        fs::write(&config_path, test_config).unwrap();

        let config = Config::load_from_path(&config_path).unwrap();
        assert!(config.behavior.verbose);
        assert_eq!(config.commands.commit.no_confirm, Some(true));
        assert_eq!(
            config.commands.commit.prompt.as_deref(),
            Some("Custom commit prompt")
        );
    }

    #[test]
    fn test_prompt_fallbacks() {
        let config = Config::default();

        // Commands should have no custom prompts by default (use built-in prompts)
        assert!(config.commands.commit.prompt.is_none());
        assert!(config.commands.pr.prompt.is_none());
        assert!(config.commands.merge.prompt.is_none());
    }

    #[test]
    fn test_prompt_overrides() {
        let mut config = Config::default();
        config.commands.commit.prompt = Some("Custom commit prompt".to_string());

        assert_eq!(
            config.commands.commit.prompt.as_deref(),
            Some("Custom commit prompt")
        );
        // Other command prompts should remain default (None)
        assert!(config.commands.pr.prompt.is_none());
        assert!(config.commands.merge.prompt.is_none());
    }
}
