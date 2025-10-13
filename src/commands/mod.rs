pub mod commit;
pub mod merge;
pub mod pr;

use anyhow::{Context, Result};
use std::process::Command as StdCommand;

use crate::Commands;
use commit::CommitCommand;
use merge::MergeCommand;
use pr::PrCommand;

/// Enum wrapper for all commands to handle async dispatch
pub enum GitAiCommand {
    Commit(CommitCommand),
    Pr(PrCommand),
    Merge(MergeCommand, String), // Include target branch
}

impl GitAiCommand {
    /// Create a GitAiCommand from the CLI command enum
    pub fn from_cli_command(command: &Commands) -> Self {
        match command {
            Commands::Commit { .. } => GitAiCommand::Commit(CommitCommand),
            Commands::Pr { .. } => GitAiCommand::Pr(PrCommand),
            Commands::Merge { branch, .. } => GitAiCommand::Merge(MergeCommand, branch.clone()),
        }
    }

    /// Get the default prompt for this command
    pub fn default_prompt(&self, custom_message: Option<String>) -> String {
        match self {
            GitAiCommand::Commit(cmd) => cmd.default_prompt(custom_message),
            GitAiCommand::Pr(cmd) => cmd.default_prompt(custom_message),
            GitAiCommand::Merge(cmd, branch) => {
                cmd.default_prompt_with_branch(branch, custom_message)
            }
        }
    }

    /// Execute the command with the given prompt
    pub async fn execute(&self, prompt: &str, force: bool) -> Result<()> {
        run_cursor_agent(prompt, force).await
    }
}

/// Run cursor-agent with the given prompt and optional force flag
async fn run_cursor_agent(prompt: &str, force: bool) -> Result<()> {
    let mut cmd = StdCommand::new("cursor-agent");
    cmd.args(["-p", prompt]);

    if force {
        cmd.arg("--force");
    }

    let status = cmd.status().context("Failed to run cursor-agent")?;

    if !status.success() {
        anyhow::bail!("cursor-agent command failed");
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_cli_command_commit() {
        let cli_command = Commands::Commit {
            message: Some("Test message".to_string()),
            force: true,
        };
        
        let git_ai_command = GitAiCommand::from_cli_command(&cli_command);
        
        match git_ai_command {
            GitAiCommand::Commit(_) => {}, // Expected
            _ => panic!("Expected GitAiCommand::Commit"),
        }
    }

    #[test]
    fn test_from_cli_command_pr() {
        let cli_command = Commands::Pr {
            message: None,
            force: false,
        };
        
        let git_ai_command = GitAiCommand::from_cli_command(&cli_command);
        
        match git_ai_command {
            GitAiCommand::Pr(_) => {}, // Expected
            _ => panic!("Expected GitAiCommand::Pr"),
        }
    }

    #[test]
    fn test_from_cli_command_merge() {
        let cli_command = Commands::Merge {
            branch: "feature/test".to_string(),
            message: Some("Merge test".to_string()),
            force: true,
        };
        
        let git_ai_command = GitAiCommand::from_cli_command(&cli_command);
        
        match git_ai_command {
            GitAiCommand::Merge(_, branch) => {
                assert_eq!(branch, "feature/test");
            },
            _ => panic!("Expected GitAiCommand::Merge"),
        }
    }

    #[test]
    fn test_default_prompt_commit() {
        let command = GitAiCommand::Commit(CommitCommand);
        let prompt = command.default_prompt(Some("Custom commit message".to_string()));
        
        // Should contain commit-specific prompts
        assert!(prompt.contains("commit message") || prompt.contains("staged"));
        assert!(prompt.contains("Custom commit message"));
    }

    #[test]
    fn test_default_prompt_pr() {
        let command = GitAiCommand::Pr(PrCommand);
        let prompt = command.default_prompt(None);
        
        assert!(prompt.contains("pull request description"));
        assert!(prompt.contains("**Summary**"));
    }

    #[test]
    fn test_default_prompt_merge() {
        let branch = "develop";
        let command = GitAiCommand::Merge(MergeCommand, branch.to_string());
        let prompt = command.default_prompt(Some("Critical merge".to_string()));
        
        assert!(prompt.contains("develop"));
        assert!(prompt.contains("Critical merge"));
        assert!(prompt.contains("merge"));
    }

    #[test]
    fn test_default_prompt_merge_no_custom_message() {
        let branch = "main";
        let command = GitAiCommand::Merge(MergeCommand, branch.to_string());
        let prompt = command.default_prompt(None);
        
        assert!(prompt.contains("main"));
        assert!(prompt.contains("merge"));
        assert!(!prompt.contains("Specific context"));
    }

    // Note: run_cursor_agent is not easily testable without mocking the Command execution
    // In a real test suite, you might want to extract this into a trait for better testability
}
