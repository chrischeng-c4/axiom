#!/usr/bin/env bash
set -euo pipefail

# /jet:build:debug — thin wrapper over jet's canonical build script.
# projects/jet/build.sh owns toolchain selection, the debug build, and install.
cd "$(git rev-parse --show-toplevel)"
exec projects/jet/build.sh debug
