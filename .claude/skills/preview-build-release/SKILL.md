---
name: preview:build:release
description: Build preview in release mode and install it to ~/.cargo/bin/preview. Use when the user asks for a local preview release-profile build.
user-invocable: true
---

# /preview:build:release

Builds the preview CLI with the release cargo profile and installs
`target/release/preview` to `~/.cargo/bin/preview` using the project-owned build
script. This is a local release-profile build/install path; `apps/preview`
does not yet have the release-prep commit/tag/GitHub Release contract used by
the published project release skills.

## Instructions

Run the release build script:

```bash
.claude/skills/preview-build-release/scripts/release.sh
```

Report the result to the user, including whether `~/.cargo/bin/preview --version`
was printed successfully.
