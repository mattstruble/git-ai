# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

- - -
## v0.7.1 - 2025-10-15
#### Bug Fixes
- test new ci/cd release tagging - (d8cee0a) - Matt Struble
#### Continuous Integration
- **(release)** fix version tag handling in release workflow - (8324abd) - Matt Struble

- - -

## v0.7.0 - 2025-10-15
#### Bug Fixes
- clean up changelog format for cocogitto compatibility - (249c040) - Matt Struble
#### Build System
- optimize cocogitto configuration for v6.3.0 - (1041eb7) - Matt Struble
- adjust cocogitto bump rules for non-code changes - (422b594) - Matt Struble
#### Continuous Integration
- **(release)** improve tag fetching in workflow - (5b1a625) - Matt Struble
- **(workflows)** refactor release workflow to use cocogitto action - (b8a8e38) - Matt Struble
- **(workflows)** improve release workflow robustness - (e4441b4) - Matt Struble
- add tag prefix - (59256a9) - Matt Struble
- migrate release workflow to use cocogitto - (63906e4) - Matt Struble
#### Features
- implement cocogitto automation with hooks - (ea17409) - Matt Struble
#### Miscellaneous
- **(config)** remove commit_msg hook configuration - (e8cf633) - Matt Struble
- bump version to 0.6.1 - (752cc2b) - github-actions[bot]
#### Testing
- validate cocogitto hooks functionality - (98818d8) - Matt Struble

- - -


## [0.6.1] - 2025-10-14

## [0.6.0] - 2025-10-14

## [0.5.0] - 2025-10-14

### Added
- **Installation**: Homebrew installation support via `mattstruble/formulae` tap
- **Commands**: New `init` command for AI-guided project initialization
- **Commands**: New `ignore` command for intelligent .gitignore management
- **Configuration**: Per-command configuration with override capabilities
- **Architecture**: Command-based organization with improved maintainability

### Installation
```bash
# New Homebrew installation method
brew tap mattstruble/formulae
brew install git-ai
```

## [0.4.7] - 2025-10-14

## [0.4.6] - 2025-10-14

## [0.4.5] - 2025-10-14

## [0.4.4] - 2025-10-14

## [0.4.3] - 2025-10-14

## [0.4.2] - 2025-10-14

## [0.4.1] - 2025-10-14


## [0.4.0] - 2025-10-14

### Added
- **Security**: User confirmation prompts for cursor-agent installation
- **Security**: Basic script validation and checksum verification framework for cursor-agent installer
- **CLI**: `--dry-run` flag to preview prompts without executing cursor-agent
- **CLI**: `--verbose` flag for detailed debugging output
- **CLI**: `--reinstall-agent` flag to force cursor-agent reinstallation
- **CLI**: `--no-confirm` flag to skip user confirmation prompts
- Timeout handling for HTTP requests (30 second timeout)
- Better error messages and installation verification

### Changed
- **BREAKING**: `--force` flag behavior clarified - now primarily for cursor-agent file changes
- Improved security for cursor-agent installation with user consent warnings
- Enhanced installation process with better error handling and validation
- More informative output with emojis and structured messages

### Security
- **CRITICAL**: Fixed insecure script execution - now requires user consent before downloading/executing scripts
- Added basic validation that downloaded content is a shell script
- Added framework for checksum verification (pending actual checksums)
- Improved error handling to prevent execution of malformed scripts


## [0.1.0] - 2024-01-XX

### Added
- Initial release of git-ai
- AI-assisted commit message generation
- AI-assisted PR description generation  
- AI-assisted merge summaries
- Automatic cursor-agent installation
- Git plugin support (`git ai` works automatically)
- Cross-platform support (macOS, Linux, Unix-like systems)
- Integration with cursor-agent for AI functionality

### Features
- **Commit Command**: Generate contextual commit messages from git diff
- **PR Command**: Create professional pull request descriptions
- **Merge Command**: Get AI assistance with merge summaries
- **Auto-setup**: Automatic cursor-agent installation
- **Flexible Prompts**: Support for custom messages to guide AI generation

### Technical
- Built with Rust using clap for CLI parsing
- Async HTTP client for cursor-agent installation  
- Comprehensive unit tests for CLI parsing
- Git integration for repository state detection
