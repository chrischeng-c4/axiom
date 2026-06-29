---
name: vat:build:debug
description: Build vat in debug mode, commit a debug checkpoint when needed, install it to ~/.cargo/bin/vat, and use a git-hash-suffixed debug version. Use when the user asks for vat debug build, local vat install, or fast iteration build of the vat CLI.
---

# /vat:build:debug

Builds the vat CLI in debug mode and installs `target/debug/vat` to
`~/.cargo/bin/vat`. The project build script checks tag collisions, commits a
dirty tree before building, appends the current git hash to the build metadata
version, and restores manifest files after the debug build.

## Instructions

Run the build script:

```bash
.claude/skills/vat-build-debug/scripts/build.sh
```

Report the installed version and whether `~/.cargo/bin/vat --version` was
printed successfully.
