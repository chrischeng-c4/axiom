---
name: mamba:build:debug
description: Build mamba (debug profile) and install to ~/.cargo/bin (no version bump)
user-invocable: true
---

# /mamba:build:debug

Builds the mamba CLI in debug mode and installs to `~/.cargo/bin/mamba`. Does **not** bump the workspace version. Use this for fast iteration; for benchmarking against CPython use `/mamba:build:release` instead (release-profile binary).

## Instructions

Run the build script:

```bash
.Codex/skills/mamba-build-debug/scripts/build.sh
```

Report the result to the user.
