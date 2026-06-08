---
change: 1141
group: check-alignment-phase2
date: 2026-04-04
status: answered
---

# Pre-Clarifications

### Q1: General
- **Question**: Should the @spec annotation parser be a separate module or part of the existing spec_alignment module?
- **Answer**: Part of spec_alignment as a submodule: spec_alignment/annotations.rs. Keeps it co-located with Phase 1 parser/rules.

### Q2: General
- **Question**: For Lens integration, should we call Lens at runtime or pre-index symbols?
- **Answer**: Pre-index via sdd daemon. Depends on the daemon's symbol index for faster lookups. check-alignment queries the pre-built index rather than running Lens analysis from scratch each time.

