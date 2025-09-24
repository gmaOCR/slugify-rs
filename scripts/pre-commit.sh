#!/usr/bin/env bash
set -euo pipefail

cmd=${1:-all}

echo "[pre-commit] running: ${cmd}"

case "$cmd" in
  fmt)
    cargo fmt --all -- --check
    ;;
  clippy)
    cargo clippy --all-targets --all-features -- -D warnings
    ;;
  audit)
    cargo audit
    ;;
  udeps)
    cargo udeps --all-targets
    ;;
  test)
    cargo test --lib
    ;;
  all)
    cargo fmt --all -- --check
    cargo clippy --all-targets --all-features -- -D warnings
    # optionally run udeps/audit but don't block if udeps isn't installed
    if command -v cargo-udeps >/dev/null 2>&1; then
      cargo udeps --all-targets || true
    fi
    if command -v cargo-audit >/dev/null 2>&1; then
      cargo audit || true
    fi
    # run the library tests only (faster than full workspace)
    cargo test --lib --quiet
    ;;
  *)
    echo "Unknown command: $cmd" >&2
    exit 2
    ;;
esac

echo "[pre-commit] ${cmd} done"
