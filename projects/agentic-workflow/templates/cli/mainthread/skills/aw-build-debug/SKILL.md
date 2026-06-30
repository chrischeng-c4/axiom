---
name: aw:build:debug
description: Build only the Agentic Workflow aw debug binary, committing a debug checkpoint when needed and installing a git-hash-suffixed debug version.
user-invocable: true
---

# /aw:build:debug

Builds Agentic Workflow (`aw`) in debug mode and installs the resulting binary
via `projects/agentic-workflow/build.sh`. The AW build script commits a dirty
tree before building, finds the next non-conflicting `aw@<version>` base, uses
`<version>+<git-sha>` for the debug build, and restores manifest files after
the local install.

## Instructions

Run the dispatcher without arguments. The optional argument is accepted only for
old muscle-memory invocations and must be `aw` or `agentic-workflow`.

```bash
.claude/skills/aw-build-debug/scripts/build.sh
```

Examples:

- `/aw:build:debug` - builds `projects/agentic-workflow`.
- `/aw:build:debug aw` - accepted compatibility form; still builds AW.
- `/aw:build:debug mamba` - rejected; this skill does not build other projects.

The dispatcher execs `projects/agentic-workflow/build.sh debug`. AW's
`build.sh` owns the actual build (cargo invocation, install, codesign, etc.).

Exit codes from the dispatcher:
- `2` unsupported argument or too many arguments
- `4` `projects/agentic-workflow/build.sh` is missing or not executable

Report the result to the user.
