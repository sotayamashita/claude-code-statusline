use clap::Parser;

#[derive(Parser)]
#[command(name = "beacon")]
#[command(version = "0.1.0")]
#[command(about = "A lightweight status line generator for Claude Code")]
struct Cli {
    // 後でサブコマンドを追加予定
}

fn main() {
    let _cli = Cli::parse();
    println!("Hello, Beacon!");
}
