---
name: vat:build:release
description: Release vat end-to-end: prepare the release build, land via git:land, tag/push vat@<version>, then monitor GitHub Actions and the GitHub Release until published. Use when the user asks to release vat or run a vat release build.
---

# /vat:build:release

Cuts and monitors a vat release using the project-owned release path:

- checks `vat@<version>` tag collisions and bumps with the repository base-64
  patch/minor carry convention when needed
- syncs `Cargo.lock`
- builds `vat` with the release cargo profile
- installs `target/release/vat` to `~/.cargo/bin/vat`
- commits the version files
- lands the release commit through `git:land`
- creates and pushes the `vat@<version>` annotated tag
- monitors the release workflow and verifies the GitHub Release exists

## Instructions

### Step 1 — release-prep

Run the release-prep script:

```bash
.claude/skills/vat-build-release/scripts/release.sh
```

Capture `RELEASE_TAG=vat@<version>` from stdout.

### Step 2 — land

Run `git:land` as-is. Stop if required checks fail.

### Step 3 — tag + push

```bash
git tag -a vat@<version> -m "Release vat@<version>"
git push origin vat@<version>
```

### Step 4 — monitor GitHub release

```bash
scripts/project-build-monitor-release.sh vat vat@<version>
```

Report the installed version, merged PR, pushed tag, GitHub Actions run URL,
and GitHub Release URL.
