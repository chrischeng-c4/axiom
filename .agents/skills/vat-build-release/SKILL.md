---
name: vat:build:release
description: Bump vat patch version, build the release binary, install it to ~/.cargo/bin/vat, commit version files, and create the vat release tag. Use when the user asks to release vat or run a vat release build.
---

# /vat:build:release

Cuts a vat release using the project-owned release path:

- bumps `projects/vat/Cargo.toml` patch version with the repository base-64
  patch/minor carry convention
- syncs `Cargo.lock`
- builds `vat` with the release cargo profile
- installs `target/release/vat` to `~/.cargo/bin/vat`
- commits the version files
- creates the `vat@<version>` annotated tag

## Instructions

Run the release script:

```bash
.agents/skills/vat-build-release/scripts/release.sh
```

Report the installed version, commit, and tag to the user.
