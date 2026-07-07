---
name: arena:build:debug
description: Build arena in debug mode, commit a debug checkpoint when needed, install it to ~/.cargo/bin/arena, and use a git-hash-suffixed debug version. Use when the user asks for arena debug build, local arena install, or fast iteration build of the arena CLI.
user-invocable: true
---

# /arena:build:debug

Builds the arena CLI in debug mode and installs `target/debug/arena` to
`~/.cargo/bin/arena`. The project build script checks tag collisions, commits a
dirty tree before building, appends the current git hash to the build metadata
version, and restores manifest files after the debug build.

## Instructions

Run the build script:

```bash
.agents/skills/arena-build-debug/scripts/build.sh
```

Report the result to the user, including whether `~/.cargo/bin/arena --version`
was printed successfully.
