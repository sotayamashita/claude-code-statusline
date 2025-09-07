# Preset: Tokyo Night

This preset approximates Starship’s “Tokyo Night” with cool, dark-friendly tones. The core supports both foreground and background colors, including 24-bit hex.

## Key decisions
- Use truecolor backgrounds with Powerline-style bridges for a cohesive look.
- Bridge arrows `` specify `fg:<prev-bg> bg:<next-bg>` for seamless transitions.
- Provide a simple `|` fallback when Powerline glyphs are unavailable.

## Configuration (~/.config/claude-code-statusline.toml)

```toml
# Background + truecolor variant
format = """
[ ](bg:#1f2335)\
$directory\
[ ](bg:#2a2f4a fg:#1f2335)\
$git_branch$git_status\
[ ](bg:#3b4261 fg:#2a2f4a)\
$claude_model\
[ ](fg:#3b4261)\
"""

[directory]
style = "fg:#c0caf5 bg:#1f2335"
format = "[ $path ]($style)"
truncation_length = 3
truncation_symbol = "…/"

[git_branch]
symbol = ""
style = "fg:#7aa2f7 bg:#2a2f4a"
format = "[$symbol $branch]($style)"

[git_status]
style = "fg:#9ece6a bg:#2a2f4a"
format = "[$all_status$ahead_behind ]($style)"

[claude_model]
style = "fg:#c0caf5 bg:#3b4261"
format = "[$model]($style)"
```

## Notes
- Outside a Git repository, `git_branch` and `git_status` hide automatically.
- If your terminal doesn’t support 24‑bit color, replace hex with nearest 8‑bit indexes or switch to a foreground‑only style.

## References
- Starship Preset (Tokyo Night): https://starship.rs/presets/tokyo-night
