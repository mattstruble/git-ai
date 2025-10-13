{
  description = "Git AI plugin — integrates cursor-agent into git workflow with AI-powered prompts";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs =
    { self
    , nixpkgs
    , flake-utils
    ,
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = import nixpkgs { inherit system; };
      in
      {
        packages.default = pkgs.rustPlatform.buildRustPackage {
          pname = "git-ai";
          version = "0.1.0";

          src = ./.;
          cargoLock.lockFile = ./Cargo.lock;

          nativeBuildInputs = with pkgs; [
            pkg-config
            openssl.dev
          ];

          buildInputs = with pkgs; [
            openssl
          ] ++ pkgs.lib.optionals pkgs.stdenv.isDarwin [
            libiconv
          ];

          meta = with pkgs.lib; {
            description = "A git plugin that integrates cursor-agent for AI-assisted commits, PRs, and merges.";
            homepage = "https://github.com/mattstruble/git-ai";
            license = licenses.mit;
            platforms = platforms.unix;
          };
        };

        # Developer shell for local testing
        devShells.default = pkgs.mkShell {
          buildInputs = with pkgs; [
            rustc
            cargo
            rustfmt
            clippy
            pkg-config
            openssl.dev
            git
          ] ++ pkgs.lib.optionals pkgs.stdenv.isDarwin [
            libiconv
          ];
          shellHook = ''
            echo "Git-AI Rust Dev Environment"
            echo "Run 'cargo build' to build and 'cargo run -- --help' to test."
          '';
        };
      }
    );
}
