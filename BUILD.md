BUILD - Build and install the Python extension (slugify-rs)
=========================================================

This document explains how to build and install the Rust-based
`slugify-rs` Python extension module. It is aimed at users who are
familiar with basic command-line usage but not necessarily with Rust
tooling.

Prerequisites
-------------

- Rust toolchain (install via `rustup`). Recommended Rust 1.56 or
  newer.
- Python 3.8 or newer and `pip`.
- `maturin` to build Python wheels. Install inside a virtualenv.
- System build tools on Linux: `build-essential`, `pkg-config`,
  `python3-dev`, and `libssl-dev`.

Quick setup (Linux example)
---------------------------

```bash
# System deps (Debian/Ubuntu)
sudo apt update
sudo apt install -y build-essential pkg-config python3-dev libssl-dev

# Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env

# Python venv and maturin
python3 -m venv .venv
. .venv/bin/activate
pip install --upgrade pip
pip install maturin
```

Build and develop
-----------------

Install the extension into the active venv for development:

```bash
maturin develop --release --features python
```

This builds the extension and installs it in the venv. You can now run
Python code that imports `python_slugify_pi`.

Build a distributable wheel
---------------------------

```bash
maturin build --release -i python --features python
pip install target/wheels/*.whl
```

`maturin build` produces wheels in `target/wheels/`. Use `pip` to
install the wheel into any Python environment.

Notes on `maturin` options
--------------------------

- `-i` or `--interpreter` specifies the Python interpreter to use. Use
  this option when the default interpreter is not the one from your
  venv.
- `--manylinux` instructs `maturin` to build manylinux-compatible
  wheels (requires Docker and proper setup). Refer to `maturin` docs.

Troubleshooting
---------------

- Linker errors (undefined references or failure to link): install
  system build deps (`build-essential`, `python3-dev`) and retry.
- `maturin` cannot find the right Python: pass `-i $(which python)` or
  `-i /path/to/venv/bin/python`.
- macOS code signing or SDK errors: ensure Xcode command line tools are
  installed (`xcode-select --install`).
- If wheel tests fail on CI, ensure you build manylinux wheels for
  PyPI (see `maturin` manylinux docs).

Testing
-------

Rust unit tests:

```bash
cargo test -p slugify-rs --lib
```

Python tests (after installing with `maturin develop` or pip):

```bash
pytest tests/python
```

Example usage
-------------

```python
import python_slugify_pi

print(python_slugify_pi.slugify("C'est déjà l'été!"))
print(python_slugify_pi.slugify("I ♥ 🚀", transliterate_icons=True))
```

Publishing to PyPI
------------------

Use `maturin publish` in a properly configured environment or build
manylinux wheels and upload them using `twine`.

Further reading
---------------

- maturin documentation: https://maturin.rs/
- PyO3 documentation: https://pyo3.rs/
