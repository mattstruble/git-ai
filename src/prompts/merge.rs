/// Single merge prompt - handles all merge scenarios
/// Use format!() to substitute the branch name: format!(MERGE_PROMPT, branch_name)
pub const MERGE_PROMPT: &str =
    "You are an expert software developer tasked with analyzing and assisting with merging the branch '{}' into the current branch.

**Your Task**:
1. **Analyze Branch Differences**: Examine what changes exist in '{}' that aren't in the current branch
2. **Check for Conflicts**: Determine if there are any merge conflicts and their nature
3. **Provide Guidance**: Based on the git status and changes, provide appropriate guidance

**If There Are Merge Conflicts**:
- Explain what caused the conflicts between the branches
- Identify the specific files and areas of conflict
- Suggest a resolution strategy for each conflict
- Provide step-by-step guidance for resolving conflicts
- Recommend an appropriate merge commit message after resolution

**If No Conflicts (Clean Merge)**:
- Summarize what changes from '{}' will be integrated
- Highlight key features, fixes, or modifications being brought in
- Generate an appropriate merge commit message following the format: 'Merge branch {}'
- Explain the impact and value of these changes to the codebase

**For Merge Commit Messages**:
- Use standard format: 'Merge branch {}' or 'Merge branch {} into current-branch'
- Include a brief description of what '{}' brings to the codebase
- Mention any significant features, fixes, or changes
- Keep it concise but informative

**General Guidance**:
- Review the current git status carefully
- Consider the branch's purpose and changes
- Provide clear, actionable next steps
- Warn about any potential breaking changes or impacts

Analyze the current repository state and provide comprehensive merge guidance for integrating '{}'.";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_merge_prompt_exists() {
        assert!(!MERGE_PROMPT.is_empty());
        assert!(MERGE_PROMPT.contains("merge"));
        assert!(MERGE_PROMPT.contains("branch"));
        assert!(MERGE_PROMPT.contains("conflicts"));
        assert!(MERGE_PROMPT.contains("commit message"));
    }

    #[test]
    fn test_merge_prompt_has_placeholders() {
        // Should have multiple {} placeholders for branch name substitution
        let placeholder_count = MERGE_PROMPT.matches("{}").count();
        assert!(
            placeholder_count >= 6,
            "Should have multiple branch placeholders"
        );
    }

    #[test]
    fn test_merge_prompt_formatting() {
        let branch = "feature/test";
        let formatted = MERGE_PROMPT.replace("{}", branch);

        assert!(formatted.contains("feature/test"));
        assert!(!formatted.contains("{}"));
    }
}
