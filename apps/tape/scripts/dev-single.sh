#!/usr/bin/env bash
# HANDWRITE-BEGIN gap="missing-generator:logic:a3fc8dfd" tracker="pending-tracker" reason="New script (mirrors projects/lumen/scripts/dev-single.sh): single-node local dev, embedded file-backed journal via TAPE_STORE, TAPE_BIND default 127.0.0.1:7137, TAPE_AUTH=off, runs cargo run -p tape --bin tape -- serve."
# Single-node local dev. Embedded file-backed journal — NO raft, NO peers.
# The simplest way to poke tape.
#
#   ./apps/tape/scripts/dev-single.sh                             # :7137, journal in .tape/
#   TAPE_BIND=127.0.0.1:17137 ./apps/tape/scripts/dev-single.sh   # different port
#
# Journal persists to TAPE_STORE (default .tape/dev-single.json) so a restart
# resumes with prior events. Delete that file to start clean. Ctrl-C to stop.

set -euo pipefail

ROOT="$(git rev-parse --show-toplevel)"
cd "$ROOT"

export TAPE_BIND="${TAPE_BIND:-127.0.0.1:7137}"
export TAPE_STORE="${TAPE_STORE:-.tape/dev-single.json}"
export TAPE_AUTH="${TAPE_AUTH:-off}"
export TAPE_GRACE_SECS="${TAPE_GRACE_SECS:-10}"
export RUST_LOG="${RUST_LOG:-info,tape=debug}"

mkdir -p "$(dirname "$TAPE_STORE")"

echo "tape serve (embedded journal) on ${TAPE_BIND}"
echo "  store: ${TAPE_STORE}"
echo "  curl http://${TAPE_BIND}/healthz"
echo "  open http://${TAPE_BIND}/docs   (Swagger UI)"
exec cargo run -q -p tape --bin tape -- serve
# HANDWRITE-END
