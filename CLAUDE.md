---
project:
  name: axiom
  owner: chrischeng-c4
  url: https://github.com/chrischeng-c4/axiom
  ssh: git@github.com:chrischeng-c4/axiom.git
  default_branch: main
---

@AGENTS.md

# CLAUDE.md - Runtime Adapter

## External Implementation Workers

*(policy-only — repository operating policy; it does not override a host's
external-data approval gate.)*

AGY and GitHub Copilot CLI are legitimate, replaceable implementation workers.
Use `$agy-dispatch` or `$copilot-dispatch` to delegate one frozen, bounded task
from a clean isolated worktree. Send only task-required repository material,
never secrets. Repository policy permits these worker adapters, but the
controller must still obtain any consent the host requires before transmitting
source or issue content to an external service.

The controller owns the issue contract, design, oracle, independent review and
tests, Git integration, tracker mutation, and final acceptance. A worker must
not commit, push, approve itself, comment on, or close an issue. Ticketed work
reuses one worker conversation for that issue and its bounded corrections;
unticketed work is one-shot and cannot resume. Do not run workers with
overlapping write ownership in parallel.

<!-- aw:start -->
## Claude Runtime Adapter

- Import `@AGENTS.md` as the shared checkout authority and do not duplicate its facts here.
- Load generated `.claude/rules/**/*.md` projections; files without `paths` apply at launch and path-scoped files apply only to matching work.
- Treat skills as human-invoked entry points. Mid-loop agent protocol comes from `aw` stdout or `aw llm`.
<!-- aw:end -->
