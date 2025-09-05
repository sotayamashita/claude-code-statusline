//! Configuration type definitions
//!
//! This module defines the configuration structures used for parsing
//! TOML configuration files. Each module has its own configuration
//! section with sensible defaults.
//!
//! # Configuration File
//!
//! The configuration file is located at `~/.config/beacon.toml` and
//! uses TOML format. All fields are optional and will use defaults
//! if not specified.

use crate::error::CoreError;
use crate::modules::ModuleConfig;
use serde::{Deserialize, Serialize};
use std::any::Any;

/// Main configuration structure
///
/// Root configuration that contains global settings and module-specific
/// configurations. All fields have sensible defaults.
///
/// # Example
///
/// ```toml
/// format = "$directory $git_branch $claude_model"
/// command_timeout = 300
/// debug = true
///
/// [directory]
/// style = "bold blue"
/// truncation_length = 5
/// ```
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Config {
    #[serde(default = "default_format")]
    pub format: String,

    #[serde(default = "default_command_timeout")]
    pub command_timeout: u64,

    #[serde(default = "default_debug")]
    pub debug: bool,

    #[serde(default)]
    pub directory: DirectoryConfig,

    #[serde(default)]
    pub claude_model: ClaudeModelConfig,

    #[serde(default)]
    pub git_branch: GitBranchConfig,

    #[serde(default)]
    pub git_status: GitStatusConfig,

    /// Unrecognized/extra top-level tables (e.g., third-party modules)
    /// Captures unknown sections like `[my_custom_module]` without losing them.
    #[serde(flatten)]
    #[serde(default)]
    pub extra_modules: toml::value::Table,
}

/// Configuration for the directory module
///
/// Controls how the current directory is displayed in the status line.
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct DirectoryConfig {
    #[serde(default = "default_directory_format")]
    pub format: String,

    #[serde(default = "default_directory_style")]
    pub style: String,

    #[serde(default = "default_directory_truncation_length")]
    pub truncation_length: usize,

    #[serde(default = "default_directory_truncate_to_repo")]
    pub truncate_to_repo: bool,

    #[serde(default = "default_disabled")]
    pub disabled: bool,
}

/// Configuration for the Claude model module
///
/// Controls how the Claude model information is displayed.
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ClaudeModelConfig {
    #[serde(default = "default_claude_model_format")]
    pub format: String,

    #[serde(default = "default_claude_model_style")]
    pub style: String,

    #[serde(default = "default_claude_model_symbol")]
    pub symbol: String,

    #[serde(default = "default_disabled")]
    pub disabled: bool,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            format: default_format(),
            command_timeout: default_command_timeout(),
            debug: default_debug(),
            directory: DirectoryConfig::default(),
            claude_model: ClaudeModelConfig::default(),
            git_branch: GitBranchConfig::default(),
            git_status: GitStatusConfig::default(),
            extra_modules: toml::value::Table::new(),
        }
    }
}

impl Default for DirectoryConfig {
    fn default() -> Self {
        DirectoryConfig {
            format: default_directory_format(),
            style: default_directory_style(),
            truncation_length: default_directory_truncation_length(),
            truncate_to_repo: default_directory_truncate_to_repo(),
            disabled: default_disabled(),
        }
    }
}

impl Default for ClaudeModelConfig {
    fn default() -> Self {
        ClaudeModelConfig {
            format: default_claude_model_format(),
            style: default_claude_model_style(),
            symbol: default_claude_model_symbol(),
            disabled: default_disabled(),
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct GitBranchConfig {
    #[serde(default = "default_git_branch_format")]
    pub format: String,

    #[serde(default = "default_git_branch_style")]
    pub style: String,

    #[serde(default = "default_git_branch_symbol")]
    pub symbol: String,

    #[serde(default = "default_disabled")]
    pub disabled: bool,
}

impl Default for GitBranchConfig {
    fn default() -> Self {
        GitBranchConfig {
            format: default_git_branch_format(),
            style: default_git_branch_style(),
            symbol: default_git_branch_symbol(),
            disabled: default_disabled(),
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct GitStatusSymbolsConfig {
    #[serde(default = "default_git_status_symbol_conflicted")]
    pub conflicted: String,
    #[serde(default = "default_git_status_symbol_stashed")]
    pub stashed: String,
    #[serde(default = "default_git_status_symbol_deleted")]
    pub deleted: String,
    #[serde(default = "default_git_status_symbol_renamed")]
    pub renamed: String,
    #[serde(default = "default_git_status_symbol_modified")]
    pub modified: String,
    #[serde(default = "default_git_status_symbol_typechanged")]
    pub typechanged: String,
    #[serde(default = "default_git_status_symbol_staged")]
    pub staged: String,
    #[serde(default = "default_git_status_symbol_untracked")]
    pub untracked: String,
    #[serde(default = "default_git_status_symbol_ahead")]
    pub ahead: String,
    #[serde(default = "default_git_status_symbol_behind")]
    pub behind: String,
    #[serde(default = "default_git_status_symbol_diverged")]
    pub diverged: String,
}

impl Default for GitStatusSymbolsConfig {
    fn default() -> Self {
        GitStatusSymbolsConfig {
            conflicted: default_git_status_symbol_conflicted(),
            stashed: default_git_status_symbol_stashed(),
            deleted: default_git_status_symbol_deleted(),
            renamed: default_git_status_symbol_renamed(),
            modified: default_git_status_symbol_modified(),
            typechanged: default_git_status_symbol_typechanged(),
            staged: default_git_status_symbol_staged(),
            untracked: default_git_status_symbol_untracked(),
            ahead: default_git_status_symbol_ahead(),
            behind: default_git_status_symbol_behind(),
            diverged: default_git_status_symbol_diverged(),
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct GitStatusConfig {
    #[serde(default = "default_git_status_format")]
    pub format: String,

    #[serde(default = "default_git_status_style")]
    pub style: String,

    #[serde(default)]
    pub symbols: GitStatusSymbolsConfig,

    #[serde(default = "default_disabled")]
    pub disabled: bool,
}

impl Default for GitStatusConfig {
    fn default() -> Self {
        GitStatusConfig {
            format: default_git_status_format(),
            style: default_git_status_style(),
            symbols: GitStatusSymbolsConfig::default(),
            disabled: default_disabled(),
        }
    }
}

// Default value functions
fn default_format() -> String {
    "$directory $claude_model".to_string()
}

fn default_command_timeout() -> u64 {
    500
}

fn default_debug() -> bool {
    false
}

fn default_disabled() -> bool {
    false
}

// Directory module defaults
fn default_directory_format() -> String {
    "[$path]($style)".to_string()
}

fn default_directory_style() -> String {
    "bold cyan".to_string()
}

fn default_directory_truncation_length() -> usize {
    3
}

fn default_directory_truncate_to_repo() -> bool {
    true
}

// Claude Model module defaults
fn default_claude_model_format() -> String {
    "[$symbol$model]($style)".to_string()
}

fn default_claude_model_style() -> String {
    "bold yellow".to_string()
}

fn default_claude_model_symbol() -> String {
    "<".to_string()
}

// Git Branch module defaults
fn default_git_branch_format() -> String {
    "[🌿 $branch]($style)".to_string()
}

fn default_git_branch_style() -> String {
    "bold green".to_string()
}

fn default_git_branch_symbol() -> String {
    "🌿".to_string()
}

// Git Status module defaults (Starship 準拠の最小形)
fn default_git_status_format() -> String {
    // ([[$all_status$ahead_behind]]($style) )
    "([[$all_status$ahead_behind]]($style) )".to_string()
}

fn default_git_status_style() -> String {
    "bold red".to_string()
}

fn default_git_status_symbol_conflicted() -> String {
    "=".to_string()
}
fn default_git_status_symbol_stashed() -> String {
    "$".to_string()
}
fn default_git_status_symbol_deleted() -> String {
    "✘".to_string()
}
fn default_git_status_symbol_renamed() -> String {
    "»".to_string()
}
fn default_git_status_symbol_modified() -> String {
    "!".to_string()
}
fn default_git_status_symbol_typechanged() -> String {
    "".to_string()
}
fn default_git_status_symbol_staged() -> String {
    "+".to_string()
}
fn default_git_status_symbol_untracked() -> String {
    "?".to_string()
}
fn default_git_status_symbol_ahead() -> String {
    "⇡".to_string()
}
fn default_git_status_symbol_behind() -> String {
    "⇣".to_string()
}
fn default_git_status_symbol_diverged() -> String {
    "⇕".to_string()
}

// ModuleConfig implementations
impl ModuleConfig for DirectoryConfig {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn format(&self) -> &str {
        &self.format
    }

    fn style(&self) -> &str {
        &self.style
    }
}

impl ModuleConfig for ClaudeModelConfig {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn format(&self) -> &str {
        &self.format
    }

    fn style(&self) -> &str {
        &self.style
    }
}

impl ModuleConfig for GitBranchConfig {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn format(&self) -> &str {
        &self.format
    }

    fn style(&self) -> &str {
        &self.style
    }
}

impl ModuleConfig for GitStatusConfig {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn format(&self) -> &str {
        &self.format
    }

    fn style(&self) -> &str {
        &self.style
    }
}

impl Config {
    /// Validate configuration values. Returns an error for clearly invalid values.
    pub fn validate(&self) -> Result<(), CoreError> {
        // Milliseconds; enforce sane bounds (50ms ..= 600_000ms)
        if self.command_timeout < 50 || self.command_timeout > 600_000 {
            return Err(CoreError::InvalidConfig(format!(
                "command_timeout out of range (50..=600000): {}",
                self.command_timeout
            )));
        }
        Ok(())
    }

    /// Collect non-fatal warnings about style/format configuration.
    /// Unknown style tokens or unknown variables in format strings should not
    /// break the program, but we surface them as warnings.
    pub fn collect_warnings(&self) -> Vec<String> {
        let mut warnings = Vec::new();

        // Allowed style tokens
        let allowed_styles = [
            "bold",
            "italic",
            "underline",
            "black",
            "red",
            "green",
            "yellow",
            "blue",
            "magenta",
            "cyan",
            "white",
        ];

        let check_style = |name: &str, style: &str, warnings: &mut Vec<String>| {
            for tok in style.split_whitespace() {
                if !allowed_styles.contains(&tok) {
                    warnings.push(crate::messages::warn_unknown_style_token(name, tok));
                }
            }
        };

        check_style("directory", &self.directory.style, &mut warnings);
        check_style("claude_model", &self.claude_model.style, &mut warnings);
        check_style("git_branch", &self.git_branch.style, &mut warnings);
        check_style("git_status", &self.git_status.style, &mut warnings);

        // Unknown $tokens in top-level format
        for part in self.format.split_whitespace() {
            if let Some(tok) = part.strip_prefix('$') {
                match tok {
                    "directory" | "claude_model" | "git_branch" | "git_status"
                    | "claude_session" | "character" => {}
                    other => warnings.push(crate::messages::warn_unknown_format_token(other)),
                }
            }
        }

        warnings
    }

    /// Get a raw TOML table for an extra/unknown module section if present
    pub fn extra_module_table(&self, name: &str) -> Option<&toml::value::Table> {
        match self.extra_modules.get(name) {
            Some(toml::Value::Table(t)) => Some(t),
            _ => None,
        }
    }
}

#[cfg(test)]
mod validation_tests {
    use super::*;

    #[test]
    fn command_timeout_bounds() {
        let mut cfg = Config::default();
        cfg.command_timeout = 10;
        assert!(cfg.validate().is_err());
        cfg.command_timeout = 50;
        assert!(cfg.validate().is_ok());
        cfg.command_timeout = 600_000;
        assert!(cfg.validate().is_ok());
        cfg.command_timeout = 600_001;
        assert!(cfg.validate().is_err());
    }

    #[test]
    fn warns_on_unknown_style_tokens() {
        let mut cfg = Config::default();
        cfg.directory.style = "sparkly rainbow".to_string();
        let ws = cfg.collect_warnings();
        assert!(ws.iter().any(|w| w.contains("Unknown style token")));
    }

    #[test]
    fn warns_on_unknown_format_token() {
        let mut cfg = Config::default();
        cfg.format = "$directory $unknown $git_branch".to_string();
        let ws = cfg.collect_warnings();
        assert!(
            ws.iter()
                .any(|w| w.contains("Unknown format token: '$unknown'"))
        );
    }
}
