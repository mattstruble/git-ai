# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.4.3] - 2025-10-14

# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).


## [0.4.2] - 2025-10-14

# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).


## [0.4.1] - 2025-10-14

# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).


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
