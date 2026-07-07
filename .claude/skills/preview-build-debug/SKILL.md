---
name: preview:build:debug
description: Build preview in debug mode and install it to ~/.cargo/bin/preview. Use when the user asks for preview debug build, local preview install, or fast iteration build of the preview CLI.
user-invocable: true
---

# /preview:build:debug

Builds the preview CLI in debug mode and installs `target/debug/preview` to
`~/.cargo/bin/preview` using the project-owned build script.

## Instructions

Run the build script:

```bash
.claude/skills/preview-build-debug/scripts/build.sh
```

Report the result to the user, including whether `~/.cargo/bin/preview --version`
was printed successfully.
