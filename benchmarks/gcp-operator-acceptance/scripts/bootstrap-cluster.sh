#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ACCEPTANCE_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
: "${PROJECT_ID:?PROJECT_ID is required}"
REGION="${REGION:-asia-east1}"
GKE_ZONE="${GKE_ZONE:-asia-east1-a}"
PERSISTENT_CLUSTER_NAME="${PERSISTENT_CLUSTER_NAME:-axiom-operator-acceptance}"
PERSISTENT_CLUSTER_STATE_DIR="${PERSISTENT_CLUSTER_STATE_DIR:-/tmp/axiom-gcp-operator-cluster}"
NODE_SERVICE_ACCOUNT_ID="${NODE_SERVICE_ACCOUNT_ID:-axiom-operator-acceptance-node}"

if gcloud container clusters describe "$PERSISTENT_CLUSTER_NAME" \
  --project="$PROJECT_ID" --zone="$GKE_ZONE" >/dev/null 2>&1; then
  printf '%s\n' "$PERSISTENT_CLUSTER_NAME"
  exit 0
fi

mkdir -p "$PERSISTENT_CLUSTER_STATE_DIR"
TF_DATA_DIR="$PERSISTENT_CLUSTER_STATE_DIR/.terraform" terraform \
  -chdir="$ACCEPTANCE_ROOT/cluster" init -input=false
TF_DATA_DIR="$PERSISTENT_CLUSTER_STATE_DIR/.terraform" terraform \
  -chdir="$ACCEPTANCE_ROOT/cluster" apply \
  -state="$PERSISTENT_CLUSTER_STATE_DIR/cluster.tfstate" -auto-approve \
  -var="project_id=$PROJECT_ID" -var="region=$REGION" -var="gke_zone=$GKE_ZONE" \
  -var="cluster_name=$PERSISTENT_CLUSTER_NAME" \
  -var="node_service_account_id=$NODE_SERVICE_ACCOUNT_ID"
printf '%s\n' "$PERSISTENT_CLUSTER_NAME"
