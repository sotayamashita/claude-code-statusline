[![CI](https://github.com/sotayamashita/beacon/actions/workflows/ci.yml/badge.svg)](https://github.com/sotayamashita/beacon/actions/workflows/ci.yml)

# Beacon

Claude Code の Status Line を生成する軽量 CLI。

- 入力: Claude Code が標準入力に流す JSON（セッション/モデル/カレントディレクトリ等）
- 出力: 1 行のステータス表示（改行なし）。`$directory` や `$git_branch` などのモジュールを組み合わせて表示

主な特徴:
- `$directory`, `$claude_model`, `$git_branch`, `$git_status` をサポート
- `~/.config/beacon.toml` による設定（フォーマットやスタイル、タイムアウト）
- 既定でグローバルタイムアウト（ハング防止）と簡易キャッシュ（同一実行内の Git/Dir 再計算抑止）

## インストール / ビルド

前提: Rust toolchain（stable）

```
cargo build --release
```

開発向け: フォーマット／Lint／テスト

```
cargo fmt
cargo clippy -- -D warnings
cargo test
```

Git フックの導入（任意）:

```
make install-hooks
```

## クイックスタート（実行例）

`stdin` に JSON を渡して実行します。

```
echo '{"cwd":"/tmp","model":{"id":"claude-opus","display_name":"Opus"}}' | cargo run -q
```

設定ファイルが無い場合は既定値で動作します。表示は 1 行（末尾に改行はありません）。

## 設定（`~/.config/beacon.toml`）

トップレベルの主要項目:

```toml
# 出力フォーマット（空白区切りでモジュールを展開）
format = "$directory $git_branch $git_status $claude_model"

# モジュール実行タイムアウト（ミリ秒）。範囲: 50..=600000
command_timeout = 500

# 追加のデバッグログを stderr へ出力
debug = false
```

### モジュール設定

- `directory`

```toml
[directory]
format = "[$path]($style)"
style = "bold cyan"
truncation_length = 3
truncate_to_repo = true
disabled = false
```

- `claude_model`

```toml
[claude_model]
format = "[$symbol$model]($style)"
style = "bold yellow"
symbol = "<"
disabled = false
```

- `git_branch`

```toml
[git_branch]
format = "[🌿 $branch]($style)"
style = "bold green"
symbol = "🌿"
disabled = false
```

- `git_status`（Starship に準拠した最小構成）

```toml
[git_status]
format = "([[$all_status$ahead_behind]]($style) )"
style = "bold red"
disabled = false

  [git_status.symbols]
  conflicted = "="
  stashed    = "$"
  deleted    = "✘"
  renamed    = "»"
  modified   = "!"
  typechanged= ""
  staged     = "+"
  untracked  = "?"
  ahead      = "⇡"
  behind     = "⇣"
  diverged   = "⇕"
```

### スタイル指定

サポート済みトークン（空白区切り）:

- 太字/装飾: `bold`, `italic`, `underline`
- 色: `black`, `red`, `green`, `yellow`, `blue`, `magenta`, `cyan`, `white`

テンプレート `[$text]($style)` は `style` が `$style` の場合、該当モジュール設定の `style` を適用します。

## タイムアウトとキャッシュ

- すべてのモジュールは `should_display` と `render` をグローバル `command_timeout` でラップします。時間超過したモジュールは「非表示」として扱われます。
- `Context` 内で Git リポジトリやディレクトリ走査結果を `OnceLock` でメモ化し、同一実行内での再計算を抑止します（プロセス終了で破棄）。

## 代表モジュールの動作

- `$directory`: `HOME` を `~` に短縮表示します。
- `$git_branch`: ブランチ名（detached HEAD の場合は短縮 SHA）。Git2 が使えない環境でも `git` コマンドでフォールバック。
- `$git_status`: 作業ツリーの状態を最小限の記号+件数で集合表示し、`ahead/behind/diverged` を追加表示します。
- `$claude_model`: モデル名（例: `Sonnet 4` → `Sonnet4` のように数字直前の単一空白を圧縮）。

より詳しい設定例や仕様は `docs/configuration.md` を参照してください。

## 開発者向け（コントリビュート）

- モジュールの追加方法、ディスパッチ登録、テスト方針は `docs/development.md` に記載しています。
- 開発コマンド: `cargo fmt`, `cargo clippy -- -D warnings`, `cargo test`
- Git フック: `make install-hooks` で pre-commit（fmt, clippy, test）を導入可能
