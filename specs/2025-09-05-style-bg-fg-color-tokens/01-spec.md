# Spec: Style Engine `fg:`/`bg:` with 8-color, 256-color, and Hex

- Date: 2025-09-05
- Status: Draft
- Owner: claude-code-statusline Core

## Summary

Extend the style engine to support foreground (`fg:`) and background (`bg:`) color tokens alongside existing text modifiers (`bold`, `italic`, `underline`). Colors accept:

- Named 8-color set (with optional `bright-` prefix)
- 8-bit ANSI index (`0..=255`)
- 24-bit Hex (`#RRGGBB`)

The feature must remain backward-compatible with existing style strings like `"bold yellow"` (treated as foreground). This enables presets like “Pastel Powerline” to require explicit color tokens and, optionally, stage a future backgrounded variant without core changes.

## Goals

- Add `fg:` and `bg:` parsing to `apply_style`.
- Support 8 named colors (and `bright-` variants), 8-bit indices, and `#RRGGBB`.
- Keep old forms working: bare color tokens map to foreground.
- Preserve the “ignore unknown tokens” behavior, while improving warnings.
- Keep performance characteristics roughly the same.

## Non‑Goals (initial scope)

- Palette names, `prev_fg`/`prev_bg` inheritance.
- Additional text modifiers beyond current (`bold`, `italic`, `underline`).
- Alternate color syntaxes (e.g., `rgb(r,g,b)` strings).

## Motivation & Context

- Issue #18 requires a preset that uses character separators with pastel-like colors. The project now mandates explicit color tokens. Supporting `fg:`/`bg:` with 8/256/Hex enables precise, portable presets without expanding higher-level format parsing.

## Terminology

- Style string: space-separated tokens applied to the bracket segment `[$content]($style)` by the style engine.
- Text modifiers: `bold`, `italic`, `underline`.
- Color tokens: `fg:<color-spec>`, `bg:<color-spec>`, or bare color name as foreground.

## Syntax

- Tokenization: split on ASCII whitespace; order-insensitive; last token wins per channel (fg, bg, each modifier).
- Color token forms:
  - Named: `black|red|green|yellow|blue|magenta|cyan|white`
  - Bright named: `bright-black|bright-red|...|bright-white`
  - 8-bit index: integer `0..=255`
  - Hex: `#RRGGBB` (case-insensitive hex digits)
  - With prefixes: `fg:<spec>` or `bg:<spec>`
  - Bare color: `<spec>` is treated as `fg:<spec>`
- Resets:
  - `fg:none` / `bg:none` — omit setting that channel (use terminal default)
  - Unknown tokens — ignored (no error), but surfaced via warnings collection

## Semantics

- Text modifiers map to ANSI SGR: `bold=1`, `italic=3`, `underline=4`.
- Color mapping priority per channel (apply last occurrence):
  1) Named (normal) → FG: 30–37, BG: 40–47
  2) Named (bright) → FG: 90–97, BG: 100–107
  3) 8-bit index `n` → FG: `38;5;n`, BG: `48;5;n`
  4) Hex `#RRGGBB` → FG: `38;2;R;G;B`, BG: `48;2;R;G;B`
  5) `none` → omit that channel entirely
- Composition: emit a single `ESC[` SGR with `;`-separated codes, wrap text, then `ESC[0m`.

### Examples

- `"bold fg:green bg:black"` → `ESC[1;32;40m...ESC[0m`
- `"bright-yellow"` (bare) → foreground bright yellow: `ESC[93m...ESC[0m`
- `"fg:196 bg:238"` → `ESC[38;5;196;48;5;238m...ESC[0m`
- `"fg:#bf5700"` → `ESC[38;2;191;87;0m...ESC[0m`
- `"bg:none unknown-token italic"` → ignores `unknown-token`, applies italic; no bg set.

## Detailed Spec

### Token Parsing

1) For each whitespace-separated token `t`:
   - If `t` is one of `bold|italic|underline` → set modifier flag true.
   - Else if `t` starts with `fg:` → parse color spec after the prefix; set foreground.
   - Else if `t` starts with `bg:` → parse color spec after the prefix; set background.
   - Else → treat as bare color → parse as foreground.
2) Unknown or unparsable color spec is ignored for rendering; warnings are recorded.
3) Last token for the same channel (fg/bg) overrides previous values.

### Color Spec Parser

Accepts (in this order of detection):

- `#RRGGBB`: parse hex into (R,G,B) [0..=255] each → TrueColor.
- Decimal `0..=255`: parse as 8-bit color index.
- Named:
  - `bright-<name>`: map to bright 8-color set.
  - `<name>`: map to normal 8-color set.

Named color lookups:

```
normal: black=30/40, red=31/41, green=32/42, yellow=33/43, blue=34/44,
        magenta=35/45, cyan=36/46, white=37/47
bright: black=90/100, red=91/101, green=92/102, yellow=93/103, blue=94/104,
        magenta=95/105, cyan=96/106, white=97/107
```

### Rendering

- Build SGR segments in this order (order of codes is not semantically significant):
  - Modifiers: `1` (bold), `3` (italic), `4` (underline)
  - Foreground: one of `30..37`, `90..97`, `38;5;n`, or `38;2;R;G;B`
  - Background: one of `40..47`, `100..107`, `48;5;n`, or `48;2;R;G;B`
- If no codes are collected, return the text unchanged (no wrapping).
- Otherwise: `format!("\x1b[{}m{}\x1b[0m", codes.join(";"), text)`

## Validation & Warnings

- `Config::collect_warnings()` must understand tokens beginning with `fg:` / `bg:`.
- For each style string encountered, split into tokens and validate colors:
  - Valid: bare named/bright, `fg:`/`bg:` with named/bright, `0..=255`, or `#RRGGBB`.
  - Invalid: any other `fg:`/`bg:` suffix (e.g., `fg:xxx`, `bg:#12AB`, `fg:300`).
  - Unknown tokens (non-color words not in modifiers) produce the existing warning via `warn_unknown_style_token(module, token)`.
- Rationale: warnings remain non-fatal and informative; behavior remains “best-effort”.

## Backward Compatibility

- Existing style strings like `"bold yellow"` continue to work (yellow as foreground).
- No change to default configuration values or module formats.
- Unknown tokens are still ignored at render time; only surfaced as warnings.

## Testing

Add unit tests in `crates/claude-code-statusline-core/src/style.rs`:

1) Named fg/bg:
   - `apply_style("X", "bold fg:green bg:black")` contains `1;32;40` and wraps `X`.
2) Bright named:
   - `apply_style("X", "bright-yellow bg:bright-blue")` contains `93` and `104` (order flexible).
3) 8-bit indexes:
   - `apply_style("X", "fg:196 bg:238")` contains `38;5;196` and `48;5;238`.
4) Hex truecolor:
   - `apply_style("X", "fg:#bf5700 bg:#003366")` contains `38;2;191;87;0` and `48;2;0;51;102`.
5) Bare color equivalence:
   - `apply_style("X", "yellow")` behaves like `fg:yellow`.
6) Unknown tokens stability:
   - `apply_style("X", "bold sparkle fg:green foo")` applies bold and green; ignores others.
7) None handling:
   - `apply_style("X", "fg:none italic")` sets italic only; no `38;` code present.

Additionally, update `types/config.rs` validation tests to ensure no warnings for valid `fg:`/`bg:` and warnings for invalid ones.

## Performance Considerations

- Token parsing is simple string operations; minimal overhead compared to I/O-bound modules.
- Avoid heap churn by reusing small vectors; keep allocations bounded.

## Rollout Plan

1) Implement parsing/mapping in `style.rs`.
2) Update `collect_warnings` to validate `fg:`/`bg:` color specs.
3) Add/adjust unit tests (style + config warnings).
4) Documentation updates:
   - `docs/configuration.md`: document `fg:`/`bg:` syntax and examples.
   - `docs/presets/pastel_powerline.md`: include both characters-only and background-optional variants.
5) Keep default config unchanged; presets require explicit fg tokens.

## Preset Notes (Pastel Powerline: characters-only)

- Global format: `"$directory $git_branch $git_status $claude_model"`
- Example module styles using explicit `fg:`:

```
[directory]
style = "bold fg:cyan"
format = "[$path ]($style)"

[git_branch]
style = "bold fg:green"
format = "[$symbol $branch ]($style)"

[git_status]
style = "bold fg:yellow"
format = "[[$all_status$ahead_behind] ]($style)"

[claude_model]
style = "bold fg:magenta"
format = "[$symbol$model]($style)"
```

## Future Work

- Support additional modifiers: `dimmed`, `inverted`, `strikethrough`, `hidden`, `blink`.
- `prev_fg` / `prev_bg` inheritance between adjacent segments.
- Named palettes resolved from config tables.
