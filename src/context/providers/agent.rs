use crate::context::{
    AgentConfigFile, AgentContext, ConfigFormat, ContextData, ContextProvider, ContextType,
};
use anyhow::Result;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

/// Agent context provider for .cursoragent and related config files
pub struct AgentContextProvider;

impl AgentContextProvider {
    pub fn new() -> Self {
        Self
    }

    /// Find all cursor-agent configuration files
    async fn find_config_files(&self) -> Result<Vec<AgentConfigFile>> {
        let mut config_files = Vec::new();
        let current_dir = std::env::current_dir()?;

        // Look for various cursor-agent config file patterns
        let config_patterns = [
            ".cursoragent",
            ".cursor-agent",
            ".aiconfig",
            ".cursor/agent.json",
            ".cursor/agent.yaml",
            ".cursor/agent.yml",
            ".cursor/config.json",
            "cursor-agent.json",
            "cursor-agent.yaml",
            "cursor-agent.yml",
            ".cursorrules",
        ];

        // Search in current directory and up the tree
        let mut search_dir = current_dir.clone();
        loop {
            for pattern in &config_patterns {
                let config_path = search_dir.join(pattern);
                if config_path.exists() && config_path.is_file() {
                    if let Ok(config_file) = self.load_config_file(&config_path).await {
                        config_files.push(config_file);
                    }
                }
            }

            // Also check .cursor directory if it exists
            let cursor_dir = search_dir.join(".cursor");
            if cursor_dir.exists() && cursor_dir.is_dir() {
                for entry in WalkDir::new(&cursor_dir)
                    .max_depth(2)
                    .into_iter()
                    .filter_map(|e| e.ok())
                {
                    let path = entry.path();
                    if path.is_file() {
                        if let Some(extension) = path.extension() {
                            let ext_str = extension.to_string_lossy().to_lowercase();
                            if matches!(ext_str.as_str(), "json" | "yaml" | "yml" | "toml") {
                                if let Ok(config_file) = self.load_config_file(path).await {
                                    config_files.push(config_file);
                                }
                            }
                        }
                    }
                }
            }

            // Move up one directory
            if !search_dir.pop() {
                break;
            }
        }

        // Remove duplicates based on path
        config_files.sort_by(|a, b| a.path.cmp(&b.path));
        config_files.dedup_by(|a, b| a.path == b.path);

        Ok(config_files)
    }

    /// Load a single config file
    async fn load_config_file(&self, path: &Path) -> Result<AgentConfigFile> {
        let content = tokio::fs::read_to_string(path).await?;
        let format = self.detect_config_format(path, &content);

        Ok(AgentConfigFile {
            path: path.to_string_lossy().to_string(),
            content,
            format,
        })
    }

    /// Detect configuration file format
    fn detect_config_format(&self, path: &Path, content: &str) -> ConfigFormat {
        // First try by extension
        if let Some(extension) = path.extension() {
            match extension.to_string_lossy().to_lowercase().as_str() {
                "json" => return ConfigFormat::Json,
                "yaml" | "yml" => return ConfigFormat::Yaml,
                "toml" => return ConfigFormat::Toml,
                _ => {}
            }
        }

        // Try to detect by content
        let trimmed = content.trim();
        if trimmed.starts_with('{') && trimmed.ends_with('}') {
            ConfigFormat::Json
        } else if trimmed.contains(':') && (trimmed.contains('\n') || trimmed.len() > 100) {
            ConfigFormat::Yaml
        } else if trimmed.contains('[') && trimmed.contains(']') && trimmed.contains('=') {
            ConfigFormat::Toml
        } else {
            ConfigFormat::Text
        }
    }

    /// Extract rules from configuration files
    async fn extract_rules(&self, config_files: &[AgentConfigFile]) -> Vec<String> {
        let mut rules = Vec::new();

        for config_file in config_files {
            match config_file.format {
                ConfigFormat::Json => {
                    if let Ok(json_value) =
                        serde_json::from_str::<serde_json::Value>(&config_file.content)
                    {
                        self.extract_rules_from_json(&json_value, &mut rules);
                    }
                }
                ConfigFormat::Yaml => {
                    if let Ok(yaml_value) =
                        serde_yaml::from_str::<serde_yaml::Value>(&config_file.content)
                    {
                        self.extract_rules_from_yaml(&yaml_value, &mut rules);
                    }
                }
                ConfigFormat::Text => {
                    // For text files like .cursorrules, treat each non-empty line as a rule
                    for line in config_file.content.lines() {
                        let trimmed = line.trim();
                        if !trimmed.is_empty() && !trimmed.starts_with('#') {
                            rules.push(trimmed.to_string());
                        }
                    }
                }
                ConfigFormat::Toml => {
                    // Basic TOML parsing for rules
                    for line in config_file.content.lines() {
                        let trimmed = line.trim();
                        if trimmed.starts_with("rule") && trimmed.contains('=') {
                            if let Some(rule_value) = trimmed.split('=').nth(1) {
                                let cleaned =
                                    rule_value.trim().trim_matches('"').trim_matches('\'');
                                if !cleaned.is_empty() {
                                    rules.push(cleaned.to_string());
                                }
                            }
                        }
                    }
                }
            }
        }

        rules
    }

    /// Extract rules from JSON value
    fn extract_rules_from_json(&self, value: &serde_json::Value, rules: &mut Vec<String>) {
        let _ = &self; // Suppress clippy warning for recursion
        match value {
            serde_json::Value::Object(map) => {
                // Look for common rule fields
                for key in &[
                    "rules",
                    "instructions",
                    "prompts",
                    "guidelines",
                    "constraints",
                ] {
                    if let Some(rule_value) = map.get(*key) {
                        self.extract_rules_from_json(rule_value, rules);
                    }
                }
            }
            serde_json::Value::Array(arr) => {
                for item in arr {
                    if let serde_json::Value::String(s) = item {
                        rules.push(s.clone());
                    } else {
                        self.extract_rules_from_json(item, rules);
                    }
                }
            }
            serde_json::Value::String(s) => {
                rules.push(s.clone());
            }
            _ => {}
        }
    }

    /// Extract rules from YAML value
    fn extract_rules_from_yaml(&self, value: &serde_yaml::Value, rules: &mut Vec<String>) {
        let _ = &self; // Suppress clippy warning for recursion
        match value {
            serde_yaml::Value::Mapping(map) => {
                for key in &[
                    "rules",
                    "instructions",
                    "prompts",
                    "guidelines",
                    "constraints",
                ] {
                    if let Some(rule_value) = map.get(serde_yaml::Value::String(key.to_string())) {
                        self.extract_rules_from_yaml(rule_value, rules);
                    }
                }
            }
            serde_yaml::Value::Sequence(seq) => {
                for item in seq {
                    if let serde_yaml::Value::String(s) = item {
                        rules.push(s.clone());
                    } else {
                        self.extract_rules_from_yaml(item, rules);
                    }
                }
            }
            serde_yaml::Value::String(s) => {
                rules.push(s.clone());
            }
            _ => {}
        }
    }

    /// Extract custom prompts from configuration files
    async fn extract_custom_prompts(
        &self,
        config_files: &[AgentConfigFile],
    ) -> HashMap<String, String> {
        let mut prompts = HashMap::new();

        for config_file in config_files {
            match config_file.format {
                ConfigFormat::Json => {
                    if let Ok(json_value) =
                        serde_json::from_str::<serde_json::Value>(&config_file.content)
                    {
                        self.extract_prompts_from_json(&json_value, &mut prompts);
                    }
                }
                ConfigFormat::Yaml => {
                    if let Ok(yaml_value) =
                        serde_yaml::from_str::<serde_yaml::Value>(&config_file.content)
                    {
                        self.extract_prompts_from_yaml(&yaml_value, &mut prompts);
                    }
                }
                _ => {} // Text and TOML don't typically contain structured prompts
            }
        }

        prompts
    }

    /// Extract prompts from JSON value
    fn extract_prompts_from_json(
        &self,
        value: &serde_json::Value,
        prompts: &mut HashMap<String, String>,
    ) {
        if let serde_json::Value::Object(map) = value {
            for (key, val) in map {
                if key.contains("prompt") || key.contains("template") {
                    if let serde_json::Value::String(s) = val {
                        prompts.insert(key.clone(), s.clone());
                    }
                }
            }
        }
    }

    /// Extract prompts from YAML value
    fn extract_prompts_from_yaml(
        &self,
        value: &serde_yaml::Value,
        prompts: &mut HashMap<String, String>,
    ) {
        if let serde_yaml::Value::Mapping(map) = value {
            for (key, val) in map {
                if let serde_yaml::Value::String(key_str) = key {
                    if key_str.contains("prompt") || key_str.contains("template") {
                        if let serde_yaml::Value::String(val_str) = val {
                            prompts.insert(key_str.clone(), val_str.clone());
                        }
                    }
                }
            }
        }
    }
}

#[async_trait::async_trait]
impl ContextProvider for AgentContextProvider {
    async fn gather(&self) -> Result<ContextData> {
        let config_files = self.find_config_files().await?;
        let rules = self.extract_rules(&config_files).await;
        let custom_prompts = self.extract_custom_prompts(&config_files).await;

        let agent_context = AgentContext {
            config_files,
            rules,
            custom_prompts,
        };

        Ok(ContextData::Agent(agent_context))
    }

    fn context_type(&self) -> ContextType {
        ContextType::Agent
    }

    async fn should_refresh(&self, _cached_data: &ContextData) -> Result<bool> {
        // Agent configuration changes infrequently, so we can cache it longer
        // The cache system will handle time-based expiry
        Ok(false)
    }

    fn get_file_dependencies(&self) -> Vec<PathBuf> {
        // Return paths to agent configuration files that this context depends on
        let mut files = Vec::new();

        // Common cursor-agent config locations in current directory
        let config_paths = [".cursor-agent", ".aiconfig", ".cursorignore"];

        for path in &config_paths {
            let path_buf = PathBuf::from(path);
            if path_buf.exists() {
                files.push(path_buf);
            }
        }

        // Also check home directory for global config files
        if let Some(home_dir) = dirs::home_dir() {
            let global_paths = [
                ".cursor/argv.json",
                ".cursor/cli-config.json",
                ".cursor/extensions/extensions.json",
                ".cursor/ide_state.json",
                ".cursor/prompt_history.json",
                ".cursor/unified_repo_list.json",
            ];

            for path in &global_paths {
                let path_buf = home_dir.join(path);
                if path_buf.exists() {
                    files.push(path_buf);
                }
            }
        }

        files
    }
}
