# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Beacon is a lightweight status line generator for Claude Code, written in Rust. It provides a starship-like configuration experience optimized for Claude Code's statusLine feature.

## Commands

### Build & Run
```bash
# Build the project
cargo build
cargo build --release

# Run the project
cargo run
cargo run -- --help

# Test with mock JSON input
echo '{"model":{"display_name":"Opus"},"workspace":{"current_dir":"/home/user/project"}}' | cargo run
```

### Development
```bash
# Check code without building
cargo check

# Format code
cargo fmt

# Run linter
cargo clippy

# Run tests
cargo test

# Generate documentation
cargo doc --open
```

### Testing Claude Code Integration
```bash
# Build release binary
cargo build --release

# Test status line directly
echo '{"hook_event_name":"Status","session_id":"abc123","transcript_path":"/path/to/transcript.json","cwd":"/current/directory","model":{"id":"claude-opus","display_name":"Opus"},"workspace":{"current_dir":"/current/directory","project_dir":"/project/root"},"version":"1.0.0","output_style":{"name":"default"}}' | ./target/release/beacon
```

## Architecture

### Project Structure
- @src/main.rs - Entry point, handles JSON from stdin
- @src/config/ - TOML configuration management
- @src/modules/ - Status line modules (directory, git, model display)
- @src/context.rs - Shared execution context
- @docs/ - Comprehensive documentation

### Key Design Patterns
- **Module System**: Each status component implements the Module trait
- **Error Handling**: Use `anyhow::Result` for Phase 1, migrate to `thiserror` later
- **Configuration**: TOML-based, starship-compatible subset
- **Performance Target**: < 50ms execution time

### Development Phase

See @docs/todo.md for detailed task list and @docs/plan.md for development phases.

## Claude Code Integration

Beacon receives JSON via stdin from Claude Code containing:
- Model information (id, display_name)
- Workspace paths (current_dir, project_dir)
- Session metadata (session_id, version)

Output is a single ANSI-formatted line to stdout, updated max every 300ms.

Configuration location: `~/.config/beacon/config.toml`

## Development Guidelines

- Follow Rust conventions (snake_case, PascalCase for types)
- Never panic in production - use graceful error handling
- Test with various JSON inputs to ensure robustness
- Keep execution time under 50ms
- Document public APIs with `///` comments
