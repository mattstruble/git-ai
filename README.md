# git-ai

AI-assisted git workflow with cursor-agent integration

## Overview

`git-ai` is a git plugin that provides AI-powered assistance for common git workflows including commits, pull requests, and merges. It integrates with cursor-agent to generate contextual prompts and suggestions.

## Features

- **Smart Commit Messages**: Generate concise, descriptive commit messages from your git diff
- **PR Descriptions**: Create professional pull request descriptions summarizing recent changes
- **Merge Summaries**: Get AI assistance with merge conflict resolution and summary messages
- **Project Initialization**: AI-guided setup of new projects with language-specific structure and tooling
- **Gitignore Management**: Intelligent .gitignore file management with structured language sections
- **Configuration Support**: Per-command configuration with override capabilities
- **Cross-platform**: Works on macOS, Linux, and other Unix-like systems

## Installation

### Using Nix Flakes (Recommended)

#### Install directly from the repository

```bash
# Install to your profile
nix profile install github:mattstruble/git-ai

# Or install temporarily for this shell session
nix shell github:mattstruble/git-ai
```

#### Build and install locally

```bash
# Clone and build
git clone https://github.com/mattstruble/git-ai.git
cd git-ai

# Install to your profile
nix profile install .

# Or build and run directly
nix build
./result/bin/git-ai --help
```

#### Development environment

```bash
# Enter development shell with all dependencies
nix develop

# Then build with cargo
cargo build --release
```

#### Declarative installation (NixOS/Home Manager)

##### NixOS System Configuration

Add to your system `flake.nix`:

```nix
{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    git-ai = {
      url = "github:mattstruble/git-ai";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { self, nixpkgs, git-ai, ... }: {
    nixosConfigurations.your-hostname = nixpkgs.lib.nixosSystem {
      system = "x86_64-linux"; # or "aarch64-linux", "aarch64-darwin", etc.
      modules = [
        ./configuration.nix
        {
          environment.systemPackages = [
            git-ai.packages.x86_64-linux.default # adjust system as needed
          ];
        }
      ];
    };
  };
}
```

##### Home Manager Configuration

Add to your Home Manager `flake.nix`:

```nix
{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    home-manager = {
      url = "github:nix-community/home-manager";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    git-ai = {
      url = "github:mattstruble/git-ai";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { nixpkgs, home-manager, git-ai, ... }: {
    homeConfigurations.your-username = home-manager.lib.homeManagerConfiguration {
      pkgs = nixpkgs.legacyPackages.x86_64-linux; # adjust system as needed
      modules = [
        ./home.nix
        {
          home.packages = [
            git-ai.packages.x86_64-linux.default # adjust system as needed
          ];
        }
      ];
    };
  };
}
```

##### Traditional NixOS configuration.nix

For non-flake NixOS configurations, add to your `configuration.nix`:

```nix
{ config, pkgs, ... }:
let
  git-ai = pkgs.callPackage (pkgs.fetchFromGitHub {
    owner = "mattstruble";
    repo = "git-ai";
    rev = "main"; # or specific commit/tag
    sha256 = ""; # nix will tell you the correct hash
  }) {};
in
{
  environment.systemPackages = [
    git-ai
    # ... other packages
  ];
}
```

Then rebuild your system:

```bash
# NixOS
sudo nixos-rebuild switch --flake .

# Home Manager
home-manager switch --flake .
```

### Using Homebrew

```bash
brew tap mattstruble/formulae
brew install git-ai
```

### Using Cargo

```bash
# From local source
cargo install --path .

# From crates.io (when published)
cargo install git-ai
```

### Manual Installation

```bash
# Build from source
cargo build --release
cp target/release/git-ai ~/.local/bin/  # or any directory in your PATH

# Make sure ~/.local/bin is in your PATH
export PATH="$HOME/.local/bin:$PATH"
```

## Prerequisites

Before using `git-ai`, you need to have `cursor-agent` installed on your system:

## Usage

Since `git-ai` is installed as a git plugin, you can use either:

```bash
git-ai <command> [options]
# or
git ai <command> [options]
```

### Commands

- `commit` - Generate AI-assisted commit message from current changes
- `pr` - Generate AI-assisted pull request description
- `merge <branch>` - Generate AI-assisted merge summary for a specific branch
- `init` - Initialize a new project with AI-guided setup and structure
- `ignore` - Manage .gitignore file with AI assistance
- `config` - Show or initialize configuration files

### Examples

#### Commit Messages

```bash
# Generate a commit message from current changes
git ai commit

# Create commit message with custom context
git ai commit -m "Focus on the security improvements in this change"

# Preview the prompt without executing
git ai commit --dry-run
```

#### Pull Request Descriptions

```bash
# Generate a PR description
git ai pr

# Create PR description with custom guidance
git ai pr -m "Summarize the refactor changes for PR body"
```

#### Merge Assistance

```bash
# Get merge assistance for a specific branch
git ai merge feature/new-auth

# Get merge assistance with custom context
git ai merge feature/api-refactor -m "Focus on database migration conflicts"
```

#### Project Initialization

```bash
# Initialize a new Python project
git ai init --language python --name myproject

# Initialize with AI guidance (interactive)
git ai init

# Preview initialization prompts
git ai init --language rust --dry-run
```

#### Gitignore Management

```bash
# Add ignore patterns for Python and Node.js
git ai ignore add python node

# Remove Python-specific ignore patterns
git ai ignore remove python

# Preview changes without applying
git ai ignore add rust --dry-run
```

#### Configuration

```bash
# Show current configuration
git ai config --show

# Generate sample configuration file
git ai config --init
```

## How it Works

1. **Git Plugin**: Works as a native git plugin with `git ai` command integration
2. **Cursor-agent Integration**: Requires cursor-agent to be pre-installed on your system
3. **Context Generation**: Creates intelligent, context-aware prompts based on the selected command
4. **AI Processing**: Passes structured prompts to cursor-agent for AI-powered assistance
5. **Configuration**: Supports per-command configuration with user overrides
6. **Dry Run Support**: Preview prompts and changes before execution

## Configuration

`git-ai` supports flexible configuration through YAML files:

- **Repository-specific**: `.git-ai.yaml` in your project root
- **User-specific**: `~/.config/git-ai/config.yaml`

Generate a sample configuration:

```bash
git ai config --init
```

Example configuration:

```yaml
behavior:
  verbose: false

commands:
  commit:
    prompt: "Custom commit prompt override"
    no_confirm: false
  init:
    prompt: "Custom initialization prompt"
    no_confirm: false
  ignore:
    no_confirm: true # Skip confirmation for ignore operations
```

## License

MIT License - see LICENSE file for details.
