---
change: fix-remaining-drift-risks
group: drift-fixes
date: 2026-03-25
---

# Requirements

Three fixes:

1. **fill_sections weak fallback** — in create_change_spec.rs agent loop, when analyze step produces empty fill_sections, current default is [overview, requirements, scenarios, changes]. Should instead derive from section_rules keyword matching on the spec's requirements text.

2. **Dead cclab/knowledge/ reference** — create_reference_context.rs prompt template says 'Search cclab/knowledge/ for relevant docs' but this directory doesn't exist. Remove the reference.

3. **merge_strategy extend undocumented** — change-merge.md spec lists only 'new' and 'update' strategies but code also uses 'extend'. Document it.
