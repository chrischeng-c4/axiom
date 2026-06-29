---
name: lumen:build:release
description: Bump lumen patch version, build the release binary, install it to ~/.cargo/bin/lumen, commit version files, and create the lumen release tag. Use when the user asks to release lumen or run a lumen release build.
---

# /lumen:build:release

Cuts a lumen release using the project-owned release path:

- bumps `projects/lumen/Cargo.toml` patch version with the repository base-64
  patch/minor carry convention
- syncs `Cargo.lock`
- builds `lumen` with the release cargo profile and published CLI ops features
- installs `target/release/lumen` to `~/.cargo/bin/lumen`
- commits the version files
- creates the `lumen@<version>` annotated tag

## Instructions

Run the release script:

```bash
.agents/skills/lumen-build-release/scripts/release.sh
```

Report the installed version, commit, and tag to the user.
