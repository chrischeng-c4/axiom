#!/usr/bin/env bash
set -euo pipefail

# /cap:build:release step 1 (release-prep) — thin wrapper over cap's canonical
# build script. It prepares the release commit and prints RELEASE_TAG. The skill
# lands first, then tags + pushes the landed commit.
ROOT="$(git rev-parse --show-toplevel)"
cd "$ROOT"

if [[ -z "${CAP_INSTALL:-}" ]]; then
  active_cap="$(command -v cap 2>/dev/null || true)"
  if [[ -n "$active_cap" ]]; then
    export CAP_INSTALL="$(dirname "$active_cap")"
  fi
fi

export CC="${CC:-/usr/bin/cc}"
export CARGO_TARGET_AARCH64_APPLE_DARWIN_LINKER="${CARGO_TARGET_AARCH64_APPLE_DARWIN_LINKER:-/usr/bin/cc}"

exec projects/cap/build.sh release
