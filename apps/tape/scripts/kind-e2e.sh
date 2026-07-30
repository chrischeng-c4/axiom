#!/usr/bin/env bash
# HANDWRITE-BEGIN gap="missing-generator:e2e-test:2004fbb4" tracker="#1590" reason="Build the checked-in Tape image, create a disposable Kind cluster, install the real CRD/operator, exercise append/replay across one pod replacement, and clean up by default. generator gap: missing-generator:service-kind-dogfood."
# Tape's bounded operator-mode Kind dogfood gate (#1590).
#
# Exercises one Tape-owned operational path only:
#   source image -> Kind -> CRD/operator -> Tape CR -> HTTP append ->
#   serving-pod replacement with its PVC retained -> HTTP replay + new append.
#
# Usage:
#   bash apps/tape/scripts/kind-e2e.sh
#   TAPE_KEEP_CLUSTER=1 bash apps/tape/scripts/kind-e2e.sh
#
# Requirements: docker, kind, kubectl, curl, jq.
# The default cleanup deletes only the named Kind cluster. Set
# TAPE_KEEP_CLUSTER=1 to preserve it for diagnostics.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
TAPE_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
REPO_ROOT="$(cd "$TAPE_DIR/../.." && pwd)"

CLUSTER_NAME="${TAPE_KIND_CLUSTER:-tape-e2e}"
NAMESPACE="${TAPE_KIND_NAMESPACE:-tape}"
OPERATOR_NAMESPACE="tape-system"
TAPE_NAME="tape"
IMAGE_TAG="${TAPE_E2E_IMAGE:-tape:kind}"
HOST_PORT="${TAPE_E2E_HOST_PORT:-17137}"
NODE_PORT="${TAPE_E2E_NODE_PORT:-30713}"
SERVER_LABEL="app.kubernetes.io/name=tape,app.kubernetes.io/instance=${TAPE_NAME},app.kubernetes.io/component=server"
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
  echo "!! Tape Kind diagnostics" >&2
  if kubectl get namespace "$NAMESPACE" >/dev/null 2>&1; then
    kubectl -n "$NAMESPACE" get all,pvc,tapes.tape.dev 2>&1 || true
    kubectl -n "$NAMESPACE" describe statefulset "$TAPE_NAME" 2>&1 || true
    kubectl -n "$NAMESPACE" logs statefulset/"$TAPE_NAME" --tail=120 2>&1 || true
    kubectl -n "$NAMESPACE" get events --sort-by=.lastTimestamp 2>&1 || true
  fi
  if kubectl get namespace "$OPERATOR_NAMESPACE" >/dev/null 2>&1; then
    kubectl -n "$OPERATOR_NAMESPACE" logs deploy/tape-operator --tail=120 2>&1 || true
  fi
}

cleanup() {
  local ec=$?
  trap - EXIT INT TERM
  if [[ "$ec" -ne 0 ]]; then
    dump_diagnostics
  fi
  if [[ "$CLUSTER_CREATED" == "1" && "${TAPE_KEEP_CLUSTER:-0}" != "1" ]]; then
    kind delete cluster --name "$CLUSTER_NAME" >/dev/null 2>&1 || true
  elif [[ "$CLUSTER_CREATED" == "1" ]]; then
    echo ">> preserving Kind cluster $CLUSTER_NAME (TAPE_KEEP_CLUSTER=1)"
  fi
  exit "$ec"
}
trap cleanup EXIT
trap 'exit 130' INT TERM

wait_for_statefulset() {
  local deadline=$(( $(date +%s) + 90 ))
  while [[ $(date +%s) -lt "$deadline" ]]; do
    if kubectl -n "$NAMESPACE" get statefulset "$TAPE_NAME" >/dev/null 2>&1; then
      return 0
    fi
    sleep 2
  done
  echo "!! operator did not create StatefulSet/$TAPE_NAME within 90s" >&2
  return 1
}

wait_for_ready_pod() {
  local previous_uid="${1:-}"
  local deadline=$(( $(date +%s) + 240 ))
  while [[ $(date +%s) -lt "$deadline" ]]; do
    local uid
    uid="$(kubectl -n "$NAMESPACE" get pod "${TAPE_NAME}-0" -o jsonpath='{.metadata.uid}' 2>/dev/null || true)"
    if [[ -n "$uid" && "$uid" != "$previous_uid" ]] && \
      kubectl -n "$NAMESPACE" wait --for=condition=Ready pod/"${TAPE_NAME}-0" --timeout=10s >/dev/null 2>&1; then
      printf '%s\n' "$uid"
      return 0
    fi
    sleep 2
  done
  echo "!! Tape pod did not become Ready within 240s" >&2
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
  echo "!! Tape API never became reachable on host port $HOST_PORT" >&2
  return 1
}

append_event() {
  local id="$1"
  curl -fsS --max-time 20 -X POST "http://127.0.0.1:${HOST_PORT}/topics/orders/append" \
    -H 'content-type: application/json' \
    -d "{\"payload\":{\"id\":\"${id}\"}}" |
    jq -e --arg id "$id" '.payload.id == $id' >/dev/null
}

assert_replay() {
  local expected_ids="$1"
  curl -fsS --max-time 20 "http://127.0.0.1:${HOST_PORT}/topics/orders/replay" |
    jq -e --argjson expected "$expected_ids" '[.events[] | .payload.id] == $expected' >/dev/null
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
  docker build -f "$TAPE_DIR/Dockerfile" -t "$IMAGE_TAG" "$REPO_ROOT"
  kind load docker-image "$IMAGE_TAG" --name "$CLUSTER_NAME"
}

install_operator() {
  kubectl apply -f "$TAPE_DIR/k8s/operator/crd.yaml"
  kubectl apply -f "$TAPE_DIR/k8s/operator/rbac.yaml"
  kubectl apply -f "$TAPE_DIR/k8s/operator/deployment.yaml"
  kubectl wait --for=condition=established crd/tapes.tape.dev --timeout=60s
  kubectl -n "$OPERATOR_NAMESPACE" set image deployment/tape-operator operator="$IMAGE_TAG"
  kubectl -n "$OPERATOR_NAMESPACE" rollout status deployment/tape-operator --timeout=180s
}

apply_tape_instance() {
  kubectl create namespace "$NAMESPACE" --dry-run=client -o yaml | kubectl apply -f -
  kubectl apply -f - <<EOF
apiVersion: tape.dev/v1alpha1
kind: Tape
metadata:
  name: ${TAPE_NAME}
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
  # Stated, not omitted: since #2765 an absent spec.auth defaults to required,
  # and a CR that asks for auth without naming a token source renders a pod
  # that fails startup for want of a registry file. This leg proves PVC-retained
  # replay across a pod replacement, not authentication, so it opts out by name.
  auth: disabled
EOF
  wait_for_statefulset
  kubectl -n "$NAMESPACE" rollout status statefulset/"$TAPE_NAME" --timeout=240s
  kubectl -n "$NAMESPACE" get pvc data-"${TAPE_NAME}"-0 >/dev/null
}

expose_api() {
  kubectl -n "$NAMESPACE" apply -f - <<EOF
apiVersion: v1
kind: Service
metadata:
  name: tape-kind
  namespace: ${NAMESPACE}
spec:
  type: NodePort
  selector:
    app.kubernetes.io/name: tape
    app.kubernetes.io/instance: ${TAPE_NAME}
    app.kubernetes.io/component: server
  ports:
    - name: http
      port: 7137
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
step "build Tape operator image and load it into Kind" build_and_load_image
step "install Tape CRD and operator" install_operator
step "apply single-node Tape CR and wait for its PVC" apply_tape_instance

INITIAL_UID="$(wait_for_ready_pod)"
step "expose operator-owned Tape pod on NodePort :$NODE_PORT" expose_api
step "append durable pre-restart event" append_event "before-restart"
step "verify pre-restart replay" assert_replay '["before-restart"]'

step "delete serving pod while retaining its PVC" \
  kubectl -n "$NAMESPACE" delete pod "${TAPE_NAME}-0" --wait=false
REPLACEMENT_UID="$(wait_for_ready_pod "$INITIAL_UID")"
if [[ "$REPLACEMENT_UID" == "$INITIAL_UID" ]]; then
  echo "!! StatefulSet did not replace the serving pod" >&2
  exit 1
fi
step "confirm API after replacement" wait_for_api
step "verify durable replay after replacement" assert_replay '["before-restart"]'
step "append fresh post-restart event" append_event "after-restart"
step "verify ordered replay across replacement" assert_replay '["before-restart","after-restart"]'

echo ">> Tape operator Kind recovery dogfood PASS"
# HANDWRITE-END
