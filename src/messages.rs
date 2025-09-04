// Centralized user-facing messages for stdout/stderr
// Keeping strings stable helps tests and future i18n.

pub const MSG_FAILED_INVALID_CONFIG: &str = "Failed to build status line due to invalid config";
pub const MSG_FAILED_EMPTY_INPUT: &str = "Failed to build status line due to empty input";
pub const MSG_FAILED_INVALID_JSON: &str = "Failed to build status line due to invalid json";

// Warning/validation message helpers
pub fn warn_unknown_style_token(module_name: &str, token: &str) -> String {
    format!("Unknown style token in {module_name}.style: '{token}' (ignored)")
}

pub fn warn_unknown_format_token(token: &str) -> String {
    format!("Unknown format token: '${token}'")
}
