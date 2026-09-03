#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ACCEPTANCE_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
source "$ACCEPTANCE_ROOT/scripts/source-prefix.sh"
source "$ACCEPTANCE_ROOT/scripts/sift-candidate.sh"

test_root="$(mktemp -d "${TMPDIR:-/tmp}/sift-candidate-cleanup-e2e.XXXXXX")"
fake_bin="$test_root/bin"
recovery_dir="$test_root/candidate.failed"
state_dir="$test_root/state"
calls="$test_root/gcloud.log"
cleanup_test() {
  find "$test_root" -depth -delete >/dev/null 2>&1 || true
}
trap cleanup_test EXIT INT TERM
mkdir -p "$fake_bin" "$recovery_dir" "$state_dir"
: > "$calls"

project_id="axiom-test"
region="asia-east1"
run_id="cleanup-retry"
git_sha="0123456789abcdef0123456789abcdef01234567"
source_sha="aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
registry="asia-east1-docker.pkg.dev/axiom-test/courier"
source_prefix="gs://axiom-test_cloudbuild/source/axiom-gcp-operator-${run_id}"
acquisition_id="11111111111111111111111111111111"
image_tag="${git_sha}-${run_id}-${acquisition_id}"
reservation_uri="$source_prefix/candidate-reservation.json"

jq -n \
  --arg project_id "$project_id" --arg region "$region" \
  --arg run_id "$run_id" --arg git_sha "$git_sha" \
  --arg source_sha "$source_sha" --arg registry "$registry" \
  --arg image_tag "$image_tag" --arg source_prefix "$source_prefix" \
  --arg acquisition_id "$acquisition_id" \
  --arg reservation_uri "$reservation_uri" '
    {
      schema:"axiom.gcp.sift.candidate-preparation-failure.v2",
      project_id:$project_id,
      region:$region,
      artifact_registry_repository:"courier",
      run_id:$run_id,
      git_sha:$git_sha,
      source_bundle_sha256:$source_sha,
      acquisition_id:$acquisition_id,
      reservation_uri:$reservation_uri,
      cloud_build_id:"build-cleanup",
      registry:$registry,
      image_tag:$image_tag,
      source_prefix:$source_prefix,
      submit_intent_published:true,
      submit_started:true,
      submit_response_received:true,
      exit_code:1,
      failed_at:"2026-09-03T00:00:00Z"
    }
  ' > "$recovery_dir/candidate-preparation-failure.json"
write_source_prefix_receipt \
  "$recovery_dir/source-prefix.json" "$project_id" "$run_id" "$source_prefix"

for image in sift rig sift-acceptance-runner; do
  printf '[]\n' > "$recovery_dir/preexisting-${image}-images.json"
  digest="sha256:$(printf '%s' "$image" | shasum -a 256 | awk '{print $1}')"
  printf '%s\n' "$digest" > "$state_dir/${image}.digest"
  printf '%s\n' "$digest" > "$state_dir/${image}.tag"
done
printf '101\n' > "$state_dir/source-reservation.generation"
printf '201\n' > "$state_dir/source-intent.generation"

jq -n \
  --arg project_id "$project_id" --arg region "$region" \
  --arg run_id "$run_id" --arg git_sha "$git_sha" \
  --arg source_sha "$source_sha" --arg registry "$registry" \
  --arg image_tag "$image_tag" --arg source_prefix "$source_prefix" \
  --arg acquisition_id "$acquisition_id" \
  --arg reservation_uri "$reservation_uri" '
    {
      schema:"axiom.gcp.sift.candidate-reservation.v1",
      project_id:$project_id,
      region:$region,
      artifact_registry_repository:"courier",
      run_id:$run_id,
      git_sha:$git_sha,
      source_bundle_sha256:$source_sha,
      acquisition_id:$acquisition_id,
      registry:$registry,
      image_tag:$image_tag,
      source_prefix:$source_prefix,
      reservation_uri:$reservation_uri,
      created_at:"2026-09-03T00:00:00Z",
      preexisting_images:{sift:[],rig:[],sift_acceptance_runner:[]}
    }
  ' > "$recovery_dir/candidate-reservation.json"
jq -n \
  --arg project_id "$project_id" --arg region "$region" \
  --arg run_id "$run_id" --arg git_sha "$git_sha" \
  --arg source_sha "$source_sha" --arg registry "$registry" \
  --arg image_tag "$image_tag" --arg source_prefix "$source_prefix" \
  --arg acquisition_id "$acquisition_id" '
    {
      schema:"axiom.gcp.sift.candidate-submit-intent.v1",
      project_id:$project_id,
      region:$region,
      run_id:$run_id,
      git_sha:$git_sha,
      source_bundle_sha256:$source_sha,
      acquisition_id:$acquisition_id,
      registry:$registry,
      image_tag:$image_tag,
      source_prefix:$source_prefix,
      submitted_at:"2026-09-03T00:00:01Z"
    }
  ' > "$recovery_dir/candidate-submit-intent.json"

jq -n \
  --arg run_id "$run_id" --arg git_sha "$git_sha" \
  --arg source_sha "$source_sha" --arg registry "$registry" \
  --arg image_tag "$image_tag" \
  --arg acquisition_id "$acquisition_id" \
  --arg sift_digest "$(cat "$state_dir/sift.digest")" \
  --arg rig_digest "$(cat "$state_dir/rig.digest")" \
  --arg runner_digest "$(cat "$state_dir/sift-acceptance-runner.digest")" '
    {
      id:"build-cleanup",
      status:"SUCCESS",
      substitutions:{
        _RUN_ID:$run_id,
        _GIT_SHA:$git_sha,
        _SOURCE_BUNDLE_SHA256:$source_sha,
        _REGISTRY:$registry,
        _TAG:$image_tag,
        _CANDIDATE_ACQUISITION_ID:$acquisition_id
      },
      source:{storageSource:{
        bucket:"axiom-test_cloudbuild",
        object:("source/axiom-gcp-operator-" + $run_id + "/source.tgz")
      }},
      tags:[
        "sift-mvp",
        ("axiom-run-" + $run_id),
        ("axiom-source-" + $source_sha),
        ("axiom-acquisition-" + $acquisition_id)
      ],
      results:{images:[
        {name:($registry + "/sift:" + $image_tag),digest:$sift_digest},
        {name:($registry + "/rig:" + $image_tag),digest:$rig_digest},
        {name:($registry + "/sift-acceptance-runner:" + $image_tag),digest:$runner_digest}
      ]}
    }
  ' > "$state_dir/build.json"

cat > "$fake_bin/gcloud" <<'EOF'
#!/usr/bin/env bash
set -euo pipefail
printf '%s\n' "$*" >> "${SIFT_CANDIDATE_CLEANUP_CALLS:?}"
state="${SIFT_CANDIDATE_CLEANUP_STATE:?}"
image_name() {
  local reference="$1"
  reference="${reference%%:*}"
  reference="${reference%%@*}"
  printf '%s\n' "${reference##*/}"
}
case "${1:-} ${2:-} ${3:-} ${4:-}" in
  "builds list --project="*)
    list_count=0
    [[ ! -f "$state/build-list-count" ]] \
      || list_count="$(cat "$state/build-list-count")"
    list_count=$((list_count + 1))
    printf '%s\n' "$list_count" > "$state/build-list-count"
    if [[ "${SIFT_CANDIDATE_EXTRA_BUILD:-0}" == "1" \
      || ( "${SIFT_CANDIDATE_LATE_BUILD:-0}" == "1" && "$list_count" -ge 2 ) ]]; then
      printf '[{"id":"build-cleanup"},{"id":"build-extra"}]\n'
    else
      printf '[{"id":"build-cleanup"}]\n'
    fi
    ;;
  "builds describe build-cleanup --project="*)
    if [[ "${SIFT_CANDIDATE_RESULTS_EMPTY:-0}" == "1" ]]; then
      jq '.results.images = []' "$state/build.json"
    elif [[ "${SIFT_CANDIDATE_SOURCE_OUTSIDE:-0}" == "1" ]]; then
      jq '.source.storageSource.object = "source/another-run/source.tgz"' \
        "$state/build.json"
    else
      cat "$state/build.json"
    fi
    ;;
  "artifacts docker images describe")
    image="$(image_name "${5:?}")"
    if [[ -f "$state/${image}.tag" ]]; then
      if [[ "${SIFT_CANDIDATE_TAG_MISMATCH:-0}" == "1" && "$image" == "sift" ]]; then
        printf 'sha256:%064d\n' 9
      else
        cat "$state/${image}.tag"
      fi
    else
      echo "NOT_FOUND" >&2
      exit 1
    fi
    ;;
  "artifacts docker images list")
    image="$(image_name "${5:?}")"
    if [[ -f "$state/${image}.digest" ]]; then
      digest="$(cat "$state/${image}.digest")"
      if [[ -f "$state/${image}.tag" && -f "$state/${image}.shared-tag" ]]; then
        jq -n --arg digest "$digest" --arg tag "${SIFT_CANDIDATE_IMAGE_TAG:?}" \
          '[{version:("image@" + $digest),tags:[$tag,"shared-run"]}]'
      elif [[ -f "$state/${image}.tag" ]]; then
        jq -n --arg digest "$digest" --arg tag "${SIFT_CANDIDATE_IMAGE_TAG:?}" \
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
  "artifacts docker tags delete")
    image="$(image_name "${5:?}")"
    if [[ -f "$state/${image}.tag" ]]; then
      rm -f "$state/${image}.tag"
    else
      echo "NOT_FOUND" >&2
      exit 1
    fi
    ;;
  "artifacts docker images delete")
    image="$(image_name "${5:?}")"
    if [[ "$image" == "sift" && ! -e "$state/sift-delete-failed-once" ]]; then
      : > "$state/sift-delete-failed-once"
      echo "injected digest delete failure" >&2
      exit 61
    fi
    if [[ "${SIFT_CANDIDATE_ATTACH_SHARED_TAG_ON_DELETE:-0}" == "1" \
      && "$image" == "sift" && ! -e "$state/sift-shared-attached-once" ]]; then
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
  "storage cp "*)
    source_uri="${3:?}"
    destination="${4:?}"
    case "$source_uri" in
      */candidate-reservation.json)
        if [[ -e "$state/source-reservation-removed" ]]; then
          echo "NOT_FOUND" >&2
          exit 1
        fi
        if [[ -e "$state/source-reservation-replaced" ]]; then
          jq '.acquisition_id="22222222222222222222222222222222"' \
            "${SIFT_CANDIDATE_RECOVERY_DIR:?}/candidate-reservation.json" \
            > "$destination"
        else
          cp "${SIFT_CANDIDATE_RECOVERY_DIR:?}/candidate-reservation.json" "$destination"
        fi
        ;;
      */candidate-submit-intent.json)
        if [[ -e "$state/source-intent-removed" ]]; then
          echo "NOT_FOUND" >&2
          exit 1
        fi
        intent_count_file="$state/submit-intent-read-count"
        intent_count=0
        [[ ! -f "$intent_count_file" ]] || intent_count="$(cat "$intent_count_file")"
        intent_count=$((intent_count + 1))
        printf '%s\n' "$intent_count" > "$intent_count_file"
        if [[ "${SIFT_CANDIDATE_INTENT_MISMATCH_ON_SECOND:-0}" == "1" \
          && "$intent_count" -ge 2 ]]; then
          jq '.run_id="other-run"' \
            "${SIFT_CANDIDATE_RECOVERY_DIR:?}/candidate-submit-intent.json" \
            > "$destination"
        else
          cp "${SIFT_CANDIDATE_RECOVERY_DIR:?}/candidate-submit-intent.json" "$destination"
        fi
        ;;
      *)
        echo "NOT_FOUND" >&2
        exit 1
        ;;
    esac
    ;;
  "storage objects describe "*)
    source_uri="${4:?}"
    case "$source_uri" in
      */candidate-reservation.json)
        [[ ! -e "$state/source-reservation-removed" ]] || {
          echo "NOT_FOUND" >&2
          exit 1
        }
        generation="$(cat "$state/source-reservation.generation")"
        ;;
      */candidate-submit-intent.json)
        [[ ! -e "$state/source-intent-removed" ]] || {
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
  "storage rm "*)
    source_uri="${3:?}"
    generation_match=""
    for argument in "$@"; do
      case "$argument" in
        --if-generation-match=*) generation_match="${argument#*=}" ;;
      esac
    done
    case "$source_uri" in
      */source.tgz)
        marker="$state/source-object-removed"
        ;;
      */candidate-submit-intent.json)
        marker="$state/source-intent-removed"
        expected_generation="$(cat "$state/source-intent.generation")"
        ;;
      */candidate-reservation.json)
        if [[ "${SIFT_CANDIDATE_FAIL_RESERVATION_DELETE_ONCE:-0}" == "1" \
          && ! -e "$state/source-reservation-delete-failed-once" ]]; then
          : > "$state/source-reservation-delete-failed-once"
          echo "injected reservation delete failure" >&2
          exit 73
        fi
        marker="$state/source-reservation-removed"
        expected_generation="$(cat "$state/source-reservation.generation")"
        if [[ "${SIFT_CANDIDATE_REPLACE_RESERVATION_BEFORE_DELETE:-0}" == "1" \
          && ! -e "$state/source-reservation-replaced" ]]; then
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
    if [[ -e "$marker" ]]; then
      echo "NOT_FOUND" >&2
      exit 1
    fi
    : > "$marker"
    if [[ -e "$state/source-object-removed" \
      && -e "$state/source-intent-removed" \
      && -e "$state/source-reservation-removed" ]]; then
      : > "$state/source-removed"
    fi
    ;;
  "storage ls --recursive "*)
    if [[ -e "$state/source-removed" ]]; then
      if [[ "${SIFT_CANDIDATE_FAIL_SOURCE_LIST_ONCE:-0}" == "1" \
        && ! -e "$state/source-list-failed-once" ]]; then
        : > "$state/source-list-failed-once"
        echo "injected source inventory failure" >&2
        exit 72
      fi
      echo "matched no URLs" >&2
      exit 1
    fi
    [[ -e "$state/source-object-removed" ]] \
      || printf '%s/source.tgz\n' "${SIFT_CANDIDATE_SOURCE_PREFIX:?}"
    [[ -e "$state/source-intent-removed" ]] \
      || printf '%s/candidate-submit-intent.json\n' \
        "${SIFT_CANDIDATE_SOURCE_PREFIX:?}"
    [[ -e "$state/source-reservation-removed" ]] \
      || printf '%s/candidate-reservation.json\n' \
        "${SIFT_CANDIDATE_SOURCE_PREFIX:?}"
    ;;
  *)
    echo "unexpected fake gcloud call: $*" >&2
    exit 90
    ;;
esac
EOF
chmod +x "$fake_bin/gcloud"

restore_source_state() {
  rm -f "$state_dir/source-removed" \
    "$state_dir/source-object-removed" \
    "$state_dir/source-intent-removed" \
    "$state_dir/source-reservation-removed" \
    "$state_dir/source-reservation-delete-failed-once" \
    "$state_dir/source-reservation-replaced"
  printf '101\n' > "$state_dir/source-reservation.generation"
  printf '201\n' > "$state_dir/source-intent.generation"
}

run_cleanup() {
  PATH="$fake_bin:$PATH" \
    SIFT_CANDIDATE_CLEANUP_CALLS="$calls" \
    SIFT_CANDIDATE_CLEANUP_STATE="$state_dir" \
    SIFT_CANDIDATE_RECOVERY_DIR="$recovery_dir" \
    SIFT_CANDIDATE_IMAGE_TAG="$image_tag" \
    SIFT_CANDIDATE_SOURCE_PREFIX="$source_prefix" \
    CANDIDATE_CLEANUP_DISCOVERY_ATTEMPTS="${SIFT_TEST_DISCOVERY_ATTEMPTS:-1}" \
    CANDIDATE_CLEANUP_DISCOVERY_DELAY_SECONDS=0 \
    CANDIDATE_CLEANUP_WAIT_ATTEMPTS=1 \
    CANDIDATE_CLEANUP_WAIT_DELAY_SECONDS=0 \
    bash "$ACCEPTANCE_ROOT/scripts/cleanup-sift-candidate.sh" "$recovery_dir"
}

first_status=0
run_cleanup > "$test_root/first.log" 2>&1 || first_status=$?
[[ "$first_status" != "0" \
  && ! -e "$state_dir/sift.tag" \
  && -e "$state_dir/sift.digest" \
  && -f "$recovery_dir/candidate-cleanup-failures.log" \
  && ! -e "$recovery_dir/candidate-cleanup.json" ]] || {
  echo "first candidate cleanup did not preserve an interrupted digest deletion" >&2
  cat "$test_root/first.log" >&2
  exit 1
}

run_cleanup > "$test_root/second.log" 2>&1
for image in sift rig sift-acceptance-runner; do
  [[ ! -e "$state_dir/${image}.tag" && ! -e "$state_dir/${image}.digest" ]] || {
    echo "candidate cleanup retry left an image artifact: $image" >&2
    exit 1
  }
done
jq -e '
  .schema == "axiom.gcp.sift.candidate-cleanup.v1"
  and .status == "clean"
  and .cloud_build_ids == ["build-cleanup"]
  ' "$recovery_dir/candidate-cleanup.json" >/dev/null
[[ ! -e "$recovery_dir/candidate-cleanup-failures.log" ]]

# Restore only the Sift tag with a different live digest. Cleanup must not
# delete a tag that another actor moved after the build receipt was recorded.
cp "$state_dir/build.json" "$state_dir/build-mismatch.json"
expected_sift_digest="$(jq -er '.results.images[] | select(.name | contains("/sift:")) | .digest' \
  "$state_dir/build.json")"
printf '%s\n' "$expected_sift_digest" > "$state_dir/sift.digest"
printf '%s\n' "$expected_sift_digest" > "$state_dir/sift.tag"
rm -f "$recovery_dir/candidate-cleanup.json" "$state_dir/source-removed"
restore_source_state
: > "$calls"
mismatch_status=0
SIFT_CANDIDATE_TAG_MISMATCH=1 run_cleanup \
  > "$test_root/mismatch.log" 2>&1 || mismatch_status=$?
[[ "$mismatch_status" != "0" && -e "$state_dir/sift.tag" ]] || {
  echo "candidate cleanup accepted a moved live tag" >&2
  exit 1
}
if rg -F "artifacts docker images delete $registry/sift@$expected_sift_digest" \
    "$calls" >/dev/null; then
  echo "candidate cleanup deleted a digest after the live tag changed" >&2
  exit 1
fi
rg -F 'candidate tag no longer matches its Cloud Build receipt' \
  "$recovery_dir/candidate-cleanup-failures.log" >/dev/null

# A concurrent build can attach another tag after the candidate tag is removed.
# Digest deletion without --delete-tags must fail safely and retain the shared
# digest while cleanup still removes only this run's exact tag.
for image in sift rig sift-acceptance-runner; do
  digest="$(jq -er --arg image "$image" '
    .results.images[] | select(.name | contains("/" + $image + ":")) | .digest
  ' "$state_dir/build.json")"
  printf '%s\n' "$digest" > "$state_dir/${image}.digest"
  printf '%s\n' "$digest" > "$state_dir/${image}.tag"
done
rm -f "$recovery_dir/candidate-cleanup.json" \
  "$recovery_dir/candidate-cleanup-failures.log" "$state_dir/source-removed" \
  "$state_dir/sift-shared-attached-once" "$state_dir/sift.shared-tag"
restore_source_state
: > "$state_dir/sift-delete-failed-once"
: > "$calls"
SIFT_CANDIDATE_ATTACH_SHARED_TAG_ON_DELETE=1 run_cleanup \
  > "$test_root/shared-digest.log" 2>&1
[[ ! -e "$state_dir/sift.tag" && -e "$state_dir/sift.digest" \
  && -e "$state_dir/sift.shared-tag" ]] || {
  echo "candidate cleanup deleted a digest after another run attached a tag" >&2
  exit 1
}
if rg -F -- '--delete-tags' "$calls" >/dev/null; then
  echo "candidate cleanup still uses unsafe digest --delete-tags" >&2
  exit 1
fi
rm -f "$state_dir/sift.digest" "$state_dir/sift.shared-tag" \
  "$state_dir/sift-shared-attached-once"

# Cloud Build may fail after pushing tags but before results.images is filled.
# Exact live tags still give the immutable cleanup target.
for image in sift rig sift-acceptance-runner; do
  digest="$(printf '%s' "$image" | shasum -a 256 | awk '{print "sha256:" $1}')"
  printf '%s\n' "$digest" > "$state_dir/${image}.digest"
  printf '%s\n' "$digest" > "$state_dir/${image}.tag"
done
rm -f "$recovery_dir/candidate-cleanup.json" \
  "$recovery_dir/candidate-cleanup-failures.log" "$state_dir/source-removed"
restore_source_state
: > "$calls"
SIFT_CANDIDATE_RESULTS_EMPTY=1 run_cleanup \
  > "$test_root/partial-push.log" 2>&1
for image in sift rig sift-acceptance-runner; do
  [[ ! -e "$state_dir/${image}.tag" && ! -e "$state_dir/${image}.digest" ]] \
    || {
      echo "partial-push cleanup left an image artifact: $image" >&2
      exit 1
    }
done

# A build with source outside the reserved prefix is not owned. Do not cancel
# it and do not delete images or source.
for image in sift rig sift-acceptance-runner; do
  digest="$(printf '%s' "$image" | shasum -a 256 | awk '{print "sha256:" $1}')"
  printf '%s\n' "$digest" > "$state_dir/${image}.digest"
  printf '%s\n' "$digest" > "$state_dir/${image}.tag"
done
rm -f "$recovery_dir/candidate-cleanup.json" \
  "$recovery_dir/candidate-cleanup-failures.log" "$state_dir/source-removed"
restore_source_state
: > "$calls"
outside_status=0
SIFT_CANDIDATE_SOURCE_OUTSIDE=1 run_cleanup \
  > "$test_root/source-outside.log" 2>&1 || outside_status=$?
[[ "$outside_status" != "0" && -e "$state_dir/sift.tag" \
  && ! -e "$state_dir/source-removed" ]] || {
  echo "source-outside cleanup mutated remote state" >&2
  exit 1
}
if rg -F 'builds cancel ' "$calls" >/dev/null \
    || rg -F 'artifacts docker images delete ' "$calls" >/dev/null \
    || rg -F 'storage rm ' "$calls" >/dev/null; then
  echo "source-outside cleanup issued a mutating command" >&2
  exit 1
fi
rg -F 'does not match the candidate acquisition' \
  "$recovery_dir/candidate-cleanup-failures.log" >/dev/null

# The reservation allows at most one build. A second discovered build makes
# the whole cleanup read-only.
rm -f "$recovery_dir/candidate-cleanup.json" \
  "$recovery_dir/candidate-cleanup-failures.log" "$state_dir/source-removed"
restore_source_state
: > "$calls"
extra_status=0
SIFT_CANDIDATE_EXTRA_BUILD=1 run_cleanup \
  > "$test_root/extra-build.log" 2>&1 || extra_status=$?
[[ "$extra_status" != "0" && -e "$state_dir/sift.tag" \
  && ! -e "$state_dir/source-removed" ]] || {
  echo "multiple-build cleanup mutated remote state" >&2
  exit 1
}
if rg -F 'builds cancel ' "$calls" >/dev/null \
    || rg -F 'artifacts docker images delete ' "$calls" >/dev/null \
    || rg -F 'storage rm ' "$calls" >/dev/null; then
  echo "multiple-build cleanup issued a mutating command" >&2
  exit 1
fi
rg -F 'more than one Cloud Build claims this candidate acquisition' \
  "$recovery_dir/candidate-cleanup-failures.log" >/dev/null

# Discovery must consume every configured snapshot. A build that appears on
# the second read must block all cleanup.
rm -f "$recovery_dir/candidate-cleanup.json" \
  "$recovery_dir/candidate-cleanup-failures.log" "$state_dir/source-removed" \
  "$state_dir/build-list-count"
restore_source_state
: > "$calls"
late_status=0
SIFT_CANDIDATE_LATE_BUILD=1 SIFT_TEST_DISCOVERY_ATTEMPTS=2 run_cleanup \
  > "$test_root/late-build.log" 2>&1 || late_status=$?
[[ "$late_status" != "0" && -e "$state_dir/sift.tag" \
  && ! -e "$state_dir/source-removed" ]] || {
  echo "late-build cleanup did not fail closed" >&2
  exit 1
}
if rg -F 'builds cancel ' "$calls" >/dev/null \
    || rg -F 'artifacts docker images delete ' "$calls" >/dev/null \
    || rg -F 'storage rm ' "$calls" >/dev/null; then
  echo "late-build cleanup issued a mutating command" >&2
  exit 1
fi

# The submit intent is checked again after image cleanup and immediately before
# source deletion. A changed intent must retain the source for investigation.
for image in sift rig sift-acceptance-runner; do
  digest="$(printf '%s' "$image" | shasum -a 256 | awk '{print "sha256:" $1}')"
  printf '%s\n' "$digest" > "$state_dir/${image}.digest"
  printf '%s\n' "$digest" > "$state_dir/${image}.tag"
done
rm -f "$recovery_dir/candidate-cleanup.json" \
  "$recovery_dir/candidate-cleanup-failures.log" "$state_dir/source-removed" \
  "$state_dir/submit-intent-read-count"
restore_source_state
: > "$calls"
intent_status=0
SIFT_CANDIDATE_INTENT_MISMATCH_ON_SECOND=1 run_cleanup \
  > "$test_root/intent-mismatch.log" 2>&1 || intent_status=$?
[[ "$intent_status" != "0" && ! -e "$state_dir/source-removed" ]] || {
  echo "candidate cleanup deleted source after the submit intent changed" >&2
  exit 1
}
if rg -F 'storage rm ' "$calls" >/dev/null; then
  echo "candidate cleanup issued source deletion after the submit intent changed" >&2
  exit 1
fi
rg -F 'candidate source retained because ownership or build state is unsafe' \
  "$recovery_dir/candidate-cleanup-failures.log" >/dev/null

# The reservation is deleted last. If the final source inventory has a
# transient failure after that deletion, the exact same cleanup retry may only
# use the read-only source-absent finalization path.
for image in sift rig sift-acceptance-runner; do
  digest="$(printf '%s' "$image" | shasum -a 256 | awk '{print "sha256:" $1}')"
  printf '%s\n' "$digest" > "$state_dir/${image}.digest"
  printf '%s\n' "$digest" > "$state_dir/${image}.tag"
done
rm -f "$recovery_dir/candidate-cleanup.json" \
  "$recovery_dir/candidate-cleanup-failures.log" "$state_dir/source-removed" \
  "$state_dir/source-list-failed-once" \
  "$state_dir/submit-intent-read-count"
restore_source_state
: > "$state_dir/sift-delete-failed-once"
: > "$calls"
source_retry_status=0
SIFT_CANDIDATE_FAIL_SOURCE_LIST_ONCE=1 run_cleanup \
  > "$test_root/source-retry-first.log" 2>&1 || source_retry_status=$?
[[ "$source_retry_status" != "0" && -e "$state_dir/source-removed" ]] || {
  echo "candidate cleanup did not reach the injected post-delete failure" >&2
  exit 1
}
[[ ! -e "$recovery_dir/candidate-cleanup.json" ]]

run_cleanup > "$test_root/source-retry-second.log" 2>&1
jq -e '.status == "clean"' "$recovery_dir/candidate-cleanup.json" >/dev/null
if rg -F -- '--delete-tags' "$calls" >/dev/null; then
  echo "candidate cleanup issued an unsafe digest delete on retry" >&2
  exit 1
fi

# A crash or lost response may remove the submit intent while the reservation
# remains. The retry must use that exact live reservation only to finish the
# final control deletion. It must not repeat image or build mutations.
for image in sift rig sift-acceptance-runner; do
  digest="$(printf '%s' "$image" | shasum -a 256 | awk '{print "sha256:" $1}')"
  printf '%s\n' "$digest" > "$state_dir/${image}.digest"
  printf '%s\n' "$digest" > "$state_dir/${image}.tag"
done
rm -f "$recovery_dir/candidate-cleanup.json" \
  "$recovery_dir/candidate-cleanup-failures.log" \
  "$state_dir/source-list-failed-once" \
  "$state_dir/submit-intent-read-count"
restore_source_state
: > "$state_dir/sift-delete-failed-once"
: > "$calls"
reservation_retry_status=0
SIFT_CANDIDATE_FAIL_RESERVATION_DELETE_ONCE=1 run_cleanup \
  > "$test_root/reservation-retry-first.log" 2>&1 \
  || reservation_retry_status=$?
[[ "$reservation_retry_status" != "0" \
  && -e "$state_dir/source-object-removed" \
  && -e "$state_dir/source-intent-removed" \
  && ! -e "$state_dir/source-reservation-removed" \
  && ! -e "$state_dir/source-removed" \
  && ! -e "$recovery_dir/candidate-cleanup.json" ]] || {
  echo "candidate cleanup did not preserve the reservation after a partial control delete" >&2
  cat "$test_root/reservation-retry-first.log" >&2
  exit 1
}

: > "$calls"
run_cleanup > "$test_root/reservation-retry-second.log" 2>&1
jq -e '.status == "clean"' "$recovery_dir/candidate-cleanup.json" >/dev/null
[[ -e "$state_dir/source-removed" ]]
if rg -e '^(builds cancel|artifacts docker tags delete|artifacts docker images delete) ' \
    "$calls" >/dev/null; then
  echo "reservation-only retry repeated a forbidden mutation" >&2
  cat "$calls" >&2
  exit 1
fi
[[ "$(rg -c '^storage rm ' "$calls")" == "1" ]] || {
  echo "reservation-only retry deleted more than the exact final reservation" >&2
  cat "$calls" >&2
  exit 1
}
rg -F "storage rm $reservation_uri --if-generation-match=101 --quiet" \
  "$calls" >/dev/null

# A second cleanup can finish the old acquisition while this process is
# paused. A new acquisition may then create the same reservation URI. The old
# process must not delete that newer GCS generation.
for image in sift rig sift-acceptance-runner; do
  digest="$(printf '%s' "$image" | shasum -a 256 | awk '{print "sha256:" $1}')"
  printf '%s\n' "$digest" > "$state_dir/${image}.digest"
  printf '%s\n' "$digest" > "$state_dir/${image}.tag"
done
rm -f "$recovery_dir/candidate-cleanup.json" \
  "$recovery_dir/candidate-cleanup-failures.log"
restore_source_state
: > "$state_dir/sift-delete-failed-once"
: > "$calls"
generation_race_status=0
SIFT_CANDIDATE_REPLACE_RESERVATION_BEFORE_DELETE=1 run_cleanup \
  > "$test_root/generation-race.log" 2>&1 || generation_race_status=$?
[[ "$generation_race_status" != "0" \
  && -e "$state_dir/source-reservation-replaced" \
  && ! -e "$state_dir/source-reservation-removed" \
  && ! -e "$recovery_dir/candidate-cleanup.json" ]] || {
  echo "candidate cleanup deleted a replacement reservation generation" >&2
  cat "$test_root/generation-race.log" >&2
  exit 1
}
[[ "$(cat "$state_dir/source-reservation.generation")" == "102" ]]
rg -F "storage rm $reservation_uri --if-generation-match=101 --quiet" \
  "$calls" >/dev/null

# A completed immutable candidate is also an exact cleanup authority. It has
# stronger evidence than a preparation-failure receipt because it binds the
# final Cloud Build and all three immutable image digests. The cleanup must
# refuse an ambiguous directory that contains both receipt types, then accept
# the completed candidate after the stale failure receipt is removed.
jq -n --arg git_sha "$git_sha" --arg source_bundle_sha256 "$source_sha" '
  {
    git_sha:$git_sha,
    source_archive:("git-archive:" + $git_sha),
    source_bundle_sha256:$source_bundle_sha256,
    source_bundle_bytes:123
  }
' > "$recovery_dir/candidate-source.json"
jq -n --arg git_sha "$git_sha" --arg source_bundle_sha256 "$source_sha" '
  {
    schema:"axiom.gcp.sift.candidate-gate.v1",
    git_sha:$git_sha,
    source_bundle_sha256:$source_bundle_sha256,
    entrypoint:"apps/sift/test.sh --candidate",
    completed_at:"2026-09-03T00:00:02Z",
    status:"passed"
  }
' > "$recovery_dir/candidate-gate.json"
printf 'candidate passed\n' > "$recovery_dir/candidate-gate.log"
cp "$state_dir/build.json" "$recovery_dir/cloud-build-submit.json"
cp "$state_dir/build.json" "$recovery_dir/cloud-build-final.json"
jq -n '{
  bucket:"axiom-test_cloudbuild",
  name:"source/axiom-gcp-operator-cleanup-retry/source.tgz"
}' > "$recovery_dir/cloud-build-source-object.json"
jq -n \
  --arg git_sha "$git_sha" --arg source_bundle_sha256 "$source_sha" \
  --arg source_uri "$source_prefix/source.tgz" '
  {
    build_id:"build-cleanup",
    git_sha:$git_sha,
    source_uri:$source_uri,
    source_bundle_sha256:$source_bundle_sha256,
    staged_source_sha256:$source_bundle_sha256
  }
' > "$recovery_dir/cloud-build-source-binding.json"
for image in sift rig sift-acceptance-runner; do
  digest="$(jq -er --arg image "$image" '
    .results.images[] | select(.name | contains("/" + $image + ":")) | .digest
  ' "$state_dir/build.json")"
  printf '%s\n' "$digest" > "$state_dir/${image}.digest"
  printf '%s\n' "$digest" > "$state_dir/${image}.tag"
done
jq -n \
  --arg sift "$registry/sift@$(cat "$state_dir/sift.digest")" \
  --arg rig "$registry/rig@$(cat "$state_dir/rig.digest")" \
  --arg acceptance_runner "$registry/sift-acceptance-runner@$(cat "$state_dir/sift-acceptance-runner.digest")" '
  {sift:$sift,rig:$rig,acceptance_runner:$acceptance_runner}
' > "$recovery_dir/images.json"
jq -n '{}' > "$recovery_dir/preexisting-artifact-registry.json"
jq -n '{}' > "$recovery_dir/preexisting-cloud-build-source-bucket.json"
: > "$recovery_dir/preexisting-cloud-build-source-objects.txt"

file_hashes='{}'
while IFS= read -r name; do
  digest="$(sift_candidate_file_sha256 "$recovery_dir/$name")"
  file_hashes="$(jq -c --arg name "$name" --arg digest "$digest" \
    '. + {($name):$digest}' <<<"$file_hashes")"
done < <(sift_candidate_required_files)
jq -n \
  --arg git_sha "$git_sha" --arg image_tag "$image_tag" \
  --arg registry "$registry" --arg source_prefix "$source_prefix" \
  --arg acquisition_id "$acquisition_id" \
  --arg reservation_uri "$reservation_uri" \
  --arg source_bundle_sha256 "$source_sha" \
  --arg source_object_uri "$source_prefix/source.tgz" \
  --arg sift_image "$registry/sift@$(cat "$state_dir/sift.digest")" \
  --arg rig_image "$registry/rig@$(cat "$state_dir/rig.digest")" \
  --arg acceptance_runner_image "$registry/sift-acceptance-runner@$(cat "$state_dir/sift-acceptance-runner.digest")" \
  --argjson file_sha256 "$file_hashes" '
  {
    schema:"axiom.gcp.sift.candidate.v1",
    project_id:"axiom-test",
    region:"asia-east1",
    artifact_registry_repository:"courier",
    registry:$registry,
    run_id:"cleanup-retry",
    git_sha:$git_sha,
    image_tag:$image_tag,
    acquisition_id:$acquisition_id,
    reservation_uri:$reservation_uri,
    source_prefix:$source_prefix,
    source_bundle_sha256:$source_bundle_sha256,
    source_object_uri:$source_object_uri,
    cloud_build_id:"build-cleanup",
    sift_image:$sift_image,
    rig_image:$rig_image,
    acceptance_runner_image:$acceptance_runner_image,
    completed_at:"2026-09-03T00:00:03Z",
    file_sha256:$file_sha256
  }
' > "$recovery_dir/candidate.json"
verify_sift_candidate_directory "$recovery_dir"

for image in sift rig sift-acceptance-runner; do
  digest="$(jq -er --arg image "$image" '
    .results.images[] | select(.name | contains("/" + $image + ":")) | .digest
  ' "$state_dir/build.json")"
  printf '%s\n' "$digest" > "$state_dir/${image}.digest"
  printf '%s\n' "$digest" > "$state_dir/${image}.tag"
done
rm -f "$recovery_dir/candidate-cleanup.json" \
  "$recovery_dir/candidate-cleanup-failures.log"
restore_source_state
: > "$state_dir/sift-delete-failed-once"
: > "$calls"
ambiguous_status=0
run_cleanup > "$test_root/ambiguous-receipts.log" 2>&1 \
  || ambiguous_status=$?
[[ "$ambiguous_status" != "0" && -e "$state_dir/sift.tag" \
  && ! -e "$state_dir/source-removed" ]] || {
  echo "candidate cleanup accepted ambiguous cleanup receipts" >&2
  cat "$test_root/ambiguous-receipts.log" >&2
  exit 1
}
if rg -e '^(builds cancel|artifacts docker tags delete|artifacts docker images delete|storage rm) ' \
    "$calls" >/dev/null; then
  echo "ambiguous candidate cleanup issued a mutating command" >&2
  cat "$calls" >&2
  exit 1
fi

rm -f "$recovery_dir/candidate-preparation-failure.json"
: > "$calls"
printf 'tampered\n' >> "$recovery_dir/candidate-gate.log"
tampered_status=0
run_cleanup > "$test_root/tampered-complete-candidate.log" 2>&1 \
  || tampered_status=$?
[[ "$tampered_status" != "0" && -e "$state_dir/sift.tag" \
  && ! -e "$state_dir/source-removed" ]] || {
  echo "candidate cleanup accepted a tampered completed candidate" >&2
  cat "$test_root/tampered-complete-candidate.log" >&2
  exit 1
}
if [[ -s "$calls" ]]; then
  echo "tampered completed candidate cleanup called gcloud" >&2
  cat "$calls" >&2
  exit 1
fi
printf 'candidate passed\n' > "$recovery_dir/candidate-gate.log"
verify_sift_candidate_directory "$recovery_dir"

: > "$calls"
run_cleanup > "$test_root/complete-candidate.log" 2>&1
for image in sift rig sift-acceptance-runner; do
  [[ ! -e "$state_dir/${image}.tag" && ! -e "$state_dir/${image}.digest" ]] || {
    echo "completed candidate cleanup left an image artifact: $image" >&2
    exit 1
  }
done
[[ -e "$state_dir/source-removed" ]]
jq -e '
  .schema == "axiom.gcp.sift.candidate-cleanup.v1"
  and .status == "clean"
  and .cloud_build_ids == ["build-cleanup"]
' "$recovery_dir/candidate-cleanup.json" >/dev/null

echo "Sift candidate cleanup retry E2E: ok"
