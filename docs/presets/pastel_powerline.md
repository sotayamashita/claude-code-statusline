# Preset: Pastel Powerline

This preset recreates Starship’s “Pastel Powerline” look: soft, pastel blocks connected by arrows. Each segment explicitly sets both foreground and background, and the arrow bridges use the previous segment’s background as the arrow foreground and the next segment’s background as the arrow background.

## Key decisions
- Avoid the left-edge curved separator ``. Start with a background-colored space instead to prevent anti‑aliasing halos in web-based renderers.
- Bridge arrows `` must always specify `fg:<prev-bg> bg:<next-bg>` to ensure seamless color.
- Use `fg:black` for text on light pastel backgrounds to keep contrast readable.

## Configuration (~/.config/beacon.toml)

```toml
format = """
[ ](#9A348E)\
$directory\
[ ](bg:#DA627D fg:#9A348E)\
$git_branch$git_status\
[ ](fg:#DA627D bg:#FCA17D)\
$claude_model\
[ ](fg:#FCA17D)\
"""

[directory]
style = "bg:#9A348E"
format = "[ $path]($style)"
truncation_length = 3
truncation_symbol = "…/"

[git_branch]
symbol = ""
style = "bg:#DA627D"
format = "[$symbol $branch]($style)"

[git_status]
style = "bg:#DA627D"
format = "[$all_status$ahead_behind ]($style)"

[claude_model]
style = "bg:#FCA17D"
format = "[$model]($style)"
```

## Why avoid the left-edge ``
- Web renderers (e.g., editor-integrated terminals) apply strong subpixel anti‑aliasing to curved glyphs. The glyph edge blends with the cell background, so a “foreground glyph” and a “painted background cell” can look slightly different even with the same color.
- At the left edge, the neighbor is often the terminal’s default background (theme), not your previous segment, making the halo more visible.
- The `` glyph lives in the Private Use Area and can vary across fonts, which hurts visual consistency.

## Notes
- The core does not yet implement “inherit previous colors automatically” between segments. Always set both `fg:` and `bg:` explicitly for blocks and bridge arrows.

## References
- Starship Pastel Powerline: https://starship.rs/presets/pastel-powerline
