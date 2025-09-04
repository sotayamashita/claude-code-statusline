# Beacon - Specification Document

## 1. Overview

**Beacon** is a lightweight, customizable status line generator for Claude Code, written in Rust. It provides a starship-like configuration experience while being specifically optimized for Claude Code's statusLine feature.

### 1.1 Goals
- Provide a blazing-fast status line generator for Claude Code
- Support TOML-based configuration similar to starship.rs
- Modular architecture for easy extension
- Minimal dependencies and resource usage
- Learning project for Rust development
- **Leverage Claude Code unique features** (session_id, model info)

### 1.2 Non-Goals
- Shell prompt replacement (Claude Code specific)
- Full backward compatibility with starship configurations
- Support for all shells (focused on Claude Code integration)

### 1.3 Starship Compatibility Level
- **Format syntax**: Partial compatibility with `[$variable](style)` notation
- **Configuration**: Subset of Starship modules and options
- **Not compatible**: Advanced features like conditionals, custom commands

## 2. Architecture

### 2.1 System Overview
```
Claude Code
    ↓ (JSON via stdin)
beacon CLI
    ↓ (reads config.toml)
Module System
    ↓ (execute modules)
Formatter
    ↓ (ANSI output)
Claude Code (display)
```

### 2.2 Core Components

```rust
beacon/
├── src/
│   ├── main.rs           // Entry point & CLI handling
│   ├── config/
│   │   ├── mod.rs        // Configuration management
│   │   └── schema.rs     // Config validation
│   ├── modules/
│   │   ├── mod.rs        // Module trait & registry
│   │   ├── git_branch.rs // Git branch module
│   │   ├── directory.rs  // Directory module
│   │   └── ...          // Other modules
│   ├── context.rs        // Execution context
│   ├── formatter.rs      // Output formatting
│   └── utils/
│       ├── mod.rs
│       └── timeout.rs    // Timeout utilities
├── config/
│   └── default.toml      // Default configuration
└── Cargo.toml
```

## 3. Configuration

### 3.1 Configuration File Location
```
~/.config/beacon.toml
```

### 3.2 Configuration Validation

- TOML schema validation on load
- Type checking for all configuration values
- Automatic merging with default values
- Environment variable overrides support

### 3.3 Configuration Builder Pattern

```rust
// Type-safe configuration building
let config = ConfigBuilder::new()
    .timeout(Duration::from_millis(500))
    .format("$directory$git_branch$character")
    .module("directory", |m| {
        m.style("blue bold")
         .truncation_length(3)
    })
    .build()?;
```

### 3.4 Configuration Schema

```toml
# Global settings
command_timeout = 500  # milliseconds
# Starship-compatible format syntax with Claude-specific modules
format = "$directory$git_branch$claude_model$character"
add_newline = false

# Module: Directory
[directory]
style = "blue bold"
format = "[$path]($style) "
truncation_length = 3
truncate_to_repo = true
home_symbol = "~"
read_only = "🔒"

# Module: Git Branch (Starship-compatible syntax)
[git_branch]
style = "green bold"
symbol = "🌱 "
# Starship format: [$variable](style) notation
format = "[$symbol$branch]($style) "
truncation_length = 20
truncation_symbol = "..."

# Module: Claude Model (Beacon-specific)
[claude_model]
style = "cyan bold"
format = "[<$model>]($style) "
show_version = true

# Module: Git Status
[git_status]
format = "[$all_status$ahead_behind]($style) "
style = "red bold"
conflicted = "🏳"
ahead = "⇡${count}"
behind = "⇣${count}"
diverged = "⇕"
untracked = "?"
stashed = "$"
modified = "!"
staged = "+"
renamed = "»"
deleted = "✘"

# Module: Character
[character]
success_symbol = "[❯](bold green)"
error_symbol = "[❯](bold red)"
format = "$symbol "
```

## 4. Module System

### 4.1 Module Interface

```rust
pub trait Module: Send + Sync {
    /// Module name
    fn name(&self) -> &'static str;
    
    /// Check if module should be displayed
    fn should_display(&self, context: &Context) -> bool;
    
    /// Generate module output
    fn render(&self, context: &Context, config: &ModuleConfig) -> Result<String, Error>;
    
    /// Default timeout for this module
    fn default_timeout(&self) -> Duration {
        Duration::from_millis(100)
    }
    
    /// Module execution priority (lower = higher priority)
    fn priority(&self) -> u8 {
        100
    }
    
    /// Cache key for this module's output
    fn cache_key(&self, context: &Context) -> Option<String> {
        None
    }
}
```

### 4.2 Module Registration Pattern (Starship-inspired)

```rust
// src/modules/mod.rs
pub fn handle_module<'a>(
    name: &str,
    context: &'a Context,
) -> Option<Box<dyn Module + 'a>> {
    match name {
        "directory" => directory::create(context),
        "git_branch" => git_branch::create(context),
        "character" => character::create(context),
        "git_status" => git_status::create(context),
        _ => None,
    }
}

// Module creation pattern
pub fn create(context: &Context) -> Option<Box<dyn Module>> {
    // Early return pattern for detection
    let repo = context.get_repo().ok()?;
    
    Some(Box::new(GitBranchModule {
        // module initialization
    }))
}
```

### 4.3 String Formatter System

```rust
// Format string variables
formatter
    .map("path", |ctx| Some(ctx.current_dir.to_string()))
    .map("branch", |ctx| ctx.get_branch())
    .map("symbol", |_| Some("🌱"))
    .map_style("style", |config| Some(config.style))
    .parse()?;
```

### 4.4 Core Modules (Phase 1 - MVP)

| Module | Description | Priority |
|--------|-------------|----------|
| `directory` | Current working directory | P0 |
| `git_branch` | Active git branch | P0 |
| `character` | Prompt character | P0 |

### 4.5 Claude Code Specific Modules

| Module | Description | Priority |
|--------|-------------|----------|
| `claude_model` | Display current model (e.g., "Opus 4.1") | P0 |
| `claude_session` | Session ID indicator | P1 |
| `claude_context` | Context window usage | P2 |

### 4.6 Standard Modules (Phase 2+)

| Module | Description | Priority |
|--------|-------------|----------|
| `python` | Python version & venv | P2 |
| `nodejs` | Node.js version | P2 |
| `rust` | Rust version | P2 |
| `docker` | Docker context | P2 |
| `time` | Current time | P2 |
| `battery` | Battery status | P3 |
| `custom` | User-defined commands | P3 |

## 5. Input/Output

### 5.1 Input (from Claude Code)

Claude Code sends comprehensive session information via stdin in JSON format:

```json
{
  "hook_event_name": "Status",
  "session_id": "abc123...",
  "transcript_path": "/path/to/transcript.json",
  "cwd": "/current/working/directory",
  "model": {
    "id": "claude-opus-4-1",
    "display_name": "Opus"
  },
  "workspace": {
    "current_dir": "/current/working/directory",
    "project_dir": "/original/project/directory"
  },
  "version": "1.0.80",
  "output_style": {
    "name": "default"
  }
}
```

#### Input Field Descriptions

| Field | Type | Description | Always Present |
|-------|------|-------------|----------------|
| `hook_event_name` | string | Fixed value "Status" | Yes |
| `session_id` | string | Unique session identifier | Yes |
| `transcript_path` | string | Path to conversation transcript | Yes |
| `cwd` | string | Current working directory | Yes |
| `model.id` | string | Model identifier (e.g., "claude-opus-4-1") | Yes |
| `model.display_name` | string | Human-readable model name | Yes |
| `workspace.current_dir` | string | Current working directory | Yes |
| `workspace.project_dir` | string | Original project directory | Yes |
| `version` | string | Claude Code version | Yes |
| `output_style.name` | string | Output style name | Yes |

### 5.2 Output Format

```
ANSI-formatted string (single line)
Example: "~/projects/beacon 🌱 main <Opus>"
```

### 5.3 ANSI Color Support

- 8-bit color support
- Bold, italic, underline
- Background colors
- RGB support (optional)

### 5.4 Status Line Update Behavior

- Updates triggered when conversation messages update
- Maximum update frequency: once every 300ms
- First line of stdout becomes the status line
- Subsequent lines are ignored
- Empty output results in no status line display

## 6. CLI Interface

### 6.1 Commands

```bash
# Main command (called by Claude Code)
beacon

# Configuration management
beacon config --path         # Show config file path
beacon config --default      # Print default configuration
beacon config --validate     # Validate current config

# Module information
beacon modules --list        # List available modules
beacon modules --enabled     # Show enabled modules

# Debugging
beacon --version            # Show version
beacon --debug              # Enable debug output
beacon --benchmark          # Show timing information
```

### 6.2 Environment Variables

```bash
BEACON_CONFIG      # Override config file location
BEACON_LOG         # Log level (error, warn, info, debug, trace)
BEACON_CACHE_DIR   # Cache directory location
```

## 7. Performance Requirements

### 7.1 Timing Constraints
- Total execution time: < 50ms (typical), 100ms acceptable
- Module timeout: configurable (default 100ms)
- Config parsing: < 5ms
- Cached when possible
- Status line updates: Maximum once every 300ms (Claude Code limit)

### 7.2 Resource Usage
- Memory: < 10MB
- CPU: Minimal
- Disk I/O: Minimize with caching

### 7.3 Optimization Strategies
- **Parallel Processing**: Execute independent modules concurrently using `rayon`
- **Lazy Initialization**: Use `once_cell` for caching expensive operations (e.g., Git repository info)
- **Compile-time Optimization**: Enable LTO and single codegen unit for release builds
- **Smart Caching**: Cache module outputs based on context-specific keys

### 7.4 Caching Implementation

```rust
use once_cell::sync::Lazy;
use std::sync::Mutex;
use std::collections::HashMap;

// Global cache for module outputs
static MODULE_CACHE: Lazy<Mutex<HashMap<String, CacheEntry>>> = 
    Lazy::new(|| Mutex::new(HashMap::new()));

struct CacheEntry {
    value: String,
    timestamp: std::time::Instant,
    ttl: Duration,
}

// Cache usage in module
fn render(&self, context: &Context, config: &ModuleConfig) -> Result<String> {
    if let Some(cache_key) = self.cache_key(context) {
        if let Some(cached) = get_cached_value(&cache_key) {
            return Ok(cached);
        }
    }
    
    let result = self.compute_output(context, config)?;
    
    if let Some(cache_key) = self.cache_key(context) {
        cache_value(&cache_key, &result, Duration::from_secs(5));
    }
    
    Ok(result)
}
```

## 8. Error Handling

### 8.1 Error Strategy
- Never panic in production
- Graceful degradation
- Minimal fallback display
- Log errors to stderr (if BEACON_LOG set)

### 8.2 Error Type Architecture
- **Library Code**: Use `thiserror` for strongly-typed custom errors
- **Application Code**: Use `anyhow` for error context and propagation
- **Module Errors**: Implement custom error types with recovery strategies

### 8.3 Fallback Behavior

```rust
// If critical error occurs
"$ "  // Minimal prompt

// If module fails
// Skip module, continue with others

// If config invalid
// Use default configuration with warning logged

// If script not executable
// Log error to stderr, return empty output
```

### 8.4 Error Handling Patterns (Rust Beginner-Friendly)

```rust
// Phase 1: Simple error handling with Box<dyn Error>
pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

fn main() -> Result<()> {
    let config = load_config()?;
    let output = generate_prompt(config)?;
    println!("{}", output);
    Ok(())
}

// Phase 2: Custom error types with thiserror
#[derive(Debug, thiserror::Error)]
enum BeaconError {
    #[error("Configuration error: {0}")]
    Config(#[from] toml::de::Error),
    
    #[error("Module error: {module}")]
    Module { module: String, source: anyhow::Error },
    
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

// Early return pattern
fn should_display(&self, context: &Context) -> bool {
    let Some(repo) = context.get_repo() else {
        return false;
    };
    
    repo.is_inside_work_tree()
}
```

## 9. Development Phases

See @docs/plan.md for detailed development phases and milestones.

## 10. Testing Strategy

### 10.1 Unit Tests
- Each module individually
- Config parsing and validation
- Formatting functions
- Error handling paths

### 10.2 Integration Tests
- Full CLI execution
- Claude Code integration
- Various git repository states
- Configuration merging and override behavior

### 10.3 Performance Tests
- Benchmark each module using `criterion`
- Memory usage profiling
- Stress testing with timeouts
- Parallel execution performance

### 10.4 Testing Tools
- **Snapshot Testing**: Use `insta` for output consistency
- **Property Testing**: Use `proptest` for input validation
- **Mocking**: Use `mockall` for external dependencies
- **Coverage**: Maintain > 80% test coverage with `tarpaulin`

### 10.5 Module Testing Pattern (Starship-inspired)

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::test::ModuleRenderer;
    
    #[test]
    fn test_directory_module() {
        let actual = ModuleRenderer::new("directory")
            .path("/home/user/projects/beacon")
            .config(toml::toml! {
                [directory]
                truncation_length = 2
                truncate_to_repo = true
            })
            .collect();
        
        assert_eq!(actual, Some("~/projects/beacon ".to_string()));
    }
    
    #[test]
    fn test_git_branch_detection() {
        let tempdir = tempfile::tempdir()?;
        // Initialize git repo
        std::process::Command::new("git")
            .args(&["init"])
            .current_dir(tempdir.path())
            .output()?;
        
        let actual = ModuleRenderer::new("git_branch")
            .path(tempdir.path())
            .collect();
        
        assert_eq!(actual, Some("🌱 main ".to_string()));
    }
}
```

## 11. Dependencies

### 11.1 Core Dependencies (Phase 1 - MVP)
```toml
[dependencies]
# Essential only for Phase 1
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
toml = "0.8"
clap = { version = "4.5", features = ["derive"] }
anyhow = "1.0"  # Simple error handling for beginners
dirs = "5.0"
```

### 11.2 Phase 2+ Dependencies (Add incrementally)
```toml
# Add these when ready
git2 = "0.18"      # Phase 2: For git modules
once_cell = "1.20"  # Phase 3: For caching
thiserror = "1.0"   # Phase 4: For custom errors
rayon = "1.10"      # Phase 4: For parallel execution
```

### 11.3 Development Dependencies
```toml
[dependencies]
git2 = "0.18"     # For git modules
chrono = "0.4"    # For time module
whoami = "1.5"    # For username/hostname

[dev-dependencies]
criterion = "0.5"  # Benchmarking
insta = "1.40"     # Snapshot testing
proptest = "1.5"   # Property-based testing
mockall = "0.13"   # Mocking framework
```

### 11.4 Build Optimization (Phase 4+)
```toml
[profile.release]
lto = true           # Link-time optimization
codegen-units = 1    # Single codegen unit for better optimization
strip = true         # Strip symbols for smaller binary
opt-level = 3        # Maximum optimization
panic = "abort"      # Smaller binary, no unwinding
```

## 12. Success Criteria

- [ ] Executes in < 50ms for typical usage (100ms acceptable)
- [ ] Compatible with Claude Code statusLine
- [ ] Handles all Claude Code JSON fields correctly
- [ ] Configuration compatible with starship format (subset)
- [ ] Zero panics in production
- [ ] Comprehensive test coverage (> 80%)
- [ ] Clear documentation
- [ ] Easy installation process
- [ ] Graceful error handling (executable permissions, missing fields)

## 13. Future Considerations

### 13.1 Short-term Enhancements
- Configuration hot-reload during development
- Module execution metrics collection
- WASM-based plugin system for safe extensions
- Async module execution with `tokio` runtime

### 13.2 Long-term Features
- Web-based configuration UI
- Performance profiling dashboard
- Cross-platform installer
- Configuration migration tool from starship
- Template/theme marketplace
- Internationalization (i18n) support
- Claude Code session-aware features

---

## Appendix A: Module Configuration Reference

Each module will have detailed configuration documentation in `/docs/modules/`.

## Appendix B: Color Reference

Standard ANSI colors and their meanings in Beacon context.

## Appendix C: Comparison with Starship

### Key Differentiators
- **Claude Code Optimized**: Specifically designed for Claude Code's statusLine feature
- **Lighter Weight**: Minimal dependencies and focused feature set
- **Faster Execution**: Target < 50ms execution time through aggressive optimization
- **Session Aware**: Utilizes Claude Code's session information for enhanced context

### Migration Guide
- Configuration format is similar but not identical
- Subset of starship modules supported
- Custom migration tool planned for Phase 4

## Appendix D: Rust Patterns for Beginners

### Start with These Patterns (Phase 1)

#### 1. Simple Error Handling
```rust
// Start with this in Phase 1
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = read_config()?;  // ? operator
    Ok(())
}
```

#### 2. Builder Pattern (Phase 2+)
```rust
ConfigBuilder::new()
    .timeout(500)
    .format("...")
    .build()
```

#### 3. Early Return Pattern
```rust
fn detect() -> Option<String> {
    let repo = context.get_repo().ok()?;
    let branch = repo.branch()?;
    Some(branch)
}
```

#### 4. Iterator Chains (Phase 2+)
```rust
modules.iter()
    .filter(|m| m.should_display(ctx))
    .map(|m| m.render(ctx))
    .collect::<Result<Vec<_>, _>>()
```

#### 5. Type Aliases for Simplicity
```rust
type Result<T> = std::result::Result<T, Box<dyn Error>>;
```

### Advanced Patterns (Phase 3+)
```rust
// Static lazy initialization
static CACHE: Lazy<Mutex<HashMap<String, String>>> = 
    Lazy::new(|| Mutex::new(HashMap::new()));
```

## Appendix E: Phase 1 Quick Start Guide

### Minimum Viable Implementation

1. **Start here**: Create a CLI that reads JSON from stdin
2. **Parse config**: Load a simple TOML file
3. **Three modules only**: directory, character, claude_model
4. **Output**: Single ANSI-formatted line

### Example Phase 1 Main Function
```rust
use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
struct ClaudeInput {
    hook_event_name: String,
    session_id: String,
    transcript_path: String,
    cwd: String,
    model: ModelInfo,
    workspace: WorkspaceInfo,
    version: String,
    output_style: OutputStyle,
}

#[derive(Debug, Deserialize)]
struct ModelInfo {
    id: String,
    display_name: String,
}

#[derive(Debug, Deserialize)]
struct WorkspaceInfo {
    current_dir: String,
    project_dir: String,
}

#[derive(Debug, Deserialize)]
struct OutputStyle {
    name: String,
}

fn main() -> Result<()> {
    // 1. Read JSON from stdin with proper structure
    let input: ClaudeInput = serde_json::from_reader(std::io::stdin())?;
    
    // 2. Load config
    let config = load_config()?;
    
    // 3. Create context from structured input
    let context = Context::new(input);
    
    // 4. Generate prompt with 3 modules
    let output = generate_prompt(&context, &config)?;
    
    // 5. Print single line (first line only)
    print!("{}", output);
    
    Ok(())
}
```

### Skip these for now:
- Git operations (add in Phase 2)
- Parallel processing (Phase 4)
- Custom error types (Phase 4)
- Complex testing (Phase 3)
- Performance optimization (Phase 4)

**Focus on getting something working first!**
