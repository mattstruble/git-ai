use crate::cli::args::IgnoreArgs;
use crate::commands::Command;
use crate::config::IgnoreConfig;
use crate::cursor_agent::CursorAgent;
use anyhow::Result;

/// AI-assisted .gitignore management prompt
const IGNORE_PROMPT: &str = r#"You are operating inside a command line interface as an AI assistant integrated with Git via `cursor-agent`.

Your task is to manage entries in the project's `.gitignore` file.

---

## Capabilities

1. **Add or Remove Ignore Patterns**
   - Support adding or removing ignore patterns for specific **languages** or **tools**.
   - Examples include: `python`, `node`, `go`, `rust`, `java`, `macos`, `vscode`, `jetbrains`, etc.

2. **Create `.gitignore` if Missing**
   - If the file does not exist, initialize it before applying changes.

3. **Add Mode**
   - Fetch or generate best-practice ignore patterns for the requested language/tool.
   - Insert them in a clearly labeled section:
     ```
     # === Python ===
     __pycache__/
     *.py[cod]
     .env
     .venv/
     # === End Python ===
     ```
   - Avoid duplicates; skip adding a section if it already exists.

4. **Remove Mode**
   - Identify the relevant section(s) for the given language.
   - Cleanly delete the entire block, from the `# === <Language> ===` line to the corresponding end marker.
   - Leave unrelated sections untouched.

5. **After Modifications**
   - Display a diff or summary of the changes.
   - Optionally prompt the user:
     > "Would you like to commit these changes?"
   - If confirmed:
     ```
     git add .gitignore
     git commit -m "Update .gitignore for <language>"
     ```

---

## ⚙️ Guidelines

- Always maintain readability and comments within `.gitignore`.
- When inserting new sections, always use this pattern:
# === <Language> ===
<patterns>
# === End <Language> ===

- Allow multiple languages or tools to be managed at once.
- Confirm major actions interactively before applying changes
- Respect existing project structure and formatting"#;

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

    async fn execute(&self, args: IgnoreArgs, agent: &CursorAgent) -> Result<()> {
        let mut prompt = self.prompt_template().to_string();

        // Add action context
        prompt = format!("{}\n\nAction: {}", prompt, args.action);

        // Add languages context
        if !args.languages.is_empty() {
            let languages_str = args.languages.join(", ");
            prompt = format!("{}\n\nLanguages/Tools: {}", prompt, languages_str);
        }

        // Handle dry run
        if args.dry_run {
            println!(
                "🔍 Dry run mode - would execute with prompt:\n---\n{}\n---",
                prompt
            );
            return Ok(());
        }

        // Execute with cursor-agent
        agent.execute(&prompt, args.no_confirm).await
    }
}
