---
change: score-handoff-takeoff
group: default
date: 2026-04-13
source: structured-issue
---

# Reference Context

### Related Specs
| Spec | Relevance | Key Requirements |
|------|-----------|------------------|
| projects/score/specs/init-command.md | high | Pattern for adding new top-level Commands enum variants and dispatching to a new module; install_claude_skills() pattern for embedding skill templates |
| crates/sdd/logic/issues-backend.md | high | Agent-first CLI design: `--json` on every verb, no interactive prompts, structured JSON errors, `--body-file -` stdin pipe; exit codes 0/1/2/3; subcommand module pattern in `projects/score/cli/src/issues.rs` |
| crates/sdd/logic/structured-issue.md | medium | Markdown document with YAML frontmatter + fixed section headers; write-time validation; section-based document model reused by handoff format |
| crates/sdd/logic/sdd-issue-author.md | low | Single-pass agent pattern (no CRR); score issues update --body-file - write-back pattern |
| crates/sdd/skills/run-change.md | low | Skill template installation mechanics: include_str! embedding, SKILL.md template format, user-invocable: true frontmatter |

### Spec Plan
| Spec ID | Action | Main Spec Ref | Sections |
|---------|--------|---------------|----------|
| score-handoff-takeoff | create | projects/score/specs/handoff-takeoff.md | overview, requirements, scenarios, logic, cli, schema, changes |
