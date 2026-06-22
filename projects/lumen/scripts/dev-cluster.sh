#!/usr/bin/env bash
# 3-node local cluster sharing one Relay write log.
#
#   pod 0: client :7373    pod 1: client :7374    pod 2: client :7375
#   all three publish to / tail from a local relay-server (:7000)
#
# Demonstrates the real data plane: write to any node, read it back from
# any other — they converge by tailing the shared Relay log (fan-out).
# Ctrl-C stops all.

set -euo pipefail

ROOT="$(git rev-parse --show-toplevel)"
cd "$ROOT"

RELAY_PORT="${RELAY_PORT:-7000}"
RELAY_URL="http://127.0.0.1:${RELAY_PORT}"
CLIENT_PORTS=(7373 7374 7375)
LOG_DIR="${LUMEN_DEV_LOG_DIR:-/tmp/lumen-dev-cluster}"
mkdir -p "$LOG_DIR" "$LOG_DIR/relay-data"

echo "→ building lumen + relay"
cargo build -q -p lumen --bin lumen --features relay-wal
cargo build -q -p relay --bin relay-server

PIDS=()
cleanup() {
  echo; echo "→ stopping ${#PIDS[@]} processes"
  kill "${PIDS[@]}" 2>/dev/null || true
  wait 2>/dev/null || true
}
trap cleanup EXIT INT TERM

echo "→ starting Relay on :${RELAY_PORT}"
RELAY_BIND="127.0.0.1:${RELAY_PORT}" \
RELAY_DATA_DIR="$LOG_DIR/relay-data" \
  "$ROOT/target/debug/relay-server" > "$LOG_DIR/relay.log" 2>&1 &
PIDS+=($!)
sleep 1

for i in 0 1 2; do
  LUMEN_HOST=127.0.0.1 \
  LUMEN_PORT="${CLIENT_PORTS[$i]}" \
  LUMEN_WAL=relay \
  LUMEN_RELAY_URL="$RELAY_URL" \
  LUMEN_RELAY_SUBJECT=lumen-wal \
  LUMEN_RELAY_SUBSCRIBER_ID="lumen-dev-$i" \
  LUMEN_AUTH=off \
  LUMEN_LOG_FORMAT=pretty \
  RUST_LOG="${RUST_LOG:-info,lumen=debug}" \
    "$ROOT/target/debug/lumen" serve > "$LOG_DIR/node-$i.log" 2>&1 &
  PIDS+=($!)
  echo "  node-$i  client=:${CLIENT_PORTS[$i]}  log=$LOG_DIR/node-$i.log"
done

echo
echo "→ cluster up (3 nodes + Relay). Try:"
echo "    curl -sS -X PUT  http://localhost:7373/collections/users -d '{\"fields\":{\"email\":{\"type\":\"keyword\"}}}'"
echo "    curl -sS -X POST http://localhost:7373/collections/users/index -d '{\"items\":[{\"external_id\":\"u1\",\"field\":\"email\",\"value\":\"a@x.com\"}]}'"
echo "    # then read it from a DIFFERENT node — it converged via Relay:"
echo "    curl -sS -X POST http://localhost:7375/collections/users/search -d '{\"query\":{\"term\":{\"field\":\"email\",\"value\":\"a@x.com\"}}}'"
echo
echo "Ctrl-C to stop. Logs in $LOG_DIR/."
wait
