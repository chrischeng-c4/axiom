---
change: spec-decomposition-rules
group: decomposition-rules
date: 2026-03-25
---

# Requirements

Two specs need updates:

1. **reference-context.md** — Add file decomposition rules to Create prompt and Review checklist:
   - 1 spec file = 1 logical unit (service, module, component)
   - No duplicate section types in one file — split into separate files
   - Spec path mirrors source path (src/api/external.rs → specs/interfaces/external-api.md)
   - Cross-file refs use $ref

2. **change-spec.md** — Add section order enforcement to Review checklist:
   - Verify fill_order was respected (data → behavior → interface → test → changes)
   - Verify no duplicate section types in any spec file

Both Create and Review prompts must enforce these rules. This is spec-only — no Rust code changes.
