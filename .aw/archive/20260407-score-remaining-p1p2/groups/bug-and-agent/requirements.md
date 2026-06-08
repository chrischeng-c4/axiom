---
change: score-remaining-p1p2
group: bug-and-agent
date: 2026-04-07
---

# Requirements

1) Fix reference context table rendering — render actual spec paths/relevance into markdown table instead of ? placeholders. In create_reference_context.rs render_specs_markdown(). 2) Fix gen test to accept .md spec files — use gen parse internally to extract RequirementPlus SpecIR. 3) Update sdd-change-implementation agent description to mandate code intelligence (score symbols/references/impact) before modifying existing files.
