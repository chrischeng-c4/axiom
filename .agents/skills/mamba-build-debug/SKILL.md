---
name: mamba:build:debug
description: Build mamba with the debug profile, commit a debug checkpoint when needed, install to ~/.cargo/bin/mamba, and use a git-hash-suffixed debug version.
user-invocable: true
---

# /mamba:build:debug

Builds the mamba CLI in debug mode and installs to `~/.cargo/bin/mamba`. The
build script commits a dirty tree before building, uses a `<version>+<git-sha>`
debug version, and restores manifest files after the local install. Use this
for fast iteration; for benchmarking against CPython use `/mamba:build:release`
instead (release-profile binary).

## Instructions

Run the build script:

```bash
.agents/skills/mamba-build-debug/scripts/build.sh
```

Report the result to the user.
