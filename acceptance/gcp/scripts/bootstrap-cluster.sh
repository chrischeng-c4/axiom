#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ACCEPTANCE_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
: "${PROJECT_ID:?PROJECT_ID is required}"
REGION="${REGION:-asia-east1}"
GKE_ZONE="${GKE_ZONE:-asia-east1-a}"
PERSISTENT_CLUSTER_NAME="${PERSISTENT_CLUSTER_NAME:-axiom-operator-acceptance}"
NODE_SERVICE_ACCOUNT_ID="${NODE_SERVICE_ACCOUNT_ID:-axiom-operator-acceptance-node}"
# State lives in GCS (see cluster/main.tf backend block); the default bucket
# name matches acceptance/gke-harness/bootstrap's derivation. TF_DATA_DIR still
# points at /tmp so provider downloads and backend config never land in the
# repo tree — unlike the state that used to live beside it, both are
# reproducible, so losing this directory costs one `init`.
TFSTATE_BUCKET="${TFSTATE_BUCKET:-${PROJECT_ID}-axiom-tfstate}"
CLUSTER_TF_DATA_DIR="${CLUSTER_TF_DATA_DIR:-/tmp/axiom-gcp-operator-cluster/.terraform}"

if cluster_json="$(gcloud container clusters describe "$PERSISTENT_CLUSTER_NAME" \
  --project="$PROJECT_ID" --zone="$GKE_ZONE" --format=json 2>/dev/null)"; then
  datapath_provider="$(jq -r '.datapathProvider // ""' <<<"$cluster_json")"
  fqdn_network_policy="$(jq -r '.enableFqdnNetworkPolicy // false' <<<"$cluster_json")"
  if [[ "$datapath_provider" != "ADVANCED_DATAPATH" || "$fqdn_network_policy" != "true" ]]; then
    echo "$PERSISTENT_CLUSTER_NAME requires Dataplane V2 and FQDN Network Policy; got datapathProvider=${datapath_provider:-unset}, enableFqdnNetworkPolicy=$fqdn_network_policy" >&2
    echo "Destroy and recreate this empty acceptance cluster from acceptance/gcp/cluster/main.tf before retrying." >&2
    exit 1
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
    echo "  TF_DATA_DIR=$CLUSTER_TF_DATA_DIR terraform -chdir=$ACCEPTANCE_ROOT/cluster init -input=false -backend-config=bucket=$TFSTATE_BUCKET" >&2
    echo "  TF_DATA_DIR=$CLUSTER_TF_DATA_DIR terraform -chdir=$ACCEPTANCE_ROOT/cluster apply -auto-approve \\" >&2
    echo "    -var=project_id=$PROJECT_ID -var=region=$REGION -var=gke_zone=$GKE_ZONE \\" >&2
    echo "    -var=cluster_name=$PERSISTENT_CLUSTER_NAME -var=node_service_account_id=$NODE_SERVICE_ACCOUNT_ID" >&2
  fi
  # Third drift check, and the one the other two would have missed: a node pool
  # can be present, correctly named, and still too small. cluster/main.tf raises
  # acceptance-pool to max 3 because required hostname anti-affinity gives the
  # 2-voter quorum leg (#2610) nowhere to put its second replica once the main
  # `lumen` instance has saturated a node -- but reuse never re-applies
  # terraform, so a cluster bootstrapped before that change keeps max 2 forever.
  # At max 2 the leg is a coin flip on how the scheduler happened to spread
  # kube-dns, and it fails as "timed out waiting for Ready" twenty minutes in,
  # indistinguishable from a product regression. Name it here for nothing.
  acceptance_pool="${ACCEPTANCE_POOL_NAME:-acceptance-pool}"
  acceptance_pool_max="$(jq -r --arg p "$acceptance_pool" \
    'first(.nodePools[]? | select(.name == $p) | .autoscaling.maxNodeCount) // 0' <<<"$cluster_json")"
  if (( acceptance_pool_max < 3 )); then
    echo "WARNING: $PERSISTENT_CLUSTER_NAME '$acceptance_pool' allows max $acceptance_pool_max nodes; the #2610 quorum leg needs 3." >&2
    echo "  raise it in place (no recreation, no restart, ~1 min):" >&2
    echo "  gcloud container clusters update $PERSISTENT_CLUSTER_NAME --project=$PROJECT_ID --zone=$GKE_ZONE \\" >&2
    echo "    --node-pool=$acceptance_pool --enable-autoscaling --min-nodes=1 --max-nodes=3" >&2
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
mkdir -p "$CLUSTER_TF_DATA_DIR"
TF_DATA_DIR="$CLUSTER_TF_DATA_DIR" terraform \
  -chdir="$ACCEPTANCE_ROOT/cluster" init -input=false \
  -backend-config="bucket=$TFSTATE_BUCKET" >&2
TF_DATA_DIR="$CLUSTER_TF_DATA_DIR" terraform \
  -chdir="$ACCEPTANCE_ROOT/cluster" apply -auto-approve \
  -var="project_id=$PROJECT_ID" -var="region=$REGION" -var="gke_zone=$GKE_ZONE" \
  -var="cluster_name=$PERSISTENT_CLUSTER_NAME" \
  -var="node_service_account_id=$NODE_SERVICE_ACCOUNT_ID" >&2
printf '%s\n' "$PERSISTENT_CLUSTER_NAME"
