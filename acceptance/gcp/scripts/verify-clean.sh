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

if [[ "$PERSISTENT_CLUSTER_CHECK_REQUIRED" == "1" ]]; then
  if ! gcloud container clusters describe "$PERSISTENT_CLUSTER_NAME" \
    --project="$PROJECT_ID" --zone="$GKE_ZONE" >/dev/null 2>&1; then
    echo "persistent GKE cluster is missing: $PERSISTENT_CLUSTER_NAME" >&2
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
    check_empty "Sift operator-managed auth-delegator ClusterRoleBinding" kubectl get clusterrolebinding \
      -l app.kubernetes.io/component=auth-delegation,app.kubernetes.io/name=sift --no-headers
  fi
  check_empty "Sift image tag" gcloud artifacts docker images describe \
    "$REGISTRY/sift:$IMAGE_TAG" --project="$PROJECT_ID" --format='value(image_summary.digest)'
  check_empty "Rig image tag" gcloud artifacts docker images describe \
    "$REGISTRY/rig:$IMAGE_TAG" --project="$PROJECT_ID" --format='value(image_summary.digest)'
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

shopt -s nullglob
deleted_digest_markers=("$EVIDENCE_DIR"/deleted-image-*.txt)
shopt -u nullglob
for marker in "${deleted_digest_markers[@]}"; do
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
check_empty "Cloud Build source" gcloud storage ls --recursive "${GCS_SOURCE_PREFIX}/**"

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

jq -n \
  --arg schema "axiom.gcp.operator.cleanup.v1" \
  --arg project_id "$PROJECT_ID" \
  --arg region "$REGION" \
  --arg gke_zone "$GKE_ZONE" \
  --arg run_id "$RUN_ID" \
  --arg verified_at "$(date -u +%Y-%m-%dT%H:%M:%SZ)" \
  '{schema:$schema, project_id:$project_id, region:$region, gke_zone:$gke_zone, run_id:$run_id, verified_at:$verified_at, status:"clean", preserved:{artifact_registry:true, preexisting_apis:true}}' \
  > "$EVIDENCE_DIR/cleanup.json"
if [[ "$acceptance_mode" == "tape" ]]; then
  echo "verified: no run-tagged Tape operator acceptance resources remain"
elif [[ "$acceptance_mode" == "sift" ]]; then
  echo "verified: no run-tagged Sift MVP acceptance resources remain"
else
  echo "verified: no run-tagged Lumen/Sift operator acceptance resources remain"
fi
