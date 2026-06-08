---
number: 1128
title: "feat(sdd): gen code / gen diff / gen parse — spec-driven code generation pipeline"
state: open
labels: [type:enhancement, priority:p1, crate:sdd]
group: "merge-3way"
---

# #1128 — feat(sdd): gen code / gen diff / gen parse — spec-driven code generation pipeline

## Problem

SDD has SpecIR (5 variants), parsers (Mermaid, OpenAPI, JSON Schema, AsyncAPI), and code generators (Rust, Python, 6+ frameworks) — but no CLI command connects spec → code. The `gen` subcommand group only has cross-language bindings (stub/python/pyo3) and test scaffolding.

## Proposed CLI Commands

### `cclab sdd gen code <spec> --lang <rust|python|ts>`
Parse spec sections → SpecIR → run code generators → output code.
Uses existing `CodeGenerator` trait implementations.

### `cclab sdd gen diff --main <spec> --change <spec> --lang <lang>`
Parse both specs → SpecIR, run generators on both → `unified_diff(old_code, new_code)`.
MVP for "spec diff → code diff" pipeline. Deterministic sections produce code patches; non-deterministic sections produce TODO comments.

### `cclab sdd gen parse <spec> [--format json|yaml]`
Parse spec → SpecIR → dump as JSON for inspection/debugging.

## Architecture: Spec Diff → Code Diff

```
main_spec → SpecIR (old) ─┐
                           ├→ diff → deterministic → code patch
change_spec → SpecIR (new) ┘       → non-deterministic → TODO comments
```

Deterministic section types: data-model, state-machine, rest-api, rpc-api, cli, config
Non-deterministic: requirements, scenarios, overview (inject as comments)

## Design Spec

Full design documented in: `cclab/specs/crates/cclab-sdd/logic/spec-diff-codegen.md`

## Phases

1. **`gen code`** — wire existing generators to CLI (2-3 days)
2. **`gen parse`** — SpecIR inspector (1 day)
3. **`gen diff`** — MVP full-gen + unified diff (2-3 weeks)
4. **SpecDiff IR** — structured per-variant diffs (future)
5. **Incremental emitters** — diff-aware code generation (future)

## Existing Infrastructure

- SpecIR: `spec/ir.rs` — DataModel, RestApi, EventApi, StateMachine, ControlFlow
- Generators: `gen/rust/` (Serde, Axum, Sqlx), `gen/python/` (Shield, Titan, Quasar, etc.)
- Template engine: Tera with case-conversion filters
- Parsers: Mermaid, OpenAPI, JSON Schema, AsyncAPI
