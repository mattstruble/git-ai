# git-ai

AI-assisted git workflow with cursor-agent integration

## Overview

`git-ai` is a git plugin that provides AI-powered assistance for common git workflows including commits, pull requests, and merges. It integrates with cursor-agent to generate contextual prompts and suggestions.

## Features

- **Smart Commit Messages**: Generate concise, descriptive commit messages from your git diff
- **PR Descriptions**: Create professional pull request descriptions summarizing recent changes
- **Merge Summaries**: Get AI assistance with merge conflict resolution and summary messages
- **Automatic Setup**: Automatically installs cursor-agent and registers git alias
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

## Usage

The tool automatically registers itself as a git alias, so you can use either:

```bash
git-ai <command> [options]
# or
git ai <command> [options]
```

### Commands

- `commit` - Generate AI-assisted commit message
- `pr` - Generate AI-assisted PR description
- `merge <branch>` - Generate AI-assisted merge summary for a specific branch

### Examples

```bash
# Generate a commit message from current changes
git ai commit

# Create a PR description with custom context
git ai pr -m "Summarize the refactor changes for PR body"

# Get merge assistance for a specific branch
git ai merge feature/new-auth

# Get merge assistance with custom context and force flag
git ai merge feature/api-refactor --force -m "Focus on database migration conflicts"
```

## How it Works

1. **Git Alias Registration**: On first run, registers `git ai` as an alias to `git-ai`
2. **Cursor-agent Setup**: Automatically downloads and installs cursor-agent if not present
3. **Context Generation**: Creates appropriate prompts based on the selected command
4. **AI Processing**: Calls cursor-agent with the generated prompt for AI assistance

## License

MIT License - see LICENSE file for details.
