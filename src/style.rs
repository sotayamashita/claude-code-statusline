/// Minimal ANSI styling utilities for module output
///
/// Supported tokens (space-separated):
/// - text styles: bold, italic, underline
/// - colors: black, red, green, yellow, blue, magenta, cyan, white
///
/// Unknown tokens are ignored. If no known tokens are present, the input text
/// is returned unchanged.
pub fn apply_style(text: &str, style: &str) -> String {
    let mut codes: Vec<&str> = Vec::new();

    for token in style.split_whitespace() {
        match token.to_lowercase().as_str() {
            // text styles
            "bold" => codes.push("1"),
            "italic" => codes.push("3"),
            "underline" => codes.push("4"),

            // foreground colors
            "black" => codes.push("30"),
            "red" => codes.push("31"),
            "green" => codes.push("32"),
            "yellow" => codes.push("33"),
            "blue" => codes.push("34"),
            "magenta" => codes.push("35"),
            "cyan" => codes.push("36"),
            "white" => codes.push("37"),

            _ => {}
        }
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
            if rest.starts_with('(') {
                if let Some(rparen) = rest.find(')') {
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
