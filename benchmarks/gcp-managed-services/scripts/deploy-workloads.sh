#!/usr/bin/env bash
set -euo pipefail

: "${REPO_ROOT:?REPO_ROOT is required}"
: "${TAPE_IMAGE:?TAPE_IMAGE is required}"
: "${DEFER_IMAGE:?DEFER_IMAGE is required}"
: "${RELAY_IMAGE:?RELAY_IMAGE is required}"
: "${TAPE_CLI:?TAPE_CLI is required}"
: "${DEFER_CLI:?DEFER_CLI is required}"
: "${RELAY_CLI:?RELAY_CLI is required}"
: "${WORKLOAD_MANIFEST_DIR:?WORKLOAD_MANIFEST_DIR is required}"

BENCH_ROOT="$REPO_ROOT/benchmarks/gcp-managed-services"
mkdir -p \
  "$WORKLOAD_MANIFEST_DIR/tape" \
  "$WORKLOAD_MANIFEST_DIR/defer" \
  "$WORKLOAD_MANIFEST_DIR/relay"

render_instance_overlay() {
  local service="$1"
  local cli="$2"
  local image="$3"
  local render_dir="$WORKLOAD_MANIFEST_DIR/$service"

  mkdir -p "$render_dir"
  "$cli" k8s instance render \
    --profile dev \
    --name "$service" \
    --namespace "$service" \
    --image "$image" \
    --out "$render_dir/instance.yaml"
  cp "$BENCH_ROOT/k8s/instance-overlay/kustomization.yaml.in" \
    "$render_dir/kustomization.yaml"
  cp "$BENCH_ROOT/k8s/instance-overlay/${service}-benchmark-patch.yaml" \
    "$render_dir/benchmark-patch.yaml"
  kubectl apply -k "$render_dir"
}

"$TAPE_CLI" k8s crd render --out "$WORKLOAD_MANIFEST_DIR/tape/crd.yaml"
"$TAPE_CLI" k8s operator render --namespace tape-system \
  --out "$WORKLOAD_MANIFEST_DIR/tape/operator.yaml"
kubectl apply -f "$WORKLOAD_MANIFEST_DIR/tape/crd.yaml"
kubectl apply -f "$WORKLOAD_MANIFEST_DIR/tape/operator.yaml"
kubectl wait --for=condition=established crd/tapes.tape.dev --timeout=90s
kubectl -n tape-system set image deployment/tape-operator operator="$TAPE_IMAGE"

"$DEFER_CLI" k8s crd render --out "$WORKLOAD_MANIFEST_DIR/defer/crd.yaml"
"$DEFER_CLI" k8s operator render --namespace defer-system \
  --out "$WORKLOAD_MANIFEST_DIR/defer/operator.yaml"
kubectl apply -f "$WORKLOAD_MANIFEST_DIR/defer/crd.yaml"
kubectl apply -f "$WORKLOAD_MANIFEST_DIR/defer/operator.yaml"
kubectl wait --for=condition=established crd/defers.defer.dev --timeout=90s
kubectl -n defer-system set image deployment/defer-operator operator="$DEFER_IMAGE"

"$RELAY_CLI" k8s crd render --out "$WORKLOAD_MANIFEST_DIR/relay/crd.yaml"
"$RELAY_CLI" k8s operator render --namespace relay-system \
  --out "$WORKLOAD_MANIFEST_DIR/relay/operator.yaml"
kubectl apply -f "$WORKLOAD_MANIFEST_DIR/relay/crd.yaml"
kubectl apply -f "$WORKLOAD_MANIFEST_DIR/relay/operator.yaml"
kubectl wait --for=condition=established crd/relays.relay.dev --timeout=90s
kubectl -n relay-system set image deployment/relay-operator operator="$RELAY_IMAGE"

kubectl -n tape-system rollout status deployment/tape-operator --timeout=300s
kubectl -n defer-system rollout status deployment/defer-operator --timeout=300s
kubectl -n relay-system rollout status deployment/relay-operator --timeout=300s

kubectl apply -f "$REPO_ROOT/benchmarks/gcp-managed-services/k8s/storage-class.yaml"

kubectl create namespace tape --dry-run=client -o yaml | kubectl apply -f -
kubectl create namespace defer --dry-run=client -o yaml | kubectl apply -f -
kubectl create namespace relay --dry-run=client -o yaml | kubectl apply -f -

render_instance_overlay tape "$TAPE_CLI" "$TAPE_IMAGE"
render_instance_overlay defer "$DEFER_CLI" "$DEFER_IMAGE"
render_instance_overlay relay "$RELAY_CLI" "$RELAY_IMAGE"

deadline=$(( $(date +%s) + 900 ))
while [[ $(date +%s) -lt "$deadline" ]]; do
  tape_ready="$(kubectl -n tape get statefulset tape -o jsonpath='{.status.readyReplicas}' 2>/dev/null || true)"
  defer_ready="$(kubectl -n defer get statefulset defer -o jsonpath='{.status.readyReplicas}' 2>/dev/null || true)"
  relay_ready="$(kubectl -n relay get statefulset relay -o jsonpath='{.status.readyReplicas}' 2>/dev/null || true)"
  if [[ "$tape_ready" == "3" && "$defer_ready" == "3" && "$relay_ready" == "3" ]]; then
    kubectl -n tape get pods,pvc
    kubectl -n defer get pods,pvc
    kubectl -n relay get pods,pvc
    exit 0
  fi
  sleep 5
done

kubectl -n tape get all,pvc,tapes.tape.dev || true
kubectl -n defer get all,pvc,defers.defer.dev || true
kubectl -n relay get all,pvc,relays.relay.dev || true
kubectl -n tape-system logs deployment/tape-operator --tail=100 || true
kubectl -n defer-system logs deployment/defer-operator --tail=100 || true
kubectl -n relay-system logs deployment/relay-operator --tail=100 || true
echo "Tape, Defer, and Relay did not all reach three ready replicas within 15 minutes" >&2
exit 1
