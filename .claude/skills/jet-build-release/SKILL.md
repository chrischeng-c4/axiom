---
name: jet:build:release
description: Bump patch version, build jet, install, commit, and tag
user-invocable: true
---

# /jet:build:release

Bumps the patch version (base-64: minor/patch 0–63 with carry), builds jet in debug mode, installs to `~/.cargo/bin/jet`, commits version files, and creates a git tag.

## Instructions

Run the release script:

```bash
.claude/skills/jet-build-release/scripts/release.sh
```

Report the result to the user.
