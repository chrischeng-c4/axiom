---
change_id: phase1
type: gap_codebase_knowledge
created_at: 2026-02-12T17:57:27.429433+00:00
updated_at: 2026-02-12T17:57:27.429433+00:00
---

# Gap Analysis: Codebase vs Knowledge

## Convention gaps

1. **No runtime module yet** — Knowledge documents NaN-boxing and RC patterns but no code implements them. Severity: HIGH (expected — this is what Phase 1 builds)
2. **setjmp/longjmp not available in Cranelift** — Knowledge mentions setjmp/longjmp for exceptions but Cranelift has no native setjmp support. Must use imported C setjmp or alternative. Severity: MEDIUM

## Pattern compliance

3. **File size limit** — All existing files are under 500 lines per CLAUDE.md. New files must maintain this. Severity: LOW
4. **Error handling** — Existing code uses `TaipanError` enum consistently. New runtime/lower modules should follow same pattern. Severity: LOW

## No gaps

- Orbit bridge internals knowledge is informational only, not prescriptive for taipan
- Dynamic config knowledge applies to taipan.toml which is already implemented

## Summary

1 HIGH gap (expected — runtime not yet built), 1 MEDIUM gap (setjmp availability in Cranelift), 2 LOW convention reminders."
