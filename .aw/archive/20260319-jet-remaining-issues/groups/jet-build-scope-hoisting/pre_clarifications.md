---
change: jet-remaining-issues
group: jet-build-scope-hoisting
date: 2026-03-19
status: answered
---

# Pre-Clarifications

### Q1: General
- **Answer**: Option A — exclude all SCC members from flattening, keep as wrapped modules. Emit a tracing::warn for cycles. Safety over coverage.

### Q2: General
- **Answer**: Conservative — any usage anywhere in the module triggers bailout. The 202.6KB result already works well with conservative detection. Don't risk correctness.

### Q3: General
- **Answer**: Already implemented as separate passes (Phase 1 in generate_scope_hoisted_bundle, Phase 2 in generate_flattened_bundle). Keep them separate. Phase 1 is default, Phase 2 activated when minify=true.

