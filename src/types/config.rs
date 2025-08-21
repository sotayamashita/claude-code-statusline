use crate::modules::ModuleConfig;
use serde::{Deserialize, Serialize};
use std::any::Any;

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
}

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
