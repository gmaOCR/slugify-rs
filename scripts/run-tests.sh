#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "${BASH_SOURCE[0]}")/.."
echo "Working directory: $(pwd)"
echo "Cargo metadata:"
cargo metadata --no-deps --format-version 1

echo "Running cargo test --verbose..."
cargo test --verbose

echo "Listing test binaries in target/debug/deps:"
ls -la target/debug/deps | grep slugify_rs || true

echo "Done." 
