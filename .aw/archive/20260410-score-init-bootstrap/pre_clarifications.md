---
change: score-init-bootstrap
date: 2026-04-10
status: answered
---

# Pre-Clarifications

### Q1: General
- **Question**: Should score-issue and score-issue-patrol skills be added to the deprecated list in install_claude_skills?
- **Answer**: No. They are new skills being added, not deprecated. The deprecated list removes old skills; these are new additions.

### Q2: General
- **Question**: For settings.json merge strategy: should we overwrite an existing settings.json or merge hooks only?
- **Answer**: Merge strategy: preserve existing hooks array entries, add SubagentStop hook only if no score-* matcher exists already. Use JSON merge — do not replace the entire file.

### Q3: General
- **Question**: Should legacy sdd-* agent files be cleaned up during init?
- **Answer**: Yes. Remove sdd-*.md files from .claude/agents/ during init if they exist (they are deprecated stubs).

### Q4: General
- **Question**: Is score-agent skill included in templates?
- **Answer**: No. score-agent is in the deprecated list and does not have a template. Only the 9 current skills are included: score-run-change, score-merge, score-fillback-main-specs, score-codex-review, score-gemini-explore-specs, score-gemini-explore-codebase, score-revise-artifact, score-issue, score-issue-patrol.

