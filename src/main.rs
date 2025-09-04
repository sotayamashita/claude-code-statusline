use anyhow::Result;
use clap::Parser;
use std::collections::HashMap;
use std::io::{self, Read};

// Import modules
mod config;
mod debug;
mod messages;
mod modules;
mod parser;
mod style;
mod timeout;
mod types;

use config::Config;
use debug::DebugLogger;
use modules::render_module_with_timeout;
use parser::{extract_modules_from_format, parse_claude_input, parse_format};
use types::context::Context;

/// Generate the status line prompt from Context
fn generate_prompt(context: &Context, logger: &DebugLogger) -> String {
    // Get format string from config (default: "$directory $claude_model")
    let format = &context.config.format;

    // Extract module names from format string
    let module_names = extract_modules_from_format(format);

    // Collect module outputs
    let mut module_outputs = HashMap::new();

    for name in &module_names {
        // Character module not implemented yet
        if name == "character" {
            continue;
        }
        if let Some(out) = render_module_with_timeout(name, context, logger) {
            module_outputs.insert(name.clone(), out);
        }
    }

    // Use format parser to generate final output
    parse_format(format, context, &module_outputs)
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

    // Load configuration with graceful error handling
    let config = match Config::load() {
        Ok(cfg) => cfg,
        Err(e) => {
            // Print detailed error to stderr, concise message to stdout
            eprintln!("Config error: {e}");
            print!("{}", messages::MSG_FAILED_INVALID_CONFIG);
            io::Write::flush(&mut io::stdout())?;
            return Ok(());
        }
    };

    // Initialize debug logger
    let logger = DebugLogger::new(config.debug);
    logger.log_execution_start();
    logger.log_config(config.debug, config.command_timeout);

    // Config validation and non-fatal warnings
    if let Err(e) = config.validate() {
        eprintln!("Config validation error: {e}");
        print!("Failed to build status line due to invalid config");
        io::Write::flush(&mut io::stdout())?;
        return Ok(());
    }
    for w in config.collect_warnings() {
        // Use stderr for warnings (visible when debug is enabled for extra detail)
        logger.log_stderr(&format!("WARN: {w}"));
    }

    // Read JSON from stdin
    let mut buffer = String::new();
    io::stdin().read_to_string(&mut buffer)?;
    logger.log_input(&buffer);

    // Check if buffer is empty (no piped input)
    if buffer.trim().is_empty() {
        logger.log_stderr("Empty input received");
        // No JSON input, display default status line without newline
        print!("{}", messages::MSG_FAILED_EMPTY_INPUT);
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
            let prompt = generate_prompt(&context, &logger);
            logger.log_prompt(&prompt);

            print!("{prompt}"); // No newline for status line
            io::Write::flush(&mut io::stdout())?;
        }
        Err(e) => {
            logger.log_error(&e.to_string());

            // On error, output a fallback status line (not error message)
            // Error details go to stderr for debugging
            eprintln!("Failed to parse JSON: {e}");
            print!("{}", messages::MSG_FAILED_INVALID_JSON); // Fallback status line
            io::Write::flush(&mut io::stdout())?;
        }
    }

    Ok(())
}
