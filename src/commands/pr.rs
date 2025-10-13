const PR_BASE_PROMPT: &str =
    "Create a comprehensive pull request description based on the recent git changes.

The PR description should include:
- **Summary**: Clear overview of what this PR accomplishes
- **Changes**: Bullet points of key modifications
- **Why**: Motivation and context for the changes  
- **Testing**: How the changes were verified
- **Notes**: Any important considerations for reviewers

Format the output in clean Markdown with proper sections and be professional yet concise.";

const PR_INSTRUCTION: &str = "Please analyze the git changes and create a detailed PR description.";

pub struct PrCommand;

impl PrCommand {
    pub fn default_prompt(&self, custom_message: Option<String>) -> String {
        match custom_message {
            Some(message) => format!(
                "{}\n\nSpecific focus areas: {}\n\n{}",
                PR_BASE_PROMPT, message, PR_INSTRUCTION
            ),
            None => format!("{}\n\n{}", PR_BASE_PROMPT, PR_INSTRUCTION),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_prompt_without_custom_message() {
        let command = PrCommand;
        let prompt = command.default_prompt(None);
        
        assert!(prompt.contains("Create a comprehensive pull request description"));
        assert!(prompt.contains("**Summary**"));
        assert!(prompt.contains("**Changes**"));
        assert!(prompt.contains("**Why**"));
        assert!(prompt.contains("**Testing**"));
        assert!(prompt.contains("**Notes**"));
        assert!(prompt.contains("Please analyze the git changes"));
        assert!(!prompt.contains("Specific focus areas"));
    }

    #[test]
    fn test_default_prompt_with_custom_message() {
        let command = PrCommand;
        let custom_message = Some("Focus on security improvements".to_string());
        let prompt = command.default_prompt(custom_message);
        
        assert!(prompt.contains("Create a comprehensive pull request description"));
        assert!(prompt.contains("Specific focus areas: Focus on security improvements"));
        assert!(prompt.contains("Please analyze the git changes"));
        assert!(prompt.contains("**Summary**"));
    }

    #[test]
    fn test_prompt_structure() {
        let command = PrCommand;
        let prompt = command.default_prompt(None);
        
        // Verify the prompt has the expected sections in markdown format
        assert!(prompt.contains("- **Summary**"));
        assert!(prompt.contains("- **Changes**"));
        assert!(prompt.contains("- **Why**"));
        assert!(prompt.contains("- **Testing**"));
        assert!(prompt.contains("- **Notes**"));
        
        // Ensure it mentions markdown formatting
        assert!(prompt.contains("Format the output in clean Markdown"));
    }

    #[test]
    fn test_empty_custom_message() {
        let command = PrCommand;
        let custom_message = Some("".to_string());
        let prompt = command.default_prompt(custom_message);
        
        // Should still include the custom message structure even if empty
        assert!(prompt.contains("Specific focus areas: "));
        assert!(prompt.contains("Create a comprehensive pull request description"));
    }
}
