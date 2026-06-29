#!/usr/bin/env bash
set -euo pipefail

# /jet:build:release — thin wrapper over jet's canonical release script.
# projects/jet/build.sh release owns: bump projects/jet/Cargo.toml patch version,
# release build, install, commit, and tag jet@<version> (it does not push).
cd "$(git rev-parse --show-toplevel)"
exec projects/jet/build.sh release
