#!/usr/bin/env bash
# 3-node local raft cluster.
#
#   pod 0: client :7373  raft :7473
#   pod 1: client :7374  raft :7474
#   pod 2: client :7375  raft :7475
#
# Demonstrates the real data plane: write to any node, read it back from
# any other — they converge through Lumen-owned raft replication.
# Ctrl-C stops all.
#
# Raft peer traffic is mutually authenticated and has no plaintext path
# (#2890), so this script mints a throwaway CA and one shared leaf certificate
# before starting anything. In a cluster that material is `spec.peerTlsSecret`;
# here it is three files in $LOG_DIR/tls, regenerated on every run and worth
# nothing outside this machine.

set -euo pipefail

ROOT="$(git rev-parse --show-toplevel)"
cd "$ROOT"

CLIENT_PORTS=(7373 7374 7375)
# Peers are addressed on the dedicated raft listener, never the client port.
RAFT_PORTS=(7473 7474 7475)
PEERS="127.0.0.1:7473,127.0.0.1:7474,127.0.0.1:7475"
LOG_DIR="${LUMEN_DEV_LOG_DIR:-/tmp/lumen-dev-cluster}"
TLS_DIR="$LOG_DIR/tls"
mkdir -p "$LOG_DIR" "$TLS_DIR"

echo "→ minting throwaway peer mTLS material in $TLS_DIR"
# The peers dial `https://127.0.0.1:<raft-port>`, so the leaf needs an IP SAN;
# rustls verifies the dialed identity, not just the chain.
openssl req -x509 -newkey rsa:2048 -nodes -days 1 \
  -keyout "$TLS_DIR/ca.key" -out "$TLS_DIR/ca.crt" \
  -subj "/CN=lumen dev-cluster CA" \
  -addext "basicConstraints=critical,CA:TRUE" 2>/dev/null
openssl req -newkey rsa:2048 -nodes \
  -keyout "$TLS_DIR/tls.key" -out "$TLS_DIR/tls.csr" \
  -subj "/CN=localhost" 2>/dev/null
openssl x509 -req -in "$TLS_DIR/tls.csr" -days 1 \
  -CA "$TLS_DIR/ca.crt" -CAkey "$TLS_DIR/ca.key" -CAcreateserial \
  -out "$TLS_DIR/tls.crt" \
  -extfile <(printf 'subjectAltName=DNS:localhost,IP:127.0.0.1\nextendedKeyUsage=serverAuth,clientAuth\n') 2>/dev/null
chmod 600 "$TLS_DIR/tls.key" "$TLS_DIR/ca.key"

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
  LUMEN_RAFT_PORT="${RAFT_PORTS[$i]}" \
  LUMEN_WAL=raft \
  LUMEN_RAFT_DATA_DIR="$LOG_DIR/node-$i-raft" \
  POD_NAME="lumen-$i" \
  SHARD_COUNT=1 \
  REPLICAS_PER_SHARD=3 \
  VOTER_COUNT=3 \
  LUMEN_HEADLESS_SERVICE=lumen-local \
  LUMEN_PEERS="$PEERS" \
  LUMEN_PEER_MTLS=on \
  LUMEN_PEER_TLS_CERT="$TLS_DIR/tls.crt" \
  LUMEN_PEER_TLS_KEY="$TLS_DIR/tls.key" \
  LUMEN_PEER_TLS_CA="$TLS_DIR/ca.crt" \
  LUMEN_AUTH=off \
  LUMEN_LOG_FORMAT=pretty \
  RUST_LOG="${RUST_LOG:-info,lumen=debug}" \
    "$ROOT/target/debug/lumen" serve > "$LOG_DIR/node-$i.log" 2>&1 &
  PIDS+=($!)
  echo "  node-$i  client=:${CLIENT_PORTS[$i]}  raft=:${RAFT_PORTS[$i]}  log=$LOG_DIR/node-$i.log"
done

echo
echo "→ cluster up (3 raft nodes, mutually authenticated peer transport). Try:"
echo "    curl -sS -X PUT  http://localhost:7373/collections/users -d '{\"fields\":{\"email\":{\"type\":\"keyword\"}}}'"
echo "    curl -sS -X POST http://localhost:7373/collections/users/index -d '{\"items\":[{\"external_id\":\"u1\",\"field\":\"email\",\"value\":\"a@x.com\"}]}'"
echo "    # then read it from a DIFFERENT node — it converged via raft:"
echo "    curl -sS -X POST http://localhost:7375/collections/users/search -d '{\"query\":{\"term\":{\"field\":\"email\",\"value\":\"a@x.com\"}}}'"
echo
echo "Ctrl-C to stop. Logs in $LOG_DIR/."
wait
