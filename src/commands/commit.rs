use crate::cli::args::CommitArgs;
use crate::commands::Command;
use crate::config::CommitConfig;
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

    async fn execute(&self, args: CommitArgs, agent: &CursorAgent) -> Result<()> {
        // Use the template with custom message if provided
        let mut prompt = self.prompt_template().to_string();

        if let Some(ref message) = args.common.message {
            prompt = format!("{}\n\nUser context: {}", prompt, message);
        }

        if args.common.dry_run {
            println!("🔍 Dry run mode - would execute with prompt:");
            println!("---");
            println!("{}", prompt);
            println!("---");
            return Ok(());
        }

        // Use shared cursor-agent service
        agent.execute(&prompt, args.no_confirm).await
    }
}
