---
change: 1142
group: check-alignment-phase3
date: 2026-04-04
---

# Requirements

Integrate spec_alignment::check() and check_with_coverage() into 4 SDD workflow integration points: (1) artifact tools — block on format violations, warn on coverage gaps; (2) merge workflow — run check-alignment post-archive, emit warnings; (3) review agent prompt — inject alignment report into reviewer context; (4) run-change response — add alignment_warnings field to response JSON. Same check function, different strictness levels per caller.
