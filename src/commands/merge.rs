const MERGE_BASE_PROMPT_TEMPLATE: &str =
    "Analyze the merge of '{}' into the current branch and provide merge assistance.

If there are merge conflicts:
- Explain what caused the conflicts between the current branch and '{}'
- Identify the conflicting areas and their purposes
- Suggest a resolution strategy specific to merging '{}'
- Provide an appropriate merge commit message

If this is for generating a merge commit message:
- Summarize what changes from '{}' are being integrated
- Highlight key features or fixes being merged
- Follow standard merge commit format: 'Merge branch {}' or similar
- Include brief description of what '{}' brings to the codebase

Focus on clarity and helping developers understand the specific merge context for '{}'.";

const MERGE_INSTRUCTION_TEMPLATE: &str =
    "Please review the git status and provide appropriate merge guidance for '{}'.";

pub struct MergeCommand;

impl MergeCommand {
    pub fn default_prompt_with_branch(
        &self,
        target_branch: &str,
        custom_message: Option<String>,
    ) -> String {
        let base_prompt = MERGE_BASE_PROMPT_TEMPLATE.replace("{}", target_branch);

        let instruction = MERGE_INSTRUCTION_TEMPLATE.replace("{}", target_branch);

        match custom_message {
            Some(message) => format!(
                "{}\n\nSpecific context: {}\n\n{}",
                base_prompt, message, instruction
            ),
            None => format!("{}\n\n{}", base_prompt, instruction),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_prompt_with_branch_no_custom_message() {
        let command = MergeCommand;
        let branch = "feature/new-feature";
        let prompt = command.default_prompt_with_branch(branch, None);
        
        assert!(prompt.contains("Analyze the merge of 'feature/new-feature' into the current branch"));
        assert!(prompt.contains("merge of 'feature/new-feature'"));
        assert!(prompt.contains("Merge branch feature/new-feature"));
        assert!(prompt.contains("Please review the git status and provide appropriate merge guidance for 'feature/new-feature'"));
        assert!(!prompt.contains("Specific context"));
    }

    #[test]
    fn test_default_prompt_with_branch_and_custom_message() {
        let command = MergeCommand;
        let branch = "hotfix/critical-bug";
        let custom_message = Some("This fixes a security vulnerability".to_string());
        let prompt = command.default_prompt_with_branch(branch, custom_message);
        
        assert!(prompt.contains("Analyze the merge of 'hotfix/critical-bug' into the current branch"));
        assert!(prompt.contains("Specific context: This fixes a security vulnerability"));
        assert!(prompt.contains("merge guidance for 'hotfix/critical-bug'"));
    }

    #[test]
    fn test_branch_name_replacement() {
        let command = MergeCommand;
        let branch = "develop";
        let prompt = command.default_prompt_with_branch(branch, None);
        
        // Count occurrences to ensure all placeholders are replaced
        let develop_count = prompt.matches("develop").count();
        let placeholder_count = prompt.matches("{}").count();
        
        assert!(develop_count >= 6); // Should appear multiple times in different contexts
        assert_eq!(placeholder_count, 0); // No unreplaced placeholders
    }

    #[test]
    fn test_merge_conflict_guidance() {
        let command = MergeCommand;
        let branch = "feature/conflicting-changes";
        let prompt = command.default_prompt_with_branch(branch, None);
        
        assert!(prompt.contains("If there are merge conflicts"));
        assert!(prompt.contains("Explain what caused the conflicts"));
        assert!(prompt.contains("Suggest a resolution strategy"));
        assert!(prompt.contains("Provide an appropriate merge commit message"));
    }

    #[test]
    fn test_merge_commit_message_guidance() {
        let command = MergeCommand;
        let branch = "release/v1.0.0";
        let prompt = command.default_prompt_with_branch(branch, None);
        
        assert!(prompt.contains("generating a merge commit message"));
        assert!(prompt.contains("Summarize what changes from 'release/v1.0.0'"));
        assert!(prompt.contains("Follow standard merge commit format"));
        assert!(prompt.contains("what 'release/v1.0.0' brings to the codebase"));
    }

    #[test]
    fn test_empty_branch_name() {
        let command = MergeCommand;
        let branch = "";
        let prompt = command.default_prompt_with_branch(branch, None);
        
        // Should still work with empty branch name
        assert!(prompt.contains("Analyze the merge of '' into the current branch"));
        assert!(prompt.contains("merge guidance for ''"));
    }

    #[test]
    fn test_special_characters_in_branch() {
        let command = MergeCommand;
        let branch = "feature/user-auth-#123";
        let prompt = command.default_prompt_with_branch(branch, None);
        
        assert!(prompt.contains("feature/user-auth-#123"));
        assert!(prompt.contains("merge guidance for 'feature/user-auth-#123'"));
    }
}
