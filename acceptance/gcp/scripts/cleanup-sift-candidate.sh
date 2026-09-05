#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$SCRIPT_DIR/source-prefix.sh"
source "$SCRIPT_DIR/sift-candidate.sh"

recovery_dir="${1:-}"
[[ "$#" == "1" && "$recovery_dir" == /* \
  && -d "$recovery_dir" && ! -L "$recovery_dir" ]] || {
  echo "usage: cleanup-sift-candidate.sh /absolute/path/to/candidate-or-failed" >&2
  exit 2
}

for command in awk cmp cp find gcloud grep jq mktemp openssl rm seq sleep sort; do
  command -v "$command" >/dev/null 2>&1 || {
    echo "required command not found: $command" >&2
    exit 1
  }
done

failure_receipt="$recovery_dir/candidate-preparation-failure.json"
candidate_receipt="$recovery_dir/candidate.json"
failure_receipt_present=0
candidate_receipt_present=0
[[ ! -e "$failure_receipt" && ! -L "$failure_receipt" ]] \
  || failure_receipt_present=1
[[ ! -e "$candidate_receipt" && ! -L "$candidate_receipt" ]] \
  || candidate_receipt_present=1

if [[ "$failure_receipt_present" == "1" \
  && "$candidate_receipt_present" == "1" ]]; then
  echo "candidate cleanup receipts are ambiguous" >&2
  exit 1
elif [[ "$candidate_receipt_present" == "1" ]]; then
  verify_sift_candidate_directory "$recovery_dir" || {
    echo "completed candidate receipt is invalid" >&2
    exit 1
  }
  identity_receipt="$candidate_receipt"
  SUBMIT_INTENT_PUBLISHED=true
  SUBMIT_STARTED=true
elif [[ "$failure_receipt_present" == "1" ]]; then
  [[ -f "$failure_receipt" && ! -L "$failure_receipt" ]] || {
    echo "candidate failure receipt is unsafe" >&2
    exit 1
  }
  jq -e '
    . as $failure
    | type == "object"
    and keys == [
      "acquisition_id", "artifact_registry_repository", "cloud_build_id",
      "exit_code", "failed_at", "git_sha", "image_tag", "project_id",
      "region", "registry", "reservation_uri", "run_id", "schema",
      "source_bundle_sha256", "source_prefix", "submit_intent_published",
      "submit_response_received", "submit_started"
    ]
    and .schema == "axiom.gcp.sift.candidate-preparation-failure.v2"
    and (.project_id | type) == "string"
    and (.project_id | test("^[a-z][a-z0-9-]{4,62}$"))
    and (.region | type) == "string"
    and (.region | test("^[a-z]+-[a-z]+[0-9]$"))
    and (.artifact_registry_repository | type) == "string"
    and (.artifact_registry_repository | test("^[a-z][a-z0-9._-]{0,62}$"))
    and (.run_id | type) == "string" and (.run_id | test("^[a-z0-9][a-z0-9-]{0,17}$"))
    and (.git_sha | type) == "string" and (.git_sha | test("^[0-9a-f]{40}$"))
    and (.source_bundle_sha256 | type) == "string"
    and (.source_bundle_sha256 | test("^[0-9a-f]{64}$"))
    and ((.cloud_build_id == "")
      or ((.cloud_build_id | type) == "string"
        and (.cloud_build_id | test("^[A-Za-z0-9-]{1,128}$"))))
    and (.submit_response_received | type) == "boolean"
    and .submit_response_received == (.cloud_build_id != "")
    and (.submit_intent_published | type) == "boolean"
    and (.submit_started | type) == "boolean"
    and ((.submit_started == false) or .submit_intent_published)
    and ((.cloud_build_id == "")
      or (.submit_started and .submit_intent_published))
    and (.acquisition_id | type) == "string"
    and (.acquisition_id | test("^[0-9a-f]{32}$"))
    and .registry == (.region + "-docker.pkg.dev/" + .project_id + "/" + .artifact_registry_repository)
    and .image_tag == (.git_sha + "-" + .run_id + "-" + .acquisition_id)
    and (.source_prefix | type) == "string"
    and (.source_prefix | endswith("/source/axiom-gcp-operator-" + $failure.run_id))
    and .reservation_uri == (.source_prefix + "/candidate-reservation.json")
    and (.exit_code | type) == "number" and (.exit_code | floor) == .exit_code
    and .exit_code > 0 and .exit_code <= 255
    and (.failed_at | type) == "string" and (.failed_at | length) > 0
  ' "$failure_receipt" >/dev/null || {
    echo "candidate failure receipt is invalid" >&2
    exit 1
  }
  identity_receipt="$failure_receipt"
  SUBMIT_INTENT_PUBLISHED="$(jq -r '.submit_intent_published' "$failure_receipt")"
  SUBMIT_STARTED="$(jq -r '.submit_started' "$failure_receipt")"
else
  echo "candidate cleanup receipt is missing" >&2
  exit 1
fi

PROJECT_ID="$(jq -er '.project_id' "$identity_receipt")"
REGION="$(jq -er '.region' "$identity_receipt")"
ARTIFACT_REGISTRY_REPOSITORY="$(jq -er '.artifact_registry_repository' "$identity_receipt")"
RUN_ID="$(jq -er '.run_id' "$identity_receipt")"
GIT_SHA="$(jq -er '.git_sha' "$identity_receipt")"
SOURCE_SHA="$(jq -er '.source_bundle_sha256' "$identity_receipt")"
ACQUISITION_ID="$(jq -er '.acquisition_id' "$identity_receipt")"
RESERVATION_URI="$(jq -er '.reservation_uri' "$identity_receipt")"
KNOWN_BUILD_ID="$(jq -er '.cloud_build_id' "$identity_receipt")"
REGISTRY="$(jq -er '.registry' "$identity_receipt")"
IMAGE_TAG="$(jq -er '.image_tag' "$identity_receipt")"
GCS_SOURCE_PREFIX="$(jq -er '.source_prefix' "$identity_receipt")"
DISCOVERY_ATTEMPTS="${CANDIDATE_CLEANUP_DISCOVERY_ATTEMPTS:-6}"
DISCOVERY_DELAY_SECONDS="${CANDIDATE_CLEANUP_DISCOVERY_DELAY_SECONDS:-10}"
WAIT_ATTEMPTS="${CANDIDATE_CLEANUP_WAIT_ATTEMPTS:-60}"
WAIT_DELAY_SECONDS="${CANDIDATE_CLEANUP_WAIT_DELAY_SECONDS:-5}"
[[ "$DISCOVERY_ATTEMPTS" =~ ^[1-9][0-9]*$ \
  && "$DISCOVERY_DELAY_SECONDS" =~ ^[0-9]+$ \
  && "$WAIT_ATTEMPTS" =~ ^[1-9][0-9]*$ \
  && "$WAIT_DELAY_SECONDS" =~ ^[0-9]+$ ]] || {
  echo "candidate cleanup retry settings are invalid" >&2
  exit 1
}
validated_source_bucket "$GCS_SOURCE_PREFIX" "$RUN_ID" >/dev/null || {
  echo "candidate source prefix is not run-scoped" >&2
  exit 1
}
verify_source_prefix_receipt \
  "$recovery_dir/source-prefix.json" "$PROJECT_ID" "$RUN_ID" \
  "$GCS_SOURCE_PREFIX" || {
  echo "candidate source-prefix receipt is missing or invalid" >&2
  exit 1
}
for image in sift rig sift-acceptance-runner; do
  inventory="$recovery_dir/preexisting-${image}-images.json"
  if [[ ! -f "$inventory" || -L "$inventory" ]] \
      || ! jq -e 'type == "array"' "$inventory" >/dev/null; then
    echo "candidate pre-run image inventory is missing or invalid: $image" >&2
    exit 1
  fi
done
reservation="$recovery_dir/candidate-reservation.json"
verify_sift_candidate_reservation "$reservation" || {
  echo "candidate reservation is missing or invalid" >&2
  exit 1
}
jq -e \
  --slurpfile identity "$identity_receipt" \
  --slurpfile sift "$recovery_dir/preexisting-sift-images.json" \
  --slurpfile rig "$recovery_dir/preexisting-rig-images.json" \
  --slurpfile runner "$recovery_dir/preexisting-sift-acceptance-runner-images.json" '
    ($identity[0]) as $i
    | .project_id == $i.project_id
    and .region == $i.region
    and .artifact_registry_repository == $i.artifact_registry_repository
    and .run_id == $i.run_id
    and .git_sha == $i.git_sha
    and .source_bundle_sha256 == $i.source_bundle_sha256
    and .acquisition_id == $i.acquisition_id
    and .registry == $i.registry
    and .image_tag == $i.image_tag
    and .source_prefix == $i.source_prefix
    and .reservation_uri == $i.reservation_uri
    and .preexisting_images.sift == $sift[0]
    and .preexisting_images.rig == $rig[0]
    and .preexisting_images.sift_acceptance_runner == $runner[0]
  ' "$reservation" >/dev/null || {
  echo "candidate reservation does not match its cleanup receipt" >&2
  exit 1
}
submit_intent="$recovery_dir/candidate-submit-intent.json"
local_submit_intent=0
if [[ -f "$submit_intent" && ! -L "$submit_intent" ]]; then
  verify_sift_candidate_submit_intent "$submit_intent" "$reservation" || {
    echo "candidate submit intent is missing or invalid" >&2
    exit 1
  }
  local_submit_intent=1
elif [[ "$SUBMIT_INTENT_PUBLISHED" == "true" ]]; then
  echo "published candidate submit intent is missing from recovery evidence" >&2
  exit 1
fi

work_root="$(mktemp -d "${TMPDIR:-/tmp}/sift-candidate-cleanup.XXXXXX")"
cleanup_local() {
  find "$work_root" -depth -delete >/dev/null 2>&1 || true
}
trap cleanup_local EXIT INT TERM
failures="$work_root/failures.log"
: > "$failures"
record_failure() {
  printf '%s\n' "$1" >> "$failures"
  echo "$1" >&2
}

remote_object_is_absent_error() {
  local error_file="$1"
  grep -Eiq \
    '(^|[^[:alpha:]])(not[ _-]?found|404)([^[:alpha:]]|$)|matched no (objects|URLs)|no URLs matched' \
    "$error_file"
}

remote_prefix_is_empty_error() {
  local error_file="$1"
  grep -Eiq 'matched no (objects|URLs)|no URLs matched' "$error_file"
}

verify_live_object_generation() {
  local uri="$1"
  local expected="$2"
  local name="$3"
  local downloaded="$work_root/${name}.live"
  local metadata="$work_root/${name}.metadata.json"
  local generation
  if ! gcloud storage objects describe "$uri" --format=json \
      > "$metadata" 2> "$work_root/${name}.stderr"; then
    return 1
  fi
  generation="$(jq -er \
    '.generation | tostring | select(test("^[1-9][0-9]*$"))' \
    "$metadata")" || return 1
  if ! gcloud storage cp "$uri" "$downloaded" \
      > /dev/null 2>> "$work_root/${name}.stderr"; then
    return 1
  fi
  [[ -f "$downloaded" && ! -L "$downloaded" ]] \
    && cmp -s "$expected" "$downloaded" \
    || return 1
  printf '%s\n' "$generation"
}

verify_live_object() {
  verify_live_object_generation "$@" >/dev/null
}

source_prefix_is_empty() {
  local phase="$1"
  local output="$work_root/source-${phase}.list"
  local error="$work_root/source-${phase}.stderr"
  local status=0
  gcloud storage ls --recursive "${GCS_SOURCE_PREFIX}/**" \
    > "$output" 2> "$error" || status=$?
  if [[ "$status" == "0" ]]; then
    [[ ! -s "$output" ]]
    return
  fi
  remote_prefix_is_empty_error "$error"
}

discover_builds() {
  local phase="$1"
  local output="$2"
  local attempt snapshot
  local snapshots=()
  for attempt in $(seq 1 "$DISCOVERY_ATTEMPTS"); do
    snapshot="$work_root/builds-${phase}-${attempt}.json"
    if ! gcloud builds list --project="$PROJECT_ID" --region="$REGION" \
        --filter="tags=axiom-acquisition-${ACQUISITION_ID}" --format=json \
        > "$snapshot"; then
      return 1
    fi
    jq -e '
      type == "array"
      and all(.[];
        (.id | type) == "string"
        and (.id | test("^[A-Za-z0-9-]{1,128}$")))
    ' "$snapshot" >/dev/null || return 1
    snapshots+=("$snapshot")
    if [[ "$attempt" != "$DISCOVERY_ATTEMPTS" \
      && "$DISCOVERY_DELAY_SECONDS" != "0" ]]; then
      sleep "$DISCOVERY_DELAY_SECONDS"
    fi
  done
  jq -s 'add | unique_by(.id)' "${snapshots[@]}" > "$output"
}

builds="$work_root/builds.json"
builds_safe=1
if ! discover_builds initial "$builds"; then
  record_failure "could not inventory acquisition-tagged Cloud Builds"
  printf '[]\n' > "$builds"
  builds_safe=0
fi
cp "$builds" "$recovery_dir/candidate-cleanup-build-inventory.json"
chmod 0600 "$recovery_dir/candidate-cleanup-build-inventory.json"

build_receipts="$work_root/build-receipts"
mkdir -p "$build_receipts"
build_ids="$work_root/build-ids.txt"
{
  jq -r '.[].id' "$builds"
  [[ -z "$KNOWN_BUILD_ID" ]] || printf '%s\n' "$KNOWN_BUILD_ID"
} | sort -u > "$build_ids"
build_count="$(awk 'NF { count += 1 } END { print count + 0 }' "$build_ids")"
if [[ "$SUBMIT_STARTED" == "true" && "$build_count" != "1" ]]; then
  if [[ "$build_count" == "0" ]]; then
    record_failure \
      "no Cloud Build is visible, so an accepted submit without a response cannot yet be excluded"
  else
    record_failure "more than one Cloud Build claims this candidate acquisition"
  fi
  builds_safe=0
elif [[ "$SUBMIT_STARTED" == "false" && "$build_count" != "0" ]]; then
  record_failure "a Cloud Build exists before the submit boundary"
  builds_safe=0
fi

verify_build_receipt() {
  local receipt="$1"
  local build_id="$2"
  local source_bucket source_object
  jq -e \
    --arg id "$build_id" --arg run_id "$RUN_ID" \
    --arg git_sha "$GIT_SHA" --arg source_sha "$SOURCE_SHA" \
    --arg acquisition_id "$ACQUISITION_ID" \
    --arg registry "$REGISTRY" --arg image_tag "$IMAGE_TAG" '
      type == "object"
      and .id == $id
      and .substitutions._RUN_ID == $run_id
      and .substitutions._GIT_SHA == $git_sha
      and .substitutions._SOURCE_BUNDLE_SHA256 == $source_sha
      and .substitutions._CANDIDATE_ACQUISITION_ID == $acquisition_id
      and .substitutions._REGISTRY == $registry
      and .substitutions._TAG == $image_tag
      and ((.tags // []) | index("sift-mvp") != null)
      and ((.tags // []) | index("axiom-run-" + $run_id) != null)
      and ((.tags // []) | index("axiom-source-" + $source_sha) != null)
      and ((.tags // []) | index("axiom-acquisition-" + $acquisition_id) != null)
      and (.source.storageSource.bucket | type) == "string"
      and (.source.storageSource.object | type) == "string"
    ' "$receipt" >/dev/null || return 1
  source_bucket="$(jq -er '.source.storageSource.bucket' "$receipt")" || return 1
  source_object="$(jq -er '.source.storageSource.object' "$receipt")" || return 1
  validated_source_object_uri \
    "$GCS_SOURCE_PREFIX" "$RUN_ID" "$source_bucket" "$source_object" \
    >/dev/null
}

source_prefix_has_only_reservation() {
  local phase="$1"
  local output="$work_root/source-${phase}.list"
  local error="$work_root/source-${phase}.stderr"
  local status=0 count only_uri
  gcloud storage ls --recursive "${GCS_SOURCE_PREFIX}/**" \
    > "$output" 2> "$error" || status=$?
  [[ "$status" == "0" ]] || return 1
  count="$(awk 'NF { count += 1 } END { print count + 0 }' "$output")"
  only_uri="$(awk 'NF { print; exit }' "$output")"
  [[ "$count" == "1" && "$only_uri" == "$RESERVATION_URI" ]]
}

validate_read_only_terminal_candidate() {
  local context="$1"
  local build_id receipt status image tagged tag_error final_builds
  [[ "$builds_safe" == "1" ]] || {
    echo "$context but Cloud Build identity is not safe" >&2
    return 1
  }
  while IFS= read -r build_id; do
    [[ -n "$build_id" ]] || continue
    receipt="$build_receipts/${build_id}.json"
    if ! gcloud builds describe "$build_id" --project="$PROJECT_ID" \
        --region="$REGION" --format=json > "$receipt" \
        || ! verify_build_receipt "$receipt" "$build_id"; then
      echo "$context but Cloud Build identity changed" >&2
      return 1
    fi
    status="$(jq -er '.status' "$receipt")" || return 1
    case "$status" in
      SUCCESS|FAILURE|INTERNAL_ERROR|TIMEOUT|CANCELLED|EXPIRED) ;;
      *)
        echo "$context while Cloud Build is still active" >&2
        return 1
        ;;
    esac
  done < "$build_ids"
  for image in sift rig sift-acceptance-runner; do
    tagged="$REGISTRY/$image:$IMAGE_TAG"
    tag_error="$work_root/${image}-read-only-tag.stderr"
    if gcloud artifacts docker images describe "$tagged" \
        --project="$PROJECT_ID" --format='value(image_summary.digest)' \
        > "$work_root/${image}-read-only-tag.out" 2> "$tag_error"; then
      echo "$context but its image tag still exists: $tagged" >&2
      return 1
    elif ! grep -Eiq '(^|[^[:alpha:]])(not[ _-]?found|404)([^[:alpha:]]|$)' \
        "$tag_error"; then
      echo "could not verify the final candidate image tag: $tagged" >&2
      return 1
    fi
  done
  final_builds="$work_root/read-only-final-builds.json"
  discover_builds read-only-final "$final_builds" || return 1
  jq -e --slurpfile initial "$builds" '
    ([.[]?.id] | sort) == ([$initial[0][]?.id] | sort)
  ' "$final_builds" >/dev/null || {
    echo "a late Cloud Build appeared during read-only finalization" >&2
    return 1
  }
}

write_candidate_cleanup_receipt() {
  local cleanup_build_ids
  cleanup_build_ids="$(jq -Rsc \
    'split("\n") | map(select(length > 0)) | sort' < "$build_ids")"
  jq -n \
    --arg project_id "$PROJECT_ID" --arg region "$REGION" \
    --arg run_id "$RUN_ID" --arg git_sha "$GIT_SHA" \
    --arg cleaned_at "$(date -u +%Y-%m-%dT%H:%M:%SZ)" \
    --argjson cloud_build_ids "$cleanup_build_ids" '
      {
        schema:"axiom.gcp.sift.candidate-cleanup.v1",
        project_id:$project_id,
        region:$region,
        run_id:$run_id,
        git_sha:$git_sha,
        cloud_build_ids:$cloud_build_ids,
        status:"clean",
        cleaned_at:$cleaned_at
      }
    ' > "$recovery_dir/candidate-cleanup.json"
  chmod 0600 "$recovery_dir/candidate-cleanup.json"
  rm -f "$recovery_dir/candidate-cleanup-failures.log"
  echo "Sift candidate cleanup passed: $recovery_dir/candidate-cleanup.json"
}

# Missing live control objects never authorize another destructive cleanup.
# They can only enter this read-only terminal check. This supports a retry when
# the previous process removed the reservation last, then lost its final local
# response or receipt write.
source_absent_finalize=0
reservation_only_finalize=0
live_submit_intent=0
if verify_live_object "$RESERVATION_URI" "$reservation" reservation; then
  if [[ "$local_submit_intent" == "1" ]]; then
    if verify_live_object "$(sift_candidate_submit_intent_uri "$GCS_SOURCE_PREFIX")" \
        "$submit_intent" submit-intent; then
      live_submit_intent=1
    elif remote_object_is_absent_error "$work_root/submit-intent.stderr"; then
      if [[ "$SUBMIT_INTENT_PUBLISHED" == "true" ]]; then
        if source_prefix_has_only_reservation missing-intent; then
          reservation_only_finalize=1
        else
          echo "published candidate submit intent is missing" >&2
          exit 1
        fi
      fi
    else
      echo "live candidate submit intent is missing or changed" >&2
      exit 1
    fi
  elif verify_live_object \
      "$(sift_candidate_submit_intent_uri "$GCS_SOURCE_PREFIX")" \
      "$reservation" unexpected-submit-intent; then
    echo "unexpected candidate submit intent matches the reservation" >&2
    exit 1
  elif ! remote_object_is_absent_error \
      "$work_root/unexpected-submit-intent.stderr"; then
    echo "an unverified candidate submit intent exists" >&2
    exit 1
  fi
elif remote_object_is_absent_error "$work_root/reservation.stderr" \
    && source_prefix_is_empty missing-control; then
  source_absent_finalize=1
else
  echo "live candidate reservation is missing or owned by another acquisition" >&2
  exit 1
fi

if [[ "$source_absent_finalize" == "1" ]]; then
  validate_read_only_terminal_candidate "candidate source is absent" || exit 1
  source_prefix_is_empty read-only-final || {
    echo "candidate source reappeared during read-only finalization" >&2
    exit 1
  }
  write_candidate_cleanup_receipt
  exit 0
fi

if [[ "$reservation_only_finalize" == "1" ]]; then
  validate_read_only_terminal_candidate \
    "only the candidate reservation remains" || exit 1
  source_prefix_has_only_reservation reservation-only-final || {
    echo "candidate source changed during reservation-only finalization" >&2
    exit 1
  }
  reservation_generation="$(verify_live_object_generation \
    "$RESERVATION_URI" "$reservation" reservation-only-final)" || {
      echo "candidate reservation changed during finalization" >&2
      exit 1
    }
  reservation_delete_error="$work_root/reservation-only-delete.stderr"
  if ! gcloud storage rm "$RESERVATION_URI" \
      --if-generation-match="$reservation_generation" --quiet \
      > /dev/null 2> "$reservation_delete_error"; then
    echo "could not delete the final candidate reservation" >&2
    exit 1
  fi
  source_prefix_is_empty reservation-only-deleted || {
    echo "candidate source remains after reservation-only finalization" >&2
    exit 1
  }
  write_candidate_cleanup_receipt
  exit 0
fi

if [[ "$builds_safe" == "1" ]]; then
  while IFS= read -r build_id; do
    [[ -n "$build_id" ]] || continue
    receipt="$build_receipts/${build_id}.json"
    if ! gcloud builds describe "$build_id" --project="$PROJECT_ID" \
        --region="$REGION" --format=json > "$receipt"; then
      record_failure "could not describe Cloud Build $build_id"
      builds_safe=0
      continue
    fi
    if ! verify_build_receipt "$receipt" "$build_id"; then
      record_failure "Cloud Build $build_id does not match the candidate acquisition"
      builds_safe=0
      continue
    fi
    status="$(jq -er '.status' "$receipt")" || {
      record_failure "Cloud Build $build_id has no status"
      builds_safe=0
      continue
    }
    case "$status" in
      SUCCESS|FAILURE|INTERNAL_ERROR|TIMEOUT|CANCELLED|EXPIRED) ;;
      *)
        if ! gcloud builds cancel "$build_id" --project="$PROJECT_ID" \
            --region="$REGION" --quiet >/dev/null 2>&1; then
          record_failure "could not cancel Cloud Build $build_id"
          builds_safe=0
          continue
        fi
        terminal=0
        for wait_attempt in $(seq 1 "$WAIT_ATTEMPTS"); do
          if gcloud builds describe "$build_id" --project="$PROJECT_ID" \
              --region="$REGION" --format=json > "$receipt" \
              && status="$(jq -er '.status' "$receipt")"; then
            case "$status" in
              SUCCESS|FAILURE|INTERNAL_ERROR|TIMEOUT|CANCELLED|EXPIRED)
                terminal=1
                break
                ;;
            esac
          fi
          if [[ "$wait_attempt" != "$WAIT_ATTEMPTS" \
            && "$WAIT_DELAY_SECONDS" != "0" ]]; then
            sleep "$WAIT_DELAY_SECONDS"
          fi
        done
        if [[ "$terminal" != "1" ]]; then
          record_failure "Cloud Build $build_id did not reach a terminal state"
          builds_safe=0
        elif ! verify_build_receipt "$receipt" "$build_id"; then
          record_failure "Cloud Build $build_id changed identity while cancellation completed"
          builds_safe=0
        fi
        ;;
    esac
  done < "$build_ids"
fi

if [[ -s "$build_ids" ]]; then
  while IFS= read -r build_id; do
    [[ -f "$build_receipts/${build_id}.json" ]] || continue
    cp "$build_receipts/${build_id}.json" \
      "$recovery_dir/candidate-cleanup-build-${build_id}.json"
    chmod 0600 "$recovery_dir/candidate-cleanup-build-${build_id}.json"
  done < "$build_ids"
fi

build_receipt_array="$work_root/build-receipts.json"
if find "$build_receipts" -type f -print -quit | grep -q .; then
  jq -s '.' "$build_receipts"/*.json > "$build_receipt_array"
else
  printf '[]\n' > "$build_receipt_array"
fi

image_cleanup_safe=1
if [[ "$builds_safe" == "1" ]]; then
  for image in sift rig sift-acceptance-runner; do
    tagged="$REGISTRY/$image:$IMAGE_TAG"
    expected_digest="$(jq -er --arg tagged "$tagged" '
      [.[] | .results.images[]? | select(.name == $tagged) | .digest] | unique
      | if length == 0 then ""
        elif length == 1 then .[0]
        else error("multiple candidate digests")
        end
    ' "$build_receipt_array")" || {
      record_failure "could not identify the candidate digest for $image"
      image_cleanup_safe=0
      continue
    }
    tag_error="$work_root/${image}-tag.stderr"
    live_digest=""
    tag_present=0
    if live_digest="$(gcloud artifacts docker images describe "$tagged" \
        --project="$PROJECT_ID" --format='value(image_summary.digest)' \
        2> "$tag_error")"; then
      tag_present=1
      [[ "$live_digest" == sha256:* ]] || {
        record_failure "live candidate tag has no digest: $tagged"
        image_cleanup_safe=0
        continue
      }
    elif grep -Eiq '(^|[^[:alpha:]])(not[ _-]?found|404)([^[:alpha:]]|$)' \
        "$tag_error"; then
      live_digest="$expected_digest"
    else
      record_failure "could not inspect candidate tag $tagged"
      image_cleanup_safe=0
      continue
    fi
    if [[ "$tag_present" == "1" && -n "$expected_digest" \
      && "$live_digest" != "$expected_digest" ]]; then
      record_failure "candidate tag no longer matches its Cloud Build receipt: $tagged"
      image_cleanup_safe=0
      continue
    fi
    if [[ -z "$live_digest" ]]; then
      continue
    fi
    [[ "$live_digest" =~ ^sha256:[0-9a-f]{64}$ ]] || {
      record_failure "candidate digest is invalid for $image"
      image_cleanup_safe=0
      continue
    }
    inventory="$recovery_dir/preexisting-${image}-images.json"
    if jq -e --arg digest "$live_digest" \
        'any(.. | strings; contains($digest))' "$inventory" >/dev/null; then
      if [[ "$tag_present" == "1" ]]; then
        record_failure "candidate tag points at a pre-existing digest: $tagged"
        image_cleanup_safe=0
      fi
      continue
    fi
    if ! current="$(gcloud artifacts docker images list "$REGISTRY/$image" \
        --project="$PROJECT_ID" --include-tags --format=json)"; then
      record_failure "could not inventory candidate digest for $image"
      image_cleanup_safe=0
      continue
    fi
    if ! jq -e 'type == "array"' <<<"$current" >/dev/null; then
      record_failure "candidate image inventory is not a JSON array: $image"
      image_cleanup_safe=0
      continue
    fi
    digest_matches="$(jq -cer --arg digest "$live_digest" '
      [.[] | select((tojson | contains($digest)))]
    ' <<<"$current")" || {
      record_failure "could not locate candidate digest for $image"
      image_cleanup_safe=0
      continue
    }
    digest_match_count="$(jq 'length' <<<"$digest_matches")"
    if [[ "$digest_match_count" == "0" && "$tag_present" == "0" ]]; then
      continue
    fi
    if [[ "$digest_match_count" != "1" ]]; then
      record_failure "candidate digest inventory is ambiguous for $image"
      image_cleanup_safe=0
      continue
    fi
    unexpected_tags="$(jq -r --arg tag "$IMAGE_TAG" '
      [.[0].tags[]? | select(. != $tag)] | length
    ' <<<"$digest_matches")" || {
      record_failure "could not validate candidate digest tags for $image"
      image_cleanup_safe=0
      continue
    }
    candidate_tag_count="$(jq -r --arg tag "$IMAGE_TAG" '
      [.[0].tags[]? | select(. == $tag)] | length
    ' <<<"$digest_matches")" || {
      record_failure "could not validate the candidate tag for $image"
      image_cleanup_safe=0
      continue
    }
    if [[ "$tag_present" == "1" \
      && ( "$unexpected_tags" != "0" || "$candidate_tag_count" != "1" ) ]]; then
      record_failure "candidate digest has tags outside this acquisition: $image"
      image_cleanup_safe=0
      continue
    fi
    if [[ "$tag_present" == "0" && "$candidate_tag_count" != "0" ]]; then
      record_failure "candidate tag inventory disagrees with the live tag lookup: $image"
      image_cleanup_safe=0
      continue
    fi

    # Remove only this acquisition's exact tag. A digest delete is safe only
    # after a second inventory proves that the digest is untagged.
    if [[ "$tag_present" == "1" ]]; then
      predelete_tag_error="$work_root/${image}-predelete-tag.stderr"
      if ! predelete_digest="$(gcloud artifacts docker images describe "$tagged" \
          --project="$PROJECT_ID" --format='value(image_summary.digest)' \
          2> "$predelete_tag_error")" \
          || [[ "$predelete_digest" != "$live_digest" ]]; then
        record_failure "candidate tag changed immediately before deletion: $tagged"
        image_cleanup_safe=0
        continue
      fi
      tag_delete_error="$work_root/${image}-tag-delete.stderr"
      if ! gcloud artifacts docker tags delete "$tagged" \
          --project="$PROJECT_ID" --quiet 2> "$tag_delete_error" \
          && ! grep -Eiq '(^|[^[:alpha:]])(not[ _-]?found|404)([^[:alpha:]]|$)' \
            "$tag_delete_error"; then
        record_failure "could not delete the exact candidate tag: $tagged"
        image_cleanup_safe=0
        continue
      fi
    elif [[ "$unexpected_tags" != "0" ]]; then
      # A prior attempt already removed this run's tag. Another run now owns
      # the shared digest, so this cleanup must retain it.
      continue
    fi
    final_tag_error="$work_root/${image}-final-tag.stderr"
    if gcloud artifacts docker images describe "$tagged" \
        --project="$PROJECT_ID" --format='value(image_summary.digest)' \
        > "$work_root/${image}-final-tag.out" 2> "$final_tag_error"; then
      record_failure "candidate tag still exists after exact tag deletion: $tagged"
      image_cleanup_safe=0
    elif ! grep -Eiq '(^|[^[:alpha:]])(not[ _-]?found|404)([^[:alpha:]]|$)' \
        "$final_tag_error"; then
      record_failure "could not verify candidate tag removal: $tagged"
      image_cleanup_safe=0
      continue
    fi
    if ! untagged_inventory="$(gcloud artifacts docker images list "$REGISTRY/$image" \
        --project="$PROJECT_ID" --include-tags --format=json)" \
        || ! jq -e 'type == "array"' >/dev/null <<<"$untagged_inventory"; then
      record_failure "could not verify candidate digest tags after tag deletion: $image"
      image_cleanup_safe=0
      continue
    fi
    untagged_matches="$(jq -cer --arg digest "$live_digest" '
      [.[] | select((tojson | contains($digest)))]
    ' <<<"$untagged_inventory")" || {
      record_failure "could not locate candidate digest after tag deletion: $image"
      image_cleanup_safe=0
      continue
    }
    untagged_match_count="$(jq 'length' <<<"$untagged_matches")"
    if [[ "$untagged_match_count" == "0" ]]; then
      continue
    fi
    if [[ "$untagged_match_count" != "1" ]]; then
      record_failure "candidate digest inventory changed ambiguously: $image"
      image_cleanup_safe=0
      continue
    fi
    remaining_candidate_tags="$(jq -r --arg tag "$IMAGE_TAG" '
      [.[0].tags[]? | select(. == $tag)] | length
    ' <<<"$untagged_matches")" || {
      record_failure "could not verify candidate tag removal from digest: $image"
      image_cleanup_safe=0
      continue
    }
    remaining_other_tags="$(jq -r --arg tag "$IMAGE_TAG" '
      [.[0].tags[]? | select(. != $tag)] | length
    ' <<<"$untagged_matches")" || {
      record_failure "could not verify shared tags on candidate digest: $image"
      image_cleanup_safe=0
      continue
    }
    if [[ "$remaining_candidate_tags" != "0" ]]; then
      record_failure "candidate tag still appears in digest inventory: $image"
      image_cleanup_safe=0
      continue
    fi
    if [[ "$remaining_other_tags" != "0" ]]; then
      continue
    fi

    digest_error="$work_root/${image}-digest-delete.stderr"
    digest_delete_status=0
    gcloud artifacts docker images delete "$REGISTRY/$image@$live_digest" \
      --project="$PROJECT_ID" --quiet \
      > /dev/null 2> "$digest_error" || digest_delete_status=$?
    if ! final_inventory="$(gcloud artifacts docker images list "$REGISTRY/$image" \
        --project="$PROJECT_ID" --include-tags --format=json)" \
        || ! jq -e 'type == "array"' >/dev/null <<<"$final_inventory"; then
      record_failure "could not verify immutable digest removal: $image"
      image_cleanup_safe=0
      continue
    fi
    final_matches="$(jq -cer --arg digest "$live_digest" '
      [.[] | select((tojson | contains($digest)))]
    ' <<<"$final_inventory")" || {
      record_failure "could not locate candidate digest after digest deletion: $image"
      image_cleanup_safe=0
      continue
    }
    final_match_count="$(jq 'length' <<<"$final_matches")"
    if [[ "$final_match_count" == "0" ]]; then
      continue
    fi
    if [[ "$final_match_count" == "1" ]] \
      && jq -e --arg tag "$IMAGE_TAG" '
        ([.[0].tags[]? | select(. == $tag)] | length) == 0
        and ([.[0].tags[]? | select(. != $tag)] | length) > 0
      ' >/dev/null <<<"$final_matches"; then
      # A concurrent owner attached a tag before the untagged delete. Keep the
      # shared digest even when Artifact Registry rejected the delete.
      continue
    fi
    if [[ "$digest_delete_status" != "0" ]]; then
      record_failure "could not delete the untagged candidate digest for $image"
    else
      record_failure "immutable candidate digest still exists: $image"
    fi
    image_cleanup_safe=0
  done
else
  image_cleanup_safe=0
  record_failure "candidate images retained because Cloud Build state is unsafe"
fi

final_builds="$work_root/final-builds.json"
if [[ "$builds_safe" == "1" && "$image_cleanup_safe" == "1" ]]; then
  if ! discover_builds final "$final_builds"; then
    record_failure "could not perform the final Cloud Build inventory"
    builds_safe=0
  elif ! jq -e --slurpfile initial "$builds" '
      ([.[]?.id] | sort) == ([$initial[0][]?.id] | sort)
    ' "$final_builds" >/dev/null; then
    record_failure "Cloud Build inventory changed during candidate cleanup"
    builds_safe=0
  fi
fi
if [[ "$builds_safe" == "1" && -s "$build_ids" ]]; then
  while IFS= read -r build_id; do
    [[ -n "$build_id" ]] || continue
    final_receipt="$work_root/final-build-${build_id}.json"
    if ! gcloud builds describe "$build_id" --project="$PROJECT_ID" \
        --region="$REGION" --format=json > "$final_receipt" \
        || ! verify_build_receipt "$final_receipt" "$build_id"; then
      record_failure "Cloud Build $build_id failed final identity validation"
      builds_safe=0
      continue
    fi
    final_status="$(jq -er '.status' "$final_receipt")" || final_status=""
    case "$final_status" in
      SUCCESS|FAILURE|INTERNAL_ERROR|TIMEOUT|CANCELLED|EXPIRED) ;;
      *)
        record_failure "Cloud Build $build_id is not terminal at final verification"
        builds_safe=0
        ;;
    esac
  done < "$build_ids"
fi

# Perform every late-build and tag check while the live control objects still
# prove this acquisition. Source controls are removed only after these checks.
post_builds="$work_root/post-builds.json"
if [[ "$builds_safe" == "1" ]]; then
  if ! discover_builds post "$post_builds"; then
    record_failure "could not repeat the final Cloud Build inventory"
    builds_safe=0
  elif ! jq -e --slurpfile initial "$builds" '
      ([.[]?.id] | sort) == ([$initial[0][]?.id] | sort)
    ' "$post_builds" >/dev/null; then
    record_failure "a late Cloud Build appeared during candidate cleanup"
    builds_safe=0
  fi
fi

if [[ "$image_cleanup_safe" == "1" ]]; then
  for image in sift rig sift-acceptance-runner; do
    tagged="$REGISTRY/$image:$IMAGE_TAG"
    final_tag_error="$work_root/${image}-post-tag.stderr"
    if gcloud artifacts docker images describe "$tagged" \
        --project="$PROJECT_ID" --format='value(image_summary.digest)' \
        > "$work_root/${image}-post-tag.out" 2> "$final_tag_error"; then
      record_failure "candidate tag exists at final cleanup verification: $tagged"
      image_cleanup_safe=0
    elif ! grep -Eiq '(^|[^[:alpha:]])(not[ _-]?found|404)([^[:alpha:]]|$)' \
        "$final_tag_error"; then
      record_failure "could not perform final candidate tag verification: $tagged"
      image_cleanup_safe=0
    fi
  done
fi

source_cleanup_safe=1
if [[ "$builds_safe" == "1" && "$image_cleanup_safe" == "1" ]] \
    && verify_live_object "$RESERVATION_URI" "$reservation" reservation-before-delete \
    && { [[ "$live_submit_intent" == "0" ]] \
      || verify_live_object \
        "$(sift_candidate_submit_intent_uri "$GCS_SOURCE_PREFIX")" \
        "$submit_intent" submit-intent-before-delete; }; then
  source_inventory="$work_root/source-before-delete.list"
  source_inventory_error="$work_root/source-before-delete.stderr"
  if ! gcloud storage ls --recursive "${GCS_SOURCE_PREFIX}/**" \
      > "$source_inventory" 2> "$source_inventory_error"; then
    record_failure "could not inventory the run-scoped candidate source objects"
    source_cleanup_safe=0
  fi
else
  source_cleanup_safe=0
  record_failure "candidate source retained because ownership or build state is unsafe"
fi

submit_intent_uri="$(sift_candidate_submit_intent_uri "$GCS_SOURCE_PREFIX")"
if [[ "$source_cleanup_safe" == "1" ]]; then
  source_delete_index=0
  while IFS= read -r source_uri; do
    [[ -n "$source_uri" ]] || continue
    case "$source_uri" in
      "$GCS_SOURCE_PREFIX"/*) ;;
      *)
        record_failure "candidate source inventory escaped the run prefix"
        source_cleanup_safe=0
        break
        ;;
    esac
    if [[ "$source_uri" == "$RESERVATION_URI" ]]; then
      continue
    fi
    if [[ "$source_uri" == "$submit_intent_uri" ]]; then
      if [[ "$live_submit_intent" == "1" ]]; then
        continue
      fi
      record_failure "an unexpected candidate submit intent appeared"
      source_cleanup_safe=0
      break
    fi
    source_delete_index=$((source_delete_index + 1))
    source_error="$work_root/source-delete-${source_delete_index}.stderr"
    if ! gcloud storage rm "$source_uri" --quiet \
        > /dev/null 2> "$source_error"; then
      record_failure "could not delete an owned candidate source object"
      source_cleanup_safe=0
      break
    fi
  done < "$source_inventory"
fi

if [[ "$source_cleanup_safe" == "1" ]]; then
  if ! verify_live_object "$RESERVATION_URI" "$reservation" \
      reservation-before-control-delete \
      || ! { [[ "$live_submit_intent" == "0" ]] \
        || verify_live_object "$submit_intent_uri" "$submit_intent" \
          submit-intent-before-control-delete; }; then
    record_failure "candidate control objects changed before final deletion"
    source_cleanup_safe=0
  fi
fi

if [[ "$source_cleanup_safe" == "1" && "$live_submit_intent" == "1" ]]; then
  source_error="$work_root/source-submit-intent-delete.stderr"
  submit_intent_generation="$(verify_live_object_generation \
    "$submit_intent_uri" "$submit_intent" submit-intent-final-delete)" || {
      record_failure "candidate submit intent changed before final deletion"
      source_cleanup_safe=0
    }
  if [[ "$source_cleanup_safe" == "1" ]] \
      && ! gcloud storage rm "$submit_intent_uri" \
        --if-generation-match="$submit_intent_generation" --quiet \
      > /dev/null 2> "$source_error"; then
    record_failure "could not delete the candidate submit intent"
    source_cleanup_safe=0
  fi
fi

# The reservation is the final remote object. Its exact live bytes are checked
# immediately before deletion, so a retry never uses a stale local claim to
# authorize destructive work.
if [[ "$source_cleanup_safe" == "1" ]]; then
  if ! reservation_generation="$(verify_live_object_generation \
      "$RESERVATION_URI" "$reservation" reservation-final-delete)"; then
    record_failure "candidate reservation changed before final deletion"
    source_cleanup_safe=0
  else
    source_error="$work_root/source-reservation-delete.stderr"
    if ! gcloud storage rm "$RESERVATION_URI" \
        --if-generation-match="$reservation_generation" --quiet \
        > /dev/null 2> "$source_error"; then
      record_failure "could not delete the candidate reservation"
      source_cleanup_safe=0
    fi
  fi
fi

if [[ "$source_cleanup_safe" == "1" ]]; then
  source_list="$work_root/source-final-list.txt"
  source_list_error="$work_root/source-final-list.stderr"
  source_list_status=0
  gcloud storage ls --recursive "${GCS_SOURCE_PREFIX}/**" \
    > "$source_list" 2> "$source_list_error" || source_list_status=$?
  if [[ "$source_list_status" == "0" && -s "$source_list" ]]; then
    record_failure "candidate source objects still exist after cleanup"
    source_cleanup_safe=0
  elif [[ "$source_list_status" != "0" ]] \
      && ! remote_prefix_is_empty_error "$source_list_error"; then
    record_failure "could not verify candidate source removal"
    source_cleanup_safe=0
  fi
fi

if [[ "$builds_safe" != "1" || "$image_cleanup_safe" != "1" \
  || "$source_cleanup_safe" != "1" || -s "$failures" ]]; then
  cp "$failures" "$recovery_dir/candidate-cleanup-failures.log"
  chmod 0600 "$recovery_dir/candidate-cleanup-failures.log"
  echo "candidate cleanup is incomplete; retry this exact command:" >&2
  printf '  %q %q\n' "$0" "$recovery_dir" >&2
  exit 1
fi

rm -f "$recovery_dir/candidate-cleanup-failures.log"
jq -n \
  --arg project_id "$PROJECT_ID" --arg region "$REGION" \
  --arg run_id "$RUN_ID" --arg git_sha "$GIT_SHA" \
  --arg cleaned_at "$(date -u +%Y-%m-%dT%H:%M:%SZ)" \
  --slurpfile builds "$build_receipt_array" '
    {
      schema:"axiom.gcp.sift.candidate-cleanup.v1",
      project_id:$project_id,
      region:$region,
      run_id:$run_id,
      git_sha:$git_sha,
      cloud_build_ids:([$builds[0][]?.id] | sort),
      status:"clean",
      cleaned_at:$cleaned_at
    }
  ' > "$recovery_dir/candidate-cleanup.json"
chmod 0600 "$recovery_dir/candidate-cleanup.json"
echo "Sift candidate cleanup passed: $recovery_dir/candidate-cleanup.json"
