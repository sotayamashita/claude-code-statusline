#!/usr/bin/env bash
# Fetch unresolved, non-outdated CodeRabbit review comments from a PR URL and output as JSON.
#
# Usage:
#   bash scripts/get-coderabbit-unresolved-reviews.sh https://github.com/<owner>/<repo>/pull/<number>
#
# Requirements:
#   - GitHub CLI `gh` (authenticated; GH_HOST respected if configured)
#
# Output:
#   JSON array of comment objects with fields: author, created_at, path, url, body
#   Notes:
#     - Only comments from unresolved AND non-outdated review threads are included.
#     - Comments authored by CodeRabbit (case-insensitive match) are selected.

set -euo pipefail

usage() {
  echo "Usage: $0 https://github.com/<owner>/<repo>/pull/<number>" >&2
}

if ! command -v gh >/dev/null 2>&1; then
  echo "Error: gh CLI not found. Install GitHub CLI and authenticate (gh auth login)." >&2
  exit 127
fi

if [[ $# -ne 1 ]]; then
  usage
  exit 2
fi

PR_URL="$1"

# Accept forms like:
# - https://github.com/owner/repo/pull/123
# - http://github.com/owner/repo/pull/123/files (the extras after number are ignored)
if [[ "$PR_URL" =~ ^https?://github\.com/([^/]+)/([^/]+)/pull/([0-9]+) ]]; then
  OWNER="${BASH_REMATCH[1]}"
  NAME="${BASH_REMATCH[2]}"
  NUMBER="${BASH_REMATCH[3]}"
else
  echo "Error: invalid PR URL. Expected https://github.com/<owner>/<repo>/pull/<number>" >&2
  usage
  exit 2
fi

# GraphQL query to obtain unresolved review threads and filter CodeRabbit comments
read -r -d '' GQL <<'GRAPHQL' || true
query($owner:String!, $name:String!, $number:Int!) {
  repository(owner: $owner, name: $name) {
    pullRequest(number: $number) {
      reviewThreads(first: 100) {
        nodes {
          isResolved
          isOutdated
          comments(first: 100) {
            nodes {
              id
              author { login }
              url
              path
              diffHunk
              createdAt
              body
            }
          }
        }
      }
    }
  }
}
GRAPHQL

# Execute query and transform to JSON list of unresolved CodeRabbit comments
gh api graphql \
  -f owner="$OWNER" \
  -f name="$NAME" \
  -F number="$NUMBER" \
  -f query="$GQL" \
  --jq '[
    .data.repository.pullRequest.reviewThreads.nodes[]
    | select((.isResolved == false) and (.isOutdated == false))
    | .comments.nodes[]
    | select(.author.login | test("coderabbit"; "i"))
    | {
        author: .author.login,
        created_at: .createdAt,
        path: (.path // null),
        url: .url,
        body: .body
      }
  ]'
