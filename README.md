# claude-code-statusline

[![CI](https://github.com/sotayamashita/claude-code-statusline/actions/workflows/ci.yml/badge.svg)](https://github.com/sotayamashita/claude-code-statusline/actions/workflows/ci.yml)

## Overview
<!-- LLM Instructions: Update @specs/project.md when you change this section -->

**claude-code-statusline** is a lightweight, high-performance status line generator written in Rust, designed for AI-powered development environments. It provides a starship-like configuration experience.

## Motivation

claude-code-statusline exists to provide a fast, embeddable status line specifically tailored for Claude Code. I deeply respect and admire Starship for setting the bar on modular, configurable prompts across shells. However, Starship is intentionally delivered as a standalone CLI and does not expose a stable, supported Rust library API or a general plugin API for embedding its internals into other binaries. Because I need programmatic composition with JSON input and tight integration within AI-driven editor workflows, I built claude-code-statusline as a small Rust library/binary that borrows Starship’s proven ideas (modules, formatting, styling) while remaining easy to integrate as part of a larger toolchain.

## Installation

```bash
# From crates.io (preferred; installs `claude-code-statusline` binary)
cargo install claude-code-statusline-cli

# From source
git clone https://github.com/sotayamashita/claude-code-statusline.git && cd claude-code-statusline
cargo build --workspace --release
cp target/release/claude-code-statusline ~/.local/bin/
```

## Development

```bash
# Build all crates (debug)
cargo build --workspace

# Run CLI
cargo run -p claude-code-statusline-cli -q -- --help

# Example run with JSON input
echo '{"session_id":"s","cwd":"/tmp","model":{"id":"claude-opus","display_name":"Opus"}}' | \
  cargo run -p claude-code-statusline-cli -q --

# Benchmarks (criterion) and threshold check (< 50ms mean by default)
make bench
make bench-check

# Run benches/tests with feature flags (optional)
# Default benches run claude-code-statusline-core without optional features.
# Enable Git modules when you need them in benches/tests:
cargo bench -p claude-code-statusline-core --features git --no-run
cargo test  -p claude-code-statusline-core --features git
```

## Feature Flags

- `git`: Enables Git-powered modules (`git_branch`, `git_status`). The CLI depends on
  `claude-code-statusline-core` with `features = ["git"]`, so the `claude-code-statusline` binary includes Git support by default.
  Library consumers and standalone benches/tests must enable it explicitly with
  `--features git` when needed.
- `parallel`: Enables Rayon-based parallel rendering (planned/optional).

## Claude Code Configuration

```json
{
  "statusLine": {
    "type": "command",
    "command": "claude-code-statusline",
    "padding": 0
  }
}
```

_**[Learn more about Claude Code status line integration](https://docs.anthropic.com/en/docs/claude-code/statusline)**_

## Configuration

### Config File Location

```bash
~/.config/claude-code-statusline.toml
```

### Supported Modules

- `directory`
- `git_branch`
- `git_status`
- `claude_model`

### Default Style

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

### Presets Styles

- Pure Prompt (single-line, no linebreak)
    - _**[See docs/presets/pure_prompt.md.](docs/presets/pure_prompt.md)**_
- Pastel Powerline
    - _**[See docs/presets/pastel_powerline.md](docs/presets/pastel_powerline.md)**_
- Tokyo Night
    - _**[See docs/presets/tokyo_night.md](docs/presets/tokyo_night.md)**_

## Acknowledgments

This project was inspired by [Starship](https://starship.rs/), the excellent cross-shell prompt. I've adapted its modular architecture for Claude Code's statusline.

## Migration

The legacy root shim crate has been removed. If you previously imported
`claude_code_statusline::...`, please migrate to `claude_code_statusline_core::...`.
