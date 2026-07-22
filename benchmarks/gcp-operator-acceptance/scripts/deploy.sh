#!/usr/bin/env bash
set -euo pipefail

: "${MANIFEST_DIR:?MANIFEST_DIR is required}"
: "${EVIDENCE_DIR:?EVIDENCE_DIR is required}"

app="${1:?usage: deploy.sh lumen|sift}"
case "$app" in
  lumen)
    crd="lumens.lumen.dev"
    cr_namespace="lumen"
    cr_resource="lumen/lumen"
    operator_namespace="lumen-system"
    operator_deployment="lumen-operator"
    statefulset="lumen"
    ;;
  sift)
    crd="sifts.sift.axiom.dev"
    cr_namespace="sift"
    cr_resource="sift/sift"
    operator_namespace="sift-system"
    operator_deployment="sift-operator"
    statefulset="sift"
    ;;
  *)
    echo "unknown app '$app'; expected lumen or sift" >&2
    exit 2
    ;;
esac

mkdir -p "$EVIDENCE_DIR/kubernetes"

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

kubectl apply -f "$MANIFEST_DIR/$app/crd.yaml"
kubectl wait --for=condition=Established "customresourcedefinition/$crd" --timeout=120s
kubectl apply -f "$MANIFEST_DIR/$app/operator.bundle.yaml"
kubectl wait -n "$operator_namespace" --for=condition=Available "deployment/$operator_deployment" --timeout=600s
kubectl apply -f "$MANIFEST_DIR/$app/instance.bundle.yaml"
wait_ready_cr
kubectl -n "$cr_namespace" rollout status "statefulset/$statefulset" --timeout=600s

kubectl get "$crd" -A -o json > "$EVIDENCE_DIR/kubernetes/${app}-crs.json"
kubectl get deployment,statefulset,cronjob,pod,pvc,serviceaccount -A -o json \
  > "$EVIDENCE_DIR/kubernetes/workloads-after-${app}-deploy.json"
