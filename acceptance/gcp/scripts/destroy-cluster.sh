#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ACCEPTANCE_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
: "${PROJECT_ID:?PROJECT_ID is required}"
REGION="${REGION:-asia-east1}"
GKE_ZONE="${GKE_ZONE:-asia-east1-a}"
PERSISTENT_CLUSTER_NAME="${PERSISTENT_CLUSTER_NAME:-axiom-operator-acceptance}"
NODE_SERVICE_ACCOUNT_ID="${NODE_SERVICE_ACCOUNT_ID:-axiom-operator-acceptance-node}"
TFSTATE_BUCKET="${TFSTATE_BUCKET:-${PROJECT_ID}-axiom-tfstate}"
CLUSTER_TF_DATA_DIR="${CLUSTER_TF_DATA_DIR:-/tmp/axiom-gcp-operator-cluster/.terraform}"

[[ "${CONFIRM_DESTROY_PERSISTENT_CLUSTER:-}" == "$PERSISTENT_CLUSTER_NAME" ]] || {
  echo "set CONFIRM_DESTROY_PERSISTENT_CLUSTER=$PERSISTENT_CLUSTER_NAME to destroy the reusable cluster" >&2
  exit 2
}

# The persistent cluster outlives any single run. Its Terraform state used to
# live under /tmp and did not — once that state was gone, `terraform destroy`
# had nothing to destroy, printed "Resources: 0 destroyed", and exited 0: a
# false green that leaves the control plane and node pool billing indefinitely.
# State now lives in versioned GCS (cluster/main.tf backend block), which ends
# the /tmp failure mode, but an empty backend prefix (fresh bucket, wrong
# bucket, state removed by hand) still destroys nothing while exiting 0.
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

mkdir -p "$CLUSTER_TF_DATA_DIR"
TF_DATA_DIR="$CLUSTER_TF_DATA_DIR" terraform \
  -chdir="$ACCEPTANCE_ROOT/cluster" init -input=false \
  -backend-config="bucket=$TFSTATE_BUCKET" >&2

# `state list` exits 0 with empty output on an empty-but-reachable backend, and
# non-zero when the backend itself cannot be read — both must block the destroy.
state_rows="$(TF_DATA_DIR="$CLUSTER_TF_DATA_DIR" terraform \
  -chdir="$ACCEPTANCE_ROOT/cluster" state list 2>/dev/null || true)"
if [[ -z "$state_rows" ]]; then
  sa="${NODE_SERVICE_ACCOUNT_ID}@${PROJECT_ID}.iam.gserviceaccount.com"
  cat >&2 <<EOF
cluster $PERSISTENT_CLUSTER_NAME exists but the Terraform state at
gs://$TFSTATE_BUCKET (prefix acceptance/gcp/cluster) is empty or unreadable.
Destroying now would report success while destroying nothing. First check the
bucket's object versions (the bucket is versioned; a deleted state object can
be restored). If the state is truly gone, rebuild it by importing the six
resources into the backend, then re-run this script:

  export TF_DATA_DIR=$CLUSTER_TF_DATA_DIR
  terraform -chdir=$ACCEPTANCE_ROOT/cluster init -input=false -backend-config=bucket=$TFSTATE_BUCKET
  vars=(-var=project_id=$PROJECT_ID -var=region=$REGION -var=gke_zone=$GKE_ZONE \\
        -var=cluster_name=$PERSISTENT_CLUSTER_NAME \\
        -var=node_service_account_id=$NODE_SERVICE_ACCOUNT_ID)
  tfi() { terraform -chdir=$ACCEPTANCE_ROOT/cluster import "\${vars[@]}" "\$1" "\$2"; }
  tfi google_service_account.nodes            projects/$PROJECT_ID/serviceAccounts/$sa
  tfi google_project_iam_member.node_baseline  "$PROJECT_ID roles/container.defaultNodeServiceAccount serviceAccount:$sa"
  tfi google_project_iam_member.node_image_pull "$PROJECT_ID roles/artifactregistry.reader serviceAccount:$sa"
  tfi google_container_cluster.acceptance      projects/$PROJECT_ID/locations/$GKE_ZONE/clusters/$PERSISTENT_CLUSTER_NAME
  tfi google_container_node_pool.acceptance    projects/$PROJECT_ID/locations/$GKE_ZONE/clusters/$PERSISTENT_CLUSTER_NAME/nodePools/acceptance-pool
  tfi google_container_node_pool.data_plane    projects/$PROJECT_ID/locations/$GKE_ZONE/clusters/$PERSISTENT_CLUSTER_NAME/nodePools/data-plane-pool

Import reads deletion_protection from the provider default (true), not from
main.tf (false), so the cluster leg will refuse to destroy until you set it
false in the state. It is client-side-only metadata — there is no API field
and no gcloud flag for it, and \`terraform apply\` cannot reconcile it (it
emits an empty update: "Error 400: Must specify a field to update"). With the
GCS backend, edit it via:

  terraform -chdir=$ACCEPTANCE_ROOT/cluster state pull > /tmp/cluster.tfstate
  # set "deletion_protection": false on the cluster resource, then:
  terraform -chdir=$ACCEPTANCE_ROOT/cluster state push /tmp/cluster.tfstate
EOF
  exit 3
fi

TF_DATA_DIR="$CLUSTER_TF_DATA_DIR" terraform \
  -chdir="$ACCEPTANCE_ROOT/cluster" destroy -auto-approve \
  -var="project_id=$PROJECT_ID" -var="region=$REGION" -var="gke_zone=$GKE_ZONE" \
  -var="cluster_name=$PERSISTENT_CLUSTER_NAME" \
  -var="node_service_account_id=$NODE_SERVICE_ACCOUNT_ID"
