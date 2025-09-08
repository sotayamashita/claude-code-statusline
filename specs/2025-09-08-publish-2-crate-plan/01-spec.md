# Publish Plan: Two-Crate Workspace and CLI Binary Name

Status: Proposed
Owner: @sotayamashita
Created: 2025-09-08

## Summary

Convert the workspace to a two-crate layout and publish to crates.io:

- Keep only two crates:
  - `claude-code-statusline-core` (library): public API, types, modules
  - `claude-code-statusline-cli` (library + binary): CLI entry with a binary named `claude-code-statusline`
- Remove the current root package (binary + shim) and make the repository root a virtual Cargo workspace (no `[package]`).
- Publish flow: `core` → `cli`.
- Provide a manual (workflow_dispatch) GitHub Actions release/publish pipeline.

This lets users install the CLI via `cargo install claude-code-statusline-cli`, which installs an executable named `claude-code-statusline` (via `[[bin]].name`).

## Background

- The current repo has three crates: root (bin + shim), `core`, and `cli`.
- The shim re-exports `core` symbols under `claude_code_statusline` for backward compatibility, but it complicates publishing and installation.
- Issue #34 requests best practices for publishing within a workspace and a plan for crates.io.

## Goals

- Simplify packaging: publish only `core` and `cli`.
- Provide a stable, well-documented `core` API (`Engine`, `Config`, `parse_claude_input`, `Context`, `CoreError`).
- Ship a single end-user binary named `claude-code-statusline` from the `cli` crate.
- Standardize versioning and release across crates with minimal friction.
- Add a manual release workflow (tag + release notes + publish).

## Non-Goals

- Keeping the legacy root shim crate. This plan removes it. Any code importing `claude_code_statusline::...` must migrate to `claude_code_statusline_core::...`.
- Automated CHANGELOG generation (may be added later).

## Target Workspace Layout

After refactor:

```
claude-code-statusline/
├── Cargo.toml                 # virtual workspace root (no [package])
├── crates/
│   ├── claude-code-statusline-core/
│   │   ├── Cargo.toml         # lib: claude_code_statusline_core
│   │   └── src/
│   └── claude-code-statusline-cli/
│       ├── Cargo.toml         # lib + [[bin]] name = "claude-code-statusline"
│       └── src/
└── specs/
```

Key changes:

- Root `src/` and root `[package]` are removed. Root becomes a virtual workspace.
- `claude-code-statusline-cli` adds `src/main.rs` and `[[bin]]` so cargo installs an executable named `claude-code-statusline`.

## CLI Binary Details

- Add `crates/claude-code-statusline-cli/src/main.rs`:

```rust
fn main() -> anyhow::Result<()> {
    claude_code_statusline_cli::run()
}
```

- Add to `crates/claude-code-statusline-cli/Cargo.toml`:

```toml
[[bin]]
name = "claude-code-statusline"
path = "src/main.rs"
```

Install command:

```
cargo install claude-code-statusline-cli
# Installs binary: ~/.cargo/bin/claude-code-statusline
```

## Cargo and Versioning Strategy

- Use a single version for both crates (e.g., `0.1.0`) at initial publish.
- Convert root to a virtual workspace:
  - Remove root `[package]` and move to `[workspace]` only.
  - Optionally use `[workspace.package]` to centralize `version`, `edition`, `license`, `repository`, `rust-version`.
- Internal dependencies use "path + version" so publishing works:

```toml
# in cli/Cargo.toml
claude-code-statusline-core = { version = "0.1.0", path = "../claude-code-statusline-core", features = ["git"] }
```

- Features: keep `core` features (`git`, `parallel`); `cli` enables `git` for user-facing builds.

## Publishing Strategy

- Crates to publish:
  1) `claude-code-statusline-core` (library)
  2) `claude-code-statusline-cli` (library + binary)

- Order matters: publish `core` first, then `cli`.
- Pre-flight checks per crate: `cargo publish --dry-run`.
- Use `cargo-workspaces` to coordinate version bumps and publishing:
  - `cargo workspaces version <level> --all --force-publish` (creates commit + tags)
  - `cargo workspaces publish --from-git --yes --no-verify`

## GitHub Actions (Manual Release)

Workflow goals:

- Trigger: `workflow_dispatch` with inputs: `version`, `dry_run` (bool).
- Steps:
  1) Setup Rust (1.75), checkout, cache.
  2) `cargo fmt --all -- --check`, `cargo clippy --workspace -- -D warnings`, `cargo test --workspace`.
  3) `cargo workspaces version` (when not `dry_run`) → commit + tag.
  4) Generate basic release notes from git log (or leave placeholder); create GitHub Release for the tag.
  5) `cargo workspaces publish --from-git` (skip when `dry_run`).

Secrets:

- `CRATES_IO_TOKEN` for publish (set at repo/org level), used via `CARGO_REGISTRIES_CRATES_IO_TOKEN`.

## Metadata and Packaging

For both crates:

- Add to `Cargo.toml`:
  - `license = "MIT"`
  - `repository = "https://github.com/sotayamashita/claude-code-statusline"`
  - `rust-version = "1.75"`
  - `readme = "README.md"`
  - `description` (short and precise)
  - `keywords` (e.g., ["statusline", "cli", "developer-tools"]) and `categories` (e.g., ["command-line-utilities"])
- Provide crate-local `README.md` files (do not reference files outside the crate dir).
- Optionally restrict package contents via `include = ["src/**", "README.md", "LICENSE*"]`.
- For docs.rs in core:

```toml
[package.metadata.docs.rs]
all-features = true
```

## API Stability and Breaking Changes

- Removing the root shim crate is a breaking change for any external code importing `claude_code_statusline::...`.
- Migration path: import from `claude_code_statusline_core::...` instead.
- Since this repository has not yet published the shim to crates.io, the impact should be minimal. Document the change clearly in README and release notes.

## Risks and Mitigations

- Path-only dependencies fail to publish → always specify `version` alongside `path`.
- Publish order issues → enforce `core` → `cli` order; prefer `cargo workspaces publish --from-git`.
- docs.rs build with features → set `all-features = true` for core.
- Binary name vs crate name confusion → Document: `cargo install claude-code-statusline-cli` installs a `claude-code-statusline` binary.

## Rollout Plan

1) Workspace refactor
   - Make root a virtual workspace; remove root `src/` and `[package]`.
   - Add `[[bin]]` and `src/main.rs` to `cli` crate.
   - Update `cli` to depend on `core` via `path + version`.
2) Metadata and docs
   - Add crate-local README, license, repository, rust-version, keywords, categories.
   - Update top-level README with new install instructions.
3) CI
   - Add release workflow (manual trigger, dry-run support).
   - Ensure fmt/clippy/test gates.
4) Validation
   - `cargo publish --dry-run` for `core` and `cli`.
   - Optional: test `cargo install --path crates/claude-code-statusline-cli` locally.
5) Release
   - Tag and publish (`core` first, then `cli`).

## Acceptance Criteria

- `cargo install claude-code-statusline-cli` produces an executable `claude-code-statusline`.
- `claude-code-statusline` works with stdin JSON as before and passes existing tests.
- `claude-code-statusline-core` docs render on docs.rs with examples for `Engine`/`Config`.
- Manual GH workflow can tag and publish successfully (dry-run path verified).
