#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ACCEPTANCE_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
source "$ACCEPTANCE_ROOT/scripts/acceptance-lock.sh"
source "$ACCEPTANCE_ROOT/scripts/process-tree.sh"

test_root="$(mktemp -d "${TMPDIR:-/tmp}/sift-cleanup-session.XXXXXX")"
fake_bin="$test_root/bin"
state_dir="$test_root/state"
evidence_dir="$test_root/evidence"
local_claim_root="$test_root/claims"
calls="$test_root/calls.log"
lock_state="$test_root/acceptance-lock.json"

cleanup_test() {
  find "$test_root" -type f -delete
  find "$test_root" -depth -type d -empty -delete
}
trap cleanup_test EXIT INT TERM

mkdir -p "$fake_bin" "$state_dir" "$evidence_dir"
: > "$calls"
active_session="bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"
active_start_token="$(process_start_token "$$")"
[[ -n "$active_start_token" ]]
lock_resource="$(
  acceptance_lock_manifest \
    "test-project" "session-red" "sift" "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa" \
    | jq --arg session "$active_session" '
        .metadata.uid="session-lock-uid"
        | .metadata.resourceVersion="31"
        | .metadata.annotations["axiom.axiom.dev/cleanup-session-id"]=$session
        | .metadata.annotations["axiom.axiom.dev/cleanup-started-at"]="2026-09-02T00:00:00Z"
      '
)"
printf '%s\n' "$lock_resource" > "$lock_state"
write_acceptance_lock_receipt \
  "$evidence_dir/acceptance-lock.json" "$lock_resource" \
  "test-project" "session-red" "sift" "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
session_intent_root="$evidence_dir/acceptance-cleanup-session-intents"
session_intent="$(acceptance_cleanup_session_intent_path \
  "$session_intent_root" "$active_session")"
write_acceptance_cleanup_session_intent \
  "$session_intent" "test-project" "session-red" "sift" \
  "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa" "$active_session" \
  "$$" "$active_start_token"
write_acceptance_cleanup_session_receipt \
  "$evidence_dir/acceptance-cleanup-session.json" "$lock_resource" \
  "test-project" "session-red" "sift" \
  "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa" "$active_session"
write_acceptance_run_owner \
  "$(acceptance_run_claim_path \
    "$local_claim_root" "test-project" "session-red" "sift")" \
  "test-project" "session-red" "sift" "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa" \
  "$state_dir" "$evidence_dir" "99999999" "99999999" "test-dead-owner" \
  "dddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddd"

cat > "$fake_bin/kubectl" <<'EOF'
#!/usr/bin/env bash
set -euo pipefail
printf 'kubectl %s\n' "$*" >> "${SIFT_SESSION_TEST_CALLS:?}"
if [[ " $* " == *" get lease axiom-gcp-operator-acceptance-lock --namespace kube-system -o json "* ]]; then
  cat "${SIFT_SESSION_TEST_LOCK_STATE:?}"
  exit 0
fi
echo "not found" >&2
exit 1
EOF

cat > "$fake_bin/gcloud" <<'EOF'
#!/usr/bin/env bash
set -euo pipefail
printf 'gcloud %s\n' "$*" >> "${SIFT_SESSION_TEST_CALLS:?}"
exit 0
EOF

cat > "$fake_bin/terraform" <<'EOF'
#!/usr/bin/env bash
printf 'terraform %s\n' "$*" >> "${SIFT_SESSION_TEST_CALLS:?}"
exit 0
EOF

chmod +x "$fake_bin/kubectl" "$fake_bin/gcloud" "$fake_bin/terraform"

set +e
PATH="$fake_bin:$PATH" \
SIFT_SESSION_TEST_CALLS="$calls" \
SIFT_SESSION_TEST_LOCK_STATE="$lock_state" \
ACCEPTANCE_LOCAL_CLAIM_ROOT="$local_claim_root" \
PROJECT_ID="test-project" \
REGION="asia-east1" \
GKE_ZONE="asia-east1-a" \
RUN_ID="session-red" \
STATE_DIR="$state_dir" \
ACCEPTANCE_ROOT="$ACCEPTANCE_ROOT" \
REGISTRY="example.invalid" \
IMAGE_TAG="candidate-session-red" \
GCS_SOURCE_PREFIX="gs://test-source/source/axiom-gcp-operator-session-red" \
EVIDENCE_DIR="$evidence_dir" \
ACCEPTANCE_APPS="sift" \
bash "$ACCEPTANCE_ROOT/scripts/cleanup.sh" >"$test_root/cleanup.log" 2>&1
status=$?
set -e

[[ "$status" -ne 0 ]] || {
  echo "a second cleanup reused an active cleanup session" >&2
  exit 1
}
if rg -q '^kubectl (patch|delete) ' "$calls" \
    || rg -q '^gcloud (builds cancel|artifacts docker tags delete|artifacts docker images delete|storage rm) ' "$calls" \
    || rg -q '^terraform .* destroy ' "$calls"; then
  echo "a second cleanup changed state while another cleanup session was active" >&2
  cat "$calls" >&2
  exit 1
fi

echo "cleanup session fence E2E: ok"
