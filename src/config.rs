use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Config {
    pub behavior: BehaviorConfig,
    pub commands: CommandConfigs,
    #[serde(default)]
    pub project: Option<ProjectConfig>,
    #[serde(default)]
    pub repository: RepositoryConfig,
}

impl Default for Config {
    fn default() -> Self {
        load_default_config()
    }
}

/// Project-specific configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ProjectConfig {
    // Project configuration can be extended here as needed
}

/// Repository-specific configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RepositoryConfig {
    #[serde(default)]
    pub dependency_files: DependencyFilesConfig,
}

/// Configuration for project dependency file patterns
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DependencyFilesConfig {
    pub package_managers: Option<Vec<String>>,
    pub build_files: Option<Vec<String>>,
    pub config_files: Option<Vec<String>>,
    pub additional_patterns: Option<Vec<String>>,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct BehaviorConfig {
    pub verbose: bool,
}

/// Configuration for individual commands
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
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
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct CommitConfig {
    pub prompt: Option<String>,
    pub no_confirm: Option<bool>,
    pub context: Option<Vec<String>>,
}

/// Configuration for PR command
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct PrConfig {
    pub prompt: Option<String>,
    pub no_confirm: Option<bool>,
    pub context: Option<Vec<String>>,
}

/// Configuration for merge command
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct MergeConfig {
    pub prompt: Option<String>,
    pub no_confirm: Option<bool>,
    pub context: Option<Vec<String>>,
}

/// Configuration for init command
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct InitConfig {
    pub prompt: Option<String>,
    pub no_confirm: Option<bool>,
    pub context: Option<Vec<String>>,
}

/// Configuration for ignore command
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct IgnoreConfig {
    pub prompt: Option<String>,
    pub no_confirm: Option<bool>,
    pub context: Option<Vec<String>>,
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
        // Start with default config and add sample customizations
        let mut sample = load_default_config();

        // Add sample custom configurations
        sample.behavior.verbose = true;
        sample.commands.commit.prompt =
            Some("Custom commit prompt (optional - overrides built-in prompt)".to_string());
        sample.commands.pr.prompt =
            Some("Custom PR prompt (optional - overrides built-in prompt)".to_string());
        sample.commands.merge.prompt =
            Some("Custom merge prompt (optional - overrides built-in prompt)".to_string());
        sample.commands.init.prompt =
            Some("Custom init prompt (optional - overrides built-in prompt)".to_string());
        sample.commands.ignore.prompt =
            Some("Custom ignore prompt (optional - overrides built-in prompt)".to_string());

        // Add sample repository customizations
        sample.repository.dependency_files.additional_patterns = Some(vec![
            "custom-package.json".to_string(),
            "custom-build.sh".to_string(),
            "custom-config.toml".to_string(),
            "*.custom".to_string(),
        ]);

        serde_yaml::to_string(&sample).context("Failed to serialize sample configuration")
    }
}

#[allow(dead_code)]
/// Default project dependency file patterns embedded in binary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DefaultPatterns {
    pub package_managers: Vec<String>,
    pub build_files: Vec<String>,
    pub config_files: Vec<String>,
}

impl Config {
    #[allow(dead_code)]
    /// Get dependency file patterns from repository config
    pub fn get_dependency_patterns(&self) -> Vec<String> {
        let mut patterns = Vec::new();
        let dep_config = &self.repository.dependency_files;

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

    /// Convert context string names to ContextType enums
    pub fn parse_context_types(context_names: &[String]) -> Vec<crate::context::ContextType> {
        context_names
            .iter()
            .filter_map(|name| match name.as_str() {
                "Git" => Some(crate::context::ContextType::Git),
                "Repository" => Some(crate::context::ContextType::Repository),
                "Project" => Some(crate::context::ContextType::Project),
                "Agent" => Some(crate::context::ContextType::Agent),
                "Interaction" => Some(crate::context::ContextType::Interaction),
                _ => {
                    eprintln!("Warning: Unknown context type '{}' - ignoring", name);
                    None
                }
            })
            .collect()
    }
}

/// Load the complete default configuration from embedded YAML
pub fn load_default_config() -> Config {
    // Embed the default configuration at compile time
    const DEFAULT_CONFIG: &str = include_str!("../config/default_config.yaml");

    serde_yaml::from_str(DEFAULT_CONFIG).expect("Failed to parse embedded default configuration")
}

#[allow(dead_code)]
/// Get the default dependency file patterns (used internally)
pub fn get_default_patterns() -> DefaultPatterns {
    let config = load_default_config();
    let dep_config = &config.repository.dependency_files;

    DefaultPatterns {
        package_managers: dep_config.package_managers.clone().unwrap_or_default(),
        build_files: dep_config.build_files.clone().unwrap_or_default(),
        config_files: dep_config.config_files.clone().unwrap_or_default(),
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
