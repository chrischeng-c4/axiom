---
name: vat:build:debug
description: Build vat in debug mode and install it to ~/.cargo/bin/vat without bumping version. Use when the user asks for vat debug build, local vat install, or fast iteration build of the vat CLI.
---

# /vat:build:debug

Builds the vat CLI in debug mode and installs `target/debug/vat` to
`~/.cargo/bin/vat`. Does **not** bump the project version, commit, or tag.

## Instructions

Run the build script:

```bash
.agents/skills/vat-build-debug/scripts/build.sh
```

Report the result to the user, including whether `~/.cargo/bin/vat --version`
was printed successfully.
