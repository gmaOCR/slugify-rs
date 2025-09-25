## slugify-rs — simple and fast slugs for text

[![status-image]][status-link]
[![version-image]][version-link]
[![coverage-image]][coverage-link]

This project provides a small Rust library that turns text into a
"slug" — a short, URL-friendly string. For example, "Hello World!"
becomes "hello-world". There is also an optional Python extension so
Python programs can use the same fast Rust logic.

This README explains how to use the library, how to build the
Python extension, and how to run tests. Instructions are written for
readers who may not be familiar with Rust or Python packaging.

Quick summary

- Rust users: call the library from your Rust code.
- Python users: build and install the small extension (wheel) using
  `maturin` (instructions below), then call `python_slugify_pi.slugify()`.

Basic example (Python)

```python
import python_slugify_pi

print(python_slugify_pi.slugify("C'est déjà l'été!"))
# control emoji handling: default keeps compatibility with Python
print(python_slugify_pi.slugify("I ♥ 🚀"))
print(python_slugify_pi.slugify("I ♥ 🚀", transliterate_icons=True))
```

Quick test after installing the extension

```bash
python -c "import python_slugify_pi; print(python_slugify_pi.slugify(\"C'est déjà l'été!\"))"
```

Prerequisites (short)

- A recent Rust toolchain (install `rustup`).
- Python 3.8 or newer when you want the Python extension.
- `maturin` (a small tool to build Python extension wheels) if you
  want the Python module.

If you use Linux, you may also need system build tools (see the
BUILD instructions below for exact commands).

How to build and install the Python extension (developer mode)

1. Create and activate a Python virtual environment (recommended):

```bash
python3 -m venv .venv
source .venv/bin/activate
```

2. Install `maturin` into the virtual environment:

```bash
pip install --upgrade pip
pip install maturin
```

3. Build and install the extension into the venv for development:

```bash
maturin develop --release --features python
```

This command compiles the Rust code and installs a small Python package
called `python_slugify_pi` into your virtual environment. You can then
import and use it from Python immediately.

How to build a distributable wheel

```bash
maturin build --release -i python --features python
pip install target/wheels/*.whl
```

This produces a wheel file in `target/wheels/` that you can share or
upload to PyPI.

Running tests

- Rust unit tests (run from the project root):

```bash
cargo test -p slugify-rs --lib
```

- Python integration tests (after installing the wheel or using
  `maturin develop`):

```bash
pytest tests/python
```

Notes and troubleshooting (simple)

- If `maturin` cannot find your Python interpreter, tell it which one
  to use by adding `-i /path/to/python` to the `maturin` command.
- On Debian/Ubuntu you may need to install system tools before
  building: `sudo apt install build-essential pkg-config python3-dev`.
- Emoji and symbols: the library tries to match Python behavior by
  default, but you can change how emoji are handled with
  `transliterate_icons` (see examples).

How the pre-translations work

There is a small table of language-specific replacements (for
Cyrillic, German, Greek). These are not applied automatically. You can
either pass them as initial `replacements` or call
`apply_pre_translations()` before slugifying if you want the same
behavior as the original Python library.

License

This project is available under the MIT license.

Golden tests — simple explanation (for anyone)
------------------------------------------------

What are golden tests?

- Golden tests compare the output of this Rust implementation to the
  output of the reference Python implementation. The goal is to
  ensure the Rust behaviour matches the Python behaviour for a set
  of representative examples.

Why this matters (plain language)

- If you change the code, golden tests help confirm we didn't
  accidentally change how text is slugified. They are especially
  useful for tricky cases like emoji, accents or custom replacements.

How to run golden tests locally (easy)

1. Create a Python virtual environment and install tools and the
   reference Python package. From the project root:

```bash
python3 -m venv .venv
source .venv/bin/activate
pip install --upgrade pip
pip install maturin
pip install python-slugify
```

2. Build and install the Rust Python binding into the venv (so the
   harness can import it):

```bash
.venv/bin/maturin develop --release --features python
```

3. Run the golden harness (it will compare Python vs Rust outputs):

```bash
.venv/bin/python tests/golden_harness.py  # returns 0 on success
```

If the harness reports differences, it will print a small diff and
exit with a non-zero code. The CI is configured to upload the harness
log so we can inspect it when a run fails.

How to regenerate golden outputs

- The golden outputs are produced by the Python reference and stored
  by the harness when you ask it to regenerate. To regenerate (and
  overwrite the saved goldens), set the environment variable
  `REGEN_GOLDEN=1` and run the harness:

```bash
REGEN_GOLDEN=1 .venv/bin/python tests/golden_harness.py
```

- Only regenerate goldens when you intentionally want to accept new
  behavior (for example after a deliberate improvement). Ask a
  reviewer to confirm the change before committing regenerated
  goldens.

If you want help

- If the harness shows differences and you are not sure why, copy
  the `golden_run.log` file into the issue or PR and someone will
  help triage it. The CI is set up to always upload that log when the
  golden step runs.


[status-image]: https://github.com/gmaOCR/slugify-rs/actions/workflows/ci.yml/badge.svg

[status-link]: https://github.com/gmaOCR/slugify-rs/actions/workflows/ci.yml

[version-image]: https://img.shields.io/pypi/v/slugify-rs.svg

[version-link]: https://pypi.python.org/pypi/slugify-rs

[coverage-image]: https://codecov.io/gh/gmaOCR/slugify-rs/branch/master/graph/badge.svg

[coverage-link]: https://coveralls.io/r/gmaOCR/python-slugify
