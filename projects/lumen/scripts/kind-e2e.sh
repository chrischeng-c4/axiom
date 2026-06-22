#!/usr/bin/env bash
# lumen — kind-based end-to-end happy-path test.
#
# Implements the README §9 happy-path on the log-replicated architecture:
# spin up a single-node kind cluster, apply the `dev` overlay (Relay broker +
# lumen serving Deployment), drive the public HTTP API
# (:7373) through schema → index 10k → search → duplicates, then KILL ALL
# SERVING PODS and verify search results are identical after the new pods
# rebuild their index by tailing the Relay log. (The broker is NOT killed;
# durability lives in Relay, not in the ephemeral serving pods.)
#
# Usage:  scripts/kind-e2e.sh
#         LUMEN_E2E_MODE=operator scripts/kind-e2e.sh   # deploy via the CRD
# Exit code 0 = success; any assertion failure exits non-zero.
#
# Deploy modes (LUMEN_E2E_MODE):
#   overlay  (default) — kubectl apply -k k8s/overlays/dev (hand-written manifests)
#   operator           — install the Lumen CRD + operator, then apply a Lumen CR
#                        and let the operator reconcile the same fleet + broker.
# Both exercise the identical index → search → kill → rebuild-from-log path.
#
# Requirements: kind, kubectl, docker, curl, jq, python3.
#
# Host → cluster reachability uses kind `extraPortMappings` (hostPort →
# node NodePort) instead of `kubectl port-forward`: the long-lived
# port-forward tunnel stalls on successive large POSTs (the 10k-doc index),
# whereas the node port mapping is a stable kernel-level forward.

set -euo pipefail

# ---------------------------------------------------------------------------
# Paths and config
# ---------------------------------------------------------------------------

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
LUMEN_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
REPO_ROOT="$(cd "$LUMEN_DIR/../.." && pwd)"

CLUSTER_NAME="${LUMEN_KIND_CLUSTER:-lumen-e2e}"
NAMESPACE="lumen"
# Deploy path: `overlay` (default) or `operator`. The operator renders the
# recommended app.kubernetes.io/* labels; the hand-written manifests use
# app/role. Resource NAMES are identical in both (CR named `lumen` in ns
# `lumen` → Deployment `lumen`, StatefulSet `lumen-relay`), so only the label
# selectors and Service handling differ between modes.
E2E_MODE="${LUMEN_E2E_MODE:-overlay}"
OPERATOR_NS="lumen-system"
LUMEN_CR_NAME="lumen"
if [[ "$E2E_MODE" == "operator" ]]; then
  APP_LABEL="app.kubernetes.io/name=lumen,app.kubernetes.io/component=server"
  BROKER_LABEL="app.kubernetes.io/name=lumen,app.kubernetes.io/component=broker"
else
  # Serving pods only — NOT the Relay broker, which also carries app=lumen.
  APP_LABEL="app=lumen,role=server"
  BROKER_LABEL="app=lumen,role=broker"
fi
# Host port (extraPortMappings) → node NodePort → Service :7373.
PORT_LOCAL="${LUMEN_PORT_LOCAL:-17373}"
PORT_REMOTE=7373
NODE_PORT="${LUMEN_NODE_PORT:-30737}"
DOC_COUNT="${LUMEN_E2E_DOC_COUNT:-10000}"
BATCH_SIZE=10000

FIXTURE_FILE=""
IMAGE_TAG="${LUMEN_E2E_IMAGE:-lumen:latest}"

# ---------------------------------------------------------------------------
# Timing helper
# ---------------------------------------------------------------------------

step() {
  local label="$1"
  shift
  local start
  start=$(date +%s)
  echo ">> $label"
  "$@"
  local end
  end=$(date +%s)
  echo "   ($label finished in $((end - start))s)"
}

# ---------------------------------------------------------------------------
# Cleanup on exit
# ---------------------------------------------------------------------------

cleanup() {
  local ec=$?
  if [[ -n "$FIXTURE_FILE" ]]; then
    rm -f "$FIXTURE_FILE" "${FIXTURE_FILE%.json}".req.*.json
  fi
  if [[ "${LUMEN_KEEP_CLUSTER:-0}" != "1" ]]; then
    kind delete cluster --name "$CLUSTER_NAME" >/dev/null 2>&1 || true
  fi
  exit "$ec"
}
trap cleanup EXIT INT TERM

# ---------------------------------------------------------------------------
# Helpers
# ---------------------------------------------------------------------------

# Poll until ≥1 pod matches the label (so a subsequent `kubectl wait` does not
# error with "no matching resources" — the operator creates the workload a beat
# after the CR is applied).
wait_pods_exist() {
  local label="$1" timeout="${2:-60}" deadline
  deadline=$(( $(date +%s) + timeout ))
  while [[ $(date +%s) -lt $deadline ]]; do
    if [[ "$(kubectl -n "$NAMESPACE" get pod -l "$label" --no-headers 2>/dev/null | wc -l | tr -d ' ')" -ge 1 ]]; then
      return 0
    fi
    sleep 2
  done
}

wait_broker_ready() {
  local timeout="${1:-120}"
  echo "   waiting up to ${timeout}s for Relay broker ($BROKER_LABEL) Ready"
  wait_pods_exist "$BROKER_LABEL" "$timeout"
  kubectl -n "$NAMESPACE" wait --for=condition=Ready pod -l "$BROKER_LABEL" \
    --timeout="${timeout}s"
}

wait_lumen_ready() {
  local timeout="${1:-180}"
  echo "   waiting up to ${timeout}s for serving pods ($APP_LABEL) Ready"
  wait_pods_exist "$APP_LABEL" "$timeout"
  kubectl -n "$NAMESPACE" wait --for=condition=Ready pod -l "$APP_LABEL" \
    --timeout="${timeout}s"
}

# Wait for the broker pod to be RECREATED (new UID) and Ready after a kill.
# A plain `kubectl wait --for=Ready` is racy here: it can match the
# still-terminating old pod (briefly still Ready → false positive) or error
# before the StatefulSet recreates lumen-relay-0. Poll on a UID change instead.
wait_broker_recovered() {
  local old_uid="$1" timeout="${2:-180}" deadline
  deadline=$(( $(date +%s) + timeout ))
  echo "   waiting up to ${timeout}s for Relay broker to recreate (old uid ${old_uid:0:8}…) + Ready"
  while [[ $(date +%s) -lt $deadline ]]; do
    local uid ready
    uid="$(kubectl -n "$NAMESPACE" get pod lumen-relay-0 -o jsonpath='{.metadata.uid}' 2>/dev/null || true)"
    ready="$(kubectl -n "$NAMESPACE" get pod lumen-relay-0 \
      -o jsonpath='{range .status.conditions[?(@.type=="Ready")]}{.status}{end}' 2>/dev/null || true)"
    if [[ -n "$uid" && "$uid" != "$old_uid" && "$ready" == "True" ]]; then
      echo "   Relay broker recovered (new uid ${uid:0:8}…)"
      return 0
    fi
    sleep 3
  done
  echo "!! Relay broker did not recover within ${timeout}s" >&2
  kubectl -n "$NAMESPACE" get pods -l "$BROKER_LABEL" >&2 || true
  return 1
}

# Build the lumen and relay images and load them into the kind node.
#
# Built from the WORKSPACE ROOT as context (the same pattern as
# projects/lumen/compose.yaml and conductor's CI): cargo resolves the whole
# workspace and `cargo build -p lumen` in the Dockerfile compiles only lumen's
# real dependency closure. The repo-root .dockerignore keeps the context to
# source-only (~MBs, not the 35G of target/). The deployment pins
# imagePullPolicy: IfNotPresent, so once `kind load` has injected the image
# into the node it is used without a registry pull.
build_and_load_image() {
  docker build -f "$LUMEN_DIR/Dockerfile" -t "$IMAGE_TAG" "$REPO_ROOT"
  kind load docker-image "$IMAGE_TAG" --name "$CLUSTER_NAME"
  docker build -f "$REPO_ROOT/projects/relay/Dockerfile" -t relay:latest "$REPO_ROOT"
  kind load docker-image relay:latest --name "$CLUSTER_NAME"
}

# Deploy lumen by the selected mode.
deploy_lumen() {
  if [[ "$E2E_MODE" == "operator" ]]; then
    deploy_via_operator
  else
    kubectl apply -k "${LUMEN_DIR}/k8s/overlays/dev"
  fi
}

# Operator path: install the CRD + RBAC + operator (same image as serving),
# then apply a dev-shaped Lumen CR and let the reconcile loop materialize the
# serving Deployment + Relay StatefulSet. Resource names match the overlay path,
# so the rest of the test is mode-agnostic.
deploy_via_operator() {
  kubectl apply -k "${LUMEN_DIR}/k8s/operator"
  # Pin the operator to the freshly-built image (the manifest hard-codes
  # lumen:latest; honor a custom $IMAGE_TAG).
  kubectl -n "$OPERATOR_NS" set image deploy/lumen-operator operator="$IMAGE_TAG"
  echo "   waiting for the Lumen CRD to be Established"
  kubectl wait --for=condition=established crd/lumens.lumen.dev --timeout=60s
  echo "   waiting for the operator Deployment to roll out"
  kubectl -n "$OPERATOR_NS" rollout status deploy/lumen-operator --timeout=180s

  # The CR's namespace must exist before the CR is applied.
  kubectl create namespace "$NAMESPACE" --dry-run=client -o yaml | kubectl apply -f -
  kubectl apply -f - <<EOF
apiVersion: lumen.dev/v1alpha1
kind: Lumen
metadata:
  name: ${LUMEN_CR_NAME}
  namespace: ${NAMESPACE}
spec:
  image: ${IMAGE_TAG}
  imagePullPolicy: IfNotPresent
  shardCount: 1
  logFormat: pretty
  serving:
    autoscaling:
      minReplicas: 1
      maxReplicas: 3
      targetCpuUtilization: 70
  broker:
    image: relay:latest
    storage: 1Gi
EOF

  echo "   Lumen/${LUMEN_CR_NAME} applied; waiting for the operator to render child objects"
  local deadline=$(( $(date +%s) + 60 ))
  while [[ $(date +%s) -lt $deadline ]]; do
    if kubectl -n "$NAMESPACE" get deploy/"${LUMEN_CR_NAME}" >/dev/null 2>&1 \
       && kubectl -n "$NAMESPACE" get statefulset/"${LUMEN_CR_NAME}-relay" >/dev/null 2>&1; then
      echo "   operator reconciled Deployment/${LUMEN_CR_NAME} + StatefulSet/${LUMEN_CR_NAME}-relay"
      return 0
    fi
    sleep 2
  done
  echo "!! operator did not render child objects within 60s" >&2
  kubectl -n "$OPERATOR_NS" logs deploy/lumen-operator --tail=60 >&2 || true
  return 1
}

# Create the kind cluster with a host→node port mapping so the host can
# reach the NodePort service directly (no port-forward tunnel to stall).
create_cluster() {
  kind create cluster --name "$CLUSTER_NAME" --wait 120s --config - <<EOF
kind: Cluster
apiVersion: kind.x-k8s.io/v1alpha4
nodes:
  - role: control-plane
    extraPortMappings:
      - containerPort: ${NODE_PORT}
        hostPort: ${PORT_LOCAL}
        protocol: TCP
EOF
}

# Expose the lumen Service on a fixed NodePort that the host port mapping
# targets. The base overlay keeps the Service ClusterIP (NodePort is a
# kind-test concern); patch it here rather than in the shipped manifests.
expose_nodeport() {
  if [[ "$E2E_MODE" == "operator" ]]; then
    # The operator owns Service/lumen and would revert a NodePort patch on its
    # next reconcile. Use a SEPARATE, operator-untouched NodePort Service that
    # selects the same serving pods.
    kubectl -n "$NAMESPACE" apply -f - <<EOF
apiVersion: v1
kind: Service
metadata:
  name: lumen-np
  namespace: ${NAMESPACE}
spec:
  type: NodePort
  selector:
    app.kubernetes.io/name: lumen
    app.kubernetes.io/instance: ${LUMEN_CR_NAME}
    app.kubernetes.io/component: server
  ports:
    - name: http
      port: ${PORT_REMOTE}
      targetPort: http
      protocol: TCP
      nodePort: ${NODE_PORT}
EOF
  else
    kubectl -n "$NAMESPACE" patch svc lumen --type merge -p \
      "{\"spec\":{\"type\":\"NodePort\",\"ports\":[{\"name\":\"http\",\"port\":${PORT_REMOTE},\"targetPort\":\"http\",\"protocol\":\"TCP\",\"nodePort\":${NODE_PORT}}]}}"
  fi
  echo "   waiting for http://127.0.0.1:${PORT_LOCAL}/healthz"
  for _ in $(seq 1 60); do
    if curl -fsS --max-time 5 "http://127.0.0.1:${PORT_LOCAL}/healthz" >/dev/null 2>&1; then
      return 0
    fi
    sleep 1
  done
  echo "!! lumen API never became reachable on :${PORT_LOCAL}" >&2
  return 1
}

base_url() {
  echo "http://127.0.0.1:${PORT_LOCAL}"
}

api_put_collection() {
  curl -fsS --max-time 30 -X PUT "$(base_url)/collections/users" \
    -H 'content-type: application/json' \
    -d '{"fields": {"bio": {"type": "text"}, "email": {"type": "keyword"}}}'
}

api_index_batch() {
  local body_file="$1"
  # Generous --max-time: the bulk batch is ~1MB and the apply path is
  # synchronous, but a hung request should fail visibly, not block forever.
  curl -fsS --max-time 120 -X POST "$(base_url)/collections/users/index" \
    -H 'content-type: application/json' \
    --data-binary "@${body_file}"
}

api_search() {
  curl -fsS --max-time 30 -X POST "$(base_url)/collections/users/search" \
    -H 'content-type: application/json' \
    -d '{"query": {"match": {"field": "bio", "text": "engineer"}}, "limit": 20}'
}

api_duplicates() {
  curl -fsS --max-time 30 -X POST "$(base_url)/collections/users/duplicates" \
    -H 'content-type: application/json' \
    -d '{"field": "email", "min_group_size": 2, "limit": 100}'
}

# Index a single distinctive doc + search for it — used to prove writes resume
# after the broker recovers (the apply loop must re-subscribe).
api_index_probe() {
  curl -fsS --max-time 30 -X POST "$(base_url)/collections/users/index" \
    -H 'content-type: application/json' \
    -d '{"items":[{"external_id":"broker-probe","field":"email","value":"broker-probe@x.com"}]}'
}
api_search_probe() {
  curl -fsS --max-time 30 -X POST "$(base_url)/collections/users/search" \
    -H 'content-type: application/json' \
    -d '{"query":{"term":{"field":"email","value":"broker-probe@x.com"}},"limit":5}'
}

# Max restartCount across serving pods (0 ⇒ no crashloop).
server_restarts() {
  kubectl -n "$NAMESPACE" get pods -l "$APP_LABEL" \
    -o jsonpath='{range .items[*]}{.status.containerStatuses[0].restartCount}{"\n"}{end}' \
    | sort -nr | head -1
}

# ---------------------------------------------------------------------------
# 1. Create kind cluster
# ---------------------------------------------------------------------------

step "1. create kind cluster '$CLUSTER_NAME' (host :${PORT_LOCAL} → node :${NODE_PORT})" \
  create_cluster

# ---------------------------------------------------------------------------
# 1b. Build the lumen image and load it into the kind node
# ---------------------------------------------------------------------------

step "1b. docker build ${IMAGE_TAG} + kind load" build_and_load_image

# ---------------------------------------------------------------------------
# 2. Deploy lumen (overlay manifests, or the Lumen CRD + operator)
# ---------------------------------------------------------------------------

step "2. deploy lumen (mode=${E2E_MODE})" deploy_lumen

# ---------------------------------------------------------------------------
# 3. Wait for pod Ready
# ---------------------------------------------------------------------------

step "3a. wait for Relay broker Ready" wait_broker_ready 180
step "3b. wait for serving pods Ready" wait_lumen_ready 240

# ---------------------------------------------------------------------------
# 4. Drive the public HTTP API
# ---------------------------------------------------------------------------

step "4a. expose lumen on NodePort :${NODE_PORT} → host :${PORT_LOCAL}" expose_nodeport

step "4b. PUT /collections/users" api_put_collection

FIXTURE_FILE="$(mktemp -t lumen-fixture.XXXXXX.json)"
# Each doc emits 2 IndexItems (bio + email); split the request bodies so no
# single POST exceeds the server's bulk-index cap (MAX_INDEX_ITEMS=10000).
step "4c. generate ${DOC_COUNT}-doc fixture (batched ≤${BATCH_SIZE} items/req)" \
  python3 "${SCRIPT_DIR}/load-fixture.py" \
    --count "$DOC_COUNT" \
    --items-per-batch "$BATCH_SIZE" \
    --output "$FIXTURE_FILE"

# The fixture script emits one NDJSON doc per line *and* one or more batched
# IndexRequest bodies (<fixture>.req.000.json, .001.json, …). POST each in
# order — a real bulk client batches within the per-request item cap.
shopt -s nullglob
INDEX_BODIES=( "${FIXTURE_FILE%.json}".req.*.json )
shopt -u nullglob
if [[ ${#INDEX_BODIES[@]} -eq 0 ]]; then
  echo "!! fixture script emitted no request bodies" >&2
  exit 1
fi

index_all_batches() {
  local n=0
  for body in "${INDEX_BODIES[@]}"; do
    n=$((n + 1))
    echo "   batch ${n}/${#INDEX_BODIES[@]}: $(basename "$body")"
    api_index_batch "$body"
  done
}
step "4d. POST /collections/users/index (${DOC_COUNT} docs, ${#INDEX_BODIES[@]} batches)" \
  index_all_batches

SEARCH_RESP="$(api_search)"
SEARCH_HITS="$(echo "$SEARCH_RESP" | jq '.hits | length')"
echo "   search hits: $SEARCH_HITS"
if [[ "$SEARCH_HITS" -le 0 ]]; then
  echo "!! expected search hits > 0, got $SEARCH_HITS" >&2
  echo "   raw: $SEARCH_RESP" >&2
  exit 1
fi

DUP_RESP="$(api_duplicates)"
DUP_GROUPS="$(echo "$DUP_RESP" | jq '.groups | length')"
echo "   duplicate groups: $DUP_GROUPS"
if [[ "$DUP_GROUPS" -le 0 ]]; then
  echo "!! expected at least one duplicate group, got $DUP_GROUPS" >&2
  echo "   raw: $DUP_RESP" >&2
  exit 1
fi

# Snapshot the search response so we can compare after leader kill.
SEARCH_BEFORE="$(echo "$SEARCH_RESP" | jq -S '{hits: .hits, total: .total}')"

# ---------------------------------------------------------------------------
# 5. Kill all SERVING pods (not the broker). New pods must rebuild their
#    index by tailing the Relay log — proving the data lives in the log,
#    not in any serving pod.
# ---------------------------------------------------------------------------

step "5. kubectl delete pod -l $APP_LABEL (serving-pod kill; broker survives)" \
  kubectl -n "$NAMESPACE" delete pod -l "$APP_LABEL" --wait=false

# ---------------------------------------------------------------------------
# 6. Wait for the replacement serving pods to come back and catch up
# ---------------------------------------------------------------------------

step "6. wait for serving pods Ready (post kill, rebuilt from log)" wait_lumen_ready 240

# The NodePort mapping survives pod churn, but the new Service endpoints need a
# moment to register; re-confirm the API is reachable before the compare.
step "6b. re-confirm API reachable post-recovery" expose_nodeport

# ---------------------------------------------------------------------------
# 7. Re-run search and assert identical (index rebuilt from the Relay log)
# ---------------------------------------------------------------------------

SEARCH_AFTER_RAW="$(api_search)"
SEARCH_AFTER="$(echo "$SEARCH_AFTER_RAW" | jq -S '{hits: .hits, total: .total}')"

if [[ "$SEARCH_BEFORE" != "$SEARCH_AFTER" ]]; then
  echo "!! search results diverged after serving-pod rebuild" >&2
  echo "   before: $SEARCH_BEFORE" >&2
  echo "   after:  $SEARCH_AFTER" >&2
  exit 1
fi

echo ">> 7. search identical after rebuild-from-log — PASS"

# ---------------------------------------------------------------------------
# 8. Broker-kill chaos. Delete the Relay pod: serving nodes must NOT crash
#    (they reconnect + the apply loop re-subscribes from its applied seq),
#    writes must resume once the broker is back (durable Relay log on the PVC),
#    and the pre-kill data must be intact.
# ---------------------------------------------------------------------------

RESTARTS_BEFORE="$(server_restarts)"
BROKER_UID_BEFORE="$(kubectl -n "$NAMESPACE" get pod lumen-relay-0 -o jsonpath='{.metadata.uid}' 2>/dev/null || true)"
step "8. kubectl delete pod -l $BROKER_LABEL (broker kill)" \
  kubectl -n "$NAMESPACE" delete pod -l "$BROKER_LABEL" --wait=false

step "9. wait for Relay broker recreated + Ready (durable PVC)" \
  wait_broker_recovered "$BROKER_UID_BEFORE" 180

RESTARTS_AFTER="$(server_restarts)"
if [[ "${RESTARTS_AFTER:-0}" -gt "${RESTARTS_BEFORE:-0}" ]]; then
  echo "!! serving pods crashed during broker outage (restarts ${RESTARTS_BEFORE} -> ${RESTARTS_AFTER})" >&2
  exit 1
fi
echo ">> 9. serving pods survived the broker outage (max restarts=${RESTARTS_AFTER})"

step "9b. re-confirm API reachable post-broker-recovery" expose_nodeport

# Writes must apply again (the apply loop re-subscribed to the new consumer).
step "10. index a probe doc after broker recovery" api_index_probe
PROBE_HITS="$(api_search_probe | jq '.hits | length')"
echo "   probe hits: $PROBE_HITS"
if [[ "${PROBE_HITS:-0}" -lt 1 ]]; then
  echo "!! probe write never applied — writes wedged after broker recovery" >&2
  exit 1
fi

# Pre-kill data must be intact + search still identical.
SEARCH_FINAL="$(api_search | jq -S '{hits: .hits, total: .total}')"
if [[ "$SEARCH_BEFORE" != "$SEARCH_FINAL" ]]; then
  echo "!! search diverged after broker kill" >&2
  echo "   before: $SEARCH_BEFORE" >&2
  echo "   final:  $SEARCH_FINAL" >&2
  exit 1
fi
echo ">> 10. writes resumed + data intact after broker kill — PASS"

# ---------------------------------------------------------------------------
# 11. Cleanup happens via the trap.
# ---------------------------------------------------------------------------

echo ">> kind-e2e PASS"
