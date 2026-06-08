---
number: 1087
title: "refactor: dissolve lens/ module into sdd top-level structure"
state: open
labels: [priority:p2, crate:sdd, type:refactor]
group: "lens-dissolution"
---

# #1087 — refactor: dissolve lens/ module into sdd top-level structure

## Problem

`crates/cclab-sdd/src/lens/` is a large sub-module (90+ files, 17 sub-modules) that acts as a separate crate-within-a-crate. The CLI already flattens lens commands into `cclab sdd *`, but the internal module structure and spec directory (`cclab/specs/crates/cclab-lens/`) still treat it as a distinct entity.

### Current lens/ sub-modules

```
lens/
├── core/          # config
├── diagnostic.rs
├── error.rs
├── format/        # file format detection
├── gen/           # codegen (Python, Rust, framework scaffolds)
├── graph/         # import/dependency graph resolution
├── handlers.rs    # request routing
├── lint/          # 40+ lint rules across 18+ file types
├── lsp/           # LSP server
├── mod.rs
├── output.rs
├── refactoring/   # rename, extract, inline, move, change-signature
├── schemas/       # K8s, GitLab CI, frontmatter schemas
├── search/        # semantic search index + query
├── semantic/      # symbols, scope, PDG (cfg, data_flow, dominator)
├── server/        # daemon, disk_cache, incremental, watch_bridge
├── spec/          # spec IR parsers (OpenAPI, AsyncAPI, Mermaid, JSON Schema)
├── storage.rs
├── syntax/        # tree-sitter parser
├── types/         # type inference, checking, narrowing
└── watch.rs
```

### Current SDD top-level modules

```
cclab-sdd/src/
├── cli/         ├── models/      ├── services/    ├── tools/
├── fillback/    ├── orchestrator/ ├── shared/      ├── ui/
├── generate/    ├── parser/      ├── spec_ir/     ├── validator/
├── generators/  ├── prompts/     ├── state/       ├── workflow/
└── lens/  ← everything behind this wall
```

## Goal

Dissolve `lens/` so its sub-modules become first-class citizens of `cclab-sdd/src/`, on par with `workflow/`, `tools/`, `services/`, etc.

## Tasks

- [ ] Migrate `cclab/specs/crates/cclab-lens/` specs into `cclab/specs/crates/cclab-sdd/` under appropriate interface/logic directories
- [ ] Promote lens sub-modules to SDD top-level (e.g. `src/lint/`, `src/semantic/`, `src/refactoring/`, `src/search/`, `src/lsp/`, `src/gen/`, etc.)
- [ ] Resolve naming collisions (e.g. lens `types/` vs SDD might need renaming to `type_inference/`)
- [ ] Remove `pub mod lens` from `lib.rs`, re-export from new locations
- [ ] Update all internal imports
- [ ] Delete `crate:lens` label — everything is `crate:sdd`
- [ ] Verify `cclab sdd *` CLI commands still work after restructure
