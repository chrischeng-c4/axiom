#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"
cd "$repo_root"

tested_commit="$(git rev-parse --verify HEAD)"
if [[ ! "$tested_commit" =~ ^[0-9a-f]{40}$ ]]; then
  printf 'expected HEAD to resolve to a complete 40-hex commit, got: %s\n' "$tested_commit" >&2
  exit 1
fi
printf 'tested commit: %s\n' "$tested_commit"

for iteration in {1..20}; do
  printf '\niteration %d/20\n' "$iteration"
  cargo test --locked -p tape --test raft_cluster -- --test-threads=1
  cargo test --locked -p tape --test raft_failover -- --test-threads=1
done
