---
number: 944
title: "feat(lens): wire cross-file type propagation — deep_inference is implemented but never called"
state: open
labels: [type:enhancement, priority:p1, crate:sdd, crate:lens]
group: "lens-dissolution"
---

# #944 — feat(lens): wire cross-file type propagation — deep_inference is implemented but never called

## Problem

`deep_inference.rs` (1035 lines) implements cross-file type propagation with:
- `DeepTypeInferencer::propagate_types()` — correct propagation logic
- `ImportGraph::topological_sort()` — correct DFS
- `update_symbol_type()` — recursive propagation on type change

**But none of this is called from the main type checker** (`check.rs` / `infer.rs`). Files are analyzed in isolation. When an AI agent calls `lens_type_at` or `lens_hover` on an imported symbol, it gets `Type::Unknown`.

### Impact on AI agents

An agent contributing to a repo constantly needs to know "what type does this imported function return?" to write correct code. Currently:

```python
from myproject.db import get_user
user = get_user(user_id)  # type: Unknown — agent can't verify .name access
```

This makes `lens_type_at`, `lens_hover`, and `lens_diagnostics` significantly less useful.

## Proposed Fix

1. Wire `DeepTypeInferencer::propagate_types()` into the analysis pipeline
2. After per-file inference, run cross-file propagation in topological order
3. Cache propagated types in daemon's `FileAnalysis` entries
4. Ensure `lens_type_at` returns propagated types, not just local inference

## Scope

- Import resolution (`imports.rs`) already indexes modules and detects `.pyi` stubs
- Module files are indexed but never parsed to extract type info — need to close this gap
- Focus on Python first (most complete type system), then Rust, then TS/Go

## Acceptance Criteria

- [ ] `lens_type_at` on an imported symbol returns the actual type from the source module
- [ ] Cross-file type propagation runs after initial per-file analysis
- [ ] Daemon cache stores propagated types and invalidates on dependency change
- [ ] Works for Python `from X import Y` and `import X` patterns
