#!/usr/bin/env bash
# loom multi-voter raft HA e2e (#110): boot a 3-node loom raft cluster (h2c raft
# transport), wait for a leader, submit a run to one node, and verify the run
# state replicated to ALL three nodes' state machines (consensus over the live
# h2c driver). Proves the production HA tier end-to-end across real processes.
#
# Requires: target/debug/loom.
set -euo pipefail
cd "$(dirname "$0")/../../.."

DATA=$(mktemp -d)
P0=7491; P1=7492; P2=7493
PEERS="0=http://127.0.0.1:$P0,1=http://127.0.0.1:$P1,2=http://127.0.0.1:$P2"

start_node() {
  local id=$1 port=$2
  LOOM_ADDR=127.0.0.1:$port LOOM_NODE_ID=$id LOOM_CLUSTER_PEERS="$PEERS" \
    LOOM_RAFT_DIR="$DATA/n$id" ./target/debug/loom controller \
    >/tmp/loom-cluster-n$id.log 2>&1 &
  echo $!
}
A=$(start_node 0 $P0); B=$(start_node 1 $P1); C=$(start_node 2 $P2)
trap 'kill $A $B $C 2>/dev/null' EXIT

echo "=== waiting for a leader to be elected ==="
leader=""
for i in $(seq 1 40); do
  for p in $P0 $P1 $P2; do
    if curl -s "http://127.0.0.1:$p/raftz" 2>/dev/null | grep -q '"is_leader":true'; then
      leader=$p; break
    fi
  done
  [ -n "$leader" ] && break
  sleep 0.5
done
if [ -z "$leader" ]; then echo "FAIL: no leader elected"; for p in $P0 $P1 $P2; do echo "node $p:"; curl -s http://127.0.0.1:$p/raftz; echo; done; exit 1; fi
echo "leader is on port $leader"
for p in $P0 $P1 $P2; do echo -n "  raftz $p: "; curl -s http://127.0.0.1:$p/raftz; echo; done

echo "=== submit a run to node 0 (port $P0) — store forwards to the leader, replicates ==="
curl -s -X POST "http://127.0.0.1:$P0/runs" -H 'content-type: application/json' \
  -d '{"run_id":"clusterrun","nodes":[{"id":"a","task_name":"echo"}]}' -w ' [%{http_code}]'; echo

echo "=== verify the run replicated to ALL three nodes' state machines ==="
ok=1
for p in $P0 $P1 $P2; do
  got=""
  for i in $(seq 1 20); do
    got=$(curl -s "http://127.0.0.1:$p/runs/clusterrun" 2>/dev/null | python3 -c 'import json,sys
try:
    d=json.load(sys.stdin); print(d.get("run_id",""))
except Exception: print("")' 2>/dev/null)
    [ "$got" = "clusterrun" ] && break
    sleep 0.3
  done
  if [ "$got" = "clusterrun" ]; then echo "  node $p: HAS run ✓"; else echo "  node $p: MISSING run ✗"; ok=0; fi
done
echo "=== result ==="
[ "$ok" = "1" ] && echo "PASS: run replicated to all 3 nodes via live h2c raft" || { echo "FAIL"; exit 1; }
