pub mod commit;
pub mod merge;
pub mod pr;

use serde::{Deserialize, Serialize};

/// Simplified prompt registry with single prompts per command
#[derive(Debug, Clone)]
pub struct PromptRegistry {
    pub commit: String,
    pub pr: String,
    pub merge: String,
}

impl Default for PromptRegistry {
    fn default() -> Self {
        Self {
            commit: commit::COMMIT_PROMPT.to_string(),
            pr: pr::PR_PROMPT.to_string(),
            merge: merge::MERGE_PROMPT.to_string(),
        }
    }
}

/// Configuration overrides for prompts - simplified to match single prompt structure
#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct PromptConfig {
    pub commit: Option<String>,
    pub pr: Option<String>,
    pub merge: Option<String>,
}

impl PromptRegistry {
    /// Create a new registry with config overrides applied
    pub fn with_overrides(&self, config: &PromptConfig) -> Self {
        let mut registry = self.clone();

        if let Some(ref prompt) = config.commit {
            registry.commit = prompt.clone();
        }

        if let Some(ref prompt) = config.pr {
            registry.pr = prompt.clone();
        }

        if let Some(ref prompt) = config.merge {
            registry.merge = prompt.clone();
        }

        registry
    }
}

/// Add custom message context to any prompt
pub fn add_custom_message(base_prompt: &str, custom_message: Option<&str>) -> String {
    match custom_message {
        Some(message) => format!(
            "{}\n\nThe user has provided this additional context to focus on: {}",
            base_prompt, message
        ),
        None => base_prompt.to_string(),
    }
}

/// Get the appropriate prompt for a command with optional custom message
pub fn get_prompt_for_command(
    registry: &PromptRegistry,
    command: &str,
    branch: Option<&str>,
    custom_message: Option<&str>,
) -> String {
    let base_prompt = match command {
        "commit" => &registry.commit,
        "pr" => &registry.pr,
        "merge" => {
            if let Some(branch_name) = branch {
                // Format the merge prompt with the branch name
                let formatted = registry.merge.replace("{}", branch_name).to_string();
                return add_custom_message(&formatted, custom_message);
            } else {
                &registry.merge
            }
        }
        _ => return format!("Unknown command: {}", command),
    };

    add_custom_message(base_prompt, custom_message)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_custom_message_formatting() {
        let base = "Base prompt";
        let custom = "Custom requirement";

        let result = add_custom_message(base, Some(custom));
        assert!(result.contains("Base prompt"));
        assert!(result.contains(
            "The user has provided this additional context to focus on: Custom requirement"
        ));

        let result_no_custom = add_custom_message(base, None);
        assert_eq!(result_no_custom, "Base prompt");
    }

    #[test]
    fn test_prompt_config_overrides() {
        let registry = PromptRegistry::default();
        let config = PromptConfig {
            commit: Some("Custom commit prompt".to_string()),
            pr: Some("Custom PR prompt".to_string()),
            ..Default::default()
        };

        let overridden = registry.with_overrides(&config);
        assert_eq!(overridden.commit, "Custom commit prompt");
        assert_eq!(overridden.pr, "Custom PR prompt");
        // Unchanged prompts should remain the same
        assert_eq!(overridden.merge, registry.merge);
    }

    #[test]
    fn test_get_prompt_for_command() {
        let registry = PromptRegistry::default();

        // Test commit command
        let commit_prompt = get_prompt_for_command(&registry, "commit", None, None);
        assert!(commit_prompt.contains("commit"));

        // Test PR command
        let pr_prompt = get_prompt_for_command(&registry, "pr", None, None);
        assert!(pr_prompt.contains("pull request"));

        // Test merge command with branch
        let merge_prompt = get_prompt_for_command(&registry, "merge", Some("feature/test"), None);
        assert!(merge_prompt.contains("feature/test"));

        // Test with custom message
        let custom_prompt =
            get_prompt_for_command(&registry, "commit", None, Some("Focus on tests"));
        assert!(custom_prompt.contains("Focus on tests"));
        assert!(custom_prompt.contains("additional context"));
    }
}
