BUILD — Simple step-by-step guide to build the Python extension
===============================================================

This short guide shows how to build and install the small Python
extension that uses the fast Rust slugify code. It is written for
readers who may not be familiar with Rust. Follow the steps below on
your development machine.

1) Install required system tools (Linux / Debian / Ubuntu)

If you are on Debian or Ubuntu, open a terminal and run:

```bash
sudo apt update
sudo apt install -y build-essential pkg-config python3-dev libssl-dev
```

If you are on macOS, install Xcode Command Line Tools:

```bash
xcode-select --install
```

2) Install Rust (if not already installed)

Rust comes with an easy installer called `rustup`.

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
```

3) Create and activate a Python virtual environment (recommended)

This keeps the Python packages for this project separate from your
system Python.

```bash
python3 -m venv .venv
source .venv/bin/activate
```

4) Install `maturin` (tool to build Python extension wheels)

```bash
pip install --upgrade pip
pip install maturin
```

5) Build and install the extension for development

To build and install the extension into the active virtual
environment (this is fast for iterative development):

```bash
maturin develop --release --features python
```

After this command completes you can test the extension in Python:

```bash
python -c "import python_slugify_pi; print(python_slugify_pi.slugify(\"C'est déjà l'été!\"))"
```

6) Build a distributable wheel (optional)

To create a wheel file that you can share or upload to PyPI:

```bash
maturin build --release -i python --features python
pip install target/wheels/*.whl
```

7) Run tests

- Run Rust unit tests:

```bash
cargo test -p slugify-rs --lib
```

- Run Python tests after installing the wheel or using `maturin develop`:

```bash
pytest tests/python
```

Troubleshooting tips (simple)

- If `maturin` complains about the wrong Python interpreter, tell it
  which one to use: `maturin build -i /path/to/your/venv/bin/python`.
- If the build fails with linker or compiler errors on Linux, ensure
  the packages from step 1 are installed.
- On macOS, ensure Xcode command line tools are installed.

Publishing to PyPI (brief)

If you want to publish the package to PyPI you can use `maturin
publish` from a properly configured environment. For automated
publishing from GitHub Actions see the project CI workflow for an
example.

More information

- maturin docs: https://maturin.rs/
- PyO3 docs: https://pyo3.rs/


