#!/usr/bin/env bash
# schema-layer bidi e2e (#440/#442): relay + keep + loom controller + loom
# schema-layer + a bidi worker. Submit an echo run; the worker consumes a
# self-describing Task envelope over the bidi stream, GET/PUTs keep at the given
# URLs, sends Done; the schema layer forwards the completion; the controller folds.
# Verifies the worker round-trips through the schema layer end-to-end.
set -uo pipefail
cd "$(dirname "$0")/../../.."
d=/tmp/loom-sl; rm -rf $d; mkdir -p $d
RELAY=http://127.0.0.1:7451; KEEP=http://127.0.0.1:7452; LOOM=http://127.0.0.1:7453; SL=http://127.0.0.1:7454
RELAY_BIND=127.0.0.1:7451 RELAY_DATA_DIR=$d/relay ./target/release/relay-server >$d/relay.log 2>&1 &
KEEP_DATA_DIR=$d/keep ./target/release/keep --host 127.0.0.1 --port 7452 >$d/keep.log 2>&1 &
LOOM_ADDR=127.0.0.1:7453 LOOM_RELAY=$RELAY LOOM_COMPLETION_SHARDS=8 LOOM_GC_RETENTION_SECS=0 ./target/release/loom controller >$d/ctl.log 2>&1 &
LOOM_ADDR=127.0.0.1:7454 LOOM_RELAY=$RELAY LOOM_KEEP=$KEEP LOOM_COMPLETION_SHARDS=8 ./target/release/loom schema-layer >$d/sl.log 2>&1 &
LOOM_SCHEMA_LAYER=$SL LOOM_KEEP=$KEEP LOOM_RUNNER=resident LOOM_WORKER_CONCURRENCY=4 ./target/release/loom worker >$d/worker.log 2>&1 &
trap 'pkill -f target/release/loom; pkill -f target/release/relay-server; pkill -f target/release/keep' EXIT
for i in $(seq 1 40); do curl -sf $LOOM/healthz>/dev/null 2>&1 && curl -sf $SL/healthz>/dev/null 2>&1 && curl -sf $KEEP/healthz>/dev/null 2>&1 && break; sleep 0.3; done
echo "=== PUT input + submit an echo run through the schema layer ==="
printf 'HELLO-SCHEMA' | curl -s -X PUT $KEEP/v1/inputs/slin -H 'content-type: application/octet-stream' --data-binary @- -w ' [%{http_code}]'; echo
curl -s -X POST $LOOM/runs -H 'content-type: application/json' -d '{"run_id":"sl1","nodes":[{"id":"a","task_name":"echo","input_refs":["slin"]}]}' -w ' submit[%{http_code}]'; echo
echo "=== poll: worker(bidi) ⟷ schema-layer ⟷ relay; Done→controller folds ==="
for i in $(seq 1 30); do
  s=$(curl -s $LOOM/runs/sl1 | python3 -c 'import json,sys;print(json.load(sys.stdin).get("status",""))' 2>/dev/null)
  echo "  t=$i: $s"; [ "$s" = succeeded ] && break; [ "$s" = failed ] && { echo FAILED; break; }; sleep 1
done
echo "=== schema-layer log tail ==="; tail -3 $d/sl.log; echo "=== worker log tail ==="; tail -3 $d/worker.log
