---
name: guard:build:release
description: Release guard end-to-end: prepare the release build, land via git:land, tag/push guard@<version>, then monitor the GitHub release workflow until the release is visible.
user-invocable: true
---

# /guard:build:release

Cuts and monitors a guard release. Release-prep checks `guard@<version>` tag
collisions, advances the version with the base-64 patch/minor carry convention
when needed, builds guard with the **release** cargo profile, installs
`~/.cargo/bin/guard`, and commits version files. The skill then lands that
commit, tags the landed `HEAD`, pushes the tag, and monitors GitHub release
publication.

For fast iteration, use `/guard:build:debug` instead.

## Instructions

### Step 1 — release-prep

Run the release-prep script:

```bash
.agents/skills/guard-build-release/scripts/release.sh
```

Capture `RELEASE_TAG=guard@<version>` from stdout.

### Step 2 — land

Run `git:land` as-is. Stop if required checks fail.

### Step 3 — tag + push

```bash
git tag -a guard@<version> -m "Release guard@<version>"
git push origin guard@<version>
```

### Step 4 — monitor GitHub release

```bash
scripts/project-build-monitor-release.sh guard guard@<version>
```

Report the installed version, merged PR, pushed tag, GitHub Actions run URL,
and GitHub Release URL.
