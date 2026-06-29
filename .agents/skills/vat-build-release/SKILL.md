---
name: vat:build:release
description: Bump vat patch version, build the release binary, install it to ~/.cargo/bin/vat, commit version files, create the vat release tag, and push the branch/tag to origin. Use when the user asks to release vat or run a vat release build.
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
- pushes the release commit and tag to `origin`

## Instructions

Run the release script:

```bash
.agents/skills/vat-build-release/scripts/release.sh
```

Report the installed version, commit, tag, and pushed remote to the user.
