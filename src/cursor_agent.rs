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

    /// Execute a one-off prompt and return the result as a string
    pub async fn prompt(&self, prompt: &str) -> Result<String> {
        let mut cmd = StdCommand::new("cursor-agent");
        cmd.arg("--print");
        cmd.arg(prompt);

        // Capture output instead of letting it run interactively
        let output = cmd.output().context("Failed to execute cursor-agent")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow::anyhow!("cursor-agent command failed: {}", stderr));
        }

        let result = String::from_utf8(output.stdout)
            .context("cursor-agent output contained invalid UTF-8")?;

        Ok(result.trim().to_string())
    }
}
