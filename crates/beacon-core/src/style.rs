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
    #[derive(Clone, Copy)]
    enum ColorSpec {
        NamedNormal(u8), // 30..=37 (FG) / 40..=47 (BG) base offset will be applied
        NamedBright(u8), // 90..=97 / 100..=107 (store 0..=7)
        Index(u8),       // 0..=255
        Rgb(u8, u8, u8), // truecolor
        NoneSet,         // explicit none
    }

    fn parse_named(name: &str) -> Option<u8> {
        match name {
            "black" => Some(0),
            "red" => Some(1),
            "green" => Some(2),
            "yellow" => Some(3),
            "blue" => Some(4),
            "magenta" => Some(5),
            "cyan" => Some(6),
            "white" => Some(7),
            _ => None,
        }
    }

    fn parse_color_spec(spec: &str) -> Option<ColorSpec> {
        let s = spec.to_lowercase();
        if s == "none" {
            return Some(ColorSpec::NoneSet);
        }
        if let Some(hex) = s.strip_prefix('#') {
            if hex.len() == 6 {
                let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
                let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
                let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
                return Some(ColorSpec::Rgb(r, g, b));
            }
        }
        if s.chars().all(|c| c.is_ascii_digit()) {
            if let Ok(n) = s.parse::<u16>() {
                if n <= 255 {
                    return Some(ColorSpec::Index(n as u8));
                }
            }
        }
        if let Some(n) = s.strip_prefix("bright-") {
            if let Some(idx) = parse_named(n) {
                return Some(ColorSpec::NamedBright(idx));
            }
        }
        if let Some(idx) = parse_named(&s) {
            return Some(ColorSpec::NamedNormal(idx));
        }
        None
    }

    // Modifiers
    let mut bold = false;
    let mut italic = false;
    let mut underline = false;

    // Color channels: last one wins
    let mut fg: Option<ColorSpec> = None;
    let mut bg: Option<ColorSpec> = None;

    for token in style.split_whitespace() {
        let t = token.to_lowercase();
        match t.as_str() {
            "bold" => {
                bold = true;
                continue;
            }
            "italic" => {
                italic = true;
                continue;
            }
            "underline" => {
                underline = true;
                continue;
            }
            _ => {}
        }

        if let Some(rest) = t.strip_prefix("fg:") {
            fg = parse_color_spec(rest);
            continue;
        }
        if let Some(rest) = t.strip_prefix("bg:") {
            bg = parse_color_spec(rest);
            continue;
        }

        // Bare color spec is treated as foreground
        if let Some(cs) = parse_color_spec(&t) {
            fg = Some(cs);
        } else {
            // Unknown token: ignore
        }
    }

    let mut codes: Vec<String> = Vec::with_capacity(5);
    if bold {
        codes.push("1".to_string());
    }
    if italic {
        codes.push("3".to_string());
    }
    if underline {
        codes.push("4".to_string());
    }

    if let Some(c) = fg {
        match c {
            ColorSpec::NamedNormal(idx) => codes.push((30 + idx).to_string()),
            ColorSpec::NamedBright(idx) => codes.push((90 + idx).to_string()),
            ColorSpec::Index(n) => codes.push(format!("38;5;{n}")),
            ColorSpec::Rgb(r, g, b) => codes.push(format!("38;2;{r};{g};{b}")),
            ColorSpec::NoneSet => {}
        }
    }
    if let Some(c) = bg {
        match c {
            ColorSpec::NamedNormal(idx) => codes.push((40 + idx).to_string()),
            ColorSpec::NamedBright(idx) => codes.push((100 + idx).to_string()),
            ColorSpec::Index(n) => codes.push(format!("48;5;{n}")),
            ColorSpec::Rgb(r, g, b) => codes.push(format!("48;2;{r};{g};{b}")),
            ColorSpec::NoneSet => {}
        }
    }

    if codes.is_empty() {
        return text.to_string();
    }
    let sgr = codes.join(";");
    format!("\x1b[{sgr}m{text}\x1b[0m")
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
        if *k == "style" {
            // Preserve "$style" for bracket style resolution to honor default_style
            continue;
        }
        let needle = format!("${k}");
        replaced = replaced.replace(&needle, v);
    }

    // Robust pass to process [text](style) while ignoring ANSI escape
    // sequences already present in the string (e.g., from substituted
    // module outputs). We skip any ESC[..terminator sequences to avoid
    // misinterpreting the '[' in "\x1b[" as a text-group opener.
    let bytes = replaced.as_bytes();
    let mut i = 0;
    let len = bytes.len();
    let mut out = String::with_capacity(len + 16);
    // Start index of the current literal chunk to be copied as-is
    let mut seg_start = 0usize;

    while i < len {
        let b = bytes[i];
        if b == 0x1b {
            // ESC: copy SGR/CSI sequence verbatim
            let start = i;
            i += 1; // Skip ESC
            if i < len && bytes[i] == b'[' {
                i += 1;
                while i < len {
                    let bb = bytes[i];
                    if (0x40..=0x7E).contains(&bb) {
                        i += 1; // include terminator
                        break;
                    }
                    i += 1;
                }
            }
            // flush preceding literal then CSI
            if seg_start < start {
                out.push_str(&replaced[seg_start..start]);
            }
            out.push_str(&replaced[start..i]);
            seg_start = i;
            continue;
        }

        if b == b'[' {
            // Potential text group
            // Flush any preceding literal chunk
            if seg_start < i {
                out.push_str(&replaced[seg_start..i]);
            }
            let mut j = i + 1;
            while j < len && bytes[j] != b']' {
                j += 1;
            }
            if j < len && j + 1 < len && bytes[j + 1] == b'(' {
                // Find right parenthesis
                let mut k = j + 2;
                while k < len && bytes[k] != b')' {
                    k += 1;
                }
                if k < len {
                    let inner = &replaced[i + 1..j];
                    let style_spec = &replaced[j + 2..k];
                    let style_to_use = if style_spec == "$style" {
                        default_style
                    } else {
                        style_spec
                    };
                    out.push_str(&apply_style(inner, style_to_use));
                    i = k + 1;
                    seg_start = i;
                    continue;
                }
            }

            // Fallback: literal '['
            out.push('[');
            i += 1;
            seg_start = i;
            continue;
        }

        // Regular byte; advance. We'll copy in bulk using seg_start when needed.
        i += 1;
    }
    // Flush remaining literal
    if seg_start < len {
        out.push_str(&replaced[seg_start..len]);
    }
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

    #[test]
    fn ignores_ansi_sequences_when_parsing_text_groups() {
        use std::collections::HashMap;
        // Pre-styled token (simulating a module output already ANSI-wrapped)
        let styled = apply_style("X", "fg:#ff0000");
        let mut tokens = HashMap::new();
        tokens.insert("t", styled);
        // Surrounding group should be styled, but the existing ANSI inside $t
        // must not confuse the parser.
        let s = render_with_style_template("[](bg:#003366)$t", &tokens, "");
        // After stripping ANSI, we should see the glyph and the token text only.
        let plain = String::from_utf8(strip_ansi_escapes::strip(s)).unwrap();
        assert_eq!(plain, "X");
    }

    // New spec tests for fg:/bg: and extended colors

    #[test]
    fn style_named_fg_bg() {
        let s = apply_style("X", "bold fg:green bg:black");
        assert!(s.starts_with("\u{1b}["));
        // Contains bold(1), fg green(32), bg black(40) in any order
        assert!(s.contains("1"));
        assert!(s.contains("32"));
        assert!(s.contains("40"));
        assert!(s.ends_with("\u{1b}[0m"));
    }

    #[test]
    fn style_bright_named() {
        let s = apply_style("X", "bright-yellow bg:bright-blue");
        // bright yellow = 93, bright blue background = 104
        assert!(s.contains("93"));
        assert!(s.contains("104"));
    }

    #[test]
    fn style_8bit_indexes() {
        let s = apply_style("X", "fg:196 bg:238");
        assert!(s.contains("38;5;196"));
        assert!(s.contains("48;5;238"));
    }

    #[test]
    fn style_hex_truecolor() {
        let s = apply_style("X", "fg:#bf5700 bg:#003366");
        assert!(s.contains("38;2;191;87;0"));
        assert!(s.contains("48;2;0;51;102"));
    }

    #[test]
    fn style_bare_color_equivalence() {
        let s1 = apply_style("X", "yellow");
        let s2 = apply_style("X", "fg:yellow");
        assert_eq!(s1, s2);
    }

    #[test]
    fn style_unknown_tokens_stability() {
        let s = apply_style("X", "bold sparkle fg:green foo");
        assert!(s.contains("1"));
        assert!(s.contains("32") || s.contains("38;2;") || s.contains("38;5;"));
        // Should not introduce 38; for fg if using named mapping 32; but mainly ensure still wrapped
        assert!(s.starts_with("\u{1b}[") && s.ends_with("\u{1b}[0m"));
    }

    #[test]
    fn style_none_handling() {
        let s = apply_style("X", "fg:none italic");
        assert!(s.contains("3"));
        assert!(!s.contains("38;"));
    }
}
