use clap::Parser;
use std::io::{self, Read};

// Import modules
mod config;
mod debug;
mod modules;
mod parser;
mod types;

use config::Config;
use debug::DebugLogger;
use modules::{ClaudeModelModule, DirectoryModule, Module};
use parser::parse_claude_input;
use types::claude::ClaudeInput;

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

    // セグメントを結合（スペースで区切る）
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

    // Load configuration
    let config = Config::load();

    // Initialize debug logger
    let logger = DebugLogger::new(config.debug);
    logger.log_execution_start();
    logger.log_config(config.debug, config.command_timeout);

    // Read JSON from stdin
    let mut buffer = String::new();
    match io::stdin().read_to_string(&mut buffer) {
        Ok(_) => {
            logger.log_input(&buffer);

            // Check if buffer is empty (no piped input)
            if buffer.trim().is_empty() {
                logger.log_stderr("Empty input received");
                // No JSON input, display default status line without newline
                print!("Failed to build status line due to empty input");
                io::Write::flush(&mut io::stdout()).unwrap();
                return;
            }

            // Parse JSON into ClaudeInput struct
            match parse_claude_input(&buffer) {
                Ok(input) => {
                    logger.log_success(&input.model.display_name, &input.cwd);

                    // Generate and output status line
                    let prompt = generate_prompt(&input);
                    logger.log_prompt(&prompt);

                    print!("{}", prompt); // No newline for status line
                    io::Write::flush(&mut io::stdout()).unwrap();
                }
                Err(e) => {
                    logger.log_error(&e.to_string());

                    // On error, output a fallback status line (not error message)
                    // Error details go to stderr for debugging
                    eprintln!("Failed to parse JSON: {}", e);
                    print!("Failed to build status line due to invalid json"); // Fallback status line
                    io::Write::flush(&mut io::stdout()).unwrap();
                }
            }
        }
        Err(e) => {
            eprintln!("Failed to read from stdin: {}", e);
            print!("Failed to build status line due to unexpected error"); // Fallback status line
            io::Write::flush(&mut io::stdout()).unwrap();
        }
    }
}
