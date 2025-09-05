## Pastel Powerline

Powerline の「セグメント背景＋区切り矢印（）」を淡い配色で実現するプリセット例です。各セグメントは必ず `fg:` と `bg:` の両方を指定し、セグメント間の矢印は「前の背景＝前景」「次の背景＝背景」でブリッジします。

ポイント:
- `[$content]($style)` を複数連ねて、ブロックと矢印を個別に着色
- セグメント本体は `fg:black`（薄色背景に対して可読性を確保）
- 例の配色は一例（Hex）です。お好みで変更してください

```toml
format = "\
[$path ](fg:black bg:#a8d8ef)\
[](fg:#a8d8ef bg:#bde5b8)\
[ $symbol $branch ](fg:black bg:#bde5b8)\
[](fg:#bde5b8 bg:#f8e3a1)\
[ $all_status$ahead_behind ](fg:black bg:#f8e3a1)\
[](fg:#f8e3a1 bg:#e4bee6)\
[ $symbol$model ](fg:black bg:#e4bee6)"

[directory]
# directory モジュールは $path を提供
# セグメント本体の style は format 側で直接指定するため、ここは未使用でも可
style = ""
format = "[$path ]($style)"  # モジュール単体でのプレビュー用（本番は上記 format を使用）

[git_branch]
style = ""

[git_status]
style = ""

[claude_model]
style = ""
```

配色例（Hex）
- directory: `bg:#a8d8ef`
- git_branch: `bg:#bde5b8`
- git_status: `bg:#f8e3a1`
- claude_model: `bg:#e4bee6`

注意:
- 現状のコアは「前セグメントの色を自動で引き継ぐ」機能は未実装です（仕様の Future Work）。本プリセットは上記のように矢印ごとに明示的に `fg:`/`bg:` を指定して再現します。
