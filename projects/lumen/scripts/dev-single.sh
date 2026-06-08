#!/usr/bin/env bash
# Single-node local dev. Embedded in-process write log — NO NATS, NO
# external dependency. The simplest way to poke lumen.
#
#   ./scripts/dev-single.sh                    # :7373, pretty logs
#   LUMEN_PORT=17373 ./scripts/dev-single.sh   # different port
#
# Storage is in-memory (lost on exit). Set LUMEN_DATA_DIR=/tmp/lumen-dev
# to keep RDB snapshots for faster restart. Ctrl-C to stop.

set -euo pipefail

export LUMEN_HOST="${LUMEN_HOST:-127.0.0.1}"
export LUMEN_PORT="${LUMEN_PORT:-7373}"
export LUMEN_WAL="${LUMEN_WAL:-embedded}"
export LUMEN_LOG_FORMAT="${LUMEN_LOG_FORMAT:-pretty}"
export LUMEN_AUTH="${LUMEN_AUTH:-off}"
export RUST_LOG="${RUST_LOG:-info,lumen=debug}"

echo "lumen serve (embedded log) on ${LUMEN_HOST}:${LUMEN_PORT}"
echo "  curl http://localhost:${LUMEN_PORT}/healthz"
echo "  open http://localhost:${LUMEN_PORT}/docs   (Swagger UI)"
exec cargo run -q -p lumen --bin lumen -- serve
