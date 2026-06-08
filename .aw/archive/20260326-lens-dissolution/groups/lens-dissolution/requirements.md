---
change: lens-dissolution
group: lens-dissolution
date: 2026-03-25
---

# Requirements

1. Dissolve lens/ sub-module: promote all lens sub-modules (lint/, semantic/, types/, refactoring/, search/, lsp/, gen/, graph/, syntax/, server/, spec/, schemas/, format/, core/) to cclab-sdd/src/ top-level. Resolve naming collisions (e.g. types/ → type_inference/). Migrate specs from cclab/specs/crates/cclab-lens/ to cclab/specs/crates/cclab-sdd/. Update all imports, remove pub mod lens, verify CLI still works.
2. Wire cross-file type propagation: Connect DeepTypeInferencer::propagate_types() into the analysis pipeline after per-file inference. Run in topological order via ImportGraph. Cache propagated types in daemon FileAnalysis. Make lens_type_at/lens_hover return propagated types. Focus Python first.
3. Agent context builder: New CLI command (cclab sdd context <file:symbol> --depth N). Combines import graph forward traversal + call graph backward traversal + test file detection. Returns must_read (dependencies) + may_affect (reverse deps) + type_context (cross-boundary signatures). Configurable depth (default 2). Multi-language. CLI-only, no MCP.
4. Agent-optimized output: New --format agent output mode for lens check. Symbol-centric JSON with symbol map, import edges, issues, impact scope. Compact (no SARIF boilerplate). CLI-only, no MCP.
