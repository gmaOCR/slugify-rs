## slugify-rs

Rust implementation of slugify with optional PyO3 bindings.

[![status-image]][status-link]
[![version-image]][version-link]
[![coverage-image]][coverage-link]


This crate provides a fast Rust core for string slugification and an
optional Python extension module (via PyO3) for use from Python
projects. The Python module is exposed as `python_slugify_pi` when the
`python` Cargo feature is enabled.

## Quick start

If you only use Rust, add the crate to your `Cargo.toml` and call
`slugify` from Rust. If you want a Python extension, build a wheel with
`maturin` (instructions below).

## Prerequisites

- Rust toolchain: install `rustup` (Rust 1.56+ recommended):

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

- Python 3.8+ and a working `pip`.
- On Debian/Ubuntu install system build deps:

```bash
sudo apt update && sudo apt install -y build-essential pkg-config python3-dev libssl-dev
```

- Install `maturin` in your Python virtualenv to build wheels:

```bash
python -m venv .venv
. .venv/bin/activate
pip install --upgrade pip
pip install maturin
```

Note: macOS users should ensure Xcode command line tools are installed.

## Build & install Python wheel (recommended)

For development (install directly into the active venv):

```bash
maturin develop --release --features python
```

To build a wheel for distribution and then install it:

```bash
maturin build --release -i python --features python
pip install target/wheels/*.whl
```

`develop` installs the built extension into your venv for quicker
iteration. `build` produces a wheel in `target/wheels/` for distribution.

Note: if `maturin` picks the wrong Python interpreter, pass the exact
interpreter path with `-i`, for example `-i /home/user/.venv/bin/python`.

Quick test (after `maturin develop` or installing the wheel):

```bash
python -c "import python_slugify_pi; print(python_slugify_pi.slugify(\"C'est déjà l'été!\"))"
```

Minimal GitHub Actions snippet for building wheels with `maturin`:

```yaml
# .github/workflows/python-wheels.yml
name: Build Python wheels

on: [push, pull_request]

jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: actions-rs/toolchain@v1
        with:
          profile: minimal
          toolchain: stable
      - name: Set up Python
        uses: actions/setup-python@v4
        with:
          python-version: '3.x'
      - name: Install maturin
        run: pip install maturin
      - name: Build wheels
        run: maturin build --release --manylinux 2014 --features python
      - name: Upload wheels
        uses: actions/upload-artifact@v4
        with:
          name: wheels
          path: target/wheels/*.whl
```

## Example (Python)

```python
import python_slugify_pi

print(python_slugify_pi.slugify("C'est déjà l'été!"))

# Control icon transliteration (default False for Python parity)
print(python_slugify_pi.slugify("I ♥ 🚀"))
print(python_slugify_pi.slugify("I ♥ 🚀", transliterate_icons=True))
```

## Cargo.toml snippet

Add the optional `python` feature and set the crate type when building
for Python (example):

```toml
[package]
name = "slugify-rs"
version = "0.1.0"

[lib]
crate-type = ["cdylib"]

[features]
default = []
python = ["pyo3/extension-module"]

[dependencies]
pyo3 = { version = "0.26", optional = true, features = ["extension-module"] }
deunicode = "*"
```

When building with `maturin`, `--features python` will enable the
PyO3 extension configuration.

## Running tests

Rust unit tests:

```bash
cargo test -p slugify-rs --lib
```

Python integration tests (after installing the wheel or using
`maturin develop`):

```bash
pytest tests/python
```

## Troubleshooting

- Linker errors on Linux: ensure `build-essential` and `python3-dev` are
  installed.
- If `maturin` complains about the Python interpreter, pass `-i` with
  the interpreter executable or path used to create the venv, e.g.
  `-i /home/user/.venv/bin/python`.
- If symbol transliteration differs from `python-slugify`, see the
  `transliterate_icons` option documented below.

## Transliteration of icons / emoji

The Rust implementation uses `deunicode` for transliteration, which may
map certain symbols or emoji to English words (for example `♥` ->
`"hearts"`, `🚀` -> `"rocket"`). The reference Python implementation
(`text-unidecode`) handles some symbols differently (often removing
them), which can cause output mismatches.

The PyO3 binding exposes a keyword argument `transliterate_icons`
(default: `False`). When `True`, symbol/emoji characters are
transliterated to words by the Rust code. When `False`, symbol/emoji
characters are removed before transliteration to match the common
Python behavior.

## CI / manylinux

For publishing to PyPI, build manylinux wheels. `maturin` has a
`--manylinux` option; see `maturin` docs for recommended Docker images
and CI workflows. In GitHub Actions you can use the official
`maturin/action` for cross-platform builds.

## License

This crate uses the license(s) MIT.

[status-image]: https://github.com/gmaOCR/slugify-rs/actions/workflows/ci.yml/badge.svg
[status-link]: https://github.com/gmaOCR/slugify-rs/actions/workflows/ci.yml
[version-image]: https://img.shields.io/pypi/v/slugify-rs.svg
[version-link]: https://pypi.python.org/pypi/slugify-rs
[coverage-image]: https://codecov.io/gh/gmaOCR/slugify-rs/branch/master/graph/badge.svg
[coverage-link]: https://codecov.io/gh/gmaOCR/slugify-rs
