#!/usr/bin/env bash
# Slow migration gate: compiles every workspace RaftStateMachine implementor.
# A cold run can compile much of the workspace.
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

echo "cargo build --workspace"
cargo build --workspace

echo "cargo build -p keep --features raft"
cargo build -p keep --features raft

echo "cargo build -p lumen --features raft-wal"
cargo build -p lumen --features raft-wal

echo "cargo test -p raft-runtime --no-run"
cargo test -p raft-runtime --no-run

echo "cargo test -p raft-runtime --test implementor_build_coverage"
cargo test -p raft-runtime --test implementor_build_coverage
