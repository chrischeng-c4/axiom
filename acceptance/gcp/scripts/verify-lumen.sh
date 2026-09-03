#!/usr/bin/env bash
set -euo pipefail

: "${PROJECT_ID:?PROJECT_ID is required}"
: "${REGION:?REGION is required}"
: "${RUN_ID:?RUN_ID is required}"
: "${BACKUP_BUCKET:?BACKUP_BUCKET is required}"
: "${EVIDENCE_DIR:?EVIDENCE_DIR is required}"
# The placement leg reads the dedicated data-plane node pool straight off the
# GKE API, so it needs the cluster coordinates and not just a kube context.
: "${GKE_CLUSTER_NAME:?GKE_CLUSTER_NAME is required}"
: "${GKE_ZONE:?GKE_ZONE is required}"

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

# ---- lumen-quorum: multi-voter membership on a CR NOT named after the binary
# (#2610) helpers ----
#
# Every pre-existing leg misses this bug at the intersection of two conditions.
# The multi-member cases (`lumen/lumen` here, `tape/tape` in verify-tape.sh)
# name their CR after the binary, so the peer prefix the binary hardcoded
# happened to equal the one derived from POD_NAME. The differently-named cases
# (`lumen-restore`) run replicasPerShard:1, where `--wal auto`
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
  # `kubectl describe` drops the Events section once GKE has aged the events
  # out, which is exactly the information a Pending pod's failure turns on.
  kubectl -n lumen get events --sort-by=.lastTimestamp \
    > "$EVIDENCE_DIR/kubernetes/lumen-quorum-events.txt" 2>&1 || true
  # Separate "could not be placed" from "placed and unhealthy". Both surface as
  # readyReplicas != 2, and only the second is a Lumen defect: a Pending pod
  # under this leg's REQUIRED hostname anti-affinity means the cluster ran out
  # of distinct nodes, and `cluster_autoscaler_unhelpable_until: Inf` is the
  # autoscaler saying no node it is allowed to add would help -- the pool is at
  # its ceiling. Naming that here stops the next reader from bisecting Lumen
  # for a node-pool ceiling (cluster/main.tf, acceptance-pool max_node_count).
  kubectl -n lumen get pods -l app.kubernetes.io/instance=lumen-quorum -o json 2>/dev/null \
    | jq -r '.items[]
        | select(any(.status.conditions[]?; .type == "PodScheduled" and .status != "True"))
        | "UNSCHEDULABLE \(.metadata.name): "
          + ((.status.conditions[] | select(.type == "PodScheduled") | .message) // "no message")
          + " | autoscaler-unhelpable-until="
          + (.metadata.annotations["cloud.google.com/cluster_autoscaler_unhelpable_until"] // "n/a")' \
      2>/dev/null | while read -r line; do
    echo "$line" >&2
    echo "  ^ node-pool capacity shortfall, not a Lumen failure: this leg's replicas carry" >&2
    echo "    required hostname anti-affinity, so each one needs a node of its own." >&2
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
  # #2678: auth fails closed, so every CR this script writes by hand must say
  # what it wants. This restore leg probes /search unauthenticated and opts out;
  # the auth legs live in their own CRs below.
  #
  # No backticks in a heredoc comment: <<EOF is unquoted, so the shell runs
  # command substitution on this line too — comment or not.
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
  # #2678: this leg proves peer naming and quorum, not auth. Opt out explicitly.
  auth: disabled
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

# ---- Placement leg: spec.placement onto a dedicated node pool ----
#
# The two halves of `spec.placement` are proven by making each one, on its own,
# the reason the pod cannot run:
#
#   phase 1  nodeSelector only  -> Pending. The label matches ONLY the tainted
#            data-plane pool, and without a toleration nothing can land there;
#            no other node carries the label. If the operator dropped
#            `nodeSelector`, the pod would schedule on acceptance-pool and this
#            phase would fail.
#   phase 2  + tolerations      -> Running, on a data-plane-pool node. If the
#            operator dropped `tolerations`, the pod would stay Pending and
#            this phase would fail.
#
# Neither half can be silently missing and still produce both outcomes, which
# is what a same-pool run cannot distinguish.
placement_pool="${DATA_PLANE_POOL_NAME:-data-plane-pool}"
placement_label_key="${DATA_PLANE_LABEL_KEY:-axiom.dev/pool}"
placement_label_value="${DATA_PLANE_LABEL_VALUE:-data-plane}"
placement_taint_key="${DATA_PLANE_TAINT_KEY:-axiom.dev/dedicated}"

capture_placement_diagnostics() {
  kubectl -n lumen get lumen/lumen-placement -o yaml \
    > "$EVIDENCE_DIR/kubernetes/lumen-placement-cr.yaml" 2>&1 || true
  kubectl -n lumen describe pod lumen-placement-0 \
    > "$EVIDENCE_DIR/kubernetes/lumen-placement-pod-describe.txt" 2>&1 || true
  kubectl get nodes -o json > "$EVIDENCE_DIR/kubernetes/nodes-during-placement.json" 2>&1 || true
}

# Fail on cluster drift BEFORE applying anything. The persistent cluster is
# reused and therefore never re-applied, so cluster/main.tf can say one thing
# while the live cluster says another — exactly how the Secret Manager add-on
# was lost. Ten seconds here beats a phase-2 timeout ten minutes in.
gcloud container node-pools describe "$placement_pool" \
  --cluster="$GKE_CLUSTER_NAME" --zone="$GKE_ZONE" --project="$PROJECT_ID" \
  --format=json > "$EVIDENCE_DIR/kubernetes/data-plane-node-pool.json" 2>"$EVIDENCE_DIR/kubernetes/data-plane-node-pool-absent.txt" || {
  echo "node pool '$placement_pool' does not exist on $GKE_CLUSTER_NAME; spec.placement cannot be proven against a pool boundary." >&2
  echo "  it is declared in acceptance/gcp/cluster/main.tf — apply it against the persistent cluster's state:" >&2
  echo "  TF_DATA_DIR=/tmp/axiom-gcp-operator-cluster/.terraform terraform -chdir=acceptance/gcp/cluster init -backend-config=bucket=\$PROJECT_ID-axiom-tfstate && terraform -chdir=acceptance/gcp/cluster apply ..." >&2
  exit 1
}
jq -e --arg k "$placement_label_key" --arg v "$placement_label_value" \
  '.config.labels[$k] == $v' \
  "$EVIDENCE_DIR/kubernetes/data-plane-node-pool.json" >/dev/null || {
  echo "node pool '$placement_pool' does not carry label $placement_label_key=$placement_label_value; the phase-1 Pending assertion would pass for the wrong reason" >&2
  jq '.config.labels' "$EVIDENCE_DIR/kubernetes/data-plane-node-pool.json" >&2 || true
  exit 1
}
jq -e --arg k "$placement_taint_key" --arg v "$placement_label_value" \
  '[.config.taints[]? | select(.key == $k and .value == $v and .effect == "NO_SCHEDULE")] | length == 1' \
  "$EVIDENCE_DIR/kubernetes/data-plane-node-pool.json" >/dev/null || {
  echo "node pool '$placement_pool' does not carry the NoSchedule taint $placement_taint_key=$placement_label_value; without it a pod missing its tolerations would still schedule and the leg would false-green" >&2
  jq '.config.taints' "$EVIDENCE_DIR/kubernetes/data-plane-node-pool.json" >&2 || true
  exit 1
}

cat <<EOF | kubectl apply -f - > "$EVIDENCE_DIR/kubernetes/lumen-placement-cr-apply.txt"
apiVersion: lumen.dev/v1alpha1
kind: Lumen
metadata:
  name: lumen-placement
  namespace: lumen
spec:
  image: ${main_cr_image}
  imagePullPolicy: IfNotPresent
  shardCount: 1
  replicasPerShard: 1
  voterCount: 1
  logFormat: json
  # #2678: this leg proves scheduling, not auth. Opt out explicitly.
  auth: disabled
  placement:
    nodeSelector:
      ${placement_label_key}: ${placement_label_value}
  serving:
    cpu: 250m
    memory: 512Mi
    raftStorage: 1Gi
EOF

# The rendered pod template must carry the selector before the scheduling
# outcome means anything: a Pending pod with NO nodeSelector would be Pending
# for an unrelated reason and would read as a pass.
placement_render_deadline=$((SECONDS + 180))
until kubectl -n lumen get statefulset/lumen-placement -o json \
  > "$EVIDENCE_DIR/kubernetes/lumen-placement-statefulset.json" 2>/dev/null \
  && jq -e --arg k "$placement_label_key" --arg v "$placement_label_value" \
    '.spec.template.spec.nodeSelector[$k] == $v' \
    "$EVIDENCE_DIR/kubernetes/lumen-placement-statefulset.json" >/dev/null 2>&1; do
  if (( SECONDS >= placement_render_deadline )); then
    echo "the operator never rendered spec.placement.nodeSelector onto the lumen-placement StatefulSet pod template" >&2
    jq '.spec.template.spec | {nodeSelector, tolerations}' \
      "$EVIDENCE_DIR/kubernetes/lumen-placement-statefulset.json" >&2 || true
    capture_placement_diagnostics
    exit 1
  fi
  sleep 5
done

# Phase 1 — hold for a stable window. A single instantaneous read of Pending is
# just "the scheduler has not run yet"; what proves the selector binds is that
# the pod is STILL unschedulable after the scheduler has had every chance.
placement_phase1_deadline=$((SECONDS + 90))
while (( SECONDS < placement_phase1_deadline )); do
  placement_phase="$(kubectl -n lumen get pod lumen-placement-0 -o jsonpath='{.status.phase}' 2>/dev/null || true)"
  if [[ -n "$placement_phase" && "$placement_phase" != "Pending" ]]; then
    echo "lumen-placement-0 reached phase '$placement_phase' with a nodeSelector for the tainted $placement_pool and NO toleration — it must not be schedulable anywhere" >&2
    kubectl -n lumen get pod lumen-placement-0 -o wide >&2 || true
    capture_placement_diagnostics
    exit 1
  fi
  sleep 5
done
kubectl -n lumen get pod lumen-placement-0 -o json \
  > "$EVIDENCE_DIR/kubernetes/lumen-placement-pod-phase1.json" 2>/dev/null || true
jq -e '[.status.conditions[]? | select(.type == "PodScheduled" and .status == "False" and .reason == "Unschedulable")] | length == 1' \
  "$EVIDENCE_DIR/kubernetes/lumen-placement-pod-phase1.json" >/dev/null || {
  echo "lumen-placement-0 is Pending but not for the Unschedulable reason — the nodeSelector may not be what is holding it" >&2
  jq '.status.conditions' "$EVIDENCE_DIR/kubernetes/lumen-placement-pod-phase1.json" >&2 || true
  capture_placement_diagnostics
  exit 1
}

# Phase 2 — add the toleration. The data-plane pool is scale-to-zero, so this
# also exercises cluster-autoscaler scale-FROM-zero against a pool whose labels
# and taints only exist on the pool definition; the wait is sized for node
# provisioning, not for reconcile.
kubectl -n lumen patch lumen/lumen-placement --type=merge --patch "$(jq -n \
  --arg key "$placement_taint_key" --arg value "$placement_label_value" \
  '{spec:{placement:{tolerations:[{key:$key, operator:"Equal", value:$value, effect:"NoSchedule"}]}}}')" \
  > "$EVIDENCE_DIR/kubernetes/lumen-placement-toleration-patch.txt"

placement_ready_deadline=$((SECONDS + 720))
until [[ "$(kubectl -n lumen get statefulset/lumen-placement -o jsonpath='{.status.readyReplicas}' 2>/dev/null || true)" == "1" ]]; do
  if (( SECONDS >= placement_ready_deadline )); then
    echo "lumen-placement-0 never became ready after the toleration was added — either the operator drops spec.placement.tolerations, or $placement_pool did not scale from zero" >&2
    capture_placement_diagnostics
    exit 1
  fi
  sleep 10
done

# It landed — but landing is only meaningful if it landed on the DEDICATED
# pool. Read the node it actually bound to and confirm the pool identity from
# the node's own labels rather than from the pod spec that requested them.
placement_node="$(kubectl -n lumen get pod lumen-placement-0 -o jsonpath='{.spec.nodeName}')"
printf '%s\n' "$placement_node" > "$EVIDENCE_DIR/kubernetes/lumen-placement-node.txt"
kubectl get node "$placement_node" -o json \
  > "$EVIDENCE_DIR/kubernetes/lumen-placement-node.json"
jq -e --arg k "$placement_label_key" --arg v "$placement_label_value" --arg pool "$placement_pool" \
  '.metadata.labels[$k] == $v and .metadata.labels["cloud.google.com/gke-nodepool"] == $pool' \
  "$EVIDENCE_DIR/kubernetes/lumen-placement-node.json" >/dev/null || {
  echo "lumen-placement-0 is running on '$placement_node', which is not a $placement_pool node" >&2
  jq '.metadata.labels' "$EVIDENCE_DIR/kubernetes/lumen-placement-node.json" >&2 || true
  capture_placement_diagnostics
  exit 1
}
# And the negative direction: the control plane, which declares no placement,
# must NOT be on the dedicated pool. Otherwise "the data plane is isolated" is
# a claim about one pod rather than about the topology.
kubectl -n lumen-system get pods -o json \
  > "$EVIDENCE_DIR/kubernetes/lumen-operator-pods-during-placement.json"
jq -e --arg node "$placement_node" \
  '[.items[] | select(.spec.nodeName == $node)] | length == 0' \
  "$EVIDENCE_DIR/kubernetes/lumen-operator-pods-during-placement.json" >/dev/null || {
  echo "an operator pod is running on the dedicated data-plane node '$placement_node'; the taint is not isolating the pool" >&2
  capture_placement_diagnostics
  exit 1
}

kubectl -n lumen delete lumen/lumen-placement --wait=true --timeout=120s
kubectl -n lumen delete statefulset/lumen-placement --wait=true --timeout=180s \
  --cascade=foreground --ignore-not-found
kubectl -n lumen delete pvc -l app.kubernetes.io/instance=lumen-placement \
  --wait=true --timeout=300s --ignore-not-found

# ---- Fleet leg: one cluster-scoped LumenFleet materializing data planes ----
#
# The control-plane model under test: the platform team applies ONE
# cluster-scoped object in lumen-system that declares every data-plane
# namespace, and the operator materializes a `Lumen` into each. What this leg
# proves is everything a unit test cannot: real cross-namespace creation, real
# server-side-apply field ownership, and the two deletion contracts.
fleet_ns_a="lumen-fleet-a"
fleet_ns_b="lumen-fleet-b"
fleet_name="acceptance"

capture_fleet_diagnostics() {
  kubectl get lumenfleet/"$fleet_name" -o yaml \
    > "$EVIDENCE_DIR/kubernetes/lumen-fleet-cr.yaml" 2>&1 || true
  kubectl get lumen -A -o yaml \
    > "$EVIDENCE_DIR/kubernetes/lumen-fleet-all-lumens.yaml" 2>&1 || true
  kubectl -n lumen-system logs deployment/lumen-operator --tail=300 \
    > "$EVIDENCE_DIR/kubernetes/lumen-fleet-operator.log" 2>&1 || true
}

# Wait until the fleet's status reflects the generation we just applied. Every
# assertion below reads status, and the loop polls on a 30s interval, so
# reading it without this wait reads the PREVIOUS pass's answer.
wait_fleet_observed() {
  local deadline=$((SECONDS + 180))
  local generation observed
  while (( SECONDS < deadline )); do
    generation="$(kubectl get lumenfleet/"$fleet_name" -o jsonpath='{.metadata.generation}' 2>/dev/null || true)"
    observed="$(kubectl get lumenfleet/"$fleet_name" -o jsonpath='{.status.observedGeneration}' 2>/dev/null || true)"
    if [[ -n "$generation" && "$observed" == "$generation" ]]; then
      return 0
    fi
    sleep 5
  done
  echo "the fleet controller never reported observedGeneration == metadata.generation" >&2
  capture_fleet_diagnostics
  return 1
}

fleet_entry_state() {
  # $1 namespace, $2 name -> the entry's state, or empty
  kubectl get lumenfleet/"$fleet_name" -o json 2>/dev/null \
    | jq -r --arg ns "$1" --arg name "$2" \
      'first(.status.entries[]? | select(.namespace == $ns and .name == $name) | .state) // ""'
}

kubectl create namespace "$fleet_ns_a" --dry-run=client -o yaml | kubectl apply -f - \
  > "$EVIDENCE_DIR/kubernetes/lumen-fleet-namespaces.txt"
kubectl create namespace "$fleet_ns_b" --dry-run=client -o yaml | kubectl apply -f - \
  >> "$EVIDENCE_DIR/kubernetes/lumen-fleet-namespaces.txt"

# Four entries, chosen so one pass exercises every outcome the design has:
# inherit-only, override, absent namespace, and a misspelled override.
cat <<EOF | kubectl apply -f - > "$EVIDENCE_DIR/kubernetes/lumen-fleet-apply.txt"
apiVersion: lumen.dev/v1alpha1
kind: LumenFleet
metadata:
  name: ${fleet_name}
spec:
  prunePolicy: Retain
  defaults:
    image: ${main_cr_image}
    imagePullPolicy: IfNotPresent
    shardCount: 1
    replicasPerShard: 1
    voterCount: 1
    logFormat: json
    # #2678: this leg proves fan-out and inheritance, not auth. Opting out in
    # defaults is also the shape a platform-wide default is meant to take —
    # unlike identities, a scalar merges cleanly into every instance.
    auth: disabled
    serving:
      cpu: 250m
      memory: 512Mi
      raftStorage: 1Gi
  instances:
    - namespace: ${fleet_ns_a}
    - namespace: ${fleet_ns_b}
      name: lumen-b
      spec:
        serving:
          memory: 640Mi
    - namespace: lumen-fleet-absent
      name: never
    - namespace: ${fleet_ns_a}
      name: bad-override
      spec:
        serving:
          memoryy: 1Gi
EOF
wait_fleet_observed
kubectl get lumenfleet/"$fleet_name" -o json \
  > "$EVIDENCE_DIR/kubernetes/lumen-fleet-status-initial.json"

jq -e '.status.desiredInstances == 4 and .status.appliedInstances == 2' \
  "$EVIDENCE_DIR/kubernetes/lumen-fleet-status-initial.json" >/dev/null || {
  echo "the fleet did not report 2 of 4 entries converged" >&2
  jq '.status' "$EVIDENCE_DIR/kubernetes/lumen-fleet-status-initial.json" >&2 || true
  capture_fleet_diagnostics
  exit 1
}
[[ "$(fleet_entry_state "$fleet_ns_a" "$fleet_name")" == "Created" ]] || {
  echo "the inherit-only entry did not report Created" >&2
  capture_fleet_diagnostics
  exit 1
}
[[ "$(fleet_entry_state lumen-fleet-absent never)" == "NamespaceMissing" ]] || {
  echo "an entry naming a nonexistent namespace did not report NamespaceMissing — the fleet may be creating namespaces, which its ClusterRole must never allow" >&2
  capture_fleet_diagnostics
  exit 1
}
[[ "$(fleet_entry_state "$fleet_ns_a" bad-override)" == "Rejected" ]] || {
  echo "a misspelled override (serving.memoryy) did not report Rejected — a typo that merges silently would leave a tenant on the defaults with no signal" >&2
  capture_fleet_diagnostics
  exit 1
}
# The rejection must NAME the field. "invalid spec" is not a diagnosis anyone
# can act on, and this is the message a human debugs a fleet from.
jq -e --arg ns "$fleet_ns_a" \
  'first(.status.entries[] | select(.namespace == $ns and .name == "bad-override") | .message) | test("serving.memoryy")' \
  "$EVIDENCE_DIR/kubernetes/lumen-fleet-status-initial.json" >/dev/null || {
  echo "the rejection message does not name the offending field serving.memoryy" >&2
  jq '.status.entries' "$EVIDENCE_DIR/kubernetes/lumen-fleet-status-initial.json" >&2 || true
  capture_fleet_diagnostics
  exit 1
}
kubectl -n "$fleet_ns_a" get lumen/bad-override -o json >/dev/null 2>&1 && {
  echo "a rejected entry still materialized a Lumen" >&2
  capture_fleet_diagnostics
  exit 1
}

kubectl -n "$fleet_ns_a" get lumen/"$fleet_name" -o json \
  > "$EVIDENCE_DIR/kubernetes/lumen-fleet-instance-a.json"
kubectl -n "$fleet_ns_b" get lumen/lumen-b -o json \
  > "$EVIDENCE_DIR/kubernetes/lumen-fleet-instance-b.json"

# The override merges rather than replaces: memory is the tenant's, cpu and
# raftStorage are still the fleet's.
jq -e '.spec.serving.memory == "640Mi" and .spec.serving.cpu == "250m" and .spec.serving.raftStorage == "1Gi"' \
  "$EVIDENCE_DIR/kubernetes/lumen-fleet-instance-b.json" >/dev/null || {
  echo "the per-instance override did not merge over the fleet defaults" >&2
  jq '.spec.serving' "$EVIDENCE_DIR/kubernetes/lumen-fleet-instance-b.json" >&2 || true
  capture_fleet_diagnostics
  exit 1
}
jq -e '.spec.serving.memory == "512Mi"' \
  "$EVIDENCE_DIR/kubernetes/lumen-fleet-instance-a.json" >/dev/null || {
  echo "the inherit-only instance did not take the fleet default memory" >&2
  capture_fleet_diagnostics
  exit 1
}
# No ownerReference, by design: an ownerRef would make deleting the fleet
# cascade into deleting every data plane and its PVCs, which is the exact
# opposite of prunePolicy: Retain.
jq -e '(.metadata.ownerReferences // []) | length == 0' \
  "$EVIDENCE_DIR/kubernetes/lumen-fleet-instance-a.json" >/dev/null || {
  echo "a fleet-materialized Lumen carries an ownerReference; deleting the fleet would cascade-delete data planes" >&2
  jq '.metadata.ownerReferences' "$EVIDENCE_DIR/kubernetes/lumen-fleet-instance-a.json" >&2 || true
  capture_fleet_diagnostics
  exit 1
}
jq -e --arg fleet "$fleet_name" '.metadata.labels["lumen.dev/fleet"] == $fleet' \
  "$EVIDENCE_DIR/kubernetes/lumen-fleet-instance-a.json" >/dev/null || {
  echo "a fleet-materialized Lumen is missing the lumen.dev/fleet label the fleet tracks it by" >&2
  capture_fleet_diagnostics
  exit 1
}

# ---- The SSA field-ownership invariant, read from the API server ----
# The reshard driver writes spec.shardCount / spec.shardMap /
# spec.reshardPolicy.workflow at runtime. If the steady-state fleet apply ever
# OWNED those paths, the next pass that omitted them would prune a completed
# split back to its seed topology. The design's answer is two field managers:
# a one-time create under lumen-fleet-seed, and a steady-state apply under
# lumen-fleet whose apply-set never names those paths. Kubernetes' own
# managedFields bookkeeping is the direct oracle for that — no behavioural
# simulation needed, and nothing to destabilize.
fleet_apply_deadline=$((SECONDS + 120))
until kubectl -n "$fleet_ns_a" get lumen/"$fleet_name" --show-managed-fields -o json \
  > "$EVIDENCE_DIR/kubernetes/lumen-fleet-managed-fields.json" 2>/dev/null \
  && jq -e '[.metadata.managedFields[] | select(.manager == "lumen-fleet")] | length == 1' \
    "$EVIDENCE_DIR/kubernetes/lumen-fleet-managed-fields.json" >/dev/null 2>&1; do
  if (( SECONDS >= fleet_apply_deadline )); then
    echo "the steady-state 'lumen-fleet' field manager never appeared; the fleet may only ever be creating, never re-applying" >&2
    jq '[.metadata.managedFields[].manager]' "$EVIDENCE_DIR/kubernetes/lumen-fleet-managed-fields.json" >&2 || true
    capture_fleet_diagnostics
    exit 1
  fi
  sleep 10
done
jq -e '[.metadata.managedFields[] | select(.manager == "lumen-fleet-seed")] | length == 1' \
  "$EVIDENCE_DIR/kubernetes/lumen-fleet-managed-fields.json" >/dev/null || {
  echo "the one-time 'lumen-fleet-seed' field manager is absent; the initial topology has no owner and the next apply could prune it" >&2
  jq '[.metadata.managedFields[].manager]' "$EVIDENCE_DIR/kubernetes/lumen-fleet-managed-fields.json" >&2 || true
  capture_fleet_diagnostics
  exit 1
}
# The three paths, spelled exactly as `DRIVER_OWNED_PATHS` in
# apps/lumen/src/operator/fleet.rs spells them -- `reshardPolicy.workflow` is a
# LEAF of reshardPolicy, not the whole subtree.
#
# This used to test the top-level key `f:reshardPolicy` and it red-flagged a
# correct fleet (run 0727153733). `prepareAtPercent`/`urgentAtPercent` ARE fleet
# policy and the steady-state apply is supposed to own them -- fleet.rs's own
# unit test asserts "the rest of reshardPolicy is fleet-declared policy and must
# stay". Only `workflow` belongs to the reshard driver. A coarse check that
# cannot tell a subtree from a leaf fails the very design it exists to protect.
jq -e '[.metadata.managedFields[] | select(.manager == "lumen-fleet") | .fieldsV1["f:spec"] // {}][0] as $applied
       | ($applied | has("f:shardCount") | not)
         and ($applied | has("f:shardMap") | not)
         and (($applied["f:reshardPolicy"] // {}) | has("f:workflow") | not)' \
  "$EVIDENCE_DIR/kubernetes/lumen-fleet-managed-fields.json" >/dev/null || {
  echo "the steady-state 'lumen-fleet' apply-set claims a path the reshard driver owns; a later pass would revert a completed split" >&2
  # The whole subtree, not `keys`: the nesting IS the finding, and printing only
  # the top level is what made this failure unreadable the first time.
  jq '[.metadata.managedFields[] | select(.manager == "lumen-fleet") | .fieldsV1["f:spec"]]' \
    "$EVIDENCE_DIR/kubernetes/lumen-fleet-managed-fields.json" >&2 || true
  capture_fleet_diagnostics
  exit 1
}
# The mirror of the check above, and the reason the coarse version was actively
# harmful rather than merely wrong: a fleet that stopped propagating its own
# policy would pass the negative test trivially, by never naming reshardPolicy
# at all. Ownership of the thresholds is the positive half of the same contract.
jq -e '[.metadata.managedFields[] | select(.manager == "lumen-fleet")
        | .fieldsV1["f:spec"]["f:reshardPolicy"] // {}][0]
       | has("f:prepareAtPercent") and has("f:urgentAtPercent")' \
  "$EVIDENCE_DIR/kubernetes/lumen-fleet-managed-fields.json" >/dev/null || {
  echo "the steady-state 'lumen-fleet' apply-set does not own its own reshard policy thresholds; fleet-declared policy is not being propagated" >&2
  jq '[.metadata.managedFields[] | select(.manager == "lumen-fleet") | .fieldsV1["f:spec"]["f:reshardPolicy"]]' \
    "$EVIDENCE_DIR/kubernetes/lumen-fleet-managed-fields.json" >&2 || true
  capture_fleet_diagnostics
  exit 1
}

# A fleet-materialized Lumen is a real instance, not just an object: hold the
# inherit-only one to Ready so "the fleet deployed it" means it serves.
fleet_ready_deadline=$((SECONDS + 600))
until [[ "$(kubectl -n "$fleet_ns_a" get lumen/"$fleet_name" -o jsonpath='{.status.phase}' 2>/dev/null || true)" == "Ready" ]]; do
  if (( SECONDS >= fleet_ready_deadline )); then
    echo "the fleet-materialized Lumen in $fleet_ns_a never reached Ready" >&2
    capture_fleet_diagnostics
    exit 1
  fi
  sleep 10
done

# ---- Prune contract, both policies ----
# Removing a line from a list is a plausible edit; deleting a search index with
# its PVCs is not a plausible consequence of one. Under Retain the instance
# must survive and be REPORTED as orphaned — silence would be the dangerous
# outcome, not the safe one.
kubectl patch lumenfleet/"$fleet_name" --type=json \
  --patch '[{"op":"remove","path":"/spec/instances/1"}]' \
  > "$EVIDENCE_DIR/kubernetes/lumen-fleet-retain-patch.txt"
wait_fleet_observed
kubectl get lumenfleet/"$fleet_name" -o json \
  > "$EVIDENCE_DIR/kubernetes/lumen-fleet-status-retain.json"
[[ "$(fleet_entry_state "$fleet_ns_b" lumen-b)" == "Orphaned" ]] || {
  echo "an undeclared instance was not reported as Orphaned under prunePolicy: Retain" >&2
  jq '.status.entries' "$EVIDENCE_DIR/kubernetes/lumen-fleet-status-retain.json" >&2 || true
  capture_fleet_diagnostics
  exit 1
}
kubectl -n "$fleet_ns_b" get lumen/lumen-b -o json >/dev/null 2>&1 || {
  echo "prunePolicy: Retain deleted an instance that merely left the fleet's list" >&2
  capture_fleet_diagnostics
  exit 1
}

kubectl patch lumenfleet/"$fleet_name" --type=merge \
  --patch '{"spec":{"prunePolicy":"Delete"}}' \
  > "$EVIDENCE_DIR/kubernetes/lumen-fleet-delete-policy-patch.txt"
wait_fleet_observed
fleet_prune_deadline=$((SECONDS + 180))
until ! kubectl -n "$fleet_ns_b" get lumen/lumen-b >/dev/null 2>&1; do
  if (( SECONDS >= fleet_prune_deadline )); then
    echo "prunePolicy: Delete did not remove the undeclared instance" >&2
    capture_fleet_diagnostics
    exit 1
  fi
  sleep 10
done
kubectl get lumenfleet/"$fleet_name" -o json \
  > "$EVIDENCE_DIR/kubernetes/lumen-fleet-status-pruned.json"

# ---- The deletion contract that the ownerReference decision exists for ----
# Deleting the fleet declaration must NOT delete the running data planes. This
# is the single assertion that would have caught an ownerReference sneaking
# back in, and it is unprovable without a live API server's GC.
kubectl delete lumenfleet/"$fleet_name" --wait=true --timeout=120s \
  > "$EVIDENCE_DIR/kubernetes/lumen-fleet-delete.txt"
sleep 45
kubectl -n "$fleet_ns_a" get lumen/"$fleet_name" -o json \
  > "$EVIDENCE_DIR/kubernetes/lumen-fleet-instance-a-after-fleet-delete.json" || {
  echo "deleting the LumenFleet took its data plane with it — cross-object garbage collection is exactly what the no-ownerReference design forbids" >&2
  capture_fleet_diagnostics
  exit 1
}
jq -e '.status.phase == "Ready"' \
  "$EVIDENCE_DIR/kubernetes/lumen-fleet-instance-a-after-fleet-delete.json" >/dev/null || {
  echo "the surviving data plane is no longer Ready after its fleet was deleted" >&2
  capture_fleet_diagnostics
  exit 1
}

kubectl -n "$fleet_ns_a" delete lumen/"$fleet_name" --wait=true --timeout=120s
kubectl -n "$fleet_ns_a" delete statefulset/"$fleet_name" --wait=true --timeout=180s \
  --cascade=foreground --ignore-not-found
kubectl -n "$fleet_ns_a" delete pvc --all --wait=true --timeout=300s --ignore-not-found
kubectl delete namespace "$fleet_ns_a" "$fleet_ns_b" --wait=true --timeout=300s --ignore-not-found

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
  --arg quorum_leader "$quorum_leader" \
  --arg quorum_follower "$quorum_follower" \
  --arg placement_pool "$placement_pool" \
  --arg placement_node "$placement_node" \
  '{schema:$schema, operator_reconcile_1x1:"passed", pod_restart_data_retention:"passed", admission_cr_exposure:"passed", gcs_backup_before_split:"passed", gcs_object:$object, gcs_object_bytes:$bytes, cold_restore_fresh_pvc:"passed", seed_set_restart_retention:"passed", auto_split_delta:1, auto_split:{from:1,to:2,ready_pods:2,pvcs_at_least:2}, cpu_memory_actuator:"not_claimed", live_replica_membership:"passed", peer_dns_follows_cr_name:{issue:"#2610", cr:"lumen-quorum", leader:$quorum_leader, follower:$quorum_follower, replicated_read:"passed"}, dedicated_node_pool_placement:{cr:"lumen-placement", pool:$placement_pool, node:$placement_node, node_selector_alone_unschedulable:"passed", toleration_scheduled_from_zero:"passed", control_plane_excluded:"passed"}, control_plane_fleet:{cr:"LumenFleet/acceptance", desired:4, applied:2, namespace_missing_reported:"passed", misspelled_override_rejected:"passed", override_merged_over_defaults:"passed", no_owner_reference:"passed", driver_owned_paths_unclaimed:"passed", retain_orphans_instance:"passed", delete_prunes_instance:"passed", fleet_delete_leaves_data_plane:"passed"}}' \
  > "$EVIDENCE_DIR/lumen-acceptance.json"
