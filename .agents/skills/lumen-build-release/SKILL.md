---
name: lumen:build:release
description: Release lumen end-to-end by building, installing, committing, tagging, pushing the source branch, and pushing the lumen release tag. Use when the user asks to release lumen or run a lumen release build.
---

# /lumen:build:release

Cuts a lumen release using the project-owned release path:

- bumps `projects/lumen/Cargo.toml` patch version with the repository base-64
  patch/minor carry convention
- syncs `Cargo.lock`
- builds `lumen` with the release cargo profile and published CLI ops features
- installs `target/release/lumen` to `~/.cargo/bin/lumen`
- commits the version files
- creates the `lumen@<version>` annotated tag
- pushes the current source branch to `origin`
- pushes the `lumen@<version>` tag to trigger `.github/workflows/lumen-release.yml`

If the current `HEAD` already has a `lumen@<version>` tag, the script treats it
as an already-prepared release and only performs the source/tag push. This keeps
the skill idempotent after a local release build finishes but the GitHub push has
not happened yet.

## Instructions

Run the release script:

```bash
.agents/skills/lumen-build-release/scripts/release.sh
```

Report the installed version, commit, and tag to the user.
