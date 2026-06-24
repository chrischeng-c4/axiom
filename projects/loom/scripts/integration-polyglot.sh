#!/usr/bin/env bash
# Polyglot integration test (#106): boot real relay + keep + loom, then run a
# Python producer + worker pool (tests/integration/polyglot_e2e.py) that drive
# loom over plain HTTP with zero deps — proving the no-SDK polyglot story.
# Uses LOOM_COMPLETION_SHARDS=1 so the Python worker reports to one completions
# subject (no shard hashing). bash 3.2-safe.
set -euo pipefail
cd "$(dirname "$0")/../../.."

echo "=== build release binaries ==="
cargo build --release -p loom -p relay -p keep 2>&1 | tail -1

RELAY=http://127.0.0.1:7421; KEEP=http://127.0.0.1:7395; LOOM=http://127.0.0.1:7496
rm -rf /tmp/loom-poly; mkdir -p /tmp/loom-poly
RELAY_BIND=127.0.0.1:7421 RELAY_DATA_DIR=/tmp/loom-poly/relay ./target/release/relay-server >/tmp/loom-poly/relay.log 2>&1 & R=$!
./target/release/keep --host 127.0.0.1 --port 7395 >/tmp/loom-poly/keep.log 2>&1 & K=$!
LOOM_ADDR=127.0.0.1:7496 LOOM_RELAY=$RELAY LOOM_COMPLETION_SHARDS=1 ./target/release/loom controller >/tmp/loom-poly/ctl.log 2>&1 & C=$!
trap 'kill $R $K $C 2>/dev/null' EXIT

for i in $(seq 1 40); do
  curl -sf "$LOOM/healthz" >/dev/null 2>&1 && curl -sf "$KEEP/healthz" >/dev/null 2>&1 && break
  sleep 0.5
done

echo "=== run the Python polyglot producer + worker pool ==="
LOOM=$LOOM RELAY=$RELAY KEEP=$KEEP python3 projects/loom/tests/integration/polyglot_e2e.py
