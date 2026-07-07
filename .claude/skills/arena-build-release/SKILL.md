---
name: arena:build:release
description: Release arena end-to-end: prepare the release build, land via git:land, tag/push arena@<version>, then monitor the GitHub release workflow until the release is visible.
user-invocable: true
---

# /arena:build:release

Cuts and monitors an arena release. Release-prep checks `arena@<version>` tag
collisions, advances the version with the base-64 patch/minor carry convention
when needed, builds arena with the **release** cargo profile, installs
`~/.cargo/bin/arena`, and commits version files. The skill then lands that
commit, tags the landed `HEAD`, pushes the tag, and monitors GitHub release
publication.

For fast iteration, use `/arena:build:debug` instead.

## Instructions

### Step 1 — release-prep

Run the release-prep script:

```bash
.claude/skills/arena-build-release/scripts/release.sh
```

Capture `RELEASE_TAG=arena@<version>` from stdout.

### Step 2 — land

Run `git:land` as-is. Stop if required checks fail.

### Step 3 — tag + push

```bash
git tag -a arena@<version> -m "Release arena@<version>"
git push origin arena@<version>
```

### Step 4 — monitor GitHub release

```bash
scripts/project-build-monitor-release.sh arena arena@<version>
```

Report the installed version, merged PR, pushed tag, GitHub Actions run URL,
and GitHub Release URL.
