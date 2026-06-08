---
change: grid-consolidate
group: consolidate-grid-crates
date: 2026-04-08
---

# Requirements

Consolidate 5 separate cclab-grid-* crates (core, formula, history, db, server) into a single cclab-grid crate with sub-modules. The cclab-grid-wasm crate stays separate because it requires a cdylib build target.

Scope:
1. Create new crate crates/cclab-grid/ with sub-modules: core/, formula/, history/, db/, server/
2. Move source files from each crate into the corresponding sub-module directory
3. Merge all Cargo.toml dependencies into a single cclab-grid/Cargo.toml
4. Update all internal imports: cross-crate deps (e.g. cclab-grid-formula depending on cclab-grid-core) become intra-crate module references (crate::core::*)
5. Update cclab-grid-wasm/Cargo.toml to depend on cclab-grid instead of the 3 separate crates (core, formula, history)
6. Update cclab-grid-server binary: the server has a [[bin]] target — this moves into the consolidated crate or becomes a separate thin binary crate
7. Update workspace Cargo.toml members: remove 5 old crate paths, add crates/cclab-grid
8. Remove the 5 old crate directories after migration
9. Ensure all tests pass: cargo test -p cclab-grid, WASM build still works

Key constraints:
- cclab-grid-server has a binary (cclab-grid-server) with a main.rs — need to preserve this as either a [[bin]] in cclab-grid or a separate thin binary
- cclab-grid-db depends on cclab-wal (external workspace crate) — this dependency must be preserved
- cclab-grid-server depends on cclab-kv (external workspace crate) — this dependency must be preserved
- Feature-gated dependencies: server has heavy deps (axum, tokio, tower, yrs, etc.) that should likely be behind a feature flag so library users of cclab-grid don't pull them in
- No functional changes — purely structural refactor
