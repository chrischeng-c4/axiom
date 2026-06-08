# Changelog

## 2026-02-01: CLI auto-registration infrastructure (cli-auto-register)

### Added
- CLI module registry backed by link-time registration to enable auto-discovery of subcommands.
- Ion CLI module registration so `cclab ion ...` is discoverable via the registry path.
- Library exports for `CliModule` and `CLI_MODULES` to support registration from other crates.

### Changed
- CLI dispatch now attempts registered subcommands first and falls back to the legacy command routing when no registry match exists.
