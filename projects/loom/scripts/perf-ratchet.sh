#!/usr/bin/env bash
# loom perf ratchet (#127): run the loom throughput bench, compare end-to-end
# runs/s to the committed baseline (docs/benchmark/baseline.json), and:
#   - FAIL (exit 1) if throughput regressed more than TOLERANCE below baseline
#     (the CI gate — blocks merges that slow loom down),
#   - RATCHET the baseline UP when throughput improves (records the new floor).
# A frontier comparison vs Celery + Temporal lives in docs/benchmark/frontier.md;
# this gate ratchets loom against its own best, the dormant axis competitors don't
# cover. bash 3.2-safe.
set -euo pipefail
cd "$(dirname "$0")/../../.."
BASELINE=projects/loom/docs/benchmark/baseline.json
TOLERANCE=${PERF_TOLERANCE:-0.20}   # allow 20% noise on a dev box
RUNS=${1:-200}; WORKERS=${2:-4}

echo "=== measuring loom throughput (bench $RUNS runs / $WORKERS workers) ==="
out=$(./projects/loom/scripts/bench.sh "$RUNS" "$WORKERS" 2>/dev/null | tr -d '\r')
echo "$out"
now=$(printf '%s\n' "$out" | sed -n 's/.*(\([0-9][0-9]*\) runs\/s completed.*/\1/p' | tail -1)
[ -n "$now" ] || { echo "could not parse runs/s from bench output"; exit 2; }

python3 - "$BASELINE" "$now" "$TOLERANCE" <<'PY'
import json, os, sys
baseline_path, now, tol = sys.argv[1], int(sys.argv[2]), float(sys.argv[3])
base = None
if os.path.exists(baseline_path):
    base = json.load(open(baseline_path)).get("loom_runs_per_sec")
floor = int(base * (1 - tol)) if base else 0
status = "OK"
keep = base
if base is None:
    print(f"no baseline yet -> seeding loom_runs_per_sec={now}")
    keep = now
elif now < floor:
    print(f"REGRESSION: {now} runs/s < floor {floor} (baseline {base}, tol {tol:.0%})")
    status = "FAIL"
elif now > base:
    print(f"IMPROVED: {now} > baseline {base} -> ratcheting baseline up")
    keep = now
else:
    print(f"OK: {now} runs/s within {tol:.0%} of baseline {base}")
json.dump({"loom_runs_per_sec": keep, "tolerance": tol}, open(baseline_path, "w"), indent=2)
open(baseline_path, "a").write("\n")
sys.exit(1 if status == "FAIL" else 0)
PY
