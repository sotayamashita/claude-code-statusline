# Tasks: Unfinished Items for Two-Crate Publish Plan

Derived from specs/2025-09-08-publish-2-crate-plan/01-spec.md. Track and complete before publish.

## Workspace Refactor
- [x] Convert root to a virtual workspace: remove `[package]` from root `Cargo.toml`.
- [x] Remove root shim crate: delete `src/main.rs` and `src/lib.rs` under repository root.
- [x] (Optional) Add `[workspace.package]` to centralize shared metadata (e.g., `edition = "2024"`, `license = "MIT"`, `repository`, `rust-version = "1.75"`).

## Crate Metadata (crates.io readiness)
- [x] `claude-code-statusline-core`: add `license`, `repository`, `rust-version`, `readme`, `keywords`, `categories` to `Cargo.toml`. Keep `description` accurate.
- [x] `claude-code-statusline-core`: add docs.rs config: `[package.metadata.docs.rs] all-features = true`.
- [x] `claude-code-statusline-cli`: add `description`, `license`, `repository`, `rust-version`, `readme`, `keywords`, `categories` to `Cargo.toml`.
- [x] Add crate-local `README.md` files for both `core` and `cli` (no cross-crate references).
- [x] (Optional) Restrict package contents via `include = ["src/**", "README.md", "LICENSE*"]`. If using `LICENSE*`, ensure a `LICENSE` file exists within each crate directory or adjust accordingly.

## Dependencies
- [x] In `crates/claude-code-statusline-cli/Cargo.toml`, specify `version` alongside `path` for `claude-code-statusline-core` dependency (e.g., `version = "0.1.0", path = "../claude-code-statusline-core"`).

## Docs & Scripts
- [x] Update root `README.md` install instructions to prefer `cargo install claude-code-statusline-cli` and document that it installs a `claude-code-statusline` binary.
- [x] Update `scripts/check_ansi.sh` to use `cargo run -p claude-code-statusline-cli -q` when not using the installed binary.
- [x] Add a migration note: the root shim crate is removed; import from `claude_code_statusline_core::...` instead of `claude_code_statusline::...`.
- [x] Search and replace any references to `-p claude-code-statusline` in scripts/docs with `-p claude-code-statusline-cli`.

## CI / Release Workflow
- [x] Add `.github/workflows/release.yml` with `workflow_dispatch` inputs: `version` (string) and `dry_run` (bool).
- [x] Release steps: setup Rust (1.75), cache → `cargo fmt --all -- --check` → `cargo clippy --workspace -- -D warnings` → `cargo test --workspace` → when not `dry_run`: `cargo workspaces version` to bump/tag → create GitHub Release → `cargo workspaces publish --from-git`.
- [x] Configure `CARGO_REGISTRY_TOKEN` secret;

## Versioning & Publishing
- [ ] Align crate versions to initial `0.1.0` using `cargo workspaces version 0.1.0 --all --force-publish` (or chosen version).
- [ ] `cargo publish --dry-run -p claude-code-statusline-core`.
- [ ] `cargo publish --dry-run -p claude-code-statusline-cli`.
- [ ] Publish order: `core` first, then `cli`.

## Validation / Acceptance
- [ ] `cargo install claude-code-statusline-cli` installs an executable named `claude-code-statusline`.
- [ ] Running `claude-code-statusline` with JSON via stdin produces ANSI-formatted output and passes existing tests.
- [ ] `cargo test --workspace` passes after refactor and metadata changes.
- [ ] docs.rs renders for `claude-code-statusline-core` with `all-features` enabled.
