#!/usr/bin/env bash
# run-all.sh — full mamba test suite with cross-target parallelism.
#
# Serial `cargo test -p mamba` runs each test binary in sequence; the three
# big targets dominate (--lib ~345s, conformance ~437s,
# conformance_cpython_lib_test ~447s ⇒ ~23 min serial). They are mutually
# independent, so running them concurrently bounds wall time by the slowest
# target plus CPU contention. The eleven small targets (~140s serial, mostly
# startup overhead) run as one sequential lane alongside.
#
# Usage:  tests/harness/cpython/tools/run-all.sh   # from projects/mamba
# Env:    MAMBA_ORACLE_CACHE=0 to disable the D5.3 CPython oracle cache.
#
# Exit code is non-zero if any target failed; per-target results and the
# combined log directory are printed at the end.
#
# This is the FULL gate (~3 min). For the fix-iteration inner loop — re-running
# only the failing fixtures or a regression canary in seconds — use
# tests/harness/cpython/tools/sweep.py (gate-parity verdicts, shared oracle
# cache): `sweep.py --failures --filter <cluster>` / `sweep.py --sample 3000`.
set -uo pipefail
cd "$(dirname "$0")/../../../.."

export CC="${CC:-/usr/bin/cc}"
export CXX="${CXX:-/usr/bin/c++}"
export CARGO_TARGET_AARCH64_APPLE_DARWIN_LINKER="${CARGO_TARGET_AARCH64_APPLE_DARWIN_LINKER:-/usr/bin/cc}"

echo "[run-all] building test binaries..."
if ! cargo test -p mamba --no-run 2>&1 | tail -2; then
    echo "[run-all] build failed" >&2
    exit 1
fi

LOGDIR=$(mktemp -d "${TMPDIR:-/tmp}/mamba-run-all.XXXXXX")
START=$(date +%s)
declare -a PIDS NAMES

run_lane() {
    local name="$1"; shift
    (
        cargo test -p mamba "$@" >"$LOGDIR/$name.log" 2>&1
        echo $? >"$LOGDIR/$name.code"
    ) &
    PIDS+=($!)
    NAMES+=("$name")
}

# The big three start first so they dominate (not trail) the wall clock.
run_lane lib --lib
run_lane conformance --test conformance
run_lane cpython_lib_test --test conformance_cpython_lib_test

# Small targets share one sequential lane — running eleven cargo invocations
# concurrently would only fight the big lanes for cores.
(
    code=0
    for t in conformance_pipeline conformance_cpython_grammar \
             conformance_contract conformance_real_world \
             conformance_runtime_shutdown cpython_status \
             mambalibs pkgmgr schema_gates mvp_gates ci_guard; do
        echo "=== $t" >>"$LOGDIR/small.log"
        if ! cargo test -p mamba --test "$t" >>"$LOGDIR/small.log" 2>&1; then
            echo "$t" >>"$LOGDIR/small.failed"
            code=1
        fi
    done
    echo $code >"$LOGDIR/small.code"
) &
PIDS+=($!)
NAMES+=(small)

fail=0
for i in "${!PIDS[@]}"; do
    wait "${PIDS[$i]}" 2>/dev/null
    name="${NAMES[$i]}"
    code=$(cat "$LOGDIR/$name.code" 2>/dev/null || echo 1)
    summary=$(grep -E "test result" "$LOGDIR/$name.log" 2>/dev/null | tail -1)
    printf "%-18s exit=%-3s %s\n" "$name" "$code" "$summary"
    if [ "$name" = small ] && [ -f "$LOGDIR/small.failed" ]; then
        printf "%-18s failed: %s\n" "" "$(tr '\n' ' ' <"$LOGDIR/small.failed")"
    fi
    [ "$code" != "0" ] && fail=1
done

echo "[run-all] wall time: $(( $(date +%s) - START ))s — logs in $LOGDIR"
exit $fail
