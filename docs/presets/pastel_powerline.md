## Pastel Powerline

Powerline の「セグメント背景＋区切り矢印（）」を淡い配色で実現するプリセット例です。各セグメントは必ず `fg:` と `bg:` の両方を指定し、セグメント間の矢印は「前の背景＝前景」「次の背景＝背景」でブリッジします。

ポイント:
- `[$content]($style)` を複数連ねて、ブロックと矢印を個別に着色
- セグメント本体は `fg:black`（薄色背景に対して可読性を確保）
- 例の配色は一例（Hex）です。お好みで変更してください

```toml
format = """
[](#9A348E)\
$directory\
[ ](bg:#DA627D fg:#9A348E)\
$git_branch$git_status\
[ ](fg:#DA627D bg:#FCA17D)\
$claude_model\
[ ](fg:#FCA17D)\
"""

[directory]
style = "bg:#9A348E"
format = "[$path]($style)"
truncation_length = 3
truncation_symbol = "…/"

[git_branch]
symbol = ""
style = "bg:#DA627D"
format = "[$symbol $branch]($style)"

[git_status]
style = "bg:#DA627D"
format = "[$all_status$ahead_behind ]($style)"

[claude_model]
style = "bg:#FCA17D"
format = "[$model]($style)"
```

配色例（Hex）
- directory: `bg:#a8d8ef`
- git_branch: `bg:#bde5b8`
- git_status: `bg:#f8e3a1`
- claude_model: `bg:#e4bee6`

注意:
- 現状のコアは「前セグメントの色を自動で引き継ぐ」機能は未実装です（仕様の Future Work）。本プリセットは上記のように矢印ごとに明示的に `fg:`/`bg:` を指定して再現します。
