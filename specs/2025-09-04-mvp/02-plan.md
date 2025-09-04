# Beacon Roadmap (全体整理)

> 本ドキュメントは `docs/init/*`（spec/plan/todo/statusline/refactoring など）と現行コード（src/*）を突き合わせ、これまでの実装状況と今後の計画をひと目で把握できるようにまとめたロードマップです。チェックボックスは完了 `- [x]`／未完了 `- [ ]` を示します。

## サマリ
- [x] Phase 1: MVP（基本骨子の実装・テスト・動作確認まで完了）
- [ ] Phase 2: Core Features（Git/Session/ANSI/Validation 等）
- [ ] Phase 3: Polish（品質向上・キャッシュ・タイムアウト・ドキュメント）
- [ ] Phase 4: Advanced（並列化・カスタムエラー・ベンチマークなど）
- [x] リファクタリング（Phase 1に必要な範囲は実施済み、一部は採用見送り）
- [x] 不要機能の整理（Character module を削除済み）

---

## Phase 1: MVP（Week 1）
- [x] 基本的な CLI 構造（`clap` 導入・`--help`/`--version`）
- [x] JSON 入力の解析（`serde`/`serde_json`・全フィールド受け取り）
- [x] `Module` トレイト＋ディスパッチャ（`modules/mod.rs::handle_module`）
- [x] コアモジュール 2種（`directory`, `claude_model`）
- [x] フォーマットパーサ（`$directory $claude_model` の置換）
- [x] 単一行出力（改行なしの `print!`／stdout フラッシュ）
- [x] 失敗時フォールバック（JSON 解析失敗・空入力時の固定文言）
- [x] 設定ファイル（`~/.config/beacon.toml` の TOML 読込）
- [x] `Context` 構造体（実行時情報＋設定の集約）
- [x] デバッグロガー（`src/debug.rs`・stderr/ファイル出力の切替）
- [x] テスト（`config.rs`/`parser.rs`/各モジュールのユニットテスト）
- [x] プロジェクト構造整理（`src/types`, `src/modules`, `src/config.rs` 等）
- [x] Character module の削除（出力専用の性質上不要）

備考:
- ANSI カラーは Phase 2 で扱う（現状はプレーンテキスト出力）

---

## Phase 2: Core Features（Week 2）
- [x] Git branch module（`git2` 想定）
- ~~Claude session module（セッション状態の表示）~~（本フェーズはスキップ）
- [x] ANSI スタイル適用（`style` 指定を解釈して装飾出力）
- [x] 基本的なエラーハンドリング拡張（ユーザ視点のメッセージ最適化）
- [x] Config のバリデーション（値域・想定外キー検出）

例（設定抜粋）:

```toml
format = "$directory $git_branch $claude_model"

[git_branch]
format = "[🌿 $branch]($style)"
style = "bold green"

[claude_model]
format = "[$symbol$model]($style)"
style = "bold yellow"
```

---

## Phase 3: Polish（Week 3）
- [ ] Git status module（変更数やアイコン表示の基礎）
- [ ] 簡易キャッシュ（高頻度情報の再計算抑制）
- [ ] モジュール実行タイムアウト（ハング防止・設定連動）
- [ ] 基本テストの拡充（モジュール横断／フォーマットの境界ケース）
- [ ] ドキュメント整備（ユーザーガイド／開発者向けガイド）

---

## Phase 4: Advanced（Optional）
- [ ] 並列実行（`rayon` 等でのモジュール並列化）
- [ ] カスタムエラー型（`thiserror`）
- [ ] ベンチマーク（`criterion`）
- [ ] 追加モジュールの拡充
- [ ] インストールスクリプト整備

---

## リファクタリング（Phase 1 後の品質向上）
- [x] モジュールシステム改善（中央ディスパッチャ導入）
- [x] `Module` トレイト拡張（`context`/`config` を引数に）
- [x] フォーマット文字列パーサ導入（`parse_format`/`extract_modules_from_format`）
- [ ] エラーハンドリング高度化（部分成功の許容・UI 最適化） → 採用見送り（現状方針で十分）
- [ ] Config のモジュール設定分離（汎用 `ModuleConfig` 除去） → 採用見送り（現状の型で妥当）
- [x] テストヘルパー整備（重複削減・`rstest` 活用）

出典: `docs/init/archive/todo-refactoring-phase1.md`（`status: Done/Decline` を反映）

---

## 仕様・設計ドキュメント連携
- 参考: `docs/init/01_spec.md`（Claude Code の Status Line 仕様）
- 参考: `docs/init/archive/statusline.md`（公式仕様の要点・例）
- 計画: `docs/init/archive/plan.md`（Phase 2 以降の粒度は本ロードマップへ統合／同期）
- 実績: `docs/init/archive/todo-phase1.md`（Phase 1 実装詳細と完了報告）
- 変更: `docs/init/archive/removed-features.md`（不要機能の整理履歴）

---

最終更新: 自動生成（docs/init 配下と src を基に反映）
