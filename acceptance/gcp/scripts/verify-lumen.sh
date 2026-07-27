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

# The admission leg (#2477) patches the main CR, not the StatefulSet, so the
# OPERATOR — not kubectl — propagates `LUMEN_ADMISSION_*` onto the pod spec.
# A bare `rollout status` immediately after the patch races that propagation:
# until the operator re-renders the StatefulSet it still reports the pre-patch
# generation as fully rolled out, so the env read sees stale, admission-free
# env (GKE run v4 0724045952 died here with observedGeneration lagging
# metadata.generation). Waiting on `observedGeneration` is itself unreliable
# here because the operator writes `spec.shardMap`/`spec.reshardPolicy.workflow`
# back into spec, advancing `metadata.generation` out from under the poll, so
# poll the actual observable — the rendered StatefulSet pod-template env — until
# the admission grammar lands.
wait_for_admission_env() {
  local envfile="$EVIDENCE_DIR/kubernetes/lumen-admission-env.txt"
  local deadline=$((SECONDS + 300))
  while (( SECONDS < deadline )); do
    kubectl -n lumen get statefulset/lumen \
      -o jsonpath='{range .spec.template.spec.containers[0].env[*]}{.name}={.value}{"\n"}{end}' \
      > "$envfile" 2>/dev/null || true
    if grep -qx 'LUMEN_ADMISSION_READ_CAPACITY=100' "$envfile" \
      && grep -qx 'LUMEN_ADMISSION_WRITE_CAPACITY=50' "$envfile" \
      && grep -qx 'LUMEN_ADMISSION_ADMIN_CAPACITY=10' "$envfile" \
      && grep -qx 'LUMEN_ADMISSION_REFILL_SECS=30' "$envfile" \
      && grep -qx 'LUMEN_ADMISSION_MAX_KEYS=256' "$envfile"; then
      return 0
    fi
    sleep 3
  done
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

# ---- lumen-authcsi: Secret Manager + SecretProviderClass provider:gke auth+CSI
# regression leg (#2457) helpers ----
authcsi_forward_pid=""
stop_authcsi_forward() {
  if [[ -n "$authcsi_forward_pid" ]]; then
    kill "$authcsi_forward_pid" >/dev/null 2>&1 || true
    wait "$authcsi_forward_pid" >/dev/null 2>&1 || true
    authcsi_forward_pid=""
  fi
}

start_authcsi_forward() {
  stop_authcsi_forward
  local deadline=$((SECONDS + 90))
  while (( SECONDS < deadline )); do
    if [[ -z "$authcsi_forward_pid" ]] || ! kill -0 "$authcsi_forward_pid" >/dev/null 2>&1; then
      stop_authcsi_forward
      kubectl -n lumen port-forward service/lumen-authcsi 17375:7373 \
        >>"$EVIDENCE_DIR/kubernetes/lumen-authcsi-port-forward.log" 2>&1 &
      authcsi_forward_pid="$!"
      sleep 1
    fi
    if curl --max-time 5 --silent --show-error --fail \
      http://127.0.0.1:17375/readyz >/dev/null 2>&1; then
      return 0
    fi
    sleep 2
  done
  echo "timed out waiting for lumen-authcsi service readiness through port-forward" >&2
  return 1
}

authcsi_search_authenticated() {
  curl --silent --show-error --fail-with-body -X POST \
    http://127.0.0.1:17375/collections/acceptance-authcsi/search \
    -H 'content-type: application/json' \
    -H "authorization: Bearer ${LUMEN_AUTHCSI_TOKEN}" \
    --data "{\"query\":{\"term\":{\"field\":\"message\",\"value\":\"gke-authcsi-${RUN_ID}\"}},\"limit\":10}"
}

# Preserve the authcsi CR/StatefulSet/pod/SecretProviderClass state before it
# gets torn down, so a mount failure stays provable instead of silently
# disappearing with the instance.
capture_authcsi_diagnostics() {
  kubectl -n lumen get lumen/lumen-authcsi statefulset/lumen-authcsi -o yaml \
    > "$EVIDENCE_DIR/kubernetes/lumen-authcsi-failure.yaml" 2>&1 || true
  kubectl -n lumen get secretproviderclass/lumen-authcsi-tokens -o yaml \
    > "$EVIDENCE_DIR/kubernetes/lumen-authcsi-secretproviderclass.yaml" 2>&1 || true
  kubectl -n lumen describe pods -l app.kubernetes.io/instance=lumen-authcsi \
    > "$EVIDENCE_DIR/kubernetes/lumen-authcsi-pods-describe.txt" 2>&1 || true
  kubectl -n lumen get events --field-selector involvedObject.name=lumen-authcsi-0 \
    -o json > "$EVIDENCE_DIR/kubernetes/lumen-authcsi-pod-events-failure.json" 2>&1 || true
  kubectl -n lumen logs pod/lumen-authcsi-0 --all-containers --tail=200 --prefix \
    >> "$EVIDENCE_DIR/kubernetes/lumen-authcsi-pods.log" 2>&1 || true
}

# ---- lumen-quorum: multi-voter membership on a CR NOT named after the binary
# (#2610) helpers ----
#
# Every pre-existing leg misses this bug at the intersection of two conditions.
# The multi-member cases (`lumen/lumen` here, `tape/tape` in verify-tape.sh)
# name their CR after the binary, so the peer prefix the binary hardcoded
# happened to equal the one derived from POD_NAME. The differently-named cases
# (`lumen-restore`, `lumen-authcsi`) run replicasPerShard:1, where `--wal auto`
# selects the embedded backend and no raft peer is ever addressed. Only a CR
# that is BOTH multi-member AND named something other than `lumen` addresses a
# peer by a name the operator did not create.
quorum_forward_pid=""
stop_quorum_forward() {
  if [[ -n "$quorum_forward_pid" ]]; then
    kill "$quorum_forward_pid" >/dev/null 2>&1 || true
    wait "$quorum_forward_pid" >/dev/null 2>&1 || true
    quorum_forward_pid=""
  fi
}

# Forwards to a specific POD, not the Service: this leg has to address
# individual members to tell a leader from a follower, which a load-balanced
# Service endpoint deliberately hides.
start_quorum_forward() {
  local pod="$1"
  stop_quorum_forward
  local deadline=$((SECONDS + 90))
  while (( SECONDS < deadline )); do
    if [[ -z "$quorum_forward_pid" ]] || ! kill -0 "$quorum_forward_pid" >/dev/null 2>&1; then
      stop_quorum_forward
      kubectl -n lumen port-forward "pod/$pod" 17376:7373 \
        >>"$EVIDENCE_DIR/kubernetes/lumen-quorum-port-forward.log" 2>&1 &
      quorum_forward_pid="$!"
      sleep 1
    fi
    if curl --max-time 5 --silent --show-error --fail \
      http://127.0.0.1:17376/readyz >/dev/null 2>&1; then
      return 0
    fi
    sleep 2
  done
  echo "timed out waiting for $pod readiness through port-forward" >&2
  return 1
}

quorum_cluster_json() { # pod  outfile
  start_quorum_forward "$1" || return 1
  curl --max-time 10 --silent --show-error --fail-with-body \
    http://127.0.0.1:17376/debug/cluster > "$2"
}

capture_quorum_diagnostics() {
  kubectl -n lumen get lumen/lumen-quorum statefulset/lumen-quorum -o yaml \
    > "$EVIDENCE_DIR/kubernetes/lumen-quorum-failure.yaml" 2>&1 || true
  kubectl -n lumen describe pods -l app.kubernetes.io/instance=lumen-quorum \
    > "$EVIDENCE_DIR/kubernetes/lumen-quorum-pods-describe.txt" 2>&1 || true
  for pod in lumen-quorum-0 lumen-quorum-1; do
    kubectl -n lumen logs "pod/$pod" --all-containers --tail=200 --prefix \
      >> "$EVIDENCE_DIR/kubernetes/lumen-quorum-pods.log" 2>&1 || true
  done
}

wait_ready_quorum_cr() {
  local expected_generation observed_generation phase ready
  local deadline=$((SECONDS + 600))
  while (( SECONDS < deadline )); do
    expected_generation="$(kubectl -n lumen get lumen/lumen-quorum -o jsonpath='{.metadata.generation}' 2>/dev/null || true)"
    observed_generation="$(kubectl -n lumen get lumen/lumen-quorum -o jsonpath='{.status.observedGeneration}' 2>/dev/null || true)"
    phase="$(kubectl -n lumen get lumen/lumen-quorum -o jsonpath='{.status.phase}' 2>/dev/null || true)"
    ready="$(kubectl -n lumen get statefulset/lumen-quorum -o jsonpath='{.status.readyReplicas}' 2>/dev/null || true)"
    if [[ -n "$expected_generation" && "$observed_generation" == "$expected_generation" \
      && "$phase" == "Ready" && "$ready" == "2" ]]; then
      return 0
    fi
    sleep 5
  done
  echo "timed out waiting for lumen/lumen-quorum to reach Ready with 2 ready replicas" >&2
  capture_quorum_diagnostics
  return 1
}

# Mirrors deploy.sh's wait_ready_cr / wait_ready_restore_cr, scoped to the
# lumen-authcsi CR.
wait_ready_authcsi_cr() {
  local expected_generation observed_generation phase
  local deadline=$((SECONDS + 600))
  while (( SECONDS < deadline )); do
    expected_generation="$(kubectl -n lumen get lumen/lumen-authcsi -o jsonpath='{.metadata.generation}' 2>/dev/null || true)"
    observed_generation="$(kubectl -n lumen get lumen/lumen-authcsi -o jsonpath='{.status.observedGeneration}' 2>/dev/null || true)"
    phase="$(kubectl -n lumen get lumen/lumen-authcsi -o jsonpath='{.status.phase}' 2>/dev/null || true)"
    if [[ -n "$expected_generation" && "$observed_generation" == "$expected_generation" && "$phase" == "Ready" ]]; then
      return 0
    fi
    sleep 5
  done
  echo "timed out waiting for lumen/lumen-authcsi status generation and Ready phase — the #2456 fail-signature is FailedMount/0-ready" >&2
  capture_authcsi_diagnostics
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

# ---- Admission CRD exposure exercise on the main instance (#2477) ----
# Pure exposure of the pre-existing `LUMEN_ADMISSION_*` env grammar
# (`libs/service-http::AdmissionConfig`, wired in `serve()` in
# apps/lumen/src/bin/lumen.rs) onto a declarative CR field (`AdmissionSpec`,
# apps/lumen/src/operator/crd.rs), rendered onto the StatefulSet pod spec by
# `serving_env` in apps/lumen/src/operator/render.rs. Enable it via a CR
# patch, assert the rendered env, then remove the block again so admission
# stays off for the rest of this run.
stop_forward
kubectl -n lumen patch lumen/lumen --type=merge --patch \
  '{"spec":{"admission":{"readCapacity":100,"writeCapacity":50,"adminCapacity":10,"refillSecs":30,"maxKeys":256}}}'
if ! wait_for_admission_env; then
  echo "StatefulSet pod spec did not carry the expected LUMEN_ADMISSION_* env after the CR patch" >&2
  cat "$EVIDENCE_DIR/kubernetes/lumen-admission-env.txt" >&2 || true
  kubectl -n lumen get lumen/lumen -o yaml >&2 || true
  exit 1
fi
kubectl -n lumen rollout status statefulset/lumen --timeout=600s
kubectl -n lumen wait --for=condition=Ready pod/lumen-0 --timeout=300s
kubectl -n lumen patch lumen/lumen --type=json \
  --patch '[{"op":"remove","path":"/spec/admission"}]'
kubectl -n lumen rollout status statefulset/lumen --timeout=600s
kubectl -n lumen wait --for=condition=Ready pod/lumen-0 --timeout=300s
start_forward

test "$(kubectl -n lumen get cronjob/lumen-backup -o jsonpath='{.spec.jobTemplate.spec.template.spec.serviceAccountName}')" = "lumen-backup"
lumen_job="lumen-backup-${RUN_ID}"
kubectl -n lumen create job --from=cronjob/lumen-backup "$lumen_job"
kubectl -n lumen wait --for=condition=Complete "job/$lumen_job" --timeout=600s
kubectl -n lumen logs "job/$lumen_job" > "$EVIDENCE_DIR/kubernetes/lumen-backup.log"
# Seed the cold-restore from the object THIS manual backup Job just wrote — its
# key is reported in the Job's own log and it provably post-dates the
# collection+doc created above. Do NOT fall back to the earliest object
# matching the run prefix: the CR's `*/5` backup cronjob can fire right after
# deploy, before the collection exists, and leave an empty
# `{"version":1,"collections":{}}` manifest. Seeding a fresh PVC from that empty
# object restores to a *queryable-empty* instance — the restore path works
# perfectly yet the `acceptance` collection is absent, which reads as a phantom
# cold-restore product bug.
backup_key="$(jq -r '.object.key // empty' "$EVIDENCE_DIR/kubernetes/lumen-backup.log")"
[[ -n "$backup_key" ]] || {
  echo "manual backup job did not report an object key" >&2
  cat "$EVIDENCE_DIR/kubernetes/lumen-backup.log" >&2
  exit 1
}
first_object="gs://${BACKUP_BUCKET}/${backup_key}"
object_size="$(gcloud storage objects describe "$first_object" --format='value(size)')"
[[ "$object_size" =~ ^[0-9]+$ && "$object_size" -gt 0 ]]
printf '%s\n' "$object_size" > "$EVIDENCE_DIR/gcs/lumen-first-object-bytes.txt"
gcloud storage cat "$first_object" > "$EVIDENCE_DIR/gcs/lumen-first-object.json"
# The backup we seed the cold-restore from must actually contain the collection
# we created — an empty manifest here proves nothing about the restore path.
jq -e '.collections.acceptance' "$EVIDENCE_DIR/gcs/lumen-first-object.json" >/dev/null || {
  echo "backup object $first_object carries no 'acceptance' collection — refusing to seed a hollow cold-restore" >&2
  cat "$EVIDENCE_DIR/gcs/lumen-first-object.json" >&2
  exit 1
}

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
  # #2678: `auth` fails closed. This restore leg probes /search unauthenticated,
  # so it opts out explicitly; the auth legs live in their own CRs below.
  auth: disabled
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

# ---- Auth+CSI regression leg: Secret Manager + SecretProviderClass provider:gke (#2457) ----
# Exercises the integrator's mainstream GKE auth stack end to end — the exact
# #2456 failure path — so it can never silently regress again: GCP Secret
# Manager -> SecretProviderClass (`provider: gke`) -> `auth: required` +
# `tokensSecretProviderClass`/`tokensSecretCsiDriver:
# secrets-store-gke.csi.k8s.io` on a second small CR (`lumen-authcsi`,
# mirrors the `lumen-restore` pattern above so it never destabilizes the main
# instance). Ordered here — after cold-restore, before reshard — so it stays
# grouped with the other secondary-instance legs and never mixes with the
# main instance's disk-pressure/reshard state.
#
# PRECONDITION the next live run must satisfy: the persistent acceptance
# cluster needs the GKE-managed Secret Manager add-on (installs the
# `secrets-store-gke.csi.k8s.io` CSIDriver). It should already be on for the
# persistent cluster, but if not, enable it once (the harness never enables
# cluster features itself — same policy as the required_apis check in
# run.sh):
#   gcloud container clusters update <cluster> --zone <zone> --enable-secret-manager
# If the CSIDriver is absent, this leg records a loud skip in evidence
# instead of failing the whole run.
authcsi_status="passed"
authcsi_skip_reason=""
if ! kubectl get csidrivers.storage.k8s.io secrets-store-gke.csi.k8s.io -o json \
  > "$EVIDENCE_DIR/kubernetes/lumen-authcsi-csidriver.json" \
  2> "$EVIDENCE_DIR/kubernetes/lumen-authcsi-csidriver-absent.txt"; then
  authcsi_status="skipped_no_addon"
  authcsi_skip_reason="secrets-store-gke.csi.k8s.io CSIDriver is not registered on this cluster; enable the GKE Secret Manager add-on with: gcloud container clusters update <cluster> --zone <zone> --enable-secret-manager"
  echo "SKIPPING auth+CSI regression leg (#2457): $authcsi_skip_reason" >&2
  jq -n --arg reason "$authcsi_skip_reason" \
    '{schema:"axiom.gcp.lumen.authcsi.v1", status:"skipped_no_addon", reason:$reason}' \
    > "$EVIDENCE_DIR/kubernetes/lumen-authcsi-skip.json"
else
  : "${LUMEN_AUTHCSI_SECRET_ID:?LUMEN_AUTHCSI_SECRET_ID is required for the auth+CSI leg}"
  # Deterministic from RUN_ID, independently computed the same way Terraform
  # computed the secret payload's key (environment/secretmanager.tf) — no
  # Terraform output roundtrip needed for the token value itself.
  LUMEN_AUTHCSI_TOKEN="axo-${RUN_ID}-lumen-authcsi-token"

  cat <<EOF | kubectl apply -f - > "$EVIDENCE_DIR/kubernetes/lumen-authcsi-secretproviderclass-apply.txt"
apiVersion: secrets-store.csi.x-k8s.io/v1
kind: SecretProviderClass
metadata:
  name: lumen-authcsi-tokens
  namespace: lumen
spec:
  provider: gke
  parameters:
    secrets: |
      - resourceName: "projects/${PROJECT_ID}/secrets/${LUMEN_AUTHCSI_SECRET_ID}/versions/latest"
        path: "token-registry.json"
EOF

  # A separate small CR (not a patch on the main `lumen/lumen` instance):
  # auth mode and the CSI driver are both real production topology choices,
  # not something to toggle on the primary serving instance mid-run — same
  # reasoning as the lumen-restore leg above.
  cat <<EOF | kubectl apply -f - > "$EVIDENCE_DIR/kubernetes/lumen-authcsi-cr-apply.txt"
apiVersion: lumen.dev/v1alpha1
kind: Lumen
metadata:
  name: lumen-authcsi
  namespace: lumen
spec:
  image: ${main_cr_image}
  imagePullPolicy: IfNotPresent
  shardCount: 1
  replicasPerShard: 1
  voterCount: 1
  logFormat: json
  auth: required
  tokensSecretProviderClass: lumen-authcsi-tokens
  tokensSecretCsiDriver: secrets-store-gke.csi.k8s.io
  serving:
    cpu: 500m
    memory: 1Gi
    raftStorage: ${main_cr_raft_storage}
EOF
  wait_ready_authcsi_cr
  kubectl -n lumen get lumen/lumen-authcsi -o json \
    > "$EVIDENCE_DIR/kubernetes/lumen-authcsi-after-apply.json"

  # Explicit CSI-mount + Ready assertions beyond CR-level status: the #2456
  # fail-signature was FailedMount events with the pod stuck at 0/1 ready.
  kubectl -n lumen get pod lumen-authcsi-0 -o json \
    > "$EVIDENCE_DIR/kubernetes/lumen-authcsi-pod.json"
  jq -e '[.spec.volumes[]? | select(.name == "lumen-token-registry" and .csi.driver == "secrets-store-gke.csi.k8s.io")] | length == 1' \
    "$EVIDENCE_DIR/kubernetes/lumen-authcsi-pod.json" >/dev/null || {
    echo "lumen-authcsi-0 pod spec is missing the expected lumen-token-registry CSI volume (driver secrets-store-gke.csi.k8s.io)" >&2
    capture_authcsi_diagnostics
    exit 1
  }
  jq -e '[.status.conditions[]? | select(.type == "Ready" and .status == "True")] | length == 1' \
    "$EVIDENCE_DIR/kubernetes/lumen-authcsi-pod.json" >/dev/null || {
    echo "lumen-authcsi-0 pod is not Ready — the #2456 fail-signature is FailedMount/0-ready" >&2
    capture_authcsi_diagnostics
    exit 1
  }
  kubectl -n lumen get events --field-selector involvedObject.name=lumen-authcsi-0 -o json \
    > "$EVIDENCE_DIR/kubernetes/lumen-authcsi-pod-events.json"
  if jq -e '[.items[] | select(.reason == "FailedMount")] | length > 0' \
    "$EVIDENCE_DIR/kubernetes/lumen-authcsi-pod-events.json" >/dev/null; then
    echo "lumen-authcsi-0 recorded a FailedMount event — the exact #2456 fail-signature" >&2
    jq '[.items[] | select(.reason == "FailedMount")]' "$EVIDENCE_DIR/kubernetes/lumen-authcsi-pod-events.json" >&2
    capture_authcsi_diagnostics
    exit 1
  fi

  start_authcsi_forward
  curl --silent --show-error --fail-with-body -X PUT \
    http://127.0.0.1:17375/collections/acceptance-authcsi \
    -H 'content-type: application/json' \
    -H "authorization: Bearer ${LUMEN_AUTHCSI_TOKEN}" \
    --data '{"fields":{"message":{"type":"keyword"}}}' \
    > "$EVIDENCE_DIR/kubernetes/lumen-authcsi-create-collection.json"
  curl --silent --show-error --fail-with-body -X POST \
    http://127.0.0.1:17375/collections/acceptance-authcsi/index \
    -H 'content-type: application/json' \
    -H "authorization: Bearer ${LUMEN_AUTHCSI_TOKEN}" \
    --data "{\"items\":[{\"external_id\":\"${RUN_ID}\",\"field\":\"message\",\"value\":\"gke-authcsi-${RUN_ID}\"}]}" \
    > "$EVIDENCE_DIR/kubernetes/lumen-authcsi-index.json"

  # Authenticated request must succeed — proves the CSI-mounted
  # token-registry.json actually loaded a working token, not just that the
  # volume mounted.
  authcsi_auth_deadline=$((SECONDS + 60))
  until authcsi_search_authenticated > "$EVIDENCE_DIR/kubernetes/lumen-authcsi-search-authenticated.json" 2>/dev/null \
    && jq -e '.total >= 1' "$EVIDENCE_DIR/kubernetes/lumen-authcsi-search-authenticated.json" >/dev/null 2>&1; do
    if (( SECONDS >= authcsi_auth_deadline )); then
      echo "authenticated search against lumen-authcsi never surfaced the indexed document — the CSI-mounted token registry may not have loaded correctly" >&2
      cat "$EVIDENCE_DIR/kubernetes/lumen-authcsi-search-authenticated.json" >&2 || true
      capture_authcsi_diagnostics
      exit 1
    fi
    kill -0 "$authcsi_forward_pid" >/dev/null 2>&1 || start_authcsi_forward
    sleep 3
  done

  # Unauthenticated request must be rejected — proves auth is actually
  # enforced using the CSI-sourced registry, not silently bypassed (e.g. an
  # empty/unreadable mount that reads as "no registry configured" -> auth
  # effectively off).
  authcsi_unauth_status="$(curl --silent --show-error -o \
    "$EVIDENCE_DIR/kubernetes/lumen-authcsi-search-unauthenticated-body.json" \
    -w '%{http_code}' -X POST \
    http://127.0.0.1:17375/collections/acceptance-authcsi/search \
    -H 'content-type: application/json' \
    --data "{\"query\":{\"term\":{\"field\":\"message\",\"value\":\"gke-authcsi-${RUN_ID}\"}},\"limit\":10}")"
  printf '%s\n' "$authcsi_unauth_status" \
    > "$EVIDENCE_DIR/kubernetes/lumen-authcsi-search-unauthenticated-status.txt"
  if [[ "$authcsi_unauth_status" != "401" ]]; then
    echo "unauthenticated request against lumen-authcsi returned HTTP $authcsi_unauth_status, expected 401 — tokens may not be enforced from the CSI mount" >&2
    cat "$EVIDENCE_DIR/kubernetes/lumen-authcsi-search-unauthenticated-body.json" >&2 || true
    capture_authcsi_diagnostics
    exit 1
  fi
  stop_authcsi_forward

  # Teardown ordering mirrors the lumen-restore leg above: the CR must be
  # gone before the StatefulSet/PVCs, or drift repair recreates the
  # StatefulSet before the owner-ref cascade GC catches up.
  kubectl -n lumen delete lumen/lumen-authcsi --wait=true --timeout=120s
  kubectl -n lumen delete statefulset/lumen-authcsi --wait=true --timeout=180s \
    --cascade=foreground --ignore-not-found
  kubectl -n lumen delete pvc -l app.kubernetes.io/instance=lumen-authcsi \
    --wait=true --timeout=300s --ignore-not-found
  kubectl -n lumen delete secretproviderclass/lumen-authcsi-tokens --ignore-not-found
fi

# ---- lumen-quorum: live multi-voter membership, CR name != binary name (#2610)
#
# Runs BEFORE the 1->2 reshard so the node pool is carrying one lumen shard
# rather than two. The required per-instance hostname anti-affinity puts these
# two pods on separate nodes; they only repel each other, so they share the
# nodes the split leg would otherwise need simultaneously.
cat <<EOF | kubectl apply -f - > "$EVIDENCE_DIR/kubernetes/lumen-quorum-cr-apply.txt"
apiVersion: lumen.dev/v1alpha1
kind: Lumen
metadata:
  name: lumen-quorum
  namespace: lumen
spec:
  image: ${main_cr_image}
  imagePullPolicy: IfNotPresent
  shardCount: 1
  replicasPerShard: 2
  voterCount: 2
  logFormat: json
  serving:
    cpu: 250m
    memory: 512Mi
    raftStorage: 1Gi
EOF
wait_ready_quorum_cr
kubectl -n lumen get lumen/lumen-quorum -o json \
  > "$EVIDENCE_DIR/kubernetes/lumen-quorum-after-apply.json"

# Assertion 1 — the peers are named after the CR, not after the binary.
# This is the direct #2610 signature: the pre-fix response named ITSELF
# `lumen-quorum-0` and its peers `lumen-0`/`lumen-1` in the same object.
quorum_cluster_json lumen-quorum-0 "$EVIDENCE_DIR/kubernetes/lumen-quorum-debug-cluster-0.json" || {
  echo "could not read /debug/cluster from lumen-quorum-0" >&2
  capture_quorum_diagnostics
  exit 1
}
jq -e '[.peers[].pod_name] == ["lumen-quorum-0","lumen-quorum-1"]' \
  "$EVIDENCE_DIR/kubernetes/lumen-quorum-debug-cluster-0.json" >/dev/null || {
  echo "lumen-quorum peers are not named after the CR — #2610 regression" >&2
  jq '{pod_name, role, peers}' "$EVIDENCE_DIR/kubernetes/lumen-quorum-debug-cluster-0.json" >&2 || true
  capture_quorum_diagnostics
  exit 1
}

# Assertion 2 — a leader actually exists. Pod readiness and `status.phase`
# never consult raft, so both members can sit Candidate forever while the CR
# reports Ready 2/2 CONVERGED=True. Roles are the only signal that separates
# a formed quorum from a permanently deadlocked one.
quorum_leader=""
quorum_follower=""
quorum_role_deadline=$((SECONDS + 180))
while (( SECONDS < quorum_role_deadline )); do
  quorum_leaders=0
  for pod in lumen-quorum-0 lumen-quorum-1; do
    quorum_cluster_json "$pod" "$EVIDENCE_DIR/kubernetes/lumen-quorum-debug-cluster-${pod##*-}.json" || continue
    role="$(jq -r '.role' "$EVIDENCE_DIR/kubernetes/lumen-quorum-debug-cluster-${pod##*-}.json" 2>/dev/null || true)"
    if [[ "$role" == "leader" ]]; then
      quorum_leaders=$((quorum_leaders + 1))
      quorum_leader="$pod"
    elif [[ "$role" == "follower" ]]; then
      quorum_follower="$pod"
    fi
  done
  [[ "$quorum_leaders" == "1" && -n "$quorum_follower" ]] && break
  sleep 5
done
if [[ "$quorum_leaders" != "1" || -z "$quorum_leader" || -z "$quorum_follower" ]]; then
  echo "lumen-quorum never settled on exactly one leader and one follower (leaders=$quorum_leaders) — the #2610 deadlock signature" >&2
  capture_quorum_diagnostics
  exit 1
fi
printf '%s\n' "$quorum_leader" > "$EVIDENCE_DIR/kubernetes/lumen-quorum-leader.txt"

# Assertion 3 — the log actually commits and replicates. Election alone is not
# the contract. `applied_index` is read off the raft applied watch channel;
# `replication_lag_ms` deliberately cannot serve here (it is a hardcoded 0 on a
# leader and u64::MAX on anything else, #1349), so it is not consulted.
start_quorum_forward "$quorum_leader"
curl --silent --show-error --fail-with-body -X PUT \
  http://127.0.0.1:17376/collections/acceptance-quorum \
  -H 'content-type: application/json' \
  --data '{"fields":{"message":{"type":"keyword"}}}' \
  > "$EVIDENCE_DIR/kubernetes/lumen-quorum-create-collection.json"
curl --silent --show-error --fail-with-body -X POST \
  http://127.0.0.1:17376/collections/acceptance-quorum/index \
  -H 'content-type: application/json' \
  --data "{\"items\":[{\"external_id\":\"${RUN_ID}\",\"field\":\"message\",\"value\":\"gke-quorum-${RUN_ID}\"}]}" \
  > "$EVIDENCE_DIR/kubernetes/lumen-quorum-index.json"

# Assertion 4 — the FOLLOWER serves a document it never received directly.
# `x-read-consistency: any` is required: the default `leader` level makes a
# follower reject rather than serve a possibly-stale local copy, so without
# this header the read would prove routing, not replication.
quorum_replicated_deadline=$((SECONDS + 120))
until start_quorum_forward "$quorum_follower" \
  && curl --max-time 10 --silent --show-error --fail-with-body -X POST \
    http://127.0.0.1:17376/collections/acceptance-quorum/search \
    -H 'content-type: application/json' \
    -H 'x-read-consistency: any' \
    --data "{\"query\":{\"term\":{\"field\":\"message\",\"value\":\"gke-quorum-${RUN_ID}\"}},\"limit\":10}" \
    > "$EVIDENCE_DIR/kubernetes/lumen-quorum-follower-search.json" 2>/dev/null \
  && jq -e '.total >= 1' "$EVIDENCE_DIR/kubernetes/lumen-quorum-follower-search.json" >/dev/null 2>&1; do
  if (( SECONDS >= quorum_replicated_deadline )); then
    echo "the document written through $quorum_leader never became readable on $quorum_follower — replication did not reach the follower" >&2
    cat "$EVIDENCE_DIR/kubernetes/lumen-quorum-follower-search.json" >&2 || true
    capture_quorum_diagnostics
    exit 1
  fi
  sleep 3
done
quorum_cluster_json "$quorum_follower" "$EVIDENCE_DIR/kubernetes/lumen-quorum-follower-applied.json" || true
stop_quorum_forward

# Teardown ordering mirrors the legs above: CR first, or drift repair recreates
# the StatefulSet before the owner-ref cascade GC catches up.
kubectl -n lumen delete lumen/lumen-quorum --wait=true --timeout=120s
kubectl -n lumen delete statefulset/lumen-quorum --wait=true --timeout=180s \
  --cascade=foreground --ignore-not-found
kubectl -n lumen delete pvc -l app.kubernetes.io/instance=lumen-quorum \
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
  --arg authcsi_status "$authcsi_status" \
  --arg authcsi_skip_reason "$authcsi_skip_reason" \
  --arg quorum_leader "$quorum_leader" \
  --arg quorum_follower "$quorum_follower" \
  '{schema:$schema, operator_reconcile_1x1:"passed", pod_restart_data_retention:"passed", admission_cr_exposure:"passed", gcs_backup_before_split:"passed", gcs_object:$object, gcs_object_bytes:$bytes, cold_restore_fresh_pvc:"passed", seed_set_restart_retention:"passed", auto_split_delta:1, auto_split:{from:1,to:2,ready_pods:2,pvcs_at_least:2}, auth_csi_gke_leg:$authcsi_status, auth_csi_gke_leg_skip_reason:$authcsi_skip_reason, cpu_memory_actuator:"not_claimed", live_replica_membership:"passed", peer_dns_follows_cr_name:{issue:"#2610", cr:"lumen-quorum", leader:$quorum_leader, follower:$quorum_follower, replicated_read:"passed"}}' \
  > "$EVIDENCE_DIR/lumen-acceptance.json"
