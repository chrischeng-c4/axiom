---
change: sdd-issue-centric
group: issue-centric-workflow
date: 2026-04-09
status: answered
---

# Pre-Clarifications

### Q1: General
- **Question**: How does phase tracking move from STATE.yaml to issue?
- **Answer**: Add phase, branch, git_workflow fields to issue frontmatter. Phase updates write to issue file instead of STATE.yaml. STATE.yaml is deprecated but kept for backward compat.

### Q2: General
- **Question**: What happens to existing changes with STATE.yaml?
- **Answer**: Backward compat: if STATE.yaml exists and issue has no phase field, read from STATE.yaml. New changes write to issue frontmatter only.

### Q3: General
- **Question**: How does run-change find the issue?
- **Answer**: score run-change --issue <slug>. Loads from .score/issues/open/{slug}.md. Reads phase from frontmatter to determine resume point.

