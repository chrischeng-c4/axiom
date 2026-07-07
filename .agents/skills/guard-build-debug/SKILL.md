---
name: guard:build:debug
description: Build guard in debug mode, commit a debug checkpoint when needed, install it to ~/.cargo/bin/guard, and use a git-hash-suffixed debug version. Use when the user asks for guard debug build, local guard install, or fast iteration build of the guard CLI.
user-invocable: true
---

# /guard:build:debug

Builds the guard CLI in debug mode and installs `target/debug/guard` to
`~/.cargo/bin/guard`. The project build script checks tag collisions, commits a
dirty tree before building, appends the current git hash to the build metadata
version, and restores manifest files after the debug build.

## Instructions

Run the build script:

```bash
.agents/skills/guard-build-debug/scripts/build.sh
```

Report the result to the user, including whether `~/.cargo/bin/guard --version`
was printed successfully.
