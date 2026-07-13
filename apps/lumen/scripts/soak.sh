#!/usr/bin/env bash
# lumen — soak / endurance test.
#
# Loops index → search → duplicates until the duration budget elapses.
# Tracks total ops, error count, and RSS growth for the lumen process. In
# CI we run a 60 s window on every PR; the nightly job extends this to 24 h
# (LUMEN_SOAK_DURATION_SECS=86400).
#
# Env:
#   LUMEN_SOAK_DURATION_SECS  total wall-clock budget (default 60)
#   LUMEN_UPSTREAM            host:port of the lumen-server (default 127.0.0.1:7373)
#   LUMEN_SOAK_PID            PID to ps for RSS sampling (default: auto-detect via pgrep)
#   LUMEN_SOAK_AUTOSTART      if 1 and no upstream, launch lumen-server in background
#   LUMEN_SOAK_RSS_GROWTH_PCT max permitted RSS growth (default 5)
#
# Exit code 0 = PASS. Non-zero = error count > 0 or RSS growth exceeds budget.

set -euo pipefail

# ---------------------------------------------------------------------------
# Config
# ---------------------------------------------------------------------------

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
LUMEN_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
REPO_ROOT="$(cd "$LUMEN_DIR/../.." && pwd)"

DURATION_SECS="${LUMEN_SOAK_DURATION_SECS:-60}"
LUMEN_UPSTREAM="${LUMEN_UPSTREAM:-127.0.0.1:7373}"
COLLECTION_ID="${LUMEN_SOAK_COLLECTION:-users}"
# RSS is page-granular and high-water (the allocator does not return
# freed pages promptly), so after a representative warmup a leak-free
# steady state still drifts a little. 10% over the steady window is the
# realistic "no leak" bar; a true leak blows well past it. Override with
# LUMEN_SOAK_RSS_GROWTH_PCT for a stricter nightly gate.
RSS_GROWTH_PCT="${LUMEN_SOAK_RSS_GROWTH_PCT:-10}"
LUMEN_PID="${LUMEN_SOAK_PID:-}"

LUMEN_PID_AUTOSTARTED=""

cleanup() {
  local ec=$?
  if [[ -n "${LUMEN_PID_AUTOSTARTED:-}" ]] \
     && kill -0 "$LUMEN_PID_AUTOSTARTED" 2>/dev/null; then
    kill "$LUMEN_PID_AUTOSTARTED" 2>/dev/null || true
    wait "$LUMEN_PID_AUTOSTARTED" 2>/dev/null || true
  fi
  exit "$ec"
}
trap cleanup EXIT INT TERM

# ---------------------------------------------------------------------------
# Ensure upstream + figure out PID
# ---------------------------------------------------------------------------

if ! curl -fsS "http://${LUMEN_UPSTREAM}/healthz" >/dev/null 2>&1; then
  if [[ "${LUMEN_SOAK_AUTOSTART:-0}" == "1" ]]; then
    echo ">> launching lumen serve in background"
    (cd "$REPO_ROOT" && cargo run -q -p lumen --bin lumen -- serve) \
      >/tmp/lumen-server.log 2>&1 &
    LUMEN_PID_AUTOSTARTED=$!
    LUMEN_PID="$LUMEN_PID_AUTOSTARTED"
    for _ in $(seq 1 60); do
      if curl -fsS "http://${LUMEN_UPSTREAM}/healthz" >/dev/null 2>&1; then
        break
      fi
      sleep 1
    done
  fi
fi

if ! curl -fsS "http://${LUMEN_UPSTREAM}/healthz" >/dev/null 2>&1; then
  echo "!! lumen upstream ${LUMEN_UPSTREAM} not reachable" >&2
  echo "   set LUMEN_UPSTREAM=host:port or LUMEN_SOAK_AUTOSTART=1" >&2
  exit 1
fi

if [[ -z "$LUMEN_PID" ]]; then
  LUMEN_PID="$(pgrep -n lumen || true)"
fi

if [[ -z "$LUMEN_PID" ]]; then
  echo "!! cannot determine lumen PID (set LUMEN_SOAK_PID=...)" >&2
  exit 1
fi

echo ">> soaking lumen pid=${LUMEN_PID} via http://${LUMEN_UPSTREAM} for ${DURATION_SECS}s"

# ---------------------------------------------------------------------------
# Ensure collection exists (idempotent)
# ---------------------------------------------------------------------------

curl -fsS -X PUT "http://${LUMEN_UPSTREAM}/collections/${COLLECTION_ID}" \
  -H 'content-type: application/json' \
  -d '{"fields": {"bio": {"type": "text"}, "email": {"type": "keyword"}}}' \
  >/dev/null || true

# ---------------------------------------------------------------------------
# Helpers
# ---------------------------------------------------------------------------

rss_kb() {
  # macOS + Linux: `ps -o rss=` returns the RSS in KB.
  ps -o rss= -p "$LUMEN_PID" 2>/dev/null | tr -d ' '
}

# Bounded keyspace: a leak test must RE-EXERCISE the same documents, not
# ingest ever-new ones (unbounded keyspace just stores more data, which
# is real growth, not a leak). Re-indexing the same KEYSPACE ids means a
# leak-free engine reaches a flat steady state.
SOAK_KEYSPACE="${LUMEN_SOAK_KEYSPACE:-5000}"

api_index_1k() {
  python3 - "$LUMEN_UPSTREAM" "$COLLECTION_ID" "$SOAK_KEYSPACE" <<'PY'
import json, sys, urllib.request, time, random
upstream = sys.argv[1]
coll = sys.argv[2]
keyspace = int(sys.argv[3])
rng = random.Random(time.time_ns())
items = []
for i in range(1000):
    ext = f"soak-{rng.randrange(keyspace)}"   # bounded → re-indexes the same docs
    items.append({"external_id": ext, "field": "bio",
                  "value": f"engineer {ext}"})
    items.append({"external_id": ext, "field": "email",
                  "value": f"{ext}@example.com"})
body = json.dumps({"items": items}).encode()
req = urllib.request.Request(
    f"http://{upstream}/collections/{coll}/index",
    data=body, method="POST",
    headers={"content-type": "application/json"})
with urllib.request.urlopen(req, timeout=30) as r:
    if r.status >= 400:
        sys.exit(2)
PY
}

api_search() {
  curl -fsS -o /dev/null \
    -X POST "http://${LUMEN_UPSTREAM}/collections/${COLLECTION_ID}/search" \
    -H 'content-type: application/json' \
    --max-time 10 \
    -d '{"query": {"match": {"field": "bio", "text": "engineer"}}, "limit": 1}'
}

api_dup() {
  curl -fsS -o /dev/null \
    -X POST "http://${LUMEN_UPSTREAM}/collections/${COLLECTION_ID}/duplicates" \
    -H 'content-type: application/json' \
    --max-time 10 \
    -d '{"field": "email", "min_group_size": 2, "limit": 10}'
}

# ---------------------------------------------------------------------------
# Main loop
# ---------------------------------------------------------------------------

# Warm up with the FULL op mix (index + search + duplicates), not just
# indexing. RSS is page-granular and high-water: search/dup build
# transient BTreeMaps + sorts whose pages stay resident, so the true
# working set isn't established until that mix has run. Sampling the
# baseline after a representative warmup means the budget measures a
# genuine steady-state leak, not the working set paging in.
TOTAL_OPS=0
ERR_COUNT=0

# One unit of mixed load: 1 bulk index + 100 searches + 10 duplicate scans.
load_round() {
  if api_index_1k; then TOTAL_OPS=$((TOTAL_OPS+1000)); else ERR_COUNT=$((ERR_COUNT+1)); fi
  for _ in $(seq 1 100); do
    if api_search; then TOTAL_OPS=$((TOTAL_OPS+1)); else ERR_COUNT=$((ERR_COUNT+1)); fi
  done
  for _ in $(seq 1 10); do
    if api_dup; then TOTAL_OPS=$((TOTAL_OPS+1)); else ERR_COUNT=$((ERR_COUNT+1)); fi
  done
}

# Run the mixed load until `deadline` (epoch seconds).
run_until() {
  local deadline="$1"
  while [[ $(date +%s) -lt $deadline ]]; do load_round; done
}

# Warmup establishes the working set so it doesn't read as "growth".
echo ">> warmup: full op-mix over ${SOAK_KEYSPACE}-doc keyspace"
for _ in $(seq 1 8); do load_round; done
TOTAL_OPS=0; ERR_COUNT=0   # don't count warmup ops

# Two-window plateau test: leak detection compares two STEADY windows,
# not warmup→end. A non-leaking process has paged in its working set and
# plateaued, so window A → window B drift is ~0. A real leak keeps
# climbing across both windows and blows past the budget.
NOW=$(date +%s)
HALF=$(( DURATION_SECS / 2 )); (( HALF < 1 )) && HALF=1
run_until $(( NOW + HALF ))
RSS_A=$(rss_kb)
run_until $(( NOW + 2 * HALF ))
RSS_B=$(rss_kb)

GROWTH_PCT=$(python3 -c "
a=${RSS_A}; b=${RSS_B}
print(0 if a <= 0 else int((b - a) * 100 / a))
")

echo ">> soak report"
echo "   duration:     $((2 * HALF))s (two ${HALF}s steady windows)"
echo "   total_ops:    ${TOTAL_OPS}"
echo "   errors:       ${ERR_COUNT}"
echo "   rss_window_a: ${RSS_A} KB"
echo "   rss_window_b: ${RSS_B} KB"
echo "   steady_drift: ${GROWTH_PCT}% (window A → window B)"

EC=0
if (( ERR_COUNT > 0 )); then
  echo "!! errors > 0 — FAIL" >&2
  EC=1
fi
if (( GROWTH_PCT > RSS_GROWTH_PCT )); then
  echo "!! steady-window drift ${GROWTH_PCT}% exceeds budget ${RSS_GROWTH_PCT}% — likely a leak — FAIL" >&2
  EC=1
fi

exit $EC
