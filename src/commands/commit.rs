use std::process::Command;

#[cfg(test)]
use mockall::{automock, predicate::*};

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

#[cfg_attr(test, automock)]
trait GitChecker {
    fn has_staged_files(&self) -> bool;
}

struct RealGitChecker;

impl GitChecker for RealGitChecker {
    fn has_staged_files(&self) -> bool {
        let output = Command::new("git")
            .args(["diff", "--cached", "--name-only"])
            .output();

        match output {
            Ok(result) => !result.stdout.is_empty(),
            Err(_) => false, // Assume no staged files if git command fails
        }
    }
}

impl CommitCommand {
    fn has_staged_files(&self) -> bool {
        self.has_staged_files_with_checker(&RealGitChecker)
    }

    #[cfg(test)]
    fn has_staged_files_with_checker<T: GitChecker>(&self, checker: &T) -> bool {
        checker.has_staged_files()
    }

    #[cfg(not(test))]
    fn has_staged_files_with_checker<T: GitChecker>(&self, checker: &T) -> bool {
        checker.has_staged_files()
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

    #[cfg(test)]
    fn default_prompt_with_checker<T: GitChecker>(&self, checker: &T, custom_message: Option<String>) -> String {
        let (base_prompt, instruction) = if checker.has_staged_files() {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_prompt_with_staged_files() {
        let mut mock_checker = MockGitChecker::new();
        mock_checker.expect_has_staged_files().returning(|| true);
        
        let command = CommitCommand;
        let prompt = command.default_prompt_with_checker(&mock_checker, None);
        
        assert!(prompt.contains("Generate a concise, descriptive commit message"));
        assert!(prompt.contains("git diff --cached"));
        assert!(!prompt.contains("No files are currently staged"));
    }

    #[test]
    fn test_default_prompt_without_staged_files() {
        let mut mock_checker = MockGitChecker::new();
        mock_checker.expect_has_staged_files().returning(|| false);
        
        let command = CommitCommand;
        let prompt = command.default_prompt_with_checker(&mock_checker, None);
        
        assert!(prompt.contains("No files are currently staged for commit"));
        assert!(prompt.contains("git diff"));
        assert!(!prompt.contains("git diff --cached"));
    }

    #[test]
    fn test_default_prompt_with_custom_message_staged() {
        let mut mock_checker = MockGitChecker::new();
        mock_checker.expect_has_staged_files().returning(|| true);
        
        let command = CommitCommand;
        let custom_message = Some("Include unit tests".to_string());
        let prompt = command.default_prompt_with_checker(&mock_checker, custom_message);
        
        assert!(prompt.contains("Specific requirements: Include unit tests"));
        assert!(prompt.contains("Generate a concise, descriptive commit message"));
    }

    #[test]
    fn test_default_prompt_with_custom_message_unstaged() {
        let mut mock_checker = MockGitChecker::new();
        mock_checker.expect_has_staged_files().returning(|| false);
        
        let command = CommitCommand;
        let custom_message = Some("Focus on documentation".to_string());
        let prompt = command.default_prompt_with_checker(&mock_checker, custom_message);
        
        assert!(prompt.contains("Specific requirements: Focus on documentation"));
        assert!(prompt.contains("No files are currently staged"));
    }
}
