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

# Build workspace
cargo build --workspace --release

# Option A) Copy the built binary to your PATH
cp target/release/beacon ~/.local/bin/

# Option B) Install from the CLI crate
cargo install --path crates/beacon-cli
```

## Development

```bash
# Build all crates (debug)
cargo build --workspace

# Run CLI
cargo run -p beacon-cli -q -- --help

# Example run with JSON input
echo '{"session_id":"s","cwd":"/tmp","model":{"id":"claude-opus","display_name":"Opus"}}' | \
  cargo run -p beacon-cli -q --

# Benchmarks (criterion) and threshold check (< 50ms mean by default)
make bench
make bench-check

# Run benches/tests with feature flags (optional)
# Default benches run beacon-core without optional features.
# Enable Git modules when you need them in benches/tests:
cargo bench -p beacon-core --features git --no-run
cargo test  -p beacon-core --features git
```

## Feature Flags

- `git`: Enables Git-powered modules (`git_branch`, `git_status`). The CLI depends on
  `beacon-core` with `features = ["git"]`, so the `beacon` binary includes Git support by default.
  Library consumers and standalone benches/tests must enable it explicitly with
  `--features git` when needed.
- `parallel`: Enables Rayon-based parallel rendering (planned/optional).

## Configuration

### Claude Code:

```json
{
  "statusLine": {
    "type": "command",
    "command": "beacon",
    "padding": 0
  }
}
```

_[Lean more about Claude Code status line integration](https://docs.anthropic.com/en/docs/claude-code/statusline)_

### Beacon Settings:

#### Config File Location

```bash
~/.config/beacon.toml
```

#### Supported Modules

- `directory`
- `git_branch`
- `git_status`
- `claude_model`

#### Default Style

```toml
Format = "$directory $git_branch $git_status $claude_model"

[directory]
style = "bold cyan"

[git_branch]
style = "bold green"

[git_status]
style = "bold red"

[claude_model]
style = "bold yellow"
```

## Presets

### Pure Prompt (single-line, no linebreak)

A minimal, single-line preset inspired by Starship’s Pure Prompt.

```toml
format = "$directory $git_branch $git_status $claude_model"

[directory]
style = "bold blue"
truncation_length = 3
truncate_to_repo = true

[git_branch]
style = "bold green"
# symbol = ""   # if your font supports it; default is "🌿"

[git_status]
style = "bold red"

[claude_model]
style = "bold yellow"
symbol = "<"
```

See docs/presets/pure_prompt.md for details and notes.

## Acknowledgments

This project was inspired by [Starship](https://starship.rs/), the excellent cross-shell prompt. I've adapted its modular architecture for Claude Code's statusline.
