{
"project": {
"name": "git-ai",
"description": "AI-assisted git workflow with cursor-agent integration",
"primary_language": "Rust",
"keywords": ["git", "ai", "cli", "cursor-agent", "workflow", "automation"],
"usage": {
"commands": [
"git ai commit",
"git ai pr",
"git ai merge <branch>",
"git ai init",
"git ai ignore add <language>",
"git ai config --show"
],
"examples": [
"git ai commit -m \"Focus on security improvements\"",
"git ai pr --dry-run",
"git ai init --language python --name myproject"
],
"configuration": [
".git-ai.yaml",
"~/.config/git-ai/config.yaml",
"--dry-run",
"--verbose",
"--no-confirm"
]
},
"development": {
"setup_steps": [
"cargo build --release",
"nix profile install github:mattstruble/git-ai",
"brew tap mattstruble/formulae && brew install git-ai"
],
"dependencies": [
"clap",
"async-trait",
"reqwest",
"serde",
"cursor-agent"
],
"testing": [
"cargo test",
"nix build"
]
},
"breaking_changes": {
"rules": [
"Adheres to Semantic Versioning (semver.org)",
"Breaking changes documented in CHANGELOG.md",
"BREAKING prefix used for incompatible changes"
],
"indicators": [
"BREAKING:",
"Major version increment",
"Changed flag behavior",
"API incompatibility"
]
},
"conventions": {
"commit_style": "Conventional commits with cocogitto automation, format: type(scope): description",
"branching": "feat/_ for features, origin/feat/context example",
"release_process": "Cocogitto hooks, automated changelog generation, GitHub Actions release workflow",
"docs_reference": "README.md, CHANGELOG.md, LICENSE"
},
"misc": {
"notable_designs": [
"Command-based architecture",
"Git plugin integration",
"Cursor-agent dependency for AI",
"Context providers system"
],
"important_files": [
"src/commands/_.rs",
"src/cursor_agent.rs",
"src/context/providers/\*.rs",
"config/default_config.yaml"
],
"security_notes": [
"User confirmation required for cursor-agent installation",
"Script validation and checksum verification",
"30 second timeout for HTTP requests"
]
}
}
}
