mod commands;

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use std::io::Write;
use std::process::{Command as StdCommand, Stdio};

use commands::GitAiCommand;

const CURSOR_INSTALL_URL: &str = "https://cursor.com/install";

#[derive(Parser)]
#[command(name = "git-ai")]
#[command(about = "AI-assisted git workflow with cursor-agent")]
#[command(version = "0.1.0")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Generate AI-assisted commit message
    Commit {
        /// Custom message to guide the AI
        #[arg(short, long)]
        message: Option<String>,

        /// Allows the agent to make direct file changes without confirmation
        #[arg(short, long)]
        force: bool,
    },
    /// Generate AI-assisted PR description
    Pr {
        /// Custom message to guide the AI
        #[arg(short, long)]
        message: Option<String>,

        /// Allows the agent to make direct file changes without confirmation
        #[arg(short, long)]
        force: bool,
    },
    /// Generate AI-assisted merge summary
    Merge {
        /// Target branch to merge
        branch: String,

        /// Custom message to guide the AI
        #[arg(short, long)]
        message: Option<String>,

        /// Allows the agent to make direct file changes without confirmation
        #[arg(short, long)]
        force: bool,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    let (message, force) = match &cli.command {
        Commands::Commit { message, force } => (message.clone(), *force),
        Commands::Pr { message, force } => (message.clone(), *force),
        Commands::Merge { message, force, .. } => (message.clone(), *force),
    };

    register_git_alias()?;
    ensure_cursor_agent(force).await?;

    let command = GitAiCommand::from_cli_command(&cli.command);

    let prompt = command.default_prompt(message);
    command.execute(&prompt, force).await?;

    Ok(())
}

/// Register git alias if not already present
fn register_git_alias() -> Result<()> {
    let output = StdCommand::new("git")
        .args(&["config", "--global", "--get", "alias.ai"])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .context("Failed to check git alias")?;

    if !output.success() {
        println!("Registering 'git ai' alias...");

        StdCommand::new("git")
            .args(&["config", "--global", "alias.ai", "!git-ai"])
            .status()
            .context("Failed to register git alias")?;

        println!("✅ Alias added: you can now use 'git ai <command>'");
    }

    Ok(())
}

/// Ensure cursor-agent is installed
async fn ensure_cursor_agent(force: bool) -> Result<()> {
    // Check if cursor-agent exists and force flag is not set
    if !force {
        if let Ok(output) = StdCommand::new("cursor-agent")
            .arg("--version")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
        {
            if output.success() {
                return Ok(());
            }
        }
    }

    println!("⚙️  Installing or updating cursor-agent...");

    // Download the install script
    let client = reqwest::Client::new();
    let response = client
        .get(CURSOR_INSTALL_URL)
        .send()
        .await
        .context("Failed to download cursor-agent installer")?;

    let script_content = response
        .text()
        .await
        .context("Failed to read installer script")?;

    // Execute the install script
    let mut child = StdCommand::new("bash")
        .stdin(Stdio::piped())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()
        .context("Failed to start bash process for installation")?;

    let stdin = child.stdin.as_mut().context("Failed to get stdin")?;
    stdin
        .write_all(script_content.as_bytes())
        .context("Failed to write install script to bash")?;

    let status = child.wait().context("Failed to wait for installation")?;

    if status.success() {
        println!("✅ cursor-agent installed successfully.");
    } else {
        anyhow::bail!("cursor-agent installation failed");
    }

    Ok(())
}
