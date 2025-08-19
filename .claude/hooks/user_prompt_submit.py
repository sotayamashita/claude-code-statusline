#!/usr/bin/env python3

import sys
import json

try:
    input_data = json.load(sys.stdin)
except json.JSONDecodeError as e:
    print(f"Error: Invalid JSON input: {e}", file=sys.stderr)
    sys.exit(1)

# Get prompt from input_data
prompt = input_data.get("prompt", "")

# Append ultrathink message if prompt ends with -u
if prompt.strip().endswith("-u"):
    print("\n\nUse the maximum amount of ultrathink. Take all the time you need. It's much better if you do too much research and thinking than not enough.")

sys.exit(0)
