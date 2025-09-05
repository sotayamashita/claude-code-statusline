use anyhow::Result;
use clap::Parser;
use std::io::{self, Read};

/// Command line interface arguments structure (placeholder for future subcommands)
#[derive(Parser)]
#[command(name = env!("CARGO_PKG_NAME"))]
#[command(version = env!("CARGO_PKG_VERSION"))]
#[command(about = env!("CARGO_PKG_DESCRIPTION"))]
struct Cli {}

/// Run the Beacon CLI: read stdin JSON, render status line, write stdout.
pub fn run() -> Result<()> {
    let _cli = Cli::parse();

    // Load configuration with graceful error handling
    let config = match beacon_core::Config::load() {
        Ok(cfg) => cfg,
        Err(e) => {
            eprintln!("Config error: {e}");
            let msg = beacon_core::messages::MSG_FAILED_INVALID_CONFIG;
            print!("{msg}");
            io::Write::flush(&mut io::stdout())?;
            return Ok(());
        }
    };

    // Initialize debug logger
    let logger = beacon_core::debug::DebugLogger::new(config.debug);
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
        logger.log_stderr(&format!("WARN: {w}"));
    }

    // Read JSON input from stdin
    let mut buffer = String::new();
    if io::stdin().read_to_string(&mut buffer).is_err() || buffer.trim().is_empty() {
        let msg = beacon_core::messages::MSG_FAILED_EMPTY_INPUT;
        print!("{msg}");
        io::Write::flush(&mut io::stdout())?;
        return Ok(());
    }
    logger.log_input(&buffer);

    // Parse JSON input
    let input = match beacon_core::parse_claude_input(&buffer) {
        Ok(i) => i,
        Err(e) => {
            eprintln!("Failed to parse JSON: {e}");
            let msg = beacon_core::messages::MSG_FAILED_INVALID_JSON;
            print!("{msg}");
            io::Write::flush(&mut io::stdout())?;
            return Ok(());
        }
    };
    logger.log_success(&input.model.display_name, &input.cwd);

    // Render via engine
    let engine = beacon_core::Engine::new(config);
    match engine.render(&input) {
        Ok(out) => {
            print!("{out}");
            io::Write::flush(&mut io::stdout())?;
        }
        Err(e) => {
            eprintln!("Render error: {e}");
        }
    }

    Ok(())
}
