# Repository Guidelines

> **Purpose**: This file serves as the single source of truth for all AI agents working with this repository.
> Tool-specific configurations should reference this file and add only their unique requirements.

## Language Policy
- **Code**: English only (variables, functions, comments, documentation)
- **Conversation**: Japanese (日本語での会話を推奨)

## Project Overview

Refer to `specs/project.md` for a high-level overview.

### Key Features
- Modular architecture with pluggable status components
- TOML-based configuration (`~/.config/claude-code-statusline.toml`)
- JSON input via stdin for dynamic context
- ANSI-formatted output for terminal display
- Performance-optimized with Rust 2024 edition

## Project Structure & Module Organization

### Directory Layout
```
claude-code-statusline/
├── crates/
│   ├── claude-code-statusline-core/                # Core library (public API, types, modules)
│   │   ├── src/
│   │   │   ├── lib.rs              # pub use {Engine, Config, parse_claude_input, Context}
│   │   │   ├── engine.rs           # Rendering engine
│   │   │   ├── parser.rs           # JSON input / format helpers
│   │   │   ├── config.rs           # Config loading (TOML)
│   │   │   ├── error.rs            # CoreError (thiserror)
│   │   │   ├── messages.rs         # Centralized messages/warnings
│   │   │   ├── style.rs            # ANSI style renderer
│   │   │   ├── timeout.rs          # Timeout utilities
│   │   │   ├── types/              # Type definitions
│   │   │   │   ├── claude.rs       # Claude Code input types
│   │   │   │   ├── config.rs       # Configuration types (with defaults)
│   │   │   │   └── context.rs      # Runtime context/memoization
│   │   │   └── modules/            # Status line components
│   │   │       ├── mod.rs          # Trait, dispatcher + timeout wrapper
│   │   │       ├── registry.rs     # ModuleFactory/Registry
│   │   │       ├── directory.rs    # Directory status
│   │   │       ├── claude_model.rs # Model display
│   │   │       ├── git_branch.rs   # Git branch (feature = "git")
│   │   │       └── git_status.rs   # Git status (feature = "git")
│   │   └── benches/engine_bench.rs # Criterion bench (engine)
│   └── claude-code-statusline-cli/                 # CLI (stdin→stdout、サブコマンド)
│       └── src/lib.rs              # `run()` entry
├── src/main.rs                     # `claude-code-statusline` binary
├── tests/                          # Integration tests (E2E)
│   ├── common/
│   ├── engine_api.rs
│   ├── integration_smoke.rs
│   ├── integration_timeout.rs
│   ├── error_handling.rs
│   └── cli_subcommands.rs
├── docs/                           # Design & development docs
├── hooks/                          # Git hooks
├── scripts/bench_check.py          # Bench threshold gate
└── Cargo.toml                      # Workspace root
```

### Module System
- Each status component implements the `Module` trait
- Dynamic creation via `Registry` + `ModuleFactory` (`crates/claude-code-statusline-core/src/modules/registry.rs`)
- Feature gates:
  - `git` enables `git_branch` / `git_status` (optional `git2` dep)
  - `parallel` enables Rayon-based parallel rendering
- Config-driven loading based on `$tokens` found in `Config.format`
- Clear separation: types (`.../types/`) vs logic (`.../modules/`)

## Quick Start

### Prerequisites
- Rust 1.75+ (2024 edition)
- GNU Make (for git hooks)
- Optional: mise for version management

### Installation
```bash
# Clone and build
git clone <repository>
cd claude-code-statusline
cargo build --workspace --release

# Install git hooks
make install-hooks

# Copy binary to PATH (example)
cp target/release/claude-code-statusline ~/.local/bin/
```

## Development Commands

### Build & Run
```bash
cargo build --workspace                 # Build all crates (debug)
cargo build --workspace --release       # Optimized build
cargo run -p claude-code-statusline-cli -q              # Run CLI quietly
cargo run -p claude-code-statusline-cli -- --help       # Show CLI help
```

### Testing & Quality
```bash
cargo test --workspace                     # Run all tests
cargo test --workspace -- --nocapture      # Show test output
cargo fmt --all                            # Format code
cargo fmt --all -- --check                 # Check formatting without changes
cargo clippy --workspace -- -D warnings    # Lint with warnings as errors
cargo doc --open                           # Generate and open documentation
make bench                                 # Run criterion bench (core)
make bench-check                           # Enforce mean < 50ms (default)
```

### Example Usage
```bash
# Basic run with JSON input
echo '{"cwd":"/tmp","session_id":"abc","model":{"id":"claude-opus","display_name":"Opus"}}' | cargo run -p claude-code-statusline-cli -q

# With full context
echo '{
  "session_id": "abc123",
  "cwd": "/project",
  "model": {
    "id": "claude-3.5-sonnet",
    "display_name": "Sonnet"
  },
  "workspace": {
    "current_dir": "/project/src",
    "project_dir": "/project"
  }
}' | cargo run -p claude-code-statusline-cli -q
```

### CLI Subcommands
```bash
# Config helpers
claude-code-statusline config --path        # Show config path (~/.config/claude-code-statusline.toml)
claude-code-statusline config --default     # Print default TOML
claude-code-statusline config --validate    # Validate current config (OK/INVALID)

# Module insights
claude-code-statusline modules --list       # List all registered modules
claude-code-statusline modules --enabled    # List modules enabled by current format/config
```

## Coding Style & Naming Conventions

### Rust Standards
- **Edition**: Rust 2024
- **Formatting**: `rustfmt` defaults (4-space indent)
- **Linting**: `clippy` with warnings as errors

### Naming Conventions
- **Functions/Modules**: `snake_case`
- **Types/Structs**: `CamelCase`
- **Constants**: `SCREAMING_SNAKE_CASE`
- **Lifetimes**: `'a`, `'b` (short, lowercase)

### Code Organization
- Keep modules cohesive and focused
- Prefer composition over inheritance
- Use traits for shared behavior
- Early returns to reduce nesting

## Testing Strategy

### Framework & Tools
- **Unit Tests**: Built-in `#[test]` attributes, colocated with code
- **Parametrized Tests**: `rstest` for data-driven testing
- **Integration Tests**: `tests/` directory with shared helpers in `tests/common/`
- **Pre-commit Hooks**: Automatic format, lint, and test on commit

### Test Coverage Focus
- Parser logic (`crates/claude-code-statusline-core/src/parser.rs`) - JSON input validation
- Configuration (`crates/claude-code-statusline-core/src/config.rs`, `.../types/config.rs`) - TOML parsing/validation
- Module implementations (`crates/claude-code-statusline-core/src/modules/*.rs`) - Status component logic
- Error paths and edge cases

### Writing Tests
```rust
// Unit test example
#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[rstest]
    #[case("input1", "expected1")]
    #[case("input2", "expected2")]
    fn test_parametrized(#[case] input: &str, #[case] expected: &str) {
        assert_eq!(process(input), expected);
    }
}
```

## Architecture & Design Patterns

### Error Handling
- Core: Structured `CoreError` via `thiserror`（`crates/claude-code-statusline-core/src/error.rs`）
- CLI: Boundary uses `anyhow::Result<()>`; logs via `tracing` to stderr
- Never panic in production - graceful degradation
- Informative error messages with context

### Performance Targets
- Execution time: < 50ms
- Memory usage: Minimal heap allocations
- Binary size: Optimized with release builds
- Startup time: Near-instant

### Configuration System
- TOML-based configuration
- Hierarchical with defaults
- Runtime reloadable
- Validation on load
 - Unknown sections preserved via `#[serde(flatten)]` as `extra_modules`
 - `ConfigProvider` exposes extra module tables for pluggable modules

## Commit & Pull Request Guidelines

### Commit Messages
Follow Conventional Commits specification:
```
type(scope): description

[optional body]

[optional footer]
```

Types: `feat`, `fix`, `docs`, `style`, `refactor`, `test`, `chore`, `perf`

Examples:
```
feat(parser): add support for workspace context
fix(config): handle missing TOML file gracefully
docs: update README with installation instructions
```

### Pull Request Process
1. Create feature branch from `main`
2. Write tests for new functionality
3. Ensure all tests pass locally
4. Run formatter and linter
5. Update documentation if needed
6. Submit PR with clear description
7. Link related issues

### Code Review Checklist
- [ ] Tests cover new code paths
- [ ] Documentation updated
- [ ] No security vulnerabilities
- [ ] Performance impact considered
- [ ] Breaking changes documented

## Security & Configuration

### Configuration Management
- User config: `~/.config/claude-code-statusline.toml`
- Never commit local configurations or secrets
- Use environment variables for sensitive data
- Validate all external input

### Input Handling
- JSON input via stdin - validate schema
- Sanitize file paths and system calls
- Debug logging controlled by config flag
- No sensitive data in default output

### Security Best Practices
- Principle of least privilege
- Input validation before processing
- No arbitrary code execution
- Secure defaults

## Dependency Management

### Adding Dependencies
```bash
cargo add <package>           # Add latest version
cargo add --dev <package>     # Add dev dependency
cargo rm <package>            # Remove dependency
```

### Important Rules
- **ALWAYS** use `cargo add` command (never edit Cargo.toml directly)
- Let cargo choose compatible versions
- Review dependency licenses
- Minimize external dependencies
- Audit for security vulnerabilities

## Documentation Standards

### Code Documentation
```rust
/// Public API documentation
///
/// # Examples
/// ```
/// let result = function(input);
/// ```
pub fn function(input: &str) -> Result<String> {
    // Implementation
}
```

### Documentation Structure
- `README.md`: Project overview and quick start
- `docs/`: Design decisions and architecture notes
- `specs/`: Feature specifications and planning
- API docs: Generated via `cargo doc`

### Documentation Rules
- Document WHY, not just WHAT
- Include examples for complex APIs
- Keep documentation current
- Use clear, concise language

## Performance Guidelines

### Optimization Priorities
1. Correctness first
2. Readability second
3. Performance third

### Performance Tips
- Profile before optimizing
- Use `cargo build --release` for benchmarks
- Minimize allocations in hot paths
- Prefer iterators over collecting
- Use `&str` over `String` when possible

## Troubleshooting

### Common Issues
1. **Build fails**: Check Rust version (`rustc --version`)
2. **Tests fail**: Run with `--nocapture` for details
3. **Slow performance**: Build with `--release`
4. **Config not loading**: Check `~/.config/claude-code-statusline.toml` syntax

### Debug Mode
Enable debug output in config:
```toml
debug = true
```

## Contributing

### Getting Started
1. Read this document thoroughly
2. Install development tools
3. Run test suite
4. Make changes in feature branch
5. Submit PR with tests

### Where to Contribute
- Bug fixes: Always welcome
- Features: Discuss in issue first
- Documentation: Improvements appreciated
- Tests: Increase coverage

## AI Agent Notes

### For All AI Agents
- This is the authoritative source for project guidelines
- Tool-specific files should reference this document
- Keep responses focused and relevant
- Follow the established patterns

### Shell Tooling Rubric
- Find files: `fd`
- Find text: `rg` (ripgrep)
- Find code structure (TS/TSX): `ast-grep`
  - Default to TypeScript:
    - `.ts` → `ast-grep --lang ts -p '<pattern>'`
    - `.tsx` (React) → `ast-grep --lang tsx -p '<pattern>'`
  - For other languages, set `--lang` appropriately (e.g., `--lang rust`).
- Select among matches: pipe to `fzf`
- JSON: `jq`
- YAML/XML: `yq`

Notes:
- If `ast-grep` is available, prefer it for code-structure queries and avoid `rg`/`grep` unless a plain-text search is explicitly requested.

### Integration Points
- Input: JSON via stdin
- Output: ANSI-formatted text to stdout
- Config: TOML at `~/.config/claude-code-statusline.toml`
- Logs: `tracing` to stderr, controlled by debug flag

---
*Last updated: 2025-09-09*
*claude-code-statusline - Fast, modular status line for AI development*
