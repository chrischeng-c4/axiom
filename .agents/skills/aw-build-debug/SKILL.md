---
name: aw:build:debug
description: Build only the Agentic Workflow aw debug binary, committing a debug checkpoint when needed and installing a git-hash-suffixed debug version.
user-invocable: true
---

# /aw:build:debug

Builds Agentic Workflow (`aw`) in debug mode and installs the resulting binary
via `apps/agentic-workflow/build.sh`. The AW build script commits a dirty
tree before building, finds the next non-conflicting `aw@<version>` base, uses
`<version>+<git-sha>` for the debug build, and restores manifest files after
the local install.

## Instructions

Run the dispatcher without arguments. The optional argument is accepted only for
old muscle-memory invocations and must be `aw` or `agentic-workflow`.

```bash
.agents/skills/aw-build-debug/scripts/build.sh
```

Examples:

- `/aw:build:debug` - builds `apps/agentic-workflow`.
- `/aw:build:debug aw` - accepted compatibility form; still builds AW.
- `/aw:build:debug mamba` - rejected; this skill does not build other projects.

The dispatcher execs `apps/agentic-workflow/build.sh debug`. AW's
`build.sh` owns the actual build (cargo invocation, install, codesign, etc.).

Exit codes from the dispatcher:
- `2` unsupported argument or too many arguments
- `4` `apps/agentic-workflow/build.sh` is missing or not executable

Report the result to the user.

## AW CLI Drift & Defect Reporting

`aw` changes frequently. If this skill's documented invocation, result shape, or
semantics contradict the current `aw --help` output or CLI envelope, treat that
as a suspected AW defect; do not silently invent a compatibility command or
work around it.

Before reporting, reproduce the smallest failing command and capture the `aw`
version, exact command, expected result, actual stdout/stderr, and any relevant
envelope fields. Confirm the current surface with the relevant `aw <verb>
--help`; when working on AW itself, prefer a freshly built
`target/debug/aw` if the installed binary could be stale.

Once confirmed, report an AW-owned defect with `aw issue create --title "aw:
<short symptom>" "<reproduction and evidence>"`. Do not pass `--yes` unless
GitHub writes are already authorized. Expected validation failures or defects
owned by the target project belong in that project's tracker, not as AW bugs.
