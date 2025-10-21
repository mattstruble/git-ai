pub mod args;

use crate::commands::{
    Command, CommitCommand, ConfigCommand, IgnoreCommand, InitCommand, MergeCommand, PrCommand,
};
use crate::config::Config;
use crate::context::ContextManager;
use crate::cursor_agent::CursorAgent;
use crate::{Commands, IgnoreAction};
use anyhow::Result;
use args::{CommitArgs, CommonArgs, ConfigArgs, IgnoreArgs, InitArgs, MergeArgs, PrArgs};

/// Command dispatcher that routes CLI commands to their implementations
pub struct CommandDispatcher {
    config: Config,
    agent: CursorAgent,
    context_manager: ContextManager,
}

impl CommandDispatcher {
    pub fn new(config: Config) -> Result<Self> {
        let context_manager = ContextManager::new()?;
        Ok(Self {
            config,
            agent: CursorAgent::new(),
            context_manager,
        })
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
                        message: message.clone(),
                    },
                    no_confirm,
                };
                let cmd = CommitCommand::new(self.config.commands.commit.clone());
                let resolved_args = cmd.resolve_args(args);

                cmd.execute(resolved_args, &self.agent, &self.context_manager)
                    .await
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
                        message: message.clone(),
                    },
                    no_confirm,
                };
                let cmd = PrCommand::new(self.config.commands.pr.clone());
                let resolved_args = cmd.resolve_args(args);

                cmd.execute(resolved_args, &self.agent, &self.context_manager)
                    .await
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
                        message: message.clone(),
                    },
                    branch: branch.clone(),
                    no_confirm,
                };
                let cmd = MergeCommand::new(self.config.commands.merge.clone());
                let resolved_args = cmd.resolve_args(args);

                cmd.execute(resolved_args, &self.agent, &self.context_manager)
                    .await
            }
            Commands::Config { show, init } => {
                let args = ConfigArgs { show, init };
                let cmd = ConfigCommand::new();
                cmd.execute(args, &self.agent, &self.context_manager).await
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
                        message: message.clone(),
                    },
                    language: language.clone(),
                    name: name.clone(),
                    no_confirm,
                };
                let cmd = InitCommand::new(self.config.commands.init.clone());
                let resolved_args = cmd.resolve_args(args);

                cmd.execute(resolved_args, &self.agent, &self.context_manager)
                    .await
            }
            Commands::Ignore { action } => {
                let (action_str, languages, no_confirm, dry_run, verbose) = match action {
                    IgnoreAction::Add {
                        languages,
                        no_confirm,
                        dry_run,
                        verbose,
                    } => ("add", languages, no_confirm, dry_run, verbose),
                    IgnoreAction::Remove {
                        languages,
                        no_confirm,
                        dry_run,
                        verbose,
                    } => ("remove", languages, no_confirm, dry_run, verbose),
                };

                let args = IgnoreArgs {
                    action: action_str.to_string(),
                    languages: languages.clone(),
                    no_confirm,
                    dry_run,
                    verbose,
                };
                let cmd = IgnoreCommand::new(self.config.commands.ignore.clone());
                let resolved_args = cmd.resolve_args(args);

                cmd.execute(resolved_args, &self.agent, &self.context_manager)
                    .await
            }
        }
    }
}
