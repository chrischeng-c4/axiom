---
name: meter:build:debug
description: Build meter in debug mode, commit a debug checkpoint when needed, install it to ~/.cargo/bin/meter, and use a git-hash-suffixed debug version. Use when the user asks for meter debug build, local meter install, or fast iteration build of the meter CLI.
user-invocable: true
---

# /meter:build:debug

Builds the meter CLI in debug mode and installs `target/debug/meter` to
`~/.cargo/bin/meter`. The project build script checks tag collisions, commits a
dirty tree before building, appends the current git hash to the build metadata
version, and restores manifest files after the debug build.

## Instructions

Run the build script:

```bash
.claude/skills/meter-build-debug/scripts/build.sh
```

Report the result to the user, including whether `~/.cargo/bin/meter --version`
was printed successfully.
