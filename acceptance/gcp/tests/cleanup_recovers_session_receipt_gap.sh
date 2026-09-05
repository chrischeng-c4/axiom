#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ACCEPTANCE_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
source "$ACCEPTANCE_ROOT/scripts/source-prefix.sh"
source "$ACCEPTANCE_ROOT/scripts/acceptance-lock.sh"
source "$ACCEPTANCE_ROOT/scripts/process-tree.sh"

test_root="$(mktemp -d "${TMPDIR:-/tmp}/sift-cleanup-session-gap.XXXXXX")"
fake_bin="$test_root/bin"
dedicated_pid=""

cleanup_test() {
  [[ -z "$dedicated_pid" ]] \
    || kill -KILL "$dedicated_pid" >/dev/null 2>&1 || true
  find "$test_root" -type f -delete
  find "$test_root" -depth -type d -empty -delete
}
trap cleanup_test EXIT INT TERM

mkdir -p "$fake_bin"

cat > "$fake_bin/gcloud" <<'EOF'
#!/usr/bin/env bash
set -euo pipefail
printf 'gcloud %s\n' "$*" >> "${SIFT_SESSION_GAP_CALLS:?}"
case " $* " in
  *" builds list "*" --format=json "*) printf '[]\n' ;;
  *" builds list "*) ;;
  *" artifacts docker images describe "*|*" container node-pools describe "*)
    echo "not found" >&2
    exit 1
    ;;
  *" artifacts docker images list "*) printf '[]\n' ;;
  *" storage ls "*|*" storage rm "*)
    echo "matched no URLs" >&2
    exit 1
    ;;
  *" storage buckets list "*|*" iam service-accounts list "*|*" compute disks list "*) ;;
esac
EOF

cat > "$fake_bin/kubectl" <<'EOF'
#!/usr/bin/env bash
set -euo pipefail
printf 'kubectl %s\n' "$*" >> "${SIFT_SESSION_GAP_CALLS:?}"
lock_state="${SIFT_SESSION_GAP_LOCK_STATE:?}"
cas_dir="${lock_state}.cas"

with_cas_lock() {
  local attempt
  for attempt in 1 2 3 4 5 6 7 8 9 10 11 12 13 14 15 16 17 18 19 20; do
    if mkdir "$cas_dir" 2>/dev/null; then
      return 0
    fi
    sleep 0.05
  done
  return 1
}

if [[ " $* " == *" patch lease axiom-gcp-operator-acceptance-lock "* ]]; then
  patch=""
  previous=""
  for argument in "$@"; do
    if [[ "$previous" == "-p" ]]; then
      patch="$argument"
      break
    fi
    previous="$argument"
  done
  [[ -n "$patch" ]] || exit 91
  expected_uid="$(jq -er '.[] | select(.op=="test" and .path=="/metadata/uid") | .value' <<<"$patch")"
  expected_rv="$(jq -er '.[] | select(.op=="test" and .path=="/metadata/resourceVersion") | .value' <<<"$patch")"
  expected_acquisition="$(jq -er '.[] | select(.op=="test" and .path=="/metadata/annotations/axiom.axiom.dev~1acquisition-id") | .value' <<<"$patch")"
  expected_session="$(jq -er '.[] | select(.op=="test" and .path=="/metadata/annotations/axiom.axiom.dev~1cleanup-session-id") | .value' <<<"$patch")"
  next_session="$(jq -er '.[] | select(.op=="replace" and .path=="/metadata/annotations/axiom.axiom.dev~1cleanup-session-id") | .value' <<<"$patch")"
  started_at="$(jq -er '.[] | select(.op=="replace" and .path=="/metadata/annotations/axiom.axiom.dev~1cleanup-started-at") | .value' <<<"$patch")"
  if [[ -n "${SIFT_SESSION_GAP_BARRIER_DIR:-}" ]]; then
    mkdir -p "$SIFT_SESSION_GAP_BARRIER_DIR"
    printf '%s\n' "$expected_rv" \
      > "$SIFT_SESSION_GAP_BARRIER_DIR/arrival.$$"
    printf 'patch-arrived %s %s\n' "$expected_rv" "$$" \
      >> "${SIFT_SESSION_GAP_CALLS:?}"
    barrier_ready=0
    for barrier_attempt in 1 2 3 4 5 6 7 8 9 10 11 12 13 14 15 16 17 18 19 20; do
      arrival_count=0
      for arrival in "$SIFT_SESSION_GAP_BARRIER_DIR"/arrival.*; do
        [[ -e "$arrival" ]] && arrival_count=$((arrival_count + 1))
      done
      if [[ "$arrival_count" -ge 2 ]]; then
        barrier_ready=1
        break
      fi
      sleep 0.05
    done
    [[ "$barrier_ready" == "1" ]] || exit 93
  fi
  with_cas_lock || exit 92
  trap 'rmdir "$cas_dir" 2>/dev/null || true' EXIT
  [[ -f "$lock_state" ]] || exit 1
  current="$(<"$lock_state")"
  jq -e \
    --arg uid "$expected_uid" \
    --arg rv "$expected_rv" \
    --arg acquisition "$expected_acquisition" \
    --arg session "$expected_session" '
      .metadata.uid == $uid
      and .metadata.resourceVersion == $rv
      and .metadata.annotations["axiom.axiom.dev/acquisition-id"] == $acquisition
      and .metadata.annotations["axiom.axiom.dev/cleanup-session-id"] == $session
    ' >/dev/null <<<"$current" || exit 1
  next_rv="$((10#$expected_rv + 1))"
  updated="$(jq \
    --arg rv "$next_rv" \
    --arg session "$next_session" \
    --arg started_at "$started_at" '
      .metadata.resourceVersion = $rv
      | .metadata.annotations["axiom.axiom.dev/cleanup-session-id"] = $session
      | .metadata.annotations["axiom.axiom.dev/cleanup-started-at"] = $started_at
    ' <<<"$current")"
  temporary="${lock_state}.$$"
  printf '%s\n' "$updated" > "$temporary"
  mv "$temporary" "$lock_state"
  printf '%s\n' "$updated" > "${SIFT_SESSION_GAP_WINNING_RESOURCE:?}"
  printf 'patch-won %s\n' "$next_session" >> "${SIFT_SESSION_GAP_CALLS:?}"
  printf '%s\n' "$updated"
  exit 0
fi

if [[ " $* " == *" get lease axiom-gcp-operator-acceptance-lock --namespace kube-system -o json "* ]]; then
  [[ -f "$lock_state" ]] || {
    echo "not found" >&2
    exit 1
  }
  cat "$lock_state"
  exit 0
fi

if [[ " $* " == *" delete --raw=/apis/coordination.k8s.io/v1/namespaces/kube-system/leases/axiom-gcp-operator-acceptance-lock -f - "* ]]; then
  options="$(cat)"
  with_cas_lock || exit 92
  trap 'rmdir "$cas_dir" 2>/dev/null || true' EXIT
  [[ -f "$lock_state" ]] || exit 1
  expected_uid="$(jq -r '.metadata.uid' "$lock_state")"
  expected_rv="$(jq -r '.metadata.resourceVersion' "$lock_state")"
  jq -e --arg uid "$expected_uid" --arg rv "$expected_rv" '
    .preconditions.uid == $uid and .preconditions.resourceVersion == $rv
  ' >/dev/null <<<"$options"
  rm -f "$lock_state"
  printf 'delete-won\n' >> "${SIFT_SESSION_GAP_CALLS:?}"
  exit 0
fi

if [[ " $* " == *" wait --for=delete lease/axiom-gcp-operator-acceptance-lock "* ]]; then
  [[ ! -f "$lock_state" ]]
  exit
fi

echo "not found" >&2
exit 1
EOF

cat > "$fake_bin/terraform" <<'EOF'
#!/usr/bin/env bash
set -euo pipefail
printf 'terraform %s\n' "$*" >> "${SIFT_SESSION_GAP_CALLS:?}"
exit 0
EOF

chmod +x "$fake_bin/gcloud" "$fake_bin/kubectl" "$fake_bin/terraform"

write_fixture() {
  local fixture_root="$1"
  local run_id="$2"
  local acquisition_id="$3"
  local session_id="$4"
  local cleanup_owner_pid="$5"
  local cleanup_owner_start_token="$6"
  local write_old_receipt="$7"
  local state_dir="$fixture_root/state"
  local evidence_dir="$fixture_root/evidence"
  local claim_root="$fixture_root/claims"
  local source_prefix="gs://test-source/source/axiom-gcp-operator-${run_id}"
  local session_resource intent_root intent_path

  mkdir -p "$state_dir" "$evidence_dir"
  session_resource="$(
    acceptance_lock_manifest "test-project" "$run_id" "sift" "$acquisition_id" \
      | jq --arg uid "${run_id}-lock-uid" --arg session "$session_id" '
          .metadata.uid=$uid
          | .metadata.resourceVersion="31"
          | .metadata.annotations["axiom.axiom.dev/cleanup-session-id"]=$session
          | .metadata.annotations["axiom.axiom.dev/cleanup-started-at"]="2026-09-02T00:00:00Z"
        '
  )"
  printf '%s\n' "$session_resource" > "$fixture_root/acceptance-lock.json"
  write_acceptance_lock_receipt \
    "$evidence_dir/acceptance-lock.json" "$session_resource" \
    "test-project" "$run_id" "sift" "$acquisition_id"
  intent_root="$evidence_dir/acceptance-cleanup-session-intents"
  intent_path="$(acceptance_cleanup_session_intent_path "$intent_root" "$session_id")"
  write_acceptance_cleanup_session_intent \
    "$intent_path" "test-project" "$run_id" "sift" "$acquisition_id" \
    "$session_id" "$cleanup_owner_pid" "$cleanup_owner_start_token"
  if [[ "$write_old_receipt" == "1" ]]; then
    write_acceptance_cleanup_session_receipt \
      "$evidence_dir/acceptance-cleanup-session.json" "$session_resource" \
      "test-project" "$run_id" "sift" "$acquisition_id" "$session_id"
  fi
  write_acceptance_run_owner \
    "$(acceptance_run_claim_path "$claim_root" "test-project" "$run_id" "sift")" \
    "test-project" "$run_id" "sift" "$acquisition_id" \
    "$state_dir" "$evidence_dir" "99999999" "99999999" "test-dead-owner" \
    "dddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddd"
  write_source_prefix_receipt \
    "$evidence_dir/source-prefix.json" "test-project" "$run_id" "$source_prefix"
  printf '[]\n' > "$evidence_dir/preexisting-sift-images.json"
  printf '[]\n' > "$evidence_dir/preexisting-rig-images.json"
  printf '[]\n' > "$evidence_dir/preexisting-sift-acceptance-runner-images.json"
}

run_cleanup() {
  local fixture_root="$1"
  local run_id="$2"
  local calls="$3"
  local barrier_dir="${4:-}"
  local state_dir="$fixture_root/state"
  local evidence_dir="$fixture_root/evidence"
  local source_prefix="gs://test-source/source/axiom-gcp-operator-${run_id}"

  PATH="$fake_bin:$PATH" \
  SIFT_SESSION_GAP_CALLS="$calls" \
  SIFT_SESSION_GAP_LOCK_STATE="$fixture_root/acceptance-lock.json" \
  SIFT_SESSION_GAP_WINNING_RESOURCE="$fixture_root/winning-resource.json" \
  SIFT_SESSION_GAP_BARRIER_DIR="$barrier_dir" \
  ACCEPTANCE_LOCAL_CLAIM_ROOT="$fixture_root/claims" \
  PROJECT_ID="test-project" \
  REGION="asia-east1" \
  GKE_ZONE="asia-east1-a" \
  RUN_ID="$run_id" \
  STATE_DIR="$state_dir" \
  ACCEPTANCE_ROOT="$ACCEPTANCE_ROOT" \
  REGISTRY="example.invalid" \
  IMAGE_TAG="candidate-${run_id}" \
  GCS_SOURCE_PREFIX="$source_prefix" \
  EVIDENCE_DIR="$evidence_dir" \
  ACCEPTANCE_APPS="sift" \
  bash "$ACCEPTANCE_ROOT/scripts/cleanup.sh"
}

assert_dead_owner_takeover() {
  local fixture_root="$1"
  local run_id="$2"
  local acquisition_id="$3"
  local old_session="$4"
  local calls="$fixture_root/calls.log"
  local new_session

  : > "$calls"
  run_cleanup "$fixture_root" "$run_id" "$calls" \
    >"$fixture_root/cleanup.log" 2>&1
  rg -q 'took over the cleanup session after its recorded owner stopped' \
    "$fixture_root/cleanup.log"
  [[ ! -e "$fixture_root/acceptance-lock.json" \
    && -f "$fixture_root/evidence/acceptance-lock-release.json" \
    && -f "$fixture_root/winning-resource.json" ]]
  new_session="$(jq -er '.cleanup_session_id' \
    "$fixture_root/evidence/acceptance-cleanup-session.json")"
  [[ "$new_session" =~ ^[0-9a-f]{32}$ && "$new_session" != "$old_session" ]]
  verify_acceptance_cleanup_session_receipt \
    "$fixture_root/evidence/acceptance-cleanup-session.json" \
    "$(<"$fixture_root/winning-resource.json")" \
    "test-project" "$run_id" "sift" "$acquisition_id" "$new_session"
  [[ "$(rg -c '^patch-won ' "$calls")" == "1" ]]

  # Once the release receipt is valid, missing local terminal evidence must
  # fail without reacquiring the Lease or repeating any deletion.
  mv "$fixture_root/evidence/cleanup.json" \
    "$fixture_root/evidence/cleanup.saved.json"
  : > "$calls"
  set +e
  run_cleanup "$fixture_root" "$run_id" "$calls" \
    >"$fixture_root/cleanup-missing-evidence.log" 2>&1
  missing_evidence_status=$?
  set -e
  [[ "$missing_evidence_status" -ne 0 ]]
  rg -q 'terminal cleanup evidence is missing or invalid after Lease release' \
    "$fixture_root/cleanup-missing-evidence.log"
  if rg -q '^(gcloud |terraform |patch-won|delete-won|kubectl (create|patch|delete))' \
      "$calls"; then
    echo "missing terminal evidence replayed a destructive operation" >&2
    cat "$calls" >&2
    exit 1
  fi
  mv "$fixture_root/evidence/cleanup.saved.json" \
    "$fixture_root/evidence/cleanup.json"

  # A valid terminal release is idempotent. A later recovery may finalize
  # local evidence, but it must not recreate the Lease or repeat deletion.
  : > "$calls"
  run_cleanup "$fixture_root" "$run_id" "$calls" \
    >"$fixture_root/cleanup-replay.log" 2>&1
  rg -q 'already released after verified cleanup' \
    "$fixture_root/cleanup-replay.log"
  if rg -q '^(gcloud |terraform |patch-won|delete-won|kubectl (create|patch|delete))' \
      "$calls"; then
    echo "terminal cleanup replay repeated a destructive operation" >&2
    cat "$calls" >&2
    exit 1
  fi
}

# A hard crash can leave either excluded helper alive after the owner dies.
# Recovery must check both dedicated generation records before it mutates the
# Lease or any cloud resource.
dedicated_root="$test_root/dedicated"
dedicated_acquisition="77777777777777777777777777777777"
dedicated_session="88888888888888888888888888888888"
write_fixture "$dedicated_root" "dedicated-live" "$dedicated_acquisition" \
  "$dedicated_session" "99999998" "test-dead-cleanup" "1"
for dedicated_record_name in watchdog-process.txt run-log-process.txt; do
  dedicated_calls="$dedicated_root/${dedicated_record_name}.calls.log"
  : > "$dedicated_calls"
  sleep 30 &
  dedicated_pid="$!"
  dedicated_token="$(process_start_token "$dedicated_pid")"
  printf '%s\t%s\n' "$dedicated_pid" "$dedicated_token" \
    > "$dedicated_root/state/$dedicated_record_name"
  set +e
  run_cleanup "$dedicated_root" "dedicated-live" "$dedicated_calls" \
    >"$dedicated_root/${dedicated_record_name}.cleanup.log" 2>&1
  dedicated_status=$?
  set -e
  [[ "$dedicated_status" -ne 0 ]] || {
    echo "recovery ignored live $dedicated_record_name" >&2
    exit 1
  }
  rg -q 'a dedicated acceptance process is still active or unverifiable' \
    "$dedicated_root/${dedicated_record_name}.cleanup.log"
  if rg -q '^(patch-won|delete-won|gcloud |terraform |kubectl (create|patch|delete))' \
      "$dedicated_calls"; then
    echo "recovery changed state while $dedicated_record_name was live" >&2
    cat "$dedicated_calls" >&2
    exit 1
  fi
  stop_process_generation_bounded "$dedicated_pid" "$dedicated_token"
  dedicated_pid=""
  rm -f "$dedicated_root/state/$dedicated_record_name"
done

# A missing local receipt after a successful Lease patch is recoverable.
gap_root="$test_root/gap"
gap_acquisition="aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
gap_session="bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"
write_fixture "$gap_root" "session-gap" "$gap_acquisition" "$gap_session" \
  "99999998" "test-dead-cleanup" "0"
assert_dead_owner_takeover \
  "$gap_root" "session-gap" "$gap_acquisition" "$gap_session"

# A valid old receipt is not a permanent fence after its exact owner stops.
receipt_root="$test_root/receipt"
receipt_acquisition="cccccccccccccccccccccccccccccccc"
receipt_session="eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee"
write_fixture "$receipt_root" "receipt-dead" "$receipt_acquisition" \
  "$receipt_session" "99999998" "test-dead-cleanup" "1"
assert_dead_owner_takeover \
  "$receipt_root" "receipt-dead" "$receipt_acquisition" "$receipt_session"

# A live owner keeps the session fenced, even when its receipt is valid.
active_root="$test_root/active"
active_acquisition="11111111111111111111111111111111"
active_session="22222222222222222222222222222222"
active_start="$(process_start_token "$$")"
[[ -n "$active_start" ]]
write_fixture "$active_root" "active-session" "$active_acquisition" \
  "$active_session" "$$" "$active_start" "1"
active_calls="$active_root/calls.log"
: > "$active_calls"
set +e
run_cleanup "$active_root" "active-session" "$active_calls" \
  >"$active_root/cleanup.log" 2>&1
active_status=$?
set -e
[[ "$active_status" -ne 0 ]]
rg -q 'the recorded cleanup session process is still active' \
  "$active_root/cleanup.log"
if rg -q '^(patch-won|delete-won|gcloud |terraform )' "$active_calls"; then
  echo "active cleanup-session recovery changed shared state" >&2
  cat "$active_calls" >&2
  exit 1
fi

# Two dead-owner recoveries race on one resourceVersion. Exactly one wins.
race_root="$test_root/race"
race_acquisition="33333333333333333333333333333333"
race_session="44444444444444444444444444444444"
write_fixture "$race_root" "session-race" "$race_acquisition" \
  "$race_session" "99999998" "test-dead-cleanup" "1"
race_calls_one="$race_root/calls-one.log"
race_calls_two="$race_root/calls-two.log"
race_barrier="$race_root/patch-barrier"
: > "$race_calls_one"
: > "$race_calls_two"
mkdir -p "$race_barrier"
set +e
run_cleanup "$race_root" "session-race" "$race_calls_one" "$race_barrier" \
  >"$race_root/cleanup-one.log" 2>&1 &
first_pid=$!
run_cleanup "$race_root" "session-race" "$race_calls_two" "$race_barrier" \
  >"$race_root/cleanup-two.log" 2>&1 &
second_pid=$!
wait "$first_pid"
first_status=$?
wait "$second_pid"
second_status=$?
set -e
[[ "$((first_status + second_status))" == "1" ]] || {
  echo "concurrent cleanup takeover did not produce one winner and one loser" >&2
  cat "$race_root/cleanup-one.log" "$race_root/cleanup-two.log" >&2
  cat "$race_calls_one" "$race_calls_two" >&2
  exit 1
}
patch_arrival_count="$(awk '/^patch-arrived 31 / { count += 1 } END { print count + 0 }' \
  "$race_calls_one" "$race_calls_two")"
patch_win_count="$(awk '/^patch-won / { count += 1 } END { print count + 0 }' \
  "$race_calls_one" "$race_calls_two")"
delete_win_count="$(awk '/^delete-won$/ { count += 1 } END { print count + 0 }' \
  "$race_calls_one" "$race_calls_two")"
[[ "$patch_arrival_count" == "2" \
  && "$patch_win_count" == "1" \
  && "$delete_win_count" == "1" ]] || {
  echo "concurrent cleanup takeover mutated the Lease more than once" >&2
  cat "$race_calls_one" "$race_calls_two" >&2
  exit 1
}
if [[ "$first_status" == "0" ]]; then
  loser_calls="$race_calls_two"
else
  loser_calls="$race_calls_one"
fi
if rg -q '^(gcloud |terraform |delete-won$)' "$loser_calls"; then
  echo "the losing cleanup contender reached a destructive operation" >&2
  cat "$loser_calls" >&2
  exit 1
fi

echo "cleanup session takeover and CAS recovery E2E: ok"
