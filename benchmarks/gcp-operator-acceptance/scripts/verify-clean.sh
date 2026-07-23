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
LUMEN_ONLY="${LUMEN_ONLY:-0}"

prefix="axo-${RUN_ID}"
bucket="${PROJECT_ID}-${prefix}-backup"
leftovers=0

check_empty() {
  local label="$1"
  shift
  local output
  output="$("$@" 2>/dev/null || true)"
  if [[ -n "$output" ]]; then
    echo "leftover ${label}:" >&2
    echo "$output" >&2
    leftovers=1
  fi
}

if ! gcloud container clusters describe "$PERSISTENT_CLUSTER_NAME" \
  --project="$PROJECT_ID" --zone="$GKE_ZONE" >/dev/null 2>&1; then
  echo "persistent GKE cluster is missing: $PERSISTENT_CLUSTER_NAME" >&2
  leftovers=1
fi
check_empty "Lumen namespace" kubectl get namespace lumen --no-headers
check_empty "Lumen operator namespace" kubectl get namespace lumen-system --no-headers
check_empty "Lumen CRD" kubectl get customresourcedefinition lumens.lumen.dev --no-headers
if [[ "$LUMEN_ONLY" != "1" ]]; then
  check_empty "Sift namespace" kubectl get namespace sift --no-headers
  check_empty "Sift operator namespace" kubectl get namespace sift-system --no-headers
  check_empty "Sift CRD" kubectl get customresourcedefinition sifts.sift.axiom.dev --no-headers
fi
check_empty "backup bucket" gcloud storage buckets list --project="$PROJECT_ID" \
  --filter="name=${bucket}" --format='value(name)'
check_empty "node service account" gcloud iam service-accounts list --project="$PROJECT_ID" \
  --filter="email:${prefix}-node@${PROJECT_ID}.iam.gserviceaccount.com" --format='value(email)'
check_empty "backup service account" gcloud iam service-accounts list --project="$PROJECT_ID" \
  --filter="email:${prefix}-backup@${PROJECT_ID}.iam.gserviceaccount.com" --format='value(email)'
check_empty "persistent disk" gcloud compute disks list --project="$PROJECT_ID" \
  --filter="name~'${prefix}|gke-${prefix}'" --format='value(name)'
check_empty "Lumen image tag" gcloud artifacts docker images describe \
  "$REGISTRY/lumen:$IMAGE_TAG" --project="$PROJECT_ID" --format='value(image_summary.digest)'
if [[ "$LUMEN_ONLY" != "1" ]]; then
  check_empty "Sift image tag" gcloud artifacts docker images describe \
    "$REGISTRY/sift:$IMAGE_TAG" --project="$PROJECT_ID" --format='value(image_summary.digest)'
fi
check_empty "Cloud Build source" gcloud storage ls --recursive "${GCS_SOURCE_PREFIX}/**"

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
echo "verified: no run-tagged Lumen/Sift operator acceptance resources remain"
