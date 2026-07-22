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

check_empty "GKE cluster" \
  gcloud container clusters list --project="$PROJECT_ID" --zone="$GKE_ZONE" \
    --filter="name=${prefix}-gke" --format='value(name)'
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
check_empty "Sift image tag" gcloud artifacts docker images describe \
  "$REGISTRY/sift:$IMAGE_TAG" --project="$PROJECT_ID" --format='value(image_summary.digest)'
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
