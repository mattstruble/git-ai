# Configuration

git-ai supports flexible configuration through YAML files, allowing you to customize behavior at both the repository and user level. The configuration system supports per-command settings, context selection, and behavior overrides.

## Configuration Files

### Hierarchy

Configuration is loaded in the following order (later files override earlier ones):

1. **Default Configuration**: Embedded in the binary
2. **User Configuration**: `~/.config/git-ai/config.yaml`
3. **Repository Configuration**: `.git-ai.yaml` in the repository root

### File Locations

```bash
# User-wide configuration
~/.config/git-ai/config.yaml

# Repository-specific configuration
.git-ai.yaml              # Repository root
.git/git-ai/config.yaml   # Alternative location
```

## Configuration Structure

### Complete Example

```yaml
# Global behavior settings
behavior:
  verbose: false
  dry_run: false

# Per-command configuration
commands:
  commit:
    prompt: null # Use built-in prompt
    no_confirm: false
    context:
      - "Git"
      - "Repository"
      - "Project"
      - "Agent"
      - "Interaction"

  pr:
    prompt: null
    no_confirm: false
    context:
      - "Git"
      - "Repository"
      - "Project"
      - "Agent"
      - "Interaction"

  merge:
    prompt: "Custom merge prompt template"
    no_confirm: false
    context:
      - "Git"
      - "Project"

  init:
    prompt: null
    no_confirm: false
    context:
      - "Git"
      - "Repository"
      - "Project"
      - "Agent"
      - "Interaction"

  ignore:
    prompt: null
    no_confirm: true # Skip confirmation for ignore operations
    context:
      - "Git"
      - "Agent"

# Project-specific settings
project:
  # Dependency file patterns for repository context
  dependency_files:
    # JavaScript/Node.js
    - "package.json"
    - "package-lock.json"
    - "yarn.lock"
    - "pnpm-lock.yaml"

    # Python
    - "requirements.txt"
    - "pyproject.toml"
    - "Pipfile"
    - "Pipfile.lock"

    # Rust
    - "Cargo.toml"
    - "Cargo.lock"

    # Go
    - "go.mod"
    - "go.sum"

    # Add custom patterns
    - "custom-deps.json"

  # Build and automation files
  build_files:
    - "Makefile"
    - "justfile"
    - "build.sh"
    - "Dockerfile"
    - "docker-compose.yml"

  # Configuration files that affect compilation/runtime
  config_files:
    - ".env*"
    - "*.config.js"
    - "tsconfig.json"
    - ".eslintrc*"

  # Additional patterns for your project
  additional_patterns:
    - "custom-config.yaml"
    - "special-file.txt"
```

## Configuration Sections

### Behavior

Global settings that affect all commands:

```yaml
behavior:
  verbose: false # Enable verbose output
  dry_run: false # Default to dry-run mode
  no_confirm: false # Skip all confirmation prompts
```

### Commands

Per-command configuration with support for:

- **Custom prompts**: Override built-in prompts
- **Confirmation behavior**: Skip prompts for specific commands
- **Context selection**: Choose which context types to include

```yaml
commands:
  commit:
    prompt: |
      You are a git commit message generator.
      Based on the provided context, create a concise commit message.
      Follow conventional commit format: type(scope): description
    no_confirm: false
    context:
      - "Git" # Repository state and diffs
      - "Repository" # File structure and dependencies
      - "Project" # Documentation and project metadata
```

#### Available Context Types

- `"Git"`: Repository status, diffs, commits, branch info
- `"Repository"`: File structure, dependencies, project organization
- `"Project"`: AI-processed project documentation and metadata
- `"Agent"`: Configuration files and development environment
- `"Interaction"`: Command-specific arguments and user intent

### Project Settings

Configure which files are considered for project context:

```yaml
project:
  dependency_files:
    - "package.json" # Node.js dependencies
    - "Cargo.toml" # Rust dependencies
    - "requirements.txt" # Python dependencies

  build_files:
    - "Makefile"
    - "justfile"
    - "Dockerfile"

  config_files:
    - ".env*"
    - "tsconfig.json"
    - ".eslintrc*"

  additional_patterns:
    - "custom-file.yaml" # Project-specific additions
```

## Command-Specific Settings

### Commit Command

```yaml
commands:
  commit:
    # Custom prompt template
    prompt: |
      Generate a commit message following these rules:
      1. Use conventional commit format
      2. Keep first line under 50 characters
      3. Include scope when relevant
      4. Focus on the 'why' not the 'what'

    # Skip confirmation prompt
    no_confirm: false

    # Include comprehensive context
    context:
      - "Git"
      - "Repository"
      - "Project"
      - "Agent"
      - "Interaction"
```

### PR Command

```yaml
commands:
  pr:
    # Custom PR template
    prompt: |
      Create a pull request description with:
      1. Clear title summarizing the change
      2. What changed and why
      3. Testing instructions
      4. Any breaking changes

    context:
      - "Git"
      - "Project"
```

### Merge Command

```yaml
commands:
  merge:
    # Merge-specific prompt
    prompt: |
      Analyze the merge and provide:
      1. Summary of changes being merged
      2. Potential conflicts or issues
      3. Recommended merge strategy

    context:
      - "Git"
      - "Project"
```

### Init Command

```yaml
commands:
  init:
    # Project initialization prompt
    prompt: |
      Help initialize a new project with:
      1. Appropriate directory structure
      2. Common configuration files
      3. Documentation templates
      4. Development tooling setup

    context:
      - "Git"
      - "Agent"
      - "Interaction"
```

### Ignore Command

```yaml
commands:
  ignore:
    # Skip confirmation for ignore operations
    no_confirm: true

    # Minimal context for ignore operations
    context:
      - "Git"
      - "Agent"
```

## Dynamic Configuration

### Environment Variables

Override configuration with environment variables:

```bash
# Force verbose mode
export GIT_AI_VERBOSE=true

# Default to dry-run
export GIT_AI_DRY_RUN=true

# Skip all confirmations
export GIT_AI_NO_CONFIRM=true
```

### Command-Line Overrides

Command-line flags override configuration file settings:

```bash
# Override no_confirm setting
git ai commit --no-confirm

# Override dry_run setting
git ai pr --dry-run

# Override verbose setting
git ai merge --verbose feature-branch
```

## Configuration Management

### Initialize Configuration

Generate a sample configuration file:

```bash
# Create user configuration
git ai config --init

# Create repository configuration
git ai config --init > .git-ai.yaml
```

### Show Current Configuration

```bash
# Display effective configuration
git ai config --show

# Show configuration status
git ai config --show
```

### View Configuration

```bash
# Check configuration file status
git ai config --show
```

## Advanced Configuration

### Conditional Configuration

Use YAML anchors and references for complex configurations:

```yaml
# Define common context sets
common_contexts: &full_context
  - "Git"
  - "Repository"
  - "Project"
  - "Agent"
  - "Interaction"

minimal_contexts: &minimal_context
  - "Git"

# Apply to commands
commands:
  commit:
    context: *full_context

  ignore:
    context: *minimal_context
```

### Template Variables

Custom prompts support template variables:

```yaml
commands:
  commit:
    prompt: |
      Project: {{project.name}}
      Language: {{project.primary_language}}

      Generate a commit message for this {{project.primary_language}} project.
      Follow the project's convention: {{project.conventions.commit_style}}
```

### Multi-Environment Configuration

```yaml
# Development environment
development:
  behavior:
    verbose: true
    dry_run: true

# Production environment
production:
  behavior:
    verbose: false
    no_confirm: true

# Use environment-specific config
commands:
  commit:
    <<: *development  # In development
    # <<: *production  # In production
```

## Configuration Best Practices

### Repository Configuration

- Keep repository configs minimal and focused
- Version control `.git-ai.yaml` for team consistency
- Document any custom prompts or unusual settings

### User Configuration

- Set personal preferences in user config
- Avoid overriding team conventions
- Use environment variables for temporary changes

### Context Selection

- Only include context types that are actually useful
- Project context is expensive - only use when needed
- Interaction context is lightweight and usually helpful

### Custom Prompts

- Test custom prompts thoroughly before committing
- Keep prompts focused and specific
- Include examples in prompt templates
- Document expected AI behavior

### Performance Considerations

- Minimize context types for frequently-used commands
- Use caching-friendly context selection
- Profile configuration changes for performance impact

## Troubleshooting

### Configuration Errors

```bash
# Show configuration status
git ai config --show

# Test with verbose output
git ai commit --verbose --dry-run
```

### Context Issues

```bash
# Clear context cache manually
rm -rf .git/git-ai/context-cache/

# Test context generation with verbose output
git ai commit --verbose --dry-run
```

### Performance Issues

```bash
# Check timing with verbose output
git ai commit --verbose --dry-run

# Check configuration files
git ai config --show
```
