#!/usr/bin/env bash
set -euo pipefail

# Orchestrator: wake the shared cluster once, run each selected app's
# deploy->verify->teardown, and ALWAYS park the pool again — the EXIT trap
# parks on every path, so a failed verify never leaves nodes billing.
#
# env:
#   PROJECT_ID                 required
#   APPS                       subset of "keep defer relay loom" (default: all)
#   <APP>_IMAGE                digest-pinned GHCR ref per selected app,
#                              e.g. KEEP_IMAGE=ghcr.io/<owner>/keep@sha256:...
#   RUN_ID / EVIDENCE_DIR      optional; derived when absent
#   PARK=0                     skip the final park (e.g. the acceptance/gcp
#                              harness is about to run on the same pool)

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
: "${PROJECT_ID:?PROJECT_ID is required}"
APPS="${APPS:-keep defer relay loom}"
RUN_ID="${RUN_ID:-$(date +%Y%m%d%H%M%S)}"
EVIDENCE_DIR="${EVIDENCE_DIR:-/tmp/axiom-gke-harness/$RUN_ID}"
PARK="${PARK:-1}"
export RUN_ID EVIDENCE_DIR
export KUBECONFIG="${KUBECONFIG:-$EVIDENCE_DIR/kubeconfig}"
mkdir -p "$EVIDENCE_DIR"

# Fail on a missing image BEFORE waking the cluster: waking costs minutes and
# a resize, and an unset image variable is a launch mistake, not a test result.
for a in $APPS; do
  case "$a" in
    keep|defer|relay|loom) ;;
    *) echo "unsupported app '$a' in APPS" >&2; exit 2 ;;
  esac
  var="$(printf '%s' "$a" | tr '[:lower:]' '[:upper:]')_IMAGE"
  [[ -n "${!var:-}" ]] || { echo "$var is required for app '$a'" >&2; exit 2; }
done

run_completed=0
cleanup() {
  local rc=$?
  if [[ "$PARK" == "1" ]]; then
    "$SCRIPT_DIR/park.sh" >> "$EVIDENCE_DIR/park.log" 2>&1 \
      || echo "WARNING: park.sh failed; the pool may still hold nodes — check $EVIDENCE_DIR/park.log" >&2
  else
    echo "PARK=0: leaving the pool awake" >&2
  fi
  # The sentinel separates "orchestrator reached the end" from "died mid-run":
  # an rc of 0 without the sentinel would be a false green from a masked exit.
  if (( run_completed == 1 )); then
    exit "$rc"
  fi
  echo "run.sh exited before completion (rc=$rc)" >&2
  exit 1
}
trap cleanup EXIT

"$SCRIPT_DIR/ensure-cluster.sh" > "$EVIDENCE_DIR/cluster-name.txt"

failures=0
results="$EVIDENCE_DIR/results.txt"
: > "$results"
for a in $APPS; do
  var="$(printf '%s' "$a" | tr '[:lower:]' '[:upper:]')_IMAGE"
  echo "=== $a ($(date -u +%H:%M:%SZ)) ===" >&2
  if IMAGE="${!var}" "$SCRIPT_DIR/run-app.sh" "$a"; then
    echo "$a PASS" >> "$results"
  else
    echo "$a FAIL" >> "$results"
    failures=$((failures + 1))
  fi
done

echo "--- results ---"
cat "$results"
run_completed=1
(( failures == 0 )) || exit 1
