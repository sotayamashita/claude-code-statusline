# Beacon リファクタリングタスク（SPAC/2025-09-04）

参照: `specs/2025-09-04-refactoring/01-spac.md`

## 完了
- [x] ワークスペース化（新規作成）
  - ルートに `[workspace]` を追加し、メンバーを `crates/beacon-core`, `crates/beacon-cli` に分割。
  - `cargo build`/`cargo test` が通過する最小構成を確認。
- [x] コア実装の切り出し（既存流用・git mv）
  - `src/{config.rs, parser.rs, style.rs, timeout.rs}` → `crates/beacon-core/src/`
  - `src/types/**` → `crates/beacon-core/src/types/**`
  - `src/modules/**` → `crates/beacon-core/src/modules/**`
  - `src/{debug.rs, messages.rs, engine.rs}` → `crates/beacon-core/src/`
- [x] beacon-core の公開面整備（新規作成）
  - `crates/beacon-core/src/lib.rs` を作成し、`pub mod` 構成と `pub use {Engine, Config, parse_claude_input, Context}` を提供。
- [x] ルートクレートのシム化（新規作成/調整）
  - `src/lib.rs` から `beacon_core` を再エクスポートし、`beacon::...` パス互換を維持。
  - （中間フェーズ）互換用バイナリ `src/bin/beacon.rs` を一時導入（内部で `beacon_cli::run()` を呼び出し）。
- [x] CLI 土台の分離（既存流用・git mv＋新規作成）
  - `src/main.rs` → `crates/beacon-cli/src/main.rs` に移動。
  - `crates/beacon-cli/src/lib.rs` を新設し、`pub fn run() -> anyhow::Result<()>` に CLI フローを集約。
  - CLI は `beacon_core::{Config, debug, messages, parse_claude_input, Engine}` を使用。
- [x] ビルド/テスト通過（確認）
  - 既存の `engine_api` を含むテストスイートが通過。互換バイナリにより既存の `beacon` バイナリ名を維持。
- [x] Registry/ModuleFactory の導入（完了）
  - `modules/registry.rs` を追加し、`handle_module`/設定参照を内部で委譲。
  - 既存の `modules::handle_module` API は互換維持。
- [x] ConfigProvider/extra_modules（完了）
  - `Config.extra_modules`（serde flatten）で未知セクションを保持。
  - `ConfigProvider::module_table()`/`list_extra_modules()` を実装。
- [x] フィーチャーゲート（完了）
  - `beacon-core` の `[features]` に `git`（= `git2` optional）と `parallel`（= `rayon` optional）を追加。
  - Git系モジュールは feature でコンパイル制御。`beacon-cli` は `git` を有効化。
- [x] エラー/ログ方針の移行（段階完了）
  - CLI は `tracing` + `tracing-subscriber` に移行（stderr 出力）。既存 `eprintln!` も併用し E2E 互換維持。
  - コアに `thiserror` ベースの `CoreError` を導入。`DebugLogger` は `tracing` にもフォワード。
  - 注: コア全体の `thiserror` 置換はフェーズ2で継続。
- [x] CLI サブコマンド（完了）
  - `config --path|--default|--validate`, `modules --list|--enabled` を実装。
- [x] ルート純ワークスペース化（完了）
  - 最終状態: 互換バイナリ（`src/bin/beacon.rs`）を撤去し、バイナリ名を `beacon`（`beacon-cli`）に統一。
- [x] 追加テスト（完了）
  - `beacon-core` に `Engine/Registry/ConfigProvider` のユニットテストを追加。既存の CLI E2E は維持。
- [x] ドキュメント/整備（完了）
  - README/開発ドキュメントをワークスペース構成・Registry・実行コマンドに更新。
- [x] パフォーマンス最適化（初期対応）
  - `parallel` 有効時にモジュール並列レンダリング（Rayon）。
  - `criterion` による `engine_bench` を追加。
- [x] エラー型の全面移行（段階完了）
  - コア内の `anyhow` 依存を撤去し、`CoreError`（`thiserror`）へ統一。
  - `parser/config/engine/timeout/modules` が `CoreError` を返すように変更。
  - `CoreError` に `InvalidJson/ConfigRead/ConfigParse/MissingConfig/TaskPanic/...` を追加。
  - CLI は引き続き境界層で `anyhow::Result` を使用（表示とハンドリングは既存どおり）。
- [x] CLI E2E 強化（完了）
  - `tests/cli_subcommands.rs` を追加し、以下を検証:
    - `beacon config --path|--default|--validate`
    - `beacon modules --list|--enabled`
- [x] パフォーマンス検証（初期運用）
  - `make bench`（criterion）と `make bench-check` を追加。
  - `scripts/bench_check.py` により平均実行時間が 50ms 未満であることを自動判定。

---
最終更新: 2025-09-05
