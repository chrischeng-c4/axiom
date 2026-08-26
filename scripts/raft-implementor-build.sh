#!/usr/bin/env bash
# Slow migration gate: compiles every registered RaftStateMachine implementor.
# Each application uses its bounded package and feature gate.
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

echo "cargo build -p defer"
cargo build -p defer

echo "cargo build -p keep --features raft"
cargo build -p keep --features raft

echo "cargo build -p loom"
cargo build -p loom

echo "cargo build -p lumen --features raft-wal"
cargo build -p lumen --features raft-wal

echo "cargo build -p relay"
cargo build -p relay

echo "cargo build -p tape"
cargo build -p tape

echo "cargo build -p sift"
cargo build -p sift

echo "cargo test -p raft-runtime --no-run"
cargo test -p raft-runtime --no-run

echo "cargo test -p raft-runtime --test implementor_build_coverage"
cargo test -p raft-runtime --test implementor_build_coverage
