use anyhow::Result;
use clap::{Parser, Subcommand};
use std::io::{self, Read};

/// Command line interface arguments structure (placeholder for future subcommands)
#[derive(Parser)]
#[command(name = env!("CARGO_PKG_NAME"))]
#[command(version = env!("CARGO_PKG_VERSION"))]
#[command(about = env!("CARGO_PKG_DESCRIPTION"))]
struct Cli {
    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Subcommand)]
enum Command {
    /// Inspect or validate configuration
    Config {
        /// Print config file path
        #[arg(long)]
        path: bool,
        /// Print default config (TOML)
        #[arg(long)]
        default: bool,
        /// Validate current config
        #[arg(long)]
        validate: bool,
    },
    /// Inspect module registry
    Modules {
        /// List all available modules
        #[arg(long)]
        list: bool,
        /// List enabled modules for current config
        #[arg(long)]
        enabled: bool,
    },
}

/// Run the claude-code-statusline CLI: read stdin JSON, render status line, write stdout.
pub fn run() -> Result<()> {
    let _cli = Cli::parse();
    if let Some(cmd) = &_cli.command {
        // Minimal subscriber for subcommands
        let _ = tracing_subscriber::fmt()
            .with_env_filter("error")
            .with_writer(std::io::stderr)
            .try_init();

        match cmd {
            Command::Config {
                path,
                default,
                validate,
            } => {
                if *path {
                    let path = claude_code_statusline_core::config_path();
                    println!("{}", path.display());
                    return Ok(());
                }
                if *default {
                    let toml =
                        toml::to_string_pretty(&claude_code_statusline_core::Config::default())
                            .unwrap_or_else(|_| "".into());
                    println!("{toml}");
                    return Ok(());
                }
                if *validate {
                    match claude_code_statusline_core::Config::load() {
                        Ok(cfg) => match cfg.validate() {
                            Ok(()) => {
                                println!("OK");
                            }
                            Err(e) => {
                                eprintln!("Config validation error: {e}");
                                println!("INVALID");
                            }
                        },
                        Err(e) => {
                            eprintln!("Config error: {e}");
                            println!("INVALID");
                        }
                    }
                    return Ok(());
                }
                // If no flags, show help
                println!("Use --path | --default | --validate");
                return Ok(());
            }
            Command::Modules { list, enabled } => {
                if *list {
                    let reg = claude_code_statusline_core::modules::Registry::with_defaults();
                    for name in reg.list() {
                        println!("{name}");
                    }
                    return Ok(());
                }
                if *enabled {
                    let cfg = claude_code_statusline_core::Config::load().unwrap_or_default();
                    let names = claude_code_statusline_core::parser::extract_modules_from_format(
                        &cfg.format,
                    );
                    let reg = claude_code_statusline_core::modules::Registry::with_defaults();
                    for name in names {
                        if name == "character" {
                            continue;
                        }
                        // Only consider registered modules
                        if !reg.list().contains(&name.as_str()) {
                            continue;
                        }
                        let is_enabled = match name.as_str() {
                            "directory" => !cfg.directory.disabled,
                            "claude_model" => !cfg.claude_model.disabled,
                            "git_branch" => !cfg.git_branch.disabled,
                            "git_status" => !cfg.git_status.disabled,
                            _ => true,
                        };
                        if is_enabled {
                            println!("{name}");
                        }
                    }
                    return Ok(());
                }
                println!("Use --list | --enabled");
                return Ok(());
            }
        }
    }

    // Load configuration with graceful error handling
    let config = match claude_code_statusline_core::Config::load() {
        Ok(cfg) => cfg,
        Err(e) => {
            // Initialize minimal subscriber to show errors (stderr)
            let _ = tracing_subscriber::fmt()
                .with_env_filter("error")
                .with_writer(std::io::stderr)
                .try_init();
            tracing::error!(error = %e, "Config error");
            eprintln!("Config error: {e}");
            let msg = claude_code_statusline_core::messages::MSG_FAILED_INVALID_CONFIG;
            print!("{msg}");
            io::Write::flush(&mut io::stdout())?;
            return Ok(());
        }
    };

    // Initialize tracing subscriber based on config.debug
    {
        let level = if config.debug { "debug" } else { "error" };
        let _ = tracing_subscriber::fmt()
            .with_env_filter(level)
            .with_target(false)
            .with_writer(std::io::stderr)
            .try_init();
    }

    // Initialize debug logger
    let logger = claude_code_statusline_core::debug::DebugLogger::new(config.debug);
    logger.log_execution_start();
    logger.log_config(config.debug, config.command_timeout);

    // Config validation and non-fatal warnings
    if let Err(e) = config.validate() {
        tracing::error!(error = %e, "Config validation error");
        eprintln!("Config validation error: {e}");
        print!("Failed to build status line due to invalid config");
        io::Write::flush(&mut io::stdout())?;
        return Ok(());
    }
    for w in config.collect_warnings() {
        tracing::warn!("{w}");
    }

    // Read JSON input from stdin
    let mut buffer = String::new();
    if io::stdin().read_to_string(&mut buffer).is_err() || buffer.trim().is_empty() {
        let msg = claude_code_statusline_core::messages::MSG_FAILED_EMPTY_INPUT;
        print!("{msg}");
        io::Write::flush(&mut io::stdout())?;
        return Ok(());
    }
    logger.log_input(&buffer);

    // Parse JSON input
    let input = match claude_code_statusline_core::parse_claude_input(&buffer) {
        Ok(i) => i,
        Err(e) => {
            tracing::error!(error = %e, "Failed to parse JSON");
            eprintln!("Failed to parse JSON: {e}");
            let msg = claude_code_statusline_core::messages::MSG_FAILED_INVALID_JSON;
            print!("{msg}");
            io::Write::flush(&mut io::stdout())?;
            return Ok(());
        }
    };
    logger.log_success(&input.model.display_name, &input.cwd);

    // Render via engine
    let engine = claude_code_statusline_core::Engine::new(config);
    match engine.render(&input) {
        Ok(out) => {
            print!("{out}");
            io::Write::flush(&mut io::stdout())?;
        }
        Err(e) => {
            tracing::error!(error = %e, "Render error");
            eprintln!("Render error: {e}");
        }
    }

    Ok(())
}
