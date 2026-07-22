#!/usr/bin/env bash
set -euo pipefail

: "${PROJECT_ID:?PROJECT_ID is required}"
: "${REGION:?REGION is required}"
: "${RUN_ID:?RUN_ID is required}"
: "${BACKUP_BUCKET:?BACKUP_BUCKET is required}"
: "${EVIDENCE_DIR:?EVIDENCE_DIR is required}"

mkdir -p "$EVIDENCE_DIR/kubernetes" "$EVIDENCE_DIR/gcs"
forward_pid=""

stop_forward() {
  if [[ -n "$forward_pid" ]]; then
    kill "$forward_pid" >/dev/null 2>&1 || true
    wait "$forward_pid" >/dev/null 2>&1 || true
    forward_pid=""
  fi
}
trap stop_forward EXIT INT TERM

start_forward() {
  stop_forward
  local deadline=$((SECONDS + 90))
  while (( SECONDS < deadline )); do
    # The operator-cell test intentionally replaces a Lumen pod immediately
    # before this probe. `kubectl port-forward service/...` pins one selected
    # endpoint, so it can exit while the Service already has a healthy
    # replacement. Recreate the local forward rather than turning that normal
    # hand-off into a false readiness failure.
    if [[ -z "$forward_pid" ]] || ! kill -0 "$forward_pid" >/dev/null 2>&1; then
      stop_forward
      kubectl -n lumen port-forward service/lumen 17373:7373 \
        >>"$EVIDENCE_DIR/kubernetes/lumen-port-forward.log" 2>&1 &
      forward_pid="$!"
      sleep 1
    fi
    if curl --max-time 5 --silent --show-error --fail \
      http://127.0.0.1:17373/readyz >/dev/null 2>&1; then
      return 0
    fi
    sleep 2
  done
  echo "timed out waiting for Lumen service readiness through port-forward" >&2
  return 1
}

search_probe() {
  curl --silent --show-error --fail-with-body -X POST \
    http://127.0.0.1:17373/collections/acceptance/search \
    -H 'content-type: application/json' \
    --data "{\"query\":{\"term\":{\"field\":\"message\",\"value\":\"gke-${RUN_ID}\"}},\"limit\":10}"
}

wait_for_gcs_object() {
  local prefix="gs://${BACKUP_BUCKET}/lumen/${RUN_ID}"
  local listing="$EVIDENCE_DIR/gcs/lumen-objects.txt"
  local deadline=$((SECONDS + 180))
  local first
  while (( SECONDS < deadline )); do
    gcloud storage ls --recursive "${prefix}/**" > "$listing" 2>/dev/null || true
    first="$(sed -n '1p' "$listing")"
    if [[ -n "$first" ]]; then
      printf '%s\n' "$first"
      return 0
    fi
    sleep 3
  done
  echo "no Lumen backup object appeared below $prefix" >&2
  return 1
}

wait_for_split() {
  local deadline=$((SECONDS + 900))
  local shard_count workflow_phase status_phase desired ready pvc_count
  while (( SECONDS < deadline )); do
    shard_count="$(kubectl -n lumen get lumen/lumen -o jsonpath='{.spec.shardCount}' 2>/dev/null || true)"
    workflow_phase="$(kubectl -n lumen get lumen/lumen -o jsonpath='{.spec.reshardPolicy.workflow.phase}' 2>/dev/null || true)"
    status_phase="$(kubectl -n lumen get lumen/lumen -o jsonpath='{.status.phase}' 2>/dev/null || true)"
    desired="$(kubectl -n lumen get statefulset/lumen -o jsonpath='{.spec.replicas}' 2>/dev/null || true)"
    ready="$(kubectl -n lumen get statefulset/lumen -o jsonpath='{.status.readyReplicas}' 2>/dev/null || true)"
    pvc_count="$(kubectl -n lumen get pvc -l app.kubernetes.io/instance=lumen --no-headers 2>/dev/null | wc -l | tr -d ' ')"
    if [[ "$shard_count" == "2" && "$workflow_phase" == "Complete" && "$status_phase" == "Ready" && "$desired" == "2" && "$ready" == "2" && "$pvc_count" -ge 2 ]]; then
      return 0
    fi
    sleep 5
  done
  echo "Lumen did not complete 1-to-2 split with two ready pods and two PVCs" >&2
  kubectl -n lumen get lumen/lumen,statefulset/lumen,pvc -o yaml >&2 || true
  return 1
}

start_forward
curl --silent --show-error --fail-with-body -X PUT \
  http://127.0.0.1:17373/collections/acceptance \
  -H 'content-type: application/json' \
  --data '{"fields":{"message":{"type":"keyword"}}}' \
  > "$EVIDENCE_DIR/kubernetes/lumen-create-collection.json"
curl --silent --show-error --fail-with-body -X POST \
  http://127.0.0.1:17373/collections/acceptance/index \
  -H 'content-type: application/json' \
  --data "{\"items\":[{\"external_id\":\"${RUN_ID}\",\"field\":\"message\",\"value\":\"gke-${RUN_ID}\"}]}" \
  > "$EVIDENCE_DIR/kubernetes/lumen-index.json"
search_probe > "$EVIDENCE_DIR/kubernetes/lumen-search-before-restart.json"
jq -e '.total >= 1' "$EVIDENCE_DIR/kubernetes/lumen-search-before-restart.json" >/dev/null

test "$(kubectl -n lumen get cronjob/lumen-backup -o jsonpath='{.spec.jobTemplate.spec.template.spec.serviceAccountName}')" = "lumen-backup"
lumen_job="lumen-backup-${RUN_ID}"
kubectl -n lumen create job --from=cronjob/lumen-backup "$lumen_job"
kubectl -n lumen wait --for=condition=Complete "job/$lumen_job" --timeout=600s
kubectl -n lumen logs "job/$lumen_job" > "$EVIDENCE_DIR/kubernetes/lumen-backup.log"
first_object="$(wait_for_gcs_object)"
object_size="$(gcloud storage objects describe "$first_object" --format='value(size)')"
[[ "$object_size" =~ ^[0-9]+$ && "$object_size" -gt 0 ]]
printf '%s\n' "$object_size" > "$EVIDENCE_DIR/gcs/lumen-first-object-bytes.txt"
gcloud storage cat "$first_object" > "$EVIDENCE_DIR/gcs/lumen-first-object.json"
jq -e 'type == "object"' "$EVIDENCE_DIR/gcs/lumen-first-object.json" >/dev/null

stop_forward
kubectl -n lumen delete pod/lumen-0 --wait=true --timeout=120s
kubectl -n lumen rollout status statefulset/lumen --timeout=600s
kubectl -n lumen wait --for=condition=Ready pod/lumen-0 --timeout=300s
start_forward
search_probe > "$EVIDENCE_DIR/kubernetes/lumen-search-after-restart.json"
jq -e '.total >= 1' "$EVIDENCE_DIR/kubernetes/lumen-search-after-restart.json" >/dev/null

# One byte is an acceptance-only pressure threshold. It exercises the existing
# disk policy without creating a chargeable GiB of test data.
kubectl -n lumen patch lumen/lumen --type=merge --patch \
  '{"spec":{"reshardPolicy":{"maxShardBytes":1,"prepareAtPercent":1,"startAtPercent":1,"urgentAtPercent":100,"maxShards":2,"migrationBytesPerSec":10485760}}}'
wait_for_split

stop_forward
start_forward
search_probe > "$EVIDENCE_DIR/kubernetes/lumen-search-after-split.json"
jq -e '.total >= 1' "$EVIDENCE_DIR/kubernetes/lumen-search-after-split.json" >/dev/null
kubectl -n lumen get lumen/lumen -o json > "$EVIDENCE_DIR/kubernetes/lumen-after-split.json"
kubectl get deployment,statefulset,cronjob,job,pod,pvc,serviceaccount -A -o json \
  > "$EVIDENCE_DIR/kubernetes/workloads-after-lumen-phase.json"

jq -n \
  --arg schema "axiom.gcp.lumen.acceptance.v1" \
  --arg object "$first_object" \
  --argjson bytes "$object_size" \
  '{schema:$schema, operator_reconcile_1x1:"passed", pod_restart_data_retention:"passed", gcs_backup_before_split:"passed", gcs_object:$object, gcs_object_bytes:$bytes, auto_split_delta:1, auto_split:{from:1,to:2,ready_pods:2,pvcs_at_least:2}, cpu_memory_actuator:"not_claimed", live_replica_membership:"not_claimed"}' \
  > "$EVIDENCE_DIR/lumen-acceptance.json"
