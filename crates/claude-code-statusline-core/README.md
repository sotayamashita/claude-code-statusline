# claude-code-statusline-core

Core library for claude-code-statusline. It exposes the public API, types, and statusline modules used by the CLI.

- Public API highlights: `Engine`, `Config`, `parse_claude_input`, `Context`, `CoreError`
- Feature flags:
  - `git`: enables Git-powered modules (`git_branch`, `git_status`)
  - `parallel`: enables Rayon-based parallel rendering (optional)

## Example

```rust
use claude_code_statusline_core::{engine::Engine, Config, parse_claude_input};

let json = r#"{\n  \"session_id\": \"abc\",\n  \"cwd\": \"/tmp\",\n  \"model\": {\n    \"id\": \"claude-3.5-sonnet\",\n    \"display_name\": \"Sonnet\"\n  }\n}"#;
let input = parse_claude_input(json)?;
let config = Config::default();
let engine = Engine::new(&config);
let output = engine.render(&input)?;
println!("{}", output);
# Ok::<(), Box<dyn std::error::Error>>(())
```

## License

MIT
