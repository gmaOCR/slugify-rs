#!/usr/bin/env bash
set -euo pipefail

# Build and test script for the PyO3 wheel. Creates a venv, builds a single
# PyO3 wheel, installs only that wheel, and runs a quick import test.

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

# Create venv
python3 -m venv .venv
. .venv/bin/activate
pip install --upgrade pip
pip install maturin
pip install pytest

# Clean previous wheels to avoid pip resolution conflicts
rm -rf target/wheels
mkdir -p target/wheels

# Build wheel with PyO3 feature enabled
maturin build --release -i python --features python

# Only install the newly created wheel (the cp<XY> wheel), choose newest file
WHEEL=$(ls -1t target/wheels/*.whl | head -n1)
if [ -z "$WHEEL" ]; then
  echo "No wheel found in target/wheels"
  exit 1
fi

# Uninstall any previously installed package with same name
pip uninstall -y slugify-rs || true

# Install the wheel
pip install "$WHEEL"

# Quick import test
pytest -q tests/python

echo "Build+test completed successfully"
