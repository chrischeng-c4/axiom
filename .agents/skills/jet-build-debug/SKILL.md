---
name: jet:build:debug
description: Build debug version of jet and install to ~/.cargo/bin (no version bump)
user-invocable: true
---

# /jet:build:debug

Builds the jet CLI in **debug** mode and installs it to `~/.cargo/bin/jet` via
jet's canonical `projects/jet/build.sh debug`. Does **not** bump the version or
tag — it is the fast local install for iterating on jet.

## Instructions

Run the build wrapper (delegates to `projects/jet/build.sh debug`):

```bash
.agents/skills/jet-build-debug/scripts/build.sh
```

Report the result to the user.
