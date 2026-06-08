---
change: sdd-structured-issue
group: structured-issue-format
date: 2026-04-09
status: answered
---

# Pre-Clarifications

### Q1: General
- **Question**: How should structured issues be detected?
- **Answer**: By checking for required markdown section headers (## Problem, ## Requirements, ## Scope, etc.). If all required sections present → structured. Otherwise → legacy flow.

### Q2: General
- **Question**: Where should the section parser live?
- **Answer**: In crates/sdd/src/services/ as a new module (e.g. issue_parser.rs). Reusable by both init_change and score issues enrich.

### Q3: General
- **Question**: How does skip work in the state machine?
- **Answer**: init_change auto-generates artifacts and sets STATE.yaml phase directly to post_clarifications_created. No new states needed.

