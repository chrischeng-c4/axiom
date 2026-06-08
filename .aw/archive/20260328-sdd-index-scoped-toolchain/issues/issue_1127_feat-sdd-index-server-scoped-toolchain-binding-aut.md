---
number: 1127
title: "feat(sdd): index server scoped toolchain binding — auto-discover project roots from Cargo.toml/pyproject.toml/package.json"
state: open
labels: [type:enhancement, priority:p1, crate:sdd]
group: "scoped-toolchain"
---

# #1127 — feat(sdd): index server scoped toolchain binding — auto-discover project roots from Cargo.toml/pyproject.toml/package.json

## Problem

The index server (daemon) currently has no toolchain awareness. It cannot resolve external dependencies:

- **Python**: `import flask` → `Type::Unknown` (only 12 hardcoded stdlib stubs)
- **Rust**: No `Cargo.toml` parsing, no workspace dep resolution
- **TypeScript**: No `tsconfig.json` paths, no `node_modules/` resolution

In a monorepo with multiple sub-projects (e.g., `projects/conductor/` has both Python + TS), each sub-project has its own toolchain (.venv, node_modules, etc.).

## Design

### Scope = toolchain root

Each scope is a sub-project with its own dependency graph:

```
cclab/.index/
├── scopes.toml                    # auto-generated + user override
└── scopes/
    ├── rust-workspace/cache/      # 1 Cargo workspace = 1 scope
    ├── py-conductor/cache/        # projects/conductor
    ├── ts-conductor-fe/cache/     # projects/conductor/fe
    └── ...
```

### Auto-discovery

Scan for 3 marker files on daemon start:

| Marker | Lang | Extract |
|--------|------|---------|
| `Cargo.toml` (workspace) | Rust | `cargo metadata` → crate paths + extern deps |
| `pyproject.toml` | Python | project root → detect `.venv/` → `site-packages/` for stubs |
| `package.json` + `tsconfig.json` | TS/JS | `paths` aliases + `node_modules/` resolution |

### File → Scope routing

When a CLI query comes in (e.g., `cclab sdd hover projects/conductor/app.py 10 5`), find the nearest marker file to determine which scope's search_paths to use.

### Config

```toml
# cclab/config.toml
[index]
auto_discover = true

[[index.scope]]
id = "py-conductor"
lang = "python"
root = "projects/conductor"
interpreter = ".venv/bin/python"  # user override
```

`auto_discover = true` (default): daemon generates scopes from marker files. User can override specific fields.

## Scope

- `crates/cclab-sdd/src/server/handler.rs` — RequestHandler needs per-scope search_paths
- `crates/cclab-sdd/src/type_inference/imports.rs` — ImportResolver.search_paths must be scope-aware
- `crates/cclab-sdd/src/type_inference/stubs.rs` — StubLoader needs to read .pyi from site-packages
- `crates/cclab-sdd/src/storage.rs` — cache directory needs per-scope partitioning
- `cclab/config.toml` — new `[index]` + `[[index.scope]]` schema
