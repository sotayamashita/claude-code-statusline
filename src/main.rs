use clap::Parser;
use serde::Deserialize;

// Claude Code JSON input structures
// Reference: https://docs.anthropic.com/en/docs/claude-code/statusline#json-input-structure
// Complete structure from official documentation
#[derive(Debug, Deserialize)]
struct ClaudeInput {
    hook_event_name: String,
    session_id: String,
    transcript_path: String,
    cwd: String,
    model: ModelInfo,
    workspace: WorkspaceInfo,
    version: String,
    output_style: OutputStyle,
}

// Model information from Claude Code
#[derive(Debug, Deserialize)]
struct ModelInfo {
    id: String,
    display_name: String,
}

// Workspace paths information
#[derive(Debug, Deserialize)]
struct WorkspaceInfo {
    current_dir: String,
    project_dir: String,
}

// Output style configuration
#[derive(Debug, Deserialize)]
struct OutputStyle {
    name: String,
}

#[derive(Parser)]
#[command(name = env!("CARGO_PKG_NAME"))]
#[command(version = env!("CARGO_PKG_VERSION"))]
#[command(about = env!("CARGO_PKG_DESCRIPTION"))]
struct Cli {
    // 後でサブコマンドを追加予定
}

fn main() {
    let _cli = Cli::parse();
    println!("Hello, Beacon!");
}
