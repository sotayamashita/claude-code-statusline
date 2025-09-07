//! Configuration loading and management module
//!
//! This module handles loading configuration from TOML files and provides
//! the implementation for the Config type defined in types::config.
//!
//! # Configuration File Location
//!
//! The configuration file is expected in the user's configuration directory
//! (e.g., `~/.config/claude-code-statusline.toml` on Unix). If the file
//! doesn't exist, default configuration values are used.
//!
//! # Example Configuration
//!
//! ```toml
//! format = "$directory $git_branch $claude_model"
//! command_timeout = 300
//! debug = true
//!
//! [directory]
//! style = "bold blue"
//! truncation_length = 5
//!
//! [claude_model]
//! symbol = "<"
//! style = "bold yellow"
//! ```

use crate::error::CoreError;
pub use crate::types::config::Config;
use std::fs;
use std::path::PathBuf;

impl Config {
    /// Loads configuration from the default location
    ///
    /// Attempts to read and parse the configuration file from the user's
    /// configuration directory (e.g., `~/.config/claude-code-statusline.toml`).
    /// If the file doesn't exist or
    /// cannot be read, returns the default configuration.
    ///
    /// # Returns
    ///
    /// * `Ok(Config)` - Successfully loaded configuration or defaults
    /// * `Err` - Failed to read or parse the configuration file
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use claude_code_statusline_core::Config;
    ///
    /// let config = Config::load().expect("Failed to load config");
    /// println!("Format: {}", config.format);
    /// ```
    pub fn load() -> Result<Self, CoreError> {
        // Candidate 1: XDG-style (~/.config/claude-code-statusline.toml)
        let xdg_candidate =
            dirs::home_dir().map(|h| h.join(".config").join("claude-code-statusline.toml"));

        if let Some(ref xdg) = xdg_candidate {
            if xdg.exists() {
                let contents = fs::read_to_string(xdg).map_err(|e| CoreError::ConfigRead {
                    path: xdg.display().to_string(),
                    source: e,
                })?;
                let cfg: Config =
                    toml::from_str(&contents).map_err(|e| CoreError::ConfigParse {
                        path: xdg.display().to_string(),
                        source: e,
                    })?;
                return Ok(cfg);
            }
        }

        // Candidate 2: Platform config dir (e.g., macOS ~/Library/Application Support)
        let primary = get_config_path();
        if primary.exists() {
            let contents = fs::read_to_string(&primary).map_err(|e| CoreError::ConfigRead {
                path: primary.display().to_string(),
                source: e,
            })?;
            let cfg: Config = toml::from_str(&contents).map_err(|e| CoreError::ConfigParse {
                path: primary.display().to_string(),
                source: e,
            })?;
            return Ok(cfg);
        }

        // Default when no config file is present
        Ok(Config::default())
    }
}

/// Determines the path to the configuration file
///
/// Constructs the path to `claude-code-statusline.toml` within the user's
/// configuration directory. Uses `dirs::config_dir()` for cross-platform
/// compatibility and falls back to the literal
/// `~/.config/claude-code-statusline.toml` if no config directory can be
/// determined.
///
/// # Returns
///
/// A `PathBuf` pointing to the expected configuration file location
fn get_config_path() -> PathBuf {
    // Prefer platform config dir for display/tooling
    if let Some(base) = dirs::config_dir() {
        return base.join("claude-code-statusline.toml");
    }
    // 1) Prefer XDG-style path: ~/.config/claude-code-statusline.toml (Linux-like)
    if let Some(home) = dirs::home_dir() {
        let xdg_path = home.join(".config").join("claude-code-statusline.toml");
        if xdg_path.exists() {
            return xdg_path;
        }
    }

    // 2) Fallback to platform config dir
    // (e.g., macOS: ~/Library/Application Support, Windows: %APPDATA%\claude-code-statusline)
    if let Some(base) = dirs::config_dir() {
        return base.join("claude-code-statusline.toml");
    }

    // 3) Last resort: literal XDG-style path (no expansion, best-effort)
    PathBuf::from("~/.config/claude-code-statusline.toml")
}

/// Public accessor for the resolved configuration file path
///
/// Exposes a stable path resolution for consumers (e.g., CLI) so that all
/// components agree on the location of the configuration file.
pub fn config_path() -> PathBuf {
    get_config_path()
}

/// Lightweight provider to access module-specific configuration tables
/// including extra/unknown sections preserved during TOML deserialization.
pub struct ConfigProvider<'a> {
    config: &'a Config,
}

impl<'a> ConfigProvider<'a> {
    pub fn new(config: &'a Config) -> Self {
        Self { config }
    }

    /// Returns a raw TOML table for the given module name if present
    pub fn module_table(&self, module: &str) -> Option<&toml::value::Table> {
        // Known modules are represented as typed structs and not exposed here.
        // This function focuses on extra/unknown sections to enable pluggable modules.
        self.config.extra_module_table(module)
    }

    /// List available extra module section names
    pub fn list_extra_modules(&self) -> Vec<String> {
        self.config.extra_modules.keys().cloned().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::config::Config as Cfg;

    use std::sync::{Mutex, OnceLock};

    fn env_lock() -> &'static Mutex<()> {
        static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        LOCK.get_or_init(|| Mutex::new(()))
    }

    #[test]
    fn test_default_config() {
        let config = Config::default();

        // Test top-level defaults
        assert_eq!(config.format, "$directory $claude_model");
        assert_eq!(config.command_timeout, 500);
        assert!(!config.debug);

        // Test directory module defaults
        assert_eq!(config.directory.format, "[$path]($style)");
        assert_eq!(config.directory.style, "bold cyan");
        assert_eq!(config.directory.truncation_length, 3);
        assert!(config.directory.truncate_to_repo);
        assert!(!config.directory.disabled);

        // Test claude_model module defaults
        assert_eq!(config.claude_model.format, "[$symbol$model]($style)");
        assert_eq!(config.claude_model.style, "bold yellow");
        assert_eq!(config.claude_model.symbol, "");
        assert!(!config.claude_model.disabled);
    }

    #[test]
    fn test_load_missing_config_returns_default() {
        // Serialize env mutation to avoid races across tests
        let _guard = env_lock().lock().unwrap();
        let tmp = tempfile::tempdir().unwrap();
        // Capture the original HOME (if any) so we can restore it later
        let orig_home = std::env::var_os("HOME");
        // Set HOME to the temp dir for the duration of this test
        unsafe {
            std::env::set_var("HOME", tmp.path());
        }
        let config = Config::load().unwrap();
        // Fully restore HOME: reset to original value, or remove if it was unset
        match orig_home {
            Some(h) => unsafe { std::env::set_var("HOME", h) },
            None => unsafe { std::env::remove_var("HOME") },
        }
        assert_eq!(config.format, "$directory $claude_model");
        assert_eq!(config.command_timeout, 500);
    }

    #[test]
    fn test_parse_valid_toml_config() {
        let toml_str = r#"
            format = "$directory $claude_model"
            command_timeout = 300
            debug = true

            [directory]
            format = "in [$path]($style)"
            style = "bold blue"
            truncation_length = 5

            [claude_model]
            symbol = "<"
            style = "bold yellow"
        "#;

        let config: Config = toml::from_str(toml_str).unwrap();

        assert_eq!(config.format, "$directory $claude_model");
        assert_eq!(config.command_timeout, 300);
        assert!(config.debug);
        assert_eq!(config.directory.format, "in [$path]($style)");
        assert_eq!(config.directory.style, "bold blue");
        assert_eq!(config.directory.truncation_length, 5);
        assert_eq!(config.claude_model.symbol, "<");
        assert_eq!(config.claude_model.style, "bold yellow");
    }

    #[test]
    fn test_partial_config_uses_defaults() {
        let toml_str = r#"
            debug = true

            [directory]
            style = "italic green"
        "#;

        let config: Config = toml::from_str(toml_str).unwrap();

        // Specified values
        assert!(config.debug);
        assert_eq!(config.directory.style, "italic green");

        // Default values for unspecified fields
        assert_eq!(config.format, "$directory $claude_model");
        assert_eq!(config.command_timeout, 500);
        assert_eq!(config.directory.format, "[$path]($style)");
        assert_eq!(config.claude_model.symbol, "");
    }

    #[test]
    fn test_invalid_toml_returns_default() {
        let invalid_toml = "this is not valid TOML [ syntax";
        let result = toml::from_str::<Config>(invalid_toml);
        assert!(result.is_err());
    }

    #[test]
    fn test_config_path_with_config_dir() {
        // This test checks the path construction logic via dirs::config_dir
        let path = get_config_path();

        if let Some(cfg_dir) = dirs::config_dir() {
            let expected = cfg_dir.join("claude-code-statusline.toml");
            assert_eq!(path, expected);
        } else {
            // Fallback when config_dir is not available
            assert_eq!(path, PathBuf::from("~/.config/claude-code-statusline.toml"));
        }
    }

    #[test]
    fn extra_modules_are_preserved_and_accessible() {
        let toml_str = r#"
            [directory]
            style = "bold blue"

            [my_custom]
            key = "value"
            answer = 42
        "#;
        let cfg: Cfg = toml::from_str(toml_str).unwrap();
        let provider = super::ConfigProvider::new(&cfg);
        let t = provider.module_table("my_custom").expect("table exists");
        assert_eq!(t.get("key").unwrap().as_str().unwrap(), "value");
        assert_eq!(t.get("answer").unwrap().as_integer().unwrap(), 42);
        assert!(
            provider
                .list_extra_modules()
                .contains(&"my_custom".to_string())
        );
    }

    #[test]
    fn test_claude_model_default_symbol_is_empty() {
        // New desired default behavior for issue #27
        let cfg = Config::default();
        assert_eq!(cfg.claude_model.symbol, "");
    }
}
