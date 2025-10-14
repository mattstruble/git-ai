default:
    @just --list

lint:
  pre-commit install
  pre-commit run --all-files

check:
    cargo check

update:
    cargo update

format:
    cargo fmt

test:
    cargo test

clean:
    cargo clean

install:
    cargo install --path .

install-to path:
    cargo install --path . --root {{path}}

nix-dev:
    nix develop

build: format check
    cargo build

release: format test check
    cargo build --release

nix-build: build
    nix build

nix-install: release
    nix profile install .
