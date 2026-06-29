---
name: mamba:build:release
description: Release mamba end-to-end: prepare the release with a release-profile cargo build, land via git:land, tag/push mamba@<version>, then monitor the GitHub release workflow until the release is visible.
user-invocable: true
---

# /mamba:build:release

Cuts and monitors a mamba release. Release-prep checks `mamba@<version>` tag
collisions, advances the version with the base-64 patch/minor carry convention
when needed, builds mamba with the **release** cargo profile, installs
`~/.cargo/bin/mamba`, and commits version files. The skill then lands that
commit, tags the landed `HEAD`, pushes the tag, and monitors GitHub release
publication.

The release profile is required here because mamba's monitoring / `bench --compare cpython` numbers are only meaningful when measured on the optimized binary. For fast iteration, use `/mamba:build:debug` instead.

## Instructions

### Step 1 — release-prep

Run the release-prep script:

```bash
.agents/skills/mamba-build-release/scripts/release.sh
```

Capture `RELEASE_TAG=mamba@<version>` from stdout.

### Step 2 — land

Run `git:land` as-is. Stop if required checks fail.

### Step 3 — tag + push

```bash
git tag -a mamba@<version> -m "Release mamba@<version>"
git push origin mamba@<version>
```

### Step 4 — monitor GitHub release

```bash
scripts/project-build-monitor-release.sh mamba mamba@<version>
```

Report the installed version, merged PR, pushed tag, GitHub Actions run URL,
and GitHub Release URL.
