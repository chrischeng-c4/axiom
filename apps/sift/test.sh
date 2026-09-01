#!/usr/bin/env bash
set -euo pipefail

repo_root="$(git -c core.fsmonitor=false rev-parse --show-toplevel)"
cd "$repo_root"
exec cargo test -p sift "$@"
