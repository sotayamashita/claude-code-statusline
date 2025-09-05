# Preset: Pure Prompt (single-line, no linebreak)

This preset emulates Starship’s "Pure Prompt (no linebreak)" — minimal, compact, and strictly single-line.
It focuses on succinct directory + git context, with an optional model indicator at the end.

## Goals

- Single-line output with no trailing newline
- Minimal visual noise; concise separators and symbols
- Uses Beacon’s built-in style engine only (bold + 8 colors)

## TOML Snippet (~/.config/beacon.toml)

```toml
format = "$directory $git_branch $git_status $claude_model"

[directory]
style = "bold blue"
truncation_length = 3
truncate_to_repo = true

[git_branch]
style = "bold green"
# symbol = ""   # if your font supports it; default is "🌿"

[git_status]
style = "bold red"

[claude_model]
style = "bold yellow"
```

Notes:
- Beacon prints without a trailing newline by design (uses `print!`, not `println!`).
- Keep spacing tight; avoid decorative padding to preserve the “pure” feel.
- Use only supported style tokens: `bold/italic/underline` and 8 colors.
- When outside a git repo, `git_branch` / `git_status` auto-hide safely.

## Example (plain, colors omitted)

```
project/src 🌿 main (+1!1?1 ⇡1) <Sonnet>
```

Depending on repo state, `(+1!1?1 ⇡1)` appears only when there are changes or upstream divergence.

## How to Use

- Put the TOML above into `~/.config/beacon.toml`.
- Verify enabled modules: `beacon modules --enabled`.
- Try a run: `echo '{"cwd":"/tmp","session_id":"s","model":{"id":"claude-3.5-sonnet","display_name":"Sonnet"}}' | beacon`.

## References

- Starship Presets: https://starship.rs/presets/pure-preset
- Pure: https://github.com/sindresorhus/pure
