#!/usr/bin/env bash
# loom CSV scale demo (#111): one csv-split run dynamically fans out ceil(N/2)
# csv-process chunks, processed by W resident workers; reports node count +
# wall-clock. Demonstrates within-run dynamic fan-out scaling. 1M rows is the
# same path at data volume (larger chunks + a release build). Usage: csv-scale.sh [ROWS] [WORKERS]
set -euo pipefail
cd "$(dirname "$0")/../../.."
ROWS=${1:-200}; WORKERS=${2:-4}
rm -rf /tmp/loom-relay-sc; mkdir -p /tmp/loom-relay-sc
RELAY_BIND=127.0.0.1:7407 RELAY_DATA_DIR=/tmp/loom-relay-sc ./target/debug/relay-server >/tmp/loom-sc-relay.log 2>&1 & R=$!
./target/debug/keep --host 127.0.0.1 --port 7386 >/tmp/loom-sc-keep.log 2>&1 & K=$!
LOOM_ADDR=127.0.0.1:7484 LOOM_RELAY=http://127.0.0.1:7407 ./target/debug/loom controller >/tmp/loom-sc-ctl.log 2>&1 & C=$!
WP=()
for i in $(seq 1 $WORKERS); do LOOM_RELAY=http://127.0.0.1:7407 LOOM_KEEP=http://127.0.0.1:7386 LOOM_RUNNER=resident ./target/debug/loom worker >/tmp/loom-sc-w$i.log 2>&1 & WP+=($!); done
trap 'kill $R $K $C ${WP[*]} 2>/dev/null' EXIT
for i in $(seq 1 30); do curl -sf http://127.0.0.1:7484/healthz>/dev/null 2>&1 && curl -sf http://127.0.0.1:7386/healthz>/dev/null 2>&1 && break; sleep 0.5; done
# build an N-row CSV and PUT it to keep
python3 -c "import sys;sys.stdout.write(''.join(f'row{i},{i}\n' for i in range($ROWS)))" | curl -s -X PUT http://127.0.0.1:7386/v1/inputs/bigcsv -H 'content-type: application/octet-stream' --data-binary @- >/dev/null
CHUNKS=$(( (ROWS+1)/2 ))
echo "=== scale: $ROWS-row CSV → $CHUNKS chunks (2 rows each), $WORKERS workers ==="
start=$(python3 -c 'import time;print(time.time())')
curl -s -X POST http://127.0.0.1:7484/runs -H 'content-type: application/json' -d '{"run_id":"scale","nodes":[{"id":"reader","task_name":"csv-split","input_refs":["bigcsv"]}]}' >/dev/null
for i in $(seq 1 120); do
  out=$(curl -s http://127.0.0.1:7484/runs/scale 2>/dev/null | python3 -c 'import json,sys
d=json.load(sys.stdin); print(d["status"], len(d["nodes"]))' 2>/dev/null)
  st=$(echo "$out"|cut -d' ' -f1); nn=$(echo "$out"|cut -d' ' -f2)
  [ "$st" = "succeeded" ] && { end=$(python3 -c 'import time;print(time.time())'); echo "SUCCEEDED: $nn nodes ($((nn-1)) chunks + reader) in $(python3 -c "print(f'{$end-$start:.1f}s')")"; break; }
  [ "$st" = "failed" ] && { echo "FAILED"; exit 1; }
  sleep 1
done
echo "=== spot-check a few chunk results (row counts) ==="
for c in 0 10 $((CHUNKS-1)); do echo -n "rows-$c = "; curl -s http://127.0.0.1:7386/v1/results/scale:rows-$c:result; echo; done
