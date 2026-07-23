#!/usr/bin/env bash
set -euo pipefail

: "${PROJECT_ID:?PROJECT_ID is required}"
: "${REGION:?REGION is required}"
: "${RUN_ID:?RUN_ID is required}"
: "${BACKUP_BUCKET:?BACKUP_BUCKET is required}"
: "${BACKUP_GSA_EMAIL:?BACKUP_GSA_EMAIL is required}"
: "${MANIFEST_DIR:?MANIFEST_DIR is required}"
: "${EVIDENCE_DIR:?EVIDENCE_DIR is required}"

mkdir -p "$EVIDENCE_DIR/kubernetes" "$EVIDENCE_DIR/gcs"
topic="acceptance"
sub="acceptance-sub"
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
    # The operator-cell test intentionally replaces a Tape pod immediately
    # before this probe. `kubectl port-forward service/...` pins one selected
    # endpoint, so it can exit while the Service already has a healthy
    # replacement. Recreate the local forward rather than turning that normal
    # hand-off into a false readiness failure.
    if [[ -z "$forward_pid" ]] || ! kill -0 "$forward_pid" >/dev/null 2>&1; then
      stop_forward
      kubectl -n tape port-forward service/tape 17137:7137 \
        >>"$EVIDENCE_DIR/kubernetes/tape-port-forward.log" 2>&1 &
      forward_pid="$!"
      sleep 1
    fi
    if curl --max-time 5 --silent --show-error --fail \
      http://127.0.0.1:17137/readyz >/dev/null 2>&1; then
      return 0
    fi
    sleep 2
  done
  echo "timed out waiting for Tape service readiness through port-forward" >&2
  return 1
}

append_event() {
  local payload="$1"
  curl --silent --show-error --fail-with-body -X POST \
    "http://127.0.0.1:17137/topics/${topic}/append" \
    -H 'content-type: application/json' \
    --data "{\"key\":\"${RUN_ID}\",\"payload\":${payload}}"
}

replay_events() {
  curl --silent --show-error --fail-with-body \
    "http://127.0.0.1:17137/topics/${topic}/replay"
}

get_checkpoint() {
  curl --silent --show-error --fail-with-body \
    "http://127.0.0.1:17137/topics/${topic}/consumers/${sub}/checkpoint"
}

wait_for_gcs_object() {
  local prefix="gs://${BACKUP_BUCKET}/tape"
  local listing="$EVIDENCE_DIR/gcs/tape-objects.txt"
  local deadline=$((SECONDS + 180))
  local first
  while (( SECONDS < deadline )); do
    gcloud storage ls --recursive "${prefix}/**" > "$listing" 2>/dev/null || true
    first="$(rg -F "/tape/${RUN_ID}-" "$listing" | sed -n '1p' || true)"
    if [[ -n "$first" ]]; then
      printf '%s\n' "$first"
      return 0
    fi
    sleep 3
  done
  echo "no Tape backup object for run $RUN_ID appeared below $prefix" >&2
  return 1
}

wait_for_topology() {
  local want="$1"
  local deadline=$((SECONDS + 900))
  local generation observed phase desired ready
  while (( SECONDS < deadline )); do
    generation="$(kubectl -n tape get tape/tape -o jsonpath='{.metadata.generation}' 2>/dev/null || true)"
    observed="$(kubectl -n tape get tape/tape -o jsonpath='{.status.observedGeneration}' 2>/dev/null || true)"
    phase="$(kubectl -n tape get tape/tape -o jsonpath='{.status.phase}' 2>/dev/null || true)"
    desired="$(kubectl -n tape get statefulset/tape -o jsonpath='{.spec.replicas}' 2>/dev/null || true)"
    ready="$(kubectl -n tape get statefulset/tape -o jsonpath='{.status.readyReplicas}' 2>/dev/null || true)"
    if [[ -n "$generation" && "$observed" == "$generation" && "$phase" == "Ready" && "$desired" == "$want" && "$ready" == "$want" ]]; then
      return 0
    fi
    sleep 5
  done
  echo "tape did not converge to $want ready replicas" >&2
  capture_topology_diagnostics
  return 1
}

# Preserve the CR/StatefulSet/PVC state, pod conditions, and serving-pod logs
# before the outer run trap deletes the namespace. Run 0723080156 failed with
# neither: the old one-liner mixed resource/name and bare-resource kubectl
# forms (invalid), and no pod logs were captured, leaving the Ready=False
# root cause unprovable.
capture_topology_diagnostics() {
  kubectl -n tape get tape/tape statefulset/tape -o yaml \
    > "$EVIDENCE_DIR/kubernetes/tape-topology-failure.yaml" 2>&1 || true
  kubectl -n tape get pvc -o yaml \
    >> "$EVIDENCE_DIR/kubernetes/tape-topology-failure.yaml" 2>&1 || true
  kubectl -n tape describe pods \
    > "$EVIDENCE_DIR/kubernetes/tape-pods-describe.txt" 2>&1 || true
  local pod
  for pod in $(kubectl -n tape get pods -o name 2>/dev/null); do
    kubectl -n tape logs "$pod" --all-containers --tail=200 --prefix \
      >> "$EVIDENCE_DIR/kubernetes/tape-pods.log" 2>&1 || true
    kubectl -n tape logs "$pod" --all-containers --tail=200 --prefix --previous \
      >> "$EVIDENCE_DIR/kubernetes/tape-pods-previous.log" 2>&1 || true
  done
}

# ---- per-pod raft status forwards (topology + failover, step D only) ----
pod_forward_pids=()

pod_port() {
  echo $((18130 + "$1"))
}

stop_pod_forwards() {
  local pid
  for pid in "${pod_forward_pids[@]:-}"; do
    kill "$pid" >/dev/null 2>&1 || true
    wait "$pid" >/dev/null 2>&1 || true
  done
  pod_forward_pids=()
}

start_pod_forwards() {
  stop_pod_forwards
  local ordinal port deadline
  for ordinal in 0 1 2; do
    port="$(pod_port "$ordinal")"
    kubectl -n tape port-forward "pod/tape-${ordinal}" "${port}:7137" \
      >>"$EVIDENCE_DIR/kubernetes/tape-raftz-tape-${ordinal}-port-forward.log" 2>&1 &
    pod_forward_pids+=("$!")
  done
  for ordinal in 0 1 2; do
    port="$(pod_port "$ordinal")"
    deadline=$((SECONDS + 60))
    while (( SECONDS < deadline )); do
      curl --max-time 3 --silent --show-error --fail \
        "http://127.0.0.1:${port}/raftz" >/dev/null 2>&1 && break
      sleep 2
    done
  done
}

raftz_ordinal() {
  curl --max-time 3 --silent --show-error --fail \
    "http://127.0.0.1:$(pod_port "$1")/raftz"
}

find_leader() {
  local ordinal
  for ordinal in 0 1 2; do
    if raftz_ordinal "$ordinal" 2>/dev/null | jq -e '.is_leader == true' >/dev/null 2>&1; then
      printf '%s\n' "$ordinal"
      return 0
    fi
  done
  return 1
}

wait_for_leader() {
  local exclude="${1:-}"
  local deadline=$((SECONDS + 120))
  local leader
  while (( SECONDS < deadline )); do
    if leader="$(find_leader)" && [[ -n "$leader" && "$leader" != "$exclude" ]]; then
      printf '%s\n' "$leader"
      return 0
    fi
    sleep 3
  done
  echo "timed out waiting for a raft leader distinct from ordinal '$exclude'" >&2
  return 1
}

# ---- Step A: domain lifecycle at 1x1 (append/replay + subscription pull/ack) ----
start_forward
: > "$EVIDENCE_DIR/kubernetes/tape-append.jsonl"
for n in 1 2 3; do
  append_event "{\"n\":${n}}" | tee -a "$EVIDENCE_DIR/kubernetes/tape-append.jsonl" \
    | jq -e --argjson want "$((n - 1))" '.offset == $want' >/dev/null
done

replay_events > "$EVIDENCE_DIR/kubernetes/tape-replay-initial.json"
jq -e '(.events | length) >= 3' "$EVIDENCE_DIR/kubernetes/tape-replay-initial.json" >/dev/null

curl --silent --show-error --fail-with-body -X POST \
  "http://127.0.0.1:17137/topics/${topic}/subscriptions" \
  -H 'content-type: application/json' \
  --data "{\"name\":\"${sub}\"}" \
  > "$EVIDENCE_DIR/kubernetes/tape-subscription-create.json"
jq -e --arg topic "$topic" --arg name "$sub" \
  '.topic == $topic and .name == $name' \
  "$EVIDENCE_DIR/kubernetes/tape-subscription-create.json" >/dev/null

curl --silent --show-error --fail-with-body -X POST \
  "http://127.0.0.1:17137/topics/${topic}/subscriptions/${sub}/pull" \
  > "$EVIDENCE_DIR/kubernetes/tape-pull-before-ack.json"
jq -e '.cursor == 0 and .next_offset == 3 and (.events | length) == 3' \
  "$EVIDENCE_DIR/kubernetes/tape-pull-before-ack.json" >/dev/null

curl --silent --show-error --fail-with-body -X POST \
  "http://127.0.0.1:17137/topics/${topic}/subscriptions/${sub}/ack" \
  -H 'content-type: application/json' \
  --data '{"offset":3}' \
  > "$EVIDENCE_DIR/kubernetes/tape-ack.json"
jq -e --arg topic "$topic" --arg consumer "$sub" \
  '.topic == $topic and .consumer == $consumer and .offset == 3' \
  "$EVIDENCE_DIR/kubernetes/tape-ack.json" >/dev/null

# Acked events are never redelivered: pull_subscription's cursor is the
# checkpoint offset (apps/tape/src/lib.rs), not a lease/redelivery lock.
curl --silent --show-error --fail-with-body -X POST \
  "http://127.0.0.1:17137/topics/${topic}/subscriptions/${sub}/pull" \
  > "$EVIDENCE_DIR/kubernetes/tape-pull-after-ack.json"
jq -e '.cursor == 3 and .next_offset == 3 and (.events | length) == 0' \
  "$EVIDENCE_DIR/kubernetes/tape-pull-after-ack.json" >/dev/null

# ---- Step B: pod-restart data retention ----
stop_forward
kubectl -n tape delete pod/tape-0 --wait=true --timeout=120s
kubectl -n tape rollout status statefulset/tape --timeout=600s
kubectl -n tape wait --for=condition=Ready pod/tape-0 --timeout=300s
start_forward

replay_events > "$EVIDENCE_DIR/kubernetes/tape-replay-after-restart.json"
jq -e '(.events | length) >= 3' "$EVIDENCE_DIR/kubernetes/tape-replay-after-restart.json" >/dev/null
get_checkpoint > "$EVIDENCE_DIR/kubernetes/tape-checkpoint-after-restart.json"
jq -e '.checkpoint.offset == 3' "$EVIDENCE_DIR/kubernetes/tape-checkpoint-after-restart.json" >/dev/null

# ---- Step C: GCS backup via CronJob trigger + readback ----
test "$(kubectl -n tape get cronjob/tape-backup -o jsonpath='{.spec.jobTemplate.spec.template.spec.serviceAccountName}')" = "tape-backup"
tape_job="tape-backup-${RUN_ID}"
kubectl -n tape create job --from=cronjob/tape-backup "$tape_job"
kubectl -n tape wait --for=condition=Complete "job/$tape_job" --timeout=600s
kubectl -n tape logs "job/$tape_job" > "$EVIDENCE_DIR/kubernetes/tape-backup.log"
first_object="$(wait_for_gcs_object)"
object_size="$(gcloud storage objects describe "$first_object" --format='value(size)')"
[[ "$object_size" =~ ^[0-9]+$ && "$object_size" -gt 0 ]]
printf '%s\n' "$object_size" > "$EVIDENCE_DIR/gcs/tape-first-object-bytes.txt"
gcloud storage cat "$first_object" > "$EVIDENCE_DIR/gcs/tape-first-object.json"
# Stronger than a generic `type == "object"` probe: the backup object is a
# `JournalSnapshot` (apps/tape/src/raft.rs) whose `journal.topics` map
# (apps/tape/src/lib.rs `TapeJournal`) must actually carry our 3 events.
jq -e --arg topic "$topic" \
  '(.up_to | type == "number") and ((.journal.topics[$topic] // []) | length) >= 3' \
  "$EVIDENCE_DIR/gcs/tape-first-object.json" >/dev/null

# ---- Step D: cold restore + 3-replica topology stand-up, including failover ----
# `prepare_bootstrap_seed` (apps/tape/src/raft.rs) hard-fails on data
# directories carrying raft state, so this must be a genuine cold restore.
# Patch the LIVE CR first (one atomic desired-state change: 3 replicas, 3
# voters, seed URI), then delete the StatefulSet and PVCs. The operator's
# server-side-apply drift repair — proven earlier by verify-operator-cell.sh —
# recreates the StatefulSet from the already-patched CR, so every replica
# starts on a fresh PVC and independently consumes the same GCS seed. Never
# apply-then-patch here: a pod racing up from the pre-patch spec could write
# journal state onto a fresh PVC and poison the seed's empty-dir requirement.
stop_forward
# The seed object is fetched by the tape SERVER itself (before Raft catch-up)
# under the operator-rendered `tape` ServiceAccount, so that KSA needs the
# same Workload Identity impersonation the backup KSA gets from Terraform
# (`tape/tape` member in environment/storage.tf). The operator's
# server-side-apply field manager does not own this externally added
# annotation, so drift repair preserves it.
kubectl -n tape annotate serviceaccount/tape \
  "iam.gke.io/gcp-service-account=${BACKUP_GSA_EMAIL}" --overwrite
kubectl -n tape patch tape/tape --type=merge --patch \
  "$(jq -n --arg seed "$first_object" '{spec:{replicasPerShard:3,voterCount:3,bootstrapSeedUri:$seed}}')"
# Mark the PVCs terminating FIRST: pvc-protection keeps them alive while the
# old pods run, and the scheduler refuses to bind any post-patch pod to a
# terminating claim — so no replacement pod can ever adopt an old journal.
kubectl -n tape delete pvc -l app.kubernetes.io/instance=tape \
  --wait=false --ignore-not-found
kubectl -n tape delete statefulset/tape --wait=true --timeout=180s --cascade=foreground
pvc_deadline=$((SECONDS + 300))
while kubectl -n tape get pvc -l app.kubernetes.io/instance=tape --no-headers 2>/dev/null | rg -q .; do
  if (( SECONDS >= pvc_deadline )); then
    echo "old tape PVCs did not finish deleting before the cold-restore rebuild" >&2
    kubectl -n tape get pvc -o yaml >&2 || true
    exit 1
  fi
  sleep 3
done
wait_for_topology 3
kubectl -n tape get tape/tape -o json > "$EVIDENCE_DIR/kubernetes/tape-after-restore.json"

start_forward
replay_events > "$EVIDENCE_DIR/kubernetes/tape-replay-after-restore.json"
# No re-append happens in this step: 3 events at offsets 0-2 proves the fresh
# 3-node cluster's data came from the GCS seed, not from a live source.
jq -e '
  (.events | length) == 3
  and ([.events[].offset] | sort) == [0,1,2]
' "$EVIDENCE_DIR/kubernetes/tape-replay-after-restore.json" >/dev/null
get_checkpoint > "$EVIDENCE_DIR/kubernetes/tape-checkpoint-after-restore.json"
jq -e '.checkpoint.offset == 3' "$EVIDENCE_DIR/kubernetes/tape-checkpoint-after-restore.json" >/dev/null
stop_forward

# Failover proof, adapted from apps/relay/scripts/kind-failover-smoke.sh:
# per-pod /raftz polling to find the leader, kill it, confirm re-election to
# a distinct node, then confirm a post-failover write commits and replays.
start_pod_forwards
initial_leader="$(wait_for_leader)"
printf '%s\n' "$initial_leader" > "$EVIDENCE_DIR/kubernetes/tape-raft-leader-initial.txt"
raftz_ordinal "$initial_leader" > "$EVIDENCE_DIR/kubernetes/tape-raftz-initial.json"

kubectl -n tape delete "pod/tape-${initial_leader}" --grace-period=1 --wait=true --timeout=120s
kubectl -n tape wait --for=condition=Ready "pod/tape-${initial_leader}" --timeout=180s
start_pod_forwards
new_leader="$(wait_for_leader "$initial_leader")"
printf '%s\n' "$new_leader" > "$EVIDENCE_DIR/kubernetes/tape-raft-leader-after-failover.txt"
raftz_ordinal "$new_leader" > "$EVIDENCE_DIR/kubernetes/tape-raftz-after-failover.json"
stop_pod_forwards

start_forward
append_event '{"marker":"post-failover"}' \
  | tee "$EVIDENCE_DIR/kubernetes/tape-append-after-failover.json" \
  | jq -e '.offset == 3' >/dev/null
replay_events > "$EVIDENCE_DIR/kubernetes/tape-replay-after-failover.json"
jq -e '(.events | length) == 4' "$EVIDENCE_DIR/kubernetes/tape-replay-after-failover.json" >/dev/null
stop_forward

kubectl -n tape get tape/tape -o json > "$EVIDENCE_DIR/kubernetes/tape-final.json"
kubectl get deployment,statefulset,cronjob,job,pod,pvc,serviceaccount -A -o json \
  > "$EVIDENCE_DIR/kubernetes/workloads-after-tape-phase.json"

jq -n \
  --arg schema "axiom.gcp.tape.acceptance.v1" \
  --arg object "$first_object" \
  --argjson bytes "$object_size" \
  --arg leader_before "$initial_leader" \
  --arg leader_after "$new_leader" \
  '{schema:$schema, operator_reconcile_1x1:"passed", append_replay_lifecycle:"passed", subscription_pull_ack_cursor:"passed", pod_restart_data_retention:"passed", gcs_backup:"passed", gcs_object:$object, gcs_object_bytes:$bytes, cold_restore_from_backup:"passed", topology_1_to_3:{from:1,to:3,ready_pods:3}, raft_failover:{leader_before:$leader_before,leader_after:$leader_after,distinct:true}, post_failover_write_committed:"passed"}' \
  > "$EVIDENCE_DIR/tape-acceptance.json"

jq -n \
  --arg schema "axiom.gcp.operator.acceptance.v1" \
  --arg project_id "$PROJECT_ID" \
  --arg region "$REGION" \
  --arg run_id "$RUN_ID" \
  --arg bucket "$BACKUP_BUCKET" \
  --slurpfile tape "$EVIDENCE_DIR/tape-acceptance.json" \
  '{schema:$schema, project_id:$project_id, region:$region, run_id:$run_id, backup_bucket:$bucket, acceptance:{tape:$tape[0]}}' \
  > "$EVIDENCE_DIR/acceptance.json"
