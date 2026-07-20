#!/usr/bin/env bash
set -euo pipefail

: "${PROJECT_ID:?PROJECT_ID is required}"
: "${REGION:?REGION is required}"
: "${RUN_ID:?RUN_ID is required}"

prefix="axb-${RUN_ID}"
long_prefix="axiom-bench-${RUN_ID}"
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

check_empty "GKE cluster" gcloud container clusters list --project="$PROJECT_ID" --filter="name:${prefix}-gke" --format='value(name)'
check_empty "Cloud Run service" gcloud run services list --project="$PROJECT_ID" --region="$REGION" --filter="metadata.name:${prefix}-receiver" --format='value(metadata.name)'
check_empty "Cloud Tasks queue" gcloud tasks queues list --project="$PROJECT_ID" --location="$REGION" --filter="name:${prefix}-defer" --format='value(name)'
check_empty "Pub/Sub topic" gcloud pubsub topics list --project="$PROJECT_ID" --filter="name:${prefix}-tape" --format='value(name)'
check_empty "Pub/Sub subscription" gcloud pubsub subscriptions list --project="$PROJECT_ID" --filter="name:${prefix}-tape" --format='value(name)'
check_empty "persistent disk" gcloud compute disks list --project="$PROJECT_ID" --filter="name~'${prefix}|gke-${prefix}'" --format='value(name)'
check_empty "reserved address" gcloud compute addresses list --project="$PROJECT_ID" --filter="name~'${prefix}|gke-${prefix}'" --format='value(name)'

if [[ "${CHECK_APIS:-1}" == "1" ]]; then
  check_empty "Artifact Registry" gcloud artifacts repositories list --project="$PROJECT_ID" --location="$REGION" --filter="name:${long_prefix}" --format='value(name)'
  check_empty "Cloud Build source archive" gcloud storage ls --recursive "gs://${PROJECT_ID}_cloudbuild/source/axiom-bench-${RUN_ID}"
  enabled="$(gcloud services list --enabled --project="$PROJECT_ID" --filter='config.name:(cloudtasks.googleapis.com OR container.googleapis.com OR file.googleapis.com OR sts.googleapis.com)' --format='value(config.name)')"
  if [[ "${ALLOW_CONTAINER_API:-0}" == "1" ]]; then
    enabled="$(printf '%s\n' "$enabled" | sed '/^container\.googleapis\.com$/d')"
    if gcloud services list --enabled --project="$PROJECT_ID" \
      --filter='config.name:container.googleapis.com' \
      --format='value(config.name)' | grep -q '^container\.googleapis\.com$'; then
      echo "warning: container.googleapis.com remains enabled while GCP asset inventory converges" >&2
    fi
  fi
  if [[ -n "$enabled" ]]; then
    echo "temporary APIs still enabled:" >&2
    echo "$enabled" >&2
    leftovers=1
  fi
fi

if [[ "$leftovers" -ne 0 ]]; then
  exit 1
fi
echo "verified: no axiom managed-benchmark cloud resources remain"
