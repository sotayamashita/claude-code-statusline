# Beacon - Detailed Tasks (Consolidated)

このドキュメントは過去の計画・タスク文書を統合した最新版です。最新の進捗や採用/見送り判断は本ファイルを信頼し、必要に応じて差分を反映します。

---

## Phase 1: MVP タスクリスト（統合）



### 実装済みの機能
- ✅ 基本的なCLI構造（clap使用）
- ✅ JSON入力の解析（serde/serde_json使用）
- ✅ Module traitとモジュールシステム
- ✅ 2つのコアモジュール（Directory, ClaudeModel）
- ✅ 単一行出力とClaude Code統合
- ✅ エラー時のフォールバック表示
- ✅ 設定ファイル（Config構造体、TOML読み込み）
- ✅ プロジェクト構造の整理（types/, config.rs, modules/）
- ✅ テストの追加（config.rs, parser.rs, modules）
- ✅ デバッグ機能のリファクタリング（DebugLoggerモジュール）
- ✅ Context構造体（アプリケーション全体のデータと設定を一元管理）
- ✅ モジュール個別の設定（DirectoryConfig, ClaudeModelConfig）
- ✅ エラーハンドリングの改善（anyhow使用）

### 詳細タスク（抜粋）
- CLI/引数・ヘルプの実装（clap）
- ClaudeInput/ModelInfo/WorkspaceInfo/OutputStyle 型定義と stdin JSON の読み込み
- Config の Default/ロード/テスト
- DebugLogger の導入と BEACON_DEBUG/設定連動
- Context の構築と各モジュールの Context 化
- Directory/ClaudeModel モジュール実装（should_display/render）
- 生成処理（generate_prompt）と単一行出力
- 基本の統合テスト（test-input.json）

### 完了基準
- JSON 入力を正しく処理し、Directory/ClaudeModel が動作
- 単一行のステータスラインを出力し、エラーでもパニックしない
- 設定/テスト/デバッグが整備され、Context で一元管理

---

## Phase 2: Core Features（詳細タスク・TDD）



### ゴール
- Git/Session 等の追加モジュール（本フェーズは Git Branch のみ採用）
- ANSI スタイル適用（最小）
- ユーザ視点のエラーハンドリング改善
- Config バリデーション導入

### 1) Git Branch Module（`$git_branch`）
- Red（テスト）
  - リポジトリ外: 非表示
  - リポジトリ内: ブランチ名 or 分離 HEAD は短SHA
  - `disabled = true` で非表示
- Green（実装）
  - 依存 `git2` 追加、設定型 `GitBranchConfig { format, style, symbol, disabled }`
  - `should_display`: `Repository::discover` 成功かつ `!disabled`
  - `render`: format/style 評価、ブランチ名/短SHA を表示
  - ディスパッチ登録（`modules/mod.rs`）
- Refactor
  - テスト重複整理、境界ケース追加
- 受け入れ条件
  - Git 管理外は無表示、管理下で名称/短SHAを表示、`disabled` で非表示

### ~~2) Claude Session Module（`$claude_session`）~~（本フェーズはスキップ）
- 将来フェーズで検討

### 3) ANSI スタイル適用（最小）
- Red（テスト）
  - `apply_style("X", "bold yellow")` で ANSI 付与
  - 未知トークンは無視
  - Directory/ClaudeModel が format/style を反映
- Green（実装）
  - `src/style.rs` 追加、`apply_style` とテンプレート評価を提供
  - 各モジュールが自モジュールの format を評価
- Refactor
  - マッピングをテーブル化（bold/italic/underline/8色）
- 受け入れ条件
  - 代表的なスタイル指定で ANSI が適用、未知でもクラッシュしない

### 4) エラーハンドリング改善（文言・出力先）
- Red（統合テスト）
  - 不正 JSON / 無効 TOML: stdout は簡潔、stderr は詳細
- Green（実装）
  - DebugLogger のエラーログ一貫性、`main.rs` の整理
- Refactor
  - メッセージの定数化、i18n を考慮した命名

### 5) Config バリデーション
- スキーマ/値域/想定外キー検出、エラーメッセージ整備

---

## Phase 3: Polish（品質向上）



### ゴール
- 品質向上（正確性・体感速度・ハング耐性・可読性）
- 簡易キャッシュとタイムアウトによる安定運用
- モジュール横断のテスト拡充とドキュメント整備

### 1) Git Status Module（`$git_status`）
- Why/Effect
  - 状態（変更/ステージ/未追跡/先行・遅延）を軽量表示、認知負荷を低減
- Starship 準拠の表示/記号/format を踏襲
- Red/Green/Refactor（要点）
  - `git2` で `statuses()` と upstream 比較
  - `$all_status` と `$ahead_behind` を組み立て、`count==0` は非表示
  - ディスパッチ登録、`disabled` 対応
  - 重い計算は簡易キャッシュを併用

### 2) 簡易キャッシュ
- 高頻度取得（Git/Dirなど）をメモ化して I/O を抑制
- `OnceLock`/`Lazy` を用いた同一実行内キャッシュ

### 3) モジュール実行タイムアウト
- ハング防止・ユーザ設定に連動したタイムアウトラッパー

### 4) テスト拡充/Docs 整備
- 境界ケース/モジュール横断、ユーザーガイド/開発者ガイド

---

## Refactoring（Phase 1 後・品質向上）



### 優先度高（Done）
1. モジュールシステム改善（中央ディスパッチャ）
2. Module トレイト改善（`context`/`config` を引数に）
3. フォーマット文字列パーサ（`format = "$directory $claude_model"` を解析）

### 優先度中（Decline/Done）
4. エラーハンドリング高度化（部分成功の許容）→ Decline
5. Config のモジュール設定分離 → Decline
6. テストヘルパー（TestRenderer）→ Done

---

最終更新: 過去文書を統合し、本ファイルを唯一のタスク基盤としました。
