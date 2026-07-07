---
name: meter:build:release
description: Release meter end-to-end: prepare the release build, land via git:land, tag/push meter@<version>, then monitor the GitHub release workflow until the release is visible.
user-invocable: true
---

# /meter:build:release

Cuts and monitors a meter release. Release-prep checks `meter@<version>` tag
collisions, advances the version with the base-64 patch/minor carry convention
when needed, builds meter with the **release** cargo profile, installs
`~/.cargo/bin/meter`, and commits version files. The skill then lands that
commit, tags the landed `HEAD`, pushes the tag, and monitors GitHub release
publication.

For fast iteration, use `/meter:build:debug` instead.

## Instructions

### Step 1 — release-prep

Run the release-prep script:

```bash
.agents/skills/meter-build-release/scripts/release.sh
```

Capture `RELEASE_TAG=meter@<version>` from stdout.

### Step 2 — land

Run `git:land` as-is. Stop if required checks fail.

### Step 3 — tag + push

```bash
git tag -a meter@<version> -m "Release meter@<version>"
git push origin meter@<version>
```

### Step 4 — monitor GitHub release

```bash
scripts/project-build-monitor-release.sh meter meter@<version>
```

Report the installed version, merged PR, pushed tag, GitHub Actions run URL,
and GitHub Release URL.
