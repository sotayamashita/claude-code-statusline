# Repository Guidelines

- Code should be in English
- Conversation should be in Japanese

## Project Structure & Module Organization
- Source: `src/` with `main.rs` (CLI entry) and `lib.rs` (shared). Domain modules live in `src/modules/` (e.g., `directory.rs`, `claude_model.rs`) and types in `src/types/` (config/context/claude).
- Tests: unit tests inline via `#[cfg(test)]` and shared test helpers under `tests/common/` using `rstest`.
- Docs & tooling: `docs/` for design notes, `hooks/` for git hooks, `Makefile` for setup, `Cargo.toml` for deps.

## Build, Test, and Development Commands
- `cargo build` — compile the project.
- `cargo run -q` — run the CLI; pipe JSON input via stdin (example below).
- `cargo test` — run unit and integration tests.
- `cargo fmt` / `cargo clippy -- -D warnings` — format and lint.
- `make install-hooks` — install the pre-commit hook (format, clippy, tests).
- Optional: `mise install` — install tool versions from `mise.toml` (Rust).

Example run:
`echo '{"cwd":"/tmp","model":{"id":"claude-opus","display_name":"Opus"}}' | cargo run -q`

## Coding Style & Naming Conventions
- Rust 2024 edition; use `rustfmt` defaults (4-space indent, max width per formatter).
- Naming: `snake_case` for functions/modules, `CamelCase` for types, `SCREAMING_SNAKE_CASE` for constants.
- Keep modules cohesive; implement the `Module` trait in `src/modules/*` and register via the dispatcher in `src/modules/mod.rs`.

## Testing Guidelines
- Frameworks: built-in `#[test]` and `rstest` for parametrization.
- Place integration-style helpers in `tests/common/`; prefer small, focused unit tests near the code.
- Run `cargo test` locally; pre-commit enforces format, lint, and tests.
- Aim to cover parser (`src/parser.rs`) and config (`src/config.rs`, `src/types/config.rs`) paths and common error cases.

## Commit & Pull Request Guidelines
- Follow Conventional Commits (e.g., `feat(parser): ...`, `fix: ...`, `docs: ...`).
- PRs should include: clear description, rationale, and testing notes. Link issues when applicable.
- Ensure CI is green and the pre-commit hook passes. Keep diffs focused and documented in `docs/` if you change behavior.

## Security & Configuration Tips
- Config loads from `~/.config/beacon.toml` (TOML). Do not commit local configs or secrets.
- The CLI reads JSON from stdin; avoid logging sensitive content unless `debug` is enabled in config.
