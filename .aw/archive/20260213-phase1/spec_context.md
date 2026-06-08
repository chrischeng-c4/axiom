---
change_id: phase1
type: spec_context
created_at: 2026-02-12T17:55:13.934367+00:00
updated_at: 2026-02-12T17:55:13.934367+00:00
iteration: 1
complexity: critical
stage: spec
scanned_groups:
  - cclab-core
  - cclab-cli
  - cclab-orbit
  - cclab-prism
---

# Spec Context

## Relevant Specs

- **02-architecture-principles** (group: cclab-core)
  - relevance: low
  - reason: General architecture principles applicable to taipan crate design
  - key sections: module boundaries, error handling

## Dependencies

- #275 HIR → #277 AST→HIR lowering → #278 HIR→MIR lowering
- #276 Name resolution → #277 AST→HIR lowering
- #279 Object model → #280 Refcounting → #281 Builtins
- #278 HIR→MIR + #281 Builtins → #282 Driver

## Gaps

- No cclab-taipan main specs exist yet — taipan is a new crate driven by GitHub issues #205-#294
- No runtime object model spec — needs to be defined during implementation
- No HIR/MIR spec — intermediate representations defined by code structure
