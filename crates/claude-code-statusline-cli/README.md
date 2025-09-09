# claude-code-statusline-cli

CLI for claude-code-statusline. It reads Claude Code JSON on stdin and prints an ANSI-formatted status line to stdout.

## Install

```
# From crates.io (after publish)
cargo install claude-code-statusline-cli
# Installs a binary named `claude-code-statusline`
```

## Usage

```
echo '{"cwd":"/tmp","session_id":"abc","model":{"id":"claude-opus","display_name":"Opus"}}' \
  | claude-code-statusline
```

## License

MIT
