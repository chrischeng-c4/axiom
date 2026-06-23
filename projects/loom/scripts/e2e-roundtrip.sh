#!/usr/bin/env bash
# loom end-to-end demo: start relay + keep + a durable loom controller + a loom
# worker, then drive three workflows through the REAL broker/store and verify
# each reaches `succeeded`:
#   1. chain   a → b
#   2. chord   root → {x,y,z} → reduce   (fan-out + 3-way fan-in barrier, #116)
#   3. data    a node whose input is a real claim-check payload in keep, echoed
#              back out to keep and read by the client (#111 data path)
#
# Requires built binaries: target/debug/{relay-server,keep,loom}
# (cargo build -p relay --bin relay-server; cargo build -p keep --bin keep;
#  cargo build -p loom --bin loom). Plaintext h2c, no TLS.
set -euo pipefail
cd "$(dirname "$0")/../../.."

RELAY=http://127.0.0.1:7402
KEEPB=http://127.0.0.1:7381
LOOM=http://127.0.0.1:7476
DATA=$(mktemp -d)/loom

RELAY_BIND=127.0.0.1:7402 RELAY_DATA_DIR="$DATA/relay" ./target/debug/relay-server >/tmp/loom-e2e-relay.log 2>&1 & R=$!
./target/debug/keep --host 127.0.0.1 --port 7381 >/tmp/loom-e2e-keep.log 2>&1 & K=$!
LOOM_ADDR=127.0.0.1:7476 LOOM_RELAY=$RELAY LOOM_DATA_DIR="$DATA/loom" ./target/debug/loom controller >/tmp/loom-e2e-ctl.log 2>&1 & C=$!
LOOM_RELAY=$RELAY LOOM_KEEP=$KEEPB LOOM_RUNNER=resident ./target/debug/loom worker >/tmp/loom-e2e-wrk.log 2>&1 & W=$!
trap 'kill $R $K $C $W 2>/dev/null' EXIT

for _ in $(seq 1 30); do curl -sf "$LOOM/healthz" >/dev/null 2>&1 && curl -sf "$KEEPB/healthz" >/dev/null 2>&1 && break; sleep 0.5; done

await() { # await <run_id>
  for _ in $(seq 1 40); do
    s=$(curl -s "$LOOM/runs/$1" | python3 -c 'import json,sys;print(json.load(sys.stdin)["status"])' 2>/dev/null || true)
    [ "$s" = succeeded ] && { echo "  run $1: succeeded"; return 0; }
    [ "$s" = failed ] && { echo "  run $1: FAILED"; return 1; }
    sleep 1
  done
  echo "  run $1: TIMEOUT"; return 1
}

echo "1) chain a->b"
curl -s -X POST "$LOOM/runs" -H 'content-type: application/json' \
  -d '{"run_id":"chain","nodes":[{"id":"a","task_name":"echo"},{"id":"b","task_name":"echo","deps":["a"]}]}' >/dev/null
await chain

echo "2) chord root->{x,y,z}->reduce"
curl -s -X POST "$LOOM/runs" -H 'content-type: application/json' \
  -d '{"run_id":"chord","nodes":[{"id":"root","task_name":"echo"},{"id":"x","task_name":"echo","deps":["root"]},{"id":"y","task_name":"echo","deps":["root"]},{"id":"z","task_name":"echo","deps":["root"]},{"id":"reduce","task_name":"echo","deps":["x","y","z"]}]}' >/dev/null
await chord

echo "3) data claim-check round-trip"
curl -s -X PUT "$KEEPB/v1/inputs/payload" -H 'content-type: application/octet-stream' --data-binary 'REAL-PAYLOAD' >/dev/null
curl -s -X POST "$LOOM/runs" -H 'content-type: application/json' \
  -d '{"run_id":"data","nodes":[{"id":"t","task_name":"echo","input_refs":["payload"]}]}' >/dev/null
await data
got=$(curl -s "$KEEPB/v1/results/data:t:result")
[ "$got" = "REAL-PAYLOAD" ] && echo "  data path verified: client read back '$got'" || { echo "  data MISMATCH: '$got'"; exit 1; }

echo "ALL E2E PASSED"
