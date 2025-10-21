pub mod commit;
pub mod config;
pub mod ignore;
pub mod init;
pub mod merge;
pub mod pr;

pub use commit::CommitCommand;
pub use config::ConfigCommand;
pub use ignore::IgnoreCommand;
pub use init::InitCommand;
pub use merge::MergeCommand;
pub use pr::PrCommand;

use crate::context::{ContextManager, ContextType};
use crate::cursor_agent::CursorAgent;
use anyhow::Result;

/// Base trait for all commands
pub trait Command {
    type Args;
    type Config;

    /// Get the prompt template for this command
    fn prompt_template(&self) -> &str;

    /// Apply config overrides to CLI arguments
    fn resolve_args(&self, args: Self::Args) -> Self::Args;

    /// Specify which context types this command requires
    fn required_context(&self) -> Vec<ContextType>;

    /// Execute the command with resolved arguments and context manager
    async fn execute(
        &self,
        args: Self::Args,
        agent: &CursorAgent,
        context_manager: &ContextManager,
    ) -> Result<()>;
}
