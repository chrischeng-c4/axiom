#!/usr/bin/env bash
set -euo pipefail

: "${PROJECT_ID:?PROJECT_ID is required}"
: "${REGION:?REGION is required}"
: "${GKE_ZONE:?GKE_ZONE is required}"
: "${RUN_ID:?RUN_ID is required}"
: "${STATE_DIR:?STATE_DIR is required}"
: "${ACCEPTANCE_ROOT:?ACCEPTANCE_ROOT is required}"
: "${REGISTRY:?REGISTRY is required}"
: "${IMAGE_TAG:?IMAGE_TAG is required}"
: "${GCS_SOURCE_PREFIX:?GCS_SOURCE_PREFIX is required}"
: "${EVIDENCE_DIR:?EVIDENCE_DIR is required}"
ARTIFACT_REGISTRY_REPOSITORY="${ARTIFACT_REGISTRY_REPOSITORY:-courier}"
PERSISTENT_CLUSTER_NAME="${PERSISTENT_CLUSTER_NAME:-axiom-operator-acceptance}"
ACCEPTANCE_APPS="${ACCEPTANCE_APPS:-lumen sift}"
KUBECONFIG="${KUBECONFIG:-$STATE_DIR/kubeconfig}"
TERRAFORM_ENVIRONMENT_DIR="${TERRAFORM_ENVIRONMENT_DIR:-$STATE_DIR/environment}"
ACCEPTANCE_LOCAL_CLAIM_ROOT="${ACCEPTANCE_LOCAL_CLAIM_ROOT:-${TMPDIR:-/tmp}/axiom-gcp-operator-claims}"
source "$ACCEPTANCE_ROOT/scripts/source-prefix.sh"
source "$ACCEPTANCE_ROOT/scripts/acceptance-lock.sh"
source "$ACCEPTANCE_ROOT/scripts/process-tree.sh"
source "$ACCEPTANCE_ROOT/scripts/sift-candidate.sh"
source "$ACCEPTANCE_ROOT/scripts/sift-container-boundary.sh"
source "$ACCEPTANCE_ROOT/scripts/sift-evidence-secrets.sh"
source "$ACCEPTANCE_ROOT/scripts/kubernetes-ownership.sh"
ACCEPTANCE_RUN_OWNER_HANDOFF_NONCE="${ACCEPTANCE_RUN_OWNER_HANDOFF_NONCE:-}"
AXIOM_GCP_ACCEPTANCE_CONTAINER_ROLE="${AXIOM_GCP_ACCEPTANCE_CONTAINER_ROLE:-}"
ACCEPTANCE_CONTAINER_OWNER_RECEIPT="${ACCEPTANCE_CONTAINER_OWNER_RECEIPT:-}"
ACCEPTANCE_CONTAINER_STOP_RECEIPT="${ACCEPTANCE_CONTAINER_STOP_RECEIPT:-}"
ACCEPTANCE_CONTAINER_CLEANUP_NONCE_FILE="${ACCEPTANCE_CONTAINER_CLEANUP_NONCE_FILE:-}"
ACCEPTANCE_CONTROLLER_IMAGE="${ACCEPTANCE_CONTROLLER_IMAGE:-}"
SIFT_CANDIDATE_DIR="${SIFT_CANDIDATE_DIR:-}"
export KUBECONFIG
case "$ACCEPTANCE_APPS" in
  "lumen sift") acceptance_mode="lumen-sift" ;;
  "lumen auth") acceptance_mode="lumen-auth" ;;
  "sift") acceptance_mode="sift" ;;
  "tape") acceptance_mode="tape" ;;
  *)
    echo "ACCEPTANCE_APPS must be exactly 'lumen sift' (default), 'lumen auth', 'sift', or 'tape'" >&2
    exit 1
    ;;
esac

sift_candidate_directory_for_cleanup() {
  [[ "$acceptance_mode" == "sift" ]] || return 1
  if [[ -n "$SIFT_CANDIDATE_DIR" ]]; then
    printf '%s\n' "$SIFT_CANDIDATE_DIR"
  elif [[ -e "$EVIDENCE_DIR/candidate.json" \
    || -L "$EVIDENCE_DIR/candidate.json" ]]; then
    printf '%s\n' "$EVIDENCE_DIR"
  else
    return 1
  fi
}

verify_sift_candidate_cleanup_binding() {
  local candidate_directory="$1"

  verify_sift_candidate_directory "$candidate_directory" || return 1
  jq -e \
    --arg project_id "$PROJECT_ID" \
    --arg region "$REGION" \
    --arg run_id "$RUN_ID" \
    --arg registry "$REGISTRY" \
    --arg image_tag "$IMAGE_TAG" \
    --arg source_prefix "$GCS_SOURCE_PREFIX" '
      .project_id == $project_id
      and .region == $region
      and .run_id == $run_id
      and .registry == $registry
      and .image_tag == $image_tag
      and .source_prefix == $source_prefix
      and .reservation_uri == ($source_prefix + "/candidate-reservation.json")
    ' "$candidate_directory/candidate.json" >/dev/null
}

verified_live_sift_candidate_object_generation() {
  local candidate_directory="$1"
  local uri="$2"
  local expected="$3"
  local phase="$4"
  local scratch generation status=0

  verify_sift_candidate_cleanup_binding "$candidate_directory" || return 1
  [[ -f "$expected" && ! -L "$expected" ]] || return 1
  scratch="$(mktemp -d "$STATE_DIR/.sift-object-${phase}.XXXXXX")" \
    || return 1
  if ! gcloud storage objects describe "$uri" --format=json \
      > "$scratch/metadata.json" 2> "$scratch/metadata.stderr"; then
    status=1
  else
    generation="$(jq -er \
      '.generation | tostring | select(test("^[1-9][0-9]*$"))' \
      "$scratch/metadata.json")" || status=1
  fi
  if [[ "$status" == "0" ]] \
      && { ! gcloud storage cp "$uri" "$scratch/object" \
        > /dev/null 2> "$scratch/object.stderr" \
        || [[ ! -f "$scratch/object" || -L "$scratch/object" ]] \
        || ! cmp -s "$expected" "$scratch/object"; }; then
    status=1
  fi
  find "$scratch" -depth -delete >/dev/null 2>&1 || status=1
  [[ "$status" == "0" ]] || return 1
  printf '%s\n' "$generation"
}

verify_live_sift_candidate_control_objects() {
  local candidate_directory="$1"
  local phase="$2"
  local reservation_uri submit_intent_uri status=0

  verify_sift_candidate_cleanup_binding "$candidate_directory" || return 1
  reservation_uri="$(jq -er '.reservation_uri' \
    "$candidate_directory/candidate.json")" || return 1
  submit_intent_uri="$(sift_candidate_submit_intent_uri "$GCS_SOURCE_PREFIX")"
  if ! verified_live_sift_candidate_object_generation \
      "$candidate_directory" "$reservation_uri" \
      "$candidate_directory/candidate-reservation.json" \
      "${phase}-reservation" >/dev/null; then
    echo "live Sift candidate reservation is missing or changed" >&2
    status=1
  fi
  if ! verified_live_sift_candidate_object_generation \
      "$candidate_directory" "$submit_intent_uri" \
      "$candidate_directory/candidate-submit-intent.json" \
      "${phase}-submit-intent" >/dev/null; then
    echo "live Sift candidate submit intent is missing or changed" >&2
    status=1
  fi
  [[ "$status" == "0" ]]
}

verify_live_sift_candidate_reservation() {
  local candidate_directory="$1"
  local phase="$2"
  local reservation_uri

  verify_sift_candidate_cleanup_binding "$candidate_directory" || return 1
  reservation_uri="$(jq -er '.reservation_uri' \
    "$candidate_directory/candidate.json")" || return 1
  if ! verified_live_sift_candidate_object_generation \
      "$candidate_directory" "$reservation_uri" \
      "$candidate_directory/candidate-reservation.json" \
      "$phase" >/dev/null; then
    echo "live Sift candidate reservation is missing or changed" >&2
    return 1
  fi
}

sift_candidate_source_prefix_is_empty() {
  local phase="$1"
  local output error status=0
  output="$(mktemp "$STATE_DIR/.sift-source-${phase}.XXXXXX")" || return 1
  error="${output}.stderr"
  gcloud storage ls --recursive "${GCS_SOURCE_PREFIX}/**" \
    > "$output" 2> "$error" || status=$?
  if [[ "$status" == "0" ]]; then
    if [[ ! -s "$output" ]]; then
      rm -f "$output" "$error"
      return 0
    fi
  elif grep -Eiq '(matched no URLs|no URLs matched|not[ _-]?found|404)' \
      "$error"; then
    rm -f "$output" "$error"
    return 0
  fi
  rm -f "$output" "$error"
  return 1
}

sift_candidate_reservation_is_absent() {
  local candidate_directory="$1"
  local reservation_uri scratch error status=0
  verify_sift_candidate_cleanup_binding "$candidate_directory" || return 1
  reservation_uri="$(jq -er '.reservation_uri' \
    "$candidate_directory/candidate.json")" || return 1
  scratch="$(mktemp "$STATE_DIR/.sift-reservation-absence.XXXXXX")" || return 1
  error="${scratch}.stderr"
  gcloud storage cp "$reservation_uri" "$scratch" \
    > /dev/null 2> "$error" || status=$?
  if [[ "$status" != "0" ]] \
      && grep -Eiq '(matched no URLs|no URLs matched|not[ _-]?found|404)' \
        "$error"; then
    rm -f "$scratch" "$error"
    return 0
  fi
  rm -f "$scratch" "$error"
  return 1
}

sift_candidate_submit_intent_is_absent() {
  local candidate_directory="$1"
  local submit_intent_uri scratch error status=0
  verify_sift_candidate_cleanup_binding "$candidate_directory" || return 1
  submit_intent_uri="$(sift_candidate_submit_intent_uri "$GCS_SOURCE_PREFIX")"
  scratch="$(mktemp "$STATE_DIR/.sift-submit-intent-absence.XXXXXX")" \
    || return 1
  error="${scratch}.stderr"
  gcloud storage cp "$submit_intent_uri" "$scratch" \
    > /dev/null 2> "$error" || status=$?
  if [[ "$status" != "0" ]] \
      && grep -Eiq '(matched no URLs|no URLs matched|not[ _-]?found|404)' \
        "$error"; then
    rm -f "$scratch" "$error"
    return 0
  fi
  rm -f "$scratch" "$error"
  return 1
}

sift_candidate_source_has_only_reservation() {
  local phase="$1"
  local output error status=0 count only_uri reservation_uri
  output="$(mktemp "$STATE_DIR/.sift-source-${phase}.XXXXXX")" || return 1
  error="${output}.stderr"
  reservation_uri="${GCS_SOURCE_PREFIX}/candidate-reservation.json"
  gcloud storage ls --recursive "${GCS_SOURCE_PREFIX}/**" \
    > "$output" 2> "$error" || status=$?
  if [[ "$status" == "0" ]]; then
    count="$(awk 'NF { count += 1 } END { print count + 0 }' "$output")"
    only_uri="$(awk 'NF { print; exit }' "$output")"
    if [[ "$count" == "1" && "$only_uri" == "$reservation_uri" ]]; then
      rm -f "$output" "$error"
      return 0
    fi
  fi
  rm -f "$output" "$error"
  return 1
}

verify_acceptance_process_group_quiescent() {
  local dedicated_process_record
  if [[ -e "$STATE_DIR/process-scan-unsafe.txt" ]]; then
    echo "the acceptance process-group scan is incomplete; refusing cleanup" >&2
    return 1
  fi
  if [[ -e "$STATE_DIR/watchdog-ready.txt" \
    && ! -f "$STATE_DIR/watchdog-descendants.txt" ]]; then
    echo "the completed process-group record is missing; refusing cleanup" >&2
    return 1
  fi
  if recorded_processes_have_live_member \
      "$STATE_DIR/watchdog-descendants.txt"; then
    echo "the recorded acceptance process group is still active; refusing cleanup" >&2
    return 1
  fi
  for dedicated_process_record in \
      "$STATE_DIR/watchdog-process.txt" \
      "$STATE_DIR/run-log-process.txt"; do
    if recorded_processes_have_live_member "$dedicated_process_record"; then
      echo "a dedicated acceptance process is still active or unverifiable; refusing cleanup" >&2
      return 1
    fi
  done
}

authorize_acceptance_cleanup() {
  local owner_receipt acquisition_id owner_pid owner_pgid owner_start_token
  local current_start_token current_start_status
  local expected_handoff_digest provided_handoff_digest

  owner_receipt="$(acceptance_run_claim_path \
    "$ACCEPTANCE_LOCAL_CLAIM_ROOT" "$PROJECT_ID" "$RUN_ID" "$acceptance_mode")" \
    || return 1

  verify_acceptance_run_owner_identity \
    "$owner_receipt" "$PROJECT_ID" "$RUN_ID" "$acceptance_mode" \
    "$STATE_DIR" "$EVIDENCE_DIR" || {
    echo "cleanup has no valid local run-owner receipt" >&2
    return 1
  }
  acquisition_id="$(jq -er '.acquisition_id' "$owner_receipt")" || return 1
  owner_pid="$(jq -er '.owner_pid' "$owner_receipt")" || return 1
  owner_pgid="$(jq -er '.owner_pgid' "$owner_receipt")" || return 1
  owner_start_token="$(jq -er '.owner_start_token' "$owner_receipt")" || return 1
  expected_handoff_digest="$(jq -er '.cleanup_handoff_digest' "$owner_receipt")" \
    || return 1
  current_start_token=""
  if current_start_token="$(process_start_token "$owner_pid")"; then
    current_start_status=0
  else
    current_start_status=$?
  fi

  if [[ -n "$ACCEPTANCE_RUN_OWNER_HANDOFF_NONCE" ]]; then
    provided_handoff_digest="$(
      printf '%s' "$ACCEPTANCE_RUN_OWNER_HANDOFF_NONCE" \
        | openssl dgst -sha256 \
        | awk '{print $NF}'
    )" || return 1
    [[ "$ACCEPTANCE_RUN_OWNER_HANDOFF_NONCE" =~ ^[0-9a-f]{64}$ \
      && "$provided_handoff_digest" == "$expected_handoff_digest" \
      && "$PPID" == "$owner_pid" \
      && "$current_start_status" == "0" \
      && "$current_start_token" == "$owner_start_token" ]] || {
      echo "cleanup was not handed off by the recorded run owner" >&2
      return 1
    }
  else
    case "$current_start_status" in
      0)
        if [[ "$current_start_token" == "$owner_start_token" ]]; then
          echo "the recorded acceptance run is still active; refusing recovery cleanup" >&2
          return 1
        fi
        ;;
      1) ;;
      *)
        echo "cannot verify the live acceptance run process generation; refusing recovery cleanup" >&2
        return 1
        ;;
    esac
  fi
  verify_acceptance_process_group_quiescent || return 1
  authorized_run_acquisition_id="$acquisition_id"
}

authorize_contained_sift_cleanup() {
  local owner_receipt cleanup_nonce controller_from_candidate

  [[ "$acceptance_mode" == "sift" \
    && "$AXIOM_GCP_ACCEPTANCE_CONTAINER_ROLE" == "cleanup" ]] || return 1
  verify_sift_candidate_cleanup_binding "$SIFT_CANDIDATE_DIR" || {
    echo "contained cleanup has no valid candidate for this exact run" >&2
    return 1
  }
  controller_from_candidate="$(jq -er '.acceptance_runner_image' \
    "$SIFT_CANDIDATE_DIR/candidate.json")" || return 1
  [[ "$ACCEPTANCE_CONTROLLER_IMAGE" == "$controller_from_candidate" ]] || {
    echo "contained cleanup does not use the candidate controller digest" >&2
    return 1
  }
  [[ -f "$ACCEPTANCE_CONTAINER_CLEANUP_NONCE_FILE" \
    && ! -L "$ACCEPTANCE_CONTAINER_CLEANUP_NONCE_FILE" \
    && "$(wc -l < "$ACCEPTANCE_CONTAINER_CLEANUP_NONCE_FILE" | tr -d ' ')" == "1" ]] \
    || {
      echo "contained cleanup nonce is missing or unsafe" >&2
      return 1
    }
  cleanup_nonce="$(<"$ACCEPTANCE_CONTAINER_CLEANUP_NONCE_FILE")"
  authorize_sift_container_cleanup \
    "$ACCEPTANCE_CONTAINER_OWNER_RECEIPT" \
    "$ACCEPTANCE_CONTAINER_STOP_RECEIPT" \
    "$cleanup_nonce" "$ACCEPTANCE_CONTROLLER_IMAGE" \
    "$STATE_DIR" "$EVIDENCE_DIR" || {
    echo "the exact Sift run container has not been proven stopped" >&2
    return 1
  }

  owner_receipt="$(acceptance_run_claim_path \
    "$ACCEPTANCE_LOCAL_CLAIM_ROOT" "$PROJECT_ID" "$RUN_ID" "$acceptance_mode")" \
    || return 1
  verify_acceptance_run_owner_identity \
    "$owner_receipt" "$PROJECT_ID" "$RUN_ID" "$acceptance_mode" \
    "$STATE_DIR" "$EVIDENCE_DIR" || {
    echo "contained cleanup has no valid run-owner receipt" >&2
    return 1
  }
  authorized_run_acquisition_id="$(jq -er '.acquisition_id' "$owner_receipt")" \
    || return 1
}

authorized_run_acquisition_id=""
if [[ "$AXIOM_GCP_ACCEPTANCE_CONTAINER_ROLE" == "cleanup" ]]; then
  authorize_contained_sift_cleanup || exit 1
else
  authorize_acceptance_cleanup || exit 1
fi

state="$STATE_DIR/environment.tfstate"
tf_data="$STATE_DIR/.terraform-environment"
cleanup_failures=0
cloud_build_cleanup_failed=0
image_cleanup_failed=0
source_cleanup_authorized=0
acceptance_lock_held=0
cleanup_session_id=""
kubernetes_cleanup_authorized=0
mkdir -p "$EVIDENCE_DIR/kubernetes"

record_cleanup_failure() {
  cleanup_failures=$((cleanup_failures + 1))
  echo "cleanup failure: $*" >&2
}

if ! sift_remove_ephemeral_evidence_secrets "$EVIDENCE_DIR"; then
  record_cleanup_failure "could not remove ephemeral Sift acceptance credentials"
fi

capture_failure_evidence() {
  kubectl get deployment,statefulset,cronjob,job,pod,pvc -A -o json \
    > "$EVIDENCE_DIR/kubernetes/workloads-before-cleanup.json" 2>/dev/null || true
  if [[ "$acceptance_mode" == "tape" ]]; then
    kubectl logs -n tape-system deployment/tape-operator --tail=500 --request-timeout=15s \
      > "$EVIDENCE_DIR/kubernetes/tape-operator.log" 2>&1 || true
  elif [[ "$acceptance_mode" == "lumen-auth" ]]; then
    kubectl logs -n lumen-system deployment/lumen-operator --tail=500 --request-timeout=15s \
      > "$EVIDENCE_DIR/kubernetes/lumen-operator.log" 2>&1 || true
  elif [[ "$acceptance_mode" == "sift" ]]; then
    kubectl logs -n sift-system deployment/sift-operator --tail=500 --request-timeout=15s \
      > "$EVIDENCE_DIR/kubernetes/sift-operator.log" 2>&1 || true
  else
    kubectl logs -n lumen-system deployment/lumen-operator --tail=500 --request-timeout=15s \
      > "$EVIDENCE_DIR/kubernetes/lumen-operator.log" 2>&1 || true
    kubectl logs -n sift-system deployment/sift-operator --tail=500 --request-timeout=15s \
      > "$EVIDENCE_DIR/kubernetes/sift-operator.log" 2>&1 || true
  fi
}

verify_terminal_cleanup_receipt() {
  local receipt="$EVIDENCE_DIR/cleanup.json"
  [[ -f "$receipt" && ! -L "$receipt" ]] || return 1
  jq -e \
    --arg project_id "$PROJECT_ID" \
    --arg region "$REGION" \
    --arg gke_zone "$GKE_ZONE" \
    --arg run_id "$RUN_ID" '
      type == "object"
      and .schema == "axiom.gcp.operator.cleanup.v1"
      and .project_id == $project_id
      and .region == $region
      and .gke_zone == $gke_zone
      and .run_id == $run_id
      and .status == "clean"
      and .preserved.artifact_registry == true
      and .preserved.preexisting_apis == true
    ' "$receipt" >/dev/null
}

cloud_build_status() {
  local build_id="$1"
  gcloud builds describe "$build_id" --project="$PROJECT_ID" \
    --region="$REGION" --format='value(status)'
}

cloud_build_is_terminal() {
  case "$1" in
    SUCCESS|FAILURE|INTERNAL_ERROR|TIMEOUT|CANCELLED|EXPIRED) return 0 ;;
    *) return 1 ;;
  esac
}

cancel_and_wait_cloud_build() {
  local build_id="$1"
  local status attempt
  status="$(cloud_build_status "$build_id")" || {
    echo "could not read Cloud Build $build_id during cleanup" >&2
    return 1
  }
  if cloud_build_is_terminal "$status"; then
    printf '%s %s\n' "$build_id" "$status" >> "$EVIDENCE_DIR/cloud-build-cleanup.log"
    return 0
  fi
  case "$status" in
    QUEUED|PENDING|WORKING)
      if ! gcloud builds cancel "$build_id" --project="$PROJECT_ID" \
        --region="$REGION" --quiet >/dev/null; then
        status="$(cloud_build_status "$build_id")" || return 1
        cloud_build_is_terminal "$status" || {
          echo "could not cancel active Cloud Build $build_id (status $status)" >&2
          return 1
        }
      fi
      ;;
    *)
      echo "Cloud Build $build_id has unknown non-terminal status: $status" >&2
      return 1
      ;;
  esac
  for ((attempt = 1; attempt <= 120; attempt++)); do
    status="$(cloud_build_status "$build_id")" || return 1
    if cloud_build_is_terminal "$status"; then
      printf '%s %s\n' "$build_id" "$status" >> "$EVIDENCE_DIR/cloud-build-cleanup.log"
      return 0
    fi
    sleep 5
  done
  echo "Cloud Build $build_id did not reach a terminal state after cancellation" >&2
  return 1
}

verify_cleanup_cloud_build_receipt() {
  local build_id="$1"
  local build_receipt="$2"
  local candidate_directory="$3"
  local source_bucket source_object

  if [[ -n "$candidate_directory" ]]; then
    verify_sift_candidate_build_receipt \
      "$candidate_directory/candidate.json" "$build_receipt"
    return
  fi

  jq -e \
    --arg id "$build_id" --arg run_id "$RUN_ID" \
    --arg registry "$REGISTRY" --arg image_tag "$IMAGE_TAG" '
      .id == $id
      and .substitutions._RUN_ID == $run_id
      and .substitutions._REGISTRY == $registry
      and .substitutions._TAG == $image_tag
      and ((.tags // []) | index("axiom-run-" + $run_id) != null)
      and (.source.storageSource.bucket | type) == "string"
      and (.source.storageSource.object | type) == "string"
    ' "$build_receipt" >/dev/null || return 1
  source_bucket="$(jq -er '.source.storageSource.bucket' "$build_receipt")" \
    || return 1
  source_object="$(jq -er '.source.storageSource.object' "$build_receipt")" \
    || return 1
  validated_source_object_uri \
    "$GCS_SOURCE_PREFIX" "$RUN_ID" "$source_bucket" "$source_object" \
    >/dev/null
}

stop_run_cloud_builds() {
  local ids="$STATE_DIR/cloud-build-ids.txt"
  local sorted="${ids}.sorted"
  local final_ids="${ids}.final"
  local inventory="$STATE_DIR/cloud-build-run-inventory.json"
  local acquisition_inventory="$STATE_DIR/cloud-build-acquisition-inventory.json"
  local receipts="$STATE_DIR/cloud-build-cleanup-receipts"
  local expected_build_id=""
  local candidate_acquisition_id=""
  local candidate_directory=""
  local sift_candidate_cleanup=0
  local ownership_failed=0
  local build_receipt post_build_receipt
  local failed=0
  : > "$ids"
  if [[ -f "$STATE_DIR/cloud-build-id.txt" ]]; then
    sed -n '1p' "$STATE_DIR/cloud-build-id.txt" >> "$ids"
  fi
  if ! gcloud builds list --project="$PROJECT_ID" --region="$REGION" \
      --filter="tags=axiom-run-${RUN_ID}" --format=json > "$inventory"; then
    echo "could not inventory run-tagged Cloud Builds during cleanup" >&2
    return 1
  fi
  jq -e '
    type == "array"
    and all(.[];
      (.id | type) == "string"
      and (.id | test("^[A-Za-z0-9-]{1,128}$")))
  ' "$inventory" >/dev/null || {
    echo "run-tagged Cloud Build inventory is invalid" >&2
    return 1
  }
  jq -r '.[].id' "$inventory" >> "$ids"

  if candidate_directory="$(sift_candidate_directory_for_cleanup)"; then
    verify_sift_candidate_cleanup_binding "$candidate_directory" || {
      echo "cannot bind Cloud Build cleanup to this exact Sift candidate run" >&2
      return 1
    }
    verify_live_sift_candidate_control_objects \
      "$candidate_directory" before-build-cleanup || {
      echo "cannot prove live ownership of the Sift candidate build" >&2
      return 1
    }
    sift_candidate_cleanup=1
  fi
  if [[ "$sift_candidate_cleanup" == "1" ]]; then
    expected_build_id="$(jq -er '.cloud_build_id' \
      "$candidate_directory/candidate.json")" || return 1
    candidate_acquisition_id="$(jq -er '.acquisition_id' \
      "$candidate_directory/candidate.json")" || return 1
    if [[ -f "$STATE_DIR/cloud-build-id.txt" \
      && "$(sed -n '1p' "$STATE_DIR/cloud-build-id.txt")" != "$expected_build_id" ]]; then
      echo "recorded Cloud Build ID does not match the Sift candidate" >&2
      return 1
    fi
    if ! gcloud builds list --project="$PROJECT_ID" --region="$REGION" \
        --filter="tags=axiom-acquisition-${candidate_acquisition_id}" \
        --format=json > "$acquisition_inventory"; then
      echo "could not inventory acquisition-tagged Cloud Builds during cleanup" >&2
      return 1
    fi
    if ! jq -e --arg expected "$expected_build_id" '
        type == "array"
        and ([.[].id] | sort) == [$expected]
      ' "$inventory" >/dev/null \
        || ! jq -e --arg expected "$expected_build_id" '
          type == "array"
          and ([.[].id] | sort) == [$expected]
        ' "$acquisition_inventory" >/dev/null; then
      echo "Cloud Build inventory contains an unknown or missing Sift candidate build" >&2
      return 1
    fi
    printf '%s\n' "$expected_build_id" >> "$ids"
  fi
  sort -u "$ids" > "$sorted"
  mv "$sorted" "$ids"
  mkdir -p "$receipts"

  # Validate every candidate before any cancellation. A run tag is searchable
  # metadata. It is not proof that this run owns the build.
  while IFS= read -r build_id; do
    [[ -n "$build_id" ]] || continue
    [[ "$build_id" =~ ^[A-Za-z0-9-]{1,128}$ ]] || {
      echo "Cloud Build inventory contains an invalid ID" >&2
      return 1
    }
    build_receipt="$receipts/${build_id}.json"
    if ! gcloud builds describe "$build_id" --project="$PROJECT_ID" \
        --region="$REGION" --format=json > "$build_receipt"; then
      echo "could not describe Cloud Build $build_id during cleanup" >&2
      ownership_failed=1
      continue
    fi
    if ! verify_cleanup_cloud_build_receipt \
        "$build_id" "$build_receipt" "$candidate_directory"; then
      echo "Cloud Build $build_id does not match this exact cleanup run" >&2
      ownership_failed=1
      continue
    fi
  done < "$ids"

  if [[ "$ownership_failed" == "1" ]]; then
    echo "one or more Cloud Builds could not be proven owned; none were cancelled" >&2
    return 1
  fi

  while IFS= read -r build_id; do
    [[ -n "$build_id" ]] || continue
    if ! cancel_and_wait_cloud_build "$build_id"; then
      failed=1
    fi
  done < "$ids"
  [[ "$failed" == "0" ]] || return 1

  # A build may change state while cancellation runs. Re-read every full
  # resource and prove that its immutable identity did not change.
  while IFS= read -r build_id; do
    [[ -n "$build_id" ]] || continue
    post_build_receipt="$receipts/${build_id}.post.json"
    if ! gcloud builds describe "$build_id" --project="$PROJECT_ID" \
        --region="$REGION" --format=json > "$post_build_receipt" \
        || ! verify_cleanup_cloud_build_receipt \
          "$build_id" "$post_build_receipt" "$candidate_directory"; then
      echo "Cloud Build $build_id changed identity during cleanup" >&2
      return 1
    fi
  done < "$ids"

  if ! gcloud builds list --project="$PROJECT_ID" --region="$REGION" \
      --filter="tags=axiom-run-${RUN_ID}" --format=json \
      > "$STATE_DIR/cloud-build-run-final.json" \
      || ! jq -e '
        type == "array"
        and all(.[].id;
          type == "string" and test("^[A-Za-z0-9-]{1,128}$"))
      ' "$STATE_DIR/cloud-build-run-final.json" >/dev/null; then
    echo "could not complete the final run-tagged Cloud Build inventory" >&2
    return 1
  fi
  jq -r '.[].id' "$STATE_DIR/cloud-build-run-final.json" \
    | sort -u > "$final_ids"
  if ! cmp -s "$ids" "$final_ids"; then
    echo "run-tagged Cloud Build inventory changed during cleanup" >&2
    return 1
  fi

  if [[ "$sift_candidate_cleanup" == "1" ]]; then
    if ! gcloud builds list --project="$PROJECT_ID" --region="$REGION" \
        --filter="tags=axiom-acquisition-${candidate_acquisition_id}" \
        --format=json > "$STATE_DIR/cloud-build-acquisition-final.json" \
        || ! jq -e --arg expected "$expected_build_id" '
          type == "array" and ([.[].id] | sort) == [$expected]
        ' "$STATE_DIR/cloud-build-acquisition-final.json" >/dev/null; then
      echo "candidate acquisition Cloud Build inventory changed during cleanup" >&2
      return 1
    fi
  fi
  return "$failed"
}

verify_current_sift_candidate_build_inventory() {
  local candidate_directory="$1"
  local phase="$2"
  local expected_build_id acquisition_id scratch status=0

  verify_sift_candidate_cleanup_binding "$candidate_directory" || return 1
  expected_build_id="$(jq -er '.cloud_build_id' \
    "$candidate_directory/candidate.json")" || return 1
  acquisition_id="$(jq -er '.acquisition_id' \
    "$candidate_directory/candidate.json")" || return 1
  scratch="$(mktemp -d "$STATE_DIR/.sift-build-${phase}.XXXXXX")" || return 1

  if ! gcloud builds list --project="$PROJECT_ID" --region="$REGION" \
      --filter="tags=axiom-run-${RUN_ID}" --format=json \
      > "$scratch/run.json" \
      || ! jq -e --arg expected "$expected_build_id" '
        type == "array" and ([.[].id] | sort) == [$expected]
      ' "$scratch/run.json" >/dev/null; then
    echo "current run-tagged Cloud Build inventory is not the candidate build" >&2
    status=1
  fi
  if ! gcloud builds list --project="$PROJECT_ID" --region="$REGION" \
      --filter="tags=axiom-acquisition-${acquisition_id}" --format=json \
      > "$scratch/acquisition.json" \
      || ! jq -e --arg expected "$expected_build_id" '
        type == "array" and ([.[].id] | sort) == [$expected]
      ' "$scratch/acquisition.json" >/dev/null; then
    echo "current acquisition-tagged Cloud Build inventory is not the candidate build" >&2
    status=1
  fi
  if ! gcloud builds describe "$expected_build_id" --project="$PROJECT_ID" \
      --region="$REGION" --format=json > "$scratch/build.json" \
      || ! verify_sift_candidate_build_receipt \
        "$candidate_directory/candidate.json" "$scratch/build.json"; then
    echo "current Cloud Build does not match the immutable candidate receipt" >&2
    status=1
  fi
  find "$scratch" -depth -delete >/dev/null 2>&1 || status=1
  [[ "$status" == "0" ]]
}

kubectl_delete_with_preconditions() {
  local raw_path="$1"
  local uid="$2"
  local resource_version="$3"

  jq -cn \
    --arg uid "$uid" \
    --arg resource_version "$resource_version" '
      {
        apiVersion: "v1",
        kind: "DeleteOptions",
        preconditions: {
          uid: $uid,
          resourceVersion: $resource_version
        }
      }
    ' | kubectl delete --raw="$raw_path" -f - >/dev/null
}

acceptance_lock_is_absent() {
  local error_file
  error_file="$(mktemp "${TMPDIR:-/tmp}/sift-acceptance-lock-get.XXXXXX")"
  if kubectl get lease "$(acceptance_lock_name)" \
      --namespace "$(acceptance_lock_namespace)" -o json \
      >/dev/null 2>"$error_file"; then
    rm -f "$error_file"
    return 1
  fi
  if grep -Eiq '(^|[^[:alpha:]])(not[ _-]?found|404)([^[:alpha:]]|$)' \
      "$error_file"; then
    rm -f "$error_file"
    return 0
  fi
  echo "could not determine whether the shared GKE Lease was deleted:" >&2
  cat "$error_file" >&2
  rm -f "$error_file"
  return 1
}

release_acceptance_lock() {
  local receipt="$EVIDENCE_DIR/acceptance-lock.json"
  local release_receipt="$EVIDENCE_DIR/acceptance-lock-release.json"
  local acquisition_id resource uid resource_version raw_path

  acquisition_id="$(jq -er \
    '.acquisition_id | strings | select(test("^[0-9a-f]{32}$"))' "$receipt")" \
    || return 1
  resource="$(kubectl get lease "$(acceptance_lock_name)" \
    --namespace "$(acceptance_lock_namespace)" -o json)" || return 1
  verify_acceptance_cleanup_session_json \
    "$resource" "$PROJECT_ID" "$RUN_ID" "$acceptance_mode" \
    "$acquisition_id" "$cleanup_session_id" || return 1
  verify_acceptance_lock_receipt \
    "$receipt" "$resource" "$PROJECT_ID" "$RUN_ID" "$acceptance_mode" || return 1

  uid="$(jq -er '.uid | strings | select(length > 0)' "$receipt")" || return 1
  resource_version="$(jq -er \
    '.resource_version | strings | select(length > 0)' "$receipt")" || return 1
  raw_path="/apis/coordination.k8s.io/v1/namespaces/$(acceptance_lock_namespace)/leases/$(acceptance_lock_name)"
  if ! kubectl_delete_with_preconditions \
      "$raw_path" "$uid" "$resource_version"; then
    acceptance_lock_is_absent || return 1
  elif ! kubectl wait --for=delete "lease/$(acceptance_lock_name)" \
      --namespace "$(acceptance_lock_namespace)" --timeout=60s >/dev/null; then
    acceptance_lock_is_absent || return 1
  fi
  write_acceptance_lock_release_receipt "$release_receipt" "$receipt"
}

assert_acceptance_cleanup_session() {
  local receipt="$EVIDENCE_DIR/acceptance-lock.json"
  local acquisition_id resource

  [[ "$cleanup_session_id" =~ ^[0-9a-f]{32}$ ]] || return 1
  acquisition_id="$(jq -er \
    '.acquisition_id | strings | select(test("^[0-9a-f]{32}$"))' "$receipt")" \
    || return 1
  resource="$(kubectl get lease "$(acceptance_lock_name)" \
    --namespace "$(acceptance_lock_namespace)" -o json)" || return 1
  verify_acceptance_cleanup_session_json \
    "$resource" "$PROJECT_ID" "$RUN_ID" "$acceptance_mode" \
    "$acquisition_id" "$cleanup_session_id" || return 1
  verify_acceptance_lock_receipt \
    "$receipt" "$resource" "$PROJECT_ID" "$RUN_ID" "$acceptance_mode"
}

delete_sift_instance() {
  local namespace="$1"
  local name="$2"
  local expected_finalizer="service-k8s.axiom.dev/sift-operator-cluster-children"
  local resource binding binding_name owner_uid owner_resource_version error_file
  local current_uid current_resource_version deletion_timestamp
  local binding_uid binding_resource_version binding_path resource_path
  local current_finalizers remaining_finalizers finalizer_patch

  [[ "$namespace" =~ ^[a-z0-9]([-a-z0-9.]*[a-z0-9])?$ \
    && "$name" =~ ^[a-z0-9]([-a-z0-9.]*[a-z0-9])?$ ]] || {
    echo "unsafe Sift namespace or name: $namespace/$name" >&2
    return 1
  }
  resource_path="/apis/sift.axiom.dev/v1alpha1/namespaces/${namespace}/sifts/${name}"
  error_file="$(mktemp "${TMPDIR:-/tmp}/sift-cleanup-resource.XXXXXX")"
  if ! resource="$(kubectl get sift.sift.axiom.dev "$name" \
      --namespace "$namespace" -o json 2>"$error_file")"; then
    if grep -Eiq '(^|[^[:alpha:]])(not[ _-]?found|404)([^[:alpha:]]|$)' \
        "$error_file"; then
      rm -f "$error_file"
      return 0
    fi
    echo "could not inspect Sift instance $namespace/$name before cleanup:" >&2
    cat "$error_file" >&2
    rm -f "$error_file"
    return 1
  fi
  rm -f "$error_file"
  owner_uid="$(jq -er '.metadata.uid | strings | select(length > 0)' \
    <<<"$resource")" || {
    echo "Sift instance $namespace/$name has no owner UID; refusing finalizer removal" >&2
    return 1
  }
  owner_resource_version="$(jq -er \
    '.metadata.resourceVersion | strings | select(length > 0)' <<<"$resource")" || {
    echo "Sift instance $namespace/$name has no resourceVersion; refusing deletion" >&2
    return 1
  }
  if kubectl_delete_with_preconditions \
      "$resource_path" "$owner_uid" "$owner_resource_version" 2>/dev/null \
      && kubectl wait --for=delete "sift.sift.axiom.dev/$name" \
        --namespace "$namespace" --timeout=180s >/dev/null 2>&1; then
    return 0
  fi

  # A transport error does not prove that the API server accepted deletion.
  # Re-read the object and enter the fallback only for the same terminating CR.
  error_file="$(mktemp "${TMPDIR:-/tmp}/sift-cleanup-resource.XXXXXX")"
  if ! resource="$(kubectl get sift.sift.axiom.dev "$name" \
      --namespace "$namespace" -o json 2>"$error_file")"; then
    if grep -Eiq '(^|[^[:alpha:]])(not[ _-]?found|404)([^[:alpha:]]|$)' \
        "$error_file"; then
      rm -f "$error_file"
      return 0
    fi
    echo "could not confirm deletion of Sift instance $namespace/$name:" >&2
    cat "$error_file" >&2
    rm -f "$error_file"
    return 1
  fi
  rm -f "$error_file"
  current_uid="$(jq -er '.metadata.uid | strings | select(length > 0)' \
    <<<"$resource")" || return 1
  deletion_timestamp="$(jq -er \
    '.metadata.deletionTimestamp | strings | select(length > 0)' \
    <<<"$resource")" || {
    echo "Sift instance $namespace/$name is not terminating; refusing finalizer fallback" >&2
    return 1
  }
  [[ "$current_uid" == "$owner_uid" ]] || {
    echo "Sift instance $namespace/$name was replaced; refusing finalizer fallback" >&2
    return 1
  }
  jq -e --arg finalizer "$expected_finalizer" \
    '(.metadata.finalizers // []) | index($finalizer) != null' \
    <<<"$resource" >/dev/null || {
    echo "Sift instance $namespace/$name lacks the expected finalizer" >&2
    return 1
  }

  # The operator normally removes its cluster-scoped child first. If it is no
  # longer able to finish, delete only the binding whose exact name and labels
  # bind it to this CR UID. Never remove the finalizer while that grant remains.
  binding_name="sift.${namespace}.${name}.auth-delegator"
  binding_path="/apis/rbac.authorization.k8s.io/v1/clusterrolebindings/${binding_name}"
  error_file="$(mktemp "${TMPDIR:-/tmp}/sift-cleanup-binding.XXXXXX")"
  if binding="$(kubectl get clusterrolebinding "$binding_name" \
      -o json 2>"$error_file")"; then
    rm -f "$error_file"
    jq -e \
      --arg binding_name "$binding_name" \
      --arg namespace "$namespace" \
      --arg name "$name" \
      --arg owner_uid "$owner_uid" '
        .metadata.name == $binding_name
        and .metadata.labels["app.kubernetes.io/name"] == "sift"
        and .metadata.labels["app.kubernetes.io/instance"] == $name
        and .metadata.labels["app.kubernetes.io/component"] == "auth-delegation"
        and .metadata.labels["sift.axiom.dev/owner-namespace"] == $namespace
        and .metadata.labels["service-k8s.axiom.dev/owner-uid"] == $owner_uid
      ' <<<"$binding" >/dev/null || {
      echo "Sift auth binding $binding_name is not owned by $namespace/$name; refusing deletion" >&2
      return 1
    }
    binding_uid="$(jq -er '.metadata.uid | strings | select(length > 0)' \
      <<<"$binding")" || {
      echo "Sift auth binding $binding_name has no UID; refusing deletion" >&2
      return 1
    }
    binding_resource_version="$(jq -er \
      '.metadata.resourceVersion | strings | select(length > 0)' \
      <<<"$binding")" || {
      echo "Sift auth binding $binding_name has no resourceVersion; refusing deletion" >&2
      return 1
    }
    if ! kubectl_delete_with_preconditions \
        "$binding_path" "$binding_uid" "$binding_resource_version" 2>/dev/null; then
      error_file="$(mktemp "${TMPDIR:-/tmp}/sift-cleanup-binding.XXXXXX")"
      if kubectl get clusterrolebinding "$binding_name" \
          -o json >/dev/null 2>"$error_file"; then
        echo "Sift auth binding $binding_name changed before deletion; refusing finalizer removal" >&2
        rm -f "$error_file"
        return 1
      elif grep -Eiq '(^|[^[:alpha:]])(not[ _-]?found|404)([^[:alpha:]]|$)' \
          "$error_file"; then
        rm -f "$error_file"
      else
        echo "could not confirm deletion of Sift auth binding $binding_name:" >&2
        cat "$error_file" >&2
        rm -f "$error_file"
        return 1
      fi
    elif ! kubectl wait --for=delete "clusterrolebinding/$binding_name" \
        --timeout=180s >/dev/null 2>&1; then
      echo "Sift auth binding $binding_name did not finish deletion" >&2
      return 1
    fi
  elif grep -Eiq '(^|[^[:alpha:]])(not[ _-]?found|404)([^[:alpha:]]|$)' \
      "$error_file"; then
    rm -f "$error_file"
  else
    echo "could not inspect Sift auth binding $binding_name before finalizer removal:" >&2
    cat "$error_file" >&2
    rm -f "$error_file"
    return 1
  fi

  # Re-read immediately before the patch. A JSON Patch test makes the UID,
  # resourceVersion, deletion state, and finalizer list one atomic condition.
  error_file="$(mktemp "${TMPDIR:-/tmp}/sift-cleanup-resource.XXXXXX")"
  if ! resource="$(kubectl get sift.sift.axiom.dev "$name" \
      --namespace "$namespace" -o json 2>"$error_file")"; then
    if grep -Eiq '(^|[^[:alpha:]])(not[ _-]?found|404)([^[:alpha:]]|$)' \
        "$error_file"; then
      rm -f "$error_file"
      return 0
    fi
    echo "could not re-read Sift instance $namespace/$name before finalizer removal:" >&2
    cat "$error_file" >&2
    rm -f "$error_file"
    return 1
  fi
  rm -f "$error_file"
  current_uid="$(jq -er '.metadata.uid | strings | select(length > 0)' \
    <<<"$resource")" || return 1
  current_resource_version="$(jq -er \
    '.metadata.resourceVersion | strings | select(length > 0)' <<<"$resource")" \
    || return 1
  deletion_timestamp="$(jq -er \
    '.metadata.deletionTimestamp | strings | select(length > 0)' <<<"$resource")" \
    || return 1
  [[ "$current_uid" == "$owner_uid" ]] || {
    echo "Sift instance $namespace/$name was replaced before finalizer removal" >&2
    return 1
  }
  current_finalizers="$(jq -c '.metadata.finalizers // []' <<<"$resource")"
  jq -e --arg finalizer "$expected_finalizer" \
    'index($finalizer) != null' <<<"$current_finalizers" >/dev/null || {
    echo "Sift instance $namespace/$name no longer has the expected finalizer" >&2
    return 1
  }
  remaining_finalizers="$(jq -c --arg finalizer "$expected_finalizer" \
    '[.[] | select(. != $finalizer)]' <<<"$current_finalizers")"
  finalizer_patch="$(jq -cn \
    --arg uid "$current_uid" \
    --arg resource_version "$current_resource_version" \
    --arg deletion_timestamp "$deletion_timestamp" \
    --argjson current_finalizers "$current_finalizers" \
    --argjson remaining_finalizers "$remaining_finalizers" '
      [
        {op:"test", path:"/metadata/uid", value:$uid},
        {op:"test", path:"/metadata/resourceVersion", value:$resource_version},
        {op:"test", path:"/metadata/deletionTimestamp", value:$deletion_timestamp},
        {op:"test", path:"/metadata/finalizers", value:$current_finalizers},
        {op:"replace", path:"/metadata/finalizers", value:$remaining_finalizers}
      ]
    ')"
  echo "Sift instance $namespace/$name did not finalize; removing its orphaned Sift finalizer" >&2
  kubectl patch sift.sift.axiom.dev "$name" --namespace "$namespace" \
    --type=json -p "$finalizer_patch" >/dev/null
  kubectl wait --for=delete "sift.sift.axiom.dev/$name" --namespace "$namespace" \
    --timeout=60s >/dev/null
}

delete_allow_not_found() {
  local label="$1"
  shift
  local error_file
  error_file="$(mktemp "${TMPDIR:-/tmp}/sift-cleanup-delete.XXXXXX")"
  if "$@" >/dev/null 2>"$error_file"; then
    rm -f "$error_file"
    return 0
  fi
  if grep -Eiq '(^|[^[:alpha:]])(not[ _-]?found|404)([^[:alpha:]]|$)|matched no (objects|URLs)|no URLs matched' \
      "$error_file"; then
    rm -f "$error_file"
    return 0
  fi
  echo "failed to delete ${label}:" >&2
  cat "$error_file" >&2
  rm -f "$error_file"
  return 1
}

delete_run_image() {
  local image="$1"
  local tagged="$REGISTRY/$image:$IMAGE_TAG"
  local digest_ref digest inventory inventory_contains_digest
  local live_digest tag_error tag_present current matches match_count
  local unexpected_tags candidate_tag_count delete_error delete_status
  local final_error final_inventory
  inventory="$EVIDENCE_DIR/preexisting-${image}-images.json"
  if [[ ! -f "$inventory" || -L "$inventory" ]] \
      || ! jq -e 'type == "array"' "$inventory" >/dev/null; then
    echo "missing or invalid pre-run image inventory for $image; refusing artifact deletion" >&2
    return 1
  fi
  if [[ -f "$EVIDENCE_DIR/images.json" ]]; then
    if [[ "$image" == "sift-acceptance-runner" ]]; then
      digest_ref="$(jq -r '.acceptance_runner // ""' \
        "$EVIDENCE_DIR/images.json")" || return 1
    else
      digest_ref="$(jq -r --arg image "$image" '.[$image] // ""' \
        "$EVIDENCE_DIR/images.json")" || return 1
    fi
  else
    digest_ref=""
  fi
  digest="${digest_ref##*@}"
  if [[ -n "$digest_ref" \
      && ( "$digest" != sha256:* \
        || "$digest_ref" != "$REGISTRY/$image@$digest" ) ]]; then
    echo "invalid immutable image receipt for $image; refusing artifact deletion" >&2
    return 1
  fi

  # Resolve the mutable tag only to check identity. Delete this run's exact tag
  # first. Delete the immutable digest only after a second inventory proves it
  # is untagged.
  tag_error="$(mktemp "${TMPDIR:-/tmp}/sift-cleanup-tag.XXXXXX")"
  tag_present=0
  if live_digest="$(gcloud artifacts docker images describe "$tagged" \
      --project="$PROJECT_ID" --format='value(image_summary.digest)' \
      2> "$tag_error")"; then
    tag_present=1
    if [[ "$digest" != sha256:* || "$live_digest" != "$digest" ]]; then
      echo "live image tag does not match the immutable receipt: $tagged" >&2
      rm -f "$tag_error"
      return 1
    fi
  elif grep -Eiq '(^|[^[:alpha:]])(not[ _-]?found|404)([^[:alpha:]]|$)' \
      "$tag_error"; then
    tag_present=0
  else
    echo "could not inspect live image tag $tagged:" >&2
    cat "$tag_error" >&2
    rm -f "$tag_error"
    return 1
  fi
  rm -f "$tag_error"
  [[ "$digest" == sha256:* ]] || return 0

  inventory_contains_digest="$(jq -r --arg digest "$digest" \
    'any(.. | strings; contains($digest))' "$inventory")" || return 1
  case "$inventory_contains_digest" in
    true)
      if [[ "$tag_present" == "1" ]]; then
        echo "run tag points at a pre-existing digest; refusing image deletion" >&2
        return 1
      fi
      return 0
      ;;
    false) ;;
    *) return 1 ;;
  esac

  current="$(gcloud artifacts docker images list "$REGISTRY/$image" \
    --project="$PROJECT_ID" --include-tags --format=json)" || return 1
  jq -e 'type == "array"' >/dev/null <<<"$current" || return 1
  matches="$(jq -cer --arg digest "$digest" \
    '[.[] | select((tojson | contains($digest)))]' <<<"$current")" || return 1
  match_count="$(jq -r 'length' <<<"$matches")" || return 1
  if [[ "$match_count" == "0" && "$tag_present" == "0" ]]; then
    return 0
  fi
  [[ "$match_count" == "1" ]] || {
    echo "image digest inventory is ambiguous for $image" >&2
    return 1
  }
  unexpected_tags="$(jq -r --arg tag "$IMAGE_TAG" \
    '[.[0].tags[]? | select(. != $tag)] | length' <<<"$matches")" || return 1
  candidate_tag_count="$(jq -r --arg tag "$IMAGE_TAG" \
    '[.[0].tags[]? | select(. == $tag)] | length' <<<"$matches")" || return 1
  if [[ "$tag_present" == "1" \
    && ( "$unexpected_tags" != "0" || "$candidate_tag_count" != "1" ) ]]; then
    echo "image digest has tags outside this run: $image" >&2
    return 1
  fi
  if [[ "$tag_present" == "0" && "$candidate_tag_count" != "0" ]]; then
    echo "image tag lookup and digest inventory disagree: $image" >&2
    return 1
  fi

  if [[ "$tag_present" == "1" ]]; then
    final_error="$(mktemp "${TMPDIR:-/tmp}/sift-cleanup-tag-predelete.XXXXXX")"
    if ! live_digest="$(gcloud artifacts docker images describe "$tagged" \
        --project="$PROJECT_ID" --format='value(image_summary.digest)' \
        2> "$final_error")" \
        || [[ "$live_digest" != "$digest" ]]; then
      echo "run image tag changed immediately before deletion: $tagged" >&2
      [[ ! -s "$final_error" ]] || cat "$final_error" >&2
      rm -f "$final_error"
      return 1
    fi
    rm -f "$final_error"
    delete_error="$(mktemp "${TMPDIR:-/tmp}/sift-cleanup-tag-delete.XXXXXX")"
    if ! gcloud artifacts docker tags delete "$tagged" \
        --project="$PROJECT_ID" --quiet 2> "$delete_error" \
        && ! grep -Eiq '(^|[^[:alpha:]])(not[ _-]?found|404)([^[:alpha:]]|$)' \
          "$delete_error"; then
      echo "failed to delete exact run image tag $tagged:" >&2
      cat "$delete_error" >&2
      rm -f "$delete_error"
      return 1
    fi
    rm -f "$delete_error"
  elif [[ "$unexpected_tags" != "0" ]]; then
    # A prior attempt removed this run's tag. Another run now owns the digest.
    return 0
  fi

  final_error="$(mktemp "${TMPDIR:-/tmp}/sift-cleanup-image-final.XXXXXX")"
  if gcloud artifacts docker images describe "$tagged" \
      --project="$PROJECT_ID" --format='value(image_summary.digest)' \
      > /dev/null 2> "$final_error"; then
    echo "run image tag still exists after exact tag deletion: $tagged" >&2
    rm -f "$final_error"
    return 1
  elif ! grep -Eiq '(^|[^[:alpha:]])(not[ _-]?found|404)([^[:alpha:]]|$)' \
      "$final_error"; then
    echo "could not verify run image tag removal: $tagged" >&2
    cat "$final_error" >&2
    rm -f "$final_error"
    return 1
  fi
  rm -f "$final_error"
  final_inventory="$(gcloud artifacts docker images list "$REGISTRY/$image" \
    --project="$PROJECT_ID" --include-tags --format=json)" || return 1
  jq -e 'type == "array"' >/dev/null <<<"$final_inventory" || return 1
  matches="$(jq -cer --arg digest "$digest" \
    '[.[] | select((tojson | contains($digest)))]' \
    <<<"$final_inventory")" || return 1
  match_count="$(jq -r 'length' <<<"$matches")" || return 1
  if [[ "$match_count" == "0" ]]; then
    printf '%s\n' "$REGISTRY/$image@$digest" \
      > "$EVIDENCE_DIR/deleted-image-${image}.txt" || return 1
    return 0
  fi
  [[ "$match_count" == "1" ]] || {
    echo "image digest inventory changed ambiguously after tag deletion: $image" >&2
    return 1
  }
  candidate_tag_count="$(jq -r --arg tag "$IMAGE_TAG" \
    '[.[0].tags[]? | select(. == $tag)] | length' <<<"$matches")" || return 1
  unexpected_tags="$(jq -r --arg tag "$IMAGE_TAG" \
    '[.[0].tags[]? | select(. != $tag)] | length' <<<"$matches")" || return 1
  if [[ "$candidate_tag_count" != "0" ]]; then
    echo "run image tag remains in digest inventory: $image" >&2
    return 1
  fi
  if [[ "$unexpected_tags" != "0" ]]; then
    return 0
  fi

  delete_error="$(mktemp "${TMPDIR:-/tmp}/sift-cleanup-image-delete.XXXXXX")"
  delete_status=0
  gcloud artifacts docker images delete "$REGISTRY/$image@$digest" \
    --project="$PROJECT_ID" --quiet \
    > /dev/null 2> "$delete_error" || delete_status=$?
  final_inventory="$(gcloud artifacts docker images list "$REGISTRY/$image" \
    --project="$PROJECT_ID" --include-tags --format=json)" || {
    rm -f "$delete_error"
    return 1
  }
  jq -e 'type == "array"' >/dev/null <<<"$final_inventory" || {
    rm -f "$delete_error"
    return 1
  }
  matches="$(jq -cer --arg digest "$digest" \
    '[.[] | select((tojson | contains($digest)))]' \
    <<<"$final_inventory")" || {
    rm -f "$delete_error"
    return 1
  }
  match_count="$(jq -r 'length' <<<"$matches")" || {
    rm -f "$delete_error"
    return 1
  }
  if [[ "$match_count" == "0" ]]; then
    rm -f "$delete_error"
    printf '%s\n' "$REGISTRY/$image@$digest" \
      > "$EVIDENCE_DIR/deleted-image-${image}.txt" || return 1
    return 0
  fi
  if [[ "$match_count" == "1" ]] \
      && jq -e --arg tag "$IMAGE_TAG" '
        ([.[0].tags[]? | select(. == $tag)] | length) == 0
        and ([.[0].tags[]? | select(. != $tag)] | length) > 0
      ' >/dev/null <<<"$matches"; then
    # Another owner attached a tag before the untagged digest delete. Retain
    # that shared digest even when Artifact Registry rejected the delete.
    rm -f "$delete_error"
    return 0
  fi
  if [[ "$delete_status" != "0" ]]; then
    echo "failed to delete untagged image digest $REGISTRY/$image@$digest:" >&2
    cat "$delete_error" >&2
  else
    echo "immutable image digest still exists after deletion: $image" >&2
  fi
  rm -f "$delete_error"
  return 1
}

# Stop every build bound to this run before deleting images or staged source.
# The tag lookup also finds a build when the async submit returned but the
# process died before it could persist cloud-build-id.txt.
acceptance_lock_receipt="$EVIDENCE_DIR/acceptance-lock.json"
acceptance_lock_intent="$EVIDENCE_DIR/acceptance-lock-intent.json"
cleanup_session_intent_root="$EVIDENCE_DIR/acceptance-cleanup-session-intents"
cleanup_session_receipt="$EVIDENCE_DIR/acceptance-cleanup-session.json"
acceptance_lock_owner_ready=0
terminal_release_complete=0
acceptance_lock_resource=""
acceptance_lock_error="$STATE_DIR/acceptance-lock-cleanup.stderr"
: > "$acceptance_lock_error"

if acceptance_lock_resource="$(kubectl get lease "$(acceptance_lock_name)" \
    --namespace "$(acceptance_lock_namespace)" -o json 2>"$acceptance_lock_error")"; then
  if [[ -e "$acceptance_lock_receipt" ]]; then
    receipt_acquisition_id="$(jq -er '.acquisition_id' \
      "$acceptance_lock_receipt" 2>/dev/null || true)"
    if [[ "$receipt_acquisition_id" == "$authorized_run_acquisition_id" ]] \
        && verify_acceptance_lock_receipt_owner \
        "$acceptance_lock_receipt" "$acceptance_lock_resource" \
        "$PROJECT_ID" "$RUN_ID" "$acceptance_mode"; then
      acceptance_lock_owner_ready=1
    else
      record_cleanup_failure \
        "the shared GKE acceptance lock is not owned by this run"
    fi
  elif verify_acceptance_lock_intent \
      "$acceptance_lock_intent" "$PROJECT_ID" "$RUN_ID" "$acceptance_mode"; then
    recovery_acquisition_id="$(jq -er '.acquisition_id' "$acceptance_lock_intent")"
    if [[ "$recovery_acquisition_id" == "$authorized_run_acquisition_id" ]] \
        && verify_acceptance_lock_json \
        "$acceptance_lock_resource" "$PROJECT_ID" "$RUN_ID" \
        "$acceptance_mode" "$recovery_acquisition_id" \
        && write_acceptance_lock_receipt \
          "$acceptance_lock_receipt" "$acceptance_lock_resource" \
          "$PROJECT_ID" "$RUN_ID" "$acceptance_mode" "$recovery_acquisition_id"; then
      acceptance_lock_owner_ready=1
      echo "recovered the shared GKE Lease from its provisional intent" >&2
    else
      record_cleanup_failure \
        "the provisional GKE acceptance lock intent does not own the live Lease"
    fi
  else
    record_cleanup_failure \
      "cleanup has no valid acceptance-lock receipt or provisional intent"
  fi
elif acceptance_lock_is_absent; then
  recovery_acquisition_id=""
  if verify_acceptance_lock_release_receipt \
      "$EVIDENCE_DIR/acceptance-lock-release.json" \
      "$acceptance_lock_receipt" \
      && [[ "$(jq -er '.acquisition_id' "$acceptance_lock_receipt")" \
        == "$authorized_run_acquisition_id" ]]; then
    terminal_release_complete=1
    echo "the shared GKE Lease was already released after verified cleanup" >&2
  elif verify_acceptance_lock_receipt_identity \
      "$acceptance_lock_receipt" "$PROJECT_ID" "$RUN_ID" "$acceptance_mode"; then
    receipt_acquisition_id="$(jq -er '.acquisition_id' \
      "$acceptance_lock_receipt" 2>/dev/null || true)"
    if [[ "$receipt_acquisition_id" == "$authorized_run_acquisition_id" ]]; then
      recovery_acquisition_id="$authorized_run_acquisition_id"
      write_acceptance_lock_intent \
        "$acceptance_lock_intent" "$PROJECT_ID" "$RUN_ID" \
        "$acceptance_mode" "$recovery_acquisition_id" || recovery_acquisition_id=""
    fi
  elif verify_acceptance_lock_intent \
      "$acceptance_lock_intent" "$PROJECT_ID" "$RUN_ID" "$acceptance_mode"; then
    recovery_acquisition_id="$(jq -er '.acquisition_id' "$acceptance_lock_intent")"
    [[ "$recovery_acquisition_id" == "$authorized_run_acquisition_id" ]] \
      || recovery_acquisition_id=""
  fi

  if [[ "$terminal_release_complete" != "1" \
      && -n "$recovery_acquisition_id" ]]; then
    if ! acceptance_lock_resource="$(
        acceptance_lock_manifest \
          "$PROJECT_ID" "$RUN_ID" "$acceptance_mode" "$recovery_acquisition_id" \
          | kubectl create -f - -o json 2>>"$acceptance_lock_error"
      )"; then
      if ! acceptance_lock_resource="$(kubectl get lease "$(acceptance_lock_name)" \
          --namespace "$(acceptance_lock_namespace)" -o json \
          2>>"$acceptance_lock_error")" \
          || ! verify_acceptance_lock_json \
            "$acceptance_lock_resource" "$PROJECT_ID" "$RUN_ID" \
            "$acceptance_mode" "$recovery_acquisition_id"; then
        acceptance_lock_resource=""
      fi
    fi
  fi
  if [[ "$terminal_release_complete" == "1" ]]; then
    :
  elif [[ -n "$acceptance_lock_resource" ]] \
      && write_acceptance_lock_receipt \
        "$acceptance_lock_receipt" "$acceptance_lock_resource" \
        "$PROJECT_ID" "$RUN_ID" "$acceptance_mode" "$recovery_acquisition_id"; then
    acceptance_lock_owner_ready=1
    rm -f "$EVIDENCE_DIR/acceptance-lock-release.json"
    echo "reacquired the shared GKE Lease to resume an interrupted cleanup" >&2
  else
    record_cleanup_failure \
      "the shared GKE acceptance lock could not be safely reacquired"
    [[ ! -s "$acceptance_lock_error" ]] || cat "$acceptance_lock_error" >&2
  fi
else
  record_cleanup_failure "could not inspect the shared GKE acceptance lock"
  [[ ! -s "$acceptance_lock_error" ]] || cat "$acceptance_lock_error" >&2
fi

if [[ "$terminal_release_complete" == "1" ]]; then
  verify_terminal_cleanup_receipt || {
    echo "terminal cleanup evidence is missing or invalid after Lease release" >&2
    exit 1
  }
  if [[ "$acceptance_mode" == "sift" \
      && -f "$EVIDENCE_DIR/sift-mvp-verification.json" ]]; then
    EVIDENCE_DIR="$EVIDENCE_DIR" \
      "$ACCEPTANCE_ROOT/scripts/finalize-sift-mvp-acceptance.sh"
  fi
  exit 0
fi

# Convert the run Lease into a one-process cleanup fence with a CAS patch.
# Every session has an immutable intent. A dead owner can be replaced only by
# a resourceVersion CAS from its exact session ID to a fresh session ID.
if [[ "$acceptance_lock_owner_ready" == "1" ]]; then
  cleanup_session_error="$STATE_DIR/acceptance-cleanup-session.stderr"
  : > "$cleanup_session_error"
  if jq -e \
      '.metadata.annotations["axiom.axiom.dev/cleanup-session-id"] | strings | length > 0' \
      <<<"$acceptance_lock_resource" >/dev/null; then
    previous_cleanup_session_id="$(jq -er \
      '.metadata.annotations["axiom.axiom.dev/cleanup-session-id"]' \
      <<<"$acceptance_lock_resource")"
    cleanup_acquisition_id="$(jq -er '.acquisition_id' "$acceptance_lock_receipt")"
    previous_cleanup_session_intent="$(acceptance_cleanup_session_intent_path \
      "$cleanup_session_intent_root" "$previous_cleanup_session_id")"
    if [[ "$cleanup_acquisition_id" == "$authorized_run_acquisition_id" ]] \
        && verify_acceptance_cleanup_session_intent_identity \
          "$previous_cleanup_session_intent" "$PROJECT_ID" "$RUN_ID" \
          "$acceptance_mode" "$cleanup_acquisition_id" \
          "$previous_cleanup_session_id"; then
      prior_cleanup_pid="$(jq -er \
        '.cleanup_owner_pid' "$previous_cleanup_session_intent")"
      prior_cleanup_start_token="$(jq -er \
        '.cleanup_owner_start_token' "$previous_cleanup_session_intent")"
      prior_cleanup_is_live=0
      current_cleanup_start_token=""
      if current_cleanup_start_token="$(process_start_token "$prior_cleanup_pid")"; then
        [[ "$current_cleanup_start_token" != "$prior_cleanup_start_token" ]] \
          || prior_cleanup_is_live=1
      else
        current_cleanup_start_status=$?
        [[ "$current_cleanup_start_status" == "1" ]] || prior_cleanup_is_live=1
      fi
      if [[ "$prior_cleanup_is_live" == "1" ]]; then
        cleanup_session_id=""
        record_cleanup_failure \
          "the recorded cleanup session process is still active"
      else
        cleanup_session_id="$(openssl rand -hex 16)"
        cleanup_started_at="$(date -u +%Y-%m-%dT%H:%M:%SZ)"
        cleanup_owner_start_token=""
        if ! cleanup_owner_start_token="$(process_start_token "$$")"; then
          cleanup_session_id=""
          record_cleanup_failure \
            "could not verify the new cleanup-session process generation"
        fi
        cleanup_session_intent=""
        if [[ -n "$cleanup_session_id" ]]; then
          cleanup_session_intent="$(acceptance_cleanup_session_intent_path \
            "$cleanup_session_intent_root" "$cleanup_session_id")"
        fi
        [[ -n "$cleanup_session_intent" ]] \
          && write_acceptance_cleanup_session_intent \
            "$cleanup_session_intent" "$PROJECT_ID" "$RUN_ID" \
            "$acceptance_mode" "$cleanup_acquisition_id" "$cleanup_session_id" \
            "$$" "$cleanup_owner_start_token" || cleanup_session_id=""
        if [[ -n "$cleanup_session_id" ]]; then
          cleanup_patch="$(acceptance_cleanup_session_takeover_patch \
            "$acceptance_lock_resource" "$PROJECT_ID" "$RUN_ID" \
            "$acceptance_mode" "$cleanup_acquisition_id" \
            "$previous_cleanup_session_id" "$cleanup_session_id" \
            "$cleanup_started_at")"
          if ! acceptance_lock_resource="$(kubectl patch lease "$(acceptance_lock_name)" \
              --namespace "$(acceptance_lock_namespace)" --type=json \
              -p "$cleanup_patch" -o json 2>"$cleanup_session_error")"; then
            if ! acceptance_lock_resource="$(kubectl get lease "$(acceptance_lock_name)" \
                --namespace "$(acceptance_lock_namespace)" -o json \
                2>>"$cleanup_session_error")" \
                || ! verify_acceptance_cleanup_session_json \
                  "$acceptance_lock_resource" "$PROJECT_ID" "$RUN_ID" \
                  "$acceptance_mode" "$cleanup_acquisition_id" \
                  "$cleanup_session_id"; then
              acceptance_lock_resource=""
            fi
          fi
        fi
        if [[ -n "$cleanup_session_id" && -n "$acceptance_lock_resource" ]] \
            && write_acceptance_lock_receipt \
              "$acceptance_lock_receipt" "$acceptance_lock_resource" \
              "$PROJECT_ID" "$RUN_ID" "$acceptance_mode" \
              "$cleanup_acquisition_id" \
            && write_acceptance_cleanup_session_receipt \
              "$cleanup_session_receipt" "$acceptance_lock_resource" \
              "$PROJECT_ID" "$RUN_ID" "$acceptance_mode" \
              "$cleanup_acquisition_id" "$cleanup_session_id"; then
          acceptance_lock_held=1
          echo "took over the cleanup session after its recorded owner stopped" >&2
        else
          cleanup_session_id=""
          record_cleanup_failure \
            "could not atomically take over the interrupted cleanup session"
          [[ ! -s "$cleanup_session_error" ]] \
            || cat "$cleanup_session_error" >&2
        fi
      fi
    else
      cleanup_session_id=""
      record_cleanup_failure \
        "another cleanup session already owns the shared GKE acceptance lock"
    fi
  else
    cleanup_session_id="$(openssl rand -hex 16)"
    cleanup_started_at="$(date -u +%Y-%m-%dT%H:%M:%SZ)"
    cleanup_acquisition_id="$(jq -er '.acquisition_id' "$acceptance_lock_receipt")"
    cleanup_owner_start_token=""
    if ! cleanup_owner_start_token="$(process_start_token "$$")"; then
      cleanup_session_id=""
      record_cleanup_failure \
        "could not verify the cleanup-session process generation"
    fi
    cleanup_session_intent=""
    if [[ -n "$cleanup_session_id" ]]; then
      cleanup_session_intent="$(acceptance_cleanup_session_intent_path \
        "$cleanup_session_intent_root" "$cleanup_session_id")"
    fi
    [[ -n "$cleanup_session_intent" ]] \
      && write_acceptance_cleanup_session_intent \
      "$cleanup_session_intent" "$PROJECT_ID" "$RUN_ID" "$acceptance_mode" \
      "$cleanup_acquisition_id" "$cleanup_session_id" \
      "$$" "$cleanup_owner_start_token" || {
      cleanup_session_id=""
      record_cleanup_failure "could not persist the cleanup-session intent"
    }
    if [[ -n "$cleanup_session_id" ]]; then
      cleanup_patch="$(acceptance_cleanup_session_patch \
        "$acceptance_lock_resource" "$cleanup_acquisition_id" \
        "$cleanup_session_id" "$cleanup_started_at")"
      if ! acceptance_lock_resource="$(kubectl patch lease "$(acceptance_lock_name)" \
          --namespace "$(acceptance_lock_namespace)" --type=json \
          -p "$cleanup_patch" -o json 2>"$cleanup_session_error")"; then
        if ! acceptance_lock_resource="$(kubectl get lease "$(acceptance_lock_name)" \
            --namespace "$(acceptance_lock_namespace)" -o json \
            2>>"$cleanup_session_error")" \
            || ! verify_acceptance_cleanup_session_json \
              "$acceptance_lock_resource" "$PROJECT_ID" "$RUN_ID" \
              "$acceptance_mode" "$cleanup_acquisition_id" "$cleanup_session_id"; then
          acceptance_lock_resource=""
        fi
      fi
      if [[ -n "$acceptance_lock_resource" ]] \
          && write_acceptance_lock_receipt \
            "$acceptance_lock_receipt" "$acceptance_lock_resource" \
            "$PROJECT_ID" "$RUN_ID" "$acceptance_mode" "$cleanup_acquisition_id" \
          && write_acceptance_cleanup_session_receipt \
            "$cleanup_session_receipt" "$acceptance_lock_resource" \
            "$PROJECT_ID" "$RUN_ID" "$acceptance_mode" \
            "$cleanup_acquisition_id" "$cleanup_session_id"; then
        acceptance_lock_held=1
      else
        cleanup_session_id=""
        record_cleanup_failure \
          "could not atomically claim the shared GKE Lease for this cleanup"
        [[ ! -s "$cleanup_session_error" ]] || cat "$cleanup_session_error" >&2
      fi
    fi
  fi
fi

# If a previous cleanup removed the reservation last but lost its final local
# response, missing candidate controls cannot authorize more remote deletion.
# This branch performs only final read checks, then releases the independently
# owned acceptance Lease.
source_absent_finalize=0
reservation_only_finalize=0
source_absent_candidate_directory=""
if [[ "$acceptance_lock_held" == "1" \
    && "$cleanup_failures" == "0" \
    && "$acceptance_mode" == "sift" ]] \
    && source_absent_candidate_directory="$(sift_candidate_directory_for_cleanup)" \
    && sift_candidate_reservation_is_absent \
      "$source_absent_candidate_directory" \
    && sift_candidate_source_prefix_is_empty read-only-finalize; then
  source_absent_finalize=1
fi
if [[ "$acceptance_lock_held" == "1" \
    && "$cleanup_failures" == "0" \
    && "$acceptance_mode" == "sift" \
    && "$source_absent_finalize" == "0" ]] \
    && source_absent_candidate_directory="$(sift_candidate_directory_for_cleanup)" \
    && verify_live_sift_candidate_reservation \
      "$source_absent_candidate_directory" reservation-only-detect \
    && sift_candidate_submit_intent_is_absent \
      "$source_absent_candidate_directory" \
    && sift_candidate_source_has_only_reservation reservation-only-detect; then
  reservation_only_finalize=1
fi

if [[ "$source_absent_finalize" == "1" ]]; then
  source_absent_verify_status=0
  if ! assert_acceptance_cleanup_session; then
    source_absent_verify_status=1
  else
    PROJECT_ID="$PROJECT_ID" REGION="$REGION" GKE_ZONE="$GKE_ZONE" \
      RUN_ID="$RUN_ID" REGISTRY="$REGISTRY" IMAGE_TAG="$IMAGE_TAG" \
      GCS_SOURCE_PREFIX="$GCS_SOURCE_PREFIX" EVIDENCE_DIR="$EVIDENCE_DIR" \
      PERSISTENT_CLUSTER_NAME="$PERSISTENT_CLUSTER_NAME" \
      PERSISTENT_CLUSTER_CHECK_REQUIRED="$([[ -s "$STATE_DIR/persistent-cluster-name.txt" ]] && echo 1 || echo 0)" \
      KUBERNETES_CHECK_REQUIRED="$([[ -f "$STATE_DIR/kube-context-ready.txt" ]] && echo 1 || echo 0)" \
      ACCEPTANCE_APPS="$ACCEPTANCE_APPS" \
      SIFT_CANDIDATE_DIR="$source_absent_candidate_directory" \
      "$ACCEPTANCE_ROOT/scripts/verify-clean.sh" \
      || source_absent_verify_status=$?
  fi
  if [[ "$source_absent_verify_status" != "0" ]]; then
    rm -f "$EVIDENCE_DIR/cleanup.json" \
      "$EVIDENCE_DIR/acceptance.json" \
      "$EVIDENCE_DIR/sift-mvp-acceptance.json"
    echo "candidate controls are absent; read-only cleanup finalization failed" >&2
    exit 1
  fi
  if ! release_acceptance_lock; then
    rm -f "$EVIDENCE_DIR/cleanup.json" \
      "$EVIDENCE_DIR/acceptance.json" \
      "$EVIDENCE_DIR/sift-mvp-acceptance.json"
    echo "could not release the acceptance Lease after read-only finalization" >&2
    exit 1
  fi
  if [[ -f "$EVIDENCE_DIR/sift-mvp-verification.json" ]]; then
    EVIDENCE_DIR="$EVIDENCE_DIR" \
      "$ACCEPTANCE_ROOT/scripts/finalize-sift-mvp-acceptance.sh"
  fi
  exit 0
fi

if [[ "$reservation_only_finalize" == "1" ]]; then
  reservation_only_verify_status=0
  reservation_generation=""
  if ! assert_acceptance_cleanup_session; then
    reservation_only_verify_status=1
  else
    PROJECT_ID="$PROJECT_ID" REGION="$REGION" GKE_ZONE="$GKE_ZONE" \
      RUN_ID="$RUN_ID" REGISTRY="$REGISTRY" IMAGE_TAG="$IMAGE_TAG" \
      GCS_SOURCE_PREFIX="$GCS_SOURCE_PREFIX" EVIDENCE_DIR="$EVIDENCE_DIR" \
      PERSISTENT_CLUSTER_NAME="$PERSISTENT_CLUSTER_NAME" \
      PERSISTENT_CLUSTER_CHECK_REQUIRED="$([[ -s "$STATE_DIR/persistent-cluster-name.txt" ]] && echo 1 || echo 0)" \
      KUBERNETES_CHECK_REQUIRED="$([[ -f "$STATE_DIR/kube-context-ready.txt" ]] && echo 1 || echo 0)" \
      ACCEPTANCE_APPS="$ACCEPTANCE_APPS" \
      SIFT_CANDIDATE_DIR="$source_absent_candidate_directory" \
      SIFT_CANDIDATE_CONTROL_OBJECTS_ALLOWED=1 \
      VERIFY_CLEAN_WRITE_RECEIPT=0 \
      "$ACCEPTANCE_ROOT/scripts/verify-clean.sh" \
      || reservation_only_verify_status=$?
  fi
  if [[ "$reservation_only_verify_status" != "0" ]] \
      || ! assert_acceptance_cleanup_session \
      || ! reservation_generation="$(
        verified_live_sift_candidate_object_generation \
          "$source_absent_candidate_directory" \
          "${GCS_SOURCE_PREFIX}/candidate-reservation.json" \
          "$source_absent_candidate_directory/candidate-reservation.json" \
          reservation-only-final
      )" \
      || ! verify_current_sift_candidate_build_inventory \
        "$source_absent_candidate_directory" reservation-only-final; then
    echo "reservation-only cleanup finalization failed its read checks" >&2
    exit 1
  fi
  reservation_uri="${GCS_SOURCE_PREFIX}/candidate-reservation.json"
  reservation_delete_error="$STATE_DIR/source-reservation-only-delete.stderr"
  if ! gcloud storage rm "$reservation_uri" \
      --if-generation-match="$reservation_generation" --quiet \
      > /dev/null 2> "$reservation_delete_error"; then
    echo "could not delete the final candidate reservation" >&2
    exit 1
  fi
  reservation_only_verify_status=0
  PROJECT_ID="$PROJECT_ID" REGION="$REGION" GKE_ZONE="$GKE_ZONE" \
    RUN_ID="$RUN_ID" REGISTRY="$REGISTRY" IMAGE_TAG="$IMAGE_TAG" \
    GCS_SOURCE_PREFIX="$GCS_SOURCE_PREFIX" EVIDENCE_DIR="$EVIDENCE_DIR" \
    PERSISTENT_CLUSTER_NAME="$PERSISTENT_CLUSTER_NAME" \
    PERSISTENT_CLUSTER_CHECK_REQUIRED="$([[ -s "$STATE_DIR/persistent-cluster-name.txt" ]] && echo 1 || echo 0)" \
    KUBERNETES_CHECK_REQUIRED="$([[ -f "$STATE_DIR/kube-context-ready.txt" ]] && echo 1 || echo 0)" \
    ACCEPTANCE_APPS="$ACCEPTANCE_APPS" \
    SIFT_CANDIDATE_DIR="$source_absent_candidate_directory" \
    "$ACCEPTANCE_ROOT/scripts/verify-clean.sh" \
    || reservation_only_verify_status=$?
  if [[ "$reservation_only_verify_status" != "0" ]]; then
    rm -f "$EVIDENCE_DIR/cleanup.json"
    echo "final verification failed after reservation-only cleanup" >&2
    exit 1
  fi
  if ! release_acceptance_lock; then
    rm -f "$EVIDENCE_DIR/cleanup.json"
    echo "could not release the acceptance Lease after reservation-only cleanup" >&2
    exit 1
  fi
  if [[ -f "$EVIDENCE_DIR/sift-mvp-verification.json" ]]; then
    EVIDENCE_DIR="$EVIDENCE_DIR" \
      "$ACCEPTANCE_ROOT/scripts/finalize-sift-mvp-acceptance.sh"
  fi
  exit 0
fi

if [[ "$acceptance_lock_held" == "1" ]] \
    && assert_acceptance_cleanup_session \
    && ! stop_run_cloud_builds; then
  cloud_build_cleanup_failed=1
  record_cleanup_failure "Cloud Build inventory or cancellation did not complete"
elif [[ "$acceptance_lock_held" != "1" ]]; then
  cloud_build_cleanup_failed=1
  echo "retaining run resources because this process does not own the shared GKE Lease" >&2
elif ! assert_acceptance_cleanup_session; then
  acceptance_lock_held=0
  cloud_build_cleanup_failed=1
  record_cleanup_failure "the cleanup session lost the shared GKE Lease"
fi

source_prefix_receipt="$EVIDENCE_DIR/source-prefix.json"
source_candidate_ready=1
source_candidate_directory=""
if [[ "$acceptance_mode" == "sift" \
    && ( -n "$SIFT_CANDIDATE_DIR" \
      || -e "$EVIDENCE_DIR/candidate.json" \
      || -L "$EVIDENCE_DIR/candidate.json" ) ]]; then
  if ! source_candidate_directory="$(sift_candidate_directory_for_cleanup)" \
      || ! verify_live_sift_candidate_control_objects \
        "$source_candidate_directory" source-authorization; then
    source_candidate_ready=0
    record_cleanup_failure \
      "the live Sift candidate reservation is not safe for source cleanup"
  fi
fi
if ! validated_source_bucket "$GCS_SOURCE_PREFIX" "$RUN_ID" >/dev/null; then
  record_cleanup_failure \
    "unsafe Cloud Build source prefix; expected gs://BUCKET/source/axiom-gcp-operator-RUN_ID"
elif verify_source_prefix_receipt \
    "$source_prefix_receipt" "$PROJECT_ID" "$RUN_ID" "$GCS_SOURCE_PREFIX"; then
  if verify_cloud_build_source_evidence \
      "$EVIDENCE_DIR" "$GCS_SOURCE_PREFIX" "$RUN_ID"; then
    if [[ "$source_candidate_ready" == "1" \
        && "$acceptance_lock_held" == "1" ]] \
        && assert_acceptance_cleanup_session; then
      source_cleanup_authorized=1
    fi
  else
    record_cleanup_failure \
      "Cloud Build reported a source object outside the run-scoped prefix"
  fi
elif [[ -e "$source_prefix_receipt" ]]; then
  record_cleanup_failure "the Cloud Build source-prefix receipt is invalid"
elif [[ "$acceptance_mode" == "sift" || -s "$STATE_DIR/cloud-build-ids.txt" ]]; then
  record_cleanup_failure "the Cloud Build source-prefix receipt is missing"
fi

if [[ -f "$STATE_DIR/kube-context-ready.txt" && "$acceptance_lock_held" == "1" ]]; then
  if assert_acceptance_cleanup_session; then
    kubernetes_cleanup_authorized=1
  else
    acceptance_lock_held=0
    record_cleanup_failure "the cleanup session lost the shared GKE Lease"
  fi
fi

if [[ "$kubernetes_cleanup_authorized" == "1" ]]; then
  capture_failure_evidence
  sift_owned_namespaces=()
  sift_namespace_cleanup_safe=1
  sift_kubernetes_identity_safe=1
  sift_crd_owned=0
  sift_primary_namespace_owned=0
  sift_restore_namespace_owned=0
  if [[ "$acceptance_mode" == "tape" ]]; then
    namespaces=(tape tape-system)
  elif [[ "$acceptance_mode" == "lumen-auth" ]]; then
    namespaces=(lumen lumen-system lumen-auth-client)
  elif [[ "$acceptance_mode" == "sift" ]]; then
    namespaces=()
    sift_owned_namespaces=(sift-restore sift sift-system)
  else
    # lumen-fleet-a/-b are the data-plane namespaces the LumenFleet leg
    # materializes into. They are swept HERE, not only at the end of that leg,
    # because a leg that fails midway leaves StatefulSets and their PVCs behind
    # -- and this cluster is persistent, so a leaked PVC is a Persistent Disk
    # that bills forever with nothing left to point at it. Every namespace the
    # run can create belongs in this list, including the ones a passing run
    # tears down itself.
    # lumen-auth-client holds the client ServiceAccount the auth leg (#2879)
    # puts in a *second* namespace to prove a SubjectAccessReview scoped to the
    # serving namespace does not honour a grant written elsewhere.
    namespaces=(lumen lumen-system lumen-fleet-a lumen-fleet-b lumen-auth-client)
    sift_owned_namespaces=(sift sift-system)
  fi
  # Sift owns cluster-scoped RBAC through a CR finalizer. Delete each known
  # acceptance instance while sift-system still hosts the operator. Deleting
  # the operator namespace first strands the finalizer and blocks both the
  # data namespace and CRD in Terminating.
  if [[ "$acceptance_mode" == "sift" || "$acceptance_mode" == "lumen-sift" ]]; then
    ownership_root="$EVIDENCE_DIR/kubernetes/ownership"
    ownership_status=0
    assert_owned_kubernetes_resource \
      customresourcedefinition sifts.sift.axiom.dev "$ownership_root" \
      "$PROJECT_ID" "$RUN_ID" "$authorized_run_acquisition_id" \
      || ownership_status=$?
    case "$ownership_status" in
      0) sift_crd_owned=1 ;;
      2) ;;
      *)
        record_cleanup_failure \
          "the live Sift CRD does not match this run's ownership receipt"
        sift_kubernetes_identity_safe=0
        ;;
    esac
    for namespace in "${sift_owned_namespaces[@]}"; do
      ownership_status=0
      assert_owned_kubernetes_resource \
        namespace "$namespace" "$ownership_root" \
        "$PROJECT_ID" "$RUN_ID" "$authorized_run_acquisition_id" \
        || ownership_status=$?
      case "$ownership_status" in
        0)
          case "$namespace" in
            sift) sift_primary_namespace_owned=1 ;;
            sift-restore) sift_restore_namespace_owned=1 ;;
          esac
          ;;
        2) ;;
        *)
          record_cleanup_failure \
            "the live Sift namespace $namespace does not match this run's ownership receipt"
          sift_kubernetes_identity_safe=0
          ;;
      esac
    done
    if [[ "$sift_kubernetes_identity_safe" == "1" \
      && "$sift_crd_owned" == "1" ]]; then
      if [[ "$sift_primary_namespace_owned" == "1" ]]; then
        if ! delete_sift_instance sift sift; then
          record_cleanup_failure \
            "could not safely delete the primary Sift instance"
          sift_namespace_cleanup_safe=0
        fi
      fi
      if [[ "$sift_restore_namespace_owned" == "1" ]]; then
        if ! delete_sift_instance sift-restore sift-restore; then
          record_cleanup_failure \
            "could not safely delete the restored Sift instance"
          sift_namespace_cleanup_safe=0
        fi
      fi
    fi
  fi
  # The fleet controller reconciles cluster-wide, so it must lose its API
  # before its target namespaces start terminating; otherwise a pass that
  # lands between two deletes re-materializes a Lumen into a namespace on its
  # way out and the no-leftovers gate trips on a resource cleanup just removed.
  if [[ "$acceptance_mode" != "tape" && "$acceptance_mode" != "sift" ]]; then
    kubectl delete customresourcedefinition lumenfleets.lumen.dev \
      --ignore-not-found --wait=true --timeout=180s >/dev/null 2>&1 || true
  fi
  if [[ "$sift_kubernetes_identity_safe" == "1" \
      && "$sift_namespace_cleanup_safe" == "1" ]]; then
    for namespace in "${sift_owned_namespaces[@]}"; do
      if ! delete_owned_kubernetes_resource \
          namespace "$namespace" "$EVIDENCE_DIR/kubernetes/ownership" \
          "$PROJECT_ID" "$RUN_ID" "$authorized_run_acquisition_id" 300; then
        record_cleanup_failure \
          "could not safely delete owned Sift namespace $namespace"
        sift_namespace_cleanup_safe=0
      fi
    done
  else
    sift_namespace_cleanup_safe=0
    record_cleanup_failure \
      "retaining Sift Kubernetes resources because safe instance or ownership cleanup did not complete"
  fi
  if [[ "$acceptance_mode" != "sift" ]]; then
    for namespace in "${namespaces[@]}"; do
      kubectl delete namespace "$namespace" --ignore-not-found --wait=false \
        >/dev/null 2>&1 || true
    done
    # Namespace deletion can outlive kubectl's delete response while the
    # apiserver clears finalizers. Do not run the no-leftovers gate against
    # that transient state.
    namespace_deadline=$((SECONDS + 300))
    while true; do
      remaining_namespaces=()
      for namespace in "${namespaces[@]}"; do
        if kubectl get namespace "$namespace" --no-headers >/dev/null 2>&1; then
          remaining_namespaces+=("$namespace")
        fi
      done
      [[ "${#remaining_namespaces[@]}" == "0" ]] && break
      if (( SECONDS >= namespace_deadline )); then
        echo "namespaces still terminating after cleanup wait: ${remaining_namespaces[*]}" >&2
        break
      fi
      sleep 5
    done
  fi
  if [[ "$acceptance_mode" == "tape" ]]; then
    kubectl delete customresourcedefinition tapes.tape.dev \
      --ignore-not-found --wait=true --timeout=180s >/dev/null 2>&1 || true
  else
    if [[ "$acceptance_mode" == "lumen-auth" ]]; then
      kubectl delete customresourcedefinition lumens.lumen.dev \
        --ignore-not-found --wait=true --timeout=180s >/dev/null 2>&1 || true
    elif [[ "$acceptance_mode" == "sift" ]]; then
      if [[ "$sift_namespace_cleanup_safe" == "1" ]]; then
        delete_owned_kubernetes_resource \
          customresourcedefinition sifts.sift.axiom.dev \
          "$EVIDENCE_DIR/kubernetes/ownership" \
          "$PROJECT_ID" "$RUN_ID" "$authorized_run_acquisition_id" 180 \
          || record_cleanup_failure \
            "could not safely delete the owned Sift CRD"
      else
        record_cleanup_failure \
          "retaining the Sift CRD because an owned namespace was not safely deleted"
      fi
      kubectl delete clusterrolebinding \
        -l axiom-owner=gcp-operator-acceptance,axiom-run-id="$RUN_ID" \
        --ignore-not-found --wait=true --timeout=180s >/dev/null 2>&1 || true
      # The Sift CR finalizer owns its operator-managed ClusterRoleBinding.
      # Do not sweep every Sift binding in this shared cluster. If either known
      # acceptance binding remains, verify-clean reports the exact leftover and
      # refuses to publish terminal evidence.
    else
      kubectl delete customresourcedefinition lumens.lumen.dev \
        --ignore-not-found --wait=true --timeout=180s >/dev/null 2>&1 || true
      if [[ "$sift_namespace_cleanup_safe" == "1" ]]; then
        delete_owned_kubernetes_resource \
          customresourcedefinition sifts.sift.axiom.dev \
          "$EVIDENCE_DIR/kubernetes/ownership" \
          "$PROJECT_ID" "$RUN_ID" "$authorized_run_acquisition_id" 180 \
          || record_cleanup_failure \
            "could not safely delete the owned Sift CRD"
      else
        record_cleanup_failure \
          "retaining the Sift CRD because an owned namespace was not safely deleted"
      fi
    fi
    # The per-instance `system:auth-delegator` binding (#2876) is
    # cluster-scoped, so namespace deletion cannot reach it. The mode helper
    # removes it only for a Lumen run. A Sift-only run does not own it.
    cleanup_lumen_auth_delegation_bindings_for_mode "$acceptance_mode" \
      || record_cleanup_failure "could not select the auth binding cleanup mode"
  fi
fi

if [[ -f "$state" && "$acceptance_lock_held" == "1" ]] \
    && assert_acceptance_cleanup_session; then
  destroy_args=(
    -state="$state"
    -auto-approve
    -var="project_id=$PROJECT_ID"
    -var="region=$REGION"
    -var="gke_zone=$GKE_ZONE"
    -var="run_id=$RUN_ID"
    -var="artifact_registry_repository=$ARTIFACT_REGISTRY_REPOSITORY"
    -var="image_tag=$IMAGE_TAG"
  )
  if [[ "$acceptance_mode" == "tape" ]]; then
    destroy_args+=(-var="acceptance_apps=tape")
  elif [[ "$acceptance_mode" == "lumen-auth" ]]; then
    destroy_args+=(-var="acceptance_apps=lumen-auth")
  elif [[ "$acceptance_mode" == "sift" ]]; then
    destroy_args+=(-var="acceptance_apps=sift")
  fi
  for attempt in 1 2 3; do
    if TF_DATA_DIR="$tf_data" terraform -chdir="$TERRAFORM_ENVIRONMENT_DIR" \
      destroy "${destroy_args[@]}"; then
      break
    fi
    if [[ "$attempt" == "3" ]]; then
      echo "Terraform destroy failed after three attempts; state retained at $state" >&2
      record_cleanup_failure "Terraform destroy failed after three attempts"
      break
    fi
    sleep 15
  done
elif [[ -f "$state" && "$acceptance_lock_held" == "1" ]]; then
  acceptance_lock_held=0
  record_cleanup_failure "the cleanup session lost the shared GKE Lease"
fi

if [[ "$cloud_build_cleanup_failed" == "1" ]]; then
  echo "retaining candidate images and staged source until Cloud Build state is known" >&2
elif [[ "$acceptance_lock_held" != "1" ]]; then
  echo "retaining candidate images and staged source because cleanup lost its Lease" >&2
elif ! assert_acceptance_cleanup_session; then
  acceptance_lock_held=0
  record_cleanup_failure "the cleanup session lost the shared GKE Lease"
  echo "retaining candidate images and staged source because cleanup lost its Lease" >&2
else
  if [[ "$acceptance_mode" == "sift" ]]; then
    if ! delete_run_image sift; then
      image_cleanup_failed=1
      record_cleanup_failure "could not delete the run Sift image"
    fi
    if ! delete_run_image rig; then
      image_cleanup_failed=1
      record_cleanup_failure "could not delete the run Rig image"
    fi
    if ! delete_run_image sift-acceptance-runner; then
      image_cleanup_failed=1
      record_cleanup_failure "could not delete the run Sift acceptance runner image"
    fi
  elif [[ "$acceptance_mode" == "tape" ]]; then
    if ! delete_run_image tape; then
      image_cleanup_failed=1
      record_cleanup_failure "could not delete the run Tape image"
    fi
  elif [[ "$acceptance_mode" == "lumen-auth" ]]; then
    if ! delete_run_image lumen; then
      image_cleanup_failed=1
      record_cleanup_failure "could not delete the run Lumen image"
    fi
  else
    if ! delete_run_image lumen; then
      image_cleanup_failed=1
      record_cleanup_failure "could not delete the run Lumen image"
    fi
    if ! delete_run_image sift; then
      image_cleanup_failed=1
      record_cleanup_failure "could not delete the run Sift image"
    fi
  fi
  if [[ "$source_cleanup_authorized" == "1" \
      && "$image_cleanup_failed" == "0" ]]; then
    source_cleanup_ready=1
    if ! assert_acceptance_cleanup_session; then
      acceptance_lock_held=0
      source_cleanup_ready=0
      record_cleanup_failure \
        "the cleanup session lost the shared GKE Lease before source cleanup"
    elif [[ -n "$source_candidate_directory" ]] \
        && { ! verify_live_sift_candidate_control_objects \
              "$source_candidate_directory" before-source-delete \
          || ! verify_current_sift_candidate_build_inventory \
              "$source_candidate_directory" before-source-delete; }; then
      source_cleanup_ready=0
      record_cleanup_failure \
        "the Sift candidate changed before staged source deletion"
    fi
    if [[ "$source_cleanup_ready" == "1" ]] \
        && verify_source_prefix_receipt \
          "$source_prefix_receipt" "$PROJECT_ID" "$RUN_ID" "$GCS_SOURCE_PREFIX"; then
      source_inventory="$(mktemp "$STATE_DIR/.source-cleanup-inventory.XXXXXX")"
      source_inventory_error="${source_inventory}.stderr"
      if ! gcloud storage ls --recursive "${GCS_SOURCE_PREFIX}/**" \
          > "$source_inventory" 2> "$source_inventory_error"; then
        if [[ -z "$source_candidate_directory" ]] \
            && grep -Eiq \
              '(matched no URLs|no URLs matched|not[ _-]?found|404)' \
              "$source_inventory_error"; then
          : > "$source_inventory"
        else
          source_cleanup_ready=0
          record_cleanup_failure \
            "could not inventory the run-staged source objects"
        fi
      fi
      if [[ "$source_cleanup_ready" == "1" ]] \
          && ! assert_acceptance_cleanup_session; then
        acceptance_lock_held=0
        source_cleanup_ready=0
        record_cleanup_failure \
          "the cleanup session lost the shared GKE Lease before source deletion"
      fi
      if [[ "$source_cleanup_ready" == "1" ]]; then
        reservation_uri="${GCS_SOURCE_PREFIX}/candidate-reservation.json"
        submit_intent_uri="$(sift_candidate_submit_intent_uri "$GCS_SOURCE_PREFIX")"
        source_delete_index=0
        while IFS= read -r source_uri; do
          [[ -n "$source_uri" ]] || continue
          case "$source_uri" in
            "$GCS_SOURCE_PREFIX"/*) ;;
            *)
              source_cleanup_ready=0
              record_cleanup_failure \
                "the source inventory escaped its run-scoped prefix"
              break
              ;;
          esac
          if [[ -n "$source_candidate_directory" \
            && ( "$source_uri" == "$reservation_uri" \
              || "$source_uri" == "$submit_intent_uri" ) ]]; then
            continue
          fi
          source_delete_index=$((source_delete_index + 1))
          source_delete_error="$STATE_DIR/source-delete-${source_delete_index}.stderr"
          if ! gcloud storage rm "$source_uri" --quiet \
              > /dev/null 2> "$source_delete_error"; then
            source_cleanup_ready=0
            record_cleanup_failure \
              "could not delete an exact run-staged source object"
            break
          fi
        done < "$source_inventory"
      fi
      rm -f "$source_inventory" "$source_inventory_error"

      if [[ "$source_cleanup_ready" == "1" \
          && -n "$source_candidate_directory" ]]; then
        pre_source_verify_status=0
        if [[ "$cleanup_failures" != "0" ]]; then
          pre_source_verify_status=1
        else
          PROJECT_ID="$PROJECT_ID" REGION="$REGION" GKE_ZONE="$GKE_ZONE" \
            RUN_ID="$RUN_ID" REGISTRY="$REGISTRY" IMAGE_TAG="$IMAGE_TAG" \
            GCS_SOURCE_PREFIX="$GCS_SOURCE_PREFIX" \
            EVIDENCE_DIR="$EVIDENCE_DIR" \
            PERSISTENT_CLUSTER_NAME="$PERSISTENT_CLUSTER_NAME" \
            PERSISTENT_CLUSTER_CHECK_REQUIRED="$([[ -s "$STATE_DIR/persistent-cluster-name.txt" ]] && echo 1 || echo 0)" \
            KUBERNETES_CHECK_REQUIRED="$([[ -f "$STATE_DIR/kube-context-ready.txt" ]] && echo 1 || echo 0)" \
            ACCEPTANCE_APPS="$ACCEPTANCE_APPS" \
            SIFT_CANDIDATE_DIR="$source_candidate_directory" \
            SIFT_CANDIDATE_CONTROL_OBJECTS_ALLOWED=1 \
            VERIFY_CLEAN_WRITE_RECEIPT=0 \
            "$ACCEPTANCE_ROOT/scripts/verify-clean.sh" \
            || pre_source_verify_status=$?
        fi
        if [[ "$pre_source_verify_status" != "0" ]]; then
          source_cleanup_ready=0
          record_cleanup_failure \
            "pre-source no-leftovers verification failed"
        fi
      fi

      if [[ "$source_cleanup_ready" == "1" \
          && -n "$source_candidate_directory" ]]; then
        if ! assert_acceptance_cleanup_session \
            || ! verify_live_sift_candidate_control_objects \
              "$source_candidate_directory" before-control-delete \
            || ! verify_current_sift_candidate_build_inventory \
              "$source_candidate_directory" before-control-delete; then
          acceptance_lock_held=0
          source_cleanup_ready=0
          record_cleanup_failure \
            "candidate ownership changed before control deletion"
        fi
      fi

      if [[ "$source_cleanup_ready" == "1" \
          && -n "$source_candidate_directory" ]]; then
        source_delete_error="$STATE_DIR/source-submit-intent-delete.stderr"
        submit_intent_generation="$(
          verified_live_sift_candidate_object_generation \
            "$source_candidate_directory" "$submit_intent_uri" \
            "$source_candidate_directory/candidate-submit-intent.json" \
            submit-intent-final-delete
        )" || {
          source_cleanup_ready=0
          record_cleanup_failure \
            "candidate submit intent changed before final deletion"
        }
        if [[ "$source_cleanup_ready" == "1" ]] \
            && ! gcloud storage rm "$submit_intent_uri" \
              --if-generation-match="$submit_intent_generation" --quiet \
            > /dev/null 2> "$source_delete_error"; then
          source_cleanup_ready=0
          record_cleanup_failure \
            "could not delete the candidate submit intent"
        fi
      fi

      if [[ "$source_cleanup_ready" == "1" \
          && -n "$source_candidate_directory" ]]; then
        if ! assert_acceptance_cleanup_session \
            || ! verify_live_sift_candidate_reservation \
              "$source_candidate_directory" final-source-delete \
            || ! verify_current_sift_candidate_build_inventory \
              "$source_candidate_directory" final-source-delete; then
          acceptance_lock_held=0
          source_cleanup_ready=0
          record_cleanup_failure \
            "candidate reservation changed before final deletion"
        else
          reservation_generation="$(
            verified_live_sift_candidate_object_generation \
              "$source_candidate_directory" "$reservation_uri" \
              "$source_candidate_directory/candidate-reservation.json" \
              reservation-final-delete
          )" || {
            source_cleanup_ready=0
            record_cleanup_failure \
              "candidate reservation changed before final deletion"
          }
          source_delete_error="$STATE_DIR/source-reservation-delete.stderr"
          if [[ "$source_cleanup_ready" == "1" ]] \
              && ! gcloud storage rm "$reservation_uri" \
                --if-generation-match="$reservation_generation" --quiet \
              > /dev/null 2> "$source_delete_error"; then
            source_cleanup_ready=0
            record_cleanup_failure \
              "could not delete the candidate reservation"
          fi
        fi
      fi
    elif [[ "$source_cleanup_ready" == "1" ]]; then
      record_cleanup_failure \
        "the Cloud Build source-prefix receipt changed before source deletion"
    fi
  elif [[ "$image_cleanup_failed" == "1" ]]; then
    echo "retaining staged source because candidate image cleanup did not complete" >&2
  else
    echo "retaining staged source because no valid run-scoped receipt exists" >&2
  fi
fi

verify_clean_status=0
if [[ "$acceptance_lock_held" == "1" ]] \
    && assert_acceptance_cleanup_session; then
  PROJECT_ID="$PROJECT_ID" REGION="$REGION" GKE_ZONE="$GKE_ZONE" RUN_ID="$RUN_ID" \
    REGISTRY="$REGISTRY" IMAGE_TAG="$IMAGE_TAG" \
    GCS_SOURCE_PREFIX="$GCS_SOURCE_PREFIX" EVIDENCE_DIR="$EVIDENCE_DIR" \
    PERSISTENT_CLUSTER_NAME="$PERSISTENT_CLUSTER_NAME" \
    PERSISTENT_CLUSTER_CHECK_REQUIRED="$([[ -s "$STATE_DIR/persistent-cluster-name.txt" ]] && echo 1 || echo 0)" \
    KUBERNETES_CHECK_REQUIRED="$([[ -f "$STATE_DIR/kube-context-ready.txt" ]] && echo 1 || echo 0)" \
    ACCEPTANCE_APPS="$ACCEPTANCE_APPS" \
    SIFT_CANDIDATE_DIR="$SIFT_CANDIDATE_DIR" \
    "$ACCEPTANCE_ROOT/scripts/verify-clean.sh" || verify_clean_status=$?
else
  acceptance_lock_held=0
  verify_clean_status=1
  record_cleanup_failure "the cleanup session lost the shared GKE Lease"
fi
if [[ "$verify_clean_status" != "0" ]]; then
  record_cleanup_failure "the no-leftovers verification failed"
fi

if [[ "$cleanup_failures" == "0" && "$acceptance_lock_held" == "1" ]]; then
  if release_acceptance_lock; then
    acceptance_lock_held=0
  else
    record_cleanup_failure "could not safely release the shared GKE acceptance lock"
  fi
fi

if [[ "$cleanup_failures" == "0" \
  && "$acceptance_mode" == "sift" \
  && -f "$EVIDENCE_DIR/sift-mvp-verification.json" \
  && -f "$EVIDENCE_DIR/cleanup.json" ]]; then
  EVIDENCE_DIR="$EVIDENCE_DIR" \
    "$ACCEPTANCE_ROOT/scripts/finalize-sift-mvp-acceptance.sh" \
    || record_cleanup_failure "Sift terminal evidence finalization failed"
fi

if [[ "$cleanup_failures" != "0" ]]; then
  rm -f "$EVIDENCE_DIR/cleanup.json" \
    "$EVIDENCE_DIR/acceptance.json" \
    "$EVIDENCE_DIR/sift-mvp-acceptance.json"
  echo "cleanup completed all safe phases with $cleanup_failures failure(s)" >&2
  exit 1
fi
