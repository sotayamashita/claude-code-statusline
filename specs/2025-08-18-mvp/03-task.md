# Beacon - Detailed Tasks (Consolidated)

This document is the latest version of the consolidated plan and task documents. The latest progress and decisions are based on this file, and if necessary, differences are incorporated.

---

## Phase 1: MVP Task List (Integrated)

### Implemented Features
- ✅ Basic CLI structure (using clap)
- ✅ JSON input parsing (using serde/serde_json)
- ✅ Module trait and module system
- ✅ Two core modules (Directory, ClaudeModel)
- ✅ Single line output and Claude Code integration
- ✅ Error fallback display during error
- ✅ Configuration file (Config structure, reading from TOML)
- ✅ Project structure organization (types/, config.rs, modules/)
- ✅ Addition of tests (config.rs, parser.rs, modules)
- ✅ Refactoring of debugging functionality (DebugLogger module)
- ✅ Context structure (centralized management of application data and settings)
- ✅ Individual module settings (DirectoryConfig, ClaudeModelConfig)
- ✅ Error handling improvement (using anyhow)

### Detailed Tasks (Excerpt)
- CLI/Arguments and Help implementation (using clap)
- ClaudeInput/ModelInfo/WorkspaceInfo/OutputStyle type definition and stdin JSON reading
- Config Default/Loading/Test
- DebugLogger introduction and BEACON_DEBUG/Configuration interaction
- Context construction and individual module Contextization
- Directory/ClaudeModel module implementation (should_display/render)
- Generation process (generate_prompt) and single line output
- Basic integration test (test-input.json)

### Completion Criteria
- JSON input is processed correctly, and Directory/ClaudeModel works
- A single line status line is output, even in the case of error
- Configuration, tests, and debugging are organized, and Context is centralized

---

## Phase 2: Core Features (Detailed Tasks/TDD)



### Goals
- Additional modules (Git Branch is only used in this phase)
- Minimal application of ANSI styles
- Improved user-oriented error handling
- Introduction of Config validation

### 1) Git Branch Module ($git_branch)
- Red (Test)
  - Outside the repository: invisible
  - Inside the repository: branch name or short SHA
  - `disabled = true` implies invisible
- Green (Implementation)
  - Add dependency on git2, configuration type GitBranchConfig { format, style, symbol, disabled }
  - `should_display`: `Repository::discover` success and `!disabled`
  - `render`: evaluate format/style, display branch name/short SHA
  - Dispatch registration (modules/mod.rs)
- Refactor
  - Reduce test duplication, add boundary cases
- Acceptance Criteria
  - Git management outside is invisible, inside shows name/short SHA, `disabled` implies invisible

### ~~2) Claude Session Module ($claude_session)~~ (This phase is skipped)
- Future phase to be considered

### 3) ANSI style application (minimal)
- Red (Test)
  - `apply_style("X", "bold yellow")` applies ANSI
  - Unknown tokens are ignored
  - Directory/ClaudeModel reflects format/style
- Green (Implementation)
  - Add `src/style.rs`, provide `apply_style` and template evaluation
  - Each module evaluates its own format
- Refactor
  - Mapping tableization (bold/italic/underline/8 colors)
- Acceptance Criteria
  - Representative style specifications apply ANSI, unknowns do not cause a crash

### 4) Error Handling Improvement (message content/output destination)
- Red (Integration Test)
  - Invalid JSON/Invalid TOML: stdout is concise, stderr is detailed
- Green (Implementation)
  - DebugLogger error log consistency, cleanup of `main.rs`
- Refactor
  - Constantization of messages, i18n-considered naming

### 5) Config Validation
- Schema/Range/Unknown keys detection and error message refinement

---

## Phase 3: Polish (Quality Improvement)



### Goals
- Quality improvement (accuracy/perceived speed/hang resistance/readability)
- Easy operation by simplistic caching and timeout for stability operation
- Extensive testing and documentation enhancement across modules

### 1) Git Status Module ($git_status)
- Why/Effect
  - Lightweight display of status (changes/staging/untracked/ahead/behind) to reduce cognitive load
- Starship compliance with display/symbol/format
- Red/Green/Refactor (Key Points)
  - `git2` to get `statuses()` and compare with upstream
  - `$all_status` and `$ahead_behind` assembly, `count==0` is invisible
  - Dispatch registration, `disabled` support
  - Simple caching in conjunction with I/O suppression

### 2) Simple Caching
- Reduction of frequent retrievals (Git/Dir, etc.) through memoization
- Simple caching using `OnceLock`/`Lazy` for the same execution

### 3) Module Execution Timeout
- Prevention of hangs and user-configurable timeout wrapper for robust operation

### 4) Extensive Testing/Documentation Enhancement
- Boundary cases/module-spanning testing, User Guide/Developer Guide

---

## Refactoring (After Phase 1 & Improving Quality)



### High Priority (Done)
1. Module System Improvement (Central Dispatcher)
2. Module Trait Improvement (using `context`/`config` as arguments)
3. Format String Parser (Parsing `"$directory $claude_model"`)

### Medium Priority (Declined/Done)
4. Error Handling Highlighting (Partial success acceptance) → Declined
5. Module Config Separation → Declined
6. Test Helper (TestRenderer) → Done

---

Last update: After consolidating historical documents, this file is used as the only task foundation.
