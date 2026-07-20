#!/usr/bin/env bash
# HANDWRITE-BEGIN gap="missing-generator:e2e-test:defer-kind" tracker="#766" reason="Disposable operator-mode Kind proof for Defer queue/task lifecycle durability across PVC-backed pod replacement."
# Defer's bounded operator-mode Kind dogfood gate.
#
# Exercises the real source image, CRD/operator reconciliation, PVC-backed
# queue/task state, batch creation, pod replacement, queue control, and a
# terminal task transition. The task targets are deliberately scheduled far in
# the future so this recovery gate does not depend on an external HTTP service.
#
# Usage:
#   bash apps/defer/scripts/kind-e2e.sh
#   DEFER_KEEP_CLUSTER=1 bash apps/defer/scripts/kind-e2e.sh
#
# Requirements: docker, kind, kubectl, curl, jq.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
DEFER_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
REPO_ROOT="$(cd "$DEFER_DIR/../.." && pwd)"

CLUSTER_NAME="${DEFER_KIND_CLUSTER:-defer-e2e}"
NAMESPACE="${DEFER_KIND_NAMESPACE:-defer}"
OPERATOR_NAMESPACE="defer-system"
DEFER_NAME="defer"
IMAGE_TAG="${DEFER_E2E_IMAGE:-defer:kind}"
HOST_PORT="${DEFER_E2E_HOST_PORT:-17141}"
NODE_PORT="${DEFER_E2E_NODE_PORT:-30714}"
CLUSTER_CREATED=0

step() {
  local label="$1"
  shift
  echo ">> $label"
  "$@"
}

require() {
  command -v "$1" >/dev/null 2>&1 || {
    echo "!! required command not found: $1" >&2
    exit 1
  }
}

dump_diagnostics() {
  [[ "$CLUSTER_CREATED" == "1" ]] || return 0
  echo "!! Defer Kind diagnostics" >&2
  if kubectl get namespace "$NAMESPACE" >/dev/null 2>&1; then
    kubectl -n "$NAMESPACE" get all,pvc,defers.defer.dev 2>&1 || true
    kubectl -n "$NAMESPACE" describe statefulset "$DEFER_NAME" 2>&1 || true
    kubectl -n "$NAMESPACE" logs statefulset/"$DEFER_NAME" --tail=120 2>&1 || true
    kubectl -n "$NAMESPACE" get events --sort-by=.lastTimestamp 2>&1 || true
  fi
  if kubectl get namespace "$OPERATOR_NAMESPACE" >/dev/null 2>&1; then
    kubectl -n "$OPERATOR_NAMESPACE" logs deploy/defer-operator --tail=120 2>&1 || true
  fi
}

cleanup() {
  local ec=$?
  local clusters
  trap - EXIT INT TERM
  if [[ "$ec" -ne 0 ]]; then
    dump_diagnostics
  fi
  # <HANDWRITE gap="missing-generator:e2e-test:defer-kind-cleanup-contract" tracker="#2214" reason="Require successful disposable-cluster deletion and explicit absence verification on a successful Defer recovery journey.">
  if [[ "$CLUSTER_CREATED" == "1" && "${DEFER_KEEP_CLUSTER:-0}" != "1" ]]; then
    if ! kind delete cluster --name "$CLUSTER_NAME"; then
      echo "!! failed to delete Kind cluster $CLUSTER_NAME" >&2
      (( ec != 0 )) || ec=1
    elif ! clusters="$(kind get clusters)"; then
      echo "!! could not verify deletion of Kind cluster $CLUSTER_NAME" >&2
      (( ec != 0 )) || ec=1
    elif grep -Fxq "$CLUSTER_NAME" <<<"$clusters"; then
      echo "!! Kind cluster $CLUSTER_NAME still exists after deletion" >&2
      (( ec != 0 )) || ec=1
    fi
  elif [[ "$CLUSTER_CREATED" == "1" ]]; then
    echo ">> preserving Kind cluster $CLUSTER_NAME (DEFER_KEEP_CLUSTER=1)"
  fi
  # </HANDWRITE>
  exit "$ec"
}
trap cleanup EXIT
trap 'exit 130' INT TERM

wait_for_statefulset() {
  local deadline=$(( $(date +%s) + 90 ))
  while [[ $(date +%s) -lt "$deadline" ]]; do
    if kubectl -n "$NAMESPACE" get statefulset "$DEFER_NAME" >/dev/null 2>&1; then
      return 0
    fi
    sleep 2
  done
  echo "!! operator did not create StatefulSet/$DEFER_NAME within 90s" >&2
  return 1
}

wait_for_ready_pod() {
  local previous_uid="${1:-}"
  local deadline=$(( $(date +%s) + 240 ))
  while [[ $(date +%s) -lt "$deadline" ]]; do
    local uid
    uid="$(kubectl -n "$NAMESPACE" get pod "${DEFER_NAME}-0" -o jsonpath='{.metadata.uid}' 2>/dev/null || true)"
    if [[ -n "$uid" && "$uid" != "$previous_uid" ]] && \
      kubectl -n "$NAMESPACE" wait --for=condition=Ready pod/"${DEFER_NAME}-0" --timeout=10s >/dev/null 2>&1; then
      printf '%s\n' "$uid"
      return 0
    fi
    sleep 2
  done
  echo "!! Defer pod did not become Ready within 240s" >&2
  return 1
}

wait_for_api() {
  local deadline=$(( $(date +%s) + 120 ))
  while [[ $(date +%s) -lt "$deadline" ]]; do
    if curl -fsS --max-time 5 "http://127.0.0.1:${HOST_PORT}/healthz" >/dev/null 2>&1; then
      return 0
    fi
    sleep 2
  done
  echo "!! Defer API never became reachable on host port $HOST_PORT" >&2
  return 1
}

put_queue() {
  curl -fsS --max-time 20 -X PUT "http://127.0.0.1:${HOST_PORT}/v1/queues/jobs" \
    -H 'content-type: application/json' \
    -d '{"max_in_flight":100,"max_dispatch_per_tick":100,"max_dispatches_per_second":100,"max_burst_size":100,"lease_ttl_ms":30000,"retry_backoff_ms":1000}' |
    jq -e '.queue == "jobs" and .control_state == "Running" and .task_count == 0' >/dev/null
}

create_task_batch() {
  curl -fsS --max-time 20 -X POST "http://127.0.0.1:${HOST_PORT}/v1/queues/jobs/tasks:batch" \
    -H 'content-type: application/json' \
    -d '{"tasks":[{"task_id":"before-restart-a","target":{"url":"http://example.invalid/a","method":"POST","headers":{}},"payload":{"kind":"a"},"schedule_at":"2099-01-01T00:00:00Z","priority":10,"max_attempts":3},{"task_id":"before-restart-b","target":{"url":"http://example.invalid/b","method":"POST","headers":{}},"payload":{"kind":"b"},"schedule_at":"2099-01-01T00:00:00Z","priority":20,"max_attempts":3}]}' |
    jq -e '.created == 2' >/dev/null
}

assert_queue() {
  local expected_state="$1"
  local expected_terminal="$2"
  curl -fsS --max-time 20 "http://127.0.0.1:${HOST_PORT}/v1/queues/jobs" |
    jq -e --arg state "$expected_state" --argjson terminal "$expected_terminal" \
      '.control_state == $state and .task_count == 2 and .terminal_count == $terminal' >/dev/null
}

assert_task_status() {
  local task_id="$1"
  local expected="$2"
  curl -fsS --max-time 20 "http://127.0.0.1:${HOST_PORT}/v1/queues/jobs/tasks/${task_id}" |
    jq -e --arg task_id "$task_id" --arg expected "$expected" \
      '.task_id == $task_id and .status == $expected' >/dev/null
}

pause_queue() {
  curl -fsS --max-time 20 -X POST "http://127.0.0.1:${HOST_PORT}/v1/queues/jobs/control" \
    -H 'content-type: application/json' -d '{"state":"Paused"}' |
    jq -e '.control_state == "Paused"' >/dev/null
}

cancel_task() {
  local code
  code="$(curl -sS --max-time 20 -o /dev/null -w '%{http_code}' -X DELETE \
    "http://127.0.0.1:${HOST_PORT}/v1/queues/jobs/tasks/before-restart-a")"
  [[ "$code" == "204" ]]
}

create_cluster() {
  kind create cluster --name "$CLUSTER_NAME" --wait 120s --config - <<EOF
kind: Cluster
apiVersion: kind.x-k8s.io/v1alpha4
nodes:
  - role: control-plane
    extraPortMappings:
      - containerPort: ${NODE_PORT}
        hostPort: ${HOST_PORT}
        protocol: TCP
EOF
  CLUSTER_CREATED=1
}

build_and_load_image() {
  docker build -f "$DEFER_DIR/Dockerfile" -t "$IMAGE_TAG" "$REPO_ROOT"
  kind load docker-image "$IMAGE_TAG" --name "$CLUSTER_NAME"
}

install_operator() {
  kubectl apply -f "$DEFER_DIR/k8s/operator/crd.yaml"
  kubectl apply -f "$DEFER_DIR/k8s/operator/rbac.yaml"
  kubectl apply -f "$DEFER_DIR/k8s/operator/deployment.yaml"
  kubectl wait --for=condition=established crd/defers.defer.dev --timeout=60s
  kubectl -n "$OPERATOR_NAMESPACE" set image deployment/defer-operator operator="$IMAGE_TAG"
  kubectl -n "$OPERATOR_NAMESPACE" rollout status deployment/defer-operator --timeout=180s
}

apply_defer_instance() {
  kubectl create namespace "$NAMESPACE" --dry-run=client -o yaml | kubectl apply -f -
  kubectl apply -f - <<EOF
apiVersion: defer.dev/v1alpha1
kind: Defer
metadata:
  name: ${DEFER_NAME}
  namespace: ${NAMESPACE}
spec:
  image: ${IMAGE_TAG}
  imagePullPolicy: IfNotPresent
  shardCount: 1
  replicasPerShard: 1
  voterCount: 1
  storage: 1Gi
  graceSecs: 1
  logLevel: info
EOF
  wait_for_statefulset
  kubectl -n "$NAMESPACE" rollout status statefulset/"$DEFER_NAME" --timeout=240s
  # <HANDWRITE gap="missing-generator:e2e-test:defer-kind-pvc-contract" tracker="#2214" reason="Observe a Bound PVC with the exact requested and provisioned 1Gi capacity before the recovery journey mutates state.">
  kubectl -n "$NAMESPACE" wait --for=jsonpath='{.status.phase}'=Bound \
    pvc/data-"${DEFER_NAME}"-0 --timeout=120s
  kubectl -n "$NAMESPACE" get pvc data-"${DEFER_NAME}"-0 -o json |
    jq -e '
      .status.phase == "Bound"
      and .spec.resources.requests.storage == "1Gi"
      and .status.capacity.storage == "1Gi"
    ' >/dev/null
  # </HANDWRITE>
}

expose_api() {
  kubectl -n "$NAMESPACE" apply -f - <<EOF
apiVersion: v1
kind: Service
metadata:
  name: defer-kind
  namespace: ${NAMESPACE}
spec:
  type: NodePort
  selector:
    app.kubernetes.io/name: defer
    app.kubernetes.io/instance: ${DEFER_NAME}
    app.kubernetes.io/component: server
  ports:
    - name: http
      port: 7141
      targetPort: http
      protocol: TCP
      nodePort: ${NODE_PORT}
EOF
  wait_for_api
}

for cmd in docker kind kubectl curl jq; do
  require "$cmd"
done

step "create Kind cluster $CLUSTER_NAME (host :$HOST_PORT -> node :$NODE_PORT)" create_cluster
step "build Defer operator image and load it into Kind" build_and_load_image
step "install Defer CRD and operator" install_operator
step "apply single-node Defer CR and wait for its PVC" apply_defer_instance

INITIAL_UID="$(wait_for_ready_pod)"
step "expose operator-owned Defer pod on NodePort :$NODE_PORT" expose_api
step "configure durable queue" put_queue
step "create two scheduled tasks in one committed batch" create_task_batch
step "verify scheduled lifecycle before replacement" assert_task_status "before-restart-a" "Scheduled"
step "verify queue inventory before replacement" assert_queue "Running" 0

step "delete serving pod while retaining its PVC" \
  kubectl -n "$NAMESPACE" delete pod "${DEFER_NAME}-0" --wait=false
REPLACEMENT_UID="$(wait_for_ready_pod "$INITIAL_UID")"
if [[ "$REPLACEMENT_UID" == "$INITIAL_UID" ]]; then
  echo "!! StatefulSet did not replace the serving pod" >&2
  exit 1
fi
step "confirm API after replacement" wait_for_api
step "verify both scheduled tasks recovered from durable Raft state" assert_queue "Running" 0
step "verify task identity and state after replacement" assert_task_status "before-restart-b" "Scheduled"
step "commit queue pause after recovery" pause_queue
step "commit task cancellation after recovery" cancel_task
step "verify terminal cancellation and queue accounting" assert_task_status "before-restart-a" "Canceled"
step "verify paused queue retains both task records" assert_queue "Paused" 1

echo ">> Defer operator Kind lifecycle recovery dogfood PASS"
# HANDWRITE-END
