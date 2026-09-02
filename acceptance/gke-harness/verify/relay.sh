#!/usr/bin/env bash
set -euo pipefail
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

APP="relay" SERVICE="relay" REMOTE_PORT=7000 LOCAL_PORT=17000
# shellcheck source=_lib.sh
source "$SCRIPT_DIR/_lib.sh"

subject="acceptance"

# POST /v1/{subject}/publish appends to the durable log; GET /v1/{subject}/len
# returns {latest_seq}, which must stay >= 1 across the pod restart.
app_round_trip_write() {
  http_json step2-write.json 200 POST "/v1/$subject/publish" \
    "{\"message_id\":\"rt-$RUN_ID\",\"payload\":{\"n\":1}}"
}

app_round_trip_read() {
  local out="$1"
  http_json "$out" 200 GET "/v1/$subject/len"
  jq -e '.latest_seq >= 1' "$EVIDENCE_DIR/$out" >/dev/null || {
    echo "relay: latest_seq did not survive for subject $subject" >&2
    cat "$EVIDENCE_DIR/$out" >&2
    return 1
  }
}

verify_main
