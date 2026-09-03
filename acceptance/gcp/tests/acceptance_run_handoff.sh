#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ACCEPTANCE_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
source "$ACCEPTANCE_ROOT/scripts/acceptance-lock.sh"
source "$ACCEPTANCE_ROOT/scripts/process-tree.sh"

test_root="$(mktemp -d "${TMPDIR:-/tmp}/sift-run-handoff.XXXXXX")"
fake_bin="$test_root/bin"
state_dir="$test_root/state"
evidence_dir="$test_root/evidence"
local_claim_root="$test_root/claims"
calls="$test_root/calls.log"
test_member_pid=""
owner_pid="$$"
owner_pgid="77777"
owner_start_token="$(process_start_token "$owner_pid")"
[[ -n "$owner_start_token" ]]
acquisition_id="aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
handoff_nonce="cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc"
handoff_digest="$(
  printf '%s' "$handoff_nonce" | openssl dgst -sha256 | awk '{print $NF}'
)"

cleanup_test() {
  if [[ -n "$test_member_pid" ]]; then
    kill "$test_member_pid" >/dev/null 2>&1 || true
    wait "$test_member_pid" >/dev/null 2>&1 || true
  fi
  find "$test_root" -type f -delete
  find "$test_root" -depth -type d -empty -delete
}
trap cleanup_test EXIT INT TERM

mkdir -p "$fake_bin" "$state_dir" "$evidence_dir"
: > "$calls"
resource="$(
  acceptance_lock_manifest \
    "test-project" "handoff-red" "sift" "$acquisition_id" \
    | jq '.metadata.uid="handoff-lock-uid" | .metadata.resourceVersion="30"'
)"
write_acceptance_lock_receipt \
  "$evidence_dir/acceptance-lock.json" "$resource" \
  "test-project" "handoff-red" "sift" "$acquisition_id"
write_acceptance_run_owner \
  "$(acceptance_run_claim_path \
    "$local_claim_root" "test-project" "handoff-red" "sift")" \
  "test-project" "handoff-red" "sift" "$acquisition_id" \
  "$state_dir" "$evidence_dir" "$owner_pid" "$owner_pgid" "$owner_start_token" \
  "$handoff_digest"

cat > "$fake_bin/kubectl" <<'EOF'
#!/usr/bin/env bash
printf 'kubectl %s\n' "$*" >> "${SIFT_HANDOFF_CALLS:?}"
exit 97
EOF

cat > "$fake_bin/gcloud" <<'EOF'
#!/usr/bin/env bash
printf 'gcloud %s\n' "$*" >> "${SIFT_HANDOFF_CALLS:?}"
exit 97
EOF

cat > "$fake_bin/terraform" <<'EOF'
#!/usr/bin/env bash
printf 'terraform %s\n' "$*" >> "${SIFT_HANDOFF_CALLS:?}"
exit 97
EOF

chmod +x "$fake_bin/kubectl" "$fake_bin/gcloud" "$fake_bin/terraform"

run_cleanup() {
  PATH="$fake_bin:$PATH" \
  SIFT_HANDOFF_CALLS="$calls" \
  ACCEPTANCE_LOCAL_CLAIM_ROOT="$local_claim_root" \
  PROJECT_ID="test-project" \
  REGION="asia-east1" \
  GKE_ZONE="asia-east1-a" \
  RUN_ID="handoff-red" \
  STATE_DIR="$state_dir" \
  ACCEPTANCE_ROOT="$ACCEPTANCE_ROOT" \
  REGISTRY="example.invalid" \
  IMAGE_TAG="candidate-handoff-red" \
  GCS_SOURCE_PREFIX="gs://test-source/source/axiom-gcp-operator-handoff-red" \
  EVIDENCE_DIR="$evidence_dir" \
  ACCEPTANCE_APPS="sift" \
  bash "$ACCEPTANCE_ROOT/scripts/cleanup.sh"
}

set +e
run_cleanup >"$test_root/active-owner.log" 2>&1
active_status=$?
set -e
[[ "$active_status" -ne 0 ]] || {
  echo "recovery cleanup accepted an active run owner" >&2
  exit 1
}
rg -q 'recorded acceptance run is still active' "$test_root/active-owner.log"
[[ ! -s "$calls" ]] || {
  echo "recovery cleanup reached shared resources while the run owner was active" >&2
  cat "$calls" >&2
  exit 1
}

# A live PID whose birth token cannot be read is unsafe. Recovery must stop
# before it reaches Kubernetes, GCP, or Terraform.
token_fail_bin="$test_root/token-fail-bin"
mkdir -p "$token_fail_bin"
cat > "$token_fail_bin/python3" <<'EOF'
#!/usr/bin/env bash
exit 4
EOF
chmod +x "$token_fail_bin/python3"
: > "$calls"
set +e
PATH="$token_fail_bin:$PATH" \
  run_cleanup >"$test_root/token-read-failure.log" 2>&1
token_read_failure_status=$?
set -e
[[ "$token_read_failure_status" -ne 0 ]] || {
  echo "cleanup accepted a live owner whose process token was unreadable" >&2
  exit 1
}
rg -q 'cannot verify the live acceptance run process generation' \
  "$test_root/token-read-failure.log"
[[ ! -s "$calls" ]] || {
  echo "an unreadable live-owner token reached shared resources" >&2
  cat "$calls" >&2
  exit 1
}

set +e
ACCEPTANCE_RUN_OWNER_ACQUISITION_ID="$acquisition_id" \
  run_cleanup >"$test_root/acquisition-only.log" 2>&1
acquisition_only_status=$?
set -e
[[ "$acquisition_only_status" -ne 0 ]] || {
  echo "cleanup accepted the public acquisition ID as a handoff capability" >&2
  exit 1
}
rg -q 'recorded acceptance run is still active' "$test_root/acquisition-only.log"
[[ ! -s "$calls" ]] || {
  echo "the acquisition ID alone reached shared resources" >&2
  cat "$calls" >&2
  exit 1
}

: > "$calls"
set +e
ACCEPTANCE_RUN_OWNER_HANDOFF_NONCE="bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb" \
  run_cleanup >"$test_root/wrong-handoff.log" 2>&1
wrong_handoff_status=$?
set -e
[[ "$wrong_handoff_status" -ne 0 ]] || {
  echo "cleanup accepted the wrong one-time handoff nonce" >&2
  exit 1
}
rg -q 'not handed off by the recorded run owner' "$test_root/wrong-handoff.log"
[[ ! -s "$calls" ]] || {
  echo "a wrong handoff nonce reached shared resources" >&2
  cat "$calls" >&2
  exit 1
}

# Only the nonce held by the parent run can authorize its direct cleanup child.
: > "$calls"
set +e
ACCEPTANCE_RUN_OWNER_HANDOFF_NONCE="$handoff_nonce" \
  run_cleanup >"$test_root/correct-handoff.log" 2>&1
correct_handoff_status=$?
set -e
[[ "$correct_handoff_status" -ne 0 && -s "$calls" ]] || {
  echo "the one-time handoff nonce did not authorize the direct cleanup child" >&2
  exit 1
}
! rg -q 'not handed off by the recorded run owner' "$test_root/correct-handoff.log"

# A reused PID belongs to a different process generation. The old token must
# not block recovery after the PID has been recycled.
write_acceptance_run_owner \
  "$(acceptance_run_claim_path \
    "$local_claim_root" "test-project" "handoff-red" "sift")" \
  "test-project" "handoff-red" "sift" "$acquisition_id" \
  "$state_dir" "$evidence_dir" "$owner_pid" "$owner_pgid" \
  "old-owner-generation" "$handoff_digest"
printf '%s\t%s\n' "$owner_pid" "old-descendant-generation" \
  > "$state_dir/watchdog-descendants.txt"
: > "$calls"
set +e
run_cleanup >"$test_root/reused-generation.log" 2>&1
reused_generation_status=$?
set -e
[[ "$reused_generation_status" -ne 0 && -s "$calls" ]] || {
  echo "cleanup did not pass authorization for a reused PID generation" >&2
  exit 1
}
! rg -q 'process group is still active' "$test_root/reused-generation.log" || {
  echo "cleanup treated a reused PID as the old process generation" >&2
  exit 1
}

# A recorded group member with the same high-resolution token still belongs
# to the old run. Recovery must wait for that exact process generation.
printf '%s\t%s\n' "$owner_pid" "$owner_start_token" \
  > "$state_dir/watchdog-descendants.txt"
: > "$calls"
set +e
run_cleanup >"$test_root/live-descendant.log" 2>&1
live_descendant_status=$?
set -e
[[ "$live_descendant_status" -ne 0 ]] || {
  echo "cleanup accepted a still-live recorded process-group member" >&2
  exit 1
}
rg -q 'recorded acceptance process group is still active' \
  "$test_root/live-descendant.log"
[[ ! -s "$calls" ]] || {
  echo "a live recorded process generation reached shared resources" >&2
  cat "$calls" >&2
  exit 1
}

# A scanned group member whose atomic snapshot token cannot be verified makes
# the whole scan fail. The scanner must not replace the last complete record
# with a partial list. Recovery then uses the durable scan-failure marker.
sleep 30 &
test_member_pid="$!"
member_token_helper="$test_root/member-token-helper"
cat > "$member_token_helper" <<'EOF'
#!/usr/bin/env bash
if [[ "$1" == "${SIFT_UNREADABLE_MEMBER_PID:?}" ]]; then
  exit 4
fi
exec "${SIFT_REAL_PROCESS_TOKEN_HELPER:?}" "$@"
EOF
chmod +x "$member_token_helper"
member_ps_bin="$test_root/member-ps-bin"
mkdir -p "$member_ps_bin"
cat > "$member_ps_bin/ps" <<'EOF'
#!/usr/bin/env bash
if [[ "${1:-}" == "-axo" ]]; then
  printf '%s %s %s\n' "${SIFT_PROCESS_SCAN_ROOT_PID:?}" "1" "88888"
  printf '%s %s %s\n' "${SIFT_UNREADABLE_MEMBER_PID:?}" "1" "77777"
  exit 0
fi
exec /bin/ps "$@"
EOF
chmod +x "$member_ps_bin/ps"
group_records="$state_dir/watchdog-descendants.txt"
complete_record="$(<"$group_records")"
if PATH="$member_ps_bin:$PATH" \
    PROCESS_START_TOKEN_HELPER="$member_token_helper" \
    SIFT_UNREADABLE_MEMBER_PID="$test_member_pid" \
    SIFT_PROCESS_SCAN_ROOT_PID="$owner_pid" \
    SIFT_REAL_PROCESS_TOKEN_HELPER="$ACCEPTANCE_ROOT/scripts/process-start-token.py" \
    record_process_group_members \
      "77777" "$$" "" "$group_records"; then
  echo "process scan accepted an unverifiable live generation" >&2
  exit 1
fi
[[ "$(<"$group_records")" == "$complete_record" ]] || {
  echo "failed process scan replaced the last complete record" >&2
  exit 1
}
printf '%s\n' "injected unreadable process generation" \
  > "$state_dir/process-scan-unsafe.txt"

# The direct parent handoff is not allowed to bypass the durable process
# fence. This is the path used by run.sh's EXIT trap after it has a nonce.
printf '%s\n' "complete" > "$state_dir/watchdog-ready.txt"
write_acceptance_run_owner \
  "$(acceptance_run_claim_path \
    "$local_claim_root" "test-project" "handoff-red" "sift")" \
  "test-project" "handoff-red" "sift" "$acquisition_id" \
  "$state_dir" "$evidence_dir" "$owner_pid" "$owner_pgid" \
  "$owner_start_token" "$handoff_digest"
: > "$calls"
set +e
ACCEPTANCE_RUN_OWNER_HANDOFF_NONCE="$handoff_nonce" \
PROCESS_START_TOKEN_HELPER="$member_token_helper" \
SIFT_UNREADABLE_MEMBER_PID="$test_member_pid" \
SIFT_REAL_PROCESS_TOKEN_HELPER="$ACCEPTANCE_ROOT/scripts/process-start-token.py" \
  run_cleanup >"$test_root/unreadable-member-handoff.log" 2>&1
unreadable_member_handoff_status=$?
set -e
[[ "$unreadable_member_handoff_status" -ne 0 ]] || {
  echo "direct EXIT handoff accepted an unverifiable process-group member" >&2
  exit 1
}
rg -q 'acceptance process-group scan is incomplete' \
  "$test_root/unreadable-member-handoff.log"
[[ ! -s "$calls" ]] || {
  echo "direct EXIT handoff reached shared resources with an unverifiable member" >&2
  cat "$calls" >&2
  exit 1
}

write_acceptance_run_owner \
  "$(acceptance_run_claim_path \
    "$local_claim_root" "test-project" "handoff-red" "sift")" \
  "test-project" "handoff-red" "sift" "$acquisition_id" \
  "$state_dir" "$evidence_dir" "$owner_pid" "$owner_pgid" \
  "old-owner-generation" "$handoff_digest"
: > "$calls"
set +e
PROCESS_START_TOKEN_HELPER="$member_token_helper" \
SIFT_UNREADABLE_MEMBER_PID="$test_member_pid" \
SIFT_REAL_PROCESS_TOKEN_HELPER="$ACCEPTANCE_ROOT/scripts/process-start-token.py" \
  run_cleanup >"$test_root/unreadable-member.log" 2>&1
unreadable_member_status=$?
set -e
[[ "$unreadable_member_status" -ne 0 ]] || {
  echo "cleanup accepted an unverifiable recorded group member" >&2
  exit 1
}
rg -q 'acceptance process-group scan is incomplete' \
  "$test_root/unreadable-member.log"
[[ ! -s "$calls" ]] || {
  echo "an unverifiable group member reached shared resources" >&2
  cat "$calls" >&2
  exit 1
}
kill "$test_member_pid" >/dev/null 2>&1 || true
wait "$test_member_pid" >/dev/null 2>&1 || true
test_member_pid=""
rm -f "$state_dir/process-scan-unsafe.txt"

# A watchdog scan failure is a durable stop condition. Recovery cannot delete
# shared resources until an operator can prove that the process tree is gone.
printf '%s\n' "injected incomplete scan" > "$state_dir/process-scan-unsafe.txt"
: > "$calls"
set +e
run_cleanup >"$test_root/incomplete-scan.log" 2>&1
incomplete_scan_status=$?
set -e
[[ "$incomplete_scan_status" -ne 0 ]]
rg -q 'acceptance process-group scan is incomplete' \
  "$test_root/incomplete-scan.log"
[[ ! -s "$calls" ]]
rm -f "$state_dir/process-scan-unsafe.txt"

# Once the watchdog reports an initial complete scan, its durable member list
# is mandatory. Losing that list is unsafe and must also stop recovery.
rm -f "$state_dir/watchdog-descendants.txt"
: > "$calls"
set +e
run_cleanup >"$test_root/missing-group-record.log" 2>&1
missing_group_record_status=$?
set -e
[[ "$missing_group_record_status" -ne 0 ]]
rg -q 'completed process-group record is missing' \
  "$test_root/missing-group-record.log"
[[ ! -s "$calls" ]]

echo "acceptance run-to-cleanup handoff E2E: ok"
