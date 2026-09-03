#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$SCRIPT_DIR/kubernetes-ownership.sh"

: "${MANIFEST_DIR:?MANIFEST_DIR is required}"
: "${EVIDENCE_DIR:?EVIDENCE_DIR is required}"

app="${1:?usage: deploy.sh lumen|sift|tape}"
case "$app" in
  lumen)
    crd="lumens.lumen.dev"
    cr_namespace="lumen"
    cr_resource="lumen/lumen"
    operator_namespace="lumen-system"
    operator_deployment="lumen-operator"
    statefulsets=(lumen)
    ;;
  sift)
    crd="sifts.sift.axiom.dev"
    cr_namespace="sift"
    cr_resource="sift/sift"
    operator_namespace="sift-system"
    operator_deployment="sift-operator"
    statefulsets=(sift-store sift-control)
    ;;
  tape)
    crd="tapes.tape.dev"
    cr_namespace="tape"
    cr_resource="tape/tape"
    operator_namespace="tape-system"
    operator_deployment="tape-operator"
    statefulsets=(tape)
    ;;
  *)
    echo "unknown app '$app'; expected lumen, sift, or tape" >&2
    exit 2
    ;;
esac

mkdir -p "$EVIDENCE_DIR/kubernetes"

if [[ "$app" == "sift" ]]; then
  : "${PROJECT_ID:?PROJECT_ID is required for Sift ownership receipts}"
  : "${RUN_ID:?RUN_ID is required for Sift ownership receipts}"
  : "${ACCEPTANCE_LOCK_ACQUISITION_ID:?ACCEPTANCE_LOCK_ACQUISITION_ID is required for Sift ownership receipts}"
  kubectl get customresourcedefinition fqdnnetworkpolicies.networking.gke.io \
    >/dev/null 2>&1 || {
      echo "Sift requires GKE Dataplane V2 with FQDN Network Policy enabled; missing fqdnnetworkpolicies.networking.gke.io" >&2
      exit 1
    }
fi

wait_ready_cr() {
  local expected_generation observed_generation phase
  local deadline=$((SECONDS + 600))
  while (( SECONDS < deadline )); do
    expected_generation="$(kubectl -n "$cr_namespace" get "$cr_resource" -o jsonpath='{.metadata.generation}' 2>/dev/null || true)"
    observed_generation="$(kubectl -n "$cr_namespace" get "$cr_resource" -o jsonpath='{.status.observedGeneration}' 2>/dev/null || true)"
    phase="$(kubectl -n "$cr_namespace" get "$cr_resource" -o jsonpath='{.status.phase}' 2>/dev/null || true)"
    if [[ -n "$expected_generation" && "$observed_generation" == "$expected_generation" && "$phase" == "Ready" ]]; then
      return 0
    fi
    sleep 5
  done
  echo "timed out waiting for $cr_namespace/$cr_resource status generation and Ready phase" >&2
  kubectl -n "$cr_namespace" get "$cr_resource" -o yaml >&2 || true
  return 1
}

wait_crd_established() {
  local name="$1"
  local deadline=$((SECONDS + 180))
  local established
  while (( SECONDS < deadline )); do
    established="$(kubectl get customresourcedefinition "$name" \
      -o jsonpath='{.status.conditions[?(@.type=="Established")].status}' 2>/dev/null || true)"
    [[ "$established" == "True" ]] && return 0
    sleep 3
  done
  echo "CRD $name was never Established" >&2
  kubectl get customresourcedefinition "$name" -o yaml >&2 || true
  return 1
}

if [[ "$app" == "sift" ]]; then
  ownership_root="$EVIDENCE_DIR/kubernetes/ownership"
  create_owned_namespace \
    "$operator_namespace" "$ownership_root" "$PROJECT_ID" "$RUN_ID" \
    "$ACCEPTANCE_LOCK_ACQUISITION_ID"
  create_owned_namespace \
    "$cr_namespace" "$ownership_root" "$PROJECT_ID" "$RUN_ID" \
    "$ACCEPTANCE_LOCK_ACQUISITION_ID"
  create_owned_kubernetes_resource \
    customresourcedefinition "$crd" "$MANIFEST_DIR/$app/crd.yaml" \
    "$ownership_root" "$PROJECT_ID" "$RUN_ID" \
    "$ACCEPTANCE_LOCK_ACQUISITION_ID"
else
  kubectl apply -f "$MANIFEST_DIR/$app/crd.yaml"
fi
# Not `kubectl wait --for=condition=Established`: between the apply returning
# and the apiextensions controller first writing status, `.status.conditions`
# does not exist, and `kubectl wait` treats an absent field as an error
# ("<nil> is of the type <nil>, expected []interface{}") rather than as a
# condition that has not been met yet. It aborts instantly instead of waiting
# out its own --timeout, so a momentarily slow control plane kills the run
# before a single instance is deployed. Poll for the condition instead.
wait_crd_established "$crd"
kubectl apply -f "$MANIFEST_DIR/$app/operator.bundle.yaml"
kubectl wait -n "$operator_namespace" --for=condition=Available "deployment/$operator_deployment" --timeout=600s
kubectl apply -f "$MANIFEST_DIR/$app/instance.bundle.yaml"
wait_ready_cr
for statefulset in "${statefulsets[@]}"; do
  kubectl -n "$cr_namespace" rollout status "statefulset/$statefulset" --timeout=600s
done

kubectl get "$crd" -A -o json > "$EVIDENCE_DIR/kubernetes/${app}-crs.json"
kubectl get deployment,statefulset,cronjob,pod,pvc,serviceaccount -A -o json \
  > "$EVIDENCE_DIR/kubernetes/workloads-after-${app}-deploy.json"
