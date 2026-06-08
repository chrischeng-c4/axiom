#!/usr/bin/env bash
# Run every conformance fixture through CPython 3.12 to find fixtures
# that don't even pass on CPython — those are fixture bugs (because the
# parity rule is "mamba PASS = matches CPython exactly").
#
# Usage:
#   projects/mamba/scripts/run-cpython-validation.sh
#   FILTER=stdlib projects/mamba/scripts/run-cpython-validation.sh
#   TIMEOUT=20 projects/mamba/scripts/run-cpython-validation.sh
#
# Output: /tmp/cpython_validation.csv with columns:
#   status,exit_code,seconds,relpath
# where status ∈ {PASS, FAIL, TIMEOUT, ERROR}.
#
# Env knobs:
#   FILTER  — substring filter on relative path (e.g. "stdlib", "3p/aio").
#   TIMEOUT — per-fixture timeout in seconds (default 10).
#   JOBS    — parallel workers (default 8).

set -u

FILTER="${FILTER:-}"
TIMEOUT="${TIMEOUT:-10}"
JOBS="${JOBS:-8}"

REPO_ROOT="$(git rev-parse --show-toplevel)"
FIXTURES_ROOT="$REPO_ROOT/projects/mamba/tests/cpython"
OUT_CSV="/tmp/cpython_validation.csv"
OUT_FAIL="/tmp/cpython_validation_failures.txt"

cd "$FIXTURES_ROOT"

echo "[cpython-validate] root    = $FIXTURES_ROOT"
echo "[cpython-validate] timeout = ${TIMEOUT}s per fixture"
echo "[cpython-validate] workers = $JOBS"
[ -n "$FILTER" ] && echo "[cpython-validate] filter  = $FILTER"

# Build the fixture list.
all_fixtures=$(find . -name "*.py" -type f | sed 's|^\./||' | sort)
if [ -n "$FILTER" ]; then
  fixtures=$(echo "$all_fixtures" | grep -F "$FILTER" || true)
else
  fixtures="$all_fixtures"
fi
total=$(echo "$fixtures" | grep -c . || true)
echo "[cpython-validate] total   = $total"
echo ""

# Per-fixture runner — emits one CSV line. Uses a tiny Python wrapper
# for the timeout because macOS lacks GNU coreutils' `timeout` by
# default and we don't want to require a brew install.
export TIMEOUT FIXTURES_ROOT
RUNNER="/tmp/_cpython_runfix.py"
cat > "$RUNNER" <<'PYEOF'
import os, sys, subprocess, time
timeout = float(os.environ["TIMEOUT"])
target = sys.argv[1]
t0 = time.time()
try:
    p = subprocess.run(
        [sys.executable, target],
        capture_output=True, timeout=timeout, check=False,
    )
    ec = p.returncode
    status = "PASS" if ec == 0 else "FAIL"
except subprocess.TimeoutExpired:
    ec = 124
    status = "TIMEOUT"
secs = int(time.time() - t0)
print(f"{status},{ec},{secs},{target}")
PYEOF
export RUNNER

run_one() {
  python3 "$RUNNER" "$1"
}
export -f run_one

> "$OUT_CSV"
echo "$fixtures" | xargs -I{} -P "$JOBS" bash -c 'run_one "$@"' _ {} >> "$OUT_CSV"

echo ""
echo "=== Summary ==="
awk -F, '{c[$1]++} END {for (k in c) printf "  %-8s %d\n", k, c[k]}' "$OUT_CSV" | sort

echo ""
echo "=== Failures by top-level dir ==="
awk -F, '$1 != "PASS" {split($4, a, "/"); c[a[1]]++} END {for (k in c) printf "  %-25s %d\n", k, c[k]}' "$OUT_CSV" | sort -k2 -rn

echo ""
echo "=== Writing failure list to $OUT_FAIL ==="
awk -F, '$1 != "PASS" {print $1 "\t" $4}' "$OUT_CSV" | sort > "$OUT_FAIL"
wc -l "$OUT_FAIL"
