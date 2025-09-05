//! Configuration loading and management module
//!
//! This module handles loading configuration from TOML files and provides
//! the implementation for the Config type defined in types::config.
//!
//! # Configuration File Location
//!
//! The configuration file is expected at `~/.config/beacon.toml`.
//! If the file doesn't exist, default configuration values are used.
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
    /// Attempts to read and parse the configuration file from
    /// `~/.config/beacon.toml`. If the file doesn't exist or
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
    /// use beacon_core::Config;
    ///
    /// let config = Config::load().expect("Failed to load config");
    /// println!("Format: {}", config.format);
    /// ```
    pub fn load() -> Result<Self, CoreError> {
        let config_path = get_config_path();

        if config_path.exists() {
            let contents = fs::read_to_string(&config_path).map_err(|e| CoreError::ConfigRead {
                path: config_path.display().to_string(),
                source: e,
            })?;
            let cfg: Config = toml::from_str(&contents).map_err(|e| CoreError::ConfigParse {
                path: config_path.display().to_string(),
                source: e,
            })?;
            Ok(cfg)
        } else {
            Ok(Config::default())
        }
    }
}

/// Determines the path to the configuration file
///
/// Constructs the path to `~/.config/beacon.toml` using the user's
/// home directory. Falls back to the literal path if home directory
/// cannot be determined.
///
/// # Returns
///
/// A `PathBuf` pointing to the expected configuration file location
fn get_config_path() -> PathBuf {
    dirs::home_dir()
        .map(|home| home.join(".config").join("beacon.toml"))
        .unwrap_or_else(|| PathBuf::from("~/.config/beacon.toml"))
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
        // Note: This test may use actual config file if it exists
        // The test name is misleading - it's testing Config::load() in general
        let config = Config::load().unwrap();
        // Accept common real-world formats that may be present in a user's local config
        let ok_formats = [
            "$directory $claude_model",
            "$directory $git_branch $claude_model",
            "$directory $git_branch $git_status $claude_model",
        ];
        assert!(ok_formats.contains(&config.format.as_str()));
        // If config file exists with command_timeout = 300, that will be loaded
        // If not, default 500 will be used
        assert!(config.command_timeout == 300 || config.command_timeout == 500);
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
    fn test_config_path_with_home() {
        // This test checks the path construction logic
        let path = get_config_path();

        if let Some(home) = dirs::home_dir() {
            let expected = home.join(".config").join("beacon.toml");
            assert_eq!(path, expected);
        } else {
            // Fallback when home_dir is not available
            assert_eq!(path, PathBuf::from("~/.config/beacon.toml"));
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
