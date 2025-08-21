use anyhow::Result;
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
use modules::handle_module;
use parser::parse_claude_input;
use types::context::Context;

/// Generate the status line prompt from Context
fn generate_prompt(context: &Context) -> String {
    let mut segments = Vec::new();

    // Use the central dispatcher to create modules
    // This allows for dynamic module loading based on configuration
    let module_names = vec!["directory", "claude_model"];

    for name in module_names {
        if let Some(module) = handle_module(name, context) {
            if module.should_display() {
                segments.push(module.render());
            }
        }
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

fn main() -> Result<()> {
    let _cli = Cli::parse();

    // Load configuration
    let config = Config::load()?;

    // Initialize debug logger
    let logger = DebugLogger::new(config.debug);
    logger.log_execution_start();
    logger.log_config(config.debug, config.command_timeout);

    // Read JSON from stdin
    let mut buffer = String::new();
    io::stdin().read_to_string(&mut buffer)?;
    logger.log_input(&buffer);

    // Check if buffer is empty (no piped input)
    if buffer.trim().is_empty() {
        logger.log_stderr("Empty input received");
        // No JSON input, display default status line without newline
        print!("Failed to build status line due to empty input");
        io::Write::flush(&mut io::stdout())?;
        return Ok(());
    }

    // Parse JSON into ClaudeInput struct
    match parse_claude_input(&buffer) {
        Ok(input) => {
            logger.log_success(&input.model.display_name, &input.cwd);

            // Create context from input and config
            let context = Context::new(input, config);

            // Generate and output status line
            let prompt = generate_prompt(&context);
            logger.log_prompt(&prompt);

            print!("{prompt}"); // No newline for status line
            io::Write::flush(&mut io::stdout())?;
        }
        Err(e) => {
            logger.log_error(&e.to_string());

            // On error, output a fallback status line (not error message)
            // Error details go to stderr for debugging
            eprintln!("Failed to parse JSON: {e}");
            print!("Failed to build status line due to invalid json"); // Fallback status line
            io::Write::flush(&mut io::stdout())?;
        }
    }

    Ok(())
}
