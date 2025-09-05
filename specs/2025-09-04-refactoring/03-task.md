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
  - 互換用バイナリ `src/bin/beacon.rs` を追加（内部で `beacon_cli::run()` を呼び出し）。
- [x] CLI 土台の分離（既存流用・git mv＋新規作成）
  - `src/main.rs` → `crates/beacon-cli/src/main.rs` に移動。
  - `crates/beacon-cli/src/lib.rs` を新設し、`pub fn run() -> anyhow::Result<()>` に CLI フローを集約。
  - CLI は `beacon_core::{Config, debug, messages, parse_claude_input, Engine}` を使用。
- [x] ビルド/テスト通過（確認）
  - 既存の `engine_api` を含むテストスイートが通過。互換バイナリにより既存の `beacon` バイナリ名を維持。

## 未完了（次ステップ）
- [ ] Registry/ModuleFactory の導入（未着手）
  - `Registry`, `ModuleFactory` を追加し、`modules::handle_module` の静的 dispatch を段階的に移行。
- [ ] ConfigProvider/extra_modules（未着手）
  - 未知セクションを保持する `Config.extra_modules` と `ConfigProvider::module_table()` を実装。
- [ ] フィーチャーゲート（未着手）
  - `beacon-core` の `[features]` で `git`（= `git2` 有効）と `parallel`（= `rayon`）を定義。`beacon-cli` で `git` を有効化。
- [ ] エラー/ログ方針の移行（未着手）
  - コアを `thiserror` に移行、CLI は `anyhow` で文脈付与。ロギングは `tracing` + `tracing-subscriber` に置換。
- [ ] CLI サブコマンド（未着手）
  - `config --path|--default|--validate`, `modules --list|--enabled` 等を実装。
- [ ] ルート純ワークスペース化（未着手）
  - 互換バイナリ（`src/bin/beacon.rs`）を撤去し、最終的なバイナリ名を `beacon` に統一（`beacon-cli` 側で名称調整）。
- [ ] 追加テスト（未着手）
  - `beacon-core` 向けに `Engine/Registry/ConfigProvider` のユニットテストを追加。CLI の E2E も更新。
- [ ] ドキュメント/整備（未着手）
  - README/Docs のインストール手順とワークスペース構成を更新、hooks/CI の調整。
- [ ] パフォーマンス最適化（未着手・将来）
  - `parallel` 有効時にモジュール並列レンダリング、`criterion` による回帰計測。

---
最終更新: 2025-09-05
