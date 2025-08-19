use clap::Parser;
use std::io::{self, Read, Write};

// Import modules
mod types;
mod parser;
mod modules;

use parser::parse_claude_input;
use modules::{Module, DirectoryModule, CharacterModule, ClaudeModelModule};
use types::ClaudeInput;

/// Generate the status line prompt from ClaudeInput
fn generate_prompt(input: &ClaudeInput) -> String {
    let mut segments = Vec::new();
    
    // Directory module
    let dir_module = DirectoryModule::new(&input.cwd);
    if dir_module.should_display() {
        segments.push(dir_module.render());
    }
    
    // Claude model module
    let model_module = ClaudeModelModule::new(&input.model.display_name);
    if model_module.should_display() {
        segments.push(model_module.render());
    }
    
    // Character module (prompt symbol)
    let char_module = CharacterModule::new();
    if char_module.should_display() {
        segments.push(char_module.render());
    }
    
    // Join segments with spaces
    segments.join(" ")
}

#[derive(Parser)]
#[command(name = env!("CARGO_PKG_NAME"))]
#[command(version = env!("CARGO_PKG_VERSION"))]
#[command(about = env!("CARGO_PKG_DESCRIPTION"))]
struct Cli {
    // 後でサブコマンドを追加予定
}

fn main() {
    let _cli = Cli::parse();
    
    // Debug: write to file in project tmp directory
    let debug_file = "./tmp/beacon-debug.log";
    use std::fs::OpenOptions;
    use std::io::Write as IoWrite;
    
    // Read JSON from stdin
    let mut buffer = String::new();
    match io::stdin().read_to_string(&mut buffer) {
        Ok(_) => {
            // Debug: log to file
            if let Ok(mut file) = OpenOptions::new()
                .create(true)
                .append(true)
                .open(debug_file) 
            {
                writeln!(file, "--- New execution ---").ok();
                writeln!(file, "Input length: {} bytes", buffer.len()).ok();
                if !buffer.is_empty() {
                    writeln!(file, "First 500 chars: {}", &buffer[..buffer.len().min(500)]).ok();
                }
            }
            
            // Check if buffer is empty (no piped input)
            if buffer.trim().is_empty() {
                // Debug: log to stderr
                eprintln!("[DEBUG] Empty input received");
                // No JSON input, display default prompt without newline
                print!("~/projects ❯ ");
                io::Write::flush(&mut io::stdout()).unwrap();
                return;
            }
            
            // Parse JSON into ClaudeInput struct
            match parse_claude_input(&buffer) {
                Ok(input) => {
                    // Debug: log successful parse to file
                    if let Ok(mut file) = OpenOptions::new()
                        .create(true)
                        .append(true)
                        .open(debug_file) 
                    {
                        writeln!(file, "SUCCESS: Model={}, CWD={}", 
                                 input.model.display_name, input.cwd).ok();
                    }
                    
                    // Generate and output status line
                    let prompt = generate_prompt(&input);
                    
                    // Debug: log generated prompt to file
                    if let Ok(mut file) = OpenOptions::new()
                        .create(true)
                        .append(true)
                        .open(debug_file) 
                    {
                        writeln!(file, "Generated: {}", prompt).ok();
                    }
                    
                    print!("{}", prompt); // No newline for status line
                    io::Write::flush(&mut io::stdout()).unwrap();
                }
                Err(e) => {
                    // Debug: log error to file
                    if let Ok(mut file) = OpenOptions::new()
                        .create(true)
                        .append(true)
                        .open(debug_file) 
                    {
                        writeln!(file, "ERROR: {}", e).ok();
                    }
                    
                    // On error, output a fallback status line (not error message)
                    // Error details go to stderr for debugging
                    eprintln!("Failed to parse JSON: {}", e);
                    print!("~/projects ❯ "); // Fallback status line
                    io::Write::flush(&mut io::stdout()).unwrap();
                }
            }
        }
        Err(e) => {
            eprintln!("Failed to read from stdin: {}", e);
            print!("~/projects ❯ "); // Fallback status line
            io::Write::flush(&mut io::stdout()).unwrap();
        }
    }
}
