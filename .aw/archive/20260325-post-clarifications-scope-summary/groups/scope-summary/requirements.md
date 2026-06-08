---
change: post-clarifications-scope-summary
group: scope-summary
date: 2026-03-25
---

# Requirements

post_clarifications.md must include a Scope Summary section with cross-references (not content duplication) to:

1. **Problem** — ref to requirements.md sections that define the gap
2. **Success Criteria** — ref to requirements.md acceptance criteria + pre_clarifications answers that confirmed behavior
3. **Boundary** — ref to spec_plan entries (in scope) + pre/post_clarifications answers that excluded things (out of scope) + constraints from contradictions

This applies to BOTH skipped and clarified outcomes. The summary is mandatory.

Changes needed:
- post-clarifications.md spec: add Scope Summary format
- create_post_clarifications.rs: update prompt to require scope summary
- Artifact validation: verify Scope Summary section exists and has all 3 subsections
