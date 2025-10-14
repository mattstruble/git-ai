mod cli;
mod config;
mod prompts;

use anyhow::{Context, Result};
use clap::{CommandFactory, Parser, Subcommand};
use clap_complete::{generate, Shell};
use std::io::Write;
use std::process::{Command as StdCommand, Stdio};

const CURSOR_INSTALL_URL: &str = "https://cursor.com/install";
const CURSOR_INSTALL_CHECKSUM: &str = ""; // TODO: Add actual checksum when available

#[derive(Parser)]
#[command(name = "git-ai")]
#[command(about = "AI-assisted git workflow with cursor-agent")]
#[command(version = "0.4.1")]
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
    /// Generate shell completions
    Completions {
        /// The shell to generate completions for
        #[arg(value_enum)]
        shell: Shell,
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
    match &cli.command {
        Commands::Completions { shell } => {
            let mut cmd = Cli::command();
            generate(*shell, &mut cmd, "git-ai", &mut std::io::stdout());
            return Ok(());
        }
        Commands::Config { show, init } => {
            return handle_config_command(*show, *init);
        }
        _ => {} // Continue with normal processing
    }

    // Load configuration
    let config = config::Config::load()?;

    let (no_confirm, dry_run, verbose) = match &cli.command {
        Commands::Commit {
            no_confirm,
            dry_run,
            verbose,
            ..
        } => (*no_confirm, *dry_run, *verbose),
        Commands::Pr {
            no_confirm,
            dry_run,
            verbose,
            ..
        } => (*no_confirm, *dry_run, *verbose),
        Commands::Merge {
            no_confirm,
            dry_run,
            verbose,
            ..
        } => (*no_confirm, *dry_run, *verbose),
        Commands::Completions { .. } | Commands::Config { .. } => unreachable!("Handled above"),
    };

    // Override CLI flags with config values where appropriate
    let effective_verbose = verbose || config.behavior.verbose;
    let effective_no_confirm = no_confirm || !config.behavior.confirm_cursor_agent_install;
    ensure_cursor_agent(false, effective_no_confirm, effective_verbose).await?;

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

    cli::execute_command(&cli.command, &config, effective_no_confirm).await?;

    Ok(())
}

/// Ensure cursor-agent is installed with security measures
async fn ensure_cursor_agent(force_reinstall: bool, no_confirm: bool, verbose: bool) -> Result<()> {
    // Check if cursor-agent exists and reinstall is not forced
    if !force_reinstall {
        if let Ok(output) = StdCommand::new("cursor-agent").arg("--version").output() {
            if output.status.success() {
                if verbose {
                    let version = String::from_utf8_lossy(&output.stdout);
                    println!("✅ cursor-agent already installed: {}", version.trim());
                }
                return Ok(());
            }
        }
    }

    if verbose {
        println!("🔍 cursor-agent not found or reinstall requested");
    }

    // Security warning and user consent
    if !no_confirm {
        println!("⚠️  SECURITY WARNING: cursor-agent installation");
        println!(
            "   This will download and execute a script from: {}",
            CURSOR_INSTALL_URL
        );
        println!("   The script will be executed with bash and may modify your system.");
        println!("   Please review the script at the URL above before proceeding.");
        println!();
        print!("   Do you want to continue with the installation? [y/N]: ");
        std::io::stdout().flush()?;

        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;

        let response = input.trim().to_lowercase();
        if response != "y" && response != "yes" {
            anyhow::bail!("cursor-agent installation cancelled by user");
        }
    }

    println!("⚙️  Downloading cursor-agent installer...");

    // Download the install script with better error handling
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .context("Failed to create HTTP client")?;

    let response = client
        .get(CURSOR_INSTALL_URL)
        .send()
        .await
        .context("Failed to download cursor-agent installer")?;

    if !response.status().is_success() {
        anyhow::bail!("Failed to download installer: HTTP {}", response.status());
    }

    let script_content = response
        .text()
        .await
        .context("Failed to read installer script")?;

    // Basic validation - ensure we got a script
    if script_content.is_empty() {
        anyhow::bail!("Downloaded script is empty");
    }

    if !script_content.trim_start().starts_with("#!/") {
        anyhow::bail!("Downloaded content doesn't appear to be a valid shell script");
    }

    // TODO: Add checksum verification when checksum is available
    #[allow(clippy::const_is_empty)]
    if !CURSOR_INSTALL_CHECKSUM.is_empty() {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        script_content.hash(&mut hasher);
        let computed_hash = format!("{:x}", hasher.finish());

        if computed_hash != CURSOR_INSTALL_CHECKSUM {
            anyhow::bail!(
                "Checksum verification failed! Expected: {}, Got: {}",
                CURSOR_INSTALL_CHECKSUM,
                computed_hash
            );
        }

        if verbose {
            println!("✅ Checksum verification passed");
        }
    } else if verbose {
        println!("⚠️  Checksum verification skipped (checksum not available)");
    }

    if verbose {
        println!("🔧 Executing installer script...");
    }

    // Execute the install script with better process handling
    let mut child = StdCommand::new("bash")
        .arg("-s") // Read script from stdin
        .stdin(Stdio::piped())
        .stdout(if verbose {
            Stdio::inherit()
        } else {
            Stdio::piped()
        })
        .stderr(Stdio::inherit())
        .spawn()
        .context("Failed to start bash process for installation")?;

    let stdin = child.stdin.as_mut().context("Failed to get stdin handle")?;
    stdin
        .write_all(script_content.as_bytes())
        .context("Failed to write install script to bash")?;

    // Close stdin to signal end of input
    drop(child.stdin.take());

    let output = child
        .wait_with_output()
        .context("Failed to wait for installation")?;

    if output.status.success() {
        println!("✅ cursor-agent installed successfully");

        // Verify the installation worked
        if let Ok(version_output) = StdCommand::new("cursor-agent").arg("--version").output() {
            if version_output.status.success() {
                let version = String::from_utf8_lossy(&version_output.stdout);
                println!("📦 Installed version: {}", version.trim());
            }
        }
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("cursor-agent installation failed: {}", stderr);
    }

    Ok(())
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
    fn test_cli_version() {
        let cli = Cli::command();
        let version = cli.get_version().unwrap();
        assert_eq!(version, "0.1.0");
    }

    #[test]
    fn test_cli_name() {
        let cli = Cli::command();
        let name = cli.get_name();
        assert_eq!(name, "git-ai");
    }

    #[test]
    fn test_message_and_no_confirm_extraction_commit() {
        let cli_command = Commands::Commit {
            message: Some("test".to_string()),
            no_confirm: true,
            dry_run: false,
            verbose: false,
        };

        let (message, no_confirm, dry_run, verbose) = match &cli_command {
            Commands::Commit {
                message,
                no_confirm,
                dry_run,
                verbose,
            } => (message.clone(), *no_confirm, *dry_run, *verbose),
            Commands::Pr {
                message,
                no_confirm,
                dry_run,
                verbose,
            } => (message.clone(), *no_confirm, *dry_run, *verbose),
            Commands::Merge {
                message,
                no_confirm,
                dry_run,
                verbose,
                ..
            } => (message.clone(), *no_confirm, *dry_run, *verbose),
            _ => unreachable!(),
        };

        assert_eq!(message, Some("test".to_string()));
        assert!(no_confirm);
        assert!(!dry_run);
        assert!(!verbose);
    }

    #[test]
    fn test_message_and_no_confirm_extraction_pr() {
        let cli_command = Commands::Pr {
            message: None,
            no_confirm: false,
            dry_run: false,
            verbose: false,
        };

        let (message, no_confirm, dry_run, verbose) = match &cli_command {
            Commands::Commit {
                message,
                no_confirm,
                dry_run,
                verbose,
            } => (message.clone(), *no_confirm, *dry_run, *verbose),
            Commands::Pr {
                message,
                no_confirm,
                dry_run,
                verbose,
            } => (message.clone(), *no_confirm, *dry_run, *verbose),
            Commands::Merge {
                message,
                no_confirm,
                dry_run,
                verbose,
                ..
            } => (message.clone(), *no_confirm, *dry_run, *verbose),
            _ => unreachable!(),
        };

        assert_eq!(message, None);
        assert!(!no_confirm);
        assert!(!dry_run);
        assert!(!verbose);
    }

    #[test]
    fn test_message_and_no_confirm_extraction_merge() {
        let cli_command = Commands::Merge {
            branch: "develop".to_string(),
            message: Some("merge develop".to_string()),
            no_confirm: true,
            dry_run: false,
            verbose: false,
        };

        let (message, no_confirm, dry_run, verbose) = match &cli_command {
            Commands::Commit {
                message,
                no_confirm,
                dry_run,
                verbose,
            } => (message.clone(), *no_confirm, *dry_run, *verbose),
            Commands::Pr {
                message,
                no_confirm,
                dry_run,
                verbose,
            } => (message.clone(), *no_confirm, *dry_run, *verbose),
            Commands::Merge {
                message,
                no_confirm,
                dry_run,
                verbose,
                ..
            } => (message.clone(), *no_confirm, *dry_run, *verbose),
            _ => unreachable!(),
        };

        assert_eq!(message, Some("merge develop".to_string()));
        assert!(no_confirm);
        assert!(!dry_run);
        assert!(!verbose);
    }

    // Note: Testing ensure_cursor_agent would require mocking
    // std::process::Command and reqwest::Client, which is complex but doable with
    // dependency injection or test-specific implementations

    #[test]
    fn test_cursor_install_url_is_valid() {
        // Basic validation that the URL is well-formed
        assert!(CURSOR_INSTALL_URL.starts_with("https://"));
        assert!(CURSOR_INSTALL_URL.contains("cursor.com"));
    }
}
