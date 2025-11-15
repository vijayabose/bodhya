#!/bin/bash
# Bodhya Quality Gate Script
# This script must pass before any commit/merge

set -e

echo "================================"
echo "  Bodhya Quality Gate Check"
echo "================================"
echo ""

echo "[1/4] Running cargo fmt check..."
cargo fmt --check
echo "✓ Formatting check passed"
echo ""

echo "[2/4] Running cargo clippy..."
cargo clippy --all-targets --all-features -- -D warnings
echo "✓ Clippy check passed"
echo ""

echo "[3/4] Running cargo test..."
cargo test --all --all-features
echo "✓ All tests passed"
echo ""

echo "[4/4] Running cargo audit (optional)..."
cargo audit || echo "⚠ Cargo audit found issues (non-blocking)"
echo ""

echo "================================"
echo "  ✅ All checks passed!"
echo "================================"
