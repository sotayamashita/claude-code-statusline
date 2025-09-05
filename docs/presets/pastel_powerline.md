## Pastel Powerline (characters-only)

淡い色調の Powerline 記号を用いた最小構成のプリセット例です。背景色は使わず、明示的な前景色（`fg:`）のみで配色します。

ポイント:
- グローバル `format` はモジュールを空白区切りで直列化
- 各モジュールの `format` は `[$content]($style)` でラップし、`$style` はモジュール設定の `style` を参照
- `fg:` で前景色を明示（将来、背景を付けるバリアントへ拡張しやすい）

```toml
format = "$directory $git_branch $git_status $claude_model"

[directory]
style = "bold fg:cyan"
format = "[$path ]($style)"

[git_branch]
style = "bold fg:green"
format = "[$symbol $branch ]($style)"

[git_status]
style = "bold fg:yellow"
format = "[[$all_status$ahead_behind] ]($style)"

[claude_model]
style = "bold fg:magenta"
format = "[$symbol$model]($style)"
```

備考:
- `fg:`/`bg:` は 8色/明色/8bit/Hex をサポートします（例: `fg:#bf5700`）。
- 背景色を使うバリアントを作る場合は `bg:` を追加するだけで済みます（例: `bg:bright-black`）。
