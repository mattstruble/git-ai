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
