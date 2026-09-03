#!/usr/bin/env bash
set -euo pipefail
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

APP="keep" SERVICE="keep" REMOTE_PORT=7117 LOCAL_PORT=17117
# shellcheck source=_lib.sh
source "$SCRIPT_DIR/_lib.sh"

key="rt-$RUN_ID"

# PUT /kv/{key} {"value": ...} -> 200; GET reads it back (apps/keep/tests/http_api.rs).
app_round_trip_write() {
  http_json step2-write.json 200 PUT "/kv/$key" \
    "{\"value\":{\"n\":1,\"run\":\"$RUN_ID\"}}"
}

app_round_trip_read() {
  local out="$1"
  http_json "$out" 200 GET "/kv/$key"
  jq -e --arg run "$RUN_ID" '.value.run == $run and .value.n == 1' \
    "$EVIDENCE_DIR/$out" >/dev/null || {
    echo "keep: read-back value mismatch for $key" >&2
    cat "$EVIDENCE_DIR/$out" >&2
    return 1
  }
}

verify_main
