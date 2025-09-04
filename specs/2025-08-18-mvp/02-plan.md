# Beacon Roadmap (Overview)

> This document merges the `docs/init/*` (spec/plan/todo/statusline/refactoring etc.) and the existing code (src/*) to provide an overview of the implementation status and the future plans. The check boxes indicate the completion status: completed `- [x]` / not completed `- [ ]` .

## Summary
- [x] Phase 1: MVP (Implement the basic structure, test, and verify the functionality)
- [x] Phase 2: Core Features (Implement Git/Session/ANSI/Validation, etc.)
- [x] Phase 3: Polish (Improve quality, cache, timeout, documentation)
- [ ] Phase 4: Advanced (Parallel execution, custom error handling, benchmark, additional modules)
- [x] Refactoring (Perform the necessary refactoring for Phase 1, some were rejected)
- [x] Reduce unnecessary functionality (Remove the Character module, which is output-only)

---

## Phase 1: MVP (Week 1)
- [x] Basic CLI structure (introduction of `clap` and `--help`/`--version`)
- [x] JSON input parsing (using `serde`/`serde_json` and receive all fields)
- [x] `Module` trait and dispatcher (`modules/mod.rs::handle_module`)
- [x] Core modules (2 types: `directory`, `claude_model`)
- [x] Format parser (`$directory $claude_model` replacement)
- [x] Single-line output (non-line output using `print!` and stdout flushing)
- [x] Fallback for failure (`JSON parsing failure` and `empty input`)
- [x] Configuration file (loading `~/.config/beacon.toml` as TOML)
- [x] `Context` structure (aggregation of runtime information and configuration)
- [x] Debug logger (`src/debug.rs` with stderr/file output toggle)
- [x] Test (unit tests for `config.rs`, `parser.rs`, and each module)
- [x] Project structure adjustment (e.g., `src/types`, `src/modules`, `src/config.rs`, etc.)
- [x] Remove the Character module (not needed due to its output-only nature)

Note: ANSI colors will be handled in Phase 2 (currently plain text output).

---

## Phase 2: Core Features (Week 2)
- [x] Git branch module (`git2` assumed)
- [ ] Claude session module (display session state) (skipped in this phase)
- [x] Apply ANSI styles (`style` specification interpretation and decoration output)
- [x] Basic error handling extension (optimize user-facing messages)
- [x] Config validation (range of values and unexpected key detection)

Example (Configuration excerpt):
