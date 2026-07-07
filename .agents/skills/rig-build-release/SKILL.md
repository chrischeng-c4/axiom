---
name: rig:build:release
description: Release rig end-to-end: prepare the release build, land via git:land, tag/push rig@<version>, then monitor the GitHub release workflow until the release is visible.
user-invocable: true
---

# /rig:build:release

Cuts and monitors a rig release. Release-prep checks `rig@<version>` tag
collisions, advances the version with the base-64 patch/minor carry convention
when needed, builds rig with the **release** cargo profile, installs
`~/.cargo/bin/rig`, and commits version files. The skill then lands that
commit, tags the landed `HEAD`, pushes the tag, and monitors GitHub release
publication.

For fast iteration, use `/rig:build:debug` instead.

## Instructions

### Step 1 — release-prep

Run the release-prep script:

```bash
.agents/skills/rig-build-release/scripts/release.sh
```

Capture `RELEASE_TAG=rig@<version>` from stdout.

### Step 2 — land

Run `git:land` as-is. Stop if required checks fail.

### Step 3 — tag + push

```bash
git tag -a rig@<version> -m "Release rig@<version>"
git push origin rig@<version>
```

### Step 4 — monitor GitHub release

```bash
scripts/project-build-monitor-release.sh rig rig@<version>
```

Report the installed version, merged PR, pushed tag, GitHub Actions run URL,
and GitHub Release URL.
