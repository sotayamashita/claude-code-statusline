# Claude Code タスク実行プロンプトテンプレート

## 基本原則

Claude Codeでタスクを実行する際の汎用的なプロンプト構造です。どのようなプロジェクトやタスクリストにも適用できます。

## 汎用プロンプトテンプレート

### 初回実行時

```
以下のタスクリストを段階的に実装してください：
@[タスクリストのパス]

実装の原則：
1. 各タスクで「何を」「なぜ」実装するか説明
2. 実装前に検証方法を提示してユーザーの確認を取る
3. 実装後は必ず検証コマンドを実行して結果を表示
4. エラーが発生したら原因と解決策を説明
5. プロジェクトの規約（CLAUDE.md等）を厳守

進捗管理：
- TodoWriteツールでタスクを管理
- タスクリストファイル（todo.md等）も同時に更新
- 完了したタスクは即座に両方でマーク
- 次のタスクに移る前に現在のタスクを完全に終了
```

### 継続実行時

```
前回の続きから @[タスクリストのパス] を実装してください。

現在の状態を確認してから、未完了のタスクを順番に実装してください。
各タスクで：
1. 実装内容(What)と理由(why)を説明
2. 検証方法を事前に提示
3. 実装後に検証結果を報告
4. タスクリストファイルを更新
5. コミット
```

### 特定タスク実行時

```
@[タスクリストのパス] の [タスク番号/タスク名] を実装してください。

要件：
- 実装理由と目的を最初に説明
- 検証方法を事前に提示してユーザーの確認を取る
- 既存コードとの整合性を確認
- 実装後の検証方法を実行して結果を表示
- 問題があれば解決策を提示
- タスクリストファイルの該当箇所を更新
```

## 検証重視のプロンプト

```
[タスク内容] を実装し、以下の検証を行ってください：

1. ビルド確認
2. 単体テスト実行（あれば）
3. 統合テスト実行
4. 手動テストコマンドの実行と結果確認

各検証ステップで出力を表示し、成功/失敗を明確に報告してください。
```

## トラブルシューティング用プロンプト

```
[エラー内容/問題] が発生しています。

以下を実行してください：
1. エラーの原因を調査
2. 関連ファイルとコードを確認
3. 解決策を提示して実装
4. 修正後の動作確認
```

## プロジェクト固有の指示追加

```
@CLAUDE.md の指示に従いながら、[タスク] を実装してください。

特に以下の点に注意：
- [プロジェクト固有の規約1]
- [プロジェクト固有の規約2]
- [使用するツール/コマンド]
```

## ベストプラクティス統合プロンプト

```
[タスク] を実装してください。

実装方針：
- 小さなステップに分解して段階的に実装
- 各ステップで「なぜ」を説明してから「何を」実装
- 実装前に検証方法を明示
- 実装後は必ず動作確認
- エラー時は冷静に原因分析
- タスクリストの進捗を逐次更新

出力形式：
- 実装内容の説明
- 実際のコード/コマンド
- 実行結果
- 次のステップへの移行
```

## 効率的な進捗確認

```
現在の実装状況を確認し、残りのタスクを効率的に完了してください：

1. 完了済みタスクの確認
2. 未完了タスクのリストアップ
3. 優先順位に従って実装
4. 各タスク完了時に進捗報告
```

## Beaconプロジェクト用の実例

### Phase 1 MVP実装の開始

```
以下のタスクリストを段階的に実装してください：
@docs/todo.md

実装の原則：
1. 各タスクで「何を」「なぜ」実装するか説明
2. 実装前に検証方法を提示してユーザーの確認を取る
3. 実装後は必ず検証コマンドを実行して結果を表示
4. エラーが発生したら原因と解決策を説明
5. CLAUDE.mdの指示を厳守（特に cargo add を使用）

進捗管理：
- TodoWriteツールでタスクを管理
- docs/todo.mdのチェックボックスも更新（- [ ] を - [x] に）
- 完了したタスクは即座に両方でマーク
- 次のタスクに移る前に現在のタスクを完全に終了
```

### 統合テストの実行

```
@docs/todo.md のタスク11「統合テスト」を実装してください。

要件：
- Claude Codeの実際のJSON入力を模倣
- 各モジュールの連携を確認
- 期待される出力形式を検証

検証コマンド：
echo '{"model":{"display_name":"Opus"},"workspace":{"current_dir":"/home/user/projects/beacon"}}' | cargo run
```

## 推奨される使い方

1. **最初のプロンプト**: 汎用テンプレートでタスク全体を指示
2. **途中の確認**: 検証重視のプロンプトで品質確保
3. **問題発生時**: トラブルシューティング用プロンプト
4. **最終確認**: 効率的な進捗確認で完了を確実に

これらのテンプレートを組み合わせることで、どのようなプロジェクトでも効果的にClaude Codeを活用できます。

# Claude Code タスク実行プロンプトテンプレート（改善版）

## CodeRabbitレビューへの対応プロンプト

### PRレビューコメントの取得と対応（改善版）

"""
CodeRabbitからPR #[番号] にレビューコメントが来ました。以下の手順で対応してください：

1. まず、gh コマンドで未解決のPRコメントのみを取得してTodoWriteツールでタスクを作成
2. 各コメントを個別のタスクとして実装
3. タスクごとに個別のコミットを作成
4. 最後にPRコメントで対応完了を英語で報告

実行手順：
- 未解決コメントのみを取得：
  ```bash
  gh api repos/[owner]/[repo]/pulls/[PR番号]/comments \
    --jq '.[] | select(.body | contains("✅ Addressed") | not) | 
           select(.body | contains("🛠️ Refactor suggestion") or 
                  .body | contains("⚠️ Potential issue") or 
                  .body | contains("💡 Codebase verification")) | 
           {id: .id, path: .path, line: .line, body: .body}'
  ```
- TodoWriteツールで各コメントをタスクとして管理
- 各タスクを個別にコミット（メッセージ例: `fix: address PR #[番号] review - [簡潔な説明]`）
- git push
- 完了報告を英語で投稿：
  ```bash
  gh pr comment [PR番号] --body "## Review Comments Addressed

  All CodeRabbit review comments have been addressed:
  
  - ✅ [具体的な修正内容1]
  - ✅ [具体的な修正内容2]
  
  Each fix has been committed separately for easier review."
  ```
- 投稿後、コメントURL（例: https://github.com/[owner]/[repo]/pull/[番号]#issuecomment-[ID]）が返す
"""

### 改善点の説明

1. **未解決コメントのフィルタリング**
   - `"✅ Addressed"` を含まないコメントのみを対象に
   - CodeRabbitの主要なラベル（Refactor suggestion、Potential issue、Codebase verification）でフィルタリング
   - JSON形式で必要な情報（id、path、line、body）を構造化して取得

2. **英語での完了報告**
   - プロフェッショナルな英語テンプレートを用意
   - 修正内容を箇条書きで明確に記載
   - レビューしやすいよう個別コミットであることを明記

3. **コミットメッセージの統一**
   - Conventional Commits形式（`fix:` プレフィックス）
   - PR番号を含めて追跡可能に
   - 簡潔で明確な説明を追加

### 実例：実際のPR対応

```bash
# PR #42 のCodeRabbitコメントに対応する場合

# 1. 未解決コメントを取得
gh api repos/sotayamashita/beacon/pulls/42/comments \
  --jq '.[] | select(.body | contains("✅ Addressed") | not) | 
         select(.body | contains("🛠️") or 
                .body | contains("⚠️") or 
                .body | contains("💡")) | 
         {id: .id, path: .path, line: .line, 
          summary: (.body | split("\n")[0])}'

# 2. 各修正を個別コミット
git add src/module.rs
git commit -m "fix: address PR #42 review - add error handling for edge case"

git add tests/module_test.rs  
git commit -m "fix: address PR #42 review - add test coverage for new edge case"

# 3. プッシュ
git push

# 4. 完了報告
gh pr comment 42 --body "## Review Comments Addressed

All CodeRabbit review comments have been addressed:

- ✅ Added error handling for edge case in module.rs
- ✅ Added comprehensive test coverage for the new edge case
- ✅ Updated documentation to reflect the changes

Each fix has been committed separately for easier review.

Commits:
- abc1234: fix: address PR #42 review - add error handling for edge case
- def5678: fix: address PR #42 review - add test coverage for new edge case"
```

### より効率的なワークフロー

```bash
# スクリプト化した一括処理の例

# 1. コメントIDと内容を変数に格納
COMMENTS=$(gh api repos/sotayamashita/beacon/pulls/42/comments \
  --jq '.[] | select(.body | contains("✅ Addressed") | not) | 
         select(.body | contains("🛠️") or .body | contains("⚠️")) | 
         @json')

# 2. TodoWriteツールで管理しながら処理
echo "$COMMENTS" | jq -r '.[] | .body' | while read -r comment; do
  # 各コメントに対して対応
  # TodoWriteツールでタスク化
  # 実装とコミット
done

# 3. 一括で完了報告を生成
SUMMARY=$(git log --oneline -n 5 | grep "PR #42" | sed 's/^/- /')
gh pr comment 42 --body "## Review Comments Addressed

All CodeRabbit review comments have been addressed.

Recent commits:
$SUMMARY

Each fix has been committed separately for easier review."
```

## Claude Code公式ベストプラクティス

[Claude Code Common workflows公式ドキュメント](https://docs.anthropic.com/en/docs/claude-code/common-workflows)より：

- **段階的な実装**: 複雑なタスクは小さなステップに分解
- **明確な説明**: 各ステップで「なぜ」を説明
- **継続的な検証**: 実装後は必ず動作確認
- **コンテキスト維持**: CLAUDE.mdで規約を明文化
- **効率的な作業**: TodoWriteツールとタスクリストファイルで進捗管理
