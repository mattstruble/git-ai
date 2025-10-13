use anyhow::{Context, Result};
use std::process::Command as StdCommand;

use crate::{config::Config, prompts::get_prompt_for_command, Commands};

/// Execute a git-ai command by getting the appropriate prompt and running cursor-agent
pub async fn execute_command(command: &Commands, config: &Config, no_confirm: bool) -> Result<()> {
    let prompts = config.get_prompts();

    let (command_name, branch, custom_message) = match command {
        Commands::Commit { message, .. } => ("commit", None, message.as_deref()),
        Commands::Pr { message, .. } => ("pr", None, message.as_deref()),
        Commands::Merge {
            branch, message, ..
        } => ("merge", Some(branch.as_str()), message.as_deref()),
        Commands::Completions { .. } | Commands::Config { .. } => {
            return Err(anyhow::anyhow!(
                "Completions and Config should be handled in main"
            ));
        }
    };

    let prompt = get_prompt_for_command(&prompts, command_name, branch, custom_message);

    run_cursor_agent(&prompt, no_confirm).await
}

/// Run cursor-agent with the given prompt
async fn run_cursor_agent(prompt: &str, no_confirm: bool) -> Result<()> {
    let mut cmd = StdCommand::new("cursor-agent");
    cmd.args(["-p", prompt]);

    if no_confirm {
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
    use crate::Commands;

    #[test]
    fn test_command_extraction() {
        let commit_cmd = Commands::Commit {
            message: Some("Test commit".to_string()),
            no_confirm: false,
            dry_run: false,
            verbose: false,
        };
        let pr_cmd = Commands::Pr {
            message: None,
            no_confirm: false,
            dry_run: false,
            verbose: false,
        };
        let merge_cmd = Commands::Merge {
            branch: "feature/test".to_string(),
            message: Some("Test merge".to_string()),
            no_confirm: false,
            dry_run: false,
            verbose: false,
        };

        // Test that we can extract command info (this would normally be tested in integration tests)
        match commit_cmd {
            Commands::Commit { message, .. } => {
                assert_eq!(message.as_deref(), Some("Test commit"));
            }
            _ => panic!("Wrong command type"),
        }

        match pr_cmd {
            Commands::Pr { message, .. } => {
                assert_eq!(message, None);
            }
            _ => panic!("Wrong command type"),
        }

        match merge_cmd {
            Commands::Merge {
                branch, message, ..
            } => {
                assert_eq!(branch, "feature/test");
                assert_eq!(message.as_deref(), Some("Test merge"));
            }
            _ => panic!("Wrong command type"),
        }
    }

    #[test]
    fn test_prompt_generation() {
        let config = Config::default();
        let prompts = config.get_prompts();

        // Test getting different command prompts
        let commit_prompt = get_prompt_for_command(&prompts, "commit", None, None);
        assert!(commit_prompt.contains("commit"));

        let pr_prompt = get_prompt_for_command(&prompts, "pr", None, None);
        assert!(pr_prompt.contains("pull request"));

        let merge_prompt = get_prompt_for_command(&prompts, "merge", Some("main"), None);
        assert!(merge_prompt.contains("main"));

        // Test with custom message
        let custom_prompt =
            get_prompt_for_command(&prompts, "commit", None, Some("Focus on tests"));
        assert!(custom_prompt.contains("Focus on tests"));
        assert!(custom_prompt.contains("additional context"));
    }
}
