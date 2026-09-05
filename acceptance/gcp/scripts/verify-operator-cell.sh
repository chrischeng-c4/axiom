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
    desired_replicas=1
    ;;
  sift)
    operator_namespace="sift-system"
    service_account="sift-operator"
    lease="sift-operator"
    app_namespace="sift"
    statefulset="sift-store"
    desired_replicas=3
    ;;
  tape)
    # Lease name = ManagedService::MANAGER in apps/tape/src/operator/reconcile.rs.
    operator_namespace="tape-system"
    service_account="tape-operator"
    lease="tape-operator"
    app_namespace="tape"
    statefulset="tape"
    desired_replicas=1
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
  # `kubectl auth can-i` exits 1 when the answer is "no". Without `|| true`,
  # `set -e` kills the script at this assignment -- before the row is recorded
  # and before the message below is printed -- so a missing grant surfaced as a
  # run that stopped after the last successful phase line with no diagnostic at
  # all. The exit status is not the signal here; the word it printed is.
  result="$(kubectl auth can-i "$verb" "$resource" \
    --namespace="$target_namespace" \
    --as="system:serviceaccount:${operator_namespace}:${service_account}" || true)"
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

# --- control-plane self-observability (#2620 / #2621) ----------------------
# The shared listener in libs/service-k8s ships for every operator, but the
# Service that makes it reachable is per-app and today only lumen wires one, so
# these helpers are used under an `$app` guard.
#
# The operator image is distroless -- no shell, no curl, nothing to exec into --
# so every HTTP assertion port-forwards and curls from the runner.
#
# The local port counter is a FILE, not a variable, and that is load-bearing.
# Callers read this through `$(...)`, which is a subshell, so an in-function
# `port=$((port + 1))` is discarded on return and every call reuses one port --
# whichever forward is still bound there answers, so two different pods report
# byte-identical metrics. That reads as "the leader gauge is stuck" rather than
# "you scraped the same pod twice", which is the wrong bug to chase in a paid
# cluster run.
metrics_port_state="$(mktemp)"
printf '39090\n' > "$metrics_port_state"
next_metrics_port() {
  local p
  p=$(( $(cat "$metrics_port_state") + 1 ))
  printf '%s\n' "$p" > "$metrics_port_state"
  printf '%s' "$p"
}
operator_metrics() {
  local pod="$1" pid body="" rc=1 metrics_local_port
  metrics_local_port="$(next_metrics_port)"
  kubectl -n "$operator_namespace" port-forward "pod/$pod" \
    "$metrics_local_port:9090" >/dev/null 2>&1 &
  pid=$!
  for _ in $(seq 1 40); do
    if body="$(curl -sf --max-time 5 "http://127.0.0.1:$metrics_local_port/metrics" 2>/dev/null)"; then
      rc=0
      break
    fi
    sleep 0.5
  done
  kill "$pid" 2>/dev/null || true
  wait "$pid" 2>/dev/null || true
  printf '%s' "$body"
  return $rc
}

# Terminating pods keep phase Running and their labels, so a naive selector
# counts the outgoing replica too; filter on deletionTimestamp.
live_operator_pods() {
  kubectl -n "$operator_namespace" get pods \
    -l "app.kubernetes.io/name=$service_account" -o json 2>/dev/null \
    | jq -r '.items[] | select(.metadata.deletionTimestamp == null) | .metadata.name' \
    || true
}

# The gauge must agree with the Lease -- and the Lease is established
# independently, above, by kubectl. That cross-check is the whole point: a pod
# publishing `1` proves nothing on its own, while a pod publishing `1` exactly
# when the Lease names it, and `0` when it does not, proves the gauge tracks
# real leadership rather than being wired to a constant.
require_leader_gauge_agrees() {
  local holder="$1" pod want got ok
  local deadline=$((SECONDS + 120))
  while (( SECONDS < deadline )); do
    ok=1
    for pod in $(live_operator_pods); do
      want=0
      [[ "$pod" == "$holder" ]] && want=1
      # `set -euo pipefail` is on: a pod that is mid-deletion refuses the
      # port-forward, and an unguarded failure here would abort the whole
      # acceptance run instead of retrying a transient miss.
      got="$( { operator_metrics "$pod" || true; } \
        | awk '$1 == "lumen_operator_leader" {print $2}' )"
      if [[ "$got" != "$want" ]]; then
        ok=0
        break
      fi
    done
    if (( ok == 1 )); then
      return 0
    fi
    sleep 3
  done
  echo "lumen_operator_leader never agreed with lease holder $holder" >&2
  kubectl -n "$operator_namespace" get pods -o wide >&2 || true
  return 1
}

# Every live replica must be an endpoint of the metrics Service. The follower
# matters as much as the leader: it is the replica whose `_leader 0` proves a
# handover actually moved, and prometheus scrapes Endpoints, not the VIP, so a
# replica missing here is a silently unscraped one.
require_metrics_endpoints_cover_replicas() {
  local expected actual
  expected="$(live_operator_pods | wc -l | tr -d ' ')"
  local deadline=$((SECONDS + 120))
  actual=0
  while (( SECONDS < deadline )); do
    actual="$( { kubectl -n "$operator_namespace" get endpoints "$service_account-metrics" \
      -o json 2>/dev/null || printf '{}'; } \
      | jq '[.subsets // [] | .[].addresses // [] | length] | add // 0' )"
    if [[ "$actual" == "$expected" ]]; then
      printf '%s\n' "$actual"
      return 0
    fi
    sleep 3
  done
  echo "metrics Service has $actual endpoints for $expected live replicas" >&2
  return 1
}

wait_statefulset_desired() {
  local name="${1:-$statefulset}"
  local expected="${2:-$desired_replicas}"
  local after_generation="${3:-0}" expected_uid="${4:-}"
  local snapshot
  local deadline=$((SECONDS + 240))
  while (( SECONDS < deadline )); do
    snapshot="$(kubectl -n "$app_namespace" get "statefulset/$name" -o json 2>/dev/null || true)"
    printf '%s\n' "$snapshot" > "$EVIDENCE_DIR/kubernetes/${app}-${name}-reconcile-last.json"
    if jq -e --argjson expected "$expected" --argjson after "$after_generation" \
        --arg uid "$expected_uid" '
        .spec.replicas == $expected and .status.readyReplicas == $expected
        and (.metadata.generation > $after)
        and (.status.observedGeneration >= .metadata.generation)
        and ($uid == "" or .metadata.uid == $uid)
      ' <<<"$snapshot" >/dev/null 2>&1; then
      return 0
    fi
    sleep 3
  done
  echo "operator did not restore $app_namespace/statefulset/$name to desired=ready=$expected" >&2
  kubectl -n "$app_namespace" get "statefulset/$name" -o yaml >&2 || true
  return 1
}

drift_sequence=0
tamper_and_require_reconcile() {
  local receipt generation uid
  drift_sequence=$((drift_sequence + 1))
  receipt="$EVIDENCE_DIR/kubernetes/${app}-drift-${drift_sequence}-patch.json"
  # The PATCH response proves the write. A later GET may already see a correct
  # reconcile and must not be required to observe the transient zero replicas.
  kubectl -n "$app_namespace" patch "statefulset/$statefulset" --type=merge \
    --patch '{"spec":{"replicas":0}}' -o json > "$receipt"
  jq -e --arg name "$statefulset" --arg namespace "$app_namespace" '
    .metadata.name == $name and .metadata.namespace == $namespace
    and .spec.replicas == 0
    and (.metadata.generation | type == "number" and . > 0 and floor == .)
    and (.metadata.uid | type == "string" and length > 0)
  ' "$receipt" >/dev/null || {
    echo "the patch response did not prove $app_namespace/$statefulset drift; see $receipt" >&2
    return 1
  }
  generation="$(jq -r '.metadata.generation' "$receipt")"
  uid="$(jq -r '.metadata.uid' "$receipt")"
  wait_statefulset_desired "$statefulset" "$desired_replicas" "$generation" "$uid"
  if [[ "$app" == "sift" ]]; then
    wait_statefulset_desired sift-control 3
  fi
}

assert_can_i get leases.coordination.k8s.io "$operator_namespace"
if [[ "$app" != "tape" ]]; then
  # Lumen/Sift operators reconcile CR-native backup fields into CronJobs;
  # tape's backup CronJob is hand-rolled by the harness (the Tape CRD has no
  # backup field), so its operator deliberately carries no batch RBAC.
  assert_can_i create cronjobs.batch "$app_namespace"
fi
# Lumen's operator used to read Secrets, for the auth token registry and the
# backup runner's bearer token. #2870 and #2871 removed both projections and
# #2889 replaced them with rendered RBAC, so `secrets` is no longer in the
# operator's ClusterRole and asserting it fails a correct deployment. The
# grant that replaced it -- `bind` on `system:auth-delegator` -- is not
# assertable with `auth can-i`, and the binding it produces is checked
# directly by verify-lumen-auth.sh (#2879).

initial_holder="$(wait_live_holder)"
printf '%s\n' "$initial_holder" > "$EVIDENCE_DIR/kubernetes/${app}-lease-holder-initial.txt"

# Reconcile drift before and after leader loss. A Lease transition alone is not
# sufficient proof that the replacement leader is running the controller.
tamper_and_require_reconcile
kubectl -n "$operator_namespace" scale "deployment/$service_account" --replicas=2
kubectl -n "$operator_namespace" rollout status "deployment/$service_account" --timeout=300s
before="$(wait_holder)"
# Two replicas are up and the Lease names one of them: the only moment in this
# script where the leader gauge can be checked against a known follower.
metrics_endpoints=""
if [[ "$app" == "lumen" ]]; then
  metrics_endpoints="$(require_metrics_endpoints_cover_replicas)"
  require_leader_gauge_agrees "$before"
fi
kubectl -n "$operator_namespace" get "pod/$before" >/dev/null
kubectl -n "$operator_namespace" delete "pod/$before" --wait=true --timeout=120s
after="$(wait_holder "$before")"
# The gauge has to move with the Lease, not just match it once. Re-checked after
# the takeover, this is what would catch a gauge set at startup and never
# updated -- which would look correct in the check above and be wrong here.
if [[ "$app" == "lumen" ]]; then
  require_leader_gauge_agrees "$after"
fi
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
  --arg metrics "${metrics_endpoints:-}" \
  '{schema:$schema, app:$app, rbac:"passed", lease_creation:"passed", steady_state_drift_repair:"passed", leader_takeover_reconcile:"passed", holders:{initial:$initial,before_takeover:$before,after_takeover:$after,settled:$settled}}
   + (if $metrics == "" then {} else {control_plane_observability:{status:"passed", metrics_endpoints:($metrics|tonumber), leader_gauge_tracks_lease:"passed"}} end)' \
  > "$EVIDENCE_DIR/${app}-operator-cell.json"
