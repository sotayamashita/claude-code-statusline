# Beacon - Phase 2 Detailed Tasks (Core Features, TDD)

> 本ドキュメントは `docs/init/02_roadmap.md` の Phase 2 を TDD（Red-Green-Refactor）で進めるためのタスクに落とし込みます。各セクションは「先にテスト（Red）→ 実装（Green）→ リファクタ（Refactor）」の順で記述します。

## ゴール（Phase 2）
- Git/Session といった追加モジュールの導入
- ANSI スタイル適用（最低限）
- ユーザ視点のエラーハンドリング改善（文言・出力先）
- Config のバリデーション導入

---

## 1) Git Branch Module（`$git_branch`）

### Red: 先に書くテスト
- [x] `src/modules/git_branch.rs` にユニットテストを追加
  - [x] リポジトリ外: `should_display == false`
  - [x] リポジトリ内（master/main ブランチ）: `should_display == true` かつ `render()` はブランチ名を含む
  - [x] 分離 HEAD: `render()` は短い SHA を含む（7〜8 桁程度）
  - [x] `disabled = true` で非表示
  - 実装ヒント（テスト側）: `tempfile::tempdir` + `git2::Repository::init` で一時リポジトリ作成、ブランチ作成/チェックアウト

### Green: 実装タスク
- [x] 依存追加: `git2` を `Cargo.toml` へ（使えない環境向けに後述フォールバック）
- [x] 設定型追加: `GitBranchConfig { format, style, symbol, disabled }` を `src/types/config.rs`
- [x] 実装: `src/modules/git_branch.rs`
  - [x] `Module` 実装（`name() -> "git_branch"`）
  - [x] `should_display`: `git2::Repository::discover` 成功時のみ true（かつ `disabled == false`）
  - [x] `render`: `format` を評価し、ブランチ名/短 SHA に `style` を適用
  - [ ] フォールバック（オプション）: `git` コマンド実行での分岐
- [x] ディスパッチ登録: `modules/mod.rs` に `pub mod git_branch;` と `handle_module` を追加

### Refactor: 仕上げ
- [ ] テストの重複を整理、エラーパスの境界ケースを追加
- [x] 使用例を `docs/init/02_roadmap.md` に追記

### 受け入れ条件
- [x] Git 管理外で無出力、管理下ではブランチ名/短 SHA を表示
- [x] `disabled = true` で非表示

---

## ~~2) Claude Session Module（`$claude_session`）~~

> 本フェーズではスキップ（実装しません）。将来フェーズで対応予定。

### Red: 先に書くテスト
- [ ] `src/modules/claude_session.rs` にユニットテストを追加
  - [ ] `session_id` あり: `should_display == true` かつ `render()` は `symbol + 短縮ID`（先頭 6〜8 桁）
  - [ ] `session_id` 空/空白: `should_display == false`
  - [ ] `disabled = true` で非表示

### Green: 実装タスク
- [ ] 設定型追加: `ClaudeSessionConfig { format, style, symbol, disabled }`
- [ ] 実装: `src/modules/claude_session.rs`（`Module` 実装）
- [ ] ディスパッチ登録 + `$claude_session` を `format` で解決可能に

### Refactor: 仕上げ
- [ ] 短縮 ID の桁数を設定で調整可能にする（任意）
- [ ] 使用例をロードマップへ追記

### 受け入れ条件
- [ ] `session_id` の有無で表示切替が機能
- [ ] `disabled = true` で非表示

---

## 3) ANSI スタイル適用（最小）

### Red: 先に書くテスト
- [x] `src/style.rs` にユニットテスト
  - [x] `apply_style("X", "bold yellow")` が ANSI 付きの文字列を返す（bold + yellow）
  - [x] 未知トークンは無視し、素の文字列を返す
- [x] 既存モジュールのテスト追加/更新
  - [x] `DirectoryModule::render` が `format/style` を反映
  - [x] `ClaudeModelModule::render` が `format/style` を反映

### Green: 実装タスク
- [x] `src/style.rs` を新規追加して `apply_style(text, style)`/`render_with_style_template` を提供
- [x] 各モジュールで自モジュールの `format` を評価し、必要に応じて `apply_style` を適用
- [x] 全体の `parse_format` は Phase 1 のまま（モジュール出力の連結に専念）

### Refactor: 仕上げ
- [ ] ANSI コードマッピングをテーブル化、最小セット（bold/italic/underline/8色）に限定

### 受け入れ条件
- [x] 代表的なスタイル指定で ANSI が適用される
- [x] 未知/無効指定でもクラッシュしない

---

## 4) エラーハンドリング改善（文言・出力先）

### Red: 先に書くテスト（統合テスト推奨）
- [x] `tests/error_handling.rs` を追加
  - [x] 不正 JSON を stdin に与えると、stdout は固定の簡潔メッセージ、stderr は詳細（少なくともエラー種別が含まれる）
  - [x] config 読込失敗（無効 TOML）時も同様の方針

### Green: 実装タスク
- [x] `DebugLogger` のエラーログ出力の一貫性を確認・補強（stderr の用途を明確化）
- [x] `main.rs` のエラーハンドリングを整理（stdout 簡潔、stderr 詳細）

### Refactor: 仕上げ
- [ ] メッセージ定数化、i18n を考慮した命名（任意）

### 受け入れ条件
- [x] 正常時は 1 行出力のみ（改行なし）
- [x] 異常時は stdout 簡潔、stderr 詳細（デバッグ有効時はさらに詳細）

---

## 5) Config バリデーション

### Red: 先に書くテスト
- [x] `src/types/config.rs` のユニットテスト
  - [x] `command_timeout` 下限/上限外でエラー
  - [x] 未知スタイルトークンで警告（エラーにはしない）
  - [x] `format` に未知 `$token` が含まれると警告

### Green: 実装タスク
- [x] `Config::validate(&self) -> Result<()>` を追加
- [x] `main.rs` で `load()` 後に `validate()` を呼び、警告は stderr（`DebugLogger` 利用）

### Refactor: 仕上げ
- [ ] バリデーションルールの見直しとメッセージ整備

### 受け入れ条件
- [x] 明らかな不正値に対して適切に反応（エラー or 警告）
- [x] 既存設定を壊さない（警告中心）

---

## 6) 配線・ドキュメント

### Red: 先に書くテスト
- [ ] ドキュメント例の `format` をそのまま適用してもエラーにならない（最低限の smoke test）

### Green: 実装タスク
- [x] `docs/init/02_roadmap.md` に `$git_branch`, `$claude_session` の例を追記（実装は Session スキップ）
- [x] 本ファイルに最小の設定例を追記

### 設定例（抜粋）
```toml
format = "$directory $git_branch $claude_model $claude_session"

[git_branch]
format = "[🌿 $branch]($style)"
style = "bold green"

[claude_session]
format = "[🔗 $short_id]($style)"
style = "italic yellow"

[claude_model]
format = "[$symbol$model]($style)"
style = "bold yellow"
```

### Refactor: 仕上げ
- [ ] ドキュメントの章立て整理、アーカイブとの整合

---

## 非目標（Phase 2では扱わない）
- 並列実行（`rayon`）→ Phase 4
- 本格的なキャッシュ/タイムアウト → Phase 3
- カスタムエラー型（`thiserror`）→ Phase 4

---

## 進行管理（チェックリスト）
- [x] Git Branch Module（Red→Green→Refactor 一部完了）
- [ ] ~~Claude Session Module（Red→Green→Refactor 完了）~~（スキップ）
- [x] ANSI スタイル最小実装（Red→Green 完了／Refactor 未）
- [x] エラーハンドリング（Red→Green 完了）
- [x] Config バリデーション（Red→Green 完了）
- [x] ドキュメント更新（例の反映・整合性確認）
