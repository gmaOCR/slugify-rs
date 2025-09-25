Measuring coverage for slugify-rs (Python + Rust)
===============================================

This document explains how to measure test coverage for both the Python
integration (PyO3 extension) and the Rust crate itself. The repo uses
Codecov in CI; locally you can produce the same artifacts.

1) Python coverage (pytest + pytest-cov)

  # create a venv and install deps
  python -m venv .venv
  . .venv/bin/activate
  pip install --upgrade pip
  pip install maturin pytest pytest-cov coverage

  # build & install the extension into the venv
  .venv/bin/maturin develop --release --features python

  # run tests and generate XML
  pytest tests/python --cov=. --cov-report=xml:coverage.xml

  # quick textual report
  coverage report -m

  The produced `coverage.xml` can be uploaded to Codecov or inspected
  locally.

2) Rust coverage (cargo-llvm-cov)

  # install tooling
  rustup component add llvm-tools-preview
  cargo install cargo-llvm-cov

  # run coverage (this runs the test suite instrumented)
  cargo llvm-cov --package slugify-rs --lcov --output-path lcov.info

  # Alternatively use the convenience alias (excludes src/bin by default):
  # cargo coverage

  # inspect or upload lcov.info to Codecov
  # codecov supports lcov format

3) Reproduce CI locally (quick)

  # Run the Python steps (above) then the Rust coverage steps.

Notes
-----
- `pytest` measures only Python code. The Rust core is not visible to
  pytest coverage tooling, so to measure core coverage use `cargo
  llvm-cov`.
- On CI we upload both `coverage.xml` (Python) and `lcov.info` (Rust)
  to Codecov to get a combined view.
