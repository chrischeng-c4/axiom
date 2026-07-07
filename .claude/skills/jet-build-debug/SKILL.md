---
name: jet:build:debug
description: Build debug version of jet, commit a debug checkpoint when needed, install to ~/.cargo/bin, and use a git-hash-suffixed debug version.
user-invocable: true
---

# /jet:build:debug

Builds the jet CLI in **debug** mode and installs it to `~/.cargo/bin/jet` via
jet's canonical `projects/jet/build.sh debug`. The build script commits a dirty
tree before building, uses a `<version>+<git-sha>` debug version, and restores
manifest files after the local install.

## Instructions

Run the build wrapper (delegates to `projects/jet/build.sh debug`):

```bash
.claude/skills/jet-build-debug/scripts/build.sh
```

Report the result to the user.
