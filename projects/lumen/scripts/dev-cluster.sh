#!/usr/bin/env bash
# 3-node local cluster sharing one NATS JetStream write log.
#
#   pod 0: client :7373    pod 1: client :7374    pod 2: client :7375
#   all three publish to / tail from a local nats-server (:4222)
#
# Demonstrates the real data plane: write to any node, read it back from
# any other — they converge by tailing the shared NATS log (fan-out).
# Requires nats-server (`brew install nats-server`). Ctrl-C stops all.

set -euo pipefail

NATS_PORT="${NATS_PORT:-4222}"
NATS_URL="nats://127.0.0.1:${NATS_PORT}"
CLIENT_PORTS=(7373 7374 7375)
LOG_DIR="${LUMEN_DEV_LOG_DIR:-/tmp/lumen-dev-cluster}"
mkdir -p "$LOG_DIR" "$LOG_DIR/nats-store"

command -v nats-server >/dev/null || { echo "need nats-server (brew install nats-server)"; exit 1; }

echo "→ building lumen"
cargo build -q -p lumen --bin lumen

PIDS=()
cleanup() {
  echo; echo "→ stopping ${#PIDS[@]} processes"
  kill "${PIDS[@]}" 2>/dev/null || true
  wait 2>/dev/null || true
}
trap cleanup EXIT INT TERM

echo "→ starting NATS JetStream on :${NATS_PORT}"
# max_payload has no CLI flag — it must come from a config file. Lift the 1MB
# default so bulk index batches (published as one message) fit, matching k8s.
printf 'max_payload: 8MB\n' > "$LOG_DIR/nats.conf"
nats-server -c "$LOG_DIR/nats.conf" -js -sd "$LOG_DIR/nats-store" -p "$NATS_PORT" > "$LOG_DIR/nats.log" 2>&1 &
PIDS+=($!)
sleep 1

for i in 0 1 2; do
  LUMEN_HOST=127.0.0.1 \
  LUMEN_PORT="${CLIENT_PORTS[$i]}" \
  LUMEN_WAL=nats \
  LUMEN_NATS_URL="$NATS_URL" \
  LUMEN_AUTH=off \
  LUMEN_LOG_FORMAT=pretty \
  RUST_LOG="${RUST_LOG:-info,lumen=debug}" \
    cargo run -q -p lumen --bin lumen -- serve > "$LOG_DIR/node-$i.log" 2>&1 &
  PIDS+=($!)
  echo "  node-$i  client=:${CLIENT_PORTS[$i]}  log=$LOG_DIR/node-$i.log"
done

echo
echo "→ cluster up (3 nodes + NATS). Try:"
echo "    curl -sS -X PUT  http://localhost:7373/collections/users -d '{\"fields\":{\"email\":{\"type\":\"keyword\"}}}'"
echo "    curl -sS -X POST http://localhost:7373/collections/users/index -d '{\"items\":[{\"external_id\":\"u1\",\"field\":\"email\",\"value\":\"a@x.com\"}]}'"
echo "    # then read it from a DIFFERENT node — it converged via NATS:"
echo "    curl -sS -X POST http://localhost:7375/collections/users/search -d '{\"query\":{\"term\":{\"field\":\"email\",\"value\":\"a@x.com\"}}}'"
echo
echo "Ctrl-C to stop. Logs in $LOG_DIR/."
wait
