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

if cluster_json="$(gcloud container clusters describe "$PERSISTENT_CLUSTER_NAME" \
  --project="$PROJECT_ID" --zone="$GKE_ZONE" --format=json 2>/dev/null)"; then
  # Reuse never re-applies terraform, so a long-lived cluster silently drifts
  # from cluster/main.tf. The add-on below is the drift that already cost us:
  # it was enabled by hand, the cluster was recreated without it, and the #2457
  # auth+CSI leg degraded to `skipped_no_addon` FORTY MINUTES into a paid run
  # instead of failing in the first ten seconds. Report it here, on stderr —
  # stdout is the cluster name and nothing else — where it is still cheap.
  if [[ "$(jq -r '.secretManagerConfig.enabled // false' <<<"$cluster_json")" != "true" ]]; then
    echo "WARNING: $PERSISTENT_CLUSTER_NAME has no GKE Secret Manager add-on; the #2457 auth+CSI leg will skip." >&2
    echo "  enable it in place (no recreation, ~2 min):" >&2
    echo "  gcloud container clusters update $PERSISTENT_CLUSTER_NAME --project=$PROJECT_ID --zone=$GKE_ZONE --enable-secret-manager" >&2
  fi
  # Same drift class, same cheap place. The dedicated data-plane pool is what
  # gives `spec.placement` a real pool boundary to be proven against; without it
  # the placement leg hard-fails, but only after the legs before it have already
  # been paid for. Say so here, where the run has cost nothing yet.
  data_plane_pool="${DATA_PLANE_POOL_NAME:-data-plane-pool}"
  if [[ "$(jq -r --arg p "$data_plane_pool" \
    '[.nodePools[]? | select(.name == $p)] | length' <<<"$cluster_json")" != "1" ]]; then
    echo "WARNING: $PERSISTENT_CLUSTER_NAME has no '$data_plane_pool' node pool; the spec.placement leg will fail." >&2
    echo "  it is declared in acceptance/gcp/cluster/main.tf; add it in place (scale-to-zero, free at rest):" >&2
    echo "  TF_DATA_DIR=$PERSISTENT_CLUSTER_STATE_DIR/.terraform terraform -chdir=$ACCEPTANCE_ROOT/cluster apply \\" >&2
    echo "    -state=$PERSISTENT_CLUSTER_STATE_DIR/cluster.tfstate -auto-approve \\" >&2
    echo "    -var=project_id=$PROJECT_ID -var=region=$REGION -var=gke_zone=$GKE_ZONE \\" >&2
    echo "    -var=cluster_name=$PERSISTENT_CLUSTER_NAME -var=node_service_account_id=$NODE_SERVICE_ACCOUNT_ID" >&2
  fi
  printf '%s\n' "$PERSISTENT_CLUSTER_NAME"
  exit 0
fi

# stdout is a CONTRACT: exactly the cluster name, one line, nothing else.
# The caller captures it into an evidence file and asserts on it, so terraform's
# plan and progress chatter goes to stderr where a human still sees it.
#
# It used to go to stdout, and only on this branch -- the reuse branch above
# prints the name and nothing more. So the run that had to create a cluster
# wrote ~19KB of terraform plan into persistent-cluster-name.txt, the caller's
# `test` on line 1 compared "Initializing the backend..." against the cluster
# name, and the run died with NO message at all (`test` prints nothing on
# failure) -- after paying the full ~10 minutes of cluster creation. Every
# prior run reused an existing cluster, so this path had never once run.
mkdir -p "$PERSISTENT_CLUSTER_STATE_DIR"
TF_DATA_DIR="$PERSISTENT_CLUSTER_STATE_DIR/.terraform" terraform \
  -chdir="$ACCEPTANCE_ROOT/cluster" init -input=false >&2
TF_DATA_DIR="$PERSISTENT_CLUSTER_STATE_DIR/.terraform" terraform \
  -chdir="$ACCEPTANCE_ROOT/cluster" apply \
  -state="$PERSISTENT_CLUSTER_STATE_DIR/cluster.tfstate" -auto-approve \
  -var="project_id=$PROJECT_ID" -var="region=$REGION" -var="gke_zone=$GKE_ZONE" \
  -var="cluster_name=$PERSISTENT_CLUSTER_NAME" \
  -var="node_service_account_id=$NODE_SERVICE_ACCOUNT_ID" >&2
printf '%s\n' "$PERSISTENT_CLUSTER_NAME"
