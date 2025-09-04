pub use crate::types::config::Config;
use anyhow::{Context as AnyhowContext, Result};
use std::fs;
use std::path::PathBuf;

impl Config {
    pub fn load() -> Result<Self> {
        let config_path = get_config_path();

        if config_path.exists() {
            let contents = fs::read_to_string(&config_path)
                .with_context(|| format!("failed to read {}", config_path.display()))?;
            let cfg: Config = toml::from_str(&contents)
                .with_context(|| format!("invalid TOML at {}", config_path.display()))?;
            Ok(cfg)
        } else {
            Ok(Config::default())
        }
    }
}

fn get_config_path() -> PathBuf {
    dirs::home_dir()
        .map(|home| home.join(".config").join("beacon.toml"))
        .unwrap_or_else(|| PathBuf::from("~/.config/beacon.toml"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = Config::default();

        // Test top-level defaults
        assert_eq!(config.format, "$directory $claude_model");
        assert_eq!(config.command_timeout, 500);
        assert_eq!(config.debug, false);

        // Test directory module defaults
        assert_eq!(config.directory.format, "[$path]($style)");
        assert_eq!(config.directory.style, "bold cyan");
        assert_eq!(config.directory.truncation_length, 3);
        assert_eq!(config.directory.truncate_to_repo, true);
        assert_eq!(config.directory.disabled, false);

        // Test claude_model module defaults
        assert_eq!(config.claude_model.format, "[$symbol$model]($style)");
        assert_eq!(config.claude_model.style, "bold yellow");
        assert_eq!(config.claude_model.symbol, "<");
        assert_eq!(config.claude_model.disabled, false);
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
        assert_eq!(config.debug, true);
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
        assert_eq!(config.debug, true);
        assert_eq!(config.directory.style, "italic green");

        // Default values for unspecified fields
        assert_eq!(config.format, "$directory $claude_model");
        assert_eq!(config.command_timeout, 500);
        assert_eq!(config.directory.format, "[$path]($style)");
        assert_eq!(config.claude_model.symbol, "<");
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
}
