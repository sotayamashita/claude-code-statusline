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
/// use claude_code_statusline_core::style::apply_style;
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

    // Heuristics to decide if the terminal supports truecolor. This keeps
    // behavior consistent across environments where 24-bit colors are not
    // fully supported and avoids foreground/background mismatch when a host
    // silently downgrades one channel differently from the other.
    fn supports_truecolor() -> bool {
        // Explicit override for tests or user preference
        if std::env::var("CCS_TRUECOLOR")
            .map(|v| v == "1")
            .unwrap_or(false)
        {
            return true;
        }
        if let Ok(v) = std::env::var("COLORTERM") {
            let v = v.to_lowercase();
            if v.contains("truecolor") || v.contains("24bit") {
                return true;
            }
        }
        if let Ok(t) = std::env::var("TERM") {
            let t = t.to_lowercase();
            if t.contains("direct") || t.contains("truecolor") {
                return true;
            }
        }
        false
    }

    // Convert an RGB color to the nearest ANSI 256-color index.
    // Algorithm: prefer xterm 6x6x6 color cube (16..231) and fall back to
    // grayscale ramp (232..255) when r≈g≈b. This mirrors common mappers.
    fn rgb_to_ansi256(r: u8, g: u8, b: u8) -> u8 {
        // If it's close to gray, map to grayscale range for better fidelity
        let rg = r as i32 - g as i32;
        let rb = r as i32 - b as i32;
        let gb = g as i32 - b as i32;
        let is_grayish = rg.abs() < 10 && rb.abs() < 10 && gb.abs() < 10;
        if is_grayish {
            // 24 grays, 8..238 step ~10
            let gray = ((r as u16 + g as u16 + b as u16) / 3) as u8;
            if gray < 8 {
                return 16; // nearest to black
            }
            if gray > 238 {
                return 231; // nearest to white from color cube
            }
            return 232 + ((gray as u16 - 8) / 10) as u8;
        }
        // Quantize each channel to 0..5 then map into 6x6x6 cube
        let to_6 = |v: u8| -> u8 { ((v as u16 * 5 + 127) / 255) as u8 };
        let r6 = to_6(r);
        let g6 = to_6(g);
        let b6 = to_6(b);
        16 + 36 * r6 + 6 * g6 + b6
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
            ColorSpec::Rgb(r, g, b) => {
                if supports_truecolor() {
                    codes.push(format!("38;2;{r};{g};{b}"));
                } else {
                    let n = rgb_to_ansi256(r, g, b);
                    codes.push(format!("38;5;{n}"));
                }
            }
            ColorSpec::NoneSet => {}
        }
    }
    if let Some(c) = bg {
        match c {
            ColorSpec::NamedNormal(idx) => codes.push((40 + idx).to_string()),
            ColorSpec::NamedBright(idx) => codes.push((100 + idx).to_string()),
            ColorSpec::Index(n) => codes.push(format!("48;5;{n}")),
            ColorSpec::Rgb(r, g, b) => {
                if supports_truecolor() {
                    codes.push(format!("48;2;{r};{g};{b}"));
                } else {
                    let n = rgb_to_ansi256(r, g, b);
                    codes.push(format!("48;5;{n}"));
                }
            }
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
    // First, replace known tokens except "$style" using deterministic,
    // longest-key-first ordering to avoid overlaps (e.g., $git vs $git_branch).
    let mut replaced = String::from(format);
    let mut keys: Vec<&str> = tokens.keys().copied().filter(|k| *k != "style").collect();
    // Sort by descending length so longer tokens are substituted first
    keys.sort_by_key(|k| std::cmp::Reverse(k.len()));
    for k in keys {
        if let Some(v) = tokens.get(k) {
            let needle = format!("${k}");
            replaced = replaced.replace(&needle, v);
        }
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
        // Accept either truecolor or ANSI-256 downgraded output depending on env
        assert!(s.contains("38;2;191;87;0") || s.contains("38;5;"));
        assert!(s.contains("48;2;0;51;102") || s.contains("48;5;"));
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
    fn token_substitution_uses_longest_key_first() {
        use std::collections::HashMap;
        // Simulate overlapping token names like $git and $git_branch
        let mut tokens = HashMap::new();
        tokens.insert("git", String::from("G"));
        tokens.insert("git_branch", String::from("BR"));

        let out = render_with_style_template("$git_branch $git", &tokens, "");
        // Expect both tokens fully replaced without partial corruption
        assert_eq!(out, "BR G");
        assert!(!out.contains("_branch"));
    }

    #[test]
    fn style_none_handling() {
        let s = apply_style("X", "fg:none italic");
        assert!(s.contains("3"));
        assert!(!s.contains("38;"));
    }

    #[test]
    fn rgb_foreground_background_downgrade_is_consistent() {
        // Ensure that when truecolor is not detected, the same RGB hex
        // maps to the same ANSI-256 index for both fg and bg.
        use std::sync::{Mutex, OnceLock};
        static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        let _g = LOCK.get_or_init(|| Mutex::new(())).lock().unwrap();
        // Force non-truecolor environment
        unsafe {
            std::env::remove_var("CCS_TRUECOLOR");
            std::env::set_var("COLORTERM", "");
            std::env::set_var("TERM", "xterm-256color");
        }

        let fg = apply_style("X", "#9A348E");
        let bg = apply_style("X", "bg:#9A348E");
        // Extract the 256-color index numbers if present
        let idx_fg = fg
            .split("38;5;")
            .nth(1)
            .and_then(|s| s.split('m').next())
            .and_then(|n| n.parse::<u16>().ok());
        let idx_bg = bg
            .split("48;5;")
            .nth(1)
            .and_then(|s| s.split('m').next())
            .and_then(|n| n.parse::<u16>().ok());
        if let (Some(a), Some(b)) = (idx_fg, idx_bg) {
            assert_eq!(a, b);
        } else {
            // In environments with truecolor this test isn't meaningful.
            // Ensure at least both contain truecolor sequences then.
            assert!(fg.contains("38;2;") && bg.contains("48;2;"));
        }
    }
}
