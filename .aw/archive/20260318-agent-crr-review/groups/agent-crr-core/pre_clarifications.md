---
change: agent-crr-review
group: agent-crr-core
date: 2026-03-18
status: answered
---

# Pre-Clarifications

### Q1: General
- **Question**: Should ReviewAgent be standalone struct or trait?
- **Answer**: Standalone struct. One ReviewAgent struct with ReviewType config (Spec, Code) to switch behavior. No trait needed — keeps it simple. Different review types use different system prompts and checklists, not different implementations.

### Q2: General
- **Question**: Max_revisions exceeded default behavior?
- **Answer**: Error. Return NovaError so the caller decides what to do. Safer than silently auto-approving.

### Q3: General
- **Question**: Where to put ReviewVerdict/ReviewIssue?
- **Answer**: In agents/review/ module (mod.rs + types.rs). Other agents import these types when they need to work with CRR.

