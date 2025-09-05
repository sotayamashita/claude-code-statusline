## Configuration

Beacon は `~/.config/beacon.toml` から設定を読み込みます（存在しない場合は既定値）。本書は主要オプションと各モジュールのカスタマイズ方法をまとめたものです。

### Top-level

```toml
# 出力フォーマット（空白区切りでモジュールを展開）
format = "$directory $git_branch $git_status $claude_model"

# モジュール実行タイムアウト（ミリ秒）。範囲: 50..=600000
command_timeout = 500

# 追加のデバッグログを stderr へ出力
debug = false
```

注意:
- `command_timeout` はすべてのモジュールの `should_display`/`render` を包括的にラップします。時間超過は「そのモジュールは表示しない」扱いです。
- `debug` 有効時は詳細ログを stderr へ出力します（機密情報のログ出力は避けてください）。

### Module: `directory`

```toml
[directory]
format = "[$path]($style)"
style = "bold cyan"
truncation_length = 3
truncate_to_repo = true
disabled = false
```

Tokens: `$path`

振る舞い:
- `HOME` 配下は `~` へ短縮表示。
- `format` の `[$text]($style)` 構文で ANSI スタイルを付与可能（`$style` はこのモジュールの `style` を指します）。
 - `truncate_to_repo = true` のとき、ディレクトリが Git リポジトリ配下であれば、`$path` を「`<repo-name>/<relative>`」形式（リポジトリ名 + リポジトリ内相対パス）で表示します。
   - 検出順序: `feature = "git"` 有効時は `git2` の `workdir()` を優先。見つからない場合や `git` 無効時は、カレントから親に向かって `.git` ディレクトリを探索して推定します。
   - `truncation_length` は表示セグメント数の上限です。常に先頭のリポジトリ名を保持し、残りは末尾のディレクトリから詰めて表示します（例: `truncation_length = 2` → `repo/last`）。
   - リポジトリ外ではこのオプションは無視され、ホーム短縮のみの通常表示になります。

例:

```
# 例1: repo 直下
truncate_to_repo = true
truncation_length = 3
# パス: /path/to/repo -> 表示: repo

# 例2: repo/src/module（セグメント3）
truncate_to_repo = true
truncation_length = 3
# パス: /path/to/repo/src/module -> 表示: repo/src/module

# 例3: 深い階層（末尾優先で短縮）
truncate_to_repo = true
truncation_length = 2
# パス: /path/to/repo/a/b/c/d -> 表示: repo/d
```

### Module: `claude_model`

```toml
[claude_model]
format = "[$symbol$model]($style)"
style  = "bold yellow"
symbol = "<"
disabled = false
```

Tokens: `$model`, `$symbol`

振る舞い:
- モデル名の数字直前の単一空白を除去（例: `Sonnet 4` → `Sonnet4`）。

### Module: `git_branch`

```toml
[git_branch]
format = "[🌿 $branch]($style)"
style  = "bold green"
symbol = "🌿"
disabled = false
```

Tokens: `$branch`, `$symbol`

振る舞い:
- ブランチ名を表示。detached HEAD の場合は短縮 SHA（7〜8 桁）。
- Git2 が失敗した環境では `git` コマンドへフォールバックします。
 - ライブラリ利用時（`beacon-core` を直接依存する場合）にこのモジュールを使うには
   crate の feature `git` を有効にしてください。CLI バイナリは既定で有効です。

### Module: `git_status`

```toml
[git_status]
format = "([[$all_status$ahead_behind]]($style) )"
style  = "bold red"
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

Tokens: `$all_status`, `$ahead_behind`

振る舞い（最小仕様）:
- `$all_status` は以下の順序の集合表示: `conflicted stashed deleted renamed modified typechanged staged untracked`
- 各セグメントは `symbol + 件数`（件数 0 は非表示）
- `$ahead_behind` は upstream が設定されているとき `⇡n` / `⇣n` / `⇕` を表示
 - ライブラリ利用時（`beacon-core` を直接依存する場合）にこのモジュールを使うには
   crate の feature `git` を有効にしてください。CLI バイナリは既定で有効です。

### ANSI スタイル指定

サポート済みトークン（空白区切り）:
- 装飾: `bold`, `italic`, `underline`
- 色: `black`, `red`, `green`, `yellow`, `blue`, `magenta`, `cyan`, `white`

`[$text]($style)` 構文で装飾を付けられます。`($style)` が `$style` の場合は、そのモジュール設定の `style` 値を適用します。

### 例: 最小構成

```toml
format = "$directory $claude_model"

[directory]
style = "bold cyan"

[claude_model]
style = "bold yellow"
symbol = "<"
```
