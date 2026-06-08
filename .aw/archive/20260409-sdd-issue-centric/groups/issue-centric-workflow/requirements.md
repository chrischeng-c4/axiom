---
change: sdd-issue-centric
group: issue-centric-workflow
date: 2026-04-09
---

# Requirements

Make issue the unit of work in SDD.

1. `score run-change` only accepts `--issue <slug>`, no `--description`
2. One issue = one change (no multi-issue bundling)
3. Issue frontmatter absorbs STATE.yaml fields (phase, branch, git_workflow)
4. Change artifacts directory uses issue slug as name
5. `score issues list` shows phase for in-progress issues
6. No issue = no change. Error with guidance.
