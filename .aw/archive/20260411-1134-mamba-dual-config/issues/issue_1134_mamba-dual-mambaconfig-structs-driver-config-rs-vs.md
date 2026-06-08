---
number: 1134
title: "mamba: dual MambaConfig structs — driver/config.rs vs config/schema.rs"
state: closed
labels: [type:bug, priority:p2, crate:mamba]
group: "unify-mamba-config"
---

# #1134 — mamba: dual MambaConfig structs — driver/config.rs vs config/schema.rs

## Problem

Two conflicting `MambaConfig` structs exist for parsing `mamba.toml`:

| Location | Used by | Format |
|----------|---------|--------|
| `driver/config.rs` | CLI (`cclab mamba run/build`) | `entry_point` at root, `crates: HashMap<String, String>`, `expose: HashMap<String, Vec<String>>` |
| `config/schema.rs` | Nothing (dead code?) | `[project]` table with name/version, `crates: HashMap<String, CrateConfig>` (structured with path/version/expose/module) |

Conductor's `mamba.toml` was written for the `config/schema.rs` format (with `[project]` and `[crates.cclab-schema-mamba]` sub-tables), but the CLI uses `driver/config.rs` which expects flat `crates` = version strings.

## Error

```
TOML parse error at line 6, column 1
  |
6 | [crates.cclab-schema-mamba]
  | ^^^^^^^^^^^^^^^^^^^^^^^^^^^
invalid type: map, expected a string
```

## Resolution options

1. **Unify**: merge both into one `MambaConfig` — use the richer `config/schema.rs` format and update the driver to use it
2. **Delete**: remove `config/schema.rs` if it's unused and standardize on the driver format

The richer format (`config/schema.rs`) is better designed (supports `path`, `module` alias, semver validation), so option 1 is preferred.
