//! Debug logging utilities
//!
//! This module provides debug logging functionality for development
//! and troubleshooting. Debug output can be enabled via configuration
//! or environment variable.

use std::fs::OpenOptions;
use std::io::Write;
use std::path::{Path, PathBuf};

/// Debug logger for development and troubleshooting
///
/// Writes debug information to a temporary log file when enabled.
/// Can be activated through configuration (`debug = true`) or
/// environment variable (`CCS_DEBUG=1`).
///
/// # Log Location
///
/// - Unix/Linux: `/tmp/claude-code-statusline.log`
/// - Windows: `%TEMP%\claude-code-statusline.log`
/// - macOS: `/var/folders/.../claude-code-statusline.log`
pub struct DebugLogger {
    enabled: bool,
    log_file: PathBuf,
}

impl DebugLogger {
    /// Creates a new DebugLogger instance
    ///
    /// # Arguments
    ///
    /// * `enabled` - Whether debug logging is enabled in configuration
    ///
    /// # Environment Variables
    ///
    /// - `CCS_DEBUG=1` - Forces debug logging regardless of config
    ///
    /// # Examples
    ///
    /// ```
    /// use claude_code_statusline_core::debug::DebugLogger;
    ///
    /// let logger = DebugLogger::new(true);
    /// logger.log("Debug message");
    /// ```
    pub fn new(enabled: bool) -> Self {
        // Check environment variable as well
        let enabled = enabled || std::env::var("CCS_DEBUG").unwrap_or_default() == "1";

        // Use cross-platform temp directory
        let log_file = std::env::temp_dir().join("claude-code-statusline.log");

        Self { enabled, log_file }
    }

    /// Log a message if debug mode is enabled
    pub fn log(&self, message: &str) {
        if !self.enabled {
            // Still forward to tracing at debug level for centralized logging
            tracing::debug!(target: "claude-code-statusline", "{message}");
            return;
        }

        // Create parent directory if it doesn't exist
        if let Some(parent) = Path::new(&self.log_file).parent() {
            std::fs::create_dir_all(parent).ok();
        }

        if let Ok(mut file) = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.log_file)
        {
            writeln!(file, "{message}").ok();
        }
    }

    /// Log to stderr if debug mode is enabled
    pub fn log_stderr(&self, message: &str) {
        // Emit via tracing; subscriber decides output
        tracing::debug!(target: "claude-code-statusline", "{message}");
        if self.enabled {
            eprintln!("[DEBUG] {message}");
        }
    }

    /// Log a new execution marker
    pub fn log_execution_start(&self) {
        self.log("--- New execution ---");
    }

    /// Log configuration information
    pub fn log_config(&self, debug: bool, command_timeout: u64) {
        self.log(&format!(
            "Config loaded: debug={debug}, command_timeout={command_timeout}"
        ));
    }

    /// Log input information
    pub fn log_input(&self, buffer: &str) {
        self.log(&format!("Input length: {} bytes", buffer.len()));
        if !buffer.is_empty() {
            self.log(&format!(
                "First 500 chars: {}",
                &buffer[..buffer.len().min(500)]
            ));
        }
    }

    /// Log successful parse
    pub fn log_success(&self, model: &str, cwd: &str) {
        self.log(&format!("SUCCESS: Model={model}, CWD={cwd}"));
    }

    /// Log generated prompt
    pub fn log_prompt(&self, prompt: &str) {
        self.log(&format!("Generated: {prompt}"));
    }

    /// Log error
    pub fn log_error(&self, error: &str) {
        tracing::error!(target: "claude-code-statusline", "{error}");
        self.log(&format!("ERROR: {error}"));
    }

    /// Check if debug mode is enabled
    #[allow(dead_code)]
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }
}
