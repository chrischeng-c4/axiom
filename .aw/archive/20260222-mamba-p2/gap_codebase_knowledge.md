---
change_id: mamba-p2
type: gap_codebase_knowledge
created_at: 2026-02-22T11:07:21.194261+00:00
updated_at: 2026-02-22T11:07:21.194261+00:00
---

# Gap Analysis: Codebase vs Knowledge

## Convention Violations

1. **class.rs file size** (severity: high)
   - File: crates/mamba/src/runtime/class.rs (~1179 lines)
   - Knowledge: File limit rule (no file > 1000 lines, consider split at 500+)
   - P2 will add more code here (__slots__, enum, dataclasses). Must split first.

## Pattern Compliance

2. **stdlib-module-pattern**: All existing modules (os, math, json, sys, time) follow the documented pattern. P2 modules should follow same pattern.
3. **symbol-registration-pattern**: All existing functions registered via rt_sym! in symbols.rs. P2 must continue this.
4. **objdata-variant-pattern**: FrozenSet will require new ObjData variant + match arms in ~7 files. Well-documented pitfall.

## No Other Gaps

Existing codebase follows knowledge base patterns consistently. No additional convention violations found.