# Phase 1 リファクタリングタスク

> Phase 1の実装完了後に取り組むべき品質向上のためのリファクタリングタスク

## 🔴 優先度高（Phase 1の品質向上に必須）

### 1. モジュールシステムの改善

**なぜ**: 現在の実装は各モジュールをmain.rsで直接インスタンス化していて、拡張性が低い

**何を**: Starshipのような中央ディスパッチャーパターンを実装

```rust
// 現在: main.rsで直接モジュール生成
let dir_module = DirectoryModule::from_context(context);

// リファクタリング後: modules/mod.rsに中央ハンドラ
pub fn handle_module(name: &str, context: &Context) -> Option<Box<dyn Module>> {
    match name {
        "directory" => Some(Box::new(DirectoryModule::from_context(context))),
        "claude_model" => Some(Box::new(ClaudeModelModule::from_context(context))),
        _ => None,
    }
}
```

**TypeScriptで例えると**: Factory Patternで、文字列からクラスインスタンスを動的生成するイメージ

### 2. Moduleトレイトの改善

**なぜ**: 現在のトレイトが設定とコンテキストを受け取れない

**何を**: render()メソッドの引数を追加

```rust
// 現在
pub trait Module {
    fn render(&self) -> String;
}

// リファクタリング後
pub trait Module {
    fn render(&self, context: &Context, config: &dyn ModuleConfig) -> String;
}
```

**TypeScriptで例えると**: インターフェースのメソッドシグネチャに必要な引数を追加

### 3. フォーマット文字列パーサーの実装

**なぜ**: 現在は固定されたモジュール順序。設定ファイルの`format`フィールドが使われていない

**何を**: 設定の`format = "$directory $claude_model"`を実際に解析して使用

```rust
// 新規実装が必要
pub fn parse_format(format: &str, context: &Context) -> Vec<String> {
    // $directory -> DirectoryModuleの出力に置換
    // $claude_model -> ClaudeModelModuleの出力に置換
}
```

## 🟡 優先度中（Phase 1の保守性向上）

### 4. エラーハンドリングの改善

**なぜ**: 現在は単純にエラーメッセージを出力するだけ

**何を**: より優雅なフォールバック

```rust
// 現在
Err(e) => {
    print!("Failed to build status line due to invalid json");
}

// リファクタリング後: 部分的な成功を許可
match parse_claude_input(&buffer) {
    Ok(input) => { /* ... */ },
    Err(_) => {
        // 最小限の情報で継続
        let fallback_prompt = "$ "; // シンプルなプロンプト
        print!("{}", fallback_prompt);
    }
}
```

### 5. Configのモジュール設定分離

**なぜ**: 現在はConfig構造体に全モジュールの設定が直接定義されている

**何を**: 各モジュールが自分の設定型を持つ

```rust
// modules/directory.rs
#[derive(Deserialize, Default)]
pub struct DirectoryConfig {
    pub format: String,
    pub style: String,
    pub truncation_length: usize,
}

impl ModuleConfig for DirectoryConfig {}
```

**TypeScriptで例えると**: 各コンポーネントが自分のPropsインターフェースを定義

### 6. テストヘルパーの追加

**なぜ**: 現在のテストは繰り返しが多く、設定しづらい

**何を**: Starshipのような`TestRenderer`パターン

```rust
#[cfg(test)]
struct TestRenderer {
    context: Context,
    config: Config,
}

impl TestRenderer {
    fn new(module: &str) -> Self { /* ... */ }
    fn with_cwd(mut self, path: &str) -> Self { /* ... */ }
    fn render(&self) -> Option<String> { /* ... */ }
}
```

## 実装順序

1. **まずModuleトレイトとディスパッチャー改善**（優先度高 1,2）
   - 今後の拡張性の基盤となる
   
2. **次にフォーマットパーサー**（優先度高 3）
   - 設定ファイルの`format`フィールドを活用
   
3. **その後エラーハンドリングとテスト改善**（優先度中 4,6）
   - 品質と保守性の向上
   
4. **最後に設定分離**（優先度中 5）
   - Phase 2でモジュール追加時に便利

## 注意事項

- これらのリファクタリングはPhase 1の基本機能が動作していることを前提とする
- 各リファクタリングは独立して実施可能
- Rust初心者でも理解しやすいように段階的に実装
- TypeScriptの知識を活用してパターンを理解

## 成果物

リファクタリング完了後、以下が改善される：

- **拡張性**: 新しいモジュールの追加が容易に
- **保守性**: テストが書きやすく、コードが理解しやすい
- **設定の活用**: TOMLファイルの設定が実際に使われる
- **エラー処理**: より堅牢なフォールバック機構
