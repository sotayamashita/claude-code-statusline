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
- TOML-based configuration (`~/.config/beacon.toml`)
- JSON input via stdin for dynamic context
- ANSI-formatted output for terminal display
- Performance-optimized with Rust 2024 edition

## Project Structure & Module Organization

### Directory Layout
```
beacon/
├── src/
│   ├── main.rs           # CLI entry point, stdin handler
│   ├── lib.rs            # Shared library code
│   ├── parser.rs         # JSON input validation
│   ├── config.rs         # Configuration loading
│   ├── types/            # Type definitions
│   │   ├── claude.rs     # Claude Code input types
│   │   └── config.rs     # Configuration types
│   └── modules/          # Status line components
│       ├── mod.rs        # Module dispatcher
│       ├── directory.rs  # Directory status
│       └── claude_model.rs # Model display
├── tests/
│   └── common/           # Shared test helpers
├── docs/                 # Design documentation
├── hooks/                # Git hooks
└── Cargo.toml           # Dependencies
```

### Module System
- Each status component implements the `Module` trait
- Modules are registered in `src/modules/mod.rs`
- Dynamic loading based on configuration
- Clear separation: types (`src/types/`) vs logic (`src/modules/`)

## Quick Start

### Prerequisites
- Rust 1.75+ (2024 edition)
- GNU Make (for git hooks)
- Optional: mise for version management

### Installation
```bash
# Clone and build
git clone <repository>
cd beacon
cargo build --release

# Install git hooks
make install-hooks

# Copy binary to PATH (example)
cp target/release/beacon ~/.local/bin/
```

## Development Commands

### Build & Run
```bash
cargo build                    # Debug build
cargo build --release          # Optimized build
cargo run -q                   # Run quietly (suppress cargo output)
cargo run -- --help            # Show CLI help
```

### Testing & Quality
```bash
cargo test                     # Run all tests
cargo test -- --nocapture      # Show test output
cargo fmt                      # Format code
cargo fmt -- --check           # Check formatting without changes
cargo clippy -- -D warnings    # Lint with warnings as errors
cargo doc --open               # Generate and open documentation
```

### Example Usage
```bash
# Basic run with JSON input
echo '{"cwd":"/tmp","model":{"id":"claude-opus","display_name":"Opus"}}' | cargo run -q

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
}' | cargo run -q
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
- Parser logic (`src/parser.rs`) - JSON input validation
- Configuration (`src/config.rs`, `src/types/config.rs`) - TOML parsing and defaults
- Module implementations (`src/modules/*.rs`) - Status component logic
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
- Phase 1: `anyhow::Result` for rapid development
- Phase 2+: Migrate to `thiserror` for structured errors
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
- User config: `~/.config/beacon.toml`
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
4. **Config not loading**: Check `~/.config/beacon.toml` syntax

### Debug Mode
Enable debug output in config:
```toml
[general]
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

### Integration Points
- Input: JSON via stdin
- Output: ANSI-formatted text to stdout
- Config: TOML at `~/.config/beacon.toml`
- Logs: Controlled by debug flag

---
*Last updated: 2025-09-04*
*Beacon - Fast, modular status line for AI development*
