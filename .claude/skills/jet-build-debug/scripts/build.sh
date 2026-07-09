#!/usr/bin/env bash
set -euo pipefail

# /jet:build:debug — thin wrapper over jet's canonical build script.
# apps/jet/build.sh owns toolchain selection, the debug build, and install.
cd "$(git rev-parse --show-toplevel)"
exec apps/jet/build.sh debug
