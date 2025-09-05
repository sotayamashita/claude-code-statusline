#!/usr/bin/env bash
set -euo pipefail

echo "🔍 Running pre-commit checks..."

echo "📝 Checking code format (workspace)..."
if ! cargo fmt --all -- --check; then
  echo "❌ Code is not formatted!"
  echo "💡 Run 'cargo fmt --all' to fix formatting"
  exit 1
fi

echo "🔎 Running clippy (deny warnings, locked, all targets)..."
if ! cargo clippy --locked --workspace --all-targets -- -D warnings; then
  echo "❌ Clippy found issues!"
  echo "💡 Run 'cargo clippy --workspace --all-targets' to see details"
  exit 1
fi

echo "🧪 Running tests (workspace, locked)..."
if ! cargo test --locked --workspace -- --nocapture; then
  echo "❌ Tests failed!"
  echo "💡 Run 'cargo test --workspace -- --nocapture' to see details"
  exit 1
fi

echo "🏗️  Building (release, workspace, locked)..."
if ! cargo build --locked --workspace --release --verbose; then
  echo "❌ Build failed!"
  echo "💡 Run 'cargo build --locked --workspace --release --verbose' to see details"
  exit 1
fi

echo "📚 Building docs (locked, no-deps)..."
if ! cargo doc --locked --no-deps; then
  echo "❌ Docs build failed!"
  echo "💡 Run 'cargo doc --locked --no-deps' to see details"
  exit 1
fi

echo "✅ All pre-commit checks passed!"
