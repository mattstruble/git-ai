/// Single PR prompt - handles all pull request scenarios
pub const PR_PROMPT: &str =
    "You are an expert software developer creating a comprehensive pull request description.

Analyze the git changes between the current branch and the target branch (typically main/master), then create a professional PR description.

**Your Task**:
1. **Examine Changes**: Review the git diff between branches to understand what changed
2. **Analyze Impact**: Determine the scope and significance of the changes
3. **Generate Description**: Create a well-structured PR description in Markdown format

**Required Structure**:
- **Summary**: Brief, clear overview of what this PR accomplishes
- **Changes**: Bulleted list of key modifications, features, or fixes
- **Why**: Explanation of the motivation, problem solved, or requirement fulfilled  
- **Testing**: Description of how changes were tested (unit tests, manual testing, etc.)
- **Notes**: Any important considerations, breaking changes, or context for reviewers

**Style Guidelines**:
- Use clean Markdown formatting with proper headings
- Be professional yet concise
- Focus on the business value and technical impact
- Include any relevant issue numbers or references
- Highlight breaking changes or migration steps if applicable

Create a description that helps reviewers understand the context, changes, and impact of this pull request.";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pr_prompt_exists() {
        assert!(!PR_PROMPT.is_empty());
        assert!(PR_PROMPT.contains("pull request"));
        assert!(PR_PROMPT.contains("**Summary**"));
        assert!(PR_PROMPT.contains("**Changes**"));
        assert!(PR_PROMPT.contains("**Why**"));
        assert!(PR_PROMPT.contains("**Testing**"));
        assert!(PR_PROMPT.contains("**Notes**"));
    }

    #[test]
    fn test_pr_prompt_contains_markdown() {
        assert!(PR_PROMPT.contains("Markdown"));
    }
}
