#!/usr/bin/env bash
set -euo pipefail
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

APP="loom" SERVICE="loom" REMOTE_PORT=7474 LOCAL_PORT=17474
# shellcheck source=_lib.sh
source "$SCRIPT_DIR/_lib.sh"

run_id="rt-$RUN_ID"

# The template overlay drops LOOM_RELAY/LOOM_KEEP, so the controller uses the
# in-process MemDispatcher while the raft store stays on the PVC — the run
# submitted here must still be readable after pod-0 is replaced.
app_round_trip_write() {
  http_json step2-write.json 201 POST "/runs" \
    "{\"run_id\":\"$run_id\",\"nodes\":[{\"id\":\"a\",\"task_name\":\"t\"}]}"
}

app_round_trip_read() {
  local out="$1"
  http_json "$out" 200 GET "/runs/$run_id"
  jq -e --arg id "$run_id" '.run_id == $id' "$EVIDENCE_DIR/$out" >/dev/null || {
    echo "loom: run read-back mismatch for $run_id" >&2
    cat "$EVIDENCE_DIR/$out" >&2
    return 1
  }
}

verify_main
