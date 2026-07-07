---
name: lumen:build:debug
description: Build lumen in debug mode, commit a debug checkpoint when needed, install it to ~/.cargo/bin/lumen, and use a git-hash-suffixed debug version. Use when the user asks for lumen debug build, local lumen install, or fast iteration build of the lumen CLI.
user-invocable: true
---

# /lumen:build:debug

Builds the lumen CLI in debug mode and installs `target/debug/lumen` to
`~/.cargo/bin/lumen`. The project build script checks tag collisions, commits a
dirty tree before building, appends the current git hash to the build metadata
version, and restores manifest files after the debug build.

## Instructions

Run the build script:

```bash
.claude/skills/lumen-build-debug/scripts/build.sh
```

Report the result to the user, including whether `~/.cargo/bin/lumen --version`
was printed successfully.
