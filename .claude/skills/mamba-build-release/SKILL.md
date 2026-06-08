---
name: mamba:build:release
description: Bump patch version, build mamba (release profile), install to ~/.cargo/bin, commit, and tag
user-invocable: true
---

# /mamba:build:release

Bumps the patch version (base-64: minor/patch 0–63 with carry), builds mamba with the **release** cargo profile (so the binary is benchmark-grade), installs to `~/.cargo/bin/mamba`, commits version files, and creates a git tag.

The release profile is required here because mamba's monitoring / `bench --compare cpython` numbers are only meaningful when measured on the optimized binary. For fast iteration, use `/mamba:build:debug` instead.

## Instructions

Run the release script:

```bash
.claude/skills/mamba-build-release/scripts/release.sh
```

Report the result to the user.
