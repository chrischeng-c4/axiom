#!/usr/bin/env bash
set -euo pipefail

: "${PROJECT_ID:?PROJECT_ID is required}"
: "${REGION:?REGION is required}"
: "${RUN_ID:?RUN_ID is required}"
: "${BACKUP_BUCKET:?BACKUP_BUCKET is required}"
: "${EVIDENCE_DIR:?EVIDENCE_DIR is required}"

mkdir -p "$EVIDENCE_DIR/kubernetes" "$EVIDENCE_DIR/gcs"
forward_pid=""
restore_forward_pid=""

stop_forward() {
  if [[ -n "$forward_pid" ]]; then
    kill "$forward_pid" >/dev/null 2>&1 || true
    wait "$forward_pid" >/dev/null 2>&1 || true
    forward_pid=""
  fi
}

stop_restore_forward() {
  if [[ -n "$restore_forward_pid" ]]; then
    kill "$restore_forward_pid" >/dev/null 2>&1 || true
    wait "$restore_forward_pid" >/dev/null 2>&1 || true
    restore_forward_pid=""
  fi
}

cleanup_forwards() {
  stop_forward
  stop_restore_forward
}
trap cleanup_forwards EXIT INT TERM

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
  local prefix="gs://${BACKUP_BUCKET}/lumen"
  local listing="$EVIDENCE_DIR/gcs/lumen-objects.txt"
  local deadline=$((SECONDS + 180))
  local first
  while (( SECONDS < deadline )); do
    gcloud storage ls --recursive "${prefix}/**" > "$listing" 2>/dev/null || true
    first="$(rg -F "/lumen/${RUN_ID}-" "$listing" | sed -n '1p' || true)"
    if [[ -n "$first" ]]; then
      printf '%s\n' "$first"
      return 0
    fi
    sleep 3
  done
  echo "no Lumen backup object for run $RUN_ID appeared below $prefix" >&2
  return 1
}

wait_for_split() {
  local deadline=$((SECONDS + 900))
  local shard_count workflow_phase status_phase desired ready pvc_count map_version converged_map_version
  while (( SECONDS < deadline )); do
    shard_count="$(kubectl -n lumen get lumen/lumen -o jsonpath='{.spec.shardCount}' 2>/dev/null || true)"
    workflow_phase="$(kubectl -n lumen get lumen/lumen -o jsonpath='{.spec.reshardPolicy.workflow.phase}' 2>/dev/null || true)"
    map_version="$(kubectl -n lumen get lumen/lumen -o jsonpath='{.spec.shardMap.version}' 2>/dev/null || true)"
    converged_map_version="$(kubectl -n lumen get lumen/lumen -o jsonpath='{.spec.reshardPolicy.workflow.convergedShardMapVersion}' 2>/dev/null || true)"
    status_phase="$(kubectl -n lumen get lumen/lumen -o jsonpath='{.status.phase}' 2>/dev/null || true)"
    desired="$(kubectl -n lumen get statefulset/lumen -o jsonpath='{.spec.replicas}' 2>/dev/null || true)"
    ready="$(kubectl -n lumen get statefulset/lumen -o jsonpath='{.status.readyReplicas}' 2>/dev/null || true)"
    pvc_count="$(kubectl -n lumen get pvc -l app.kubernetes.io/instance=lumen --no-headers 2>/dev/null | wc -l | tr -d ' ')"
    # `Complete` means the bucket map has been cut over, not that every
    # serving pod has picked up that map. Until the driver writes
    # `convergedShardMapVersion`, it deliberately holds a post-cutover fence;
    # a Service-routed read can reach a newly created shard before the copied
    # bucket data is queryable and return 404. Do not treat that expected
    # transition window as a stable post-split read surface.
    if [[ "$shard_count" == "2" && "$workflow_phase" == "Complete" && "$map_version" =~ ^[1-9][0-9]*$ && "$converged_map_version" == "$map_version" && "$status_phase" == "Ready" && "$desired" == "2" && "$ready" == "2" && "$pvc_count" -ge 2 ]]; then
      return 0
    fi
    sleep 5
  done
  echo "Lumen did not converge 1-to-2 split with two ready pods and two PVCs" >&2
  # Two calls: kubectl rejects mixing resource/name form with a bare resource
  # type in one argument list (the tape campaign hit this exact silent dump).
  kubectl -n lumen get lumen/lumen statefulset/lumen -o yaml >&2 || true
  kubectl -n lumen get pvc -o yaml >&2 || true
  return 1
}

# ---- lumen-restore: second instance for the cold-restore leg (#2492) ----
start_restore_forward() {
  stop_restore_forward
  local deadline=$((SECONDS + 90))
  while (( SECONDS < deadline )); do
    if [[ -z "$restore_forward_pid" ]] || ! kill -0 "$restore_forward_pid" >/dev/null 2>&1; then
      stop_restore_forward
      kubectl -n lumen port-forward service/lumen-restore 17374:7373 \
        >>"$EVIDENCE_DIR/kubernetes/lumen-restore-port-forward.log" 2>&1 &
      restore_forward_pid="$!"
      sleep 1
    fi
    if curl --max-time 5 --silent --show-error --fail \
      http://127.0.0.1:17374/readyz >/dev/null 2>&1; then
      return 0
    fi
    sleep 2
  done
  echo "timed out waiting for lumen-restore service readiness through port-forward" >&2
  return 1
}

restore_search_probe() {
  curl --silent --show-error --fail-with-body -X POST \
    http://127.0.0.1:17374/collections/acceptance/search \
    -H 'content-type: application/json' \
    --data "{\"query\":{\"term\":{\"field\":\"message\",\"value\":\"gke-${RUN_ID}\"}},\"limit\":10}"
}

# Mirrors deploy.sh's wait_ready_cr, scoped to the lumen-restore CR.
wait_ready_restore_cr() {
  local expected_generation observed_generation phase
  local deadline=$((SECONDS + 600))
  while (( SECONDS < deadline )); do
    expected_generation="$(kubectl -n lumen get lumen/lumen-restore -o jsonpath='{.metadata.generation}' 2>/dev/null || true)"
    observed_generation="$(kubectl -n lumen get lumen/lumen-restore -o jsonpath='{.status.observedGeneration}' 2>/dev/null || true)"
    phase="$(kubectl -n lumen get lumen/lumen-restore -o jsonpath='{.status.phase}' 2>/dev/null || true)"
    if [[ -n "$expected_generation" && "$observed_generation" == "$expected_generation" && "$phase" == "Ready" ]]; then
      return 0
    fi
    sleep 5
  done
  echo "timed out waiting for lumen/lumen-restore status generation and Ready phase" >&2
  capture_restore_diagnostics
  return 1
}

# Preserve the restore CR/StatefulSet/pod state before it gets torn down, so a
# failure stays provable instead of silently disappearing with the instance.
capture_restore_diagnostics() {
  kubectl -n lumen get lumen/lumen-restore statefulset/lumen-restore -o yaml \
    > "$EVIDENCE_DIR/kubernetes/lumen-restore-failure.yaml" 2>&1 || true
  kubectl -n lumen describe pods -l app.kubernetes.io/instance=lumen-restore \
    > "$EVIDENCE_DIR/kubernetes/lumen-restore-pods-describe.txt" 2>&1 || true
  kubectl -n lumen logs pod/lumen-restore-0 --all-containers --tail=200 --prefix \
    >> "$EVIDENCE_DIR/kubernetes/lumen-restore-pods.log" 2>&1 || true
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

# ---- Cold-restore leg: fresh PVC seeded from the GCS backup object (#2492) ----
# Ordered before the reshard patch below so the known #2489 post-split-read
# defect can never mask a seed regression. `spec.serving.bootstrap.seedUri`
# (apps/lumen/src/operator/crd.rs `ServingBootstrapSpec`) is a real
# CR-expressible seed path: `serving_env` (apps/lumen/src/operator/render.rs)
# turns a set `bootstrap.seedUri` into `LUMEN_BOOTSTRAP_SEED_URI` on the
# rendered pod, and `apply_bootstrap_seed` (apps/lumen/src/bin/lumen.rs)
# restores it before WAL/raft catch-up on every boot — no fallback/skip is
# needed here. Apply a second, complete Lumen CR (`lumen-restore`) next to
# the still-running main instance and prove a genuinely fresh PVC actually
# restores from `$first_object`.
stop_forward
main_cr_image="$(kubectl -n lumen get lumen/lumen -o jsonpath='{.spec.image}')"
main_cr_raft_storage="$(kubectl -n lumen get lumen/lumen -o jsonpath='{.spec.serving.raftStorage}')"
[[ -n "$main_cr_image" ]] || {
  echo "could not read the main lumen CR's image for the restore instance" >&2
  exit 1
}
[[ -n "$main_cr_raft_storage" ]] || {
  echo "could not read the main lumen CR's serving.raftStorage for the restore instance" >&2
  exit 1
}
cat <<EOF | kubectl apply -f - > "$EVIDENCE_DIR/kubernetes/lumen-restore-cr-apply.txt"
apiVersion: lumen.dev/v1alpha1
kind: Lumen
metadata:
  name: lumen-restore
  namespace: lumen
spec:
  image: ${main_cr_image}
  imagePullPolicy: IfNotPresent
  shardCount: 1
  replicasPerShard: 1
  voterCount: 1
  logFormat: json
  serving:
    cpu: 500m
    memory: 1Gi
    raftStorage: ${main_cr_raft_storage}
    bootstrap:
      seedUri: ${first_object}
EOF
wait_ready_restore_cr
kubectl -n lumen get lumen/lumen-restore -o json \
  > "$EVIDENCE_DIR/kubernetes/lumen-restore-after-apply.json"

start_restore_forward
restore_seed_deadline=$((SECONDS + 120))
until restore_search_probe > "$EVIDENCE_DIR/kubernetes/lumen-restore-search.json" 2>/dev/null \
  && jq -e '.total >= 1' "$EVIDENCE_DIR/kubernetes/lumen-restore-search.json" >/dev/null 2>&1; do
  if (( SECONDS >= restore_seed_deadline )); then
    echo "lumen-restore never surfaced the seeded document from $first_object" >&2
    cat "$EVIDENCE_DIR/kubernetes/lumen-restore-search.json" >&2 || true
    capture_restore_diagnostics
    exit 1
  fi
  kill -0 "$restore_forward_pid" >/dev/null 2>&1 || start_restore_forward
  sleep 3
done

# Restart-while-seed-set: unlike tape's one-shot bootstrapSeedUri contract
# (#2468), lumen's field stays set on the CR, so this also proves a pod
# replacement re-applies the same seed idempotently instead of regressing
# state while the field remains set.
stop_restore_forward
kubectl -n lumen delete pod/lumen-restore-0 --wait=true --timeout=120s
kubectl -n lumen rollout status statefulset/lumen-restore --timeout=600s
kubectl -n lumen wait --for=condition=Ready pod/lumen-restore-0 --timeout=300s
start_restore_forward
restore_restart_deadline=$((SECONDS + 120))
until restore_search_probe > "$EVIDENCE_DIR/kubernetes/lumen-restore-after-restart-search.json" 2>/dev/null \
  && jq -e '.total >= 1' "$EVIDENCE_DIR/kubernetes/lumen-restore-after-restart-search.json" >/dev/null 2>&1; do
  if (( SECONDS >= restore_restart_deadline )); then
    echo "lumen-restore lost the seeded document across a pod restart with bootstrap.seedUri still set" >&2
    cat "$EVIDENCE_DIR/kubernetes/lumen-restore-after-restart-search.json" >&2 || true
    capture_restore_diagnostics
    exit 1
  fi
  kill -0 "$restore_forward_pid" >/dev/null 2>&1 || start_restore_forward
  sleep 3
done
stop_restore_forward

# Teardown ordering mirrors verify-tape.sh's Step D: the CR must be gone
# before the StatefulSet/PVCs, or drift repair recreates the StatefulSet
# (owner-ref cascade GC — apps/lumen/src/operator/render.rs sets
# `ownerReferences` via `libs/service-k8s::render::RenderCtx::meta` — is
# async and not guaranteed complete by the time `--wait` on the CR delete
# returns) and its pods keep the old PVCs claimed, deadlocking PVC deletion
# against pod deletion.
kubectl -n lumen delete lumen/lumen-restore --wait=true --timeout=120s
kubectl -n lumen delete statefulset/lumen-restore --wait=true --timeout=180s \
  --cascade=foreground --ignore-not-found
kubectl -n lumen delete pvc -l app.kubernetes.io/instance=lumen-restore \
  --wait=true --timeout=300s --ignore-not-found

# One byte is an acceptance-only pressure threshold. It exercises the existing
# disk policy without creating a chargeable GiB of test data.
kubectl -n lumen patch lumen/lumen --type=merge --patch \
  '{"spec":{"reshardPolicy":{"maxShardBytes":1,"prepareAtPercent":1,"startAtPercent":1,"urgentAtPercent":100,"maxShards":2,"migrationBytesPerSec":10485760}}}'
wait_for_split

stop_forward
start_forward
# Run 0723160506 (lumen@0.4.24): a single probe immediately after the fence
# converged still hit "collection not found" on the Service-pinned new-shard
# pod. Poll with a bound and RECORD how long readability lagged convergence —
# a nonzero lag is fence-gap evidence for the product (tracked as an
# app:lumen finding), while a never-readable split stays a hard failure.
split_read_started=$SECONDS
split_read_deadline=$((SECONDS + 180))
until search_probe > "$EVIDENCE_DIR/kubernetes/lumen-search-after-split.json" 2>/dev/null \
  && jq -e '.total >= 1' "$EVIDENCE_DIR/kubernetes/lumen-search-after-split.json" >/dev/null 2>&1; do
  if (( SECONDS >= split_read_deadline )); then
    echo "post-split search never became readable through the converged fence" >&2
    cat "$EVIDENCE_DIR/kubernetes/lumen-search-after-split.json" >&2 || true
    kubectl -n lumen get lumen/lumen -o yaml >&2 || true
    kubectl -n lumen describe pods >&2 || true
    exit 1
  fi
  kill -0 "$forward_pid" >/dev/null 2>&1 || start_forward
  sleep 3
done
printf '%s\n' "$((SECONDS - split_read_started))" \
  > "$EVIDENCE_DIR/kubernetes/lumen-split-readable-after-seconds.txt"
kubectl -n lumen get lumen/lumen -o json > "$EVIDENCE_DIR/kubernetes/lumen-after-split.json"
kubectl get deployment,statefulset,cronjob,job,pod,pvc,serviceaccount -A -o json \
  > "$EVIDENCE_DIR/kubernetes/workloads-after-lumen-phase.json"

jq -n \
  --arg schema "axiom.gcp.lumen.acceptance.v1" \
  --arg object "$first_object" \
  --argjson bytes "$object_size" \
  '{schema:$schema, operator_reconcile_1x1:"passed", pod_restart_data_retention:"passed", gcs_backup_before_split:"passed", gcs_object:$object, gcs_object_bytes:$bytes, cold_restore_fresh_pvc:"passed", seed_set_restart_retention:"passed", auto_split_delta:1, auto_split:{from:1,to:2,ready_pods:2,pvcs_at_least:2}, cpu_memory_actuator:"not_claimed", live_replica_membership:"not_claimed"}' \
  > "$EVIDENCE_DIR/lumen-acceptance.json"
