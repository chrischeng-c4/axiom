#!/usr/bin/env bash
# Run the mamba conformance harness with a per-process CPU-time cap so
# fixtures that hang in tight infinite loops (e.g. broken generator
# semantics) get killed by SIGXCPU instead of stalling the suite.
#
# Usage:
#   projects/mamba/scripts/run-conformance.sh                 # 60s cap, default
#   CAP=120 projects/mamba/scripts/run-conformance.sh         # bump cap
#   FILTER=stdlib projects/mamba/scripts/run-conformance.sh   # only stdlib::* tests
#
# Env knobs:
#   CAP     — per-process CPU-second cap (default 60). Inherited by every
#             mamba subprocess via the kernel; no harness changes needed.
#   FILTER  — substring filter forwarded to `cargo test ... -- <FILTER>`.
#   JOBS    — test-thread count (default unset → cargo's default).
#
# Exit code: cargo's. Stuck mamba subprocesses get SIGXCPU and surface as
# regular test failures, so a hang no longer wedges the whole run.

set -u

CAP="${CAP:-60}"
FILTER="${FILTER:-}"
JOBS="${JOBS:-}"

REPO_ROOT="$(git rev-parse --show-toplevel)"
cd "$REPO_ROOT"

# Reap any stuck mamba subprocesses from a prior aborted run before
# arming the cap — orphaned 100% CPU processes will skew the next run.
stuck=$(pgrep -f "target/debug/mamba run" 2>/dev/null || true)
if [ -n "$stuck" ]; then
  echo "[run-conformance] killing $(echo "$stuck" | wc -l | tr -d ' ') stale mamba subprocess(es)..."
  echo "$stuck" | xargs kill -9 2>/dev/null || true
  sleep 1
fi

echo "[run-conformance] CPU cap = ${CAP}s per subprocess"
if [ -n "$FILTER" ]; then echo "[run-conformance] filter   = ${FILTER}"; fi
if [ -n "$JOBS" ];   then echo "[run-conformance] threads  = ${JOBS}"; fi

# Build test args. Empty FILTER would otherwise be a literal empty arg.
test_args=()
[ -n "$FILTER" ] && test_args+=("$FILTER")
[ -n "$JOBS" ]   && test_args+=("--test-threads=$JOBS")

# ulimit -t is CPU seconds, inherited by every child of this subshell.
# When a mamba run exceeds the cap the kernel sends SIGXCPU; the
# harness sees the non-zero exit and reports a fail, then keeps going.
(
  ulimit -t "$CAP"
  cargo test --quiet -p mamba --test conformance_tests -- "${test_args[@]}"
)
ec=$?

echo "[run-conformance] cargo exit=$ec"
exit $ec
