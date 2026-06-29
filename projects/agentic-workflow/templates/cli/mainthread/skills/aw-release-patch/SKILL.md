---
name: aw:release-patch
description: Deprecated legacy AW release path. Use aw:build:release for release builds or aw:build:debug for debug installs.
user-invocable: true
---

# /aw:release-patch

Deprecated. Do not use this legacy direct tag path.

Use:

- `/aw:build:debug aw` for a local debug install.
- `/aw:build:release aw` for a release that lands through `git:land`, pushes
  `aw@<version>`, and monitors GitHub release publication.
