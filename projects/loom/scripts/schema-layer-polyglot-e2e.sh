#!/usr/bin/env bash
# Polyglot schema-layer conformance e2e (#442): relay + keep + loom controller +
# loom schema-layer + a PYTHON bidi worker (tests/integration/bidi_worker.py) that
# round-trips over a cleartext h2c bidi stream. Submits two tasks; the Python
# worker consumes self-describing envelopes, does keep I/O at the given URLs,
# sends Done; the schema layer forwards completions; the controller folds → run
# succeeds. Proves the no-SDK polyglot contract end to end.
#
# Python deps (h2 + httpx) auto-installed into a throwaway venv.
set -uo pipefail
cd "$(dirname "$0")/../../.."
d=/tmp/loom-poly-bidi; rm -rf "$d"; mkdir -p "$d"
RELAY=http://127.0.0.1:7461; KEEP=http://127.0.0.1:7462; LOOM=http://127.0.0.1:7463; SL=http://127.0.0.1:7464

VENV=/tmp/loom-pyv
[ -d "$VENV" ] || python3 -m venv "$VENV"
"$VENV/bin/python" -c 'import h2, httpx' 2>/dev/null || "$VENV/bin/pip" install -q h2 httpx

RELAY_BIND=127.0.0.1:7461 RELAY_DATA_DIR=$d/relay ./target/release/relay-server >$d/relay.log 2>&1 &
KEEP_DATA_DIR=$d/keep ./target/release/keep --host 127.0.0.1 --port 7462 >$d/keep.log 2>&1 &
LOOM_ADDR=127.0.0.1:7463 LOOM_RELAY=$RELAY LOOM_COMPLETION_SHARDS=8 LOOM_GC_RETENTION_SECS=0 ./target/release/loom controller >$d/ctl.log 2>&1 &
LOOM_ADDR=127.0.0.1:7464 LOOM_RELAY=$RELAY LOOM_KEEP=$KEEP LOOM_COMPLETION_SHARDS=8 ./target/release/loom schema-layer >$d/sl.log 2>&1 &
trap 'pkill -f target/release/loom; pkill -f target/release/relay-server; pkill -f target/release/keep; pkill -f bidi_worker.py' EXIT
for i in $(seq 1 40); do curl -sf $LOOM/healthz>/dev/null 2>&1 && curl -sf $SL/healthz>/dev/null 2>&1 && curl -sf $KEEP/healthz>/dev/null 2>&1 && break; sleep 0.3; done

"$VENV/bin/python" projects/loom/tests/integration/bidi_worker.py 7464 >$d/pyworker.log 2>&1 &
sleep 1
echo "=== submit two tasks (upper + echo) through the schema layer to the Python bidi worker ==="
printf 'hello world' | curl -s -X PUT $KEEP/v1/inputs/g -H 'content-type: application/octet-stream' --data-binary @- >/dev/null
curl -s -X POST $LOOM/runs -H 'content-type: application/json' \
  -d '{"run_id":"p1","nodes":[{"id":"a","task_name":"upper","input_refs":["g"]},{"id":"b","task_name":"echo","input_refs":["g"]}]}' -w ' submit[%{http_code}]'; echo
ok=0
for i in $(seq 1 30); do
  s=$(curl -s $LOOM/runs/p1 | python3 -c 'import json,sys;print(json.load(sys.stdin).get("status",""))' 2>/dev/null)
  [ "$s" = succeeded ] && { ok=1; break; }; [ "$s" = failed ] && break; sleep 1
done
echo "=== python worker log ==="; cat $d/pyworker.log
[ "$ok" = 1 ] && echo "PASS: polyglot Python bidi worker round-tripped through the schema layer" || { echo "FAIL ($s)"; exit 1; }
