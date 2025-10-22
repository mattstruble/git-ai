# Context Types: What git-ai Knows About Your Project

git-ai understands your project through five different types of context. Each type provides specific information that helps the AI give you better, more relevant suggestions.

## Git Context: Your Repository State

**What it knows**: Everything about your current git repository  
**When it updates**: Every time you make commits, stage files, or change branches  
**Performance impact**: Fast (updates automatically as needed)

### What Git Context Includes

**Current Changes**

- Files you've staged for commit
- Files you've modified but not staged
- New files you haven't tracked yet
- Whether you have any merge conflicts

**Recent History**

- Your last few commits
- What branch you're on
- How far ahead/behind you are from remote
- Your git user name and email

**Repository Info**

- Remote repository URLs
- Repository root directory
- Current git configuration

### Real-World Examples

**Commit Messages**

```bash
$ git add src/auth.py tests/test_auth.py
$ git ai commit

# Git context tells the AI:
# - You've added authentication code and tests
# - Your last commits were about user management
# - You're on branch "feature/auth"
# Result: "feat(auth): add user authentication with tests"
```

**Pull Request Descriptions**

```bash
$ git ai pr

# Git context tells the AI:
# - 5 commits since last merge
# - Modified authentication, added tests, updated docs
# - Ahead of main by 5 commits
# Result: Comprehensive PR description covering all changes
```

### When Git Context is Most Useful

- **Commit messages**: Understands what files changed and how
- **PR descriptions**: Knows the full scope of your changes
- **Merge assistance**: Sees conflict areas and recent history
- **Any command where current changes matter**

## Repository Context: Your Project Structure

**What it knows**: Your repository's file structure and organization  
**When it updates**: When you add/remove files or change dependency files  
**Performance impact**: Fast (analyzes file system, cached for 12 hours)

### What Repository Context Includes

**Directory Structure**

- Visual directory tree (up to 3 levels deep)
- File and folder organization
- Project layout and architecture

**File Analysis**

- Count of files by extension (.js, .py, .md, etc.)
- Total number of files and repository size
- Recently changed files

**Dependency Information**

- Package files (package.json, Cargo.toml, requirements.txt, etc.)
- Build configuration files (Makefile, justfile, Dockerfile, etc.)
- Contents of key dependency and configuration files

### Real-World Examples

**Understanding Project Structure**

```bash
# You're working in a complex monorepo
$ git ai init --language javascript

# Repository context tells the AI:
# - This has frontend/ and backend/ directories
# - Uses yarn workspaces (multiple package.json files)
# - Has Docker setup and custom build scripts
# Result: Initialization suggestions that fit your monorepo structure
```

**Dependency-Aware Suggestions**

```bash
# You're adding a new feature
$ git ai commit

# Repository context tells the AI:
# - Recently modified package.json (added new dependency)
# - Updated multiple TypeScript files
# - This is a TypeScript React project structure
# Result: "feat(deps): add lodash for utility functions in user module"
```

### Files That Shape Repository Context

Repository context analyzes your project structure by examining:

**Dependency Files**

- `package.json`, `package-lock.json`, `yarn.lock` (Node.js)
- `Cargo.toml`, `Cargo.lock` (Rust)
- `requirements.txt`, `pyproject.toml` (Python)
- `go.mod`, `pom.xml`, `build.gradle` (Go, Java)
- And many more...

**Build & Configuration Files**

- `Makefile`, `justfile`, `Taskfile.yml`
- `Dockerfile`, `docker-compose.yml`
- `flake.nix`, `shell.nix`
- Build tool configs (webpack, vite, etc.)

**Project Structure**

- Directory tree visualization
- File distribution and organization patterns
- Recently modified files

### When Repository Context is Most Useful

- **Project initialization**: Sets up structure matching your existing patterns
- **Commit messages**: Understands when you've changed dependencies or structure
- **Large projects**: Helps AI understand complex project organization
- **Multi-language projects**: Recognizes different parts of your tech stack

### Performance Notes

Repository context is designed to be efficient:

- **First time**: 1-2 seconds (analyzes file structure)
- **Cached**: Instant (until files are added/removed or dependencies change)
- **Updates**: Only when project structure or dependency files change
- **File limits**: Only reads small config files, skips large files for performance

## Project Context: Your Project's Personality

**What it knows**: Your project's conventions, style, and documentation  
**When it updates**: When you change README, docs, CHANGELOG, or other documentation  
**Performance impact**: Moderate (AI processes documentation, cached for 12 hours)

### What Project Context Includes

**Project Basics**

- Project name, description, and main programming language
- Key features and purpose
- Dependencies and build tools

**Your Team's Conventions**

- Commit message style (Conventional Commits, etc.)
- Branching strategy (gitflow, feature branches, etc.)
- Code style and formatting rules
- Testing approach

**Development Workflow**

- How to build and test the project
- Release process
- Breaking change policies
- Documentation standards

### Real-World Examples

**Following Your Commit Style**

```bash
# Your README says you use Conventional Commits
$ git ai commit

# Project context tells the AI:
# - This project uses "feat:", "fix:", "docs:" prefixes
# - Commit messages should be under 50 characters
# - Include scope when relevant
# Result: "feat(auth): add JWT token validation"
```

**Matching Your PR Style**

```bash
# Your CONTRIBUTING.md has PR templates
$ git ai pr

# Project context tells the AI:
# - PRs need "Testing" and "Breaking Changes" sections
# - Link to related issues
# - Include performance impact notes
# Result: PR description following your team's template
```

### Files That Shape Project Context

git-ai analyzes these files to understand your project:

**Core Documentation**

- `README.md` - Project overview, usage, conventions
- `CHANGELOG.md` - Release history and breaking changes
- `CONTRIBUTING.md` - Development and PR guidelines
- `LICENSE` - Project licensing information

**Development Docs**

- `docs/` directory - Detailed guides and specifications
- `SECURITY.md` - Security policies and reporting
- `CODE_OF_CONDUCT.md` - Community guidelines

### When Project Context is Most Useful

- **Commit messages**: Follows your established commit conventions
- **PR descriptions**: Matches your team's PR template and style
- **Project initialization**: Sets up new projects using your patterns
- **Any command where consistency with existing practices matters**

### Performance Notes

Project context is the most intelligent but also the slowest to generate:

- **First time**: 3-5 seconds (calls cursor-agent to analyze your documentation)
- **Cached**: Instant (until documentation changes)
- **Updates**: Only when you modify documentation files
- **Why slow**: Uses AI to understand your project's conventions and structure

## Agent Context: Your Development Environment

**What it knows**: Your git-ai and cursor-agent configuration  
**When it updates**: When you change configuration files  
**Performance impact**: Fast (just reads config files, cached for 24 hours)

### What Agent Context Includes

**Your Preferences**

- Your git-ai configuration settings
- Custom prompts you've defined
- Commands you prefer to skip confirmation for
- Context types you use for each command

**Development Environment**

- Your operating system and shell
- Available development tools
- cursor-agent configuration
- Editor preferences

### Real-World Examples

**Respecting Your Preferences**

```bash
# Your .git-ai.yaml says you prefer detailed commits
$ git ai commit

# Agent context tells the AI:
# - User likes comprehensive commit messages
# - Include technical details
# - Use formal tone
# Result: Detailed technical commit message
```

**Using Your Tools**

```bash
# Your environment has specific package managers
$ git ai init --language rust

# Agent context tells the AI:
# - User has cargo and nix available
# - Suggest nix flake setup
# - Include cargo commands in instructions
# Result: Setup instructions using your preferred tools
```

### Configuration Files That Matter

**git-ai Configuration**

- `.git-ai.yaml` - Repository-specific preferences
- `~/.config/git-ai/config.yaml` - Your personal defaults
- Environment variables

**Development Environment**

- `.cursoragent` - cursor-agent configuration
- Shell and system information
- Available package managers and build tools

### When Agent Context is Most Useful

- **All commands**: Ensures responses match your preferences
- **Project initialization**: Uses your preferred tools and setup
- **Configuration**: Provides relevant suggestions based on your environment

## Interaction Context: What You're Trying to Do

**What it knows**: The specific command you're running and how you want it done  
**When it updates**: Every single command (never cached)  
**Performance impact**: None (lightweight, always fresh)

### What Interaction Context Includes

**Current Command**

- Which git-ai command you're running (`commit`, `pr`, `merge`, etc.)
- Any flags or options you provided
- Custom messages or guidance you included

**Your Intent**

- What you're trying to accomplish
- Any specific focus you requested
- Whether you want a dry run or final result

**Session Information**

- Recent git commands you ran
- Timestamp of this request
- Sequence of related commands

### Real-World Examples

**Custom Focus**

```bash
$ git ai commit -m "Focus on the security improvements"

# Interaction context tells the AI:
# - Command: commit
# - User intent: Focus on security aspects
# - Flag: Custom message provided
# Result: Commit message emphasizing security changes
```

**Dry Run Requests**

```bash
$ git ai pr --dry-run

# Interaction context tells the AI:
# - Command: pr (pull request)
# - Mode: dry run (show preview, don't execute)
# - User wants to see the result first
# Result: Displays PR description without creating it
```

### Why Interaction Context Matters

**Command-Specific Behavior**

- `commit` commands focus on change summaries
- `pr` commands focus on feature explanations
- `merge` commands focus on integration concerns
- `init` commands focus on project setup

**Respects Your Requests**

- Custom messages guide the AI's focus
- Dry run mode shows previews
- Confirmation preferences are honored

### When Interaction Context is Most Useful

- **Every command**: Ensures the AI understands what you want
- **Custom guidance**: When you provide specific instructions
- **Workflow continuity**: Connecting related commands in sequence

## How Context Types Work Together

All five context types combine to give git-ai a complete picture:

### Example: Creating the Perfect Commit Message

```bash
$ git add src/auth/ docs/auth.md tests/test_auth.py
$ git ai commit
```

**Git Context** sees:

- 3 files changed: auth code, documentation, tests
- This is a feature branch called "feature/jwt-auth"
- Previous commits were about user management

**Repository Context** understands:

- This is a Node.js project with TypeScript
- Has src/auth/ directory structure established
- Uses Jest for testing (sees jest.config.js)
- Recently added @types/jwt dependency

**Project Context** knows:

- This project uses Conventional Commits
- Format should be "feat(scope): description"
- Tests are required for new features
- Documentation should be updated with code

**Agent Context** remembers:

- You prefer concise but informative commit messages
- You include scope when it's relevant
- You like to mention test coverage

**Interaction Context** understands:

- You're running the `commit` command
- No custom message, so generate one automatically
- Not a dry run, you want the actual commit

**Result**: `feat(auth): add JWT authentication with tests and docs`

## Choosing the Right Context for Your Commands

### Full Context (Recommended)

```yaml
# .git-ai.yaml
commands:
  commit:
    context: ["Git", "Repository", "Project", "Agent", "Interaction"] # All five
  pr:
    context: ["Git", "Repository", "Project", "Agent", "Interaction"] # All five
```

**Best for**: Most commands where you want highest quality results

### Minimal Context (For Speed)

```yaml
commands:
  ignore:
    context: ["Git"] # Just repository state
```

**Best for**: Simple operations where context doesn't matter much

### Balanced Context

```yaml
commands:
  commit:
    context: ["Git", "Repository", "Interaction"] # Skip slower contexts
```

**Best for**: When you want good results but faster performance

## Context Performance Tips

### What Affects Speed

**Slowest**: Project Context (calls cursor-agent to analyze documentation)

```bash
$ git ai commit  # First time: ~3 seconds (AI analysis)
$ git ai commit  # Cached: ~1 second
```

**Fast**: Repository Context (analyzes file structure), Agent Context (reads config files), Git Context (git commands)  
**Instant**: Interaction Context (current command info)

### Optimizing for Your Workflow

**If documentation changes rarely:**

- Keep Project Context enabled
- It'll stay cached and be very fast

**If you commit frequently:**

- Project Context will usually be cached
- Commands will be fast after the first run

**If you want maximum speed:**

- Configure commands to use minimal context
- Trade some quality for faster responses

### Monitoring Context Performance

```bash
# See timing for each context type
git ai commit --verbose

# Check configuration files
git ai config --show
```
