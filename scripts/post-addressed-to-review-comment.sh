#!/usr/bin/env bash
# Reply to a specific PR review comment with "Addressed in commit `<hash>`".
#
# Usage:
#   bash scripts/post-addressed-to-review-comment.sh \
#     "https://github.com/<owner>/<repo>/pull/<number>" \
#     <review_comment_url_or_id> \
#     <commit_hash>
#
# Examples:
#   bash scripts/post-addressed-to-review-comment.sh \
#     https://github.com/sotayamashita/beacon/pull/12 \
#     https://github.com/sotayamashita/beacon/pull/12#discussion_r2324015980 \
#     abc1234
#
#   bash scripts/post-addressed-to-review-comment.sh \
#     https://github.com/sotayamashita/beacon/pull/12 \
#     2324015980 \
#     abc1234
#
# Requirements:
#   - GitHub CLI `gh` (authenticated)
#
# Output:
#   Prints the JSON of the created reply comment (from REST API).

set -euo pipefail

usage() {
  echo "Usage: $0 <pr_url> <review_comment_url_or_id> <commit_hash>" >&2
}

if ! command -v gh >/dev/null 2>&1; then
  echo "Error: gh CLI not found. Install GitHub CLI and authenticate (gh auth login)." >&2
  exit 127
fi

if [[ $# -lt 3 ]]; then
  usage
  exit 2
fi

PR_URL="$1"
REF_ARG="$2"
COMMIT_HASH="$3"

# Parse owner/repo/number from PR URL
if [[ "$PR_URL" =~ ^https?://github\.com/([^/]+)/([^/]+)/pull/([0-9]+) ]]; then
  OWNER="${BASH_REMATCH[1]}"
  REPO="${BASH_REMATCH[2]}"
  PR_NUMBER="${BASH_REMATCH[3]}"
else
  echo "Error: invalid PR URL. Expected https://github.com/<owner>/<repo>/pull/<number>" >&2
  usage
  exit 2
fi

# Extract review comment ID
COMMENT_ID=""
if [[ "$REF_ARG" =~ ^https?://github\.com/[^/]+/[^/]+/pull/[0-9]+#discussion_r([0-9]+)$ ]]; then
  COMMENT_ID="${BASH_REMATCH[1]}"
elif [[ "$REF_ARG" =~ ^[0-9]+$ ]]; then
  COMMENT_ID="$REF_ARG"
else
  echo "Error: second argument must be a review comment URL or numeric comment ID" >&2
  usage
  exit 2
fi

# Compose message (single-line to keep it simple)
BODY="Addressed in commit \`$COMMIT_HASH\`"

# Create a reply to the specific review comment using the supported endpoint
# Note: GitHub REST v3 recommends POST /pulls/{pull_number}/comments with `in_reply_to`
gh api \
  -X POST \
  "repos/${OWNER}/${REPO}/pulls/${PR_NUMBER}/comments" \
  -f body="$BODY" \
  -F in_reply_to="$COMMENT_ID"
