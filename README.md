# Beacon

[![CI](https://github.com/sotayamashita/beacon/actions/workflows/ci.yml/badge.svg)](https://github.com/sotayamashita/beacon/actions/workflows/ci.yml)

## Overview
<!-- LLM Instructions: Update @specs/project.md when you change this section -->

**Beacon** is a lightweight, high-performance status line generator written in Rust, designed for AI-powered development environments. It provides a starship-like configuration experience.

## Motivation

Beacon exists to provide a fast, embeddable status line specifically tailored for Claude Code. I deeply respect and admire Starship for setting the bar on modular, configurable prompts across shells. However, Starship is intentionally delivered as a standalone CLI and does not expose a stable, supported Rust library API or a general plugin API for embedding its internals into other binaries. Because I need programmatic composition with JSON input and tight integration within AI-driven editor workflows, I built Beacon as a small Rust library/binary that borrows Starship’s proven ideas (modules, formatting, styling) while remaining easy to integrate as part of a larger toolchain.

## Installation

```bash
# Clone the repository
git clone https://github.com/sotayamashita/beacon.git && cd beacon

# Build and install
cargo build --release

# Install the binary
cargo install --path .
```

## Project Structure

```
beacon/
├── crates/
│   ├── beacon-core/   # Core library (engine, modules, types)
│   └── beacon-cli/    # CLI (stdin→stdout) + subcommands
├── tests/             # Integration tests
├── docs/              # Design documentation
└── Cargo.toml         # Workspace manifest
```

## Development

```bash
# Build all crates
cargo build

# Run CLI
cargo run -q -p beacon-cli -- --help

# Example run with JSON input
echo '{"session_id":"s","cwd":"/tmp","model":{"id":"claude-opus","display_name":"Opus"}}' | \
  cargo run -q -p beacon-cli --

# Tests, fmt, clippy
cargo test
cargo fmt
cargo clippy -- -D warnings
```

## Configuration

Claude Code のステータスラインに表示するための設定

```json
{
  "statusLine": {
    "type": "command",
    "command": "beacon",
    "padding": 0
  }
}
```

Becon の見た目を設定する

## Acknowledgments

This project was inspired by [Starship](https://starship.rs/), the excellent cross-shell prompt. I've adapted its modular architecture for Claude Code's statusline.
