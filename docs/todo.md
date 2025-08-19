# Beacon - Phase 1 TODO List

> このファイルは @docs/plan.md の Phase 1 を詳細化したタスクリストです。
> 各タスクは約10分で完了できるように設計されています。

## Phase 1: MVP タスクリスト

[plan.md - Phase 1](./plan.md#phase-1-mvp-week-1---minimum-viable-product) の実装詳細：

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

### 未実装の機能
- ⏳ Context構造体
- ⏳ ModuleConfig構造体（各モジュールの個別設定）
- ⏳ エラーハンドリングの改善（anyhow使用）

### 1. プロジェクト初期設定 (20分) ✅
- [x] src/main.rsに基本的なmain関数を作成 (10分)
  - "Hello, Beacon!"を出力するだけの最小実装
- [x] プロジェクトがビルドできることを確認 (10分)
  - `cargo build`
  - `cargo run`

### 2. CLI構造の実装 (30分) ✅
- [x] clapクレートを追加 (10分)
  - `cargo add clap --features derive`
- [x] CLIの引数構造体を定義 (10分)
  - `use clap::Parser`
  - `#[derive(Parser)]`構造体の作成
  - version, about情報をenv!マクロでCargo.tomlから取得
- [x] ヘルプメッセージが正しく表示されることを確認 (10分)
  - `cargo run -- --help`
  - `cargo run -- --version`

### 3. JSON入力の処理 (50分) ✅
- [x] serde/serde_jsonクレートを追加 (10分)
  - `cargo add serde --features derive`
  - `cargo add serde_json`
- [x] ClaudeInput構造体を定義 (10分)
  - `#[derive(Debug, Deserialize)]`
  - hook_event_name, session_id, cwdなどのフィールド
- [x] ModelInfo/WorkspaceInfo/OutputStyle構造体を定義 (10分)
  - types/claude.rsモジュールに分離
  - 公式ドキュメントの構造に準拠
- [x] stdinからJSONを読み込むテストコード作成 (10分)
  - test-input.jsonを作成
  - `cat test-input.json | cargo run`で動作確認
- [x] parser.rsモジュールと単体テストを追加 (10分)
  - Rustベストプラクティスに従った構造
  - 3つのテストケース（正常、エラー、必須フィールド欠落）

### 4. 設定ファイルの基本実装 (40分) ✅
- [x] Config構造体を定義 (10分)
  - formatフィールド
  - command_timeoutフィールド
  - debugフィールド（デバッグモードの有効/無効）
  - types/config.rsに実装
- [x] デフォルト設定を定義 (10分)
  - Default traitの実装
  - 基本的なformat文字列の設定: "$directory $claude_model"
  - debug: falseをデフォルトに
- [x] TOML設定ファイルの読み込み (10分)
  - `~/.config/beacon/config.toml`のパスを構築
  - ファイルが存在しない場合のデフォルト処理
  - config.rsにload()メソッドを実装
- [x] 設定のテストを追加 (10分)
  - 6つのテストケースを実装
  - デフォルト値、TOML解析、パス構築のテスト

### 4.1. デバッグ機能のリファクタリング (30分) ✅
- [x] 現在のハードコードされたデバッグ実装を削除 (10分)
  - main.rs内の一時的なデバッグコードを削除
  - OpenOptionsを使った直接的なファイル書き込みを削除
- [x] 専用のDebugLoggerモジュールを作成 (10分)
  - src/debug.rsを新規作成
  - Config構造体からdebugフラグを読み取り
  - ログ出力を一元管理
- [x] 設定ベースのデバッグ切り替えを実装 (10分)
  - 環境変数 BEACON_DEBUG でも制御可能に
  - デバッグ有効時のみログファイルを作成（/tmp/beacon-debug.log）

### 5. Context構造体の実装 (20分)
- [ ] Context構造体を定義 (10分)
  - ClaudeInputからの情報を保持
  - 現在のディレクトリ情報
- [ ] Contextのnewメソッドを実装 (10分)
  - ClaudeInputを受け取ってContextを生成

### 6. Module traitの定義 (20分) ✅
- [x] Module trait を定義 (10分)
  - name(), should_display(), render()メソッド
- [ ] ModuleConfigの基本構造を定義 (10分)
  - style, formatフィールド
  - 各モジュールの共通設定として使用

### 7. Directory モジュールの実装 (30分) ✅
- [x] DirectoryModule構造体を作成 (10分)
  - 基本的なフィールド定義
- [x] Module traitの実装 (10分)
  - should_display: 常にtrue
  - render: 現在のディレクトリパスを返す
- [x] ホームディレクトリの~置換を実装 (10分)
  - dirs::home_dir()を使用
  - パスの短縮表示

### 8. Character モジュール（削除済み）
- [x] ~~CharacterModule構造体を作成~~ → 削除
  - Claude Codeのステータスラインは出力専用のため不要
  - docs/removed-features.mdに削除理由を記載
  - ✅ CharacterConfig関連のコードも完全削除（後方互換性不要）

### 9. Claude Model モジュールの実装 (20分) ✅
- [x] ClaudeModelModule構造体を作成 (10分)
  - model情報を保持
- [x] Module traitの実装 (10分)
  - should_display: model情報がある場合true
  - render: "<Opus>"形式で表示

### 10. 出力の組み立て (30分) ✅
- [x] generate_prompt関数を実装 (10分)
  - 各モジュールを順番に実行
  - 結果を連結
- [x] ANSI色なしでの基本出力 (10分)
  - プレーンテキストでの出力確認
- [x] 単一行での出力を保証 (10分)
  - 改行を含まない出力
  - print!()でstdoutに出力

### 11. 統合テスト (20分) ✅
- [x] テスト用JSONファイルを作成 (10分)
  - test-input.jsonを作成
- [x] Claude Code設定を追加 (10分)
  - .claude/settings.local.jsonに設定追加
  - リリースビルドを.claude/beaconに配置
- [x] エンドツーエンドテストの実行 (10分)
  - `echo '{"hook_event_name":"Status",...}' | cargo run`
  - 期待される出力: "~/projects/beacon <Opus>"

### 12. プロジェクト構造の整理 (30分) ✅
- [x] 型定義の整理 (10分)
  - types/claude.rs: Claude Code入力の型
  - types/config.rs: 設定の型とデフォルト値
  - types/mod.rsから不要な再エクスポートを削除
- [x] 設定ロジックの分離 (10分)
  - config.rs: 設定のロードロジックのみ
  - config/ディレクトリを廃止し、単一ファイルに
- [x] CLAUDE.mdの更新 (10分)
  - プロジェクト構造を明確に記載
  - 各ファイルの責務を明記

## 完了基準
- [x] Claude CodeのJSON入力を正しく処理できる
- [x] 2つの基本モジュール（directory, claude_model）が動作する
- [x] 単一行のステータスラインが出力される
- [x] エラーが発生してもパニックしない（基本的なエラーハンドリング実装済み）
- [x] 設定ファイルの構造が整備されている
- [x] テストカバレッジが十分（13個のテストがすべて成功）
- [x] デバッグ機能が設定ベースで制御可能（設定ファイル or 環境変数）

## 注意事項
- ANSIカラーコードは後回し（[Phase 2](./plan.md#phase-2-core-features-week-2---add-more-modules)で実装）
- Git関連機能は実装しない（[Phase 2](./plan.md#phase-2-core-features-week-2---add-more-modules)で実装）
- 複雑なエラーハンドリングは避ける（anyhow::Resultで統一予定）
- パフォーマンス最適化は考えない（[Phase 4](./plan.md#phase-4-advanced-optional---for-continued-learning)で実装）

## 次のステップ

Phase 1 の残りタスク：
- Context構造体の実装
- ModuleConfigの実装
- エラーハンドリングの改善（anyhow導入）

Phase 1 完了後：
- → [Phase 2: Core Features](./plan.md#phase-2-core-features-week-2---add-more-modules)
- → [Phase 3: Polish](./plan.md#phase-3-polish-week-3---improve-quality)
- → [Phase 4: Advanced](./plan.md#phase-4-advanced-optional---for-continued-learning)
