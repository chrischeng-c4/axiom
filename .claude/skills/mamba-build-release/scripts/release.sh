#!/usr/bin/env bash
set -euo pipefail

cd "$(git rev-parse --show-toplevel)"
exec projects/mamba/build.sh release
