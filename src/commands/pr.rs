use crate::cli::args::PrArgs;
use crate::commands::Command;
use crate::config::PrConfig;
use crate::context::{apply_context, ContextManager, ContextType};
use crate::cursor_agent::CursorAgent;
use anyhow::Result;

/// PR prompt template
pub const PR_PROMPT: &str =
    "You are an expert software developer creating a comprehensive pull request description.

Analyze the git changes between the current branch and the target branch (typically main/master), then create a professional PR description.

**Your Task**:
1. **Examine Changes**: Review the git diff between branches to understand what changed
2. **Analyze Impact**: Determine the scope and significance of the changes
3. **Generate Description**: Create a well-structured PR description in Markdown format

**Required Structure**:
- **Summary**: Brief, clear overview of what this PR accomplishes
- **Changes**: Bulleted list of key modifications, features, or fixes
- **Why**: Explanation of the motivation, problem solved, or requirement fulfilled
- **Testing**: Description of how changes were tested (unit tests, manual testing, etc.)
- **Notes**: Any important considerations, breaking changes, or context for reviewers

**Style Guidelines**:
- Use clean Markdown formatting with proper headings
- Be professional yet concise
- Focus on the business value and technical impact
- Include any relevant issue numbers or references
- Highlight breaking changes or migration steps if applicable

Create a description that helps reviewers understand the context, changes, and impact of this pull request.";

/// PR command implementation
pub struct PrCommand {
    config: PrConfig,
}

impl PrCommand {
    pub fn new(config: PrConfig) -> Self {
        Self { config }
    }
}

impl Command for PrCommand {
    type Args = PrArgs;
    type Config = PrConfig;

    fn prompt_template(&self) -> &str {
        // Use custom prompt from config, or default
        self.config.prompt.as_deref().unwrap_or(PR_PROMPT)
    }

    fn resolve_args(&self, mut args: PrArgs) -> PrArgs {
        // Apply config overrides to args
        if let Some(no_confirm) = self.config.no_confirm {
            if !args.no_confirm {
                // Only override if not explicitly set by CLI
                args.no_confirm = no_confirm;
            }
        }
        args
    }

    fn required_context(&self) -> Vec<ContextType> {
        vec![
            ContextType::Git,
            ContextType::Agent,
            ContextType::Interaction,
        ]
    }

    async fn execute(
        &self,
        args: PrArgs,
        agent: &CursorAgent,
        context_manager: &ContextManager,
    ) -> Result<()> {
        // Build base prompt with custom message if provided
        let base_prompt = if let Some(ref message) = args.common.message {
            format!("{}\n\nUser context: {}", self.prompt_template(), message)
        } else {
            self.prompt_template().to_string()
        };

        // Gather context and apply to prompt
        // Get context types from configuration or use defaults
        let context_types = self.configured_context(self.config.context.as_ref());
        let context_bundle = context_manager
            .gather_context_with_command(&context_types, Some("pr".to_string()))
            .await?;
        let enhanced_prompt = apply_context(&base_prompt, &context_bundle)?;

        if args.common.dry_run {
            println!("🔍 Dry run mode - would execute with prompt:");
            println!("---");
            println!("{}", enhanced_prompt);
            println!("---");
            return Ok(());
        }

        // Execute with cursor-agent
        agent.execute(&enhanced_prompt, args.no_confirm).await
    }
}
