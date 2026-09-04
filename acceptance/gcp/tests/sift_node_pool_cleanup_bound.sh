#!/usr/bin/env bash
set -euo pipefail

test_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
acceptance_root="$(cd "$test_dir/.." && pwd)"
node_pool_file="$acceptance_root/environment/gke.tf"
cleanup_script="$acceptance_root/scripts/cleanup.sh"

sift_node_pool="$({
  awk '
    /^resource "google_container_node_pool" "sift_mvp" \{/ {
      found = 1
    }
    found {
      print
      line = $0
      opens += gsub(/\{/, "{", line)
      line = $0
      closes += gsub(/\}/, "}", line)
      if (opens > 0 && opens == closes) {
        exit
      }
    }
  ' "$node_pool_file"
} )"

[[ -n "$sift_node_pool" ]] || {
  echo "the run-scoped Sift node pool is missing" >&2
  exit 1
}
grep -Eq '^[[:space:]]*timeouts[[:space:]]*\{' <<<"$sift_node_pool" || {
  echo "the Sift node pool has no explicit operation timeouts" >&2
  exit 1
}
grep -Eq '^[[:space:]]*delete[[:space:]]*=[[:space:]]*"10m"[[:space:]]*$' \
  <<<"$sift_node_pool" || {
  echo "the Sift node pool delete wait is not bounded to ten minutes" >&2
  exit 1
}
rg -F -- 'for attempt in 1 2 3; do' "$cleanup_script" >/dev/null || {
  echo "cleanup no longer retries a timed-out Terraform destroy" >&2
  exit 1
}

echo "Sift node-pool cleanup bound: ok"
