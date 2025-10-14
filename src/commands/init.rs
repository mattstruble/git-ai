use crate::cli::args::InitArgs;
use crate::commands::Command;
use crate::config::InitConfig;
use crate::cursor_agent::CursorAgent;
use anyhow::Result;

/// AI-assisted project initialization prompt
const INIT_PROMPT: &str = r#"You are operating inside a command line interface (CLI) as an AI assistant integrated with Git via `cursor-agent`.

Your goal is to **initialize a new project repository** based on the user's chosen programming language and preferences.

## Task Overview

1. **Gather Requirements**
   - Ask the user for the target language (e.g. Python, JavaScript, Go, Rust, etc.)
   - Prompt for:
     - Project name
     - Package manager / environment tool (e.g. `uv`, `poetry`, `npm`, `pnpm`, `cargo`, etc.)
     - Whether to include a GitHub Actions workflow for CI/CD
     - Common tooling choices (e.g. linter, formatter, test framework)
     - Whether to initialize with a license, README, or contributing guide

2. **Generate Repository Structure**
   - Create a standard directory layout for the chosen language (e.g. `src/`, `tests/`, `docs/`, etc.)
   - Initialize `git` and create a `.gitignore` suited for the language.
   - Add a `README.md` with the project name and a short description.
   - If applicable, create an environment setup file:
     - For Python: `pyproject.toml` (supporting `uv` or `poetry`)
     - For JS: `package.json`
     - For Rust: `Cargo.toml`
     - For Go: `go.mod`

3. **Add Pre-commit Hooks**
   - Suggest and optionally configure pre-commit hooks to enforce code quality:
     - Example: `black`, `ruff`, or `mypy` for Python; `eslint` or `prettier` for JS.
   - If the user agrees, initialize `.pre-commit-config.yaml` or similar.

4. **Set Up GitHub Workflows (optional)**
   - If requested, scaffold a `.github/workflows/ci.yml` file appropriate for the language.
   - Include common actions like linting, testing, and build steps.

5. **Finalize and Commit**
   - Stage all files and make an initial commit:
     ```
     git add .
     git commit -m "Initialize new {language} project"
     ```
   - Provide a short summary of what was created and suggest the next steps (e.g. "Run `make test` to verify setup").

## Notes
- Always favor **sensible defaults** but confirm key decisions with the user before proceeding.
- Use official templates or community best practices for structure and configuration files.
- Ensure all files created are formatted correctly and validated for syntax.

When ready, proceed with creating the repository as described above."#;

/// Command for AI-assisted project initialization
pub struct InitCommand {
    config: InitConfig,
}

impl InitCommand {
    pub fn new(config: InitConfig) -> Self {
        Self { config }
    }
}

impl Command for InitCommand {
    type Args = InitArgs;
    type Config = InitConfig;

    fn prompt_template(&self) -> &str {
        // Use custom prompt from config if available, otherwise use built-in
        self.config.prompt.as_deref().unwrap_or(INIT_PROMPT)
    }

    fn resolve_args(&self, mut args: Self::Args) -> Self::Args {
        // Apply config overrides
        if let Some(no_confirm) = self.config.no_confirm {
            args.no_confirm = no_confirm;
        }
        args
    }

    async fn execute(&self, args: InitArgs, agent: &CursorAgent) -> Result<()> {
        let mut prompt = self.prompt_template().to_string();

        // Add language context if provided
        if let Some(ref language) = args.language {
            prompt = format!("{}\n\nTarget Language: {}", prompt, language);
        }

        // Add project name context if provided
        if let Some(ref name) = args.name {
            prompt = format!("{}\n\nProject Name: {}", prompt, name);
        }

        // Add user message if provided
        if let Some(ref message) = args.common.message {
            prompt = format!("{}\n\nUser Context: {}", prompt, message);
        }

        // Handle dry run
        if args.common.dry_run {
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
