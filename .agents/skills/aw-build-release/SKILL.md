---
name: aw:build:release
description: Prepare and publish a project release: resolve project from .aw/config.toml, build release, commit version files, land via git:land, tag/push, then monitor the GitHub release workflow until the GitHub release is visible.
user-invocable: true
---

# /aw:build:release

Cuts and monitors a release for the requested project. The project `build.sh`
does release-prep only: check `<project>@<version>` tag collisions, advance the
version with the base-64 patch/minor carry convention when needed, build with
the release profile, install locally, and commit version files. The skill then
lands that commit, tags the landed `HEAD`, pushes the tag, and monitors GitHub
Actions plus the GitHub Release before declaring completion.

## Instructions

### Step 1 — release-prep

Run the dispatcher with the project name or alias as configured in
`.aw/config.toml` `[[projects]]`. Omit the argument on a `project-<name>` branch
to infer the project from the current branch.

```bash
.agents/skills/aw-build-release/scripts/release.sh [<project>]
```

Examples:

- `/aw:build:release aw` — release `projects/agentic-workflow`.
- `/aw:build:release` on branch `project-aw` — infers `aw`.

The dispatcher reads `.aw/config.toml`, resolves the project's `path`, and
execs `<path>/build.sh release`. Capture `RELEASE_TAG=<project>@<version>` from
stdout.

### Step 2 — land

Run the `git:land` flow as-is to land the release-prep commit to `main`. Stop if
required checks fail.

### Step 3 — tag + push

After `git:land` completes and the working branch is synced to `origin/main`,
tag the landed commit and push the tag:

```bash
git tag -a <project>@<version> -m "Release <project>@<version>"
git push origin <project>@<version>
```

### Step 4 — monitor GitHub release

Monitor the project release workflow and verify the GitHub Release exists:

```bash
scripts/project-build-monitor-release.sh <project> <project>@<version>
```

Report the version, merged PR, pushed tag, GitHub Actions run URL, and GitHub
Release URL.

Exit codes from the dispatcher:
- `2` no arg and current branch is not `project-<name>`
- `3` project not declared in `.aw/config.toml`
- `4` resolved project has no executable `build.sh`
