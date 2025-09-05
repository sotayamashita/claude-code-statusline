//! Centralized message constants and formatting utilities
//!
//! This module provides consistent error messages and warnings used
//! throughout the application. Centralizing messages helps maintain
//! consistency and facilitates future internationalization.

/// Error message displayed when configuration is invalid
pub const MSG_FAILED_INVALID_CONFIG: &str = "Failed to build status line due to invalid config";

/// Error message displayed when no input is received from stdin
pub const MSG_FAILED_EMPTY_INPUT: &str = "Failed to build status line due to empty input";

/// Error message displayed when JSON parsing fails
pub const MSG_FAILED_INVALID_JSON: &str = "Failed to build status line due to invalid json";

/// Generates a warning message for unknown style tokens
///
/// # Arguments
///
/// * `module_name` - Name of the module with the invalid style
/// * `token` - The unknown style token
///
/// # Returns
///
/// A formatted warning message
///
/// # Examples
///
/// ```
/// use beacon_core::messages::warn_unknown_style_token;
///
/// let msg = warn_unknown_style_token("directory", "blink");
/// assert_eq!(msg, "Unknown style token in directory.style: 'blink' (ignored)");
/// ```
pub fn warn_unknown_style_token(module_name: &str, token: &str) -> String {
    format!("Unknown style token in {module_name}.style: '{token}' (ignored)")
}

/// Generates a warning message for unknown format tokens
///
/// # Arguments
///
/// * `token` - The unknown format token (without $ prefix)
///
/// # Returns
///
/// A formatted warning message
///
/// # Examples
///
/// ```
/// use beacon_core::messages::warn_unknown_format_token;
///
/// let msg = warn_unknown_format_token("unknown");
/// assert_eq!(msg, "Unknown format token: '$unknown'");
/// ```
pub fn warn_unknown_format_token(token: &str) -> String {
    format!("Unknown format token: '${token}'")
}
