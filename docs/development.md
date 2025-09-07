## Development Guide

claude-code-statusline の内部構造と、モジュール追加・テスト・開発ワークフローの指針をまとめます。

### プロジェクト構成

- コア: `crates/claude-code-statusline-core/src/`
  - `engine.rs`（レンダリングエンジン）/ `lib.rs`（公開面）
  - `config.rs` / `types/config.rs`（TOML 設定の型と読み込み/検証）
  - `types/claude.rs` / `types/context.rs`（入力/コンテキスト）
  - `modules/*`（各モジュール・`registry.rs`）
  - `parser.rs` / `style.rs` / `timeout.rs`
- CLI: `crates/claude-code-statusline-cli/`（stdin→stdout とサブコマンド）

### モジュール実装（追加手順）

1) `crates/claude-code-statusline-core/src/modules/<name>.rs` を作成し、`Module` を実装

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

2) レジストリ登録: `crates/claude-code-statusline-core/src/modules/registry.rs`

```rust
// 例: `struct MyModuleFactory;` を追加して `ModuleFactory` を実装
// `Registry::with_defaults()` に `reg.register_factory(MyModuleFactory);` を追加
```

3) 設定型の追加（必要に応じて）: `crates/claude-code-statusline-core/src/types/config.rs`

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

### ベンチマークと閾値チェック（パフォーマンス運用）

- ベンチ実行（criterion）

```
make bench    # crates/claude-code-statusline-core のベンチを実行
```

- 閾値チェック（既定: 平均 < 50ms）

```
make bench-check
# 実行後、target/criterion/engine_render_default/new/estimates.json の mean を読み取り判定
```

しきい値は `scripts/bench_check.py --threshold-ms <ms>` で変更可能です。

#### Feature flags（git）

- 既定の `make bench` は optional features（例: `git`）を無効のまま `claude-code-statusline-core` をベンチビルドします。
- Git 連携を含むベンチ/テストを実行したい場合は feature を明示的に有効化してください。

```
cargo bench -p claude-code-statusline-core --features git --no-run
cargo test  -p claude-code-statusline-core --features git
```

CLI バイナリ（`crates/claude-code-statusline-cli`）は `claude-code-statusline-core` を `features = ["git"]` で依存しているため、
通常の `cargo run -p claude-code-statusline-cli` やインストール済み `claude-code-statusline` 実行では Git 機能が有効です。

### 開発ワークフロー

- ブランチ戦略・コミット規約: Conventional Commits（例: `feat(modules): add git_status`）
- 事前チェック: `make install-hooks` で pre-commit に `fmt`/`clippy`/`test` を導入可能
- CI: GitHub Actions（バッジは README 冒頭）
