---
change: sdd-structured-issue
group: structured-issue-format
date: 2026-04-09
---

# Requirements

Add structured issue format to SDD that absorbs phases 2-3 and 7.

1. **Issue section parser**: Extract structured sections (Problem, Requirements, Acceptance Criteria, Scope, Key Decisions, Reference Context) from issue markdown
2. **init_change update**: When structured issue detected, auto-generate requirements.md, pre_clarifications.md (answered), post_clarifications.md (skipped), reference_context.md from issue sections. Advance STATE to post_clarifications_created.
3. **`score issues enrich` command**: Run reference_context agent on an issue to fill Reference Context section
4. **Backward compat**: Unstructured issues fall back to current flow
5. **SKILL.md update**: Mainthread detects structured issues and skips early phases
