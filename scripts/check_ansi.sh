#!/usr/bin/env bash
set -euo pipefail

# Usage:
#   bash scripts/check_ansi.sh [CWD]
#   USE_INSTALLED=1 bash scripts/check_ansi.sh [CWD]
#   MODEL_DISPLAY_NAME='Opus4.1' bash scripts/check_ansi.sh

CWD="${1:-$PWD}"
MODEL_DISPLAY_NAME="${MODEL_DISPLAY_NAME:-Opus4.1}"

if [[ "${USE_INSTALLED:-0}" == "1" ]] && command -v beacon >/dev/null 2>&1; then
  RUN=(beacon)
else
  RUN=(cargo run -p beacon -q)
fi

JSON=$(cat <<EOF
{
  "hook_event_name": "Status",
  "session_id": "debug-ansi",
  "transcript_path": null,
  "cwd": "${CWD}",
  "model": {"id": "claude-opus", "display_name": "${MODEL_DISPLAY_NAME}"},
  "workspace": {"current_dir": "${CWD}", "project_dir": "${CWD}"},
  "version": "1.0.0",
  "output_style": null
}
EOF
)

tmpdir=$(mktemp -d)
stdout_bin="$tmpdir/stdout.bin"
stderr_txt="$tmpdir/stderr.txt"

printf '%s' "$JSON" | "${RUN[@]}" >"$stdout_bin" 2>"$stderr_txt" || true
code=$?

STDOUT_BIN="$stdout_bin" STDERR_TXT="$stderr_txt" CODE="$code" python3 - <<'PY'
import os, pathlib
out = pathlib.Path(os.environ["STDOUT_BIN"]).read_bytes()
err = pathlib.Path(os.environ["STDERR_TXT"]).read_text(errors="replace")
print("exit:", os.environ.get("CODE"))
print("len:", len(out))
print(out[:200])
print("ESC count:", out.count(b"\x1b["))
print("hex[:200]:", out[:200].hex())
if len(out) == 0 or os.environ.get("CODE") != "0":
    print("--- stderr ---\n" + err)
PY

rm -rf "$tmpdir"

