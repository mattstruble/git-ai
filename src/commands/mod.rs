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
    cmd.args(&["-p", prompt]);

    if force {
        cmd.arg("--force");
    }

    let status = cmd.status().context("Failed to run cursor-agent")?;

    if !status.success() {
        anyhow::bail!("cursor-agent command failed");
    }

    Ok(())
}
