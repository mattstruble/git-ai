pub mod args;

use crate::commands::{
    Command, CommitCommand, ConfigCommand, InitCommand, MergeCommand, PrCommand,
};
use crate::config::Config;
use crate::cursor_agent::CursorAgent;
use crate::Commands;
use anyhow::Result;
use args::{CommitArgs, CommonArgs, ConfigArgs, InitArgs, MergeArgs, PrArgs};

/// Command dispatcher that routes CLI commands to their implementations
pub struct CommandDispatcher {
    config: Config,
    agent: CursorAgent,
}

impl CommandDispatcher {
    pub fn new(config: Config) -> Self {
        Self {
            config,
            agent: CursorAgent::new(),
        }
    }

    pub async fn dispatch(&self, command: Commands) -> Result<()> {
        match command {
            Commands::Commit {
                message,
                no_confirm,
                dry_run,
                verbose,
            } => {
                let args = CommitArgs {
                    common: CommonArgs {
                        dry_run,
                        verbose,
                        message,
                    },
                    no_confirm,
                };
                let cmd = CommitCommand::new(self.config.commands.commit.clone());
                let resolved_args = cmd.resolve_args(args);
                cmd.execute(resolved_args, &self.agent).await
            }
            Commands::Pr {
                message,
                no_confirm,
                dry_run,
                verbose,
            } => {
                let args = PrArgs {
                    common: CommonArgs {
                        dry_run,
                        verbose,
                        message,
                    },
                    no_confirm,
                };
                let cmd = PrCommand::new(self.config.commands.pr.clone());
                let resolved_args = cmd.resolve_args(args);
                cmd.execute(resolved_args, &self.agent).await
            }
            Commands::Merge {
                branch,
                message,
                no_confirm,
                dry_run,
                verbose,
            } => {
                let args = MergeArgs {
                    common: CommonArgs {
                        dry_run,
                        verbose,
                        message,
                    },
                    branch,
                    no_confirm,
                };
                let cmd = MergeCommand::new(self.config.commands.merge.clone());
                let resolved_args = cmd.resolve_args(args);
                cmd.execute(resolved_args, &self.agent).await
            }
            Commands::Config { show, init } => {
                let args = ConfigArgs { show, init };
                let cmd = ConfigCommand::new();
                cmd.execute(args, &self.agent).await
            }
            Commands::Init {
                language,
                name,
                message,
                no_confirm,
                dry_run,
                verbose,
            } => {
                let args = InitArgs {
                    common: CommonArgs {
                        dry_run,
                        verbose,
                        message,
                    },
                    language,
                    name,
                    no_confirm,
                };
                let cmd = InitCommand::new(self.config.commands.init.clone());
                let resolved_args = cmd.resolve_args(args);
                cmd.execute(resolved_args, &self.agent).await
            }
        }
    }
}
