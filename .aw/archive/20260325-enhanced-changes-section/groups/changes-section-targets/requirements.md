---
change: enhanced-changes-section
group: changes-section-targets
date: 2026-03-25
---

# Requirements

Two levels in one change:

Level 1 — Enhanced changes section schema:
- Add targets array to changes YAML: {type: function|struct|enum|trait|impl|method, name, change, anchor (optional), position: before|after|replace|append}
- Add do_not_touch list (function/type names agent must not modify)
- Update changes section CLI flags and fill prompt
- Update change_spec review checklist to verify targets are function/type-level

Level 2 — Lens-assisted implementation prompt:
- Parse targets from changes section in common_change_impl.rs
- For each MODIFY target, call lens to extract function/type source code + line range
- Build enriched implementation prompt: current code with insertion point marked + change description + DO NOT MODIFY list with line ranges
- Agent sees exactly what to modify, not the whole file
