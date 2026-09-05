#!/usr/bin/env bash
set -euo pipefail

# Own each mutation and its fresh identity check in one path. Kubernetes labels
# alone are not ownership. Bind the Node to the exact Compute instance ID and
# to live managed-group membership obtained from this run's GKE node pool.
[[ "$#" == "2" ]] || { echo "usage: sift-failover-vm.sh cordon|stop|uncordon <node>" >&2; exit 2; }
action="$1"
node="$2"
case "$action" in cordon|stop|uncordon) ;; *) exit 2 ;; esac
: "${PROJECT_ID:?PROJECT_ID is required}"
: "${GKE_ZONE:?GKE_ZONE is required}"
: "${GKE_CLUSTER_NAME:?GKE_CLUSTER_NAME is required}"
: "${RUN_ID:?RUN_ID is required}"
: "${SIFT_NODE_POOL:?SIFT_NODE_POOL is required}"
: "${EVIDENCE_DIR:?EVIDENCE_DIR is required}"
[[ "$SIFT_NODE_POOL" == "axo-${RUN_ID}-sift" \
  && "$node" =~ ^[a-z]([-a-z0-9]{0,61}[a-z0-9])?$ ]] || {
  echo "failover requires the exact run pool and a valid instance name" >&2
  exit 1
}
stem="$EVIDENCE_DIR/kubernetes/failover-${action}"
mkdir -p "$EVIDENCE_DIR/kubernetes"
compute_zone="https://www.googleapis.com/compute/v1/projects/${PROJECT_ID}/zones/${GKE_ZONE}"
instance_uri="${compute_zone}/instances/${node}"
provider_id="gce://${PROJECT_ID}/${GKE_ZONE}/${node}"
group_prefix="${compute_zone}/instanceGroupManagers/"

kubectl get "node/${node}" -o json > "${stem}-node.json"
jq -e --arg name "$node" --arg pool "$SIFT_NODE_POOL" \
  --arg run "$RUN_ID" --arg provider "$provider_id" '
  .metadata.name == $name
  and (.metadata.uid | type == "string" and length > 0)
  and .metadata.labels["axiom-run-id"] == $run
  and .metadata.labels["cloud.google.com/gke-nodepool"] == $pool
  and .spec.providerID == $provider
' "${stem}-node.json" >/dev/null || {
  echo "failover node identity does not match this run" >&2; exit 1;
}

gcloud container node-pools describe "$SIFT_NODE_POOL" \
  --cluster="$GKE_CLUSTER_NAME" --project="$PROJECT_ID" --zone="$GKE_ZONE" \
  --format=json > "${stem}-pool.json"
jq -e --arg pool "$SIFT_NODE_POOL" --arg run "$RUN_ID" --arg prefix "$group_prefix" '
  .name == $pool and .initialNodeCount == 3 and .status == "RUNNING"
  and .config.machineType == "e2-standard-4"
  and .config.labels["axiom-run-id"] == $run
  and .management.autoRepair == true
  and (.instanceGroupUrls | type == "array" and length > 0)
  and all(.instanceGroupUrls[];
    type == "string" and startswith($prefix)
    and (ltrimstr($prefix) | test("^[a-z]([-a-z0-9]{0,61}[a-z0-9])?$")))
  and (.instanceGroupUrls | length == (unique | length))
' "${stem}-pool.json" >/dev/null || {
  echo "failover pool or managed-group scope does not match this run" >&2; exit 1;
}
gcloud compute instances describe "$node" --project="$PROJECT_ID" \
  --zone="$GKE_ZONE" --format=json > "${stem}-vm.json"
jq -e --arg name "$node" --arg uri "$instance_uri" --arg zone "$compute_zone" '
  .name == $name and .selfLink == $uri and .zone == $zone
  and .status == "RUNNING"
  and (.id | type == "string" and test("^[1-9][0-9]*$"))
' "${stem}-vm.json" >/dev/null || {
  echo "failover Compute instance identity is invalid" >&2; exit 1;
}
instance_id="$(jq -er '.id' "${stem}-vm.json")"
matched_group=""
match_count=0
group_index=0
while IFS= read -r group_uri; do
  group_name="${group_uri#"$group_prefix"}"
  group_file="${stem}-group-${group_index}.json"
  gcloud compute instance-groups managed list-instances "$group_name" \
    --project="$PROJECT_ID" --zone="$GKE_ZONE" --format=json > "$group_file"
  jq -e 'type == "array"' "$group_file" >/dev/null
  matches="$(jq --arg uri "$instance_uri" --arg id "$instance_id" '
    [.[] | select(.instance == $uri and .id == $id
      and .instanceStatus == "RUNNING" and .currentAction == "NONE")] | length
  ' "$group_file")"
  if [[ "$matches" != "0" ]]; then
    matched_group="$group_uri"
    match_count=$((match_count + matches))
  fi
  group_index=$((group_index + 1))
done < <(jq -r '.instanceGroupUrls[]' "${stem}-pool.json")
[[ "$match_count" == "1" ]] || {
  echo "failover VM is not one live member of this run's managed groups" >&2; exit 1;
}

node_uid="$(jq -er '.metadata.uid' "${stem}-node.json")"
jq -n --arg project "$PROJECT_ID" --arg zone "$GKE_ZONE" \
  --arg cluster "$GKE_CLUSTER_NAME" --arg run "$RUN_ID" --arg pool "$SIFT_NODE_POOL" \
  --arg node "$node" --arg node_uid "$node_uid" --arg instance "$instance_uri" \
  --arg instance_id "$instance_id" --arg group "$matched_group" '
  {project:$project,zone:$zone,cluster:$cluster,run:$run,pool:$pool,node:$node,
   node_uid:$node_uid,instance:$instance,instance_id:$instance_id,group:$group}
' > "${stem}-identity.json"

if [[ "$action" == "stop" ]]; then
  # The same name can identify a replacement. Only the exact generation that
  # this run cordoned can be stopped. Re-read all ownership immediately above.
  prior="$EVIDENCE_DIR/kubernetes/failover-cordon-applied.json"
  [[ -f "$prior" && ! -L "$prior" ]] || {
    echo "failover has no applied cordon receipt" >&2; exit 1;
  }
  jq -e --slurpfile prior "$prior" '. == $prior[0]' \
    "${stem}-identity.json" >/dev/null || {
    echo "failover target changed after cordon" >&2; exit 1;
  }
  cp "${stem}-vm.json" "$EVIDENCE_DIR/kubernetes/failover-vm-before.json"
  gcloud compute instances stop "$node" \
    --project="$PROJECT_ID" --zone="$GKE_ZONE" --quiet
else
  # JSON Patch tests are checked atomically by the API server. Never cordon or
  # uncordon a same-name replacement between the ownership read and the write.
  unschedulable=true
  [[ "$action" != "uncordon" ]] || unschedulable=false
  patch="$(jq -n --arg uid "$node_uid" --arg provider "$provider_id" \
    --arg pool "$SIFT_NODE_POOL" --arg run "$RUN_ID" --argjson value "$unschedulable" '
    [{op:"test",path:"/metadata/uid",value:$uid},
     {op:"test",path:"/spec/providerID",value:$provider},
     {op:"test",path:"/metadata/labels/cloud.google.com~1gke-nodepool",value:$pool},
     {op:"test",path:"/metadata/labels/axiom-run-id",value:$run},
     {op:"add",path:"/spec/unschedulable",value:$value}]
  ')"
  kubectl patch "node/${node}" --type=json --patch "$patch" -o=json \
    > "${stem}-response.json"
fi
cp "${stem}-identity.json" "${stem}-applied.json"
