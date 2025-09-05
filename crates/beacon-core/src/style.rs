//! ANSI color and text styling utilities
//!
//! This module provides functions for applying ANSI escape codes to
//! terminal text, enabling colored and styled output in the status line.

/// Applies ANSI styling to text for terminal display
///
/// Takes a text string and a style specification, returning the text
/// wrapped in appropriate ANSI escape codes.
///
/// # Arguments
///
/// * `text` - The text to style
/// * `style` - Space-separated style tokens
///
/// # Supported Style Tokens
///
/// Text styles:
/// - `bold` - Bold text
/// - `italic` - Italic text
/// - `underline` - Underlined text
///
/// Colors:
/// - `black`, `red`, `green`, `yellow`
/// - `blue`, `magenta`, `cyan`, `white`
///
/// # Examples
///
/// ```
/// use beacon_core::style::apply_style;
///
/// let styled = apply_style("Hello", "bold red");
/// // Returns: "\x1b[1;31mHello\x1b[0m"
///
/// let multi = apply_style("World", "bold italic blue");
/// // Returns: "\x1b[1;3;34mWorld\x1b[0m"
/// ```
///
/// # Notes
///
/// - Unknown tokens are silently ignored
/// - If no valid tokens are found, returns the original text
/// - Multiple styles can be combined (e.g., "bold red underline")
pub fn apply_style(text: &str, style: &str) -> String {
    // Table-driven mapping for minimal, portable ANSI codes
    const STYLE_CODES: &[(&str, &str)] = &[("bold", "1"), ("italic", "3"), ("underline", "4")];
    const COLOR_CODES: &[(&str, &str)] = &[
        ("black", "30"),
        ("red", "31"),
        ("green", "32"),
        ("yellow", "33"),
        ("blue", "34"),
        ("magenta", "35"),
        ("cyan", "36"),
        ("white", "37"),
    ];

    let mut codes: Vec<&'static str> = Vec::new();

    for token in style.split_whitespace() {
        let t = token.to_lowercase();
        if let Some((_, code)) = STYLE_CODES.iter().find(|(k, _)| *k == t) {
            codes.push(*code);
            continue;
        }
        if let Some((_, code)) = COLOR_CODES.iter().find(|(k, _)| *k == t) {
            codes.push(*code);
            continue;
        }
        // Unknown tokens are ignored
    }

    if codes.is_empty() {
        return text.to_string();
    }

    format!("\x1b[{}m{}\x1b[0m", codes.join(";"), text)
}

/// Render a simple module-local format string that can contain variable tokens
/// like `$path`, `$model`, `$symbol`, `$branch` and optional bracket-style
/// annotations: `[$content]($style)`.
///
/// - Variables inside the bracket content are substituted first.
/// - The style inside parentheses can be a literal (e.g. "bold yellow") or
///   `$style` which resolves to `default_style`.
/// - If there is no bracket-style annotation, the variables are substituted and
///   returned as-is.
pub fn render_with_style_template(
    format: &str,
    tokens: &std::collections::HashMap<&str, String>,
    default_style: &str,
) -> String {
    // First, replace known tokens except "$style"
    let mut replaced = String::from(format);
    for (k, v) in tokens.iter() {
        let needle = format!("${k}");
        replaced = replaced.replace(&needle, v);
    }

    // Process zero or more occurrences of [text](style)
    let mut out = String::new();
    let mut rest = replaced.as_str();
    while let Some(lbrack) = rest.find('[') {
        out.push_str(&rest[..lbrack]);
        rest = &rest[lbrack + 1..];

        if let Some(rbrack) = rest.find(']') {
            let inner = &rest[..rbrack];
            rest = &rest[rbrack + 1..];
            if rest.starts_with('(')
                && let Some(rparen) = rest.find(')')
            {
                let style_spec = &rest[1..rparen];
                rest = &rest[rparen + 1..];

                // Resolve style
                let style_to_use = if style_spec == "$style" {
                    default_style
                } else {
                    style_spec
                };
                out.push_str(&apply_style(inner, style_to_use));
                continue;
            }
            // If we get here, brackets weren't in the expected form; restore literally
            out.push('[');
            out.push_str(inner);
            out.push(']');
            continue;
        } else {
            // No closing bracket, push the rest literally
            out.push('[');
            out.push_str(rest);
            rest = "";
            break;
        }
    }
    out.push_str(rest);
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn applies_bold_yellow() {
        let s = apply_style("X", "bold yellow");
        assert!(s.starts_with("\u{1b}[") && s.contains("1;33") && s.ends_with("\u{1b}[0m"));
        assert!(s.contains('X'));
    }

    #[test]
    fn ignores_unknown_tokens() {
        assert_eq!(apply_style("X", "unknown"), "X");
    }

    #[test]
    fn mixed_known_and_unknown_tokens_are_stable() {
        // Unknown tokens should be ignored, known tokens applied
        let s = apply_style("Y", "bold sparkly yellow foo");
        // Should include ANSI for bold (1) and yellow (33)
        assert!(s.starts_with("\u{1b}["));
        assert!(s.contains("1;33") || s.contains("33;1"));
        assert!(s.ends_with("\u{1b}[0m"));
        assert!(s.contains('Y'));
    }

    #[test]
    fn renders_bracket_style_template() {
        use std::collections::HashMap;
        let mut tokens = HashMap::new();
        tokens.insert("path", String::from("~/proj"));
        let out = render_with_style_template("[$path]($style)", &tokens, "bold blue");
        assert!(out.contains("~/proj"));
        assert!(out.starts_with("\u{1b}["));
        assert!(out.ends_with("\u{1b}[0m"));
    }
}
