use anyhow::{Context, Result};
use std::process::Command as StdCommand;

/// Service for interacting with cursor-agent
#[derive(Debug, Clone)]
pub struct CursorAgent;

impl CursorAgent {
    pub fn new() -> Self {
        Self
    }

    /// Execute cursor-agent with the given prompt
    pub async fn execute(&self, prompt: &str, no_confirm: bool) -> Result<()> {
        let mut cmd = StdCommand::new("cursor-agent");
        cmd.args(["prompt", prompt]);

        if no_confirm {
            cmd.arg("--force");
        }

        let status = cmd.status().context("Failed to run cursor-agent")?;

        if !status.success() {
            anyhow::bail!("cursor-agent command failed");
        }

        Ok(())
    }
}
