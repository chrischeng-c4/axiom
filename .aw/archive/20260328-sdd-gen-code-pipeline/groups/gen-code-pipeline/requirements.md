---
change: sdd-gen-code-pipeline
group: gen-code-pipeline
date: 2026-03-27
---

# Requirements

1. `cclab sdd gen parse <spec-path>` — Parse spec file, extract sections, convert to SpecIR, output as JSON to stdout
2. `cclab sdd gen code <spec-path> --lang <rust|python|ts>` — Parse spec → SpecIR → run CodeGenerator registry → output generated code
3. Both commands reuse existing infrastructure: spec/mermaid/parser, spec/openapi/parser, gen/registry, gen/rust/*, gen/python/*
4. Wire through CLI: add Code and Parse variants to GenCommands enum in commands.rs
5. Phase 1 only — gen diff deferred to later iteration
