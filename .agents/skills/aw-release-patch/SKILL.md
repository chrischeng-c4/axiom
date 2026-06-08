---
name: aw:release-patch
description: Bump patch version, build debug, and install
user-invocable: true
---

# /aw:release-patch

Bumps the patch version (base-64: minor/patch 0–63 with carry), builds in debug mode, installs to `~/.cargo/bin/cclab`, commits version files, and creates a git tag.

## Instructions

Run the release script:

```bash
.agents/skills/aw-release-patch/scripts/release.sh
```

Report the result to the user.
