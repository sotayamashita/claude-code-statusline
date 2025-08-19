use clap::Parser;

// Import types from the types module
mod types;
use types::ClaudeInput;

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
