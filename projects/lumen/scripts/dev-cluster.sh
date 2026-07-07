#!/usr/bin/env bash
# 3-node local raft cluster.
#
#   pod 0: client :7373    pod 1: client :7374    pod 2: client :7375
#
# Demonstrates the real data plane: write to any node, read it back from
# any other — they converge through Lumen-owned raft replication.
# Ctrl-C stops all.

set -euo pipefail

ROOT="$(git rev-parse --show-toplevel)"
cd "$ROOT"

CLIENT_PORTS=(7373 7374 7375)
PEERS="127.0.0.1:7373,127.0.0.1:7374,127.0.0.1:7375"
LOG_DIR="${LUMEN_DEV_LOG_DIR:-/tmp/lumen-dev-cluster}"
mkdir -p "$LOG_DIR"

echo "→ building lumen with raft-wal"
cargo build -q -p lumen --bin lumen --features raft-wal

PIDS=()
cleanup() {
  echo; echo "→ stopping ${#PIDS[@]} processes"
  kill "${PIDS[@]}" 2>/dev/null || true
  wait 2>/dev/null || true
}
trap cleanup EXIT INT TERM

for i in 0 1 2; do
  LUMEN_HOST=127.0.0.1 \
  LUMEN_PORT="${CLIENT_PORTS[$i]}" \
  LUMEN_WAL=raft \
  LUMEN_RAFT_DATA_DIR="$LOG_DIR/node-$i-raft" \
  POD_NAME="lumen-$i" \
  SHARD_COUNT=1 \
  REPLICAS_PER_SHARD=3 \
  VOTER_COUNT=3 \
  LUMEN_HEADLESS_SERVICE=lumen-local \
  LUMEN_PEERS="$PEERS" \
  LUMEN_AUTH=off \
  LUMEN_LOG_FORMAT=pretty \
  RUST_LOG="${RUST_LOG:-info,lumen=debug}" \
    "$ROOT/target/debug/lumen" serve > "$LOG_DIR/node-$i.log" 2>&1 &
  PIDS+=($!)
  echo "  node-$i  client=:${CLIENT_PORTS[$i]}  log=$LOG_DIR/node-$i.log"
done

echo
echo "→ cluster up (3 raft nodes). Try:"
echo "    curl -sS -X PUT  http://localhost:7373/collections/users -d '{\"fields\":{\"email\":{\"type\":\"keyword\"}}}'"
echo "    curl -sS -X POST http://localhost:7373/collections/users/index -d '{\"items\":[{\"external_id\":\"u1\",\"field\":\"email\",\"value\":\"a@x.com\"}]}'"
echo "    # then read it from a DIFFERENT node — it converged via raft:"
echo "    curl -sS -X POST http://localhost:7375/collections/users/search -d '{\"query\":{\"term\":{\"field\":\"email\",\"value\":\"a@x.com\"}}}'"
echo
echo "Ctrl-C to stop. Logs in $LOG_DIR/."
wait
