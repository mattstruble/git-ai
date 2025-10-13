use std::process::Command;

const COMMIT_STAGED_PROMPT: &str =
    "Generate a concise, descriptive commit message based on the currently staged changes.

The commit message should:
- Be under 72 characters for the subject line
- Focus on WHAT changed and WHY (not HOW)
- Follow conventional commit format
- Be written in present tense

Prefer staging and committing in separate logical groups whenever possible.";

const COMMIT_UNSTAGED_PROMPT: &str = "No files are currently staged for commit. Please stage your changes first using 'git add' and then run this command again.

However, if you want to commit all modified files, I can help you understand what would be committed:

The commit message should:
- Be under 72 characters for the subject line
- Focus on WHAT changed and WHY (not HOW)
- Follow conventional commit format
- Be written in present tense

Prefer staging and committing in separate logical groups whenever possible.";

const COMMIT_STAGED_INSTRUCTION: &str = "Please review the staged changes (git diff --cached) and generate an appropriate commit message.";

const COMMIT_UNSTAGED_INSTRUCTION: &str = "Please review all changes (git diff) and suggest what should be staged and committed, or generate a commit message if committing all changes.";

pub struct CommitCommand;

impl CommitCommand {
    fn has_staged_files(&self) -> bool {
        let output = Command::new("git")
            .args(&["diff", "--cached", "--name-only"])
            .output();

        match output {
            Ok(result) => !result.stdout.is_empty(),
            Err(_) => false, // Assume no staged files if git command fails
        }
    }

    pub fn default_prompt(&self, custom_message: Option<String>) -> String {
        let (base_prompt, instruction) = if self.has_staged_files() {
            (COMMIT_STAGED_PROMPT, COMMIT_STAGED_INSTRUCTION)
        } else {
            (COMMIT_UNSTAGED_PROMPT, COMMIT_UNSTAGED_INSTRUCTION)
        };

        match custom_message {
            Some(message) => format!(
                "{}\n\nSpecific requirements: {}\n\n{}",
                base_prompt, message, instruction
            ),
            None => format!("{}\n\n{}", base_prompt, instruction),
        }
    }
}
