#!/usr/bin/env bash
# HANDWRITE-BEGIN gap="missing-generator:e2e-test:defer-bounded-soak" tracker="#766" reason="Defer-specific fixed-task lifecycle workload and RSS plateau assertion."
# Defer bounded scheduled-task soak.
#
# Creates one successful task and one permanently failing task against Defer's
# own HTTP surface. The successful task must remain terminal while the failing
# task repeatedly exercises committed lease/nack/reschedule retry transitions.
# The task keyspace stays fixed so RSS, descriptor, thread/task, and p99 growth
# represent instability rather than expected scheduler-state growth.
# The default warmup performs at least 2,048 queue-control transitions in
# addition to the live retry traffic, filling the bounded 1,024-entry proposal
# cache and crossing the 1,024-entry snapshot cadence before measurement.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
DEFER_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
REPO_ROOT="$(cd "$DEFER_DIR/../.." && pwd)"
source "$REPO_ROOT/libs/service-observability/scripts/soak-metrics.sh"

DURATION_SECS="${DEFER_SOAK_DURATION_SECS:-60}"
UPSTREAM="${DEFER_UPSTREAM:-127.0.0.1:7141}"
RSS_GROWTH_PCT="${DEFER_SOAK_RSS_GROWTH_PCT:-10}"
FD_GROWTH="${DEFER_SOAK_FD_GROWTH:-8}"
TASK_GROWTH="${DEFER_SOAK_TASK_GROWTH:-4}"
P99_MS="${DEFER_SOAK_P99_MS:-250}"
P99_GROWTH_PCT="${DEFER_SOAK_P99_GROWTH_PCT:-100}"
WARMUP_ROUNDS="${DEFER_SOAK_WARMUP_ROUNDS:-1024}"
QUEUE="${DEFER_SOAK_QUEUE:-soak-jobs}"
DEFER_PID="${DEFER_SOAK_PID:-}"
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

healthy() { curl -fsS --max-time 5 "http://${UPSTREAM}/healthz" >/dev/null 2>&1; }

if ! healthy && [[ "${DEFER_SOAK_AUTOSTART:-0}" == "1" ]]; then
  AUTOSTART_DIR="$(mktemp -d "${TMPDIR:-/tmp}/defer-soak.XXXXXX")"
  echo ">> building and launching isolated Defer at http://${UPSTREAM}"
  (cd "$REPO_ROOT" && cargo build -q -p defer --bin defer)
  "$REPO_ROOT/target/debug/defer" serve --bind "$UPSTREAM" --data-dir "$AUTOSTART_DIR/data" \
    >"$AUTOSTART_DIR/defer.log" 2>&1 &
  AUTOSTART_PID=$!
  DEFER_PID="$AUTOSTART_PID"
  for _ in $(seq 1 60); do healthy && break; sleep 1; done
fi

if ! healthy; then
  echo "!! Defer upstream ${UPSTREAM} is not reachable" >&2
  echo "   set DEFER_UPSTREAM/DEFER_SOAK_PID or DEFER_SOAK_AUTOSTART=1" >&2
  exit 1
fi

if [[ -z "$DEFER_PID" ]]; then
  require pgrep
  DEFER_PID="$(pgrep -n '^defer$' || true)"
fi
[[ -n "$DEFER_PID" ]] || { echo "!! cannot determine Defer PID" >&2; exit 1; }

if ! [[ "$DURATION_SECS" =~ ^[0-9]+$ ]] || (( DURATION_SECS < 2 )); then
  echo "!! DEFER_SOAK_DURATION_SECS must be an integer >= 2" >&2
  exit 1
fi
if ! [[ "$RSS_GROWTH_PCT" =~ ^[0-9]+$ ]]; then
  echo "!! DEFER_SOAK_RSS_GROWTH_PCT must be a non-negative integer" >&2
  exit 1
fi
if ! [[ "$WARMUP_ROUNDS" =~ ^[0-9]+$ ]] || (( WARMUP_ROUNDS < 1 )); then
  echo "!! DEFER_SOAK_WARMUP_ROUNDS must be an integer >= 1" >&2
  exit 1
fi
for value in "$FD_GROWTH" "$TASK_GROWTH" "$P99_MS" "$P99_GROWTH_PCT"; do
  [[ "$value" =~ ^[0-9]+$ ]] || {
    echo "!! Defer soak resource and latency limits must be non-negative integers" >&2
    exit 1
  }
done

echo ">> configure fixed queue, one successful task, and one retrying fault task"
curl -fsS --max-time 10 -X PUT "http://${UPSTREAM}/v1/queues/${QUEUE}" \
  -H 'content-type: application/json' \
  -d '{"max_in_flight":16,"max_dispatch_per_tick":16,"max_dispatches_per_second":1000,"max_burst_size":1000,"lease_ttl_ms":30000,"retry_backoff_ms":0}' |
  jq -e '.task_count == 0' >/dev/null
curl -fsS --max-time 10 -X POST "http://${UPSTREAM}/v1/queues/${QUEUE}/tasks" \
  -H 'content-type: application/json' \
  -d "{\"task_id\":\"fixed\",\"target\":{\"url\":\"http://${UPSTREAM}/healthz\",\"method\":\"GET\",\"headers\":{}},\"payload\":{\"kind\":\"soak\"},\"schedule_at\":\"2000-01-01T00:00:00Z\",\"priority\":10,\"max_attempts\":3}" \
  -o /dev/null

for _ in $(seq 1 100); do
  STATUS="$(curl -fsS --max-time 10 "http://${UPSTREAM}/v1/queues/${QUEUE}/tasks/fixed" | jq -r '.status')"
  [[ "$STATUS" == "Succeeded" ]] && break
  sleep 0.1
done
[[ "${STATUS:-}" == "Succeeded" ]] || { echo "!! fixed task did not succeed" >&2; exit 1; }

# A route that Defer does not expose returns a real HTTP 404. Keep max_attempts
# deliberately high and use this queue's explicit zero-delay retry policy so
# the same durable task record exercises retry progress in both measured
# windows without growing the task keyspace or reaching DLQ.
curl -fsS --max-time 10 -X POST "http://${UPSTREAM}/v1/queues/${QUEUE}/tasks" \
  -H 'content-type: application/json' \
  -d "{\"task_id\":\"fixed-retry\",\"target\":{\"url\":\"http://${UPSTREAM}/soak-fault\",\"method\":\"POST\",\"headers\":{}},\"payload\":{\"kind\":\"retry-soak\"},\"schedule_at\":\"2000-01-01T00:00:00Z\",\"priority\":10,\"max_attempts\":1000000}" \
  -o /dev/null

metric_value() {
  local metric="$1"
  curl -fsS --max-time 10 "http://${UPSTREAM}/metrics" |
    awk -v metric="$metric" '$1 == metric { print int($2); found = 1 } END { if (!found) exit 1 }'
}

for _ in $(seq 1 100); do
  RETRIES="$(metric_value defer_dispatch_retried_total)"
  (( RETRIES > 0 )) && break
  sleep 0.1
done
(( ${RETRIES:-0} > 0 )) || { echo "!! retry task made no committed retry progress" >&2; exit 1; }

control() {
  local state="$1"
  curl -fsS --max-time 10 -X POST "http://${UPSTREAM}/v1/queues/${QUEUE}/control" \
    -H 'content-type: application/json' -d "{\"state\":\"${state}\"}" |
    jq -e --arg state "$state" '.control_state == $state' >/dev/null
}

inspect_fixed() {
  curl -fsS --max-time 10 "http://${UPSTREAM}/v1/queues/${QUEUE}" |
    jq -e '.task_count == 2 and .terminal_count == 1' >/dev/null
  curl -fsS --max-time 10 "http://${UPSTREAM}/v1/queues/${QUEUE}/tasks/fixed" |
    jq -e '.status == "Succeeded"' >/dev/null
  curl -fsS --max-time 10 "http://${UPSTREAM}/v1/queues/${QUEUE}/tasks/fixed-retry" |
    jq -e '.status == "Scheduled" or ((.status | type) == "object" and (.status.Leased != null))' >/dev/null
  curl -fsS --max-time 10 "http://${UPSTREAM}/metrics" >/dev/null
  healthy
}

latency_probe() {
  curl -fsS --max-time 10 -o /dev/null -w '%{time_total}\n' \
    "http://${UPSTREAM}/v1/queues/${QUEUE}/tasks/fixed" >>"$LATENCY_FILE"
}

TOTAL_OPS=0
ERR_COUNT=0
load_round() {
  if control Paused; then TOTAL_OPS=$((TOTAL_OPS + 1)); else ERR_COUNT=$((ERR_COUNT + 1)); fi
  if inspect_fixed; then TOTAL_OPS=$((TOTAL_OPS + 5)); else ERR_COUNT=$((ERR_COUNT + 1)); fi
  if control Running; then TOTAL_OPS=$((TOTAL_OPS + 1)); else ERR_COUNT=$((ERR_COUNT + 1)); fi
  if latency_probe; then TOTAL_OPS=$((TOTAL_OPS + 1)); else ERR_COUNT=$((ERR_COUNT + 1)); fi
}
run_until() {
  local deadline="$1"
  while [[ $(date +%s) -lt "$deadline" ]]; do load_round; done
}

echo ">> warmup: ${WARMUP_ROUNDS} fixed-task rounds (fills the bounded 1024-entry proposal cache and crosses one snapshot interval)"
for _ in $(seq 1 "$WARMUP_ROUNDS"); do load_round; done
TOTAL_OPS=0
ERR_COUNT=0
RETRY_START="$(metric_value defer_dispatch_retried_total)"
SOAK_TMP_DIR="$(mktemp -d "${TMPDIR:-/tmp}/defer-soak-metrics.XXXXXX")"
LATENCY_A="$SOAK_TMP_DIR/latency-a.seconds"
LATENCY_B="$SOAK_TMP_DIR/latency-b.seconds"

START="$(date +%s)"
HALF=$((DURATION_SECS / 2))
LATENCY_FILE="$LATENCY_A"
run_until $((START + HALF))
RSS_A="$(service_soak_rss_kb "$DEFER_PID")"
FD_A="$(service_soak_fd_count "$DEFER_PID")"
TASK_A="$(service_soak_task_count "$DEFER_PID")"
P99_A="$(service_soak_p99_ms "$LATENCY_A")"
RETRY_A="$(metric_value defer_dispatch_retried_total)"
LATENCY_FILE="$LATENCY_B"
run_until $((START + 2 * HALF))
RSS_B="$(service_soak_rss_kb "$DEFER_PID")"
FD_B="$(service_soak_fd_count "$DEFER_PID")"
TASK_B="$(service_soak_task_count "$DEFER_PID")"
P99_B="$(service_soak_p99_ms "$LATENCY_B")"
RETRY_B="$(metric_value defer_dispatch_retried_total)"

[[ "$RSS_A" =~ ^[0-9]+$ && "$RSS_B" =~ ^[0-9]+$ && \
  "$FD_A" =~ ^[0-9]+$ && "$FD_B" =~ ^[0-9]+$ && \
  "$TASK_A" =~ ^[0-9]+$ && "$TASK_B" =~ ^[0-9]+$ && \
  "$P99_A" =~ ^[0-9]+$ && "$P99_B" =~ ^[0-9]+$ ]] || {
  echo "!! could not read Defer soak metrics for pid ${DEFER_PID}" >&2
  exit 1
}
GROWTH_PCT="$(service_soak_percent_growth "$RSS_A" "$RSS_B")"

echo ">> Defer soak report"
echo "   duration:     $((2 * HALF))s (two ${HALF}s steady windows)"
echo "   total_ops:    ${TOTAL_OPS}"
echo "   errors:       ${ERR_COUNT}"
echo "   rss_window_a: ${RSS_A} KB"
echo "   rss_window_b: ${RSS_B} KB"
echo "   steady_drift: ${GROWTH_PCT}% (window A -> window B)"
echo "   fd_window:    ${FD_A} -> ${FD_B}"
echo "   tasks_window: ${TASK_A} -> ${TASK_B}"
echo "   p99_window:   ${P99_A}ms -> ${P99_B}ms"
echo "   retry_window: ${RETRY_START} -> ${RETRY_A} -> ${RETRY_B}"

(( TOTAL_OPS > 0 )) || { echo "!! Defer soak observed zero measured operations" >&2; exit 1; }
(( ERR_COUNT == 0 )) || { echo "!! Defer soak errors observed" >&2; exit 1; }
(( RETRY_A > RETRY_START && RETRY_B > RETRY_A )) || {
  echo "!! Defer retry scheduler made no progress in one or both steady windows (${RETRY_START} -> ${RETRY_A} -> ${RETRY_B})" >&2
  exit 1
}
(( GROWTH_PCT <= RSS_GROWTH_PCT )) || {
  echo "!! Defer RSS drift ${GROWTH_PCT}% exceeds ${RSS_GROWTH_PCT}%" >&2
  exit 1
}
service_soak_assert_max_growth "file descriptor" "$FD_A" "$FD_B" "$FD_GROWTH"
service_soak_assert_max_growth "thread/task" "$TASK_A" "$TASK_B" "$TASK_GROWTH"
service_soak_assert_latency_plateau "$P99_A" "$P99_B" "$P99_MS" "$P99_GROWTH_PCT"
echo ">> Defer bounded scheduled-task soak PASS"
# HANDWRITE-END
