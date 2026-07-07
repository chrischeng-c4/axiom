---
name: lumen:build:release
description: Release lumen end-to-end: prepare the release build, land via git:land, tag/push lumen@<version>, then monitor the GitHub release workflow until the release is visible.
user-invocable: true
---

# /lumen:build:release

Cuts and monitors a lumen release. Release-prep checks `lumen@<version>` tag
collisions, advances the version with the base-64 patch/minor carry convention
when needed, builds lumen with the **release** cargo profile and published CLI
ops features, installs `~/.cargo/bin/lumen`, and commits version files. The
skill then lands that commit, tags the landed `HEAD`, pushes the tag, and
monitors GitHub release publication.

The release profile is required here because lumen's service / operator behavior
should be verified on the optimized binary before publishing. For fast
iteration, use `/lumen:build:debug` instead.

## Instructions

### Step 1 — release-prep

Run the release-prep script:

```bash
.agents/skills/lumen-build-release/scripts/release.sh
```

Capture `RELEASE_TAG=lumen@<version>` from stdout.

### Step 2 — land

Run `git:land` as-is. Stop if required checks fail.

### Step 3 — tag + push

```bash
git tag -a lumen@<version> -m "Release lumen@<version>"
git push origin lumen@<version>
```

### Step 4 — monitor GitHub release

```bash
scripts/project-build-monitor-release.sh lumen lumen@<version>
```

Report the installed version, merged PR, pushed tag, GitHub Actions run URL,
and GitHub Release URL.
