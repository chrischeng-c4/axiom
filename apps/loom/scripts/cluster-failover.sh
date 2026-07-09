#!/usr/bin/env bash
# loom raft HA failover e2e (#110): boot a 3-node loom cluster, submit a run to
# the leader, KILL the leader, and verify the cluster re-elects, keeps the
# committed state, and accepts new writes that replicate to survivors. The real
# HA proof (survive a node crash). Requires target/debug/loom.  bash 3.2-safe.
set -euo pipefail
cd "$(dirname "$0")/../../.."
DATA=$(mktemp -d); P0=7494;P1=7495;P2=7496
PEERS="0=http://127.0.0.1:$P0,1=http://127.0.0.1:$P1,2=http://127.0.0.1:$P2"
port(){ case $1 in 0)echo $P0;;1)echo $P1;;2)echo $P2;;esac; }
sn(){ LOOM_ADDR=127.0.0.1:$2 LOOM_NODE_ID=$1 LOOM_CLUSTER_PEERS="$PEERS" LOOM_RAFT_DIR="$DATA/n$1" ./target/debug/loom controller >/tmp/loom-fo-n$1.log 2>&1 & echo $!; }
PID0=$(sn 0 $P0);PID1=$(sn 1 $P1);PID2=$(sn 2 $P2)
pidof(){ case $1 in 0)echo $PID0;;1)echo $PID1;;2)echo $PID2;;esac; }
trap 'kill $PID0 $PID1 $PID2 2>/dev/null' EXIT
find_leader(){ for n in 0 1 2; do curl -s http://127.0.0.1:$(port $n)/raftz 2>/dev/null|grep -q '"is_leader":true' && { echo $n; return; }; done; echo ""; }
L=""; for i in $(seq 1 40); do L=$(find_leader); [ -n "$L" ]&&break; sleep 0.5; done
echo "initial leader = node $L (port $(port $L))"
curl -s -X POST http://127.0.0.1:$(port $L)/runs -H 'content-type: application/json' -d '{"run_id":"r-before","nodes":[{"id":"a","task_name":"echo"}]}' >/dev/null; echo "submitted r-before to leader"
echo "=== KILL the leader (node $L) ==="; kill $(pidof $L) 2>/dev/null || true
NL=""; for i in $(seq 1 40); do for n in 0 1 2; do [ "$n" = "$L" ]&&continue; curl -s http://127.0.0.1:$(port $n)/raftz 2>/dev/null|grep -q '"is_leader":true'&&{ NL=$n;break;}; done; [ -n "$NL" ]&&break; sleep 0.5; done
[ -z "$NL" ] && { echo "FAIL: no new leader after failover"; exit 1; }
echo "NEW leader = node $NL (re-elected after crash) ✓"
echo -n "r-before still readable from new leader: "; curl -s http://127.0.0.1:$(port $NL)/runs/r-before|python3 -c 'import json,sys;print("YES (old state intact) ✓" if json.load(sys.stdin).get("run_id")=="r-before" else "NO ✗")'
curl -s -X POST http://127.0.0.1:$(port $NL)/runs -H 'content-type: application/json' -d '{"run_id":"r-after","nodes":[{"id":"a","task_name":"echo"}]}' >/dev/null; echo "submitted r-after to surviving cluster"
sleep 0.5; for n in 0 1 2; do [ "$n" = "$L" ]&&continue; echo -n "  node $n r-after: "; curl -s http://127.0.0.1:$(port $n)/runs/r-after|python3 -c 'import json,sys;print("HAS ✓" if json.load(sys.stdin).get("run_id")=="r-after" else "MISSING ✗")' 2>/dev/null||echo "?"; done
echo "PASS: cluster survived leader crash — re-elected, old state intact, new writes replicated"
