#!/usr/bin/env bash
# HANDWRITE-BEGIN gap="missing-generator:e2e-test:relay-bounded-soak" tracker="#125" reason="Relay-specific fixed-state producer/lease heartbeat workload and RSS plateau assertion."
# Relay bounded work-queue soak.
#
# Seeds and leases one message, then repeatedly exercises idempotent publish,
# fenced heartbeat, log inspection, metrics, and health over two steady
# windows. The keyspace and live lease count stay fixed so RSS, descriptor,
# thread/task, and p99 growth signal instability rather than backlog growth.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
RELAY_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
REPO_ROOT="$(cd "$RELAY_DIR/../.." && pwd)"
source "$REPO_ROOT/libs/service-observability/scripts/soak-metrics.sh"

DURATION_SECS="${RELAY_SOAK_DURATION_SECS:-60}"
UPSTREAM="${RELAY_UPSTREAM:-127.0.0.1:7000}"
RSS_GROWTH_PCT="${RELAY_SOAK_RSS_GROWTH_PCT:-10}"
FD_GROWTH="${RELAY_SOAK_FD_GROWTH:-8}"
TASK_GROWTH="${RELAY_SOAK_TASK_GROWTH:-4}"
P99_MS="${RELAY_SOAK_P99_MS:-250}"
P99_GROWTH_PCT="${RELAY_SOAK_P99_GROWTH_PCT:-100}"
SUBJECT="${RELAY_SOAK_SUBJECT:-soak-jobs}"
RELAY_PID="${RELAY_SOAK_PID:-}"
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
  [[ -z "$AUTOSTART_DIR" ]] || rm -rf "$AUTOSTART_DIR"
  [[ -z "$SOAK_TMP_DIR" ]] || rm -rf "$SOAK_TMP_DIR"
  exit "$ec"
}
trap cleanup EXIT
trap 'exit 130' INT TERM

for cmd in curl jq ps; do require "$cmd"; done

healthy() {
  curl -fsS --max-time 5 "http://${UPSTREAM}/healthz" >/dev/null 2>&1
}

if ! healthy && [[ "${RELAY_SOAK_AUTOSTART:-0}" == "1" ]]; then
  AUTOSTART_DIR="$(mktemp -d "${TMPDIR:-/tmp}/relay-soak.XXXXXX")"
  echo ">> building and launching isolated Relay at http://${UPSTREAM}"
  (cd "$REPO_ROOT" && cargo build -q -p relay --bin relay)
  "$REPO_ROOT/target/debug/relay" --bind "$UPSTREAM" --data-dir "$AUTOSTART_DIR/data" \
    >"$AUTOSTART_DIR/relay.log" 2>&1 &
  AUTOSTART_PID=$!
  RELAY_PID="$AUTOSTART_PID"
  for _ in $(seq 1 60); do healthy && break; sleep 1; done
fi

if ! healthy; then
  echo "!! Relay upstream ${UPSTREAM} is not reachable" >&2
  echo "   set RELAY_UPSTREAM/RELAY_SOAK_PID or RELAY_SOAK_AUTOSTART=1" >&2
  exit 1
fi

if [[ -z "$RELAY_PID" ]]; then
  require pgrep
  RELAY_PID="$(pgrep -n '^relay$' || true)"
fi
[[ -n "$RELAY_PID" ]] || { echo "!! cannot determine Relay PID" >&2; exit 1; }

if ! [[ "$DURATION_SECS" =~ ^[0-9]+$ ]] || (( DURATION_SECS < 2 )); then
  echo "!! RELAY_SOAK_DURATION_SECS must be an integer >= 2" >&2
  exit 1
fi
if ! [[ "$RSS_GROWTH_PCT" =~ ^[0-9]+$ ]]; then
  echo "!! RELAY_SOAK_RSS_GROWTH_PCT must be a non-negative integer" >&2
  exit 1
fi
for value in "$FD_GROWTH" "$TASK_GROWTH" "$P99_MS" "$P99_GROWTH_PCT"; do
  [[ "$value" =~ ^[0-9]+$ ]] || {
    echo "!! Relay soak resource and latency limits must be non-negative integers" >&2
    exit 1
  }
done

publish_fixed() {
  curl -fsS --max-time 10 -X POST "http://${UPSTREAM}/v1/${SUBJECT}/publish" \
    -H 'content-type: application/json' \
    -d '{"message_id":"fixed","payload":{"kind":"soak"},"headers":{},"priority":10}' |
    jq -e '.seq == 0' >/dev/null
}

echo ">> seed one fixed message and acquire one fenced lease"
publish_fixed
LEASE="$(curl -fsS --max-time 10 -X POST "http://${UPSTREAM}/v1/${SUBJECT}/lease" \
  -H 'content-type: application/json' -d '{"consumer_id":"soak-worker"}')"
LEASE_ID="$(jq -er '.lease.lease_id' <<<"$LEASE")"
EPOCH="$(jq -er '.lease.epoch' <<<"$LEASE")"

heartbeat() {
  curl -fsS --max-time 10 -X POST "http://${UPSTREAM}/v1/${SUBJECT}/heartbeat" \
    -H 'content-type: application/json' \
    -d "{\"lease_id\":\"${LEASE_ID}\",\"epoch\":${EPOCH}}" |
    jq -e '.extended == true' >/dev/null
}

inspect_fixed() {
  curl -fsS --max-time 10 "http://${UPSTREAM}/v1/${SUBJECT}/len" |
    jq -e '.latest_seq == 1' >/dev/null
  curl -fsS --max-time 10 "http://${UPSTREAM}/metrics" >/dev/null
  healthy
}

latency_probe() {
  curl -fsS --max-time 10 -o /dev/null -w '%{time_total}\n' \
    "http://${UPSTREAM}/v1/${SUBJECT}/len" >>"$LATENCY_FILE"
}

TOTAL_OPS=0
ERR_COUNT=0
load_round() {
  if publish_fixed; then TOTAL_OPS=$((TOTAL_OPS + 1)); else ERR_COUNT=$((ERR_COUNT + 1)); fi
  if heartbeat; then TOTAL_OPS=$((TOTAL_OPS + 1)); else ERR_COUNT=$((ERR_COUNT + 1)); fi
  if inspect_fixed; then TOTAL_OPS=$((TOTAL_OPS + 3)); else ERR_COUNT=$((ERR_COUNT + 1)); fi
  if latency_probe; then TOTAL_OPS=$((TOTAL_OPS + 1)); else ERR_COUNT=$((ERR_COUNT + 1)); fi
}
run_until() {
  local deadline="$1"
  while [[ $(date +%s) -lt "$deadline" ]]; do load_round; done
}

echo ">> warmup: fixed-key publish/heartbeat/inspect"
for _ in 1 2 3; do load_round; done
TOTAL_OPS=0
ERR_COUNT=0
SOAK_TMP_DIR="$(mktemp -d "${TMPDIR:-/tmp}/relay-soak-metrics.XXXXXX")"
LATENCY_A="$SOAK_TMP_DIR/latency-a.seconds"
LATENCY_B="$SOAK_TMP_DIR/latency-b.seconds"

START="$(date +%s)"
HALF=$((DURATION_SECS / 2))
LATENCY_FILE="$LATENCY_A"
run_until $((START + HALF))
RSS_A="$(service_soak_rss_kb "$RELAY_PID")"
FD_A="$(service_soak_fd_count "$RELAY_PID")"
TASK_A="$(service_soak_task_count "$RELAY_PID")"
P99_A="$(service_soak_p99_ms "$LATENCY_A")"
LATENCY_FILE="$LATENCY_B"
run_until $((START + 2 * HALF))
RSS_B="$(service_soak_rss_kb "$RELAY_PID")"
FD_B="$(service_soak_fd_count "$RELAY_PID")"
TASK_B="$(service_soak_task_count "$RELAY_PID")"
P99_B="$(service_soak_p99_ms "$LATENCY_B")"

[[ "$RSS_A" =~ ^[0-9]+$ && "$RSS_B" =~ ^[0-9]+$ && \
  "$FD_A" =~ ^[0-9]+$ && "$FD_B" =~ ^[0-9]+$ && \
  "$TASK_A" =~ ^[0-9]+$ && "$TASK_B" =~ ^[0-9]+$ && \
  "$P99_A" =~ ^[0-9]+$ && "$P99_B" =~ ^[0-9]+$ ]] || {
  echo "!! could not read Relay soak metrics for pid ${RELAY_PID}" >&2
  exit 1
}
GROWTH_PCT="$(service_soak_percent_growth "$RSS_A" "$RSS_B")"

curl -fsS --max-time 10 -X POST "http://${UPSTREAM}/v1/${SUBJECT}/ack" \
  -H 'content-type: application/json' \
  -d "{\"lease_id\":\"${LEASE_ID}\",\"epoch\":${EPOCH}}" |
  jq -e '.acked == true' >/dev/null

echo ">> Relay soak report"
echo "   duration:     $((2 * HALF))s (two ${HALF}s steady windows)"
echo "   total_ops:    ${TOTAL_OPS}"
echo "   errors:       ${ERR_COUNT}"
echo "   rss_window_a: ${RSS_A} KB"
echo "   rss_window_b: ${RSS_B} KB"
echo "   steady_drift: ${GROWTH_PCT}% (window A -> window B)"
echo "   fd_window:    ${FD_A} -> ${FD_B}"
echo "   tasks_window: ${TASK_A} -> ${TASK_B}"
echo "   p99_window:   ${P99_A}ms -> ${P99_B}ms"

(( ERR_COUNT == 0 )) || { echo "!! Relay soak errors observed" >&2; exit 1; }
(( GROWTH_PCT <= RSS_GROWTH_PCT )) || {
  echo "!! Relay RSS drift ${GROWTH_PCT}% exceeds ${RSS_GROWTH_PCT}%" >&2
  exit 1
}
service_soak_assert_max_growth "file descriptor" "$FD_A" "$FD_B" "$FD_GROWTH"
service_soak_assert_max_growth "thread/task" "$TASK_A" "$TASK_B" "$TASK_GROWTH"
service_soak_assert_latency_plateau "$P99_A" "$P99_B" "$P99_MS" "$P99_GROWTH_PCT"
echo ">> Relay bounded work-queue soak PASS"
# HANDWRITE-END
