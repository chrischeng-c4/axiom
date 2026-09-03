#!/usr/bin/env bash
set -euo pipefail
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

APP="defer" SERVICE="defer" REMOTE_PORT=7141 LOCAL_PORT=17141
# shellcheck source=_lib.sh
source "$SCRIPT_DIR/_lib.sh"

queue="acceptance"
task_id="rt-$RUN_ID"

# The queue must exist before a task lands in it (QueueMissing -> 400), and the
# far-future schedule_at keeps the dispatcher from ever calling the dummy
# target — the datum just has to sit durably on the raft log.
app_round_trip_write() {
  http_json step2-queue-put.json 200 PUT "/v1/queues/$queue" \
    '{"max_in_flight":100,"max_dispatch_per_tick":100,"max_dispatches_per_second":100,"max_burst_size":100,"lease_ttl_ms":30000,"retry_backoff_ms":1000}'
  http_json step2-write.json 201 POST "/v1/queues/$queue/tasks" \
    "{\"task_id\":\"$task_id\",\"target\":{\"url\":\"http://127.0.0.1:9/never\"},\"schedule_at\":\"2099-01-01T00:00:00Z\"}"
}

app_round_trip_read() {
  local out="$1"
  http_json "$out" 200 GET "/v1/queues/$queue/tasks/$task_id"
  jq -e --arg id "$task_id" '.task_id == $id and (.status | length > 0)' \
    "$EVIDENCE_DIR/$out" >/dev/null || {
    echo "defer: task read-back mismatch for $task_id" >&2
    cat "$EVIDENCE_DIR/$out" >&2
    return 1
  }
}

verify_main
