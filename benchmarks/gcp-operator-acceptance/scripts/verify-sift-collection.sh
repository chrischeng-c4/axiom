#!/usr/bin/env bash
set -euo pipefail

: "${PROJECT_ID:?PROJECT_ID is required}"
: "${REGION:?REGION is required}"
: "${GKE_ZONE:?GKE_ZONE is required}"
: "${RUN_ID:?RUN_ID is required}"
: "${BACKUP_BUCKET:?BACKUP_BUCKET is required}"
: "${MANIFEST_DIR:?MANIFEST_DIR is required}"
: "${EVIDENCE_DIR:?EVIDENCE_DIR is required}"
: "${LUMEN_ACCEPTANCE_EVIDENCE:=$EVIDENCE_DIR/lumen-acceptance.json}"
: "${LUMEN_ACCEPTANCE_PROVENANCE:=current-run}"
test -f "$LUMEN_ACCEPTANCE_EVIDENCE"

mkdir -p "$EVIDENCE_DIR/kubernetes" "$EVIDENCE_DIR/gcs"
forward_pids=()

stop_forwards() {
  local pid
  for pid in "${forward_pids[@]:-}"; do
    kill "$pid" >/dev/null 2>&1 || true
    wait "$pid" >/dev/null 2>&1 || true
  done
}
trap stop_forwards EXIT INT TERM

start_forward() {
  local namespace="$1"
  local service="$2"
  local local_port="$3"
  local remote_port="$4"
  local log="$5"
  local forward_pid=""
  local deadline=$((SECONDS + 90))
  while (( SECONDS < deadline )); do
    # A service forward binds to one endpoint pod.  The operator's bounded
    # drift/takeover tests can replace that pod after the forward starts, so
    # renew a dead forward instead of converting expected reconciliation churn
    # into a false readiness failure.
    if [[ -z "$forward_pid" ]] || ! kill -0 "$forward_pid" >/dev/null 2>&1; then
      if [[ -n "$forward_pid" ]]; then
        wait "$forward_pid" >/dev/null 2>&1 || true
      fi
      kubectl -n "$namespace" port-forward "service/$service" \
        "${local_port}:${remote_port}" >>"$log" 2>&1 &
      forward_pid="$!"
      forward_pids+=("$forward_pid")
    fi
    if curl --silent --show-error --fail "http://127.0.0.1:${local_port}/readyz" >/dev/null 2>&1; then
      return 0
    fi
    sleep 2
  done
  echo "timed out waiting for $namespace/$service readiness through port-forward" >&2
  return 1
}

capture_collector_diagnostics() {
  # Preserve the producer, collector, and durable collector state before the
  # outer run trap removes the namespaces.  A query miss can be caused by the
  # producer's JSONL contract, CRI discovery, decoder quarantine, or delivery;
  # these artifacts distinguish those cases without leaving cloud resources.
  kubectl -n lumen logs statefulset/lumen --all-containers --tail=500 \
    > "$EVIDENCE_DIR/kubernetes/lumen-serving.log" 2>&1 || true
  kubectl -n sift logs daemonset/sift-collector --all-containers --tail=500 \
    > "$EVIDENCE_DIR/kubernetes/sift-collector.log" 2>&1 || true
  kubectl -n sift exec daemonset/sift-collector -c collector -- \
    cat /var/lib/sift-collector/checkpoint.json \
    > "$EVIDENCE_DIR/kubernetes/sift-collector-checkpoint.json" 2>&1 || true
  kubectl -n sift exec daemonset/sift-collector -c collector -- \
    cat /var/lib/sift-collector/rejected.jsonl \
    > "$EVIDENCE_DIR/kubernetes/sift-collector-rejected.jsonl" 2>&1 || true
}

wait_for_collected_lumen_log() {
  local collection="$1"
  local result="$EVIDENCE_DIR/kubernetes/sift-collected-lumen-log.json"
  local deadline=$((SECONDS + 240))
  while (( SECONDS < deadline )); do
    curl --silent --show-error --fail-with-body -X POST \
      http://127.0.0.1:17380/v1/logs:query \
      -H 'content-type: application/json' \
      --data '{"project":"operator-acceptance","environment":"gke","service_name":"lumen","limit":100}' \
      > "$result"
    if jq -e --arg collection "$collection" '
      .records[] | select(
        .json_payload.attributes.collection_id == $collection
        and .resource["k8s.namespace.name"] == "lumen"
        and .resource["k8s.container.name"] == "lumen"
        and .resource["gcp.resource.type"] == "k8s_container"
      )
    ' "$result" >/dev/null; then
      return 0
    fi
    sleep 3
  done
  echo "Sift did not materialize the collector-tagged Lumen log for $collection" >&2
  capture_collector_diagnostics
  cat "$EVIDENCE_DIR/kubernetes/sift-collector.log" >&2 || true
  return 1
}

wait_for_gcs_object() {
  local prefix="gs://${BACKUP_BUCKET}/sift"
  local listing="$EVIDENCE_DIR/gcs/sift-objects.txt"
  local deadline=$((SECONDS + 180))
  local first
  while (( SECONDS < deadline )); do
    gcloud storage ls --recursive "${prefix}/**" > "$listing" 2>/dev/null || true
    first="$(rg -F "/sift/${RUN_ID}-" "$listing" | sed -n '1p' || true)"
    if [[ -n "$first" ]]; then
      printf '%s\n' "$first"
      return 0
    fi
    sleep 3
  done
  echo "no Sift backup object for run $RUN_ID appeared below $prefix" >&2
  return 1
}

# Sift starts only after the independent Lumen phase. Its collector is rendered
# by the Sift CLI, but Standard GKE is deliberately used so this DaemonSet can
# read the node CRI log directory.
kubectl apply -f "$MANIFEST_DIR/sift/collector.bundle.yaml"
kubectl -n sift rollout status daemonset/sift-collector --timeout=600s
kubectl get daemonset/sift-collector -n sift -o json > "$EVIDENCE_DIR/kubernetes/sift-collector-daemonset.json"

start_forward lumen lumen 17373 7373 "$EVIDENCE_DIR/kubernetes/lumen-sift-port-forward.log"
start_forward sift sift 17380 7380 "$EVIDENCE_DIR/kubernetes/sift-port-forward.log"

collection="collector-${RUN_ID}"
curl --silent --show-error --fail-with-body -X PUT \
  "http://127.0.0.1:17373/collections/${collection}" \
  -H 'content-type: application/json' \
  --data '{"fields":{"message":{"type":"keyword"}}}' \
  > "$EVIDENCE_DIR/kubernetes/lumen-collector-trigger.json"
wait_for_collected_lumen_log "$collection"

test "$(kubectl -n sift get cronjob/sift-backup -o jsonpath='{.spec.jobTemplate.spec.template.spec.serviceAccountName}')" = "sift-backup"
test "$(kubectl -n sift get sift/sift -o jsonpath='{.status.backupPhase}')" = "Configured"
sift_job="sift-backup-${RUN_ID}"
kubectl -n sift create job --from=cronjob/sift-backup "$sift_job"
kubectl -n sift wait --for=condition=Complete "job/$sift_job" --timeout=600s
kubectl -n sift logs "job/$sift_job" > "$EVIDENCE_DIR/kubernetes/sift-backup.log"
first_object="$(wait_for_gcs_object)"
object_size="$(gcloud storage objects describe "$first_object" --format='value(size)')"
[[ "$object_size" =~ ^[0-9]+$ && "$object_size" -gt 0 ]]
printf '%s\n' "$object_size" > "$EVIDENCE_DIR/gcs/sift-first-object-bytes.txt"
gcloud storage cat "$first_object" > "$EVIDENCE_DIR/gcs/sift-first-object.json"
jq -e 'type == "object"' "$EVIDENCE_DIR/gcs/sift-first-object.json" >/dev/null

capture_collector_diagnostics
kubectl -n sift get sift/sift -o json > "$EVIDENCE_DIR/kubernetes/sift-final.json"
kubectl get deployment,daemonset,statefulset,cronjob,job,pod,pvc,serviceaccount -A -o json \
  > "$EVIDENCE_DIR/kubernetes/workloads-final.json"

jq -n \
  --arg schema "axiom.gcp.sift.acceptance.v1" \
  --arg object "$first_object" \
  --argjson bytes "$object_size" \
  '{schema:$schema, operator_reconcile_1x1:"passed", standard_gke_cri_collector:"passed", lumen_structured_stdout_materialized:"passed", scheduled_backup:"passed", gcs_backup:"passed", gcs_object:$object, gcs_object_bytes:$bytes, topology_beyond_1x1:"not_claimed"}' \
  > "$EVIDENCE_DIR/sift-acceptance.json"

jq -n \
  --arg schema "axiom.gcp.operator.acceptance.v1" \
  --arg project_id "$PROJECT_ID" \
  --arg region "$REGION" \
  --arg gke_zone "$GKE_ZONE" \
  --arg run_id "$RUN_ID" \
  --arg bucket "$BACKUP_BUCKET" \
  --arg lumen_evidence "$LUMEN_ACCEPTANCE_EVIDENCE" \
  --arg lumen_provenance "$LUMEN_ACCEPTANCE_PROVENANCE" \
  --slurpfile lumen "$LUMEN_ACCEPTANCE_EVIDENCE" \
  --slurpfile sift "$EVIDENCE_DIR/sift-acceptance.json" \
  '{schema:$schema, project_id:$project_id, region:$region, gke_zone:$gke_zone, run_id:$run_id, backup_bucket:$bucket, lumen_evidence:$lumen_evidence, lumen_provenance:$lumen_provenance, acceptance:{lumen:$lumen[0],sift:$sift[0]}}' \
  > "$EVIDENCE_DIR/acceptance.json"
