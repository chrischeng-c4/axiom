#!/usr/bin/env bash
set -euo pipefail

: "${PROJECT_ID:?PROJECT_ID is required}"
: "${REGION:?REGION is required}"
: "${GKE_ZONE:?GKE_ZONE is required}"
: "${RUN_ID:?RUN_ID is required}"
: "${REGISTRY:?REGISTRY is required}"
: "${IMAGE_TAG:?IMAGE_TAG is required}"
: "${GCS_SOURCE_PREFIX:?GCS_SOURCE_PREFIX is required}"
: "${EVIDENCE_DIR:?EVIDENCE_DIR is required}"
PERSISTENT_CLUSTER_NAME="${PERSISTENT_CLUSTER_NAME:-axiom-operator-acceptance}"
ACCEPTANCE_APPS="${ACCEPTANCE_APPS:-lumen sift}"
PERSISTENT_CLUSTER_CHECK_REQUIRED="${PERSISTENT_CLUSTER_CHECK_REQUIRED:-1}"
KUBERNETES_CHECK_REQUIRED="${KUBERNETES_CHECK_REQUIRED:-1}"
SIFT_CANDIDATE_CONTROL_OBJECTS_ALLOWED="${SIFT_CANDIDATE_CONTROL_OBJECTS_ALLOWED:-0}"
VERIFY_CLEAN_WRITE_RECEIPT="${VERIFY_CLEAN_WRITE_RECEIPT:-1}"
SIFT_CANDIDATE_DIR="${SIFT_CANDIDATE_DIR:-}"
source "$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/source-prefix.sh"
source "$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/acceptance-lock.sh"
source "$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/sift-candidate.sh"

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
[[ "$SIFT_CANDIDATE_CONTROL_OBJECTS_ALLOWED" =~ ^[01]$ \
  && "$VERIFY_CLEAN_WRITE_RECEIPT" =~ ^[01]$ ]] || {
  echo "verify-clean mode flags must be 0 or 1" >&2
  exit 1
}
if [[ "$SIFT_CANDIDATE_CONTROL_OBJECTS_ALLOWED" == "1" \
  && "$acceptance_mode" != "sift" ]]; then
  echo "candidate control objects are allowed only in Sift mode" >&2
  exit 1
fi
if [[ "$SIFT_CANDIDATE_CONTROL_OBJECTS_ALLOWED" == "1" \
  && "$VERIFY_CLEAN_WRITE_RECEIPT" != "0" ]]; then
  echo "candidate control objects cannot produce a clean receipt" >&2
  exit 1
fi

sift_candidate_verification_dir=""
if [[ "$acceptance_mode" == "sift" ]]; then
  if [[ -n "$SIFT_CANDIDATE_DIR" ]]; then
    sift_candidate_verification_dir="$SIFT_CANDIDATE_DIR"
  elif [[ -e "$EVIDENCE_DIR/candidate.json" \
    || -L "$EVIDENCE_DIR/candidate.json" ]]; then
    sift_candidate_verification_dir="$EVIDENCE_DIR"
  fi
fi

prefix="axo-${RUN_ID}"
bucket="${PROJECT_ID}-${prefix}-backup"
leftovers=0

inventory_output() {
  local label="$1"
  shift
  local error_file output
  error_file="$(mktemp "${TMPDIR:-/tmp}/sift-verify-clean.XXXXXX")"
  if output="$("$@" 2>"$error_file")"; then
    rm -f "$error_file"
    printf '%s' "$output"
    return 0
  fi
  if grep -Eiq '(^|[^[:alpha:]])(not[ _-]?found|404)([^[:alpha:]]|$)|matched no (objects|URLs)|no URLs matched' \
      "$error_file"; then
    rm -f "$error_file"
    return 0
  fi
  echo "could not verify cleanup for ${label}:" >&2
  cat "$error_file" >&2
  rm -f "$error_file"
  return 1
}

check_empty() {
  local label="$1"
  shift
  local output
  if ! output="$(inventory_output "$label" "$@")"; then
    leftovers=1
    return 0
  fi
  if [[ -n "$output" ]]; then
    echo "leftover ${label}:" >&2
    echo "$output" >&2
    leftovers=1
  fi
}

wait_for_empty() {
  local label="$1"
  shift
  local deadline=$((SECONDS + 90))
  local output
  while true; do
    if ! output="$(inventory_output "$label" "$@")"; then
      leftovers=1
      return 0
    fi
    [[ -z "$output" ]] && return 0
    if (( SECONDS >= deadline )); then
      echo "leftover ${label} after 90-second propagation wait:" >&2
      echo "$output" >&2
      leftovers=1
      return 0
    fi
    sleep 5
  done
}

source_prefix_receipt="$EVIDENCE_DIR/source-prefix.json"
source_prefix_receipt_valid=0
if ! validated_source_bucket "$GCS_SOURCE_PREFIX" "$RUN_ID" >/dev/null; then
  echo "unsafe Cloud Build source prefix; refusing to inspect it" >&2
  leftovers=1
elif verify_source_prefix_receipt \
    "$source_prefix_receipt" "$PROJECT_ID" "$RUN_ID" "$GCS_SOURCE_PREFIX"; then
  if verify_cloud_build_source_evidence \
      "$EVIDENCE_DIR" "$GCS_SOURCE_PREFIX" "$RUN_ID"; then
    source_prefix_receipt_valid=1
  else
    echo "Cloud Build source evidence is outside the run-scoped prefix" >&2
    leftovers=1
  fi
elif [[ -e "$source_prefix_receipt" ]]; then
  echo "invalid Cloud Build source-prefix receipt" >&2
  leftovers=1
elif [[ "$acceptance_mode" == "sift" || -f "$EVIDENCE_DIR/cloud-build-submit.json" ]]; then
  echo "missing Cloud Build source-prefix receipt" >&2
  leftovers=1
fi

if [[ "$PERSISTENT_CLUSTER_CHECK_REQUIRED" == "1" ]]; then
  if ! gcloud container clusters describe "$PERSISTENT_CLUSTER_NAME" \
    --project="$PROJECT_ID" --zone="$GKE_ZONE" >/dev/null 2>&1; then
    echo "persistent GKE cluster is missing: $PERSISTENT_CLUSTER_NAME" >&2
    leftovers=1
  fi
fi

if [[ "$KUBERNETES_CHECK_REQUIRED" == "1" ]]; then
  acceptance_lock_receipt="$EVIDENCE_DIR/acceptance-lock.json"
  if acceptance_lock_resource="$(kubectl get lease "$(acceptance_lock_name)" \
      --namespace "$(acceptance_lock_namespace)" -o json)" \
      && verify_acceptance_lock_receipt \
        "$acceptance_lock_receipt" "$acceptance_lock_resource" \
        "$PROJECT_ID" "$RUN_ID" "$acceptance_mode"; then
    :
  else
    echo "the shared GKE acceptance lock is not owned by this run" >&2
    leftovers=1
  fi
fi

if [[ "$acceptance_mode" == "sift" ]]; then
  if [[ "$KUBERNETES_CHECK_REQUIRED" == "1" ]]; then
    check_empty "Sift namespace" kubectl get namespace sift --no-headers
    check_empty "Sift restore namespace" kubectl get namespace sift-restore --no-headers
    check_empty "Sift operator namespace" kubectl get namespace sift-system --no-headers
    check_empty "Sift CRD" kubectl get customresourcedefinition sifts.sift.axiom.dev --no-headers
    check_empty "Sift auth-delegator ClusterRoleBinding" kubectl get clusterrolebinding \
      -l axiom-owner=gcp-operator-acceptance,axiom-run-id="$RUN_ID" --no-headers
    check_empty "primary Sift auth-delegator ClusterRoleBinding" kubectl get clusterrolebinding \
      sift.sift.sift.auth-delegator --no-headers
    check_empty "restored Sift auth-delegator ClusterRoleBinding" kubectl get clusterrolebinding \
      sift.sift-restore.sift-restore.auth-delegator --no-headers
  fi
  check_empty "Sift image tag" gcloud artifacts docker images describe \
    "$REGISTRY/sift:$IMAGE_TAG" --project="$PROJECT_ID" --format='value(image_summary.digest)'
  check_empty "Rig image tag" gcloud artifacts docker images describe \
    "$REGISTRY/rig:$IMAGE_TAG" --project="$PROJECT_ID" --format='value(image_summary.digest)'
  check_empty "Sift acceptance runner image tag" gcloud artifacts docker images describe \
    "$REGISTRY/sift-acceptance-runner:$IMAGE_TAG" --project="$PROJECT_ID" \
    --format='value(image_summary.digest)'
  check_empty "Sift run node pool" gcloud container node-pools describe "axo-${RUN_ID}-sift" \
    --cluster="$PERSISTENT_CLUSTER_NAME" --project="$PROJECT_ID" --zone="$GKE_ZONE" --format='value(name)'
elif [[ "$acceptance_mode" == "tape" ]]; then
  if [[ "$KUBERNETES_CHECK_REQUIRED" == "1" ]]; then
    check_empty "Tape namespace" kubectl get namespace tape --no-headers
    check_empty "Tape operator namespace" kubectl get namespace tape-system --no-headers
    check_empty "Tape CRD" kubectl get customresourcedefinition tapes.tape.dev --no-headers
  fi
  check_empty "Tape image tag" gcloud artifacts docker images describe \
    "$REGISTRY/tape:$IMAGE_TAG" --project="$PROJECT_ID" --format='value(image_summary.digest)'
elif [[ "$acceptance_mode" == "lumen-auth" ]]; then
  if [[ "$KUBERNETES_CHECK_REQUIRED" == "1" ]]; then
    check_empty "Lumen namespace" kubectl get namespace lumen --no-headers
    check_empty "Lumen operator namespace" kubectl get namespace lumen-system --no-headers
    check_empty "Lumen CRD" kubectl get customresourcedefinition lumens.lumen.dev --no-headers
    check_empty "Lumen auth client namespace" kubectl get namespace lumen-auth-client --no-headers
    check_empty "Lumen auth-delegator ClusterRoleBinding" kubectl get clusterrolebinding \
      -l app.kubernetes.io/component=auth-delegation,app.kubernetes.io/name=lumen --no-headers
  fi
  check_empty "Lumen image tag" gcloud artifacts docker images describe \
    "$REGISTRY/lumen:$IMAGE_TAG" --project="$PROJECT_ID" --format='value(image_summary.digest)'
else
  if [[ "$KUBERNETES_CHECK_REQUIRED" == "1" ]]; then
    check_empty "Lumen namespace" kubectl get namespace lumen --no-headers
    check_empty "Lumen operator namespace" kubectl get namespace lumen-system --no-headers
    check_empty "Lumen CRD" kubectl get customresourcedefinition lumens.lumen.dev --no-headers
    # The fleet is cluster-scoped and materializes into namespaces of its own.
    check_empty "LumenFleet CRD" kubectl get customresourcedefinition lumenfleets.lumen.dev --no-headers
    check_empty "Lumen fleet namespace a" kubectl get namespace lumen-fleet-a --no-headers
    check_empty "Lumen fleet namespace b" kubectl get namespace lumen-fleet-b --no-headers
    check_empty "Lumen auth client namespace" kubectl get namespace lumen-auth-client --no-headers
    check_empty "Lumen auth-delegator ClusterRoleBinding" kubectl get clusterrolebinding \
      -l app.kubernetes.io/component=auth-delegation,app.kubernetes.io/name=lumen --no-headers
    check_empty "Sift namespace" kubectl get namespace sift --no-headers
    check_empty "Sift operator namespace" kubectl get namespace sift-system --no-headers
    check_empty "Sift CRD" kubectl get customresourcedefinition sifts.sift.axiom.dev --no-headers
  fi
  check_empty "auth+CSI Secret Manager secret" gcloud secrets list --project="$PROJECT_ID" \
    --filter="name:${prefix}-lumen-tokens" --format='value(name)'
  check_empty "Lumen image tag" gcloud artifacts docker images describe \
    "$REGISTRY/lumen:$IMAGE_TAG" --project="$PROJECT_ID" --format='value(image_summary.digest)'
  check_empty "Sift image tag" gcloud artifacts docker images describe \
    "$REGISTRY/sift:$IMAGE_TAG" --project="$PROJECT_ID" --format='value(image_summary.digest)'
fi

for marker in "$EVIDENCE_DIR"/deleted-image-*.txt; do
  [[ -f "$marker" ]] || continue
  digest_ref="$(sed -n '1p' "$marker")"
  if [[ "$digest_ref" != *@sha256:* ]]; then
    echo "invalid deleted image digest marker: $marker" >&2
    leftovers=1
    continue
  fi
  check_empty "deleted image digest $digest_ref" \
    gcloud artifacts docker images describe "$digest_ref" \
      --project="$PROJECT_ID" --format='value(image_summary.digest)'
done

if [[ "$acceptance_mode" != "lumen-auth" ]]; then
  check_empty "backup bucket" gcloud storage buckets list --project="$PROJECT_ID" \
    --filter="name=${bucket}" --format='value(name)'
fi
check_empty "node service account" gcloud iam service-accounts list --project="$PROJECT_ID" \
  --filter="email:${prefix}-node@${PROJECT_ID}.iam.gserviceaccount.com" --format='value(email)'
if [[ "$acceptance_mode" != "lumen-auth" ]]; then
  wait_for_empty "backup service account" gcloud iam service-accounts list --project="$PROJECT_ID" \
    --filter="email:${prefix}-backup@${PROJECT_ID}.iam.gserviceaccount.com" --format='value(email)'
fi
check_empty "persistent disk" gcloud compute disks list --project="$PROJECT_ID" \
  --filter="name~'${prefix}|gke-${prefix}'" --format='value(name)'
if [[ "$source_prefix_receipt_valid" == "1" ]]; then
  if [[ "$SIFT_CANDIDATE_CONTROL_OBJECTS_ALLOWED" == "1" ]]; then
    source_inventory=""
    if ! source_inventory="$(inventory_output "Cloud Build source" \
        gcloud storage ls --recursive "${GCS_SOURCE_PREFIX}/**")"; then
      leftovers=1
    elif [[ -n "$source_inventory" ]]; then
      reservation_uri="${GCS_SOURCE_PREFIX}/candidate-reservation.json"
      submit_intent_uri="$(sift_candidate_submit_intent_uri "$GCS_SOURCE_PREFIX")"
      while IFS= read -r source_uri; do
        [[ -n "$source_uri" ]] || continue
        if [[ "$source_uri" != "$reservation_uri" \
          && "$source_uri" != "$submit_intent_uri" ]]; then
          echo "leftover Cloud Build source outside candidate controls:" >&2
          echo "$source_uri" >&2
          leftovers=1
        fi
      done <<<"$source_inventory"
    fi
  else
    check_empty "Cloud Build source" gcloud storage ls --recursive "${GCS_SOURCE_PREFIX}/**"
  fi
fi

run_builds=""
if ! run_builds="$(inventory_output "run-tagged Cloud Builds" \
  gcloud builds list --project="$PROJECT_ID" --region="$REGION" \
    --filter="tags=axiom-run-${RUN_ID}" --format=json)"; then
  leftovers=1
elif ! jq -e '
  all(.[]; (.status | IN("SUCCESS","FAILURE","INTERNAL_ERROR","TIMEOUT","CANCELLED","EXPIRED")))
' >/dev/null <<<"$run_builds"; then
  echo "a run-tagged Cloud Build is still active or has an unknown status:" >&2
  jq -r '.[] | "\(.id) \(.status)"' <<<"$run_builds" >&2 || true
  leftovers=1
fi
if [[ "$acceptance_mode" == "sift" \
  && -n "$sift_candidate_verification_dir" ]]; then
  if ! verify_sift_candidate_directory "$sift_candidate_verification_dir"; then
    echo "the final cleanup has no valid Sift candidate receipt" >&2
    leftovers=1
  else
    immutable_candidate_receipt="$sift_candidate_verification_dir/candidate.json"
    expected_build_id="$(jq -er '.cloud_build_id' \
      "$immutable_candidate_receipt")"
    candidate_acquisition_id="$(jq -er '.acquisition_id' \
      "$immutable_candidate_receipt")"
    candidate_build_receipt="$(mktemp \
      "${TMPDIR:-/tmp}/sift-final-candidate-build.XXXXXX")"
    acquisition_builds=""
    if ! acquisition_builds="$(inventory_output \
        "Sift acquisition-tagged Cloud Builds" \
        gcloud builds list --project="$PROJECT_ID" --region="$REGION" \
          --filter="tags=axiom-acquisition-${candidate_acquisition_id}" \
          --format=json)"; then
      leftovers=1
    elif ! jq -e --arg expected "$expected_build_id" '
        type == "array"
        and ([.[].id] | sort) == [$expected]
        and all(.[];
          (.status | IN("SUCCESS","FAILURE","INTERNAL_ERROR","TIMEOUT","CANCELLED","EXPIRED")))
      ' >/dev/null <<<"$run_builds" \
        || ! jq -e --arg expected "$expected_build_id" '
          type == "array"
          and ([.[].id] | sort) == [$expected]
          and all(.[];
            (.status | IN("SUCCESS","FAILURE","INTERNAL_ERROR","TIMEOUT","CANCELLED","EXPIRED")))
        ' >/dev/null <<<"$acquisition_builds"; then
      echo "final Cloud Build inventory does not match the one Sift candidate build" >&2
      leftovers=1
    fi
    if ! gcloud builds describe "$expected_build_id" \
        --project="$PROJECT_ID" --region="$REGION" --format=json \
        > "$candidate_build_receipt" \
        || ! verify_sift_candidate_build_receipt \
          "$immutable_candidate_receipt" "$candidate_build_receipt"; then
      echo "final Cloud Build resource does not match the immutable Sift candidate" >&2
      leftovers=1
    fi
    rm -f "$candidate_build_receipt"
  fi
fi

# The repository and APIs predate this run and are deliberately not Terraform
# resources. Their continued presence is part of cleanup acceptance.
gcloud artifacts repositories describe "${REGISTRY##*/}" \
  --project="$PROJECT_ID" --location="$REGION" >/dev/null
if [[ -f "$EVIDENCE_DIR/preexisting-apis.txt" ]]; then
  while IFS= read -r api; do
    [[ -z "$api" ]] && continue
    if ! gcloud services list --enabled --project="$PROJECT_ID" \
      --filter="config.name=${api}" --format='value(config.name)' | grep -qx "$api"; then
      echo "pre-existing API was not preserved: $api" >&2
      leftovers=1
    fi
  done < "$EVIDENCE_DIR/preexisting-apis.txt"
fi

if [[ "$leftovers" -ne 0 ]]; then
  exit 1
fi

if [[ "$VERIFY_CLEAN_WRITE_RECEIPT" == "0" ]]; then
  echo "verified: no run resources remain outside Sift candidate controls"
  exit 0
fi

cleanup_candidate=null
if [[ "$acceptance_mode" == "sift" \
  && -f "$EVIDENCE_DIR/sift-mvp-verification.json" \
  && ! -L "$EVIDENCE_DIR/sift-mvp-verification.json" ]]; then
  candidate_component_dir="$EVIDENCE_DIR"
  if [[ -n "$sift_candidate_verification_dir" ]]; then
    candidate_component_dir="$sift_candidate_verification_dir"
  fi
  for candidate_receipt in \
    "$EVIDENCE_DIR/run.json" \
    "$candidate_component_dir/candidate-source.json" \
    "$candidate_component_dir/candidate-gate.json" \
    "$candidate_component_dir/cloud-build-source-binding.json" \
    "$candidate_component_dir/images.json"; do
    [[ -f "$candidate_receipt" && ! -L "$candidate_receipt" ]] || {
      echo "candidate cleanup receipt input is missing or unsafe: $candidate_receipt" >&2
      exit 1
    }
  done
  cleanup_candidate="$(jq -n \
    --slurpfile run "$EVIDENCE_DIR/run.json" \
    --slurpfile source "$candidate_component_dir/candidate-source.json" \
    --slurpfile gate "$candidate_component_dir/candidate-gate.json" \
    --slurpfile build "$candidate_component_dir/cloud-build-source-binding.json" \
    --slurpfile images "$candidate_component_dir/images.json" '
      ($run[0].git_sha) as $git_sha
      | ($source[0].source_bundle_sha256) as $source_sha
      | if ($git_sha | type) != "string" or ($git_sha | test("^[0-9a-f]{40}$") | not)
          or ($source_sha | type) != "string" or ($source_sha | test("^[0-9a-f]{64}$") | not)
          or $source[0].git_sha != $git_sha
          or $gate[0].schema != "axiom.gcp.sift.candidate-gate.v1"
          or $gate[0].git_sha != $git_sha
          or $gate[0].source_bundle_sha256 != $source_sha
          or $gate[0].entrypoint != "apps/sift/test.sh --candidate"
          or $gate[0].status != "passed"
          or $build[0].git_sha != $git_sha
          or $build[0].source_bundle_sha256 != $source_sha
          or $build[0].staged_source_sha256 != $source_sha
          or (($build[0].build_id | type) != "string")
          or (($build[0].source_uri | type) != "string")
          or (($images[0].sift | type) != "string")
          or (($images[0].rig | type) != "string")
          or (($images[0].acceptance_runner | type) != "string")
          or (($images[0].sift | test("@sha256:[0-9a-f]{64}$")) | not)
          or (($images[0].rig | test("@sha256:[0-9a-f]{64}$")) | not)
          or (($images[0].acceptance_runner | test("@sha256:[0-9a-f]{64}$")) | not)
        then error("candidate cleanup receipt inputs do not describe one immutable build")
        else {
          sift_image:$images[0].sift,
          rig_image:$images[0].rig,
          acceptance_runner_image:$images[0].acceptance_runner,
          git_sha:$git_sha,
          source_bundle_sha256:$source_sha,
          cloud_build_id:$build[0].build_id,
          source_object_uri:$build[0].source_uri,
          immutable:true
        }
        end
    ')" || exit 1
elif [[ "$acceptance_mode" == "sift" \
  && -e "$EVIDENCE_DIR/sift-mvp-verification.json" ]]; then
  echo "Sift MVP verification receipt is not a regular file" >&2
  exit 1
fi

jq -n \
  --arg schema "axiom.gcp.operator.cleanup.v1" \
  --arg project_id "$PROJECT_ID" \
  --arg region "$REGION" \
  --arg gke_zone "$GKE_ZONE" \
  --arg run_id "$RUN_ID" \
  --arg verified_at "$(date -u +%Y-%m-%dT%H:%M:%SZ)" \
  --argjson candidate "$cleanup_candidate" \
  '{schema:$schema, project_id:$project_id, region:$region, gke_zone:$gke_zone, run_id:$run_id, verified_at:$verified_at, status:"clean", preserved:{artifact_registry:true, preexisting_apis:true}}
   + (if $candidate == null then {} else {candidate:$candidate} end)' \
  > "$EVIDENCE_DIR/cleanup.json"
if [[ "$acceptance_mode" == "tape" ]]; then
  echo "verified: no run-tagged Tape operator acceptance resources remain"
elif [[ "$acceptance_mode" == "sift" ]]; then
  echo "verified: no run-tagged Sift MVP acceptance resources remain"
else
  echo "verified: no run-tagged Lumen/Sift operator acceptance resources remain"
fi
