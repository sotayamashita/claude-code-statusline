# Beacon - Phase 3 Detailed Tasks (Polish / 品質向上)

> 本ドキュメントは `docs/init/02_roadmap.md` の Phase 3 を、実装タスクへ具体化したものです。各タスクに「なぜ必要か（Why）」「どのような効果があるか（Effect）」を明示し、着手順序と受け入れ条件を定義します。必要に応じてテスト観点（Red→Green→Refactor）も付記します。

## ゴール（Phase 3）
- 品質向上（正確性・体感速度・ハング耐性・可読性）
- 簡易キャッシュとタイムアウトによる安定運用
- モジュール横断のテスト拡充とドキュメント整備

---

## 1) Git Status Module（`$git_status`）

### Why（なぜ必要か）
- Git リポジトリでの作業が一般的なため、変更数・状態をワンショットで把握できると利便性が大きく向上する。
- Branch 表示（Phase 2）に加え、作業状況（変更, ステージ, 未追跡など）を最小限で示すと、プロンプトからの認知コストが下がる。

### Effect（効果）
- ステージング・未ステージ・未追跡の差分を軽量に可視化でき、意図しないコミット漏れや状態の取り違えを減らす。
- Starship と近い表現を採用することで、既存ユーザーの学習コストを下げ、設定移行もしやすくなる。

### Starship の挙動（調査結果の要約）
- 既定の `format`: `([\[$all_status$ahead_behind\]]($style) )`
- `$all_status`: `$conflicted$stashed$deleted$renamed$modified$typechanged$staged$untracked`
- `$ahead_behind`: `diverged`/`ahead`/`behind`/`up_to_date` を表示
- 既定シンボル（例）:
  - `conflicted`: `=`（競合あり）
  - `stashed`: `$`（スタッシュあり）
  - `deleted`: `✘`（削除をステージ）
  - `renamed`: `»`（リネームをステージ）
  - `modified`: `!`（未ステージの変更）
  - `typechanged`: ''（型変更、既定は空）
  - `staged`: `+`（ステージ済み）
  - `untracked`: `?`（未追跡）
  - `ahead`: `⇡`、`behind`: `⇣`、`diverged`: `⇕`（それぞれコミット数をサフィックスで表示）

例: 変更1件、ステージ2件、未追跡3件、1コミット分 ahead → `[+2!1?3⇡1]`

### Red（テスト観点の例）
- [x] リポジトリ外では `should_display == false`。
- [ ] 変更なし: `$all_status` が空、`$ahead_behind` が `''` なら非表示 or クリーン表示（設定依存）。
- [x] 未ステージ変更あり: `!{count}` が含まれる。
- [x] ステージ済み変更あり: `+{count}` が含まれる。
- [x] 未追跡あり: `?{count}` が含まれる。
- [ ] ahead/behind/diverged の表示とカウントが整合。
- [x] `disabled = true` で非表示。

### Green（実装タスク）
- [x] 設定型追加（`src/types/config.rs`）
  - [x] `GitStatusConfig { format, style, symbols, disabled }`
  - [x] `symbols`: `conflicted, stashed, deleted, renamed, modified, typechanged, staged, untracked, ahead, behind, diverged`。
  - [x] 既定の `format` は Starship に準拠（`([\[$all_status$ahead_behind\]]($style) )`）。
- [x] モジュール実装（`src/modules/git_status.rs`）
  - [x] `Module` 実装（`name() -> "git_status"`）
  - [x] `git2::Repository::discover` → `statuses()` と upstream 比較で ahead/behind/diverged を取得。
  - [x] `$all_status` と `$ahead_behind` を評価し、各セグメントは `symbol + count` を生成（count==0 は非表示）。
  - [x] `style` は全体に適用、空なら生文字列。
- [x] ディスパッチ登録（`src/modules/mod.rs`）と `format` 解決対応。

### Refactor（仕上げ）
- [ ] 記号や順序は設定で変更可能（Starship に準じたキー名）。
- [x] 重い計算を避けるため、Phase 3 の「簡易キャッシュ」を利用（同一 CWD での再計算抑制）。

### 受け入れ条件
- リポジトリ外では非表示、内では状態を最小限の表記で表示。
- Starship と同等の情報量（記号+件数、ahead/behind/diverged）を表示可能。
- 設定を変更することで、記号とスタイルが変えられる。

---

## 2) 簡易キャッシュ（高頻度情報の再計算抑制）

### Why（なぜ必要か）
- 同一実行内で同じ情報を繰り返し参照するケース（複数モジュールが Git 情報を共有など）で無駄な I/O を避けたい。
- 今後のモジュール拡張に備え、最小限のキャッシュ抽象を導入して性能を底上げする。

### Effect（効果）
- 体感速度の改善（特に大型リポジトリでの Git 参照）。
- 設計面で「高コストな取得はキャッシュ経由」に統一され、保守性が上がる。

### Starship の挙動（調査結果の要約）
- 仕組み: `Context` 構造体内で `OnceLock` によるメモ化を実施。
  - `dir_contents: OnceLock<Result<DirContents, io::Error>>`（カレントディレクトリ走査結果を一度だけ取得）
  - `repo: OnceLock<Result<Repo, gix::discover::Error>>`（Git リポジトリ情報を一度だけ取得）
- アクセス: モジュールは `context.dir_contents()` や `context.get_repo()` 経由で `get_or_init()` を使用し、未初期化なら初回計算、以降は再利用。
- 範囲: プロセス（1 回のプロンプト生成）内の簡易キャッシュ。TTL なし。CWD が変われば別 `Context` になる前提で再評価。
- 付記（ログ）: `STARSHIP_CACHE` と `STARSHIP_SESSION_KEY` はセッション別ログファイル保存のためのパス/キーであり、データキャッシュ用途ではない。

### Green（実装タスク）
- [x] 依存方針: 標準ライブラリの `std::sync::OnceLock` を優先（必要に応じて `once_cell` で代替可）。
- [x] `Context`（`src/types/context.rs`）に簡易キャッシュを追加
  - [x] `dir_contents: OnceLock<Result<DirContents, io::Error>>`
  - [x] `repo: OnceLock<Result<Mutex<git2::Repository>, git2::Error>>`（可変API対応のため `Mutex` 包装）
  - [x] `fn dir_contents(&self) -> Result<&DirContents, &io::Error>` と `fn repo(&self) -> Result<MutexGuard<'_, Repository>, &git2::Error>` を追加（内部で `get_or_init`）。
  - [ ] 便利関数: `read_file_from_pwd(name)` は `dir_contents` を先に参照して存在チェック後に読み込む。
- [x] 利用箇所の置き換え
  - [x] `git_status` / `git_branch` で `Context` のメモ化 API を利用（直読み/二重スキャン回避）。
  - [x] 同一 CWD 内で複数モジュールが同じ情報を要求する場合でも 1 回の I/O で済むことをテストで確認。
- [x] 無効化/ライフサイクル
  - [x] プロセス/実行単位でのみ有効（TTL なし）。CWD 変更は新しい `Context` として扱う。
  - ~~デバッグ用にヒット/ミスをカウントできる簡易トレースを `debug` 時のみ出力（任意）~~

### Red/Refactor（テスト観点）
- [x] カウンタで呼び出し回数を記録し、同一 CWD 同一要求で 2 回目以降は計算されないことを確認。
- [ ] 実装重複を避けるため、取得→格納のヘルパ関数を用意して責務を統一。

### 受け入れ条件
- キャッシュ導入により、代表モジュール（`git_status`）の 2 回目取得が高速化。
- キャッシュミス時も正確性が担保される（不整合が起きない）。

---

## 3) モジュール実行タイムアウト（ハング防止・設定連動）

### Why（なぜ必要か）
- 外部 I/O（Git 操作など）で想定外のハング/遅延が発生しうる。プロンプトは即時性が重要。
- 既存の `command_timeout`（ms）設定を活用し、全モジュールの最悪時間を制限したい。

### Effect（効果）
- ハングによる全体停止を防止し、UX（遅延体感）が安定する。
- タイムアウト時は空出力 or 省略表記でフェイルソフトに動作（stderr に警告を記録）。

### Green（実装タスク）
- [x] 汎用ユーティリティ追加（例: `src/timeout.rs`）
  - [x] `run_with_timeout<F, T>(dur: Duration, f: F) -> Result<Option<T>>` のような同期 API（`std::thread::spawn` + `join_timeout` パターン）。
  - [x] タイムアウト時は `Ok(None)` を返し、呼び出し側で「無出力/警告」に分岐。
- [x] 代表モジュールへ適用（`git_status`, `git_branch`）
  - [x] 既存ロジックを `run_with_timeout` でラップ。
  - [x] `Config.command_timeout` を参照。

### Red/Refactor（テスト観点）
- [x] 疑似的にスリープの長い処理を呼び、設定閾値下でタイムアウトになること。
- [ ] タイムアウトでもクラッシュせず、最終出力 1 行の制約が維持されること。

### 受け入れ条件
- 代表モジュールでタイムアウトが機能し、ハングしない。
- 警告ログが `DebugLogger` で確認できる。

---

### 参考（Starship の対応）
- グローバル設定に `command_timeout`（既定 500ms）と `scan_timeout`（既定 30ms）があり、外部コマンドやファイルスキャンの待ち時間を制限。
- 外部コマンドはユーティリティ層（例: `exec_timeout(cmd, time_limit)`）で一元的にタイムアウト制御。Git 操作は `Repo::exec_git` 経由で `command_timeout` を継承。
- カスタムモジュールは `command_timeout` を標準適用しつつ、モジュール側で `ignore_timeout` による無効化も可能。
- 中央の `Context` が設定値を保持し、各モジュールが参照して実行時にタイムアウトを渡す設計。

本プロジェクトでは Starship と同様にグローバル `command_timeout` を採用しつつ、「モジュール境界（should_display/render）」で包括的にラップしてハングの波及を防止。将来、外部コマンド実行を導入する場合は Starship の方式にならい、実行ユーティリティ層で `command_timeout` を適用する。また、必要に応じて `scan_timeout` 相当（ディレクトリ・VCS スキャン向け）の導入や、モジュール個別のタイムアウト上書き/無効化（例: `ignore_timeout`）を検討する。

---

## 4) 基本テストの拡充（モジュール横断／フォーマット境界）

### Why（なぜ必要か）
- Phase 1/2 で増えたパス（ANSI, Git, バリデーション）を横断で検証し、回帰を早期検知したい。
- フォーマット境界（未知トークン、空出力、複合スタイルなど）の取り扱いを明文化し、仕様ブレを防ぐ。

### Effect（効果）
- 変更に強い土台（安全に機能追加/改善ができる）。
- ドキュメントとテストの相互参照でチーム内合意が取りやすくなる。

### Red（追加する代表テスト）
- [x] 統合テスト（`tests/`）
  - [x] `$directory $git_branch $git_status $claude_model` を使ったスモーク（Git あり/なし）。
  - [x] タイムアウト有効時でも 1 行出力が維持されること。
- [x] ユニットテスト（`src/parser.rs`, `src/types/config.rs`, 各モジュール）
  - [x] 未知 `$token` の扱い（既存警告の維持）。
  - [x] ANSI の複合指定での安定性（未知トークン混在時でもクラッシュしない）。

### Green/Refactor（実装タスク）
- [ ] `rstest` の活用でケース増加を簡潔に表現。
- [ ] 重複ヘルパは `tests/common/` に集約。

### 受け入れ条件
- 代表的なユーザ設定の組合せで回帰が出ない。
- フォーマット境界ケースの期待が明文化され、テストで担保される。

---

## 5) ドキュメント整備（ユーザー・開発者向け）

### Why（なぜ必要か）
- 新規ユーザーが最短で使い始められる「導入手順」と、開発者が迷わない「実装規約/拡張方針」が必要。
- Phase 3 で追加される `git_status`/キャッシュ/タイムアウトの仕様を明文化して、問い合わせや設定ミスを減らす。

### Effect（効果）
- 導入の障壁低下、チームへの展開が容易に。
- Issue/PR の議論が事実ベースで進めやすくなり、レビュー効率が上がる。

### Green（作成/更新するドキュメント）
- [ ] ユーザーガイド（`README.md` もしくは `docs/guide/configuration.md`）
  - [ ] インストール、最小設定例、`format` の基本、代表モジュールの使い方。
  - [ ] タイムアウト/キャッシュの概要と注意点（安全なデフォルト）。
- [ ] 開発者ガイド（`docs/guide/contributing.md`）
  - [ ] モジュールの追加手順（`Module` トレイト、`ModuleConfig`）
  - [ ] キャッシュ/タイムアウトの適用規約（どこでラップするか、ログ方針）。
- [ ] ロードマップ同期（`specs/2025-09-04-mvp/02-plan.md`）
  - [ ] 実装/仕様更新点の反映（チェックボックス更新）。

### 受け入れ条件
- 新規ユーザーがガイド通りに最小構成で実行できる。
- 開発者がモジュール追加時にガイドだけで実装を進められる。

---

## 進行管理（チェックリスト）
- [x] Git Status Module（Green 完了／Red・Refactor 未）
- [x] 簡易キャッシュ（Green→適用）
- [x] タイムアウト（Green→適用）
- [x] テスト拡充（統合/ユニット）
- [ ] ドキュメント整備（ユーザー/開発者）

---

## 参考・補足
- 仕様: `specs/2025-09-04-mvp/01_spec.md`（Phase 3 で `once_cell` によるキャッシュ導入を示唆）
- 設定: `~/.config/beacon.toml`（`command_timeout` は既存。キャッシュは Phase 3 では実行内のみ）
- ロードマップ: `specs/2025-09-04-mvp/02_plan.md`（Phase 3: 品質向上・キャッシュ・タイムアウト・ドキュメント）
- Starship 参考実装: `Context` に `OnceLock` で `dir_contents`/`repo` を保持し、`get_or_init` により同一実行内の再計算を抑止。`STARSHIP_CACHE` はログ保存用途でありデータキャッシュではない。
