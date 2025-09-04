## Development Guide

Beacon の内部構造と、モジュール追加・テスト・開発ワークフローの指針をまとめます。

### プロジェクト構成

- コア: `src/`
  - `main.rs`（CLI エントリ）/ `lib.rs`（共有）
  - `config.rs` / `types/config.rs`（TOML 設定の型と読み込み/検証）
  - `types/claude.rs`（入力 JSON の構造体）
  - `types/context.rs`（実行時コンテキスト。OnceLock による簡易キャッシュを保持）
  - `modules/*`（各モジュール。`Module` トレイトを実装）
  - `parser.rs`（`$module` を展開するフォーマッタ）
  - `style.rs`（簡易 ANSI レンダラ）
  - `timeout.rs`（処理のタイムアウトユーティリティ）

### モジュール実装（追加手順）

1) `src/modules/<name>.rs` を作成し、`Module` を実装

```rust
use super::{Module, ModuleConfig};
use crate::types::context::Context;

pub struct MyModule;
impl MyModule { pub fn from_context(_ctx: &Context) -> Self { Self } }

impl Module for MyModule {
    fn name(&self) -> &str { "my_module" }
    fn should_display(&self, _ctx: &Context, _cfg: &dyn ModuleConfig) -> bool { true }
    fn render(&self, _ctx: &Context, _cfg: &dyn ModuleConfig) -> String { "OK".into() }
}
```

2) ディスパッチャ登録: `src/modules/mod.rs`

```rust
mod my_module; // ファイルを追加
use my_module::MyModule;

pub fn handle_module(name: &str, context: &Context) -> Option<Box<dyn Module>> {
    match name {
        "my_module" => Some(Box::new(MyModule::from_context(context))),
        _ => /* 既存 */ super::handle_module(name, context),
    }
}
```

3) 設定型の追加（必要に応じて）: `src/types/config.rs`

- `struct MyModuleConfig { format, style, disabled, ... }`
- `impl Default for MyModuleConfig { ... }`
- `impl ModuleConfig for MyModuleConfig { ... }`
- `Config` にフィールドを追加し、`module_config_for()` でひけるようにする

4) レンダリングとスタイル

- テンプレート `[$text]($style)` を使う場合は `crate::style::render_with_style_template()` を利用
- スタイルは `$style` を渡すとモジュール既定の `style` を使用

### タイムアウトとキャッシュの規約

- すべてのモジュール呼び出しは `render_module_with_timeout()`（`modules/mod.rs`）でラップされます
  - `Config.command_timeout` の範囲: 50..=600000ms
  - タイムアウトした場合は `None`（＝非表示）として扱う
- 高コストな取得（Git リポジトリ、ディレクトリ走査）は `Context` の `OnceLock` を利用して同一実行内でメモ化

### テスト方針

- 単体テストは各モジュール・各ユーティリティ内に `#[cfg(test)]` で配置
- 共有ヘルパは `tests/common/` に配置し、`rstest` を活用
- 実行コマンド:

```
cargo test
cargo clippy -- -D warnings
cargo fmt
```

### 開発ワークフロー

- ブランチ戦略・コミット規約: Conventional Commits（例: `feat(modules): add git_status`）
- 事前チェック: `make install-hooks` で pre-commit に `fmt`/`clippy`/`test` を導入可能
- CI: GitHub Actions（バッジは README 冒頭）

