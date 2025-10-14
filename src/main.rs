mod cli;
mod commands;
mod config;
mod cursor_agent;

use anyhow::Result;
use clap::{Parser, Subcommand};
use std::process::Command as StdCommand;

#[derive(Parser)]
#[command(name = "git-ai")]
#[command(about = "AI-assisted git workflow with cursor-agent")]
#[command(version = "0.5.0")]
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

        /// Skip user confirmation prompts
        #[arg(long)]
        no_confirm: bool,

        /// Print the prompt without executing cursor-agent
        #[arg(long)]
        dry_run: bool,

        /// Show verbose output for debugging
        #[arg(short, long)]
        verbose: bool,
    },
    /// Generate AI-assisted PR description
    Pr {
        /// Custom message to guide the AI
        #[arg(short, long)]
        message: Option<String>,

        /// Skip user confirmation prompts
        #[arg(long)]
        no_confirm: bool,

        /// Print the prompt without executing cursor-agent
        #[arg(long)]
        dry_run: bool,

        /// Show verbose output for debugging
        #[arg(short, long)]
        verbose: bool,
    },
    /// Generate AI-assisted merge summary
    Merge {
        /// Target branch to merge
        branch: String,

        /// Custom message to guide the AI
        #[arg(short, long)]
        message: Option<String>,

        /// Skip user confirmation prompts
        #[arg(long)]
        no_confirm: bool,

        /// Print the prompt without executing cursor-agent
        #[arg(long)]
        dry_run: bool,

        /// Show verbose output for debugging
        #[arg(short, long)]
        verbose: bool,
    },
    /// Generate sample configuration file
    Config {
        /// Show current configuration path and status
        #[arg(long)]
        show: bool,

        /// Generate sample configuration
        #[arg(long)]
        init: bool,
    },
    /// Initialize a new project repository
    Init {
        /// Target programming language (e.g., python, javascript, rust, go)
        #[arg(short, long)]
        language: Option<String>,

        /// Project name
        #[arg(short, long)]
        name: Option<String>,

        /// Custom message to guide the AI
        #[arg(short, long)]
        message: Option<String>,

        /// Skip user confirmation prompts
        #[arg(long)]
        no_confirm: bool,

        /// Print the prompt without executing cursor-agent
        #[arg(long)]
        dry_run: bool,

        /// Show verbose output for debugging
        #[arg(short, long)]
        verbose: bool,
    },
    /// Manage .gitignore file entries
    Ignore {
        #[command(subcommand)]
        action: IgnoreAction,
    },
}

#[derive(Subcommand)]
enum IgnoreAction {
    /// Add ignore patterns for specified languages/tools
    Add {
        /// Languages or tools to add ignore patterns for (e.g., python, node, rust)
        languages: Vec<String>,

        /// Skip user confirmation prompts
        #[arg(long)]
        no_confirm: bool,

        /// Print the prompt without executing cursor-agent
        #[arg(long)]
        dry_run: bool,

        /// Show verbose output for debugging
        #[arg(short, long)]
        verbose: bool,
    },
    /// Remove ignore patterns for specified languages/tools
    Remove {
        /// Languages or tools to remove ignore patterns for
        languages: Vec<String>,

        /// Skip user confirmation prompts
        #[arg(long)]
        no_confirm: bool,

        /// Print the prompt without executing cursor-agent
        #[arg(long)]
        dry_run: bool,

        /// Show verbose output for debugging
        #[arg(short, long)]
        verbose: bool,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // Load configuration (all commands get consistent access)
    let config = config::Config::load()?;

    let (_dry_run, verbose) = match &cli.command {
        Commands::Commit {
            dry_run, verbose, ..
        } => (*dry_run, *verbose),
        Commands::Pr {
            dry_run, verbose, ..
        } => (*dry_run, *verbose),
        Commands::Merge {
            dry_run, verbose, ..
        } => (*dry_run, *verbose),
        Commands::Init {
            dry_run, verbose, ..
        } => (*dry_run, *verbose),
        Commands::Config { .. } => (false, false), // Config doesn't use cursor-agent
        Commands::Ignore { action } => match action {
            IgnoreAction::Add {
                dry_run, verbose, ..
            } => (*dry_run, *verbose),
            IgnoreAction::Remove {
                dry_run, verbose, ..
            } => (*dry_run, *verbose),
        },
    };

    // Override CLI flags with config values where appropriate
    let effective_verbose = verbose || config.behavior.verbose;
    ensure_cursor_agent_available(effective_verbose)?;

    // Dry run is now handled by individual commands

    if effective_verbose {
        println!("🔧 Executing git-ai command...");
    }

    let dispatcher = cli::CommandDispatcher::new(config);
    dispatcher.dispatch(cli.command).await?;

    Ok(())
}

/// Ensure cursor-agent is available on the system
fn ensure_cursor_agent_available(verbose: bool) -> Result<()> {
    if let Ok(output) = StdCommand::new("cursor-agent").arg("--version").output() {
        if output.status.success() {
            if verbose {
                let version = String::from_utf8_lossy(&output.stdout);
                println!("✅ cursor-agent found: {}", version.trim());
            }
            return Ok(());
        }
    }

    eprintln!("❌ cursor-agent is not installed or not found in PATH");
    eprintln!();
    eprintln!("Please install cursor-agent before using git-ai:");
    eprintln!("  Visit: https://cursor.com/");
    eprintln!("  Or use your package manager:");
    eprintln!("    • macOS: brew install cursor");
    eprintln!("    • Linux: Check your distribution's package manager");
    eprintln!("    • Windows: Download from https://cursor.com/");
    eprintln!();
    eprintln!("After installation, make sure cursor-agent is in your PATH.");

    anyhow::bail!("cursor-agent not found");
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::{CommandFactory, Parser};

    #[test]
    fn test_cli_parsing_commit_command() {
        let args = vec!["git-ai", "commit", "-m", "test message", "--no-confirm"];
        let cli = Cli::try_parse_from(args).unwrap();

        match cli.command {
            Commands::Commit {
                message,
                no_confirm,
                dry_run,
                verbose,
            } => {
                assert_eq!(message, Some("test message".to_string()));
                assert!(no_confirm);
                assert!(!dry_run);
                assert!(!verbose);
            }
            _ => panic!("Expected commit command"),
        }
    }

    #[test]
    fn test_cli_parsing_commit_command_minimal() {
        let args = vec!["git-ai", "commit"];
        let cli = Cli::try_parse_from(args).unwrap();

        match cli.command {
            Commands::Commit {
                message,
                no_confirm,
                dry_run,
                verbose,
            } => {
                assert_eq!(message, None);
                assert!(!no_confirm);
                assert!(!dry_run);
                assert!(!verbose);
            }
            _ => panic!("Expected commit command"),
        }
    }

    #[test]
    fn test_cli_parsing_pr_command() {
        let args = vec!["git-ai", "pr", "--message", "pr description"];
        let cli = Cli::try_parse_from(args).unwrap();

        match cli.command {
            Commands::Pr {
                message,
                no_confirm,
                dry_run,
                verbose,
            } => {
                assert_eq!(message, Some("pr description".to_string()));
                assert!(!no_confirm);
                assert!(!dry_run);
                assert!(!verbose);
            }
            _ => panic!("Expected pr command"),
        }
    }

    #[test]
    fn test_cli_parsing_merge_command() {
        let args = vec![
            "git-ai",
            "merge",
            "feature/branch",
            "-m",
            "merge message",
            "--no-confirm",
        ];
        let cli = Cli::try_parse_from(args).unwrap();

        match cli.command {
            Commands::Merge {
                branch,
                message,
                no_confirm,
                dry_run,
                verbose,
            } => {
                assert_eq!(branch, "feature/branch");
                assert_eq!(message, Some("merge message".to_string()));
                assert!(no_confirm);
                assert!(!dry_run);
                assert!(!verbose);
            }
            _ => panic!("Expected merge command"),
        }
    }

    #[test]
    fn test_cli_parsing_merge_command_minimal() {
        let args = vec!["git-ai", "merge", "main"];
        let cli = Cli::try_parse_from(args).unwrap();

        match cli.command {
            Commands::Merge {
                branch,
                message,
                no_confirm,
                dry_run,
                verbose,
            } => {
                assert_eq!(branch, "main");
                assert_eq!(message, None);
                assert!(!no_confirm);
                assert!(!dry_run);
                assert!(!verbose);
            }
            _ => panic!("Expected merge command"),
        }
    }

    #[test]
    fn test_cli_name() {
        let cli = Cli::command();
        let name = cli.get_name();
        assert_eq!(name, "git-ai");
    }
}
