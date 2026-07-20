#!/usr/bin/env bash
# HANDWRITE-BEGIN gap="missing-generator:e2e-test:bounded-replay-soak" tracker="#1589" reason="Tape's bounded HTTP replay/checkpoint soak has service-specific append-only workload setup, process lifecycle, and RSS plateau assertions."
# Tape bounded replay soak.
#
# A replay journal must not use an ever-growing append stream as a leak test:
# retained history is expected to grow memory and disk. This runner seeds a
# fixed topic once, then repeatedly exercises replay and checkpoint reads/writes
# across two steady windows. It fails on request errors or unbounded
# RSS/FD/thread/p99 drift.
#
# Usage:
#   TAPE_SOAK_AUTOSTART=1 bash apps/tape/scripts/soak.sh
#   TAPE_UPSTREAM=127.0.0.1:7137 TAPE_SOAK_PID=<pid> bash apps/tape/scripts/soak.sh
#
# Environment:
#   TAPE_SOAK_DURATION_SECS      two-window wall-clock budget (default: 60)
#   TAPE_UPSTREAM                host:port of tape serve (default: 127.0.0.1:7137)
#   TAPE_SOAK_PID                server PID for RSS sampling (default: auto-detect)
#   TAPE_SOAK_AUTOSTART          1 builds and launches an isolated local Tape
#   TAPE_SOAK_RSS_GROWTH_PCT     permitted steady-window RSS growth (default: 10)
#   TAPE_SOAK_FD_GROWTH           permitted descriptor growth (default: 8)
#   TAPE_SOAK_TASK_GROWTH         permitted thread/task growth (default: 4)
#   TAPE_SOAK_P99_MS              absolute p99 plateau budget (default: 250)
#   TAPE_SOAK_P99_GROWTH_PCT      permitted p99 drift above the budget (default: 100)
#   TAPE_SOAK_EVENTS             fixed seed event count (default: 64)
#   TAPE_SOAK_TOPIC              topic to seed and replay (default: soak-orders)
#   TAPE_SOAK_CONSUMER           checkpoint consumer name (default: soak-worker)

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
TAPE_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
REPO_ROOT="$(cd "$TAPE_DIR/../.." && pwd)"
source "$REPO_ROOT/libs/service-observability/scripts/soak-metrics.sh"

DURATION_SECS="${TAPE_SOAK_DURATION_SECS:-60}"
UPSTREAM="${TAPE_UPSTREAM:-127.0.0.1:7137}"
RSS_GROWTH_PCT="${TAPE_SOAK_RSS_GROWTH_PCT:-10}"
FD_GROWTH="${TAPE_SOAK_FD_GROWTH:-8}"
TASK_GROWTH="${TAPE_SOAK_TASK_GROWTH:-4}"
P99_MS="${TAPE_SOAK_P99_MS:-250}"
P99_GROWTH_PCT="${TAPE_SOAK_P99_GROWTH_PCT:-100}"
SEED_EVENTS="${TAPE_SOAK_EVENTS:-64}"
TOPIC="${TAPE_SOAK_TOPIC:-soak-orders}"
CONSUMER="${TAPE_SOAK_CONSUMER:-soak-worker}"
TAPE_PID="${TAPE_SOAK_PID:-}"
AUTOSTART_PID=""
AUTOSTART_DIR=""
SOAK_TMP_DIR=""
LATENCY_FILE="/dev/null"

require() {
  command -v "$1" >/dev/null 2>&1 || {
    echo "!! required command not found: $1" >&2
    exit 1
  }
}

cleanup() {
  local ec=$?
  trap - EXIT INT TERM
  if [[ -n "$AUTOSTART_PID" ]] && kill -0 "$AUTOSTART_PID" 2>/dev/null; then
    kill "$AUTOSTART_PID" 2>/dev/null || true
    wait "$AUTOSTART_PID" 2>/dev/null || true
  fi
  if [[ -n "$AUTOSTART_DIR" ]]; then
    rm -rf "$AUTOSTART_DIR"
  fi
  if [[ -n "$SOAK_TMP_DIR" ]]; then
    rm -rf "$SOAK_TMP_DIR"
  fi
  exit "$ec"
}
trap cleanup EXIT
trap 'exit 130' INT TERM

for cmd in curl jq ps; do
  require "$cmd"
done

healthy() {
  curl -fsS --max-time 5 "http://${UPSTREAM}/healthz" >/dev/null 2>&1
}

if ! healthy && [[ "${TAPE_SOAK_AUTOSTART:-0}" == "1" ]]; then
  AUTOSTART_DIR="$(mktemp -d "${TMPDIR:-/tmp}/tape-soak.XXXXXX")"
  echo ">> building and launching isolated Tape at http://${UPSTREAM}"
  (
    cd "$REPO_ROOT"
    cargo build -q -p tape --bin tape
  )
  (
    cd "$REPO_ROOT"
    exec "$REPO_ROOT/target/debug/tape" serve \
      --bind "$UPSTREAM" \
      --store "$AUTOSTART_DIR/journal.json"
  ) >"$AUTOSTART_DIR/tape.log" 2>&1 &
  AUTOSTART_PID=$!
  TAPE_PID="$AUTOSTART_PID"
  for _ in $(seq 1 60); do
    healthy && break
    sleep 1
  done
fi

if ! healthy; then
  echo "!! Tape upstream ${UPSTREAM} is not reachable" >&2
  echo "   set TAPE_UPSTREAM/TAPE_SOAK_PID or TAPE_SOAK_AUTOSTART=1" >&2
  exit 1
fi

if [[ -z "$TAPE_PID" ]]; then
  require pgrep
  TAPE_PID="$(pgrep -n '^tape$' || true)"
fi
if [[ -z "$TAPE_PID" ]]; then
  echo "!! cannot determine Tape PID (set TAPE_SOAK_PID=...)" >&2
  exit 1
fi

if ! [[ "$DURATION_SECS" =~ ^[0-9]+$ ]] || (( DURATION_SECS < 2 )); then
  echo "!! TAPE_SOAK_DURATION_SECS must be an integer >= 2" >&2
  exit 1
fi
if ! [[ "$SEED_EVENTS" =~ ^[0-9]+$ ]] || (( SEED_EVENTS < 1 )); then
  echo "!! TAPE_SOAK_EVENTS must be an integer >= 1" >&2
  exit 1
fi
if ! [[ "$RSS_GROWTH_PCT" =~ ^[0-9]+$ ]]; then
  echo "!! TAPE_SOAK_RSS_GROWTH_PCT must be a non-negative integer" >&2
  exit 1
fi
for value in "$FD_GROWTH" "$TASK_GROWTH" "$P99_MS" "$P99_GROWTH_PCT"; do
  [[ "$value" =~ ^[0-9]+$ ]] || {
    echo "!! Tape soak resource and latency limits must be non-negative integers" >&2
    exit 1
  }
done

echo ">> bounded replay soak: tape pid=${TAPE_PID} upstream=http://${UPSTREAM} duration=${DURATION_SECS}s"

append_seed() {
  local n="$1"
  curl -fsS --max-time 10 -X POST "http://${UPSTREAM}/topics/${TOPIC}/append" \
    -H 'content-type: application/json' \
    -d "{\"key\":\"soak-${n}\",\"payload\":{\"id\":${n},\"kind\":\"soak\"}}" |
    jq -e --argjson expected "$n" '.offset == $expected' >/dev/null
}

replay_fixed_window() {
  curl -fsS --max-time 10 "http://${UPSTREAM}/topics/${TOPIC}/replay?limit=${SEED_EVENTS}" |
    jq -e --argjson expected "$SEED_EVENTS" '
      (.events | length) == $expected
      and .events[0].offset == 0
      and .events[-1].offset == ($expected - 1)
    ' >/dev/null
}

checkpoint_fixed_window() {
  curl -fsS --max-time 10 -X PUT \
    "http://${UPSTREAM}/topics/${TOPIC}/consumers/${CONSUMER}/checkpoint" \
    -H 'content-type: application/json' \
    -d "{\"offset\":${SEED_EVENTS}}" |
    jq -e --argjson expected "$SEED_EVENTS" '.offset == $expected' >/dev/null
  curl -fsS --max-time 10 \
    "http://${UPSTREAM}/topics/${TOPIC}/consumers/${CONSUMER}/checkpoint" |
    jq -e --argjson expected "$SEED_EVENTS" '.checkpoint.offset == $expected' >/dev/null
}

latency_probe() {
  curl -fsS --max-time 10 -o /dev/null -w '%{time_total}\n' \
    "http://${UPSTREAM}/topics/${TOPIC}/replay?limit=${SEED_EVENTS}" >>"$LATENCY_FILE"
}

echo ">> seed fixed replay window: ${SEED_EVENTS} events"
for n in $(seq 0 $((SEED_EVENTS - 1))); do
  append_seed "$n"
done

TOTAL_OPS=0
ERR_COUNT=0
load_round() {
  if replay_fixed_window; then TOTAL_OPS=$((TOTAL_OPS + 1)); else ERR_COUNT=$((ERR_COUNT + 1)); fi
  if checkpoint_fixed_window; then TOTAL_OPS=$((TOTAL_OPS + 2)); else ERR_COUNT=$((ERR_COUNT + 1)); fi
  if healthy; then TOTAL_OPS=$((TOTAL_OPS + 1)); else ERR_COUNT=$((ERR_COUNT + 1)); fi
  if latency_probe; then TOTAL_OPS=$((TOTAL_OPS + 1)); else ERR_COUNT=$((ERR_COUNT + 1)); fi
}
run_until() {
  local deadline="$1"
  while [[ $(date +%s) -lt "$deadline" ]]; do
    load_round
  done
}

echo ">> warmup: replay/checkpoint/health"
for _ in 1 2 3; do load_round; done
TOTAL_OPS=0
ERR_COUNT=0
SOAK_TMP_DIR="$(mktemp -d "${TMPDIR:-/tmp}/tape-soak-metrics.XXXXXX")"
LATENCY_A="$SOAK_TMP_DIR/latency-a.seconds"
LATENCY_B="$SOAK_TMP_DIR/latency-b.seconds"

START="$(date +%s)"
HALF=$((DURATION_SECS / 2))
LATENCY_FILE="$LATENCY_A"
run_until $((START + HALF))
RSS_A="$(service_soak_rss_kb "$TAPE_PID")"
FD_A="$(service_soak_fd_count "$TAPE_PID")"
TASK_A="$(service_soak_task_count "$TAPE_PID")"
P99_A="$(service_soak_p99_ms "$LATENCY_A")"
LATENCY_FILE="$LATENCY_B"
run_until $((START + 2 * HALF))
RSS_B="$(service_soak_rss_kb "$TAPE_PID")"
FD_B="$(service_soak_fd_count "$TAPE_PID")"
TASK_B="$(service_soak_task_count "$TAPE_PID")"
P99_B="$(service_soak_p99_ms "$LATENCY_B")"

if ! [[ "$RSS_A" =~ ^[0-9]+$ && "$RSS_B" =~ ^[0-9]+$ && \
  "$FD_A" =~ ^[0-9]+$ && "$FD_B" =~ ^[0-9]+$ && \
  "$TASK_A" =~ ^[0-9]+$ && "$TASK_B" =~ ^[0-9]+$ && \
  "$P99_A" =~ ^[0-9]+$ && "$P99_B" =~ ^[0-9]+$ ]]; then
  echo "!! could not read Tape soak metrics for pid ${TAPE_PID}" >&2
  exit 1
fi
GROWTH_PCT="$(service_soak_percent_growth "$RSS_A" "$RSS_B")"

echo ">> Tape soak report"
echo "   duration:     $((2 * HALF))s (two ${HALF}s steady windows)"
echo "   seed_events:  ${SEED_EVENTS} (fixed before measurement)"
echo "   total_ops:    ${TOTAL_OPS}"
echo "   errors:       ${ERR_COUNT}"
echo "   rss_window_a: ${RSS_A} KB"
echo "   rss_window_b: ${RSS_B} KB"
echo "   steady_drift: ${GROWTH_PCT}% (window A -> window B)"
echo "   fd_window:    ${FD_A} -> ${FD_B}"
echo "   tasks_window: ${TASK_A} -> ${TASK_B}"
echo "   p99_window:   ${P99_A}ms -> ${P99_B}ms"

if (( ERR_COUNT > 0 )); then
  echo "!! replay/checkpoint/health errors observed" >&2
  exit 1
fi
if (( GROWTH_PCT > RSS_GROWTH_PCT )); then
  echo "!! steady-window RSS drift ${GROWTH_PCT}% exceeds ${RSS_GROWTH_PCT}%" >&2
  exit 1
fi
service_soak_assert_max_growth "file descriptor" "$FD_A" "$FD_B" "$FD_GROWTH"
service_soak_assert_max_growth "thread/task" "$TASK_A" "$TASK_B" "$TASK_GROWTH"
service_soak_assert_latency_plateau "$P99_A" "$P99_B" "$P99_MS" "$P99_GROWTH_PCT"

echo ">> Tape bounded replay soak PASS"
# HANDWRITE-END
