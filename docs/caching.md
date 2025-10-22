# Smart Caching: Why git-ai Gets Faster Over Time

git-ai uses smart caching to make your workflow faster. Instead of analyzing your project from scratch every time, it remembers information that doesn't change often and only updates what's actually different.

## What Gets Cached and Why

git-ai caches different types of information for different amounts of time:

### Repository Structure (Cached for 12 hours)

- Your file organization, dependencies, and project structure
- **Why cached**: Project structure doesn't change often
- **Updates when**: You add/remove files or change dependency files
- **Performance**: Fast file system analysis

### Project Information (Cached for 12 hours)

- Your README, documentation, and project conventions
- **Why cached**: Documentation doesn't change often and AI analysis is expensive
- **Updates when**: You modify README, docs/, CHANGELOG, etc.
- **Performance**: Calls cursor-agent to understand your project (slow first time)

### Your Configuration (Cached for 24 hours)

- Your git-ai settings and cursor-agent config
- **Why cached**: Configuration changes rarely and reading files is fast
- **Updates when**: You modify .git-ai.yaml or cursor-agent config

### Git Repository State (Cached for 5 minutes)

- Current staged/unstaged files, recent commits, branch info
- **Why short cache**: Repository state changes frequently
- **Updates when**: You make commits, stage files, switch branches

### Command Information (Never cached)

- What command you're running and any custom options
- **Why not cached**: Always specific to your current action

## How You'll Experience Caching

### First Run (Cold Cache)

```bash
$ git ai commit
# Analyzing repository structure... (1 second)
# Processing 3 documentation files to extract project metadata... (3-4 seconds)
# Gathering git context... (0.5 seconds)
# Generated commit message ready
```

### Subsequent Runs (Warm Cache)

```bash
$ git ai commit
# Using cached repository structure... (instant)
# Using cached project info... (instant)
# Checking git repository changes... (0.5 seconds)
# Generated commit message ready
```

### After Documentation Changes

```bash
# You update your README with new commit conventions
$ git ai commit
# Processing 3 documentation files to extract project metadata... (3-4 seconds)
# Updated project context now cached for next time
```

## How git-ai Knows When to Update Cache

git-ai uses smart detection to know exactly when cached information is outdated:

### Smart File Monitoring

**For Documentation (Project Context)**
git-ai watches these files for changes:

- `README.*` (README.md, README.rst, etc.)
- `CHANGELOG.*`, `HISTORY.*`
- `docs/` directory contents
- `LICENSE`, `CONTRIBUTING.*`, etc.

When you change any of these files, git-ai automatically knows to re-analyze your project the next time you run a command.

```bash
# Scenario: You update your commit convention in README.md
$ echo "Use feat: prefix for features" >> README.md
$ git ai commit
# git-ai detects README.md changed, re-analyzes project context
# New commit follows the updated conventions
```

**For Configuration (Agent Context)**
git-ai watches:

- `.git-ai.yaml` in your repository
- `~/.config/git-ai/config.yaml`
- `.cursoragent` configuration files

### Smart Git Monitoring

**For Repository State (Git Context)**
git-ai automatically detects when you:

- Make new commits
- Stage or unstage files
- Switch branches
- Change working directory state

```bash
# Scenario: You stage files for commit
$ git add src/new_feature.py
$ git ai commit
# git-ai sees the staged files and includes them in context
# No cache invalidation needed - it just sees current state
```

### Automatic Expiration

Even if file monitoring misses something, cached information expires automatically:

- **Project info**: Expires after 12 hours
- **Configuration**: Expires after 24 hours
- **Git state**: Expires after 5 minutes
- **Command info**: Never cached (always fresh)

This ensures you never get stale information, even in edge cases.

## Where Cache is Stored

git-ai stores cached information in your repository, but it's safe and doesn't interfere with your work:

```
.git/git-ai/context-cache/    # Cache directory (safe to delete)
├── git.json                  # Repository state cache
├── project.json             # Project documentation analysis
├── agent.json               # Configuration cache
└── file_hashes.json         # File change tracking
```

**Important**: These files are completely safe to delete. git-ai will simply rebuild the cache the next time you run a command.

## Managing Your Cache

### Viewing Cache Status

```bash
# See configuration file status (not cache, but useful)
git ai config --show

# See detailed timing during commands
git ai commit --verbose
```

### Clearing Cache

```bash
# Clear all cached information (manual approach)
rm -rf .git/git-ai/context-cache/

# git-ai will rebuild cache automatically on next run
git ai commit
```

### When to Clear Cache

**Cache seems outdated:**

- Commands not reflecting recent documentation changes
- Configuration changes not being picked up
- Generally suspicious AI responses

**Performance testing:**

- Want to see "cold start" timing
- Testing different context configurations
- Benchmarking performance improvements

**Troubleshooting:**

- Weird errors related to context
- Inconsistent behavior between runs
- Cache corruption (very rare)

## Cache Performance Tips

### Getting the Best Performance

**First Run in a New Repository:**

```bash
$ git ai commit
# Expect 3-5 seconds as git-ai learns about your project
```

**Typical Usage After Cache Warm-up:**

```bash
$ git ai commit
# Expect 0.5-1 second for most operations
```

**After Changing Documentation:**

```bash
# You updated README.md
$ git ai commit
# Expect 3-4 seconds as project context rebuilds
# Subsequent runs will be fast again
```

### Optimizing for Your Workflow

**If you commit frequently:**

- Cache will help a lot - first run slower, then very fast
- Consider using minimal context for simple commits

**If your documentation changes often:**

- Project context will rebuild more frequently
- Consider whether you need Project context for all commands

**If you work on many projects:**

- Each project has its own cache
- First run in each project will be slower

### Cache Directory Size

The cache directory typically uses:

- **Small projects**: 10-50 KB
- **Medium projects**: 50-200 KB
- **Large projects with extensive docs**: 200KB-1MB

Cache doesn't grow much over time - it just stores the latest analysis.

## Troubleshooting Cache Issues

### "git-ai seems to ignore my recent documentation changes"

```bash
# Force cache refresh by removing cache directory
rm -rf .git/git-ai/context-cache/
git ai commit
```

### "git-ai is slower than expected"

```bash
# Check what's happening
git ai commit --verbose

# Look for:
# - "Processing X documentation files..." (normal on first run)
# - Long project analysis times (normal for complex documentation)
# - Repeated project analysis (might indicate file change detection issue)
```

### "Weird or inconsistent behavior"

```bash
# Nuclear option - clear everything and start fresh
rm -rf .git/git-ai/context-cache/

# Then check if issue persists
git ai commit --verbose
```

### "Cache directory taking up too much space"

Cache directories are usually small, but if needed:

```bash
# Safe to delete - git-ai will rebuild
rm -rf .git/git-ai/context-cache/
```

## Understanding Cache Behavior

### When Cache Hits vs Misses

**Cache Hit (Fast)**:

- Documentation unchanged since last run
- Configuration unchanged
- Repository state cached (within 5 minutes)

**Cache Miss (Slower)**:

- First run in repository
- Documentation files modified
- Configuration files changed
- Repository state changed (commits, staging)
- Cache expired (safety expiration)

This caching system is designed to be invisible when working well - you'll just notice git-ai getting faster after the first few uses in each project.
