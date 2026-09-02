#!/usr/bin/env bash
set -euo pipefail

# Ensure the shared persistent acceptance cluster exists, is awake (>=1 node in
# acceptance-pool), and this process can reach it. Creation is delegated to
# acceptance/gcp/scripts/bootstrap-cluster.sh so the two harnesses can never
# drift into two clusters — the free tier covers exactly one zonal cluster.
#
# stdout: the cluster name, one line, nothing else (same contract as
# bootstrap-cluster.sh; all chatter goes to stderr).

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/../../.." && pwd)"
: "${PROJECT_ID:?PROJECT_ID is required}"
GKE_ZONE="${GKE_ZONE:-asia-east1-a}"
PERSISTENT_CLUSTER_NAME="${PERSISTENT_CLUSTER_NAME:-axiom-operator-acceptance}"
ACCEPTANCE_POOL_NAME="${ACCEPTANCE_POOL_NAME:-acceptance-pool}"
# A task-local kubeconfig is required so this run never rewrites the caller's
# real one — in CI it also keeps parallel jobs from racing on ~/.kube/config.
: "${KUBECONFIG:?KUBECONFIG must point at a task-local file}"

cluster_name="$("$REPO_ROOT/acceptance/gcp/scripts/bootstrap-cluster.sh")"
[[ "$cluster_name" == "$PERSISTENT_CLUSTER_NAME" ]] || {
  echo "bootstrap-cluster.sh returned '$cluster_name', expected '$PERSISTENT_CLUSTER_NAME'" >&2
  exit 1
}

gcloud container clusters get-credentials "$cluster_name" \
  --project="$PROJECT_ID" --zone="$GKE_ZONE" >&2

# Wake from park: the pool sits at 0 nodes between runs (park.sh), and the
# autoscaler cannot scale up a pool that terraform/park left at zero fast
# enough to matter — resize to 1 explicitly, then let the autoscaler own 1..max.
ready_nodes() {
  kubectl get nodes \
    -l "cloud.google.com/gke-nodepool=$ACCEPTANCE_POOL_NAME" -o json 2>/dev/null \
    | jq '[.items[] | select(any(.status.conditions[]; .type == "Ready" and .status == "True"))] | length'
}

if (( "$(ready_nodes)" == 0 )); then
  echo "waking $ACCEPTANCE_POOL_NAME from park (resize 0 -> 1)" >&2
  gcloud container clusters resize "$cluster_name" \
    --project="$PROJECT_ID" --zone="$GKE_ZONE" \
    --node-pool="$ACCEPTANCE_POOL_NAME" --num-nodes=1 --quiet >&2
fi

deadline=$((SECONDS + 600))
until (( "$(ready_nodes)" >= 1 )); do
  if (( SECONDS >= deadline )); then
    echo "timed out waiting for a Ready node in $ACCEPTANCE_POOL_NAME" >&2
    kubectl get nodes -o wide >&2 || true
    exit 1
  fi
  sleep 10
done

printf '%s\n' "$cluster_name"
