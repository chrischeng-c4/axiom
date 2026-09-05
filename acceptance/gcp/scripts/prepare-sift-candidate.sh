#!/usr/bin/env bash
set -euo pipefail
umask 077

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ACCEPTANCE_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
REPO_ROOT="$(cd "$ACCEPTANCE_ROOT/../.." && pwd)"
source "$SCRIPT_DIR/source-prefix.sh"
source "$SCRIPT_DIR/sift-candidate.sh"

: "${PROJECT_ID:?PROJECT_ID is required}"
REGION="${REGION:-asia-east1}"
ARTIFACT_REGISTRY_REPOSITORY="${ARTIFACT_REGISTRY_REPOSITORY:-courier}"
RUN_ID="${RUN_ID:-$(date -u +%m%d%H%M%S)}"
CANDIDATE_DIR="${CANDIDATE_DIR:-${TMPDIR:-/tmp}/axiom-sift-candidates/${RUN_ID}}"
MAX_BUILD_SECONDS="${MAX_BUILD_SECONDS:-5400}"
GIT_SHA="$(git -c core.fsmonitor=false -C "$REPO_ROOT" rev-parse HEAD)"
REGISTRY="${REGION}-docker.pkg.dev/${PROJECT_ID}/${ARTIFACT_REGISTRY_REPOSITORY}"
GCS_SOURCE_PREFIX="${GCS_SOURCE_PREFIX:-gs://${PROJECT_ID}_cloudbuild/source/axiom-gcp-operator-${RUN_ID}}"

require() {
  command -v "$1" >/dev/null 2>&1 || {
    echo "required command not found: $1" >&2
    exit 1
  }
}

for command in awk chmod cmp cp docker find gcloud git jq mkdir mktemp mv \
  openssl rg sed sleep sort tar wc; do
  require "$command"
done
[[ "$RUN_ID" =~ ^[a-z0-9][a-z0-9-]{0,17}$ ]] || {
  echo "RUN_ID must be 1-18 lowercase letters, digits, or hyphens" >&2
  exit 1
}
[[ "$GIT_SHA" =~ ^[0-9a-f]{40}$ ]] || {
  echo "Git did not return one full commit SHA" >&2
  exit 1
}
[[ "$MAX_BUILD_SECONDS" =~ ^[1-9][0-9]*$ \
  && "$MAX_BUILD_SECONDS" -le 5400 ]] || {
  echo "MAX_BUILD_SECONDS must be 1-5400" >&2
  exit 1
}
[[ "$CANDIDATE_DIR" == /* && ! -e "$CANDIDATE_DIR" && ! -L "$CANDIDATE_DIR" ]] \
  || {
    echo "CANDIDATE_DIR must be a new absolute path" >&2
    exit 1
  }
FAILED_CANDIDATE_DIR="${CANDIDATE_DIR}.failed"
[[ ! -e "$FAILED_CANDIDATE_DIR" && ! -L "$FAILED_CANDIDATE_DIR" ]] || {
  echo "refusing to overwrite failed candidate evidence: $FAILED_CANDIDATE_DIR" >&2
  exit 1
}
SOURCE_BUCKET="$(validated_source_bucket "$GCS_SOURCE_PREFIX" "$RUN_ID")" || {
  echo "GCS_SOURCE_PREFIX must be exactly gs://BUCKET/source/axiom-gcp-operator-RUN_ID" >&2
  exit 1
}

acquisition_id="$(openssl rand -hex 16)"
[[ "$acquisition_id" =~ ^[0-9a-f]{32}$ ]] || {
  echo "could not create a candidate acquisition ID" >&2
  exit 1
}
# The tag is an acquisition-scoped cleanup handle. A later preparation that
# reuses the same Git SHA and run ID receives a different tag, so a delayed
# cleanup cannot delete the later acquisition's tag.
IMAGE_TAG="${GIT_SHA}-${RUN_ID}-${acquisition_id}"
reservation_uri="$(sift_candidate_reservation_uri "$GCS_SOURCE_PREFIX")"
submit_intent_uri="$(sift_candidate_submit_intent_uri "$GCS_SOURCE_PREFIX")"
candidate_parent="$(dirname "$CANDIDATE_DIR")"
mkdir -p "$candidate_parent"
[[ -d "$candidate_parent" && ! -L "$candidate_parent" ]] || {
  echo "candidate parent must be a real directory" >&2
  exit 1
}
if ! mkdir "$CANDIDATE_DIR"; then
  echo "another process already owns the candidate directory" >&2
  exit 1
fi
chmod 0700 "$CANDIDATE_DIR"
receipts="$CANDIDATE_DIR"
claim_created=1
local_claim_tmp="$(mktemp "$receipts/.candidate-local-claim.XXXXXX")"
jq -n \
  --arg acquisition_id "$acquisition_id" \
  --arg run_id "$RUN_ID" \
  --arg created_at "$(date -u +%Y-%m-%dT%H:%M:%SZ)" '
    {
      schema:"axiom.gcp.sift.candidate-local-claim.v1",
      acquisition_id:$acquisition_id,
      run_id:$run_id,
      created_at:$created_at
    }
  ' > "$local_claim_tmp"
chmod 0600 "$local_claim_tmp"
mv "$local_claim_tmp" "$receipts/candidate-local-claim.json"

work_root="$(mktemp -d "${TMPDIR:-/tmp}/axiom-sift-candidate.XXXXXX")"
chmod 0700 "$work_root"
source_dir="$work_root/source"
target_dir="$work_root/target"
source_archive="$work_root/sift-candidate-${GIT_SHA}.tar.gz"
source_sha=""
build_id=""
submit_intent_published=false
submit_started=false
reservation_attempted=0
candidate_complete=0
preserve_work_root=0

write_failure_receipt() {
  local exit_code="$1"
  local submit_response_received=false
  local temporary
  [[ -z "$build_id" ]] || submit_response_received=true
  temporary="$(mktemp "$receipts/.candidate-preparation-failure.XXXXXX")" \
    || return 1
  if ! jq -n \
      --arg project_id "$PROJECT_ID" \
      --arg region "$REGION" \
      --arg artifact_registry_repository "$ARTIFACT_REGISTRY_REPOSITORY" \
      --arg run_id "$RUN_ID" \
      --arg git_sha "$GIT_SHA" \
      --arg source_bundle_sha256 "$source_sha" \
      --arg acquisition_id "$acquisition_id" \
      --arg reservation_uri "$reservation_uri" \
      --arg build_id "$build_id" \
      --arg registry "$REGISTRY" \
      --arg image_tag "$IMAGE_TAG" \
      --arg source_prefix "$GCS_SOURCE_PREFIX" \
      --argjson submit_intent_published "$submit_intent_published" \
      --argjson submit_started "$submit_started" \
      --argjson submit_response_received "$submit_response_received" \
      --argjson exit_code "$exit_code" \
      --arg failed_at "$(date -u +%Y-%m-%dT%H:%M:%SZ)" '
        {
          schema:"axiom.gcp.sift.candidate-preparation-failure.v2",
          project_id:$project_id,
          region:$region,
          artifact_registry_repository:$artifact_registry_repository,
          run_id:$run_id,
          git_sha:$git_sha,
          source_bundle_sha256:$source_bundle_sha256,
          acquisition_id:$acquisition_id,
          reservation_uri:$reservation_uri,
          cloud_build_id:$build_id,
          registry:$registry,
          image_tag:$image_tag,
          source_prefix:$source_prefix,
          submit_intent_published:$submit_intent_published,
          submit_started:$submit_started,
          submit_response_received:$submit_response_received,
          exit_code:$exit_code,
          failed_at:$failed_at
        }
      ' > "$temporary"; then
    find "$temporary" -delete >/dev/null 2>&1 || true
    return 1
  fi
  chmod 0600 "$temporary" || return 1
  mv "$temporary" "$receipts/candidate-preparation-failure.json"
}

download_exact_object() {
  local uri="$1"
  local expected="$2"
  local label="$3"
  local downloaded="$work_root/${label}.live"
  if ! gcloud storage cp "$uri" "$downloaded" \
      > /dev/null 2> "$work_root/${label}.download.stderr"; then
    return 1
  fi
  [[ -f "$downloaded" && ! -L "$downloaded" ]] \
    && cmp -s "$expected" "$downloaded"
}

publish_create_only_object() {
  local input="$1"
  local uri="$2"
  local label="$3"
  local upload_status=0
  gcloud storage cp "$input" "$uri" --if-generation-match=0 \
    > /dev/null 2> "$work_root/${label}.upload.stderr" || upload_status=$?
  if download_exact_object "$uri" "$input" "$label"; then
    return 0
  fi
  if [[ "$upload_status" == "0" ]]; then
    echo "uploaded $label could not be read back exactly" >&2
  else
    echo "$label is owned by another acquisition or could not be verified" >&2
  fi
  return 1
}

cleanup_work_root() {
  local ec=$?
  local cleanup_status=0
  local recovery_dir="$CANDIDATE_DIR"
  trap - EXIT INT TERM
  set +e
  if [[ "$candidate_complete" != "1" && "$claim_created" == "1" ]]; then
    [[ "$ec" != "0" ]] || ec=1
    if [[ "$reservation_attempted" == "1" ]]; then
      write_failure_receipt "$ec" || {
        echo "could not update candidate recovery receipt" >&2
        ec=1
      }
      if [[ -n "$build_id" ]]; then
        gcloud builds describe "$build_id" --project="$PROJECT_ID" \
          --region="$REGION" --format=json \
          > "$receipts/cloud-build-failure-final.json" 2>/dev/null
      fi
      if [[ -f "$source_archive" && ! -L "$source_archive" \
        && ! -e "$receipts/candidate-source.tar.gz" ]]; then
        cp "$source_archive" "$receipts/candidate-source.tar.gz" || ec=1
      fi
      chmod -R go-rwx "$receipts" >/dev/null 2>&1 || ec=1
      if [[ ! -e "$FAILED_CANDIDATE_DIR" && ! -L "$FAILED_CANDIDATE_DIR" ]] \
          && mv "$CANDIDATE_DIR" "$FAILED_CANDIDATE_DIR"; then
        recovery_dir="$FAILED_CANDIDATE_DIR"
      else
        echo "failed candidate evidence remains at $CANDIDATE_DIR" >&2
        recovery_dir="$CANDIDATE_DIR"
        ec=1
      fi
      CANDIDATE_CLEANUP_DISCOVERY_ATTEMPTS="${CANDIDATE_CLEANUP_DISCOVERY_ATTEMPTS:-6}" \
      CANDIDATE_CLEANUP_DISCOVERY_DELAY_SECONDS="${CANDIDATE_CLEANUP_DISCOVERY_DELAY_SECONDS:-10}" \
      CANDIDATE_CLEANUP_WAIT_ATTEMPTS="${CANDIDATE_CLEANUP_WAIT_ATTEMPTS:-60}" \
      CANDIDATE_CLEANUP_WAIT_DELAY_SECONDS="${CANDIDATE_CLEANUP_WAIT_DELAY_SECONDS:-5}" \
        bash "$SCRIPT_DIR/cleanup-sift-candidate.sh" "$recovery_dir" \
        || cleanup_status=$?
      if [[ "$cleanup_status" == "0" ]]; then
        echo "candidate preparation failed; remote candidate artifacts were cleaned" >&2
      else
        echo "candidate preparation failed and remote cleanup is incomplete" >&2
        printf 'retry: bash %q %q\n' \
          "$SCRIPT_DIR/cleanup-sift-candidate.sh" "$recovery_dir" >&2
        ec=1
      fi
      echo "recovery evidence: $recovery_dir" >&2
    elif jq -e --arg acquisition_id "$acquisition_id" \
        '.acquisition_id == $acquisition_id' \
        "$receipts/candidate-local-claim.json" >/dev/null 2>&1; then
      find "$CANDIDATE_DIR" -depth -delete >/dev/null 2>&1 || {
        echo "could not remove the unused candidate directory" >&2
        ec=1
      }
    else
      echo "candidate local claim changed; refusing local cleanup" >&2
      ec=1
    fi
  fi
  if [[ "$preserve_work_root" != "1" \
    && "$work_root" == "${TMPDIR:-/tmp}/axiom-sift-candidate."* \
    && -d "$work_root" && ! -L "$work_root" ]]; then
    find "$work_root" -depth -delete >/dev/null 2>&1 || true
  fi
  exit "$ec"
}
trap cleanup_work_root EXIT
trap 'exit 130' INT TERM

mkdir -p "$source_dir" "$target_dir"
git -c core.fsmonitor=false -C "$REPO_ROOT" status --porcelain=v1 \
  > "$receipts/source-git-status.txt"
if [[ -s "$receipts/source-git-status.txt" ]]; then
  echo "refusing a candidate from a dirty tree" >&2
  cat "$receipts/source-git-status.txt" >&2
  exit 1
fi
git -c core.fsmonitor=false -C "$REPO_ROOT" archive \
  --format=tar.gz --output="$source_archive" "$GIT_SHA"
chmod 0400 "$source_archive"
source_sha="$(sift_candidate_file_sha256 "$source_archive")"
source_bytes="$(wc -c < "$source_archive" | tr -d ' ')"
[[ "$source_sha" =~ ^[0-9a-f]{64}$ && "$source_bytes" =~ ^[1-9][0-9]*$ ]] \
  || {
    echo "could not bind the candidate source archive" >&2
    exit 1
  }
tar -xzf "$source_archive" -C "$source_dir"
chmod -R a-w "$source_dir"
jq -n \
  --arg git_sha "$GIT_SHA" \
  --arg source_archive "git-archive:${GIT_SHA}" \
  --arg source_bundle_sha256 "$source_sha" \
  --argjson source_bundle_bytes "$source_bytes" '
    {
      git_sha:$git_sha,
      source_archive:$source_archive,
      source_bundle_sha256:$source_bundle_sha256,
      source_bundle_bytes:$source_bundle_bytes
    }
  ' > "$receipts/candidate-source.json"

echo ">> candidate gate from clean Git archive"
if ! SIFT_REPO_ROOT="$source_dir" \
    CARGO_TARGET_DIR="$target_dir" \
    SIFT_SOURCE_REVISION="$GIT_SHA" \
    bash "$source_dir/apps/sift/test.sh" --candidate \
    > "$receipts/candidate-gate.log" 2>&1; then
  cat "$receipts/candidate-gate.log" >&2
  echo "the fixed candidate gate failed" >&2
  exit 1
fi
gate_completed_at="$(date -u +%Y-%m-%dT%H:%M:%SZ)"
jq -n \
  --arg git_sha "$GIT_SHA" \
  --arg source_bundle_sha256 "$source_sha" \
  --arg completed_at "$gate_completed_at" '
    {
      schema:"axiom.gcp.sift.candidate-gate.v1",
      git_sha:$git_sha,
      source_bundle_sha256:$source_bundle_sha256,
      entrypoint:"apps/sift/test.sh --candidate",
      completed_at:$completed_at,
      status:"passed"
    }
  ' > "$receipts/candidate-gate.json"

echo ">> candidate cloud preflight"
gcloud artifacts repositories describe "$ARTIFACT_REGISTRY_REPOSITORY" \
  --project="$PROJECT_ID" --location="$REGION" --format=json \
  > "$receipts/preexisting-artifact-registry.json"
gcloud storage buckets describe "gs://${SOURCE_BUCKET}" --project="$PROJECT_ID" \
  --format=json > "$receipts/preexisting-cloud-build-source-bucket.json"
if ! gcloud storage ls --recursive "gs://${SOURCE_BUCKET}" \
    > "$receipts/preexisting-cloud-build-source-objects.txt" \
    2> "$work_root/preexisting-cloud-build-source-objects.stderr"; then
  echo "could not inventory the pre-existing Cloud Build source bucket" >&2
  cat "$work_root/preexisting-cloud-build-source-objects.stderr" >&2
  exit 1
fi
if rg -Fx "$GCS_SOURCE_PREFIX" \
    "$receipts/preexisting-cloud-build-source-objects.txt" >/dev/null \
    || rg -F "${GCS_SOURCE_PREFIX}/" \
      "$receipts/preexisting-cloud-build-source-objects.txt" >/dev/null; then
  echo "refusing to reuse Cloud Build source prefix: $GCS_SOURCE_PREFIX" >&2
  exit 1
fi
write_source_prefix_receipt \
  "$receipts/source-prefix.json" "$PROJECT_ID" "$RUN_ID" "$GCS_SOURCE_PREFIX"

for image in sift rig sift-acceptance-runner; do
  inventory="$receipts/preexisting-${image}-images.json"
  inventory_stderr="$work_root/preexisting-${image}-images.stderr"
  if gcloud artifacts docker images list "$REGISTRY/$image" \
      --project="$PROJECT_ID" --include-tags --format=json \
      > "$inventory" 2> "$inventory_stderr"; then
    :
  elif rg -F "NOT_FOUND" "$inventory_stderr" >/dev/null; then
    printf '[]\n' > "$inventory"
  else
    echo "could not inventory existing $image images" >&2
    cat "$inventory_stderr" >&2
    exit 1
  fi
  jq -e 'type == "array"' "$inventory" >/dev/null || exit 1
  if jq -e --arg tag "$IMAGE_TAG" \
      'any(.[]; ((.tags // []) | index($tag)) != null)' "$inventory" >/dev/null; then
    echo "refusing to overwrite existing image tag: $REGISTRY/$image:$IMAGE_TAG" >&2
    exit 1
  fi
done

jq -n \
  --arg project_id "$PROJECT_ID" \
  --arg region "$REGION" \
  --arg artifact_registry_repository "$ARTIFACT_REGISTRY_REPOSITORY" \
  --arg run_id "$RUN_ID" \
  --arg git_sha "$GIT_SHA" \
  --arg source_bundle_sha256 "$source_sha" \
  --arg acquisition_id "$acquisition_id" \
  --arg registry "$REGISTRY" \
  --arg image_tag "$IMAGE_TAG" \
  --arg source_prefix "$GCS_SOURCE_PREFIX" \
  --arg reservation_uri "$reservation_uri" \
  --arg created_at "$(date -u +%Y-%m-%dT%H:%M:%SZ)" \
  --slurpfile sift "$receipts/preexisting-sift-images.json" \
  --slurpfile rig "$receipts/preexisting-rig-images.json" \
  --slurpfile runner "$receipts/preexisting-sift-acceptance-runner-images.json" '
    {
      schema:"axiom.gcp.sift.candidate-reservation.v1",
      project_id:$project_id,
      region:$region,
      artifact_registry_repository:$artifact_registry_repository,
      run_id:$run_id,
      git_sha:$git_sha,
      source_bundle_sha256:$source_bundle_sha256,
      acquisition_id:$acquisition_id,
      registry:$registry,
      image_tag:$image_tag,
      source_prefix:$source_prefix,
      reservation_uri:$reservation_uri,
      created_at:$created_at,
      preexisting_images:{sift:$sift[0],rig:$rig[0],sift_acceptance_runner:$runner[0]}
    }
  ' > "$receipts/candidate-reservation.json"
verify_sift_candidate_reservation "$receipts/candidate-reservation.json" || {
  echo "candidate reservation is invalid" >&2
  exit 1
}
write_failure_receipt 1
reservation_attempted=1
publish_create_only_object \
  "$receipts/candidate-reservation.json" "$reservation_uri" reservation || exit 1

echo ">> Cloud Build: Sift and Rig from the same candidate archive"
jq -n \
  --arg project_id "$PROJECT_ID" --arg region "$REGION" \
  --arg run_id "$RUN_ID" --arg git_sha "$GIT_SHA" \
  --arg source_bundle_sha256 "$source_sha" --arg registry "$REGISTRY" \
  --arg image_tag "$IMAGE_TAG" --arg source_prefix "$GCS_SOURCE_PREFIX" \
  --arg acquisition_id "$acquisition_id" \
  --arg submitted_at "$(date -u +%Y-%m-%dT%H:%M:%SZ)" '
    {
      schema:"axiom.gcp.sift.candidate-submit-intent.v1",
      project_id:$project_id,
      region:$region,
      run_id:$run_id,
      git_sha:$git_sha,
      source_bundle_sha256:$source_bundle_sha256,
      registry:$registry,
      image_tag:$image_tag,
      source_prefix:$source_prefix,
      acquisition_id:$acquisition_id,
      submitted_at:$submitted_at
    }
  ' > "$receipts/candidate-submit-intent.json"
verify_sift_candidate_submit_intent \
  "$receipts/candidate-submit-intent.json" \
  "$receipts/candidate-reservation.json" || {
  echo "candidate submit intent is invalid" >&2
  exit 1
}
publish_create_only_object \
  "$receipts/candidate-submit-intent.json" "$submit_intent_uri" submit-intent \
  || exit 1
submit_intent_published=true
write_failure_receipt 1
submit_started=true
write_failure_receipt 1
submit_output="$work_root/cloud-build-submit-id.txt"
if ! gcloud builds submit "$source_archive" \
  --async \
  --project="$PROJECT_ID" \
  --region="$REGION" \
  --config="$source_dir/acceptance/gcp/cloudbuild.sift-mvp.yaml" \
  --gcs-source-staging-dir="$GCS_SOURCE_PREFIX" \
  --substitutions="_REGISTRY=$REGISTRY,_TAG=$IMAGE_TAG,_RUN_ID=$RUN_ID,_GIT_SHA=$GIT_SHA,_SOURCE_BUNDLE_SHA256=$source_sha,_CANDIDATE_ACQUISITION_ID=$acquisition_id" \
  --format='value(id)' > "$submit_output" \
  2> "$receipts/cloud-build-submit.stderr"; then
  echo "Cloud Build submit response was not received; recovery will inventory the acquisition tag" >&2
  exit 1
fi
build_id="$(sed -n '1p' "$submit_output")"
[[ "$build_id" =~ ^[A-Za-z0-9-]{1,128}$ ]] || {
  echo "Cloud Build did not return a build ID" >&2
  exit 1
}
write_failure_receipt 1
printf '%s\n' "$build_id" > "$receipts/cloud-build-id.txt"
gcloud builds describe "$build_id" --project="$PROJECT_ID" --region="$REGION" \
  --format=json > "$receipts/cloud-build-submit.json"
source_bucket="$(jq -er '.source.storageSource.bucket' \
  "$receipts/cloud-build-submit.json")"
source_object="$(jq -er '.source.storageSource.object' \
  "$receipts/cloud-build-submit.json")"
source_uri="$(validated_source_object_uri \
  "$GCS_SOURCE_PREFIX" "$RUN_ID" "$source_bucket" "$source_object")" || {
  echo "Cloud Build staged source outside the run-scoped prefix" >&2
  exit 1
}
source_generation="$(jq -er '
  .source.storageSource.generation | select(type == "string" and test("^[1-9][0-9]*$"))
' "$receipts/cloud-build-submit.json")" || {
  echo "Cloud Build did not identify an immutable source generation" >&2
  exit 1
}
# An asynchronous build can be queued before output provenance is available.
# The final receipt must contain it; any value already present must match now.
jq -e '.sourceProvenance.resolvedStorageSource == null
  or .sourceProvenance.resolvedStorageSource == .source.storageSource' \
  "$receipts/cloud-build-submit.json" >/dev/null || {
  echo "Cloud Build resolved a different source version" >&2
  exit 1
}
versioned_source_uri="${source_uri}#${source_generation}"
gcloud storage objects describe "$versioned_source_uri" --format=json \
  > "$receipts/cloud-build-source-object.json"
jq -e --arg generation "$source_generation" '.generation == $generation' \
  "$receipts/cloud-build-source-object.json" >/dev/null || {
  echo "GCS source receipt does not match the Cloud Build generation" >&2
  exit 1
}
staged_source="$work_root/cloud-build-staged-source.tar.gz"
gcloud storage cp "$versioned_source_uri" "$staged_source" >/dev/null
staged_sha="$(sift_candidate_file_sha256 "$staged_source")"
[[ "$staged_sha" == "$source_sha" ]] || {
  echo "Cloud Build staged source does not match the candidate archive" >&2
  exit 1
}
jq -n \
  --arg build_id "$build_id" \
  --arg git_sha "$GIT_SHA" \
  --arg source_uri "$source_uri" \
  --arg source_bundle_sha256 "$source_sha" \
  --arg staged_source_sha256 "$staged_sha" '
    {
      build_id:$build_id,
      git_sha:$git_sha,
      source_uri:$source_uri,
      source_bundle_sha256:$source_bundle_sha256,
      staged_source_sha256:$staged_source_sha256
    }
  ' > "$receipts/cloud-build-source-binding.json"

build_deadline=$((SECONDS + MAX_BUILD_SECONDS))
while true; do
  build_status="$(gcloud builds describe "$build_id" --project="$PROJECT_ID" \
    --region="$REGION" --format='value(status)')"
  printf '%s %s\n' "$(date -u +%Y-%m-%dT%H:%M:%SZ)" "$build_status" \
    >> "$receipts/cloud-build-status.log"
  case "$build_status" in
    SUCCESS) break ;;
    FAILURE|INTERNAL_ERROR|TIMEOUT|CANCELLED|EXPIRED)
      echo "Cloud Build ended with $build_status" >&2
      exit 1
      ;;
  esac
  if (( SECONDS >= build_deadline )); then
    gcloud builds cancel "$build_id" --project="$PROJECT_ID" \
      --region="$REGION" --quiet >/dev/null 2>&1 || true
    echo "Cloud Build exceeded ${MAX_BUILD_SECONDS}s" >&2
    exit 1
  fi
  sleep 10
done
gcloud builds describe "$build_id" --project="$PROJECT_ID" --region="$REGION" \
  --format=json > "$receipts/cloud-build-final.json"

resolve_digest() {
  local image="$1"
  local digest
  digest="$(gcloud artifacts docker images describe "$REGISTRY/$image:$IMAGE_TAG" \
    --project="$PROJECT_ID" --format='value(image_summary.digest)')"
  [[ "$digest" == sha256:* ]] || return 1
  printf '%s@%s\n' "$REGISTRY/$image" "$digest"
}

sift_image="$(resolve_digest sift)"
rig_image="$(resolve_digest rig)"
acceptance_runner_image="$(resolve_digest sift-acceptance-runner)"
jq -n \
  --arg sift "$sift_image" \
  --arg rig "$rig_image" \
  --arg acceptance_runner "$acceptance_runner_image" \
  '{sift:$sift,rig:$rig,acceptance_runner:$acceptance_runner}' \
  > "$receipts/images.json"
jq -e \
  --arg git_sha "$GIT_SHA" \
  --arg run_id "$RUN_ID" \
  --arg source_sha "$source_sha" \
  --arg acquisition_id "$acquisition_id" \
  --arg registry "$REGISTRY" \
  --arg image_tag "$IMAGE_TAG" \
  --arg source_bucket "$source_bucket" \
  --arg source_object "$source_object" \
  --arg sift_name "$REGISTRY/sift:$IMAGE_TAG" \
  --arg sift_digest "${sift_image##*@}" \
  --arg rig_name "$REGISTRY/rig:$IMAGE_TAG" \
  --arg rig_digest "${rig_image##*@}" \
  --arg runner_name "$REGISTRY/sift-acceptance-runner:$IMAGE_TAG" \
  --arg runner_digest "${acceptance_runner_image##*@}" '
    .status == "SUCCESS"
    and .substitutions._GIT_SHA == $git_sha
    and .substitutions._RUN_ID == $run_id
    and .substitutions._SOURCE_BUNDLE_SHA256 == $source_sha
    and .substitutions._CANDIDATE_ACQUISITION_ID == $acquisition_id
    and .substitutions._REGISTRY == $registry
    and .substitutions._TAG == $image_tag
    and .source.storageSource.bucket == $source_bucket
    and .source.storageSource.object == $source_object
    and ((.tags // []) | index("sift-mvp") != null)
    and ((.tags // []) | index("axiom-run-" + $run_id) != null)
    and ((.tags // []) | index("axiom-source-" + $source_sha) != null)
    and ((.tags // []) | index("axiom-acquisition-" + $acquisition_id) != null)
    and any(.results.images[]?; .name == $sift_name and .digest == $sift_digest)
    and any(.results.images[]?; .name == $rig_name and .digest == $rig_digest)
    and any(.results.images[]?; .name == $runner_name and .digest == $runner_digest)
  ' "$receipts/cloud-build-final.json" >/dev/null || {
  echo "Cloud Build final receipt does not bind all three image digests" >&2
  exit 1
}
download_exact_object \
  "$reservation_uri" "$receipts/candidate-reservation.json" reservation-final || {
  echo "candidate reservation changed before completion" >&2
  exit 1
}
download_exact_object \
  "$submit_intent_uri" "$receipts/candidate-submit-intent.json" submit-intent-final || {
  echo "candidate submit intent changed before completion" >&2
  exit 1
}

file_hashes='{}'
while IFS= read -r name; do
  digest="$(sift_candidate_file_sha256 "$receipts/$name")"
  file_hashes="$(jq -c --arg name "$name" --arg digest "$digest" \
    '. + {($name):$digest}' <<<"$file_hashes")"
done < <(sift_candidate_required_files)
completed_at="$(date -u +%Y-%m-%dT%H:%M:%SZ)"
jq -n \
  --arg project_id "$PROJECT_ID" \
  --arg region "$REGION" \
  --arg artifact_registry_repository "$ARTIFACT_REGISTRY_REPOSITORY" \
  --arg registry "$REGISTRY" \
  --arg run_id "$RUN_ID" \
  --arg git_sha "$GIT_SHA" \
  --arg image_tag "$IMAGE_TAG" \
  --arg acquisition_id "$acquisition_id" \
  --arg reservation_uri "$reservation_uri" \
  --arg source_prefix "$GCS_SOURCE_PREFIX" \
  --arg source_bundle_sha256 "$source_sha" \
  --arg source_object_uri "$source_uri" \
  --arg cloud_build_id "$build_id" \
  --arg sift_image "$sift_image" \
  --arg rig_image "$rig_image" \
  --arg acceptance_runner_image "$acceptance_runner_image" \
  --arg completed_at "$completed_at" \
  --argjson file_sha256 "$file_hashes" '
    {
      schema:"axiom.gcp.sift.candidate.v1",
      project_id:$project_id,
      region:$region,
      artifact_registry_repository:$artifact_registry_repository,
      registry:$registry,
      run_id:$run_id,
      git_sha:$git_sha,
      image_tag:$image_tag,
      acquisition_id:$acquisition_id,
      reservation_uri:$reservation_uri,
      source_prefix:$source_prefix,
      source_bundle_sha256:$source_bundle_sha256,
      source_object_uri:$source_object_uri,
      cloud_build_id:$cloud_build_id,
      sift_image:$sift_image,
      rig_image:$rig_image,
      acceptance_runner_image:$acceptance_runner_image,
      completed_at:$completed_at,
      file_sha256:$file_sha256
    }
  ' > "$receipts/candidate.json"
chmod 0600 "$receipts"/*
verify_sift_candidate_directory "$receipts" || {
  echo "the completed candidate receipt did not validate" >&2
  exit 1
}
find "$receipts/candidate-local-claim.json" \
  "$receipts/candidate-preparation-failure.json" \
  "$receipts/cloud-build-submit.stderr" -delete >/dev/null 2>&1 || true
candidate_complete=1
printf 'candidate: %s\n' "$CANDIDATE_DIR/candidate.json"
printf 'sift: %s\n' "$sift_image"
printf 'rig: %s\n' "$rig_image"
printf 'acceptance runner: %s\n' "$acceptance_runner_image"
