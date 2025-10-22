# Understanding git-ai Context

git-ai provides intelligent, contextual assistance by understanding your project in four key ways. This context system is what makes git-ai's suggestions accurate and relevant to your specific project and situation.

## What is Context?

Context is the background information git-ai gathers about your project to provide better AI assistance. Instead of just looking at your current changes, git-ai understands:

- **Your current git state** (staged files, recent commits, branches)
- **Your repository structure** (file organization, dependencies, project layout)
- **Your project conventions** (from documentation and README)
- **Your development environment** (configuration files, tools)
- **What you're trying to do** (the specific command and your intent)

## How Context Works

When you run a git-ai command, it automatically:

1. **Gathers relevant information** about your project
2. **Caches information** that doesn't change often for speed
3. **Combines everything** into a comprehensive picture
4. **Sends structured context** to the AI for better responses

## Example: Context in Action

Let's say you run `git ai commit` after adding authentication features:

```bash
# You've made changes to auth-related files
git add src/auth/ tests/auth_test.py

# git-ai gathers context automatically
git ai commit
```

git-ai automatically understands:

- **Git Context**: You've modified authentication files and tests
- **Repository Context**: This is a Python project with pytest structure
- **Project Context**: Testing conventions and commit message format preferences
- **Agent Context**: You prefer detailed commit messages (from your config)
- **Interaction Context**: You're creating a commit message

Result: A commit message that follows your project's conventions and accurately describes the auth changes.

## Context Performance

### Automatic Optimization

git-ai is designed to be fast:

- **Smart Caching**: Information is cached when it doesn't change often
- **Selective Updates**: Only gathers fresh context when needed
- **Efficient Processing**: Most context gathering happens in milliseconds

### What You'll Notice

```bash
# First run in a session (gathers fresh context)
$ git ai commit
# Takes 2-3 seconds while analyzing project

# Subsequent runs (uses cached context)
$ git ai commit
# Takes <1 second, context is already cached
```

## Configuring Context for Your Workflow

Different commands work better with different context types. You can customize which context each command uses:

### Default Context (Recommended)

Most commands use all five context types by default:

```bash
# These commands get full context automatically
git ai commit    # Uses: Git + Repository + Project + Agent + Interaction
git ai pr        # Uses: Git + Repository + Project + Agent + Interaction
git ai merge     # Uses: Git + Repository + Project + Agent + Interaction
```

### Customizing Context

You can override context selection in your configuration:

```yaml
# .git-ai.yaml
commands:
  commit:
    context:
      - "Git" # Repository state
      - "Repository" # File structure
      - "Project" # Project conventions

  ignore:
    context:
      - "Git" # Minimal context for speed
```

### Performance vs. Quality Trade-offs

**More Context = Better Results, Slower Performance**

```bash
# Full context (best quality, ~2-3 seconds)
git ai commit    # All five context types

# Minimal context (fastest, ~0.5 seconds)
git ai commit    # Just Git context (if configured)
```

## When Context Updates

git-ai intelligently updates context only when needed:

### Automatic Updates

- **Git changes**: New commits, staged files, branch switches
- **Documentation changes**: README, CHANGELOG, docs/ modifications
- **Config changes**: .git-ai.yaml, cursor-agent config updates

### Manual Cache Management

Currently, git-ai doesn't provide direct cache management commands, but you can:

```bash
# View configuration status (shows config files, not cache)
git ai config --show

# Clear cache manually by deleting the cache directory
rm -rf .git/git-ai/context-cache/
```

## Context Storage

git-ai stores context in your repository for speed:

```
.git/git-ai/context-cache/    # Context cache directory
├── git.json                  # Repository state cache
├── project.json             # Project info cache
├── agent.json               # Configuration cache
└── file_hashes.json         # Change tracking
```

**These files are safe to delete** - git-ai will rebuild them automatically.

## Troubleshooting Context Issues

### Context Seems Outdated

```bash
# Clear cache and try again (manually delete cache)
rm -rf .git/git-ai/context-cache/
git ai commit
```

### Slow Performance

```bash
# Check what context is being used
git ai commit --verbose

# Try with minimal context (after configuring minimal context)
git ai commit
```

### Context Errors

```bash
# Check configuration files
git ai config --show

# See detailed output during execution
git ai commit --verbose --dry-run
```

## Understanding Context Quality

Good context leads to better AI responses:

- **Accurate commit messages** that follow your conventions
- **Proper PR descriptions** that match your project style
- **Relevant suggestions** based on your actual changes
- **Consistent tone** matching your documentation
