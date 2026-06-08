---
change: sdd-workflow-cleanup
group: spec-plan
date: 2026-03-17
status: answered
---

# Pre-Clarifications

### Q1: General
- **Question**: Should the agent be allowed to override the Spec Plan manually during the analyze step?
- **Answer**: Yes. The spec plan from reference_context is the default, but the agent can override main_spec_ref and merge_strategy during analyze if it finds the plan is wrong. The artifact CLI should accept these as parameters and update frontmatter.

### Q2: General
- **Question**: For action: modify, if the main_spec_ref file does not exist, should it fail or fall back to create_new?
- **Answer**: Fall back to create_new with a warning log. The spec might have been renamed or moved.

