---
change: mamba-config-unify
group: unify-mamba-config
date: 2026-04-09
---

# Requirements

Unify dual MambaConfig structs into a single canonical definition.

## Current State
- `driver/config.rs::MambaConfig` -- simple struct (entry_point, crates: HashMap<String, String>, expose: HashMap<String, Vec<String>>). Has discover(), from_file(), is_symbol_exposed(). Used by driver/mod.rs, main.rs, and all tests.
- `config/schema.rs::MambaConfig` -- richer struct (project: ProjectConfig, crates: HashMap<String, CrateConfig>, build: BuildConfig, paths: PathsConfig). Has from_file(), from_str(), validate(). Exported from config/mod.rs but NOT used by any code outside its own module.

## What Needs to Change
1. Keep `config/schema.rs::MambaConfig` as the canonical definition (it is richer and more forward-looking with project metadata, build config, paths, per-crate CrateConfig)
2. Migrate features from `driver/config.rs::MambaConfig` into the canonical one: discover() method, is_symbol_exposed() method
3. Add an `entry_point` accessor or field to the canonical struct (currently uses project.name, but driver needs entry_point)
4. Update `driver/config.rs` to remove MambaConfig, keep CompilerConfig/Backend/EmitMode/OptLevel
5. Update `driver/mod.rs` re-exports to pull MambaConfig from `crate::config` instead of `config::MambaConfig`
6. Update `main.rs` imports
7. Update all tests in driver/mod.rs that construct MambaConfig with the old shape
8. Ensure `CompilerConfig.project_config: Option<MambaConfig>` uses the unified type

## Acceptance Criteria
- Single MambaConfig struct in crate, defined in config/schema.rs
- All code compiles: cargo check -p mamba
- All existing tests pass: cargo test -p mamba --lib
- driver/config.rs no longer defines a MambaConfig struct
- The is_symbol_exposed() and discover() methods work on the unified struct
