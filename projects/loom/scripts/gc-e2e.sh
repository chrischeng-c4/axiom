#!/usr/bin/env bash
# loom completed-DAG GC e2e (#106): boot relay+keep+loom (LOOM_GC_RETENTION_SECS=5)
# + a worker, run an echo workflow to succeeded, then verify the run is reaped
# from the store after the retention window (GET /runs/{id} -> 404).
set -euo pipefail
cd "$(dirname "$0")/../../.."
rm -rf /tmp/loom-gc; mkdir -p /tmp/loom-gc
RELAY_BIND=127.0.0.1:7431 RELAY_DATA_DIR=/tmp/loom-gc/relay ./target/release/relay-server >/tmp/loom-gc/relay.log 2>&1 & R=$!
./target/release/keep --host 127.0.0.1 --port 7396 >/tmp/loom-gc/keep.log 2>&1 & K=$!
LOOM_ADDR=127.0.0.1:7497 LOOM_RELAY=http://127.0.0.1:7431 LOOM_GC_RETENTION_SECS=5 LOOM_COMPLETION_SHARDS=1 ./target/release/loom controller >/tmp/loom-gc/ctl.log 2>&1 & C=$!
LOOM_RELAY=http://127.0.0.1:7431 LOOM_KEEP=http://127.0.0.1:7396 LOOM_RUNNER=resident LOOM_COMPLETION_SHARDS=1 ./target/release/loom worker >/tmp/loom-gc/w.log 2>&1 & W=$!
trap 'kill $R $K $C $W 2>/dev/null' EXIT
for i in $(seq 1 40); do curl -sf http://127.0.0.1:7497/healthz>/dev/null 2>&1 && curl -sf http://127.0.0.1:7396/healthz>/dev/null 2>&1 && break; sleep 0.5; done
curl -s -X POST http://127.0.0.1:7497/runs -H 'content-type: application/json' -d '{"run_id":"gcr","nodes":[{"id":"a","task_name":"echo"}]}' >/dev/null
for i in $(seq 1 20); do s=$(curl -s http://127.0.0.1:7497/runs/gcr|python3 -c 'import json,sys;print(json.load(sys.stdin).get("status",""))' 2>/dev/null); [ "$s" = "succeeded" ]&&break; sleep 0.5; done
echo "run reached: $s"; code=$(curl -s -o /dev/null -w '%{http_code}' http://127.0.0.1:7497/runs/gcr); echo "GET /runs/gcr right after success: HTTP $code (expect 200)"
echo "waiting for GC retention(5s)+sweep..."; sleep 13
code=$(curl -s -o /dev/null -w '%{http_code}' http://127.0.0.1:7497/runs/gcr); echo "GET /runs/gcr after GC: HTTP $code (expect 404 = reaped)"
grep -i 'GC' /tmp/loom-gc/ctl.log | head -1
[ "$code" = "404" ] && echo "PASS: completed run was GC'd from the store" || echo "FAIL: run not reaped"
