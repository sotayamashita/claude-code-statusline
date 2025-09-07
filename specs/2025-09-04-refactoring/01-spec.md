# claude-code-statusline リファクタリング計画（2025-09-04）

この文書は claude-code-statusline を「再利用可能なライブラリ API」と「薄い CLI」に明確分離し、将来的に外部ライブラリからモジュール拡張できるアーキテクチャへ移行するための具体的な計画です。実装はオーナー承認後に着手します。

- 対象リポジトリ: claude-code-statusline
- 参照仕様: `specs/project.md`, `specs/2025-08-18-mvp/01-spec.md`
- 目的: API/CLI の境界を明確化し、拡張可能なモジュールシステムと安定した公開 API を提供
- 非目的: 動的リンク型プラグイン（`dlopen` 等）の導入、完全な Starship 互換パーサの実装

---

## 1. 背景と課題

現状は単一クレートで CLI とライブラリが同居し、以下の課題があります:
- `generate_prompt` が `main.rs` 内にあり、外部から再利用しにくい
- モジュール登録が静的 `match` で外部拡張が困難
- 設定は内蔵型に固定で、未知モジュールの設定を取り込めない
- ロギング/エラー方針が CLI とコアで混在

この計画では、コア機能を公開 API として切り出し、拡張ポイントと責務の分離を明確にします。

---

## 2. 目標アーキテクチャ

Rust ワークスペース化しクレート分割します。

```
claude-code-statusline/ (workspace)
├─ crates/
│  ├─ claude-code-statusline-core/   # ライブラリ: 公開 API、型、フォーマッタ、タイムアウト、Context、ConfigProvider
│  └─ claude-code-statusline-cli/    # バイナリ: CLI エントリ、stdin/stdout、ログ初期化、サブコマンド
└─ (既存 tests/ はトップで維持しつつ調整)
```

オプション案（将来）:
- `ccs-modules`: 標準モジュール群を分離（または `claude-code-statusline-core` に内蔵 + feature で制御）

---

## 3. 公開 API 設計（claude-code-statusline-core）

### 3.1 エンジンとレジストリ

```rust
pub struct Engine {
    registry: Registry,
    config: Config,
    timeout_ms: u64,
}

impl Engine {
    pub fn new(config: Config) -> Self { /* ... */ }

    pub fn with_registry(mut self, registry: Registry) -> Self { /* ... */ }

    pub fn render(&self, input: &ClaudeInput) -> Result<String, CoreError> { /* ... */ }
}

pub struct Registry { /* 内部: HashMap<String, Arc<dyn ModuleFactory>> */ }

impl Registry {
    pub fn new() -> Self { /* ... */ }
    pub fn register(&mut self, name: &str, factory: Arc<dyn ModuleFactory>) { /* ... */ }
    pub fn get(&self, name: &str) -> Option<Arc<dyn ModuleFactory>> { /* clone して返す */ }
}
```

### 3.2 モジュール拡張ポイント

```rust
pub trait Module: Send + Sync {
    fn name(&self) -> &str;
    fn should_display(&self, ctx: &Context) -> bool;
    fn render(&self, ctx: &Context) -> Result<String, ModuleError>;
}

pub trait ModuleFactory: Send + Sync {
    fn create(&self, ctx: &Context, cfg: &dyn ConfigProvider) -> Result<Box<dyn Module>, ModuleError>;
}
```

- 内蔵モジュールはこれまで通り型付き設定（`types::config::*`）を参照
- 外部モジュールは `ConfigProvider` 経由で自前の型にデシリアライズ

### 3.3 設定プロバイダ

```rust
pub trait ConfigProvider {
    // 既存の型付き getter（内蔵モジュール用）
    fn core_config(&self) -> &Config;

    // 未知セクション取得（外部モジュール用）
    fn module_table(&self, name: &str) -> Option<&toml::value::Table>;
}
```

`Config` は従来の型に加え、型未定義セクションを `extra_modules: HashMap<String, toml::value::Table>` に保持します。

### 3.4 フォーマッタ/スタイル/タイムアウト
- フォーマッタ: トップレベル形式は「空白区切り `$module` 展開」を維持
- モジュール内テンプレート: `style::render_with_style_template()` をパブリック API 化
- タイムアウト: `timeout::run_with_timeout` を `Engine` 内部で利用（将来 `rayon` 並列化をオプション化）

---

## 4. CLI 設計（claude-code-statusline-cli）

- `stdin` から `ClaudeInput` を読み、`Engine::render()` の 1 呼び出しに集約
- サブコマンドの整理:
  - `config --path|--default|--validate`
  - `modules --list|--enabled`
  - `--debug`, `--version`
- ログ初期化は CLI 側で実施（`tracing-subscriber`）

---

## 5. 依存関係と Feature Gate 方針

- コア（claude-code-statusline-core）
  - 必須: `serde`, `serde_json`, `toml`, `once_cell`
  - エラー: `thiserror`（コアの型付けエラー）、CLI 側は `anyhow`
  - Git 機能: `git2` を `features = ["git"]` でオプション化（既定 off、CLI で on）
  - 並列: `rayon` をオプション `feature = "parallel"`（既定 off）
  - ログ: `tracing`（ロガーは CLI）
- CLI（claude-code-statusline-cli）
  - `clap`, `tracing-subscriber`, 必要に応じて `anstream`/`colorchoice`

---

## 6. エラー/ログ方針

- コア: `thiserror` による `Error`（例: `ConfigError`, `ParseError`, `ModuleError`, `TimeoutError`）
- CLI: `anyhow` で文脈付与しつつ、既存の簡潔なユーザ向けメッセージ（`messages.rs`）は維持
- ロギング: コアは `tracing` マクロのみ使用。出力先は CLI が初期化
- 例外系: タイムアウト・未知モジュールは黙ってスキップ（現行仕様を踏襲）

---

## 7. モジュール拡張戦略

- 初期段階: 静的リンク拡張（外部クレートが `Registry` に登録）
- 将来案: `inventory` による自動登録（要検討。リンカ依存/可観測性のトレードオフ）
- 動的プラグイン（`libloading`）は当面対象外（安全性・移植性コストが高い）

拡張時の設定取得例（外部モジュール側）:

```rust
#[derive(serde::Deserialize)]
struct MyModuleConfig { style: String, format: String }

impl ModuleFactory for MyFactory {
    fn create(&self, _ctx: &Context, cfgp: &dyn ConfigProvider) -> Result<Box<dyn Module>, ModuleError> {
        let table = cfgp.module_table("my_module").ok_or(ModuleError::ConfigMissing)?;
        let mycfg: MyModuleConfig = toml::Value::Table(table.clone()).try_into().map_err(ModuleError::from)?;
        Ok(Box::new(MyModule::new(mycfg)))
    }
}
```

---

## 8. フォーマッタ/レンダラ方針

- 現行: トップレベルは空白区切り `$name` のみサポート（仕様と一致）。
- 将来: 連結書式（`$directory$git_branch...`）や Starship 互換の一部強化は別フェーズで検討。
- モジュール内の `[$text]($style)` は既存の軽量テンプレータを公開して継続利用。

---

## 9. 段階的移行計画（マイルストーン）

### フェーズ 1（API 抽出・無破壊移行）
- ワークスペース化し `crates/claude-code-statusline-core`, `crates/claude-code-statusline-cli` を作成
- `Engine`, `Registry`, `ConfigProvider` を導入（箱だけ）
- 既存 `generate_prompt` を `Engine::render` に移し、CLI から呼ぶ
- 既存テストを極力変更せずに通るよう shims を用意

### フェーズ 2（拡張性の実効化）
- `Config` に `extra_modules: HashMap<String, toml::Table>` を追加
- `Registry` に内蔵モジュールを登録する初期化関数を導入
- `git2` を feature gate 化（CLI では on）

### フェーズ 3（品質向上）
- コアのエラーを `thiserror` 化し、CLI では `anyhow` で wrap
- 既存 `DebugLogger` を `tracing` に置換（CLI で subscriber 初期化）
- `cargo clippy -- -D warnings` を維持

### フェーズ 4（最適化/将来）
- `feature = "parallel"` 時にモジュール並列レンダリング（順序は format の順を維持）
- フォーマッタ強化（必要時）。ベンチ（`criterion`）で回帰確認

---

## 10. テスト/移行ガイド

- 既存の単体・結合テストは最大限温存。
- 新規: `claude-code-statusline-core` 単体テスト（Engine/Registry/ConfigProvider）
- CLI は E2E テストで `stdin -> stdout` の 1 行出力保証を継続検証
- パフォーマンステスト（任意）: `criterion` を core に追加検討

---

## 11. 成功基準（Definition of Done）

- `claude-code-statusline-core` を他クレートから依存して `Engine::render` が利用できる
- 既存 CLI 動作・メッセージ互換（ユーザ体験は変えない）
- 既存テストスイートがグリーン（微調整を除く）
- ドキュメント更新（README/開発ドキュメント/この計画の反映）

---

## 12. 影響範囲と互換性

- CLI オプション/出力形式: 従来通り（非互換変更なし）
- ライブラリ API: 新規導入（外部利用者にとっては追加の恩恵）
- 構成ファイル: 既存キーはそのまま。未知セクションは `extra_modules` に保持

---

## 13. リスクと緩和策

- リスク: ワークスペース化に伴うパス・テストの崩れ
  - 緩和: 段階導入、CI で逐次検証
- リスク: エラー/ログスタックの二重化
  - 緩和: コアは `thiserror`、CLI は `anyhow` に統一、`tracing` で出力統一
- リスク: feature gate によるビルド分岐の複雑化
  - 緩和: 既定は単純構成（`git` off, `parallel` off）、CLI プロファイルのみ on

---

## 14. 実装タスク（チェックリスト）

- [ ] ワークスペース化 (`Cargo.toml` ルート + `crates/` 生成)
- [ ] `claude-code-statusline-core` へ型/ユーティリティ移行（`types/*`, `parser`, `style`, `timeout`, `messages`）
- [ ] `Engine`/`Registry`/`ConfigProvider` の追加
- [ ] 既存モジュールを `Registry` 登録方式に移行（内部は最小変更）
- [ ] `generate_prompt` → `Engine::render` 移動
- [ ] CLI を `Engine` 呼び出しへ簡素化、`tracing-subscriber` 初期化
- [ ] `git2` を feature gate 化、CLI 側で有効化
- [ ] `thiserror` 導入とコアのエラー整理
- [ ] ドキュメント更新（README, docs/configuration.md, docs/development.md）
- [ ] テスト調整と新規追加（core/cli）

---

## 15. スケジュール案

- W1: フェーズ 1 実装 + テスト通過
- W2: フェーズ 2 実装（extra_modules, registry 初期化, feature gate）
- W3: フェーズ 3 実装（thiserror/tracing）+ ドキュメント更新
- W4: 予備（並列化 PoC or フォーマッタ拡張検討）

---

## 16. ロールバック戦略

- フェーズ毎に小さな PR とし、問題発生時は直近フェーズ単位で巻き戻し
- 既存の単一クレート構成に戻すスクリプトを用意（必要なら）

---

## 17. オープン事項（要オーナー確認）

- 外部モジュールの設定は claude-code-statusline 既定の TOML と同ファイルで運用する想定で良いか
- トップレベル format の仕様拡張（連結表記）は後方互換のため別フェーズで良いか
- ログは `tracing` への一本化で問題ないか（既存 DebugLogger はラッパに留める/削除）

---

以上。
