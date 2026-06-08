---
change_id: phase1
type: gap_spec_knowledge
created_at: 2026-02-12T18:00:03.328400+00:00
updated_at: 2026-02-12T18:00:03.328400+00:00
---

# Gap Analysis: Spec vs Knowledge

## Spec–Knowledge Contradictions

None found. The single relevant spec (02-architecture-principles) does not contradict any knowledge patterns.

## Knowledge patterns not reflected in specs

1. **NaN-boxing value model has no spec** — Knowledge documents the NaN-boxing pattern with 51-bit integer limit and bigint fallback, but no taipan spec exists to define the object model layout or tagging scheme. Severity: HIGH (expected — taipan has no main specs yet)

2. **Reference counting + cycle collector has no spec** — Knowledge documents RC with cycle collector pattern but no spec defines when cycle collection runs, which types are cyclic, or the RC API. Severity: HIGH (expected)

3. **setjmp/longjmp exception model has no spec** — Knowledge documents the pattern and pitfall (skips destructors) but no spec defines the exception handling contract or cleanup obligations. Severity: MEDIUM

## Responsibility boundary misalignments

4. **Orbit bridge patterns vs taipan runtime** — Knowledge documents Orbit's GIL batching and object lifetime patterns. Taipan's runtime will manage its own object model without GIL (single-threaded). No boundary conflict, but orbit patterns should not be assumed applicable. Severity: LOW

## Summary

2 HIGH gaps (both expected — no taipan specs exist yet, this change will create them implicitly via implementation), 1 MEDIUM gap (exception handling contract undefined), 1 LOW boundary note.