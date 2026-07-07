---
name: aw:build:release
description: Prepare and publish only an Agentic Workflow aw release: build release, commit version files, land via git:land, tag/push, then monitor the GitHub release workflow until the GitHub release is visible.
user-invocable: true
---

# /aw:build:release

Cuts and monitors an Agentic Workflow (`aw`) release. The AW `build.sh` does
release-prep only: check `aw@<version>` tag collisions, advance the version
with the base-64 patch/minor carry convention when needed, build with the
release profile, install locally, and commit version files. The skill then
lands that commit, tags the landed `HEAD`, pushes the tag, and monitors GitHub
Actions plus the GitHub Release before declaring completion.

## Instructions

### Step 1 — release-prep

Run the dispatcher without arguments. The optional argument is accepted only for
old muscle-memory invocations and must be `aw` or `agentic-workflow`.

```bash
.agents/skills/aw-build-release/scripts/release.sh
```

Examples:

- `/aw:build:release` - releases `projects/agentic-workflow`.
- `/aw:build:release aw` - accepted compatibility form; still releases AW.
- `/aw:build:release mamba` - rejected; this skill does not release other projects.

The dispatcher execs `projects/agentic-workflow/build.sh release`. Capture
`RELEASE_TAG=aw@<version>` from stdout.

### Step 2 — land

Run the `git:land` flow as-is to land the release-prep commit to `main`. Stop if
required checks fail.

### Step 3 — tag + push

After `git:land` completes and the working branch is synced to `origin/main`,
tag the landed commit and push the tag:

```bash
git tag -a aw@<version> -m "Release aw@<version>"
git push origin aw@<version>
```

### Step 4 — monitor GitHub release

Monitor the project release workflow and verify the GitHub Release exists:

```bash
scripts/project-build-monitor-release.sh aw aw@<version>
```

Report the version, merged PR, pushed tag, GitHub Actions run URL, and GitHub
Release URL.

Exit codes from the dispatcher:
- `2` unsupported argument or too many arguments
- `4` `projects/agentic-workflow/build.sh` is missing or not executable
