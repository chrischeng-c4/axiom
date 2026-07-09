#!/usr/bin/env bash
# HANDWRITE-BEGIN gap="missing-generator:logic:8dbe3ccf" tracker="pending-tracker" reason="New script (mirrors projects/lumen/scripts/dev-cluster.sh): 3-node local raft cluster, sets REPLICAS_PER_SHARD=3/SHARD_COUNT=1/VOTER_COUNT=3/POD_NAME plus TAPE_DATA_DIR/TAPE_PEER_SERVICE/TAPE_PEERS so raft_host::ClusterTopology::from_env resolves peers and TapeRaft::from_topology replicates append/checkpoint-put across 3 tape serve processes on distinct ports."
# 3-node local raft cluster (#1327).
#
#   node 0: :7137    node 1: :7138    node 2: :7139
#
# Demonstrates the real data plane: append/checkpoint-put on any node
# replicates to the others through raft-host's TapeRaft state machine; replay
# and checkpoint-get are served locally by whichever node you hit. Ctrl-C
# stops all three.
#
# This is auto-mode HA: setting REPLICAS_PER_SHARD=3 (plus the standard
# SHARD_COUNT/VOTER_COUNT/POD_NAME downward-API quartet) is the only signal —
# there is no tape-specific "--raft" flag. Peers resolve through
# raft_host::ClusterTopology::from_env("tape", <peer-service>, <peer-port>,
# "TAPE_PEERS"), so a flat comma-separated TAPE_PEERS list stands in here for
# the k8s headless-service DNS lookup the operator wires in-cluster.

set -euo pipefail

ROOT="$(git rev-parse --show-toplevel)"
cd "$ROOT"

CLIENT_PORTS=(7137 7138 7139)
PEERS="127.0.0.1:7137,127.0.0.1:7138,127.0.0.1:7139"
LOG_DIR="${TAPE_DEV_LOG_DIR:-/tmp/tape-dev-cluster}"
mkdir -p "$LOG_DIR"

echo "→ building tape"
cargo build -q -p tape --bin tape

PIDS=()
cleanup() {
  echo; echo "→ stopping ${#PIDS[@]} processes"
  kill "${PIDS[@]}" 2>/dev/null || true
  wait 2>/dev/null || true
}
trap cleanup EXIT INT TERM

for i in 0 1 2; do
  TAPE_BIND="127.0.0.1:${CLIENT_PORTS[$i]}" \
  TAPE_DATA_DIR="$LOG_DIR/node-$i-raft" \
  TAPE_PEER_SERVICE="tape-local" \
  TAPE_PEERS="$PEERS" \
  POD_NAME="tape-$i" \
  SHARD_COUNT=1 \
  REPLICAS_PER_SHARD=3 \
  VOTER_COUNT=3 \
  TAPE_AUTH=off \
  RUST_LOG="${RUST_LOG:-info,tape=debug}" \
    "$ROOT/target/debug/tape" serve > "$LOG_DIR/node-$i.log" 2>&1 &
  PIDS+=($!)
  echo "  node-$i  client=:${CLIENT_PORTS[$i]}  log=$LOG_DIR/node-$i.log"
done

echo
echo "→ cluster up (3 raft nodes). Try:"
echo "    curl -sS -X POST http://localhost:7137/topics/orders.created/append \\"
echo "      -H 'content-type: application/json' -d '{\"payload\":{\"hello\":\"world\"}}'"
echo "    # then replay it from a DIFFERENT node — it converged via raft:"
echo "    curl -sS http://localhost:7139/topics/orders.created/replay"
echo
echo "Ctrl-C to stop. Logs in $LOG_DIR/."
wait
# HANDWRITE-END
