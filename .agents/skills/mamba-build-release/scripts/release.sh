#!/usr/bin/env bash
set -euo pipefail

cd "$(git rev-parse --show-toplevel)"
exec apps/mamba/build.sh release
