---
change: vortex-p1-batch
date: 2026-02-14
---

# Clarifications

## Q1: Implementation Approach
- **Question**: Should we implement in strict dependency order or parallelize?
- **Answer**: Dependency order — implement sequentially respecting deps: event bus → render → game state → player interaction → etc.
- **Rationale**: Ensures each feature builds on a stable foundation.

## Q2: Text Rendering
- **Question**: Which approach for text rendering (#342)?
- **Answer**: Bitmap font — simple embedded bitmap font atlas, no external deps. Good enough for debug/HUD.
- **Rationale**: Minimal deps, fast to implement, sufficient for current needs.

## Q3: Git Workflow
- **Question**: Which git workflow for this batch?
- **Answer**: in_place — work on current branch (feat/vortex-engine).
- **Rationale**: All vortex work stays on the same feature branch.

