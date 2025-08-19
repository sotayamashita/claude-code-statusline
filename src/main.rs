use clap::Parser;
use std::io::{self, Read};

// Import modules
mod types;
mod parser;

use parser::parse_claude_input;

#[derive(Parser)]
#[command(name = env!("CARGO_PKG_NAME"))]
#[command(version = env!("CARGO_PKG_VERSION"))]
#[command(about = env!("CARGO_PKG_DESCRIPTION"))]
struct Cli {
    // 後でサブコマンドを追加予定
}

fn main() {
    let _cli = Cli::parse();
    
    // Read JSON from stdin
    let mut buffer = String::new();
    match io::stdin().read_to_string(&mut buffer) {
        Ok(_) => {
            // Check if buffer is empty (no piped input)
            if buffer.trim().is_empty() {
                // No JSON input, display default prompt
                println!("No json input detected");
                return;
            }
            
            // Parse JSON into ClaudeInput struct
            match parse_claude_input(&buffer) {
                Ok(input) => {
                    // Successfully parsed, print debug info for now
                    println!("Received Claude Code input:");
                    println!("  Model: {}", input.model.display_name);
                    println!("  CWD: {}", input.cwd);
                    println!("  Session: {}", input.session_id);
                }
                Err(e) => {
                    eprintln!("Failed to parse JSON: {}", e);
                    eprintln!("Raw input: {}", buffer);
                }
            }
        }
        Err(e) => {
            eprintln!("Failed to read from stdin: {}", e);
        }
    }
}
