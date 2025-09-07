# Preset: Tokyo Night (foreground-only)

This preset approximates Starship’s “Tokyo Night” using foreground-only colors and simple character separators. It targets dark terminals with cool-toned accents and avoids backgrounds/truecolor for maximum portability.

## Key decisions
- Foreground-only styling: no backgrounds, no truecolor required.
- Subtle separators using `` in gray;
- Cool tones (cyan/blue/magenta) for a Tokyo Night feel on dark themes.

## Configuration (~/.config/claude-code-statusline.toml)

```toml
# Characters-only, foreground-only approximation
format = """
[░▒▓](#a3aed2)\
[ ](bg:#769ff0 fg:#a3aed2)\
$directory\
[ ](fg:#769ff0 bg:#394260)\
$git_branch\
$git_status\
[ ](fg:#394260 bg:#212736)\
$claude_model\
[ ](fg:#212736)\

[directory]
style = "fg:#e3e5e5 bg:#769ff0"
format = "[ $path ]($style)"
truncation_length = 3
truncation_symbol = "…/"

[git_branch]
symbol = ""
style = "bg:#394260"

[git_status]
style = "bg:#394260"

[claude_model]
style = "bg:#212736"
```

## Notes
- This preset intentionally avoids backgrounds to remain readable and consistent across terminals that don’t support truecolor or custom palettes.
- Outside a Git repository, `git_branch` and `git_status` hide automatically.

## References
- Starship Preset (Tokyo Night): https://starship.rs/presets/tokyo-night
