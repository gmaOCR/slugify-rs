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

Trusted Publisher (PyPI) — Quick Guide
-------------------------------------

If you want to publish using GitHub Actions without storing a long-lived PyPI token,
use PyPI Trusted Publishers (OIDC). Steps:

1. Create or register a Trusted Publisher on PyPI
   - Go to https://pypi.org/trusted-publishers/ and follow the "Create a Trusted Publisher"
     flow. Choose GitHub Actions as the provider and authorize the connection to your
     GitHub organization.

2. Associate the publisher with this project
   - On your project's PyPI page, add the Trusted Publisher to allow the publisher to
     publish releases for this project.

3. Ensure GitHub workflow has `id-token: write`
   - The publish job must include `permissions: id-token: write` at job level. Our
     `.github/workflows/publish-pypi.yml` already sets that.

4. Use `pypa/gh-action-pypi-publish` in the workflow
   - The repository contains a workflow that builds the wheel via `maturin` and then
     calls `pypa/gh-action-pypi-publish@release/v1`. This action automatically uses
     OIDC when the Trusted Publisher is configured.

5. Create a release/tag to trigger publishing
   - Push an annotated tag `vX.Y.Z` or create a GitHub Release — the workflow triggers
     when a tag is pushed (see `.github/workflows/publish-pypi.yml`).

6. Approvals and Environments
   - If you configure a GitHub `environment` for publishing (e.g. `pypi`) and require
     approvals, the job will pause until approved. Consider using job-level `id-token`
     permissions to avoid workflow-level exposure.

7. Troubleshooting
   - If publishing fails, inspect the Actions run logs for the `Publish package` step.
   - Common errors: missing `id-token` permission, Trusted Publisher not associated
     with the project, or environment protections requiring approval.

