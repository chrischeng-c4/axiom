#!/usr/bin/env bash
# loom LITERAL 1M-row CSV e2e (#111): generate a 1,000,000-row CSV, store it in
# keep (claim-check), submit one csv-split → 100 chunks of 10k rows fanned out
# to 100 csv-process across 8 workers; verify total rows processed == 1,000,000.
# Uses release binaries + KEEP_BODY_LIMIT for the 16MB blob + LOOM_CSV_CHUNK_ROWS.
set -euo pipefail
cd "$(dirname "$0")/../../.."
ROWS=1000000; CHUNK=10000; WORKERS=8
rm -rf /tmp/loom-1m; mkdir -p /tmp/loom-1m
RELAY_BIND=127.0.0.1:7411 RELAY_DATA_DIR=/tmp/loom-1m/relay ./target/release/relay-server >/tmp/loom-1m/relay.log 2>&1 & R=$!
KEEP_BODY_LIMIT=134217728 ./target/release/keep --host 127.0.0.1 --port 7390 >/tmp/loom-1m/keep.log 2>&1 & K=$!
LOOM_ADDR=127.0.0.1:7486 LOOM_RELAY=http://127.0.0.1:7411 ./target/release/loom controller >/tmp/loom-1m/ctl.log 2>&1 & C=$!
WP=(); for i in $(seq 1 $WORKERS); do LOOM_RELAY=http://127.0.0.1:7411 LOOM_KEEP=http://127.0.0.1:7390 LOOM_RUNNER=resident LOOM_CSV_CHUNK_ROWS=$CHUNK ./target/release/loom worker >/tmp/loom-1m/w$i.log 2>&1 & WP+=($!); done
trap 'kill $R $K $C ${WP[*]} 2>/dev/null' EXIT
for i in $(seq 1 30); do curl -sf http://127.0.0.1:7486/healthz>/dev/null 2>&1 && curl -sf http://127.0.0.1:7390/healthz>/dev/null 2>&1 && break; sleep 0.5; done
echo "=== generating $ROWS-row CSV and PUTting to keep ==="
gent0=$(python3 -c 'import time;print(time.time())')
python3 -c "import sys;w=sys.stdout.write;[w(f'row{i},{i}\n') for i in range($ROWS)]" > /tmp/loom-1m/big.csv
echo "CSV size: $(du -h /tmp/loom-1m/big.csv | cut -f1), gen $(python3 -c "import time;print(f'{time.time()-$gent0:.1f}s')")"
curl -s -X PUT http://127.0.0.1:7390/v1/inputs/onemillion -H 'content-type: application/octet-stream' --data-binary @/tmp/loom-1m/big.csv -w 'PUT [%{http_code}]'; echo
echo "=== submit csv-split over 1M rows → $((ROWS/CHUNK)) chunks of $CHUNK rows, $WORKERS workers ==="
start=$(python3 -c 'import time;print(time.time())')
curl -s -X POST http://127.0.0.1:7486/runs -H 'content-type: application/json' -d '{"run_id":"onemil","nodes":[{"id":"reader","task_name":"csv-split","input_refs":["onemillion"]}]}' >/dev/null
for i in $(seq 1 300); do
  out=$(curl -s http://127.0.0.1:7486/runs/onemil 2>/dev/null | python3 -c 'import json,sys
d=json.load(sys.stdin);print(d["status"],len(d["nodes"]))' 2>/dev/null)
  st=${out%% *}; nn=${out##* }
  if [ "$st" = "succeeded" ]; then end=$(python3 -c 'import time;print(time.time())'); echo "SUCCEEDED: $nn nodes in $(python3 -c "print(f'{$end-$start:.1f}s')") ($(python3 -c "print(f'{$ROWS/($end-$start):.0f}')") rows/s incl. split+fan-out+process)"; break; fi
  [ "$st" = "failed" ] && { echo "FAILED"; cat /tmp/loom-1m/ctl.log|tail -5; exit 1; }
  sleep 1
done
echo "=== verify total rows processed = sum of chunk row-counts ==="
python3 - <<PY
import urllib.request,json
tot=0; nc=$((ROWS/CHUNK))
for c in range(nc):
    try: tot+=int(urllib.request.urlopen(f"http://127.0.0.1:7390/v1/results/onemil:rows-{c}:result",timeout=5).read())
    except Exception as e: print("miss",c,e)
print(f"chunks={nc} total_rows_processed={tot} (expected {$ROWS}) -> {'OK' if tot==$ROWS else 'MISMATCH'}")
PY
