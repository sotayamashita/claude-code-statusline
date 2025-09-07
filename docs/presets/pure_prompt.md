# Preset: Pure Prompt (single line)

This preset mirrors Starship’s “Pure (no linebreak)”: minimal, compact, and strictly single-line. It focuses on concise directory and Git context, with an optional model indicator at the end.

## Configuration (~/.config/beacon.toml)

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
- Beacon prints without a trailing newline by design (`print!` not `println!`).
- Keep spacing tight and avoid decorative padding to preserve the “pure” feel.
- Supported style tokens: `bold`, `italic`, `underline`, and 8 named colors.
- Outside a Git repo, `git_branch` and `git_status` hide automatically.

## References
- Starship Pure preset: https://starship.rs/presets/pure-preset
