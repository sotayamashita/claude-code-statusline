# Development Guide

## Claude Code Status Line Development with Rust

This guide explains how to build and test a Rust-based status line for Claude Code.

### Quick Start

1. **Build the Rust binary**
   ```bash
   cargo build --release
   ```

2. **Configure Claude Code**
   ```json
   // ~/.claude/settings.json
   {
     "statusLine": {
       "type": "command",
       "command": "/path/to/beacon/target/release/beacon"
     }
   }
   ```

3. **Test the status line**
   ```bash
   # Test with mock JSON input
   echo '{"model":{"display_name":"Opus"},"workspace":{"current_dir":"/home/user/project"}}' | ./target/release/beacon
   ```

### Development Setup

#### Project Structure
```
beacon/
├── Cargo.toml
├── src/
│   └── main.rs
├── tests/
│   └── integration_test.rs
└── docs/
    └── development.md
```

#### Dependencies
```toml
# Cargo.toml
[package]
name = "beacon"
version = "0.1.0"
edition = "2021"

[dependencies]
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
```

### Implementation Example

```rust
// src/main.rs
use serde::{Deserialize, Serialize};
use std::io::{self, Read};

#[derive(Debug, Deserialize)]
struct StatusInput {
    model: Model,
    workspace: Workspace,
    #[serde(default)]
    session_id: String,
    #[serde(default)]
    version: String,
}

#[derive(Debug, Deserialize)]
struct Model {
    id: String,
    display_name: String,
}

#[derive(Debug, Deserialize)]
struct Workspace {
    current_dir: String,
    project_dir: String,
}

fn main() -> io::Result<()> {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input)?;
    
    let data: StatusInput = serde_json::from_str(&input)
        .unwrap_or_else(|_| {
            // Fallback for invalid JSON
            eprintln!("Failed to parse JSON input");
            std::process::exit(1);
        });
    
    let dir_name = data.workspace.current_dir
        .split('/')
        .last()
        .unwrap_or(".");
    
    println!("[{}] 📁 {}", data.model.display_name, dir_name);
    
    Ok(())
}
```

### Testing

#### Unit Tests
```rust
// src/main.rs or separate test module
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_parse_status_input() {
        let json = r#"{
            "model": {"id": "claude-opus", "display_name": "Opus"},
            "workspace": {
                "current_dir": "/home/user/project",
                "project_dir": "/home/user/project"
            }
        }"#;
        
        let input: StatusInput = serde_json::from_str(json).unwrap();
        assert_eq!(input.model.display_name, "Opus");
        assert_eq!(input.workspace.current_dir, "/home/user/project");
    }
}
```

#### Integration Tests
```rust
// tests/integration_test.rs
use std::process::{Command, Stdio};
use std::io::Write;

#[test]
fn test_status_line_output() {
    let json = r#"{
        "model": {"id": "claude-opus", "display_name": "Opus"},
        "workspace": {
            "current_dir": "/home/user/project",
            "project_dir": "/home/user"
        }
    }"#;
    
    let mut child = Command::new("./target/debug/beacon")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("Failed to spawn process");
    
    let stdin = child.stdin.as_mut().expect("Failed to open stdin");
    stdin.write_all(json.as_bytes()).expect("Failed to write to stdin");
    
    let output = child.wait_with_output().expect("Failed to read output");
    let stdout = String::from_utf8_lossy(&output.stdout);
    
    assert!(stdout.contains("[Opus]"));
    assert!(stdout.contains("project"));
}
```

### Testing in Claude Code

1. **Build and install**
   ```bash
   cargo build --release
   ```

2. **Start Claude Code**
   ```bash
   claude
   ```

3. **Verify the status line**
   - Check the bottom of the Claude Code interface
   - The status line should update with each message

4. **Monitor for errors**
   ```bash
   # Check Claude Code logs if status line doesn't appear
   tail -f ~/.claude/logs/claude.log
   ```

### Performance Optimization

```rust
// Optimized version with error handling
use std::time::Instant;

fn main() -> io::Result<()> {
    let start = Instant::now();
    
    let mut input = String::new();
    io::stdin().read_to_string(&mut input)?;
    
    // Parse with timeout consideration
    match serde_json::from_str::<StatusInput>(&input) {
        Ok(data) => {
            let output = format_status_line(&data);
            println!("{}", output);
        }
        Err(_) => {
            // Fallback output
            println!("[Unknown] 📁 .");
        }
    }
    
    // Log performance in debug builds
    #[cfg(debug_assertions)]
    eprintln!("Execution time: {:?}", start.elapsed());
    
    Ok(())
}

fn format_status_line(data: &StatusInput) -> String {
    let dir_name = data.workspace.current_dir
        .rsplit('/')
        .next()
        .unwrap_or(".");
    
    format!("[{}] 📁 {}", data.model.display_name, dir_name)
}
```

### Advanced Features

#### Git Integration
```rust
use std::process::Command;

fn get_git_branch() -> Option<String> {
    let output = Command::new("git")
        .args(&["branch", "--show-current"])
        .output()
        .ok()?;
    
    if output.status.success() {
        Some(String::from_utf8_lossy(&output.stdout).trim().to_string())
    } else {
        None
    }
}

fn format_status_with_git(data: &StatusInput) -> String {
    let dir_name = data.workspace.current_dir
        .rsplit('/')
        .next()
        .unwrap_or(".");
    
    match get_git_branch() {
        Some(branch) => format!("[{}] 📁 {} | 🌿 {}", 
            data.model.display_name, dir_name, branch),
        None => format!("[{}] 📁 {}", 
            data.model.display_name, dir_name),
    }
}
```

#### ANSI Colors
```rust
const GREEN: &str = "\x1b[32m";
const BLUE: &str = "\x1b[34m";
const RESET: &str = "\x1b[0m";

fn format_status_with_colors(data: &StatusInput) -> String {
    let dir_name = data.workspace.current_dir
        .rsplit('/')
        .next()
        .unwrap_or(".");
    
    format!("{GREEN}[{}]{RESET} {BLUE}📁 {}{RESET}", 
        data.model.display_name, dir_name)
}
```

### Debugging

1. **Enable debug output**
   ```rust
   #[cfg(debug_assertions)]
   {
       eprintln!("Input: {:?}", input);
       eprintln!("Parsed: {:?}", data);
   }
   ```

2. **Test with various inputs**
   ```bash
   # Test with minimal input
   echo '{"model":{"display_name":"Test"},"workspace":{"current_dir":"/"}}' | cargo run
   
   # Test with full input
   cat test_data/full_input.json | cargo run --release
   ```

3. **Benchmark performance**
   ```bash
   # Measure execution time
   time echo '{"model":{"display_name":"Test"},"workspace":{"current_dir":"/"}}' | ./target/release/beacon
   
   # Should complete in < 50ms
   ```

### Key Points

- Status line receives JSON via stdin from Claude Code
- First line of stdout becomes the status line
- Updates occur with conversation changes (max every 300ms)
- Target execution time: < 50ms
- Use `cargo build --release` for production
- Handle invalid JSON gracefully with fallback output
