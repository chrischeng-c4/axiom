#!/usr/bin/env bash
set -euo pipefail

: "${EVIDENCE_DIR:?EVIDENCE_DIR is required}"
app="${1:?usage: verify-operator-cell.sh lumen|sift|tape}"

case "$app" in
  lumen)
    operator_namespace="lumen-system"
    service_account="lumen-operator"
    lease="lumen-operator"
    app_namespace="lumen"
    statefulset="lumen"
    ;;
  sift)
    operator_namespace="sift-system"
    service_account="sift-operator"
    lease="sift-operator"
    app_namespace="sift"
    statefulset="sift"
    ;;
  tape)
    operator_namespace="tape-system"
    service_account="tape-operator"
    lease="tape-operator"
    app_namespace="tape"
    statefulset="tape"
    ;;
  *)
    echo "unknown app '$app'; expected lumen, sift, or tape" >&2
    exit 2
    ;;
esac

mkdir -p "$EVIDENCE_DIR/kubernetes"
rbac_evidence="$EVIDENCE_DIR/kubernetes/${app}-operator-rbac.tsv"
: > "$rbac_evidence"

assert_can_i() {
  local verb="$1"
  local resource="$2"
  local target_namespace="$3"
  local result
  result="$(kubectl auth can-i "$verb" "$resource" \
    --namespace="$target_namespace" \
    --as="system:serviceaccount:${operator_namespace}:${service_account}")"
  printf '%s\t%s\t%s\t%s\n' \
    "${operator_namespace}/${service_account}" "$verb" "$resource" "$result" \
    >> "$rbac_evidence"
  [[ "$result" == "yes" ]] || {
    echo "$operator_namespace/$service_account cannot $verb $resource in $target_namespace" >&2
    return 1
  }
}

wait_holder() {
  local previous="${1:-}"
  local holder
  local deadline=$((SECONDS + 180))
  while (( SECONDS < deadline )); do
    holder="$(kubectl -n "$operator_namespace" get "lease/$lease" \
      -o jsonpath='{.spec.holderIdentity}' 2>/dev/null || true)"
    if [[ -n "$holder" && "$holder" != "$previous" ]]; then
      printf '%s\n' "$holder"
      return 0
    fi
    sleep 3
  done
  echo "timed out waiting for $operator_namespace/lease/$lease holder to differ from '$previous'" >&2
  return 1
}

wait_live_holder() {
  local holder
  local deadline=$((SECONDS + 180))
  while (( SECONDS < deadline )); do
    holder="$(kubectl -n "$operator_namespace" get "lease/$lease" \
      -o jsonpath='{.spec.holderIdentity}' 2>/dev/null || true)"
    if [[ -n "$holder" ]] && kubectl -n "$operator_namespace" get "pod/$holder" >/dev/null 2>&1; then
      printf '%s\n' "$holder"
      return 0
    fi
    sleep 3
  done
  echo "timed out waiting for $operator_namespace/lease/$lease to name a live pod" >&2
  return 1
}

wait_statefulset_one() {
  local desired ready
  local deadline=$((SECONDS + 240))
  while (( SECONDS < deadline )); do
    desired="$(kubectl -n "$app_namespace" get "statefulset/$statefulset" \
      -o jsonpath='{.spec.replicas}' 2>/dev/null || true)"
    ready="$(kubectl -n "$app_namespace" get "statefulset/$statefulset" \
      -o jsonpath='{.status.readyReplicas}' 2>/dev/null || true)"
    if [[ "$desired" == "1" && "$ready" == "1" ]]; then
      return 0
    fi
    sleep 3
  done
  echo "operator did not restore $app_namespace/statefulset/$statefulset to desired=ready=1" >&2
  kubectl -n "$app_namespace" get "statefulset/$statefulset" -o yaml >&2 || true
  return 1
}

tamper_and_require_reconcile() {
  kubectl -n "$app_namespace" patch "statefulset/$statefulset" --type=merge \
    --patch '{"spec":{"replicas":0}}'
  test "$(kubectl -n "$app_namespace" get "statefulset/$statefulset" -o jsonpath='{.spec.replicas}')" = "0"
  wait_statefulset_one
}

assert_can_i get leases.coordination.k8s.io "$operator_namespace"
if [[ "$app" != "tape" ]]; then
  # Tape's operator RBAC (apps/tape/k8s/operator/rbac.yaml) does not grant
  # cronjobs.batch: its backup CronJob is a handcrafted acceptance-harness
  # resource, not something the operator reconciles from the CR.
  assert_can_i create cronjobs.batch "$app_namespace"
fi
if [[ "$app" == "lumen" ]]; then
  assert_can_i get secrets "$app_namespace"
fi

initial_holder="$(wait_live_holder)"
printf '%s\n' "$initial_holder" > "$EVIDENCE_DIR/kubernetes/${app}-lease-holder-initial.txt"

# Reconcile drift before and after leader loss. A Lease transition alone is not
# sufficient proof that the replacement leader is running the controller.
tamper_and_require_reconcile
kubectl -n "$operator_namespace" scale "deployment/$service_account" --replicas=2
kubectl -n "$operator_namespace" rollout status "deployment/$service_account" --timeout=300s
before="$(wait_holder)"
kubectl -n "$operator_namespace" get "pod/$before" >/dev/null
kubectl -n "$operator_namespace" delete "pod/$before" --wait=true --timeout=120s
after="$(wait_holder "$before")"
tamper_and_require_reconcile
kubectl -n "$operator_namespace" scale "deployment/$service_account" --replicas=1
kubectl -n "$operator_namespace" rollout status "deployment/$service_account" --timeout=300s
settled="$(wait_live_holder)"

printf '%s\n' "$before" > "$EVIDENCE_DIR/kubernetes/${app}-lease-holder-before.txt"
printf '%s\n' "$after" > "$EVIDENCE_DIR/kubernetes/${app}-lease-holder-after.txt"
printf '%s\n' "$settled" > "$EVIDENCE_DIR/kubernetes/${app}-lease-holder-settled.txt"

jq -n \
  --arg schema "axiom.gcp.operator.cell.v1" \
  --arg app "$app" \
  --arg initial "$initial_holder" \
  --arg before "$before" \
  --arg after "$after" \
  --arg settled "$settled" \
  '{schema:$schema, app:$app, rbac:"passed", lease_creation:"passed", steady_state_drift_repair:"passed", leader_takeover_reconcile:"passed", holders:{initial:$initial,before_takeover:$before,after_takeover:$after,settled:$settled}}' \
  > "$EVIDENCE_DIR/${app}-operator-cell.json"
