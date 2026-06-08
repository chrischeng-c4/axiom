---
change: sdd-codegen-testgen
group: specir-codegen
date: 2026-03-19
---

# Requirements

All codegen infrastructure layers are complete (SpecIR 6 variants, SemanticType 12 typed operations, SpecBundle dependency graph, Tera template engine with filters), but no generator actually consumes them. This group implements the last-mile codegen pipeline:

1. **Phase 1 — Single-section generators**: Implement per-SemanticType code generators. Priority order by translatability: (a) `schema` (JSON Schema) → Rust struct / Python dataclass / TS interface; (b) `db-model` (ErdPlus) → SQL DDL + ORM model; (c) `rest-api` (OpenAPI) → route handler skeleton; (d) `cli` (YAML) → clap derive struct; (e) `state-machine` (StatePlus) → enum + transition match arms; (f) `config` (JSON Schema) → config struct.

2. **Template files**: Create `.j2`/`.tera` template files for each supported section type, loaded by the existing Tera engine.

3. **Phase 2 — Cross-section composition**: SpecBundle traversal so generators can resolve cross-section references. Define composition rules (e.g., route handler = rest-api × schema × logic × interaction). Generator receives SpecBundle, traverses dependency graph, and produces an integrated output unit.

4. **Phase 3 — Hybrid cutover**: Section types with verified templates auto-generate; section types without templates fall back to agent. Mixed output integrated into a single implementation.

**Acceptance criteria**: At least one generator (Axum recommended) produces compilable Rust code from SpecIR. Template files exist and are loaded by the Tera engine. Template test suite validates known spec input → known correct output.
