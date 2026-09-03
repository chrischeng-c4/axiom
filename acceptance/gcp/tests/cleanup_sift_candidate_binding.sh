#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ACCEPTANCE_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
source "$ACCEPTANCE_ROOT/scripts/acceptance-lock.sh"
source "$ACCEPTANCE_ROOT/scripts/sift-candidate.sh"

test_root="$(mktemp -d "${TMPDIR:-/tmp}/sift-main-candidate-cleanup.XXXXXX")"
candidate_dir="$test_root/candidate"
evidence_dir="$test_root/evidence"
state_dir="$test_root/state"
fake_bin="$test_root/bin"
local_claim_root="$test_root/claims"
calls="$test_root/calls.log"
lock_state="$test_root/acceptance-lock.json"

cleanup_test() {
  if [[ "${SIFT_KEEP_TEST_TMP:-0}" == "1" ]]; then
    echo "preserved test root: $test_root" >&2
    return
  fi
  find "$test_root" -depth -delete >/dev/null 2>&1 || true
}
trap cleanup_test EXIT INT TERM

mkdir -p "$evidence_dir" "$state_dir" "$fake_bin"
SIFT_CANDIDATE_FIXTURE_OUT="$candidate_dir" \
  bash "$SCRIPT_DIR/sift_candidate_receipt.sh" >/dev/null
copy_sift_candidate_evidence "$candidate_dir" "$evidence_dir"

project_id="$(jq -er '.project_id' "$candidate_dir/candidate.json")"
region="$(jq -er '.region' "$candidate_dir/candidate.json")"
run_id="$(jq -er '.run_id' "$candidate_dir/candidate.json")"
registry="$(jq -er '.registry' "$candidate_dir/candidate.json")"
image_tag="$(jq -er '.image_tag' "$candidate_dir/candidate.json")"
source_prefix="$(jq -er '.source_prefix' "$candidate_dir/candidate.json")"
acquisition_id="$(jq -er '.acquisition_id' "$candidate_dir/candidate.json")"
build_id="$(jq -er '.cloud_build_id' "$candidate_dir/candidate.json")"

for image in sift rig sift-acceptance-runner; do
  if [[ "$image" == "sift-acceptance-runner" ]]; then
    digest_ref="$(jq -er '.acceptance_runner' "$candidate_dir/images.json")"
  else
    digest_ref="$(jq -er --arg image "$image" '.[$image]' \
      "$candidate_dir/images.json")"
  fi
  printf '%s\n' "${digest_ref##*@}" > "$state_dir/${image}.digest"
  : > "$state_dir/${image}.tag"
done
printf '101\n' > "$state_dir/source-reservation.generation"
printf '201\n' > "$state_dir/source-intent.generation"
: > "$calls"

cat > "$fake_bin/gcloud" <<'EOF'
#!/usr/bin/env bash
set -euo pipefail
printf 'gcloud %s\n' "$*" >> "${SIFT_MAIN_CLEANUP_CALLS:?}"
state="${SIFT_MAIN_CLEANUP_STATE:?}"
candidate="${SIFT_MAIN_CANDIDATE_DIR:?}"

image_from_ref() {
  case "$1" in
    */sift-acceptance-runner*) printf 'sift-acceptance-runner\n' ;;
    */sift*) printf 'sift\n' ;;
    */rig*) printf 'rig\n' ;;
    *) return 1 ;;
  esac
}

case " $* " in
  *" builds list "*)
    jq -n --arg id "${SIFT_MAIN_BUILD_ID:?}" \
      '[{id:$id,status:"SUCCESS"}]'
    ;;
  *" builds describe "*)
    if [[ " $* " == *" --format=value(status) "* ]]; then
      printf 'SUCCESS\n'
    else
      cat "$candidate/cloud-build-final.json"
    fi
    ;;
  *" storage cp "*)
    source_uri="${3:?}"
    destination="${4:?}"
    case "$source_uri" in
      */candidate-reservation.json)
        if [[ -f "$state/source-reservation-removed" ]]; then
          echo "NOT_FOUND" >&2
          exit 1
        fi
        if [[ -f "$state/source-reservation-replaced" ]]; then
          jq '.acquisition_id="22222222222222222222222222222222"' \
            "$candidate/candidate-reservation.json" > "$destination"
        else
          cp "$candidate/candidate-reservation.json" "$destination"
        fi
        ;;
      */candidate-submit-intent.json)
        if [[ -f "$state/source-intent-removed" ]]; then
          echo "NOT_FOUND" >&2
          exit 1
        fi
        cp "$candidate/candidate-submit-intent.json" "$destination"
        ;;
      *)
        echo "NOT_FOUND" >&2
        exit 1
        ;;
    esac
    ;;
  *" storage objects describe "*)
    source_uri="${4:?}"
    case "$source_uri" in
      */candidate-reservation.json)
        [[ ! -f "$state/source-reservation-removed" ]] || {
          echo "NOT_FOUND" >&2
          exit 1
        }
        generation="$(cat "$state/source-reservation.generation")"
        ;;
      */candidate-submit-intent.json)
        [[ ! -f "$state/source-intent-removed" ]] || {
          echo "NOT_FOUND" >&2
          exit 1
        }
        generation="$(cat "$state/source-intent.generation")"
        ;;
      *)
        echo "NOT_FOUND" >&2
        exit 1
        ;;
    esac
    jq -n --arg generation "$generation" '{generation:$generation}'
    ;;
  *" artifacts docker images describe "*)
    ref="${5:?}"
    image="$(image_from_ref "$ref")"
    if [[ "$ref" == *@* && -f "$state/${image}.digest" ]]; then
      cat "$state/${image}.digest"
    elif [[ "$ref" != *@* && -f "$state/${image}.tag" ]]; then
      cat "$state/${image}.digest"
    else
      echo "NOT_FOUND" >&2
      exit 1
    fi
    ;;
  *" artifacts docker images list "*)
    image="$(image_from_ref "${5:?}")"
    if [[ -f "$state/${image}.digest" ]]; then
      digest="$(cat "$state/${image}.digest")"
      if [[ -f "$state/${image}.tag" && -f "$state/${image}.shared-tag" ]]; then
        jq -n --arg digest "$digest" --arg tag "${SIFT_MAIN_IMAGE_TAG:?}" \
          '[{version:("image@" + $digest),tags:[$tag,"shared-run"]}]'
      elif [[ -f "$state/${image}.tag" ]]; then
        jq -n --arg digest "$digest" --arg tag "${SIFT_MAIN_IMAGE_TAG:?}" \
          '[{version:("image@" + $digest),tags:[$tag]}]'
      elif [[ -f "$state/${image}.shared-tag" ]]; then
        jq -n --arg digest "$digest" \
          '[{version:("image@" + $digest),tags:["shared-run"]}]'
      else
        jq -n --arg digest "$digest" \
          '[{version:("image@" + $digest),tags:[]}]'
      fi
    else
      printf '[]\n'
    fi
    ;;
  *" artifacts docker tags delete "*)
    ref="${5:?}"
    image="$(image_from_ref "$ref")"
    if [[ -f "$state/${image}.tag" ]]; then
      rm -f "$state/${image}.tag"
    else
      echo "NOT_FOUND" >&2
      exit 1
    fi
    ;;
  *" artifacts docker images delete "*)
    ref="${5:?}"
    image="$(image_from_ref "$ref")"
    if [[ "${SIFT_MAIN_FAIL_SIFT_DELETE:-0}" == "1" \
      && "$image" == "sift" ]]; then
      echo "injected immutable digest delete failure" >&2
      exit 51
    fi
    if [[ "${SIFT_MAIN_ATTACH_SHARED_TAG_ON_DELETE:-0}" == "1" \
      && "$image" == "sift" \
      && ! -f "$state/sift-shared-attached-once" ]]; then
      : > "$state/sift-shared-attached-once"
      : > "$state/sift.shared-tag"
      echo "FAILED_PRECONDITION: image still has tags" >&2
      exit 1
    fi
    if [[ -f "$state/${image}.shared-tag" ]]; then
      echo "FAILED_PRECONDITION: image still has tags" >&2
      exit 1
    fi
    if [[ -f "$state/${image}.digest" ]]; then
      rm -f "$state/${image}.digest"
    else
      echo "NOT_FOUND" >&2
      exit 1
    fi
    ;;
  *" artifacts repositories describe "*)
    ;;
  *" storage rm "*)
    source_uri="${3:?}"
    generation_match=""
    for argument in "$@"; do
      case "$argument" in
        --if-generation-match=*) generation_match="${argument#*=}" ;;
      esac
    done
    case "$source_uri" in
      */source.tgz) marker="$state/source-object-removed" ;;
      */candidate-submit-intent.json)
        marker="$state/source-intent-removed"
        expected_generation="$(cat "$state/source-intent.generation")"
        ;;
      */candidate-reservation.json)
        if [[ "${SIFT_MAIN_FAIL_RESERVATION_DELETE_ONCE:-0}" == "1" \
          && ! -f "$state/source-reservation-delete-failed-once" ]]; then
          : > "$state/source-reservation-delete-failed-once"
          echo "injected reservation delete failure" >&2
          exit 73
        fi
        marker="$state/source-reservation-removed"
        expected_generation="$(cat "$state/source-reservation.generation")"
        if [[ "${SIFT_MAIN_REPLACE_RESERVATION_BEFORE_DELETE:-0}" == "1" \
          && ! -f "$state/source-reservation-replaced" ]]; then
          : > "$state/source-reservation-replaced"
          expected_generation=$((expected_generation + 1))
          printf '%s\n' "$expected_generation" \
            > "$state/source-reservation.generation"
        fi
        ;;
      *)
        echo "refusing unexpected source delete: $source_uri" >&2
        exit 91
        ;;
    esac
    if [[ "$source_uri" == */candidate-*.json ]]; then
      if [[ -z "$generation_match" || "$generation_match" != "$expected_generation" ]]; then
        echo "PRECONDITION_FAILED: generation changed" >&2
        exit 1
      fi
    fi
    [[ ! -f "$marker" ]] || {
      echo "NOT_FOUND" >&2
      exit 1
    }
    : > "$marker"
    if [[ -f "$state/source-object-removed" \
      && -f "$state/source-intent-removed" \
      && -f "$state/source-reservation-removed" ]]; then
      : > "$state/source-removed"
    fi
    ;;
  *" storage ls --recursive "*)
    if [[ -f "$state/source-removed" ]]; then
      if [[ "${SIFT_MAIN_FAIL_SOURCE_LIST_ONCE:-0}" == "1" \
        && ! -f "$state/source-list-failed-once" ]]; then
        : > "$state/source-list-failed-once"
        echo "injected source inventory failure" >&2
        exit 72
      fi
      echo "matched no URLs" >&2
      exit 1
    fi
    [[ -f "$state/source-object-removed" ]] \
      || printf '%s/source.tgz\n' "${SIFT_MAIN_SOURCE_PREFIX:?}"
    [[ -f "$state/source-intent-removed" ]] \
      || printf '%s/candidate-submit-intent.json\n' \
        "${SIFT_MAIN_SOURCE_PREFIX:?}"
    [[ -f "$state/source-reservation-removed" ]] \
      || printf '%s/candidate-reservation.json\n' \
        "${SIFT_MAIN_SOURCE_PREFIX:?}"
    ;;
  *" storage buckets list "*|*" iam service-accounts list "*|*" compute disks list "*|*" services list "*)
    ;;
  *" container node-pools describe "*)
    echo "NOT_FOUND" >&2
    exit 1
    ;;
  *)
    echo "unexpected fake gcloud call: $*" >&2
    exit 90
    ;;
esac
EOF

cat > "$fake_bin/kubectl" <<'EOF'
#!/usr/bin/env bash
set -euo pipefail
if [[ " $* " == *" patch lease axiom-gcp-operator-acceptance-lock "* ]]; then
  while [[ "$#" -gt 0 && "$1" != "-p" ]]; do shift; done
  shift
  patch="${1:?}"
  session_id="$(jq -er '.[] | select(.path | endswith("cleanup-session-id")) | .value' \
    <<<"$patch")"
  started_at="$(jq -er '.[] | select(.path | endswith("cleanup-started-at")) | .value' \
    <<<"$patch")"
  jq --arg session_id "$session_id" --arg started_at "$started_at" '
    .metadata.annotations["axiom.axiom.dev/cleanup-session-id"] = $session_id
    | .metadata.annotations["axiom.axiom.dev/cleanup-started-at"] = $started_at
    | .metadata.resourceVersion = (((.metadata.resourceVersion | tonumber) + 1) | tostring)
  ' "${SIFT_MAIN_LOCK_STATE:?}" > "${SIFT_MAIN_LOCK_STATE}.tmp"
  mv "${SIFT_MAIN_LOCK_STATE}.tmp" "${SIFT_MAIN_LOCK_STATE}"
  cat "${SIFT_MAIN_LOCK_STATE}"
  exit 0
fi
if [[ " $* " == *" get lease axiom-gcp-operator-acceptance-lock --namespace kube-system -o json "* ]]; then
  if [[ -f "${SIFT_MAIN_LOCK_STATE:?}" ]]; then
    cat "$SIFT_MAIN_LOCK_STATE"
    exit 0
  fi
  echo "NotFound" >&2
  exit 1
fi
if [[ "${1:-}" == "delete" && "${2:-}" == --raw=* ]]; then
  find "${SIFT_MAIN_LOCK_STATE:?}" -delete
  printf '{}\n'
  exit 0
fi
echo "NotFound" >&2
exit 1
EOF

cat > "$fake_bin/ps" <<'EOF'
#!/usr/bin/env bash
set -euo pipefail
case "${1:-}" in
  --status) printf 'running\n' ;;
  --snapshot) printf '[]\n' ;;
  *) printf 'test-start-%s\n' "${1:?}" ;;
esac
EOF
chmod +x "$fake_bin/gcloud" "$fake_bin/kubectl" "$fake_bin/ps"

restore_images() {
  local image digest_ref
  for image in sift rig sift-acceptance-runner; do
    if [[ "$image" == "sift-acceptance-runner" ]]; then
      digest_ref="$(jq -er '.acceptance_runner' "$candidate_dir/images.json")"
    else
      digest_ref="$(jq -er --arg image "$image" '.[$image]' \
        "$candidate_dir/images.json")"
    fi
    printf '%s\n' "${digest_ref##*@}" > "$state_dir/${image}.digest"
    : > "$state_dir/${image}.tag"
    rm -f "$state_dir/${image}.shared-tag" \
      "$evidence_dir/deleted-image-${image}.txt"
  done
  rm -f "$state_dir/sift-shared-attached-once"
}

restore_source_state() {
  rm -f "$state_dir/source-removed" \
    "$state_dir/source-object-removed" \
    "$state_dir/source-intent-removed" \
    "$state_dir/source-reservation-removed" \
    "$state_dir/source-list-failed-once" \
    "$state_dir/source-reservation-delete-failed-once" \
    "$state_dir/source-reservation-replaced"
  printf '101\n' > "$state_dir/source-reservation.generation"
  printf '201\n' > "$state_dir/source-intent.generation"
}

lock_resource="$({
  acceptance_lock_manifest "$project_id" "$run_id" "sift" \
    "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
} | jq '.metadata.uid="owned-lock-uid" | .metadata.resourceVersion="30"')"

reset_cleanup_lock() {
  printf '%s\n' "$lock_resource" > "$lock_state"
  write_acceptance_lock_receipt \
    "$evidence_dir/acceptance-lock.json" "$lock_resource" \
    "$project_id" "$run_id" "sift" "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
  find "$evidence_dir/acceptance-cleanup-session.json" \
    "$evidence_dir/acceptance-cleanup-session-intent.json" \
    "$evidence_dir/cleanup.json" -delete >/dev/null 2>&1 || true
}

write_acceptance_run_owner \
  "$(acceptance_run_claim_path \
    "$local_claim_root" "$project_id" "$run_id" "sift")" \
  "$project_id" "$run_id" "sift" "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa" \
  "$state_dir" "$evidence_dir" "99999999" "99999999" "test-dead-owner" \
  "dddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddd"
reset_cleanup_lock

run_cleanup() {
  PATH="$fake_bin:$PATH" \
    PROCESS_START_TOKEN_HELPER="$fake_bin/ps" \
    SIFT_MAIN_CLEANUP_CALLS="$calls" \
    SIFT_MAIN_CLEANUP_STATE="$state_dir" \
    SIFT_MAIN_LOCK_STATE="$lock_state" \
    SIFT_MAIN_CANDIDATE_DIR="$candidate_dir" \
    SIFT_MAIN_BUILD_ID="$build_id" \
    SIFT_MAIN_IMAGE_TAG="$image_tag" \
    SIFT_MAIN_SOURCE_PREFIX="$source_prefix" \
    SIFT_CANDIDATE_DIR="$candidate_dir" \
    ACCEPTANCE_LOCAL_CLAIM_ROOT="$local_claim_root" \
    PROJECT_ID="$project_id" REGION="$region" GKE_ZONE="asia-east1-a" \
    RUN_ID="$run_id" STATE_DIR="$state_dir" ACCEPTANCE_ROOT="$ACCEPTANCE_ROOT" \
    REGISTRY="$registry" IMAGE_TAG="$image_tag" \
    GCS_SOURCE_PREFIX="$source_prefix" EVIDENCE_DIR="$evidence_dir" \
    ACCEPTANCE_APPS="sift" \
    bash "$ACCEPTANCE_ROOT/scripts/cleanup.sh"
}

first_status=0
SIFT_MAIN_FAIL_SIFT_DELETE=1 run_cleanup \
  > "$test_root/first.log" 2>&1 || first_status=$?
[[ "$first_status" != "0" && ! -e "$state_dir/source-removed" \
  && -e "$state_dir/sift.digest" ]] || {
  echo "main cleanup did not retain source after an image deletion failure" >&2
  cat "$test_root/first.log" >&2
  exit 1
}
if rg -F 'gcloud storage rm ' "$calls" >/dev/null; then
  echo "main cleanup deleted source after an image deletion failure" >&2
  exit 1
fi
rg -F 'gcloud artifacts docker images delete ' "$calls" >/dev/null || {
  echo "main cleanup did not reach the injected image deletion failure" >&2
  exit 1
}
rg -F 'gcloud artifacts docker tags delete ' "$calls" >/dev/null || {
  echo "main cleanup did not remove the exact run tag before digest cleanup" >&2
  exit 1
}

reset_cleanup_lock
: > "$calls"
run_cleanup > "$test_root/second.log" 2>&1
[[ -e "$state_dir/source-removed" ]] || {
  echo "main cleanup retry did not delete its verified source prefix" >&2
  cat "$test_root/second.log" >&2
  exit 1
}
for image in sift rig sift-acceptance-runner; do
  [[ ! -e "$state_dir/${image}.digest" \
    && ! -e "$state_dir/${image}.tag" ]] || {
    echo "main cleanup retry left an image: $image" >&2
    exit 1
  }
done
jq -e '.status == "clean"' "$evidence_dir/cleanup.json" >/dev/null
rg -F 'gcloud storage cp ' "$calls" >/dev/null
rg -F "gcloud builds describe $build_id " "$calls" >/dev/null
rg -F 'gcloud artifacts docker images delete ' "$calls" >/dev/null
rg -F "gcloud storage rm $source_prefix/source.tgz --quiet" "$calls" >/dev/null
rg -F "gcloud storage rm $source_prefix/candidate-submit-intent.json --if-generation-match=201 --quiet" \
  "$calls" >/dev/null
rg -F "gcloud storage rm $source_prefix/candidate-reservation.json --if-generation-match=101 --quiet" \
  "$calls" >/dev/null
if rg -F -- '--delete-tags' "$calls" >/dev/null; then
  echo "main cleanup used unsafe digest --delete-tags" >&2
  exit 1
fi
# Another run may attach a tag after this run's exact tag is removed. The
# cleanup must retain that shared digest and still finish its own cleanup.
restore_images
restore_source_state
reset_cleanup_lock
: > "$calls"
SIFT_MAIN_ATTACH_SHARED_TAG_ON_DELETE=1 run_cleanup \
  > "$test_root/shared-digest.log" 2>&1
[[ ! -f "$state_dir/sift.tag" \
  && -f "$state_dir/sift.digest" \
  && -f "$state_dir/sift.shared-tag" ]] || {
  echo "main cleanup deleted a digest after a concurrent tag attach" >&2
  cat "$test_root/shared-digest.log" >&2
  exit 1
}
[[ ! -f "$evidence_dir/deleted-image-sift.txt" ]] || {
  echo "main cleanup marked a retained shared digest as deleted" >&2
  exit 1
}
jq -e '.status == "clean"' "$evidence_dir/cleanup.json" >/dev/null
if rg -F -- '--delete-tags' "$calls" >/dev/null; then
  echo "main cleanup used unsafe digest --delete-tags during the shared race" >&2
  exit 1
fi

# If the final source inventory fails after the reservation was deleted last,
# an exact retry may only run final read checks. It must not repeat cleanup
# mutations without the live candidate controls.
restore_images
restore_source_state
reset_cleanup_lock
: > "$calls"
source_retry_status=0
SIFT_MAIN_FAIL_SOURCE_LIST_ONCE=1 run_cleanup \
  > "$test_root/source-retry-first.log" 2>&1 || source_retry_status=$?
[[ "$source_retry_status" != "0" \
  && -f "$state_dir/source-removed" \
  && ! -f "$evidence_dir/cleanup.json" ]] || {
  echo "main cleanup did not reach the injected final source check failure" >&2
  cat "$test_root/source-retry-first.log" >&2
  exit 1
}

reset_cleanup_lock
: > "$calls"
run_cleanup > "$test_root/source-retry-second.log" 2>&1
jq -e '.status == "clean"' "$evidence_dir/cleanup.json" >/dev/null
if rg -e '^gcloud (builds cancel|artifacts docker tags delete|artifacts docker images delete|storage rm) ' \
    "$calls" >/dev/null; then
  echo "source-absent retry performed a forbidden cleanup mutation" >&2
  cat "$calls" >&2
  exit 1
fi

# A failed final reservation delete leaves only that exact live control object.
# The retry may delete it after read-only checks. It must not repeat any other
# cleanup mutation.
restore_images
restore_source_state
reset_cleanup_lock
: > "$calls"
reservation_retry_status=0
SIFT_MAIN_FAIL_RESERVATION_DELETE_ONCE=1 run_cleanup \
  > "$test_root/reservation-retry-first.log" 2>&1 \
  || reservation_retry_status=$?
[[ "$reservation_retry_status" != "0" \
  && -f "$state_dir/source-object-removed" \
  && -f "$state_dir/source-intent-removed" \
  && ! -f "$state_dir/source-reservation-removed" \
  && ! -f "$state_dir/source-removed" \
  && ! -f "$evidence_dir/cleanup.json" ]] || {
  echo "main cleanup did not retain the last live reservation" >&2
  cat "$test_root/reservation-retry-first.log" >&2
  exit 1
}

reset_cleanup_lock
: > "$calls"
run_cleanup > "$test_root/reservation-retry-second.log" 2>&1
jq -e '.status == "clean"' "$evidence_dir/cleanup.json" >/dev/null
[[ -f "$state_dir/source-removed" ]]
if rg -e '^gcloud (builds cancel|artifacts docker tags delete|artifacts docker images delete) ' \
    "$calls" >/dev/null; then
  echo "reservation-only retry repeated a forbidden cleanup mutation" >&2
  cat "$calls" >&2
  exit 1
fi
[[ "$(rg -c '^gcloud storage rm ' "$calls")" == "1" ]] || {
  echo "reservation-only retry deleted more than its final reservation" >&2
  cat "$calls" >&2
  exit 1
}
rg -F "gcloud storage rm $source_prefix/candidate-reservation.json --if-generation-match=101 --quiet" \
  "$calls" >/dev/null

# A later acquisition may replace the same reservation URI after another
# cleanup removes the old object. The stale cleanup must fail its generation
# precondition and leave the replacement intact.
restore_images
restore_source_state
reset_cleanup_lock
: > "$calls"
generation_race_status=0
SIFT_MAIN_REPLACE_RESERVATION_BEFORE_DELETE=1 run_cleanup \
  > "$test_root/generation-race.log" 2>&1 || generation_race_status=$?
[[ "$generation_race_status" != "0" \
  && -f "$state_dir/source-reservation-replaced" \
  && ! -f "$state_dir/source-reservation-removed" \
  && ! -f "$evidence_dir/cleanup.json" ]] || {
  echo "main cleanup deleted a replacement reservation generation" >&2
  cat "$test_root/generation-race.log" >&2
  exit 1
}
[[ "$(cat "$state_dir/source-reservation.generation")" == "102" ]]
rg -F "gcloud storage rm $source_prefix/candidate-reservation.json --if-generation-match=101 --quiet" \
  "$calls" >/dev/null

echo "main Sift candidate cleanup binding E2E: ok"
