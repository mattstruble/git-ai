use crate::cli::args::MergeArgs;
use crate::commands::Command;
use crate::config::MergeConfig;
use crate::context::{apply_context, ContextManager, ContextType};
use crate::cursor_agent::CursorAgent;
use anyhow::Result;

/// Merge prompt template
pub const MERGE_PROMPT: &str =
    "You are an expert software developer tasked with analyzing and assisting with merging the branch '{}' into the current branch.

**Your Task**:
1. **Analyze Branch Differences**: Examine what changes exist in '{}' that aren't in the current branch
2. **Check for Conflicts**: Determine if there are any merge conflicts and their nature
3. **Provide Guidance**: Based on the git status and changes, provide appropriate guidance

**If There Are Merge Conflicts**:
- Explain what caused the conflicts between the branches
- Identify the specific files and areas of conflict
- Suggest a resolution strategy for each conflict
- Provide step-by-step guidance for resolving conflicts
- Recommend an appropriate merge commit message after resolution

**If No Conflicts (Clean Merge)**:
- Summarize what changes from '{}' will be integrated
- Highlight key features, fixes, or modifications being brought in
- Generate an appropriate merge commit message following the format: 'Merge branch {}'
- Explain the impact and value of these changes to the codebase

**For Merge Commit Messages**:
- Use standard format: 'Merge branch {}' or 'Merge branch {} into current-branch'
- Include a brief description of what '{}' brings to the codebase
- Mention any significant features, fixes, or changes
- Keep it concise but informative

**General Guidance**:
- Review the current git status carefully
- Consider the branch's purpose and changes
- Provide clear, actionable next steps
- Warn about any potential breaking changes or impacts

Analyze the current repository state and provide comprehensive merge guidance for integrating '{}'.";

/// Merge command implementation
pub struct MergeCommand {
    config: MergeConfig,
}

impl MergeCommand {
    pub fn new(config: MergeConfig) -> Self {
        Self { config }
    }
}

impl Command for MergeCommand {
    type Args = MergeArgs;
    type Config = MergeConfig;

    fn prompt_template(&self) -> &str {
        // Use custom prompt from config, or default
        self.config.prompt.as_deref().unwrap_or(MERGE_PROMPT)
    }

    fn resolve_args(&self, mut args: MergeArgs) -> MergeArgs {
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
        args: MergeArgs,
        agent: &CursorAgent,
        context_manager: &ContextManager,
    ) -> Result<()> {
        // Build base prompt with branch substitution and custom message
        let mut prompt = self.prompt_template().replace("{}", &args.branch);

        if let Some(ref message) = args.common.message {
            prompt = format!("{}\n\nUser context: {}", prompt, message);
        }

        // Gather context and apply to prompt
        let required_context = self.required_context();
        let context_bundle = context_manager
            .gather_context_with_command(&required_context, Some("merge".to_string()))
            .await?;
        let enhanced_prompt = apply_context(&prompt, &context_bundle)?;

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
