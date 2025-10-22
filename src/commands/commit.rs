use crate::cli::args::CommitArgs;
use crate::commands::Command;
use crate::config::CommitConfig;
use crate::context::{apply_context, ContextData, ContextManager, ContextType};
use crate::cursor_agent::CursorAgent;
use anyhow::Result;

/// Commit prompt template
pub const COMMIT_PROMPT: &str =
"You are operating in a command line interface, performing automated commit generation for a Git repository.

Your task:

1. **Analyze changes in the current Git repository.**
   - If there are staged files, only consider those.
   - If there are no staged files, consider all unstaged changes instead.
   - Use `git diff --cached` for staged changes, or `git diff` for unstaged changes.
   - Group related changes into small, logical commits that follow best practices for incremental commits.

2. **Generate commit messages following the Conventional Commits standard.**
   - Use the format: `<type>(<optional scope>): <short description>`
   - Keep each message concise and clear.
   - Limit the body to **at most two bullet points**, summarizing what and why the change was made.
   - Subject line under **72 characters**, written in **present tense**.
   - Focus on **what changed** and **why**, not how.

3. **Respect existing repository or app-level rules.**
   - If the repository or `cursor-agent` configuration defines custom commit rules or LLM behavior rules, those take **precedence** over this prompt.
   - Harmonize your output with any detected `.cursor-agent`, `.aiconfig`, or other LLM configuration files.

4. **Commit grouping guidance.**
   - Suggest logical groupings of files or changes to be committed together.
   - Recommend separate commits for distinct change types (e.g., `feat`, `fix`, `docs`, `refactor`).
   - Once you've created your recommended list of commits, execute them using `git commit`.

**Output Format Example:**
feat(api): add JWT authentication middleware
- implement token validation and route protection
- update user endpoints to require authentication

fix(ui): correct navbar alignment on mobile
- adjust CSS grid for better responsiveness
";

/// Commit command implementation
pub struct CommitCommand {
    config: CommitConfig,
}

impl CommitCommand {
    pub fn new(config: CommitConfig) -> Self {
        Self { config }
    }
}

impl Command for CommitCommand {
    type Args = CommitArgs;
    type Config = CommitConfig;

    fn prompt_template(&self) -> &str {
        // Use custom prompt from config, or default
        self.config.prompt.as_deref().unwrap_or(COMMIT_PROMPT)
    }

    fn resolve_args(&self, mut args: CommitArgs) -> CommitArgs {
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
        args: CommitArgs,
        agent: &CursorAgent,
        context_manager: &ContextManager,
    ) -> Result<()> {
        // Build base prompt with custom message if provided
        let base_prompt = if let Some(ref message) = args.common.message {
            format!("{}\n\nUser context: {}", self.prompt_template(), message)
        } else {
            self.prompt_template().to_string()
        };

        // Gather context and apply business logic
        let required_context = self.required_context();
        let context_bundle = context_manager
            .gather_context_with_command(&required_context, Some("commit".to_string()))
            .await?;

        // Check git context for staged files
        if let Some(ContextData::Git(git_context)) = context_bundle.get(ContextType::Git) {
            let staged_files = &git_context.repository_status.staged_files;
            let unstaged_files = &git_context.repository_status.unstaged_files;

            // Warn user if no staged files but there are unstaged files
            if staged_files.is_empty() && !unstaged_files.is_empty() {
                println!(
                    "\x1b[33m⚠️  No staged files found, but there are unstaged changes.\x1b[0m"
                );
                println!(
                    "\x1b[36m   Consider staging files first with: \x1b[1mgit add <files>\x1b[0m"
                );
                println!(
                    "\x1b[90m   Unstaged files ({}):\x1b[0m",
                    unstaged_files.len()
                );
                for file in unstaged_files.iter().take(5) {
                    println!(
                        "     \x1b[31m{}\x1b[0m \x1b[37m{}\x1b[0m",
                        file.status, file.path
                    );
                }
                if unstaged_files.len() > 5 {
                    println!(
                        "     \x1b[90m... and {} more\x1b[0m",
                        unstaged_files.len() - 5
                    );
                }
                println!();

                // Ask user if they want to continue
                if !args.no_confirm {
                    print!("\x1b[33mContinue with unstaged files? [y/N]: \x1b[0m");
                    use std::io::{self, Write};
                    io::stdout().flush()?;

                    let mut input = String::new();
                    io::stdin().read_line(&mut input)?;

                    if !input.trim().to_lowercase().starts_with('y') {
                        println!("\x1b[31mAborted. Stage files and try again.\x1b[0m");
                        return Ok(());
                    }
                }
            }

            // Warn if there are no changes at all
            if staged_files.is_empty()
                && unstaged_files.is_empty()
                && git_context.repository_status.untracked_files.is_empty()
            {
                println!("\x1b[34mℹ️  No changes detected in the repository.\x1b[0m");
                println!("\x1b[90m   Repository is clean - nothing to commit.\x1b[0m");
                return Ok(());
            }
        }

        // Apply context to prompt and execute
        let enhanced_prompt = apply_context(&base_prompt, &context_bundle)?;

        if args.common.dry_run {
            println!("🔍 Dry run mode - would execute with prompt:");
            println!("---");
            println!("{}", enhanced_prompt);
            println!("---");
            return Ok(());
        }

        agent.execute(&enhanced_prompt, args.no_confirm).await
    }
}
