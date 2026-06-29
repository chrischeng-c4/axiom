---
name: jet:build:release
description: Bump jet's patch version, build release, install, commit, and tag jet@<version>
user-invocable: true
---

# /jet:build:release

Cuts a local jet release via jet's canonical `projects/jet/build.sh release`:
bumps the patch version in `projects/jet/Cargo.toml` (base-64: minor/patch
0–63 with carry), builds jet in **release** mode (`cargo build --release`),
installs to `~/.cargo/bin/jet`, commits the version files, and creates the
`jet@<version>` git tag.

It does **not** push. Pushing the branch and the `jet@<version>` tag is what
publishes the release: `.github/workflows/jet-release.yml` builds the
cross-platform binaries (macOS arm64 + Linux x64/arm64) when the tag is pushed.

## Instructions

Run the release wrapper (delegates to `projects/jet/build.sh release`):

```bash
.claude/skills/jet-build-release/scripts/release.sh
```

Then report the new version + `jet@<version>` tag, and the push commands the
script prints:

```bash
git push origin HEAD
git push origin jet@<version>
```
