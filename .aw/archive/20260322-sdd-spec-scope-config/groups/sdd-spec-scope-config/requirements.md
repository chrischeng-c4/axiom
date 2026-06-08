---
change: sdd-spec-scope-config
group: sdd-spec-scope-config
date: 2026-03-22
---

# Requirements

## What needs to be built

Replace hardcoded `crates/` / `projects/` fallback logic for spec directory resolution with a config-driven approach via `[specs.scopes]` in `cclab/config.toml`.

Four deliverables:

1. **Config model** — Add `[specs.scopes]` table to `cclab/config.toml` (prefix string → subdir path mapping). Extend `SddConfig` struct and serialization to read/write this section.

2. **`resolve_spec_dir()`** — New function replacing the hardcoded `try crates/ → try projects/ → try root` probe in two callers:
   - `crates/cclab-sdd/src/workflow/scope.rs::pre_filter_specs()`
   - `crates/cclab-sdd/src/services/file_service.rs::read_main_spec_scoped()`

3. **`cclab sdd init` auto-detection** — During fresh install, detect project type from `Cargo.toml` workspace, `pyproject.toml`, or `package.json` and populate `[specs.scopes]` in the generated `config.toml` accordingly (e.g., Rust workspace → `crates` and `projects` entries).

4. **`cclab-agent FileSystemSpecStore`** — New `crates/cclab-agent/src/spec_store.rs` that reads specs from disk using the same config-driven scope resolution. No such file exists today.

## Key constraints

- The `knowledge_service.rs` `write_main_spec()` uses `cclab/specs/` as a flat base path and does not perform prefix-based lookup; it likely needs no change unless the write path also becomes config-driven.
- Existing projects without `[specs.scopes]` in their `config.toml` must continue to work — either via a backward-compat default or preserved fallback.
- Path traversal protection in `file_service.rs::validate_path_component()` must remain intact after refactoring.
- The `cclab-agent` crate currently has no spec store abstraction; `SpecProtocol` in `protocols/spec.rs` is the domain type that `FileSystemSpecStore` would produce/consume.

## Integration points

- `SddConfig` struct (`crates/cclab-sdd/src/models.rs` or equivalent config module) — needs new `specs` or `scopes` field.
- `cclab/config.toml` schema — new `[specs.scopes]` section.
- `cclab sdd init` (`crates/cclab-sdd/src/cli/init.rs`) — `run_fresh_install()` must detect workspace type and write scope config.
- `cclab-agent` context types (`context.rs`) — `FileSystemSpecStore` may be injected into agent constructors.
