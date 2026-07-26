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

<!-- aw:start -->
## Claude Runtime Adapter

- Import `@AGENTS.md` as the shared checkout authority and do not duplicate its facts here.
- Load generated `.claude/rules/**/*.md` projections; files without `paths` apply at launch and path-scoped files apply only to matching work.
- Treat skills as human-invoked entry points. Mid-loop agent protocol comes from `aw` stdout or `aw llm`.
<!-- aw:end -->
