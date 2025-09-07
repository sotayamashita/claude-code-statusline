# Preset: Pure Prompt (single line)

This preset mirrors Starship’s “Pure (no linebreak)”: minimal, compact, and strictly single-line. It focuses on concise directory and Git context, with an optional model indicator at the end.

## Configuration (~/.config/claude-code-statusline.toml)

```toml
format = "$directory $git_branch $git_status $claude_model"

[directory]
style = "bold blue"
truncation_length = 3
truncate_to_repo = true

[git_branch]
style = "bold green"
symbol = ""

[git_status]
style = "bold red"

[claude_model]
style = "bold yellow"
```

## Notes
- The CLI prints without a trailing newline by design (`print!` not `println!`).
- Keep spacing tight and avoid decorative padding to preserve the “pure” feel.
- Supported style tokens: `bold`, `italic`, `underline`, plus colors as:
  - named and bright (e.g., `blue`, `bright-blue`)
  - 8-bit indexes `0..=255` (e.g., `196`)
  - 24-bit hex (e.g., `#RRGGBB`)
  - `fg:`/`bg:` prefixes and `none`
  This preset uses simple named foreground colors for portability.
- Outside a Git repo, `git_branch` and `git_status` hide automatically.

## References
- Starship Pure preset: https://starship.rs/presets/pure-preset
