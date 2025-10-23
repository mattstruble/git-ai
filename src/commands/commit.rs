use crate::cli::args::CommitArgs;
use crate::commands::Command;
use crate::config::CommitConfig;
use crate::context::{apply_context, ContextData, ContextManager, ContextType};
use crate::cursor_agent::CursorAgent;
use anyhow::Result;

/// Commit prompt template
pub const COMMIT_PROMPT: &str =
"You are an expert software engineer and commit author operating within a Git-based project.
You are operating in a command line interface, performing automated commit generation for a Git repository.

---

### 🧭 **Your Role**
Analyze the contextual information from the repository, recent diffs, and project conventions to produce one or more **atomic, high-quality commit messages**.

---

### 🎯 **Objectives**
1. **Analyze the provided Git context.**
   - Prioritize staged changes.
   - If none are staged, fall back to unstaged diffs.
   - Each diff group represents a logical commit candidate.

2. **Generate commit messages following Conventional Commits.**
   - Format: `<type>(<optional scope>): <short description>`
   - Example types: `feat`, `fix`, `docs`, `refactor`, `test`, `chore`.
   - Keep the subject line under 72 characters, written in **present tense**.
   - Include a short **body** (1–3 concise bullet points) summarizing “what” and “why.”

3. **Incorporate project-specific rules from context.**
   - Use commit style rules from `Project.conventions.commit_style`.
   - If `Project.breaking_changes.rules` or `Project.breaking_changes.indicators` exist,
     mark commits containing such changes with a `BREAKING CHANGE:` footer or a `!` after the type.
   - Respect any repository-defined scopes (e.g., `core`, `ui`, `docs`) found in directory names or conventions.

4. **Respect grouping conventions.**
   - Keep commits small and logical.
   - Separate different change types (e.g., features, fixes, docs) into separate messages if necessary.

5. **Tone and structure.**
   - Prefer technical precision and clarity.
   - Avoid generic messages like “update code” or “misc changes.”

6. **Execution**
    - Once you've created your recommended list of commits, execute them using `git commit`.

---

### 🧩 **How to Use Context**
Use:
- `Git` → for diffs, file names, and commit history patterns.
- `Project` → for commit style, breaking change indicators, and language/framework hints.
- `Repository` → for directory names and file organization to infer scopes.
- `Agent` → for allowed operations or enforced templates.

If relevant, summarize detected change patterns (e.g., documentation-only, code refactor, dependency updates).

---

### ⚙️ **Context Data**
Below is structured repository context information that describes the project’s conventions, recent diffs, and repo metadata.

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
            ContextType::Project,
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

        // Get context types from configuration or use defaults
        let context_types = self.configured_context(self.config.context.as_ref());
        let context_bundle = context_manager
            .gather_context_with_command(&context_types, Some("commit".to_string()))
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
