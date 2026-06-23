#!/usr/bin/env bash
# loom throughput micro-benchmark (#127): boot relay + keep + a loom controller
# + N resident workers, submit M single-node `echo` runs, and measure wall-clock
# from first submit to all-succeeded → runs/sec end-to-end through the real
# broker/store. A starting point for a perf gate (set a floor in CI).
#
# Usage: bench.sh [M_runs] [N_workers]   (defaults: 200 runs, 4 workers)
# Requires built binaries: target/debug/{relay-server,keep,loom}.
set -euo pipefail
cd "$(dirname "$0")/../../.."

M=${1:-200}
N=${2:-4}
DATA=$(mktemp -d)
RELAY=http://127.0.0.1:7404
KEEPB=http://127.0.0.1:7383
LOOM=http://127.0.0.1:7479

RELAY_BIND=127.0.0.1:7404 RELAY_DATA_DIR="$DATA/relay" ./target/debug/relay-server >/tmp/loom-bench-relay.log 2>&1 & R=$!
./target/debug/keep --host 127.0.0.1 --port 7383 >/tmp/loom-bench-keep.log 2>&1 & K=$!
LOOM_ADDR=127.0.0.1:7479 LOOM_RELAY=$RELAY ./target/debug/loom controller >/tmp/loom-bench-ctl.log 2>&1 & C=$!
WPIDS=()
for i in $(seq 1 "$N"); do
  LOOM_RELAY=$RELAY LOOM_KEEP=$KEEPB LOOM_RUNNER=resident ./target/debug/loom worker >/tmp/loom-bench-wrk-$i.log 2>&1 & WPIDS+=($!)
done
trap 'kill $R $K $C ${WPIDS[*]} 2>/dev/null' EXIT

for _ in $(seq 1 40); do curl -sf "$LOOM/healthz" >/dev/null 2>&1 && curl -sf "$KEEPB/healthz" >/dev/null 2>&1 && break; sleep 0.5; done

echo "submitting $M runs across $N workers..."
start=$(python3 -c 'import time;print(time.time())')
for i in $(seq 1 "$M"); do
  curl -s -X POST "$LOOM/runs" -H 'content-type: application/json' \
    -d "{\"run_id\":\"b$i\",\"nodes\":[{\"id\":\"t\",\"task_name\":\"echo\"}]}" >/dev/null
done
submitted=$(python3 -c 'import time;print(time.time())')

# wait until all M runs are succeeded
done_count=0
for _ in $(seq 1 600); do
  done_count=$(curl -s "$LOOM/runs/b$M" >/dev/null 2>&1; \
    python3 - "$LOOM" "$M" <<'PY'
import sys,urllib.request,json
base,m=sys.argv[1],int(sys.argv[2])
n=0
for i in range(1,m+1):
    try:
        d=json.load(urllib.request.urlopen(f"{base}/runs/b{i}",timeout=2))
        if d.get("status")=="succeeded": n+=1
    except Exception: pass
print(n)
PY
)
  [ "$done_count" -ge "$M" ] && break
  sleep 0.5
done
end=$(python3 -c 'import time;print(time.time())')

python3 - "$start" "$submitted" "$end" "$M" "$N" "$done_count" <<'PY'
import sys
start,submitted,end,m,n,done=map(float,sys.argv[1:7])
m=int(m); done=int(done)
print(f"workers={int(n)} runs={m} succeeded={done}")
print(f"submit time:    {submitted-start:.2f}s ({m/(submitted-start):.0f} submits/s)")
print(f"end-to-end:     {end-start:.2f}s ({done/(end-start):.0f} runs/s completed)")
PY
