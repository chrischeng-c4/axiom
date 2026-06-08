---
name: aw:build:release
description: Bump a project's patch version, build release, install, commit, and tag. Project resolved from .aw/config.toml [[projects]]; infer from project-<name> branch when no arg.
user-invocable: true
---

# /aw:build:release

Cuts a release for the requested project: bumps the patch version (base-64:
minor/patch 0-63 with carry), builds, installs the binary, commits the version
files, and creates a git tag — all via that project's own `build.sh release`.

## Instructions

Run the dispatcher with the project name or alias as configured in
`.aw/config.toml` `[[projects]]`. Omit the argument on a `project-<name>` branch
to infer the project from the current branch.

```bash
.claude/skills/aw-build-release/scripts/release.sh [<project>]
```

Examples:

- `/aw:build:release aw` — release `projects/agentic-workflow`.
- `/aw:build:release` on branch `project-aw` — infers `aw`.

The dispatcher reads `.aw/config.toml`, resolves the project's `path`, and
execs `<path>/build.sh release`. The per-project `build.sh` owns the version
bump, build, install, commit, and tag.

Exit codes from the dispatcher:
- `2` no arg and current branch is not `project-<name>`
- `3` project not declared in `.aw/config.toml`
- `4` resolved project has no executable `build.sh`

Report the result to the user.
