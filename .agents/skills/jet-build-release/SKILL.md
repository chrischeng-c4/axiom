---
name: jet:build:release
description: Release jet end-to-end — prep (bump+build+install+commit), land to main via git:land, then tag jet@<version> and push to trigger the release CI
user-invocable: true
---

# /jet:build:release

Cuts **and lands** a jet release, in three phases. `git:land` is the middle
sub-action — invoke it as-is; do **not** modify it.

1. **release-prep** — `projects/jet/build.sh release`: bump the patch version in
   `projects/jet/Cargo.toml`, `cargo build --release`, install `~/.cargo/bin/jet`,
   and commit `release(jet): jet@X`. **No tag, no push.**
2. **land** — run the **/git:land** flow to land the release commit on `main`
   (rebase `origin/main` → push → PR → squash-merge → rebase back).
3. **tag + push** — once the release commit is on `main`, tag the landed `HEAD`
   `jet@X` and push the tag. Pushing a `jet@*` tag triggers
   `.github/workflows/jet-release.yml`, which builds the cross-platform binaries
   (macOS arm64 + Linux x64/arm64) and publishes the GitHub release.

## Why the tag is last

`git:land` rebases and squash-merges, which rewrites commit SHAs. Tagging before
the land would leave `jet@X` pointing at an orphaned commit that is not on
`main`. The tag must be created on the landed commit so the released binaries are
built from `main`.

## Instructions

### Step 1 — release-prep

Run the prep wrapper (delegates to `projects/jet/build.sh release`):

```bash
.agents/skills/jet-build-release/scripts/release.sh
```

Capture the new tag from its `RELEASE_TAG=jet@<version>` output line.

### Step 2 — land (sub-action: /git:land)

`git:land` stages **all** changes, so first confirm there are no unintended
untracked files. Then run the **/git:land** flow on the current branch to land
the `release(jet): jet@<version>` commit to `main`: rebase `origin/main` → push →
PR to `main` → squash-merge → rebase the branch back onto `origin/main`. Do
**not** modify git:land. If required checks fail, stop and report.

### Step 3 — tag + push the tag

After git:land finishes (the release commit is on `main` and `HEAD ==
origin/main`), tag the landed commit and push the tag:

```bash
git tag -a jet@<version> -m "Release jet@<version>"
git push origin jet@<version>
```

Report the new version, the merged PR, and the pushed tag (CI then publishes the
release).
