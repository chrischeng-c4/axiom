---
name: rig:build:debug
description: Build rig in debug mode, commit a debug checkpoint when needed, install it to ~/.cargo/bin/rig, and use a git-hash-suffixed debug version. Use when the user asks for rig debug build, local rig install, or fast iteration build of the rig CLI.
user-invocable: true
---

# /rig:build:debug

Builds the rig CLI in debug mode and installs `target/debug/rig` to
`~/.cargo/bin/rig`. The project build script checks tag collisions, commits a
dirty tree before building, appends the current git hash to the build metadata
version, and restores manifest files after the debug build.

## Instructions

Run the build script:

```bash
.claude/skills/rig-build-debug/scripts/build.sh
```

Report the result to the user, including whether `~/.cargo/bin/rig --version`
was printed successfully.
