#!/usr/bin/env bash
# lumen_scale.sh — LUMEN-ONLY disk scale bench across a row-count ladder.
#
# Measures lumen's DISK engine (segment-backed via flush_to_segments) across a
# row ladder. NO Postgres, NO OpenSearch — this is lumen's own scale story.
# The standard local benchmark cap is 100k docs. For each N it reports per-cell
# latency + the qps ladder PLUS per-N on-disk index MiB, bytes/doc, peak RSS,
# and the RSS/on-disk ratio.
#
# Usage:
#   ./scripts/lumen_scale.sh                       # default ladder 1000,10000,100000
#   ./scripts/lumen_scale.sh 1000,10000,100000     # explicit (same as default)
#   LUMEN_SCALE_ALLOW_ABOVE_STANDARD=1 LUMEN_GATE_WINDOW_S=0.2 LUMEN_SCALE_CHUNK_ROWS=100000 ./scripts/lumen_scale.sh 1000000
#       # reopened-shard HTTP qps smoke: 2 sealed chunks behind the test router
#
# NOTE: the bench stream-generates docs (no full corpus Vec), but Engine indexing
# still holds the mutable index until `flush_to_segments` on the single-segment
# path. 100k is now the local readiness target; 1M+ rows are intentionally
# blocked unless an explicit release-soak/research run sets
# LUMEN_SCALE_ALLOW_ABOVE_STANDARD=1.
# This is a MEASUREMENT/REPORT (no WIN assertions); read stdout.

set -euo pipefail

# rustup toolchain (not Homebrew rustc).
export PATH="$HOME/.rustup/toolchains/stable-aarch64-apple-darwin/bin:$PATH"

# Optional first arg overrides the row ladder; default stays in the safe local tier.
ROWS="${1:-1000,10000,100000}"
export LUMEN_SCALE_ROWS="$ROWS"
export LUMEN_SCALE_MAX_ROWS="${LUMEN_SCALE_MAX_ROWS:-100000}"

_allow_above_standard="${LUMEN_SCALE_ALLOW_ABOVE_STANDARD:-${LUMEN_SCALE_ALLOW_ABOVE_1M:-0}}"
if [[ "$_allow_above_standard" != "1" && "$_allow_above_standard" != "true" && "$_allow_above_standard" != "yes" ]]; then
    IFS=',' read -r -a _lumen_scale_rows <<< "$ROWS"
    for _row in "${_lumen_scale_rows[@]}"; do
        _row="${_row//[[:space:]]/}"
        if [[ -n "$_row" && "$_row" =~ ^[0-9]+$ && "$_row" -gt "$LUMEN_SCALE_MAX_ROWS" ]]; then
            echo "error: LUMEN_SCALE_ROWS contains $_row, above the standard local cap $LUMEN_SCALE_MAX_ROWS" >&2
            echo "       100k docs is the local readiness target; larger rows require LUMEN_SCALE_ALLOW_ABOVE_STANDARD=1." >&2
            exit 2
        fi
    done
fi

# Disk path is the point of this bench (default on; set LUMEN_SCALE_DISK=0 to A/B
# against the in-RAM driver path).
export LUMEN_SCALE_DISK="${LUMEN_SCALE_DISK:-1}"

# Full reports include the fixed-window qps ladder by default. For large
# preflights, set LUMEN_SCALE_QPS=0 to measure index+seal+latency+storage first.
export LUMEN_SCALE_QPS="${LUMEN_SCALE_QPS:-1}"
export LUMEN_SCALE_CHUNK_ROWS="${LUMEN_SCALE_CHUNK_ROWS:-}"
export LUMEN_SCALE_STORAGE_ONLY="${LUMEN_SCALE_STORAGE_ONLY:-0}"
export LUMEN_SCALE_CHUNK_WORKERS="${LUMEN_SCALE_CHUNK_WORKERS:-1}"
export LUMEN_SCALE_REOPEN_SHARDS="${LUMEN_SCALE_REOPEN_SHARDS:-1}"

echo "== LUMEN-ONLY DISK SCALE BENCH =="
echo "   LUMEN_SCALE_ROWS=$LUMEN_SCALE_ROWS  LUMEN_SCALE_MAX_ROWS=$LUMEN_SCALE_MAX_ROWS  LUMEN_SCALE_DISK=$LUMEN_SCALE_DISK  LUMEN_SCALE_QPS=$LUMEN_SCALE_QPS  LUMEN_SCALE_CHUNK_ROWS=${LUMEN_SCALE_CHUNK_ROWS:-<off>}  LUMEN_SCALE_STORAGE_ONLY=$LUMEN_SCALE_STORAGE_ONLY  LUMEN_SCALE_CHUNK_WORKERS=$LUMEN_SCALE_CHUNK_WORKERS  LUMEN_SCALE_REOPEN_SHARDS=$LUMEN_SCALE_REOPEN_SHARDS"
echo "   (lumen-only — no pg / no OpenSearch)"

exec cargo test --release -p lumen --test perf_gate_vs_db -- \
    --ignored --nocapture lumen_scale_bench
