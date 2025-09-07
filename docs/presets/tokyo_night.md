# Preset: Tokyo Night

This preset approximates Starship’s “Tokyo Night” with cool, dark-friendly tones. The core supports both foreground and background colors, including 24-bit hex.

## Key decisions
- Use truecolor backgrounds with Powerline-style bridges for a cohesive look.
- Bridge arrows `` specify `fg:<prev-bg> bg:<next-bg>` for seamless transitions.
- Provide a simple `|` fallback when Powerline glyphs are unavailable.

## Configuration (~/.config/claude-code-statusline.toml)

```toml
format = """
[░▒▓](#a3aed2)\
[ ](bg:#769ff0 fg:#a3aed2)\
$directory\
[ ](fg:#769ff0 bg:#394260)\
$git_branch$git_status\
[ ](fg:#394260 bg:#212736)\
$claude_model\
[ ](fg:#212736)\
"""

[directory]
style = "fg:#e3e5e5 bg:#769ff0"
format = "[ $path ]($style)"
truncation_length = 3
truncation_symbol = "…/"

[git_branch]
style = "bg:#394260"
symbol = ""
format = "[$symbol $branch]($style)"

[git_status]
style = "bg:#394260"
format = "[$all_status$ahead_behind ]($style)"

[claude_model]
style = "bg:#212736"
format = "[$model]($style)"
```

## Notes
- Outside a Git repository, `git_branch` and `git_status` hide automatically.
- If your terminal doesn’t support 24‑bit color, replace hex with nearest 8‑bit indexes or switch to a foreground‑only style.

## References
- Starship Preset (Tokyo Night): https://starship.rs/presets/tokyo-night
