---
name: cap:build:debug
description: Build cap in debug mode and install cap, cap-fast, and cap-full to the active local install directory. Use when the user asks for cap debug build, local cap install, or fast iteration build of the cap CLI/frontends.
user-invocable: true
---

# /cap:build:debug

Builds the cap CLI/frontends in debug mode through the project-owned build
script and installs `target/debug/{cap,cap-fast,cap-full}`. The project build
script commits a dirty tree before building, appends the current git hash to the
debug build metadata version, and restores manifest files afterward.

`CAP_INSTALL` selects the install directory; the wrapper defaults it to the
active `cap` directory when one is on `PATH`, otherwise `~/.cargo/bin`.

## Instructions

Run the build wrapper:

```bash
.agents/skills/cap-build-debug/scripts/build.sh
```

Report the installed version and whether the active `cap --version` resolves to
that build.
