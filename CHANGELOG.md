# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).

## [Unreleased]

### Deprecated

- **`jet test --playwright`** — The `--playwright` escape hatch is now a
  formally supported but deprecated migration aid. Every invocation prints a
  deprecation warning on stderr:

  ```
  warning: --playwright is deprecated and will be removed; see <migration-guide-url>
  ```

  Set `JET_SUPPRESS_PLAYWRIGHT_WARNING=1` to suppress the warning during the
  transition period.

  **Removal version**: `--playwright` will be removed in the **second
  subsequent minor release** (planned v0.7.x).

  **Migration**: See `projects/jet/docs/migration-from-playwright.md` for the
  flag mapping table, import rewrite recipes (from @playwright/test to @jet/test, legacy), and
  trace-viewer / HTML-reporter deep-link usage.

  **Incompatible flags**: `--reporter`, `--trace`, `--workers`, `--shard`,
  and `--report-dir` cannot be combined with `--playwright` and will produce a
  hard error (exit code 2).
