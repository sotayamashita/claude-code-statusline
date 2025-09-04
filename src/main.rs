//! Beacon - A fast, modular status line for AI development
//!
//! This is the main entry point for the Beacon CLI application.
//! It reads JSON input from stdin, parses it, and generates a formatted
//! status line based on the configuration and available modules.
//!
//! # Architecture
//!
//! The application follows a modular design where each status component
//! (directory, git branch, model info, etc.) is implemented as a separate
//! module that can be enabled/disabled via configuration.
//!
//! # Input Format
//!
//! Expects JSON input via stdin with the following structure:
//! ```json
//! {
//!     "cwd": "/current/working/directory",
//!     "model": {
//!         "id": "claude-3.5-sonnet",
//!         "display_name": "Sonnet"
//!     }
//! }
//! ```

use anyhow::Result;
use clap::Parser;
// std collections used implicitly by modules and parser
use std::io::{self, Read};

// Import modules
mod config;
mod debug;
mod engine;
mod messages;
mod modules;
mod parser;
mod style;
mod timeout;
mod types;

use config::Config;
use debug::DebugLogger;
use engine::Engine;
use parser::parse_claude_input;
use types::context::Context;

/// Command line interface arguments structure
///
/// Currently a placeholder for future subcommands and CLI options.
/// Uses clap's derive macros to automatically generate CLI parsing.
#[derive(Parser)]
#[command(name = env!("CARGO_PKG_NAME"))]
#[command(version = env!("CARGO_PKG_VERSION"))]
#[command(about = env!("CARGO_PKG_DESCRIPTION"))]
struct Cli {
    // Future subcommands will be added here
}

/// Main entry point for the Beacon application
///
/// # Workflow
///
/// 1. Parse command line arguments (reserved for future use)
/// 2. Load configuration from `~/.config/beacon.toml`
/// 3. Initialize debug logger based on configuration
/// 4. Read JSON input from stdin
/// 5. Parse and validate the JSON input
/// 6. Generate formatted status line based on configuration
/// 7. Output the status line to stdout
///
/// # Errors
///
/// Returns `Ok(())` even on failures to ensure graceful degradation.
/// Error details are written to stderr while a fallback status line
/// is displayed on stdout.
///
/// # Examples
///
/// ```bash
/// echo '{"cwd":"/tmp","model":{"id":"claude","display_name":"Claude"}}' | beacon
/// ```
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
            let context = Context::new(input, config.clone());

            // Generate and output status line
            let engine = Engine::new(config.clone());
            let prompt = engine
                .render(&context.input)
                .unwrap_or_else(|_| String::new());
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
