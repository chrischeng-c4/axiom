---
name: aw:build:debug
description: Build a project's debug binary, committing a debug checkpoint when needed and installing a git-hash-suffixed debug version. Project resolved from .aw/config.toml [[projects]]; infer from project-<name> branch when no arg.
user-invocable: true
---

# /aw:build:debug

Builds the requested project in debug mode and installs the resulting binary via
that project's own `build.sh`. The project build script commits a dirty tree
before building, finds the next non-conflicting `<project>@<version>` base, uses
`<version>+<git-sha>` for the debug build, and restores manifest files after the
local install.

## Instructions

Run the dispatcher with the project name or alias as configured in
`.aw/config.toml` `[[projects]]`. Omit the argument on a `project-<name>` branch
to infer the project from the current branch.

```bash
.agents/skills/aw-build-debug/scripts/build.sh [<project>]
```

Examples:

- `/aw:build:debug aw` — alias, builds `projects/agentic-workflow`.
- `/aw:build:debug mamba` — builds `projects/mamba` (requires `projects/mamba/build.sh`).
- `/aw:build:debug` on branch `project-aw` — infers `aw`.

The dispatcher reads `.aw/config.toml`, resolves the project's `path`, and
execs `<path>/build.sh debug`. Per-project `build.sh` owns the actual build
(cargo invocation, install, codesign, etc.).

Exit codes from the dispatcher:
- `2` no arg and current branch is not `project-<name>`
- `3` project not declared in `.aw/config.toml`
- `4` resolved project has no executable `build.sh`

Report the result to the user.
