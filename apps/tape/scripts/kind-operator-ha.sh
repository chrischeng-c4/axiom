#!/usr/bin/env bash
# Tape operator control-plane HA gate (#3053 AC1-AC4).
#
# The sibling gate `kind-e2e.sh` proves the *data* path on a single-node
# cluster. This one proves the *control* path, and it needs three nodes to say
# anything at all: on a single-node cluster a drain evicts every pod including
# the replacement, so "the drain completed and reconciliation continued" is not
# a statement that cluster can make.
#
# What it proves, in order:
#
#   AC1  `kubectl apply -k k8s/operator` installs the layer as one unit and
#        brings up >1 replica, exactly one of which holds the Lease.
#   AC4  the operator Service is scrapeable, and every replica answers --
#        the follower publishing `tape_operator_leader 0` is what makes a
#        failed handover visible instead of silent.
#   AC2  deleting the leader hands the Lease to another replica, and that
#        replica *applies* -- proven by a spec change reaching the child
#        StatefulSet, not merely by a status write.
#   AC3  draining the node hosting the leader completes, and reconciliation
#        continues across the whole window -- proven by a heartbeat running
#        concurrently with the drain, not by a single probe afterwards. The
#        window runs past the drain's return until a surviving replica holds
#        the Lease: `kubectl drain` returns on eviction, and the orphaned Lease
#        cannot be taken for at least its 15s duration, so the drain's own
#        window never contains the handover it causes. Every sample records the
#        holder and the holder must be seen to change, because reconciliation
#        is leader-gated and continuity under an unmoved holder only says the
#        surviving replica served the probes by itself.
#
# Usage:
#   bash apps/tape/scripts/kind-operator-ha.sh
#   TAPE_KEEP_CLUSTER=1 bash apps/tape/scripts/kind-operator-ha.sh
#
# Requirements: docker, kind, kubectl, jq.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
TAPE_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
REPO_ROOT="$(cd "$TAPE_DIR/../.." && pwd)"

CLUSTER_NAME="${TAPE_KIND_CLUSTER:-tape-cp-e2e}"
NAMESPACE="${TAPE_KIND_NAMESPACE:-tape}"
OPERATOR_NAMESPACE="tape-system"
OPERATOR_NAME="tape-operator"
# The Lease is named for the manager and lives in the operator's own namespace
# (libs/service-k8s/src/controller.rs:58-72), and its holderIdentity is the
# holder's POD_NAME. That is the whole leader-election observable.
LEASE_NAME="$OPERATOR_NAME"
METRICS_SERVICE="tape-operator-metrics"
TAPE_NAME="tape"
IMAGE_TAG="${TAPE_E2E_IMAGE:-tape:kind}"
CLUSTER_CREATED=0
PROBE_PID=""
PROBE_LOG=""
DRAINED_NODE=""

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
  echo "!! Tape control-plane diagnostics" >&2
  kubectl -n "$OPERATOR_NAMESPACE" get pods,svc,pdb,lease -o wide 2>&1 || true
  kubectl -n "$OPERATOR_NAMESPACE" logs "deploy/$OPERATOR_NAME" --all-containers --tail=120 2>&1 || true
  kubectl -n "$OPERATOR_NAMESPACE" get events --sort-by=.lastTimestamp 2>&1 || true
  kubectl -n "$NAMESPACE" get tapes.tape.dev,statefulset -o wide 2>&1 || true
  if [[ -n "$PROBE_LOG" && -s "$PROBE_LOG" ]]; then
    echo "!! reconcile heartbeat samples:" >&2
    cat "$PROBE_LOG" >&2
  fi
}

cleanup() {
  local ec=$?
  trap - EXIT INT TERM
  stop_heartbeat || true
  if [[ "$ec" -ne 0 ]]; then
    dump_diagnostics
  fi
  # Uncordon before deleting, so a preserved cluster is left usable.
  if [[ -n "$DRAINED_NODE" ]]; then
    kubectl uncordon "$DRAINED_NODE" >/dev/null 2>&1 || true
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

create_cluster() {
  # Three nodes, not one. Two workers so the preferred podAntiAffinity has
  # somewhere to spread the two operator replicas -- if they co-locate, the
  # drain in AC3 evicts both at once and the PDB blocks, which is precisely the
  # failure this gate exists to catch.
  kind create cluster --name "$CLUSTER_NAME" --wait 180s --config - <<'EOF'
kind: Cluster
apiVersion: kind.x-k8s.io/v1alpha4
nodes:
  - role: control-plane
  - role: worker
  - role: worker
EOF
  CLUSTER_CREATED=1
}

build_and_load_image() {
  docker build -f "$TAPE_DIR/Dockerfile" -t "$IMAGE_TAG" "$REPO_ROOT"
  kind load docker-image "$IMAGE_TAG" --name "$CLUSTER_NAME"
}

install_operator_layer() {
  # AC1's first half: one kustomization, not a file-by-file apply. If
  # k8s/operator/kustomization.yaml does not list every manifest in the
  # directory, this installs a partial control plane and the assertions below
  # are what notice.
  kubectl apply -k "$TAPE_DIR/k8s/operator"
  kubectl wait --for=condition=established crd/tapes.tape.dev --timeout=60s
  kubectl -n "$OPERATOR_NAMESPACE" set image "deployment/$OPERATOR_NAME" \
    "operator=$IMAGE_TAG"
  kubectl -n "$OPERATOR_NAMESPACE" rollout status "deployment/$OPERATOR_NAME" \
    --timeout=240s
}

operator_pods() {
  kubectl -n "$OPERATOR_NAMESPACE" get pods \
    -l "app.kubernetes.io/name=$OPERATOR_NAME" \
    -o jsonpath='{range .items[*]}{.metadata.name}{"\n"}{end}' | sed '/^$/d'
}

ready_operator_pods() {
  kubectl -n "$OPERATOR_NAMESPACE" get pods \
    -l "app.kubernetes.io/name=$OPERATOR_NAME" \
    -o json |
    jq -r '.items[]
           | select(any(.status.conditions[]?; .type == "Ready" and .status == "True"))
           | .metadata.name'
}

assert_two_ready_replicas() {
  local count
  count="$(ready_operator_pods | wc -l | tr -d ' ')"
  if [[ "$count" -lt 2 ]]; then
    echo "!! AC1: expected >1 ready operator replica, found $count" >&2
    kubectl -n "$OPERATOR_NAMESPACE" get pods -o wide >&2
    return 1
  fi
  echo "   AC1: $count ready operator replicas"
}

assert_replicas_on_distinct_nodes() {
  local nodes
  nodes="$(kubectl -n "$OPERATOR_NAMESPACE" get pods \
    -l "app.kubernetes.io/name=$OPERATOR_NAME" \
    -o jsonpath='{range .items[*]}{.spec.nodeName}{"\n"}{end}' | sed '/^$/d' | sort -u | wc -l | tr -d ' ')"
  if [[ "$nodes" -lt 2 ]]; then
    echo "!! replicas share a node; the AC3 drain would evict both at once and" >&2
    echo "   the PDB would block it. Check the podAntiAffinity term." >&2
    kubectl -n "$OPERATOR_NAMESPACE" get pods -o wide >&2
    return 1
  fi
  echo "   replicas spread across $nodes nodes"
}

lease_holder() {
  kubectl -n "$OPERATOR_NAMESPACE" get lease "$LEASE_NAME" \
    -o jsonpath='{.spec.holderIdentity}' 2>/dev/null || true
}

wait_for_lease() {
  local deadline=$(( $(date +%s) + 120 ))
  while [[ $(date +%s) -lt "$deadline" ]]; do
    local holder
    holder="$(lease_holder)"
    if [[ -n "$holder" ]]; then
      echo "   lease $OPERATOR_NAMESPACE/$LEASE_NAME held by $holder"
      return 0
    fi
    sleep 2
  done
  echo "!! no Lease holder within 120s" >&2
  return 1
}

assert_exactly_one_leader() {
  # Two independent sources must agree: the Lease object, and each replica's
  # own `tape_operator_leader` gauge. Asserting only the Lease would pass even
  # if a replica believed itself leader without holding it.
  local holder leaders=0 pod value
  holder="$(lease_holder)"
  [[ -n "$holder" ]] || { echo "!! no Lease holder" >&2; return 1; }
  for pod in $(ready_operator_pods); do
    value="$(scrape_pod "$pod" | awk '/^tape_operator_leader[ {]/ { print $NF }' | tail -1)"
    if [[ -z "$value" ]]; then
      echo "!! $pod published no tape_operator_leader series" >&2
      return 1
    fi
    echo "   $pod: tape_operator_leader $value"
    [[ "$value" == "1" ]] && leaders=$(( leaders + 1 ))
  done
  if [[ "$leaders" -ne 1 ]]; then
    echo "!! AC1: expected exactly one replica reporting leader 1, found $leaders" >&2
    return 1
  fi
}

# Scrape one pod directly, by pod IP, from inside the cluster.
scrape_pod() {
  local pod="$1" ip
  ip="$(kubectl -n "$OPERATOR_NAMESPACE" get pod "$pod" -o jsonpath='{.status.podIP}')"
  curl_in_cluster "http://${ip}:9090/metrics"
}

# Scrape through the Service -- AC4. This is the assertion that the Service's
# selector and its `targetPort: metrics` name actually resolve; a Service whose
# selector matches nothing returns a connection error, not an empty body.
assert_service_is_scrapeable() {
  local body
  body="$(curl_in_cluster "http://${METRICS_SERVICE}.${OPERATOR_NAMESPACE}.svc:9090/metrics")"
  if ! grep -q '^tape_operator_reconcile_total' <<<"$body"; then
    echo "!! AC4: Service scrape returned no tape_operator_reconcile_total series" >&2
    echo "$body" | head -20 >&2
    return 1
  fi
  local endpoints
  endpoints="$(kubectl -n "$OPERATOR_NAMESPACE" get endpoints "$METRICS_SERVICE" \
    -o jsonpath='{range .subsets[*].addresses[*]}{.ip}{"\n"}{end}' | sed '/^$/d' | wc -l | tr -d ' ')"
  if [[ "$endpoints" -lt 2 ]]; then
    echo "!! AC4: Service has $endpoints endpoint(s); both replicas must be" >&2
    echo "   separate scrape targets or a lease handover is invisible." >&2
    return 1
  fi
  echo "   AC4: Service scrapeable, $endpoints endpoints"
}

curl_in_cluster() {
  kubectl -n "$OPERATOR_NAMESPACE" run "curl-$RANDOM" \
    --rm -i --restart=Never --quiet \
    --image=curlimages/curl:8.10.1 -- \
    curl -fsS --max-time 15 "$1"
}

assert_monitoring_component_renders() {
  # AC4's second half. The component is deliberately NOT part of k8s/operator
  # -- it is monitoring.coreos.com/v1, and shipping it in the base layer would
  # make `apply -k` fail on any cluster without prometheus-operator, taking the
  # operator install down along with the alerts. So it is rendered and
  # inspected here, never applied to this cluster.
  local out
  out="$(kubectl kustomize "$TAPE_DIR/k8s/components/operator-monitoring")"
  grep -q 'kind: ServiceMonitor' <<<"$out" ||
    { echo "!! component renders no ServiceMonitor" >&2; return 1; }
  grep -q 'kind: PrometheusRule' <<<"$out" ||
    { echo "!! component renders no PrometheusRule" >&2; return 1; }
  # And the base layer must stay CRD-free apart from tape's own CRD. The
  # property is about the *objects installed*, not about the string appearing:
  # the ClusterRole legitimately grants `monitoring.coreos.com` verbs (the
  # operator renders instance-level ServiceMonitors when a CR opts in), and the
  # CRD's own doc comment names the group in prose. Neither is applied as an
  # object of that group, so match on `apiVersion:` alone.
  local base
  base="$(kubectl kustomize "$TAPE_DIR/k8s/operator")"
  if grep -qE '^apiVersion: monitoring\.coreos\.com/' <<<"$base"; then
    echo "!! k8s/operator installs a monitoring.coreos.com object; apply -k would" >&2
    echo "   fail on a cluster without prometheus-operator." >&2
    grep -nE '^apiVersion: monitoring\.coreos\.com/' <<<"$base" >&2
    return 1
  fi
  echo "   AC4: monitoring component renders ServiceMonitor + PrometheusRule; base stays CRD-free"
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
  # Since #2765 an absent spec.auth defaults to required, and a CR asking for
  # auth without a token source renders a pod that cannot start. This gate
  # proves control-plane reconciliation, not authentication.
  auth: disabled
EOF
}

# One reconcile heartbeat: bump the spec, then wait for the change to reach the
# child StatefulSet. Deliberately observes the *child*, not status -- a status
# write proves a replica is watching; only the child proves it is applying, and
# applying is what a standby replica must take over.
reconcile_once() {
  local level="$1" deadline
  kubectl -n "$NAMESPACE" patch tape "$TAPE_NAME" --type=merge \
    -p "{\"spec\":{\"logLevel\":\"$level\"}}" >/dev/null
  deadline=$(( $(date +%s) + 90 ))
  while [[ $(date +%s) -lt "$deadline" ]]; do
    local got
    got="$(kubectl -n "$NAMESPACE" get statefulset "$TAPE_NAME" -o json 2>/dev/null |
      jq -r '.spec.template.spec.containers[]?.env[]? | select(.name == "RUST_LOG") | .value' |
      head -1)"
    if [[ "$got" == "$level" ]]; then
      return 0
    fi
    sleep 2
  done
  return 1
}

wait_for_statefulset() {
  local deadline=$(( $(date +%s) + 120 ))
  while [[ $(date +%s) -lt "$deadline" ]]; do
    kubectl -n "$NAMESPACE" get statefulset "$TAPE_NAME" >/dev/null 2>&1 && return 0
    sleep 2
  done
  echo "!! operator did not create StatefulSet/$TAPE_NAME within 120s" >&2
  return 1
}

# AC3's real assertion. A single probe after the drain would pass even if
# reconciliation had been dead for the entire drain window, so the heartbeat
# runs *concurrently* with the drain and every sample is recorded.
start_heartbeat() {
  PROBE_LOG="$(mktemp)"
  (
    local n=0
    while true; do
      n=$(( n + 1 ))
      local level start elapsed
      if (( n % 2 == 0 )); then level=debug; else level=info; fi
      local holder
      holder="$(lease_holder)"
      start=$(date +%s)
      if reconcile_once "$level"; then
        elapsed=$(( $(date +%s) - start ))
        echo "ok $n $elapsed ${holder:-none}" >>"$PROBE_LOG"
      else
        echo "FAIL $n timeout ${holder:-none}" >>"$PROBE_LOG"
      fi
      sleep 1
    done
  ) &
  PROBE_PID=$!
}

stop_heartbeat() {
  if [[ -n "$PROBE_PID" ]]; then
    kill "$PROBE_PID" >/dev/null 2>&1 || true
    wait "$PROBE_PID" 2>/dev/null || true
    PROBE_PID=""
  fi
}

assert_heartbeat_unbroken() {
  local max="${1:-75}" failures samples worst holders transition
  failures="$(grep -c '^FAIL' "$PROBE_LOG" || true)"
  samples="$(grep -c '^ok' "$PROBE_LOG" || true)"
  if [[ "$failures" -ne 0 ]]; then
    echo "!! AC3: $failures reconcile heartbeat sample(s) never converged" >&2
    cat "$PROBE_LOG" >&2
    return 1
  fi
  if [[ "$samples" -lt 3 ]]; then
    echo "!! AC3: only $samples heartbeat samples; the drain window was too" >&2
    echo "   short for this to mean anything. Not calling that a pass." >&2
    cat "$PROBE_LOG" >&2
    return 1
  fi
  worst="$(awk '/^ok/ { if ($3 > m) m = $3 } END { print m + 0 }' "$PROBE_LOG")"
  if [[ "$worst" -gt "$max" ]]; then
    echo "!! AC3: worst reconcile latency ${worst}s exceeds ${max}s" >&2
    cat "$PROBE_LOG" >&2
    return 1
  fi
  # "Reconciliation continued" only means something if the drain actually took
  # the leader away. Reconciliation is leader-gated -- reconcile_entry() in
  # libs/service-k8s/src/controller.rs returns a bare requeue on a follower --
  # so if the Lease holder never changed across the whole window, the surviving
  # replica did every one of these reconciles and the drain proved nothing
  # about a handover. That reads as a pass and is not one, which is the exact
  # failure this whole script is built to refuse.
  holders="$(awk '/^(ok|FAIL)/ { print $4 }' "$PROBE_LOG" | grep -vx none | sort -u)"
  if [[ "$(wc -l <<<"$holders" | tr -d ' ')" -lt 2 ]]; then
    echo "!! AC3: the Lease holder never changed during the drain (${holders//$'\n'/, })." >&2
    echo "   The drained node was chosen because it hosted the leader, so either" >&2
    echo "   the leader was not evicted or it re-acquired before any sample saw" >&2
    echo "   the gap. Either way this run did not exercise a handover." >&2
    cat "$PROBE_LOG" >&2
    return 1
  fi
  transition="$(tr '\n' ' ' <<<"$holders")"
  echo "   AC3: $samples heartbeat samples across the drain, worst ${worst}s"
  echo "   AC3: reconciled under ${transition% } -- the leader moved mid-drain"
}

prove_lease_handover() {
  local before after standby deadline start elapsed
  before="$(lease_holder)"
  # The replica that is NOT the leader right now. Recorded before the delete,
  # because after it the Deployment creates a replacement and "the other pod"
  # stops being unambiguous -- which is exactly the distinction this gate has
  # to draw: a warm standby acquiring in seconds is the point of running two
  # replicas, a fresh replacement acquiring in a minute is not, and both look
  # identical if all you check is "the holder changed".
  standby="$(ready_operator_pods | grep -vx "$before" | head -1)"
  echo "   leader before: $before (standby: ${standby:-none})"
  start=$(date +%s)
  kubectl -n "$OPERATOR_NAMESPACE" delete pod "$before" --wait=false >/dev/null
  deadline=$(( start + 180 ))
  while [[ $(date +%s) -lt "$deadline" ]]; do
    after="$(lease_holder)"
    if [[ -n "$after" && "$after" != "$before" ]]; then
      elapsed=$(( $(date +%s) - start ))
      echo "   leader after:  $after (after ${elapsed}s)"
      if [[ -n "$standby" && "$after" == "$standby" ]]; then
        echo "   AC2: the warm standby acquired the Lease"
      else
        # Not a failure on its own -- reconciliation resumed either way, and
        # AC3 measures the latency that actually matters. But say which
        # happened, because "replicas: 2 buys a fast takeover" is only true
        # in the first branch.
        echo "   AC2: NOTE the Lease went to $after, not the standby ${standby:-none};"
        echo "        a replacement pod won the race. Takeover still took ${elapsed}s."
      fi
      if [[ "$elapsed" -gt 90 ]]; then
        echo "!! AC2: ${elapsed}s to reacquire the Lease. Two replicas exist to make" >&2
        echo "   this fast; that is slow enough that the standby is not earning its" >&2
        echo "   keep. Check the Lease durationSeconds and retry period." >&2
        return 1
      fi
      # The handover is only meaningful if the new leader *applies*. Prove it
      # with a spec change that must reach the child StatefulSet.
      reconcile_once debug ||
        { echo "!! AC2: new leader $after did not reconcile a spec change" >&2; return 1; }
      echo "   AC2: new leader applied a spec change to the child StatefulSet"
      return 0
    fi
    sleep 2
  done
  echo "!! AC2: lease still held by $before after 180s" >&2
  return 1
}

# `kubectl drain` returns when the pods are *evicted*, not when the control
# plane has recovered from losing them, and the Lease it just orphaned cannot
# be taken for lease.rs's 15s duration -- longer in practice, because the
# departing leader goes on renewing through its termination grace. So the drain
# window on its own never contains the handover it causes.
#
# The condition is that the *heartbeat* has recorded a sample under a new
# holder, not merely that the Lease moved. Each sample stamps the holder it saw
# when it started, so waiting only for the Lease leaves the last sample stamped
# with the old holder -- a run here saw the Lease move 9s after the drain
# returned and still had every recorded sample under the outgoing pod. The
# handover evidence and the continuity evidence have to come out of the same
# log, or they are two claims about two different windows.
wait_for_heartbeat_under_new_holder() {
  local before="$1" timeout="$2" deadline start holder elapsed
  start=$(date +%s)
  deadline=$(( start + timeout ))
  while [[ $(date +%s) -lt "$deadline" ]]; do
    holder="$(awk -v b="$before" \
      '/^(ok|FAIL)/ && $4 != b && $4 != "none" { print $4; exit }' "$PROBE_LOG")"
    if [[ -n "$holder" ]]; then
      elapsed=$(( $(date +%s) - start ))
      echo "   AC3: heartbeat reconciled under new holder $holder," \
        "${elapsed}s after the drain returned"
      return 0
    fi
    sleep 2
  done
  return 1
}

prove_drain_completes() {
  local leader node
  leader="$(lease_holder)"
  node="$(kubectl -n "$OPERATOR_NAMESPACE" get pod "$leader" -o jsonpath='{.spec.nodeName}')"
  echo "   draining $node (hosts leader $leader)"
  DRAINED_NODE="$node"
  start_heartbeat
  # --timeout is the assertion: with replicas: 2 and maxUnavailable: 1 this
  # must complete. If the PDB is wrong, or both replicas share the node, the
  # eviction API refuses forever and this exits non-zero.
  if ! kubectl drain "$node" \
    --ignore-daemonsets --delete-emptydir-data --force --timeout=240s; then
    stop_heartbeat
    echo "!! AC3: drain of $node did not complete within 240s" >&2
    kubectl -n "$OPERATOR_NAMESPACE" get pdb,pods -o wide >&2
    return 1
  fi
  # Keep probing past the drain's return until a surviving replica actually
  # holds the Lease. Stopping here instead is what made the first green run of
  # this gate report a 3s worst latency for a handover that had not started.
  if ! wait_for_heartbeat_under_new_holder "$leader" 180; then
    stop_heartbeat
    echo "!! AC3: no surviving replica reconciled as a new Lease holder within" >&2
    echo "   180s of the drain returning (was $leader; Lease now names" >&2
    echo "   '$(lease_holder)'). The drain completed, so the eviction path is" >&2
    echo "   fine; the control plane did not come back from it." >&2
    kubectl -n "$OPERATOR_NAMESPACE" get lease,pods -o wide >&2
    return 1
  fi
  stop_heartbeat
  assert_heartbeat_unbroken 75
  kubectl uncordon "$node"
  DRAINED_NODE=""
}

main() {
  require docker
  require kind
  require kubectl
  require jq

  step "create 3-node Kind cluster $CLUSTER_NAME" create_cluster
  step "build + load $IMAGE_TAG" build_and_load_image
  step "AC1: apply k8s/operator as one kustomization" install_operator_layer
  step "AC1: >1 ready replica" assert_two_ready_replicas
  step "replicas on distinct nodes" assert_replicas_on_distinct_nodes
  step "AC1: a Lease holder exists" wait_for_lease
  step "AC1: exactly one replica reports leader" assert_exactly_one_leader
  step "AC4: Service is scrapeable with both replicas as endpoints" assert_service_is_scrapeable
  step "AC4: monitoring component renders, base stays CRD-free" assert_monitoring_component_renders
  step "apply Tape CR" apply_tape_instance
  step "operator reconciles it into a StatefulSet" wait_for_statefulset
  step "AC2: delete the leader, prove handover + apply" prove_lease_handover
  step "AC3: drain the leader's node with a live reconcile heartbeat" prove_drain_completes

  echo "== tape operator control-plane HA gate PASSED (#3053 AC1-AC4)"
}

main "$@"
