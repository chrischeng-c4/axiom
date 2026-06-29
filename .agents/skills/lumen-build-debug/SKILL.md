---
name: lumen:build:debug
description: Build lumen in debug mode and install it to ~/.cargo/bin/lumen without bumping version. Use when the user asks for lumen debug build, local lumen install, or fast iteration build of the lumen CLI.
---

# /lumen:build:debug

Builds the lumen CLI in debug mode and installs `target/debug/lumen` to
`~/.cargo/bin/lumen`. Does **not** bump the project version, commit, or tag.

## Instructions

Run the build script:

```bash
.agents/skills/lumen-build-debug/scripts/build.sh
```

Report the result to the user, including whether `~/.cargo/bin/lumen --version`
was printed successfully.
