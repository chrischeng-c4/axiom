# Pre-Clarifications: score-init-bootstrap / default

## Key Decisions

- D1: All assets are embedded via `include_str!` — `score init` is self-contained, no runtime fetching
- D2: Update mode is non-destructive for user content (`project.md` preserved) but replaces system assets (skills, agents, hooks) to ensure consistency
- D3: `settings.json` uses JSON merge strategy, not replacement — user hooks/tools are preserved
- D4: Hook scripts get `chmod +x` on install; on Windows, this is a no-op
- D5: Legacy `sdd-*` agent stubs are removed during init update
- D6: `score init` is only wired via the CLI, not exposed as an MCP tool (MCP was removed)

## Clarifications

Q: Should score-issue and score-issue-patrol skills be added to the deprecated list?
A: No. They are new skills being added, not deprecated.

Q: For settings.json merge strategy: should we overwrite or merge?
A: Merge strategy — preserve existing hooks array entries, add SubagentStop hook only if no score-* matcher exists already.

Q: Should legacy sdd-* agent files be cleaned up during init?
A: Yes. Remove sdd-*.md files from .claude/agents/ during init.

Q: Is score-agent skill included in templates?
A: No. Only the 9 current skills: score-run-change, score-merge, score-fillback-main-specs, score-codex-review, score-gemini-explore-specs, score-gemini-explore-codebase, score-revise-artifact, score-issue, score-issue-patrol.
