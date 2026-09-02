# shellcheck shell=bash
# Shared verify driver for the StatefulSet-essentials contract. Sourced by
# verify/<app>.sh, which must set APP, SERVICE, REMOTE_PORT, LOCAL_PORT and
# define two functions before calling verify_main:
#   app_round_trip_write  — write one run-scoped datum through the API
#   app_round_trip_read   — read it back and FAIL (non-zero) on any mismatch
#
# The contract is three steps, each leaving evidence in $EVIDENCE_DIR:
#   1. /readyz answers 200 through a port-forward
#   2. the write/read round-trip holds
#   3. pod-0 is deleted, reschedules to Ready, and the SAME read still holds —
#      the PVC-durability proof, and the reason these apps are StatefulSets.

: "${APP:?APP is required}"
: "${RUN_ID:?RUN_ID is required}"
: "${EVIDENCE_DIR:?EVIDENCE_DIR is required}"
: "${SERVICE:?SERVICE is required}"
: "${REMOTE_PORT:?REMOTE_PORT is required}"
: "${LOCAL_PORT:?LOCAL_PORT is required}"

ns="$APP"
base_url="http://127.0.0.1:${LOCAL_PORT}"
forward_pid=""

stop_forward() {
  if [[ -n "$forward_pid" ]]; then
    kill "$forward_pid" >/dev/null 2>&1 || true
    wait "$forward_pid" >/dev/null 2>&1 || true
    forward_pid=""
  fi
}
trap stop_forward EXIT INT
# Same rationale as verify-tape.sh: a stray TERM into a recycled pid must not
# kill the verify mid-loop; the deadline TERM targets the orchestrator, not us.
trap '' TERM

start_forward() {
  stop_forward
  local deadline=$((SECONDS + 120))
  while (( SECONDS < deadline )); do
    # port-forward pins one endpoint; step 3 replaces the pod under it, so the
    # forward is recreated whenever it dies rather than treated as a failure.
    if [[ -z "$forward_pid" ]] || ! kill -0 "$forward_pid" >/dev/null 2>&1; then
      stop_forward
      kubectl -n "$ns" port-forward "service/$SERVICE" "${LOCAL_PORT}:${REMOTE_PORT}" \
        >>"$EVIDENCE_DIR/port-forward.log" 2>&1 &
      forward_pid="$!"
      sleep 1
    fi
    if curl --max-time 5 --silent --fail "$base_url/readyz" >/dev/null 2>&1; then
      return 0
    fi
    sleep 2
  done
  echo "timed out waiting for $APP /readyz through port-forward" >&2
  return 1
}

# JSON helper: http_json <evidence-file> <expected-status> <method> <path> [body]
# Writes the response body to the evidence file and fails on a status mismatch.
http_json() {
  local out="$1" want="$2" method="$3" path="$4" body="${5:-}"
  local args=(--max-time 15 --silent --show-error -o "$EVIDENCE_DIR/$out" \
    -w '%{http_code}' -X "$method" "$base_url$path")
  [[ -n "$body" ]] && args+=(-H 'content-type: application/json' --data "$body")
  local status
  status="$(curl "${args[@]}")"
  if [[ "$status" != "$want" ]]; then
    echo "$APP: $method $path returned $status, expected $want" >&2
    cat "$EVIDENCE_DIR/$out" >&2 || true
    return 1
  fi
}

kill_pod_and_wait() {
  echo "deleting ${APP}-0 to prove PVC durability" >&2
  kubectl -n "$ns" delete pod "${APP}-0" --wait=true --timeout=180s >&2
  # After delete returns the old pod is gone; the StatefulSet recreates -0.
  kubectl -n "$ns" wait --for=condition=Ready "pod/${APP}-0" --timeout=300s >&2
}

verify_main() {
  mkdir -p "$EVIDENCE_DIR"

  echo "[$APP] step 1: readyz" >&2
  start_forward
  curl --max-time 5 --silent "$base_url/readyz" \
    > "$EVIDENCE_DIR/step1-readyz.txt" || true

  echo "[$APP] step 2: API round-trip" >&2
  app_round_trip_write
  app_round_trip_read step2-read.json

  echo "[$APP] step 3: pod kill + data survives" >&2
  kill_pod_and_wait
  start_forward
  app_round_trip_read step3-read-after-restart.json

  echo "[$APP] PASS: readyz + round-trip + durability" | tee "$EVIDENCE_DIR/verdict.txt" >&2
}
