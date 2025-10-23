use crate::cli::args::PrArgs;
use crate::commands::Command;
use crate::config::PrConfig;
use crate::context::{apply_context, ContextManager, ContextType};
use crate::cursor_agent::CursorAgent;
use anyhow::Result;

/// PR prompt template
pub const PR_PROMPT: &str =
"You are an expert software engineer and release maintainer.
You are generating a **comprehensive pull request description** for a Git-based project.

---

### 🎯 **Your Task**
Analyze the provided Git and Project context to produce a high-quality PR description that:
1. Explains the purpose and impact of the proposed changes.
2. Lists the major technical modifications.
3. Highlights **breaking changes**, **new features**, and **fixes** where relevant.
4. Reflects project-specific commit and release conventions.

---

### 🧩 **How to Use Context**
Use:
- `Git` → to understand branch diffs, commits, and affected files.
- `Project` → to infer breaking change indicators, versioning rules, and commit conventions.
- `Repository` → to understand the structure and type of project.
- `Interaction` → to understand current command metadata and CLI state.

If the Project context defines `breaking_changes.indicators`, explicitly check for those in diffs or commit messages and flag them.

---

### 🧱 **Output Format (Markdown)**
Produce the PR description as Markdown, using this structure:

#### Summary
A 2–3 sentence explanation of what this PR accomplishes and why it’s valuable.

#### Changes
A bulleted list of key changes and affected components or files.

#### Why
Explain the motivation or issue this PR addresses.

#### Impact / Breaking Changes
Mention any user-facing or API-level breaking changes. Use a `⚠️` emoji or a bold “Breaking Change” note.

#### Testing
Briefly describe how the changes were tested (unit tests, CI, manual verification).

#### References
List any related issues, tickets, or changelog sections.

---

### 🧭 **Style Guidelines**
- Use concise and professional language.
- Follow Conventional Commit and semantic versioning hints from Project context.
- Use Markdown headings and lists.
- Prefer technical precision over verbosity.

---

### ⚙️ **Context**
Below is the contextual JSON data from the repository. Use it to reason about code, project rules, and versioning conventions.

";
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
