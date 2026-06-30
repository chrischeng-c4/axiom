---
name: cap:build:release
description: "Release cap end-to-end: prepare the release build, land via git:land, tag/push cap@<version>, then monitor GitHub Actions and the GitHub Release until published. Use when the user asks to release cap or run a cap release build."
user-invocable: true
---

# /cap:build:release

Cuts **and lands** a cap release in four phases. `git:land` is the middle
sub-action; invoke it as-is.

1. **release-prep** — `projects/cap/build.sh release`: check tag collisions,
   bump when needed, build release, install `cap`, `cap-fast`, and `cap-full`,
   and commit `release(cap): cap@X`. **No tag, no push.**
2. **land** — run the **/git:land** flow to land the release commit on `main`
   (rebase `origin/main` -> push -> PR -> squash-merge -> rebase back).
3. **tag + push** — once the release commit is on `main`, tag the landed `HEAD`
   `cap@X` and push the tag. Pushing a `cap@*` tag triggers
   `.github/workflows/cap-release.yml` when present.
4. **monitor release** — watch the GitHub Actions run and verify the GitHub
   Release exists before reporting success.

## Why the tag is last

`git:land` rebases and squash-merges, which rewrites commit SHAs. Tagging before
the land leaves `cap@X` pointing at a commit that is not on `main`; create the
tag only after the release commit has landed.

## Instructions

### Step 1 — release-prep

Run the prep wrapper:

```bash
.claude/skills/cap-build-release/scripts/release.sh
```

Capture the new tag from its `RELEASE_TAG=cap@<version>` output line.

### Step 2 — land

Run `git:land` on the current branch. Stop if required checks fail.

### Step 3 — tag + push

After `git:land` finishes and the release commit is on `main`, tag the landed
commit and push the tag:

```bash
git tag -a cap@<version> -m "Release cap@<version>"
git push origin cap@<version>
```

### Step 4 — monitor GitHub release

```bash
scripts/project-build-monitor-release.sh cap cap@<version>
```

Report the new version, merged PR, pushed tag, GitHub Actions run URL, and
GitHub Release URL.
