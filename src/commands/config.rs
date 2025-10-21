use crate::cli::args::ConfigArgs;
use crate::commands::Command;
use crate::config::Config;
use crate::context::{ContextManager, ContextType};
use crate::cursor_agent::CursorAgent;
use anyhow::Result;
use std::path::PathBuf;

/// Config command implementation (no prompt needed)
pub struct ConfigCommand;

impl ConfigCommand {
    pub fn new() -> Self {
        Self
    }
}

impl Command for ConfigCommand {
    type Args = ConfigArgs;
    type Config = (); // Config command doesn't need config

    fn prompt_template(&self) -> &str {
        "" // No prompt for config command
    }

    fn resolve_args(&self, args: ConfigArgs) -> ConfigArgs {
        // No overrides for config command
        args
    }

    fn required_context(&self) -> Vec<ContextType> {
        // Config command doesn't need any context
        vec![]
    }

    async fn execute(
        &self,
        args: ConfigArgs,
        _agent: &CursorAgent,
        _context_manager: &ContextManager,
    ) -> Result<()> {
        // Config command doesn't need cursor-agent or context
        self.handle_config(args.show, args.init)
    }
}

impl ConfigCommand {
    /// Handle the config command logic
    fn handle_config(&self, show: bool, init: bool) -> Result<()> {
        if init {
            let sample_config = Config::create_sample_config()?;
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
            let repo_config_path = PathBuf::from(".git-ai.yaml");
            if repo_config_path.exists() {
                println!("✅ Repository config: .git-ai.yaml");
            } else {
                println!("❌ Repository config: .git-ai.yaml (not found)");
            }

            // Check for user config
            if let Some(user_config_path) = Config::user_config_path() {
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
}
