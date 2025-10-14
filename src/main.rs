mod cli;
mod config;
mod prompts;

use anyhow::Result;
use clap::{Parser, Subcommand};
use std::process::Command as StdCommand;

#[derive(Parser)]
#[command(name = "git-ai")]
#[command(about = "AI-assisted git workflow with cursor-agent")]
#[command(version = "0.4.5")]
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
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // Handle special commands that don't need config loading
    if let Commands::Config { show, init } = &cli.command {
        return handle_config_command(*show, *init);
    }

    // Load configuration
    let config = config::Config::load()?;

    let (dry_run, verbose) = match &cli.command {
        Commands::Commit {
            dry_run, verbose, ..
        } => (*dry_run, *verbose),
        Commands::Pr {
            dry_run, verbose, ..
        } => (*dry_run, *verbose),
        Commands::Merge {
            dry_run, verbose, ..
        } => (*dry_run, *verbose),
        Commands::Config { .. } => unreachable!("Handled above"),
    };

    // Override CLI flags with config values where appropriate
    let effective_verbose = verbose || config.behavior.verbose;
    ensure_cursor_agent_available(effective_verbose)?;

    if dry_run {
        // Generate prompt for dry-run display
        let prompts = config.get_prompts();
        let (command_name, branch, custom_message) = match &cli.command {
            Commands::Commit { message, .. } => ("commit", None, message.as_deref()),
            Commands::Pr { message, .. } => ("pr", None, message.as_deref()),
            Commands::Merge {
                branch, message, ..
            } => ("merge", Some(branch.as_str()), message.as_deref()),
            _ => unreachable!(),
        };
        let prompt =
            prompts::get_prompt_for_command(&prompts, command_name, branch, custom_message);

        println!("🔍 Dry run mode - would execute with prompt:");
        println!("---");
        println!("{}", prompt);
        println!("---");
        return Ok(());
    }

    if effective_verbose {
        println!("🔧 Executing git-ai command...");
    }

    cli::execute_command(&cli.command, &config).await?;

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

/// Handle the config command
fn handle_config_command(show: bool, init: bool) -> Result<()> {
    if init {
        let sample_config = config::Config::create_sample_config()?;
        println!("# Sample git-ai configuration");
        println!("# Copy this to ~/.config/git-ai/config.yaml or .git-ai.yaml");
        println!();
        println!("{}", sample_config);
        return Ok(());
    }

    if show {
        println!("🔍 git-ai configuration status:");
        println!();

        // Check for repo-specific config
        let repo_config_path = std::path::PathBuf::from(".git-ai.yaml");
        if repo_config_path.exists() {
            println!("✅ Repository config: .git-ai.yaml");
        } else {
            println!("❌ Repository config: .git-ai.yaml (not found)");
        }

        // Check for user config
        if let Some(user_config_path) = config::Config::user_config_path() {
            if user_config_path.exists() {
                println!("✅ User config: {}", user_config_path.display());
            } else {
                println!("❌ User config: {} (not found)", user_config_path.display());
                if let Some(parent) = user_config_path.parent() {
                    if !parent.exists() {
                        println!("   💡 Create directory: mkdir -p {}", parent.display());
                    }
                }
            }
        } else {
            println!("❌ User config: Unable to determine config directory");
        }

        println!();
        println!(
            "💡 To create a sample config: git ai config --init > ~/.config/git-ai/config.yaml"
        );

        return Ok(());
    }

    // If no flags provided, show help
    println!("git-ai config management");
    println!();
    println!("Options:");
    println!("  --show  Show current configuration status");
    println!("  --init  Generate sample configuration");
    println!();
    println!("Examples:");
    println!("  git ai config --show");
    println!("  git ai config --init > ~/.config/git-ai/config.yaml");
    println!("  git ai config --init > .git-ai.yaml  # Repository-specific config");

    Ok(())
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
