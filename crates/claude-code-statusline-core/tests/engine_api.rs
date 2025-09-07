mod common;

use claude_code_statusline::{Config, parse_claude_input};

// TDD Red: Introduce public Engine API and verify basic rendering.
// This test will fail to compile/run until `claude_code_statusline::engine::Engine`
// is implemented per specs/2025-09-04-refactoring/01-spac.md.
#[test]
fn engine_renders_basic_status_line() {
    // Minimal JSON input
    let json = r#"{
        "session_id": "abc123",
        "cwd": "/tmp",
        "model": { "id": "claude-opus", "display_name": "Opus" },
        "workspace": { "current_dir": "/tmp", "project_dir": "/tmp" }
    }"#;

    let input = parse_claude_input(json).expect("valid input");
    let engine = claude_code_statusline::engine::Engine::new(Config::default());

    let out = engine.render(&input).expect("render ok");
    assert!(!out.is_empty());
    assert!(
        out.contains("Opus"),
        "output should contain model name: {out}"
    );
}
