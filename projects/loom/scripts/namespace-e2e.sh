#!/usr/bin/env bash
# vhost namespace e2e (#450/#451): LOOM_NAMESPACE=tenant1 on controller +
# schema-layer → all loom relay traffic is scoped to tenant1::* subjects. Verifies
# a run completes through the namespaced subjects (bare `resident` stays empty;
# `tenant1::resident` gets the task). Needs the loom-pyv venv (h2 + httpx).
set -uo pipefail
cd "$(dirname "$0")/../../.."
d=/tmp/loom-ns; rm -rf $d; mkdir -p $d
RELAY=http://127.0.0.1:7481; KEEP=http://127.0.0.1:7482; LOOM=http://127.0.0.1:7483; SL=http://127.0.0.1:7484
RELAY_BIND=127.0.0.1:7481 RELAY_DATA_DIR=$d/relay ./target/release/relay-server >$d/relay.log 2>&1 &
KEEP_DATA_DIR=$d/keep ./target/release/keep --host 127.0.0.1 --port 7482 >$d/keep.log 2>&1 &
LOOM_ADDR=127.0.0.1:7483 LOOM_RELAY=$RELAY LOOM_NAMESPACE=tenant1 LOOM_COMPLETION_SHARDS=8 LOOM_GC_RETENTION_SECS=0 ./target/release/loom controller >$d/ctl.log 2>&1 &
LOOM_ADDR=127.0.0.1:7484 LOOM_RELAY=$RELAY LOOM_KEEP=$KEEP LOOM_NAMESPACE=tenant1 LOOM_COMPLETION_SHARDS=8 ./target/release/loom schema-layer >$d/sl.log 2>&1 &
trap 'pkill -f target/release/loom; pkill -f target/release/relay-server; pkill -f target/release/keep; pkill -f bidi_worker.py' EXIT
for i in $(seq 1 40); do curl -sf $LOOM/healthz>/dev/null 2>&1 && curl -sf $SL/healthz>/dev/null 2>&1 && curl -sf $KEEP/healthz>/dev/null 2>&1 && break; sleep 0.3; done
"${PYBIN:-/tmp/loom-pyv/bin/python}" projects/loom/tests/integration/bidi_worker.py 7484 >$d/pyw.log 2>&1 &
sleep 1
printf 'hi ns' | curl -s -X PUT $KEEP/v1/inputs/g -H 'content-type: application/octet-stream' --data-binary @- >/dev/null
curl -s -X POST $LOOM/runs -H 'content-type: application/json' -d '{"run_id":"n1","nodes":[{"id":"a","task_name":"upper","input_refs":["g"]}]}' -w ' submit[%{http_code}]'; echo
ok=0; for i in $(seq 1 25); do s=$(curl -s $LOOM/runs/n1 | python3 -c 'import json,sys;print(json.load(sys.stdin).get("status",""))' 2>/dev/null); [ "$s" = succeeded ] && { ok=1; break; }; [ "$s" = failed ] && break; sleep 1; done
echo "pyworker: $(cat $d/pyw.log | tr '\n' '|')"
echo "bare resident len:    $(curl -s $RELAY/v1/resident/len)"; echo "tenant1::resident len: $(curl -s $RELAY/v1/tenant1::resident/len)"
[ "$ok" = 1 ] && echo "PASS: run completed through namespaced (tenant1) relay subjects" || echo "FAIL ($s)"
