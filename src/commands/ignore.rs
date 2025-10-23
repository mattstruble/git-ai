use crate::cli::args::IgnoreArgs;
use crate::commands::Command;
use crate::config::IgnoreConfig;
use crate::context::{apply_context, ContextManager, ContextType};
use crate::cursor_agent::CursorAgent;
use anyhow::Result;

/// AI-assisted .gitignore management prompt
const IGNORE_PROMPT: &str =
"You are an AI assistant operating in a Git-enabled command-line environment.
You have permission to safely modify files and execute Git commands.

Your task is to **manage entries in the project's `.gitignore` file**,
based on user input, repository structure, and detected project context.

---

### 🧭 **Your Role**
You are responsible for:
- Detecting project languages and tools.
- Ensuring `.gitignore` follows best practices for those technologies.
- Adding or removing language-specific sections in a readable, structured format.
- Explaining what changes you make and why.

---

### 🧩 **Context Data**
You have access to structured repository information that includes:
- **Repository** → files, languages, and project layout.
- **Project** → configuration files, conventions, and frameworks.
- **Agent** → allowed commands and safety policies.

Use these contexts to infer appropriate ignore patterns
(e.g., `node_modules/` for JavaScript, `target/` for Rust, `__pycache__/` for Python, `.vscode/` for VSCode).

---

### ⚙️ **Capabilities**

1. **Detect or Create `.gitignore`**
   - If `.gitignore` does not exist, create one at the repository root.
   - Always add a header comment:
     ```
     # Managed by git-ai
     # Manual edits are preserved between AI-managed sections.
     ```

2. **Add Ignore Sections**
   - Use the format:
     ```
     # === <Language or Tool> ===
     <patterns>
     # === End <Language or Tool> ===
     ```
   - Avoid duplicates — skip adding a section if it already exists.
   - Pull patterns from known best practices (GitHub templates, language conventions, or inferred from context).
   - Ensure clarity: one section per language or tool.

3. **Remove Ignore Sections**
   - Identify a matching section header by name (e.g., `# === Python ===`).
   - Cleanly remove from the start marker to its corresponding end marker.
   - Preserve all other content.

4. **Update Mode**
   - If a section exists but is outdated, replace only that block with an updated template.

5. **Summarize or Commit Changes**
   - After modification, show a unified diff of the `.gitignore` changes.
   - Then ask interactively:
     > “Would you like to commit these changes?”
   - If confirmed:
     ```
     git add .gitignore
     ```

---

### 🧱 **Output Format**

When running non-interactively, output the following structured sections:

#### Summary
List what was added, updated, or removed.

#### Diff Preview
Show the unified diff of `.gitignore` changes (if any).

#### Suggested Commit Message
`chore: update .gitignore for <language(s)>`

#### Next Steps
If user confirmation is needed or conflicts exist, clearly state them.

---
### 🧰 **Execution Rules**
You can execute safe Git and file commands:
- Read/write `.gitignore`
- `git status`, `git add .gitignore`, `git diff .gitignore`
- `git commit -m '<message>'`

You must **not** run destructive shell commands or modify unrelated files.
If a language or tool is unknown, state that and suggest manual confirmation.

---

### ⚙️ **Context**
Below is the structured repository context for reasoning about project type and configuration.
";
/// Command for AI-assisted .gitignore management
pub struct IgnoreCommand {
    config: IgnoreConfig,
}

impl IgnoreCommand {
    pub fn new(config: IgnoreConfig) -> Self {
        Self { config }
    }
}

impl Command for IgnoreCommand {
    type Args = IgnoreArgs;
    type Config = IgnoreConfig;

    fn prompt_template(&self) -> &str {
        // Use custom prompt from config if available, otherwise use built-in
        self.config.prompt.as_deref().unwrap_or(IGNORE_PROMPT)
    }

    fn resolve_args(&self, mut args: Self::Args) -> Self::Args {
        // Apply config overrides
        if let Some(no_confirm) = self.config.no_confirm {
            args.no_confirm = no_confirm;
        }
        args
    }

    fn required_context(&self) -> Vec<ContextType> {
        vec![
            ContextType::Project,
            ContextType::Agent,
            ContextType::Interaction,
        ]
    }

    async fn execute(
        &self,
        args: IgnoreArgs,
        agent: &CursorAgent,
        context_manager: &ContextManager,
    ) -> Result<()> {
        let mut prompt = self.prompt_template().to_string();

        // Add action context
        prompt = format!("{}\n\nAction: {}", prompt, args.action);

        // Add languages context
        if !args.languages.is_empty() {
            let languages_str = args.languages.join(", ");
            prompt = format!("{}\n\nLanguages/Tools: {}", prompt, languages_str);
        }

        // Gather context and apply to prompt
        // Get context types from configuration or use defaults
        let context_types = self.configured_context(self.config.context.as_ref());
        let context_bundle = context_manager
            .gather_context_with_command(&context_types, Some("ignore".to_string()))
            .await?;
        let enhanced_prompt = apply_context(&prompt, &context_bundle)?;

        // Handle dry run
        if args.dry_run {
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
