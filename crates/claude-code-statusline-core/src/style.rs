//! ANSI color and text styling utilities
//!
//! This module parses the same style syntax as Starship and evaluates
//! templates in a way that preserves the preceding segment's colors and
//! attributes.

use std::collections::HashMap;

/// Convenience utility that applies a style to the provided text.
///
/// Internally this shares the same parser and renderer as
/// `render_with_style_template`, giving Starship-compatible token handling.
pub fn apply_style(text: &str, style: &str) -> String {
    let mut tokens = HashMap::new();
    tokens.insert("value", text.to_string());
    render_with_style_template("[$value]($style)", &tokens, style)
}

/// Parses `[$text](style)` markup and produces an ANSI-formatted string.
///
/// - Replaces `$foo` tokens using the supplied map (with `$style` treated
///   specially).
/// - Interprets the trailing style argument using Starship-compatible rules,
///   supporting inheritance via `prev_fg`/`prev_bg` and resets such as
///   `bg:none`.
/// - When the style is invalid (for example `fg:none`), inserts a reset and
///   resumes with the default style.
/// - Always appends a trailing `\x1b[0m` to close out styling.
pub fn render_with_style_template(
    format: &str,
    tokens: &HashMap<&str, String>,
    default_style: &str,
) -> String {
    // Step 1: replace `$token` occurrences, longest key first to match the
    // legacy rules.
    let mut replaced = String::from(format);
    let mut keys: Vec<&str> = tokens.keys().copied().filter(|k| *k != "style").collect();
    keys.sort_by_key(|k| std::cmp::Reverse(k.len()));
    for key in keys {
        if let Some(value) = tokens.get(key) {
            let needle = format!("${key}");
            replaced = replaced.replace(&needle, value);
        }
    }

    // Step 2: split `[text](style)` groups while leaving existing ANSI escape
    // sequences untouched.
    let mut segments = Vec::new();
    let bytes = replaced.as_bytes();
    let mut i = 0usize;
    let len = bytes.len();
    let mut literal_start = 0usize;

    while i < len {
        let b = bytes[i];
        if b == 0x1b {
            if literal_start < i {
                push_plain_segment(&mut segments, &replaced[literal_start..i]);
            }
            let esc_start = i;
            i += 1;
            if i < len && bytes[i] == b'[' {
                i += 1;
                while i < len {
                    let bb = bytes[i];
                    if (0x40..=0x7e).contains(&bb) {
                        i += 1;
                        break;
                    }
                    i += 1;
                }
            }
            push_plain_segment(&mut segments, &replaced[esc_start..i]);
            literal_start = i;
            continue;
        }

        if b == b'[' {
            if literal_start < i {
                push_plain_segment(&mut segments, &replaced[literal_start..i]);
            }
            let mut close = i + 1;
            while close < len && bytes[close] != b']' {
                close += 1;
            }
            if close < len && close + 1 < len && bytes[close + 1] == b'(' {
                let mut paren = close + 2;
                while paren < len && bytes[paren] != b')' {
                    paren += 1;
                }
                if paren < len {
                    let inner = &replaced[i + 1..close];
                    let style_spec = &replaced[close + 2..paren];
                    let style_to_use = if style_spec == "$style" {
                        default_style
                    } else {
                        style_spec
                    };
                    match StyleSpec::parse(style_to_use) {
                        ParseOutcome::None => {
                            push_plain_segment(&mut segments, inner);
                        }
                        ParseOutcome::Invalid => {
                            segments.push(Segment {
                                text: inner.to_string(),
                                style: SegmentStyle::Invalid,
                            });
                        }
                        ParseOutcome::Spec(spec) => {
                            segments.push(Segment {
                                text: inner.to_string(),
                                style: SegmentStyle::Explicit(spec),
                            });
                        }
                    }
                    i = paren + 1;
                    literal_start = i;
                    continue;
                }
            }
            // Fall back to treating the byte as a literal '[' when we cannot
            // find a closing pair.
            push_plain_segment(&mut segments, "[");
            i += 1;
            literal_start = i;
            continue;
        }

        i += 1;
    }

    if literal_start < len {
        push_plain_segment(&mut segments, &replaced[literal_start..len]);
    }

    // Step 3: render the collected segments into a single ANSI string.
    let truecolor = supports_truecolor();
    let body = render_segments(&segments, truecolor);
    let mut out = String::with_capacity(body.len() + 4);
    out.push_str(&body);
    out.push_str("\x1b[0m");
    out
}

// --- Internal implementation ------------------------------------------------

#[derive(Clone, Debug)]
struct Segment {
    text: String,
    style: SegmentStyle,
}

#[derive(Clone, Debug)]
enum SegmentStyle {
    /// No explicit style; inherit whatever came before.
    None,
    /// An explicit style spec was provided.
    Explicit(StyleSpec),
    /// The style spec was invalid (for example `fg:none`).
    Invalid,
    /// Raw ANSI escape sequence that should update the tracked style.
    AnsiEscape,
}

#[derive(Clone, Debug)]
struct StyleSpec {
    bold: bool,
    italic: bool,
    underline: bool,
    fg: ColorDirective,
    bg: ColorDirective,
}

#[derive(Clone, Copy, Debug, Default)]
struct AppliedStyle {
    bold: bool,
    italic: bool,
    underline: bool,
    fg: Option<ColorValue>,
    bg: Option<ColorValue>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ColorValue {
    NamedNormal(u8),
    NamedBright(u8),
    Index(u8),
    Rgb(u8, u8, u8),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ColorDirective {
    Unspecified,
    Reset,
    Set(ColorValue),
    PrevFg,
    PrevBg,
}

enum ParseOutcome {
    None,
    Invalid,
    Spec(StyleSpec),
}

enum Channel {
    Foreground,
    Background,
}

fn push_plain_segment(segments: &mut Vec<Segment>, text: &str) {
    if text.is_empty() {
        return;
    }
    if text.starts_with("\u{1b}[") && text.ends_with('m') {
        segments.push(Segment {
            text: text.to_string(),
            style: SegmentStyle::AnsiEscape,
        });
        return;
    }
    match segments.last_mut() {
        Some(Segment {
            style: SegmentStyle::None,
            text: existing,
        }) => existing.push_str(text),
        _ => segments.push(Segment {
            text: text.to_string(),
            style: SegmentStyle::None,
        }),
    }
}

impl StyleSpec {
    fn parse(spec: &str) -> ParseOutcome {
        let trimmed = spec.trim();
        if trimmed.is_empty() {
            return ParseOutcome::None;
        }

        let mut style = StyleSpec {
            bold: false,
            italic: false,
            underline: false,
            fg: ColorDirective::Unspecified,
            bg: ColorDirective::Unspecified,
        };
        let mut seen_any = false;

        for raw in trimmed.split_whitespace() {
            let token = raw.to_lowercase();
            match token.as_str() {
                "bold" => {
                    style.bold = true;
                    seen_any = true;
                    continue;
                }
                "italic" => {
                    style.italic = true;
                    seen_any = true;
                    continue;
                }
                "underline" => {
                    style.underline = true;
                    seen_any = true;
                    continue;
                }
                "prev_fg" => {
                    style.fg = ColorDirective::PrevFg;
                    seen_any = true;
                    continue;
                }
                "prev_bg" => {
                    style.fg = ColorDirective::PrevBg;
                    seen_any = true;
                    continue;
                }
                _ => {}
            }

            if let Some(rest) = token.strip_prefix("fg:") {
                if rest == "none" {
                    return ParseOutcome::Invalid;
                }
                style.fg = parse_color_directive(rest, true);
                if matches!(style.fg, ColorDirective::Unspecified) {
                    continue;
                }
                seen_any = true;
                continue;
            }
            if let Some(rest) = token.strip_prefix("bg:") {
                style.bg = parse_color_directive(rest, false);
                if matches!(style.bg, ColorDirective::Unspecified) {
                    continue;
                }
                seen_any = true;
                continue;
            }

            // Bare color tokens are treated as foreground colors.
            if let Some(value) = parse_color_value(&token) {
                style.fg = ColorDirective::Set(value);
                seen_any = true;
            }
        }

        if !seen_any {
            ParseOutcome::None
        } else {
            ParseOutcome::Spec(style)
        }
    }

    fn apply(
        &self,
        prev: Option<&AppliedStyle>,
        truecolor: bool,
    ) -> (AppliedStyle, Option<String>, bool) {
        let mut current = prev.copied().unwrap_or_default();
        let mut codes: Vec<String> = Vec::new();

        if self.bold {
            codes.push("1".to_string());
            current.bold = true;
        }
        if self.italic {
            codes.push("3".to_string());
            current.italic = true;
        }
        if self.underline {
            codes.push("4".to_string());
            current.underline = true;
        }

        let (fg_value, fg_codes) =
            apply_color_directive(self.fg, prev, current.fg, Channel::Foreground, truecolor);
        if let Some(component) = fg_codes {
            codes.push(component.clone());
            current.fg = fg_value;
        }

        let (bg_value, bg_codes) =
            apply_color_directive(self.bg, prev, current.bg, Channel::Background, truecolor);
        if let Some(component) = bg_codes {
            codes.push(component.clone());
            current.bg = bg_value;
        }

        let active = current.is_active();
        let sgr = if codes.is_empty() {
            None
        } else {
            Some(format!("\x1b[{}m", codes.join(";")))
        };
        (current, sgr, active)
    }
}

impl AppliedStyle {
    fn is_active(&self) -> bool {
        self.bold || self.italic || self.underline || self.fg.is_some() || self.bg.is_some()
    }
}

fn render_segments(segments: &[Segment], truecolor: bool) -> String {
    let mut out = String::new();
    let mut prev_style = AppliedStyle::default();
    let mut style_active = false;

    for segment in segments {
        match &segment.style {
            SegmentStyle::None => {
                out.push_str(&segment.text);
            }
            SegmentStyle::Invalid => {
                if style_active {
                    out.push_str("\x1b[0m");
                    prev_style = AppliedStyle::default();
                    style_active = false;
                }
                out.push_str(&segment.text);
            }
            SegmentStyle::AnsiEscape => {
                out.push_str(&segment.text);
                if absorb_ansi_sequence(&segment.text, &mut prev_style) {
                    style_active = prev_style.is_active();
                }
            }
            SegmentStyle::Explicit(spec) => {
                let (applied, sgr, active) = spec.apply(
                    if style_active {
                        Some(&prev_style)
                    } else {
                        None
                    },
                    truecolor,
                );
                if let Some(code) = sgr {
                    out.push_str(&code);
                }
                out.push_str(&segment.text);
                prev_style = applied;
                style_active = active;
            }
        }
    }

    out
}

fn apply_color_directive(
    directive: ColorDirective,
    prev: Option<&AppliedStyle>,
    current_value: Option<ColorValue>,
    channel: Channel,
    truecolor: bool,
) -> (Option<ColorValue>, Option<String>) {
    match directive {
        ColorDirective::Unspecified => (current_value, None),
        ColorDirective::Reset => (
            None,
            Some(match channel {
                Channel::Foreground => "39".to_string(),
                Channel::Background => "49".to_string(),
            }),
        ),
        ColorDirective::Set(value) => {
            let (component, stored) = color_to_sgr(value, channel, truecolor);
            (Some(stored), Some(component))
        }
        ColorDirective::PrevFg => prev
            .and_then(|p| p.fg)
            .map(|value| color_to_sgr(value, channel, truecolor))
            .map(|(component, stored)| (Some(stored), Some(component)))
            .unwrap_or((current_value, None)),
        ColorDirective::PrevBg => prev
            .and_then(|p| p.bg)
            .map(|value| color_to_sgr(value, channel, truecolor))
            .map(|(component, stored)| (Some(stored), Some(component)))
            .unwrap_or((current_value, None)),
    }
}

fn absorb_ansi_sequence(seq: &str, style: &mut AppliedStyle) -> bool {
    if !seq.starts_with("\u{1b}[") || !seq.ends_with('m') {
        return false;
    }
    let inner = &seq[2..seq.len() - 1];
    if inner.is_empty() {
        return false;
    }
    let mut parts = inner.split(';').peekable();
    while let Some(token) = parts.next() {
        if token.is_empty() {
            continue;
        }
        match token {
            "0" => {
                *style = AppliedStyle::default();
            }
            "1" => style.bold = true,
            "22" => style.bold = false,
            "3" => style.italic = true,
            "23" => style.italic = false,
            "4" => style.underline = true,
            "24" => style.underline = false,
            "39" => style.fg = None,
            "49" => style.bg = None,
            "38" => {
                if let Some(next) = parts.next() {
                    match next {
                        "2" => {
                            let r = parts.next().and_then(|v| v.parse::<u8>().ok());
                            let g = parts.next().and_then(|v| v.parse::<u8>().ok());
                            let b = parts.next().and_then(|v| v.parse::<u8>().ok());
                            if let (Some(r), Some(g), Some(b)) = (r, g, b) {
                                style.fg = Some(ColorValue::Rgb(r, g, b));
                            }
                        }
                        "5" => {
                            if let Some(idx) = parts.next().and_then(|v| v.parse::<u16>().ok()) {
                                if idx <= 255 {
                                    style.fg = Some(ColorValue::Index(idx as u8));
                                }
                            }
                        }
                        other => {
                            if let Ok(code) = other.parse::<i16>() {
                                apply_simple_color(code, Channel::Foreground, style);
                            }
                        }
                    }
                }
            }
            "48" => {
                if let Some(next) = parts.next() {
                    match next {
                        "2" => {
                            let r = parts.next().and_then(|v| v.parse::<u8>().ok());
                            let g = parts.next().and_then(|v| v.parse::<u8>().ok());
                            let b = parts.next().and_then(|v| v.parse::<u8>().ok());
                            if let (Some(r), Some(g), Some(b)) = (r, g, b) {
                                style.bg = Some(ColorValue::Rgb(r, g, b));
                            }
                        }
                        "5" => {
                            if let Some(idx) = parts.next().and_then(|v| v.parse::<u16>().ok()) {
                                if idx <= 255 {
                                    style.bg = Some(ColorValue::Index(idx as u8));
                                }
                            }
                        }
                        other => {
                            if let Ok(code) = other.parse::<i16>() {
                                apply_simple_color(code, Channel::Background, style);
                            }
                        }
                    }
                }
            }
            other => {
                if let Ok(code) = other.parse::<i16>() {
                    apply_simple_color(code, Channel::Foreground, style);
                }
            }
        }
    }
    true
}

fn apply_simple_color(code: i16, channel: Channel, style: &mut AppliedStyle) {
    match channel {
        Channel::Foreground => match code {
            30..=37 => style.fg = Some(ColorValue::NamedNormal((code - 30) as u8)),
            90..=97 => style.fg = Some(ColorValue::NamedBright((code - 90) as u8)),
            _ => {}
        },
        Channel::Background => match code {
            40..=47 => style.bg = Some(ColorValue::NamedNormal((code - 40) as u8)),
            100..=107 => style.bg = Some(ColorValue::NamedBright((code - 100) as u8)),
            _ => {}
        },
    }
}

fn parse_color_directive(token: &str, is_foreground: bool) -> ColorDirective {
    match token {
        "prev_fg" => ColorDirective::PrevFg,
        "prev_bg" => ColorDirective::PrevBg,
        "none" if !is_foreground => ColorDirective::Reset,
        _ => parse_color_value(token)
            .map(ColorDirective::Set)
            .unwrap_or(ColorDirective::Unspecified),
    }
}

fn parse_color_value(token: &str) -> Option<ColorValue> {
    if let Some(hex) = token.strip_prefix('#') {
        if hex.len() == 6 {
            let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
            let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
            let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
            return Some(ColorValue::Rgb(r, g, b));
        }
    }
    if token.chars().all(|c| c.is_ascii_digit()) {
        if let Ok(num) = token.parse::<u16>() {
            if num <= 255 {
                return Some(ColorValue::Index(num as u8));
            }
        }
    }
    if let Some(rest) = token.strip_prefix("bright-") {
        if let Some(idx) = parse_named(rest) {
            return Some(ColorValue::NamedBright(idx));
        }
    }
    parse_named(token).map(ColorValue::NamedNormal)
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

fn color_to_sgr(color: ColorValue, channel: Channel, truecolor: bool) -> (String, ColorValue) {
    match color {
        ColorValue::NamedNormal(idx) => (
            format!(
                "{}",
                match channel {
                    Channel::Foreground => 30 + idx,
                    Channel::Background => 40 + idx,
                }
            ),
            ColorValue::NamedNormal(idx),
        ),
        ColorValue::NamedBright(idx) => (
            format!(
                "{}",
                match channel {
                    Channel::Foreground => 90 + idx,
                    Channel::Background => 100 + idx,
                }
            ),
            ColorValue::NamedBright(idx),
        ),
        ColorValue::Index(n) => (
            format!(
                "{};5;{}",
                match channel {
                    Channel::Foreground => "38",
                    Channel::Background => "48",
                },
                n
            ),
            ColorValue::Index(n),
        ),
        ColorValue::Rgb(r, g, b) => {
            if truecolor {
                (
                    format!(
                        "{};2;{};{};{}",
                        match channel {
                            Channel::Foreground => "38",
                            Channel::Background => "48",
                        },
                        r,
                        g,
                        b
                    ),
                    ColorValue::Rgb(r, g, b),
                )
            } else {
                let idx = rgb_to_ansi256(r, g, b);
                (
                    format!(
                        "{};5;{}",
                        match channel {
                            Channel::Foreground => "38",
                            Channel::Background => "48",
                        },
                        idx
                    ),
                    ColorValue::Index(idx),
                )
            }
        }
    }
}

fn supports_truecolor() -> bool {
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

fn rgb_to_ansi256(r: u8, g: u8, b: u8) -> u8 {
    let rg = r as i32 - g as i32;
    let rb = r as i32 - b as i32;
    let gb = g as i32 - b as i32;
    let is_grayish = rg.abs() < 10 && rb.abs() < 10 && gb.abs() < 10;
    if is_grayish {
        let gray = ((r as u16 + g as u16 + b as u16) / 3) as u8;
        if gray < 8 {
            return 16;
        }
        if gray > 238 {
            return 231;
        }
        return 232 + ((gray as u16 - 8) / 10) as u8;
    }
    let to_6 = |v: u8| -> u8 { ((v as u16 * 5 + 127) / 255) as u8 };
    let r6 = to_6(r);
    let g6 = to_6(g);
    let b6 = to_6(b);
    16 + 36 * r6 + 6 * g6 + b6
}

// --- テスト --------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn collect_tokens<'a>(
        map: &'a [(&'a str, &'a str)],
    ) -> std::collections::HashMap<&'a str, String> {
        let mut tokens = std::collections::HashMap::new();
        for (k, v) in map {
            tokens.insert(*k, (*v).to_string());
        }
        tokens
    }

    fn strip_reset_once(s: &str) -> (&str, bool) {
        if let Some(pos) = s.rfind("\u{1b}[0m") {
            (&s[..pos], true)
        } else {
            (s, false)
        }
    }

    #[test]
    fn background_persists_until_explicit_change() {
        let tokens = collect_tokens(&[("text", "BAR")]);
        let rendered = render_with_style_template("[FOO](bg:#112233) $text", &tokens, "");
        let (without_final_reset, has_final_reset) = strip_reset_once(&rendered);
        assert!(has_final_reset, "final reset missing");
        assert!(without_final_reset.contains("FOO"));
        assert!(without_final_reset.contains("BAR"));
        assert_eq!(without_final_reset.matches("48;2;17;34;51").count(), 1);
        let before_bar = &without_final_reset[..without_final_reset.find("BAR").unwrap()];
        assert!(!before_bar.contains("\u{1b}[0m"));
    }

    #[test]
    fn prev_fg_uses_previous_background_color() {
        let rendered = render_with_style_template(
            "[A](bg:#111213)[B](fg:prev_bg bg:#202122)",
            &std::collections::HashMap::new(),
            "",
        );
        let (without_final_reset, _) = strip_reset_once(&rendered);
        assert!(without_final_reset.contains("48;2;17;18;19"));
        assert!(without_final_reset.contains("38;2;17;18;19"));
        assert!(without_final_reset.contains("48;2;32;33;34"));
    }

    #[test]
    fn bg_none_resets_background_for_following_text() {
        let rendered = render_with_style_template(
            "[A](bg:#010203)[B](bg:none)C",
            &std::collections::HashMap::new(),
            "",
        );
        let (without_final_reset, _) = strip_reset_once(&rendered);
        assert!(without_final_reset.contains("48;2;1;2;3"));
        assert_eq!(without_final_reset.matches("\u{1b}[49m").count(), 1);
        let after_reset = without_final_reset
            .split("\u{1b}[49m")
            .last()
            .expect("split produced at least one part");
        assert!(!after_reset.contains("48;2;"));
    }

    #[test]
    fn fg_none_resets_foreground_to_default() {
        let rendered = render_with_style_template(
            "[X](bg:#112233)[Y](fg:none)",
            &std::collections::HashMap::new(),
            "",
        );
        let (without_final_reset, _) = strip_reset_once(&rendered);
        assert!(without_final_reset.contains("48;2;17;34;51"));
        // fg:none triggers an intermediate reset to default
        assert!(without_final_reset.contains("\u{1b}[0m"));
    }

    #[test]
    fn plain_text_inherits_prior_style() {
        let rendered =
            render_with_style_template("[A](fg:#AA5500)B", &std::collections::HashMap::new(), "");
        let (without_final_reset, _) = strip_reset_once(&rendered);
        assert_eq!(without_final_reset.matches("38;2;170;85;0").count(), 1);
        assert!(without_final_reset.contains("A"));
        assert!(without_final_reset.contains("B"));
    }
}
