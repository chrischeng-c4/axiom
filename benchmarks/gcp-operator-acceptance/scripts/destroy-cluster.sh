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

[[ "${CONFIRM_DESTROY_PERSISTENT_CLUSTER:-}" == "$PERSISTENT_CLUSTER_NAME" ]] || {
  echo "set CONFIRM_DESTROY_PERSISTENT_CLUSTER=$PERSISTENT_CLUSTER_NAME to destroy the reusable cluster" >&2
  exit 2
}

# The persistent cluster outlives any single run, but its Terraform state lives
# under /tmp and does not.  Once that state is gone, `terraform destroy` has
# nothing to destroy, prints "Resources: 0 destroyed", and exits 0 — a false
# green that leaves the control plane and node pool billing indefinitely.
# bootstrap-cluster.sh cannot resurrect the state either: it short-circuits on
# `clusters describe` before it ever reaches Terraform.  So compare the cluster's
# real existence against the state and refuse to report success on a no-op.
cluster_exists=0
gcloud container clusters describe "$PERSISTENT_CLUSTER_NAME" \
  --project="$PROJECT_ID" --zone="$GKE_ZONE" >/dev/null 2>&1 && cluster_exists=1

if [[ "$cluster_exists" == "0" ]]; then
  echo "cluster $PERSISTENT_CLUSTER_NAME is already absent in $PROJECT_ID/$GKE_ZONE; nothing to destroy"
  exit 0
fi

if [[ ! -s "$PERSISTENT_CLUSTER_STATE_DIR/cluster.tfstate" ]]; then
  sa="${NODE_SERVICE_ACCOUNT_ID}@${PROJECT_ID}.iam.gserviceaccount.com"
  cat >&2 <<EOF
cluster $PERSISTENT_CLUSTER_NAME exists but its Terraform state is missing:
  $PERSISTENT_CLUSTER_STATE_DIR/cluster.tfstate
Destroying now would report success while destroying nothing. Rebuild the state
by importing the five resources, then re-run this script:

  export TF_DATA_DIR=$PERSISTENT_CLUSTER_STATE_DIR/.terraform
  mkdir -p $PERSISTENT_CLUSTER_STATE_DIR
  terraform -chdir=$ACCEPTANCE_ROOT/cluster init -input=false
  vars=(-var=project_id=$PROJECT_ID -var=region=$REGION -var=gke_zone=$GKE_ZONE \\
        -var=cluster_name=$PERSISTENT_CLUSTER_NAME \\
        -var=node_service_account_id=$NODE_SERVICE_ACCOUNT_ID)
  tfi() { terraform -chdir=$ACCEPTANCE_ROOT/cluster import \\
    -state=$PERSISTENT_CLUSTER_STATE_DIR/cluster.tfstate "\${vars[@]}" "\$1" "\$2"; }
  tfi google_service_account.nodes            projects/$PROJECT_ID/serviceAccounts/$sa
  tfi google_project_iam_member.node_baseline  "$PROJECT_ID roles/container.defaultNodeServiceAccount serviceAccount:$sa"
  tfi google_project_iam_member.node_image_pull "$PROJECT_ID roles/artifactregistry.reader serviceAccount:$sa"
  tfi google_container_cluster.acceptance      projects/$PROJECT_ID/locations/$GKE_ZONE/clusters/$PERSISTENT_CLUSTER_NAME
  tfi google_container_node_pool.acceptance    projects/$PROJECT_ID/locations/$GKE_ZONE/clusters/$PERSISTENT_CLUSTER_NAME/nodePools/acceptance-pool

Import reads deletion_protection from the provider default (true), not from
main.tf (false), so the cluster leg will refuse to destroy until you set it
false in cluster.tfstate. It is client-side-only metadata — there is no API
field and no gcloud flag for it, and \`terraform apply\` cannot reconcile it
(it emits an empty update: "Error 400: Must specify a field to update").
EOF
  exit 3
fi

TF_DATA_DIR="$PERSISTENT_CLUSTER_STATE_DIR/.terraform" terraform \
  -chdir="$ACCEPTANCE_ROOT/cluster" destroy \
  -state="$PERSISTENT_CLUSTER_STATE_DIR/cluster.tfstate" -auto-approve \
  -var="project_id=$PROJECT_ID" -var="region=$REGION" -var="gke_zone=$GKE_ZONE" \
  -var="cluster_name=$PERSISTENT_CLUSTER_NAME" \
  -var="node_service_account_id=$NODE_SERVICE_ACCOUNT_ID"
