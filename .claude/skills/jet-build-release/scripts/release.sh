#!/usr/bin/env bash
set -euo pipefail

# /jet:build:release step 1 (release-prep) — thin wrapper over jet's canonical
# build script. `projects/jet/build.sh release` bumps projects/jet/Cargo.toml,
# builds --release, installs to ~/.cargo/bin/jet, and commits
# `release(jet): jet@<version>`. It does NOT tag or push.
#
# The /jet:build:release skill then lands the commit to main via /git:land,
# tags + pushes after landing, and monitors GitHub release publication (see
# SKILL.md) — tagging before the land would orphan the tag off main.
cd "$(git rev-parse --show-toplevel)"
exec projects/jet/build.sh release
