#!/bin/sh
set -e

echo "🔍 Running pre-commit checks..."

echo "📝 Checking code format..."
if ! cargo fmt -- --check; then
    echo "❌ Code is not formatted!"
    echo "💡 Run 'cargo fmt' to fix formatting"
    exit 1
fi

echo "🔎 Running clippy..."
if ! cargo clippy -- -D warnings; then
    echo "❌ Clippy found issues!"
    echo "💡 Run 'cargo clippy' to see details"
    exit 1
fi

echo "🧪 Running tests..."
if ! cargo test --quiet; then
    echo "❌ Tests failed!"
    echo "💡 Run 'cargo test' to see details"
    exit 1
fi

echo "✅ All pre-commit checks passed!"
