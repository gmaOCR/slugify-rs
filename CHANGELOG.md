# Changelog

All notable changes to this project will be documented in this file.

The format is based on Keep a Changelog and this file is maintained in
English.

## [v0.1.2] - 2025-09-25
### Added
- Improved handling of special characters in the CLI builder and slugification
  pipeline (better transliteration and options handling).
- New `read_input<R: Read>` helper with unit tests for both success and error
  paths (makes stdin handling testable without mutating process environment).
- Centralized env-map runner (`run_with_env_map`) and in-process CLI helpers to
  exercise CLI logic without spawning external processes.
### Changed
- Refactored CLI parsing and option building to use an injected environment map
  (improves testability and avoids unsafe global env manipulation in tests).
- CI: changed coverage invocation to use `cargo coverage` alias which excludes
  `src/bin/` from coverage reports by default.
### Fixed
- Various test and CI improvements (macOS maturin fixes, packaging tweaks).

## [v0.1.1] - 2025-09-24
### Added
- Initial set of CLI tests and integration with library slugify logic.
### Changed
- Improved binary path detection in the CLI to consider multiple candidate
  locations (debug/release and CI-specific target dirs).

## [v0.1.0] - 2025-09-23
Initial public release.

---

Notes
-----
- The v0.1.2 notes were drafted from the commit history between `v0.1.1` and
  HEAD; if you want a more granular per-commit bullet list I can expand the
  changelog with individual commit messages.

## Release notes for v0.1.2 (detailed)

- This release focuses on improving special-character handling in the CLI and
  slugification pipeline. The CLI now includes a `read_input` helper that trims
  trailing newlines and returns an IO result (unit-tested). The option parsing
  and builder code were refactored to use an injected environment map so the
  parsing logic can be tested deterministically in-process without mutating
  global process state.

Changes since v0.1.1 (high level):

- Added tests for environment parsing and read_input error path.
- Switched CI coverage to use an alias that excludes `src/bin/` to avoid
  environment/FS-dependent glue code from skewing coverage numbers.

