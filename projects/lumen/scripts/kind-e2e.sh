#!/usr/bin/env bash
# lumen — kind-based end-to-end happy-path test.
#
# Implements the Lumen-only kind happy path: spin up a single-node kind
# cluster, apply the `dev` overlay, force the serving Deployment to use
# embedded WAL (no Relay broker dependency), drive the public HTTP API (:7373)
# through schema → index 10k → search → duplicates, then kill all serving pods
# and verify the replacement pod becomes reachable and accepts fresh writes.
#
# Usage:  scripts/kind-e2e.sh
#         LUMEN_E2E_MODE=operator scripts/kind-e2e.sh   # deploy via the CRD
# Exit code 0 = success; any assertion failure exits non-zero.
#
# Deploy modes (LUMEN_E2E_MODE):
#   overlay  (default) — kubectl apply -k k8s/overlays/dev (hand-written manifests)
#   operator           — install the Lumen CRD + operator, then apply a Lumen CR
#                        and let the operator reconcile the serving fleet.
# Both exercise the identical Lumen-only API → restart → fresh-write path.
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
# `lumen` → Deployment `lumen`), so only the label selectors and Service
# handling differ between modes.
E2E_MODE="${LUMEN_E2E_MODE:-overlay}"
OPERATOR_NS="lumen-system"
LUMEN_CR_NAME="lumen"
if [[ "$E2E_MODE" == "operator" ]]; then
  APP_LABEL="app.kubernetes.io/name=lumen,app.kubernetes.io/component=server"
else
  APP_LABEL="app=lumen,role=server"
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

wait_lumen_ready() {
  local timeout="${1:-180}"
  echo "   waiting up to ${timeout}s for serving pods ($APP_LABEL) Ready"
  wait_pods_exist "$APP_LABEL" "$timeout"
  kubectl -n "$NAMESPACE" wait --for=condition=Ready pod -l "$APP_LABEL" \
    --timeout="${timeout}s"
}

# Build the Lumen image and load it into the kind node.
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
# serving Deployment. The Deployment is pinned to embedded WAL by
# configure_lumen_only_deployment below so this gate remains single-node and
# service-only.
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
EOF

  echo "   Lumen/${LUMEN_CR_NAME} applied; waiting for the operator to render child objects"
  local deadline=$(( $(date +%s) + 60 ))
  while [[ $(date +%s) -lt $deadline ]]; do
    if kubectl -n "$NAMESPACE" get deploy/"${LUMEN_CR_NAME}" >/dev/null 2>&1; then
      echo "   operator reconciled Deployment/${LUMEN_CR_NAME}"
      return 0
    fi
    sleep 2
  done
  echo "!! operator did not render Deployment/${LUMEN_CR_NAME} within 60s" >&2
  kubectl -n "$OPERATOR_NS" logs deploy/lumen-operator --tail=60 >&2 || true
  return 1
}

configure_lumen_only_deployment() {
  # This dogfood gate is intentionally Lumen-only and single-node.
  kubectl -n "$NAMESPACE" set env deploy/"${LUMEN_CR_NAME}" \
    LUMEN_WAL=embedded

  kubectl -n "$NAMESPACE" rollout status deploy/"${LUMEN_CR_NAME}" --timeout=180s
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

# Index a single distinctive doc + search for it after the serving pod restart.
api_index_probe() {
  curl -fsS --max-time 30 -X POST "$(base_url)/collections/users/index" \
    -H 'content-type: application/json' \
    -d '{"items":[{"external_id":"restart-probe","field":"email","value":"restart-probe@x.com"}]}'
}
api_search_probe() {
  curl -fsS --max-time 30 -X POST "$(base_url)/collections/users/search" \
    -H 'content-type: application/json' \
    -d '{"query":{"term":{"field":"email","value":"restart-probe@x.com"}},"limit":5}'
}

# ---------------------------------------------------------------------------
# 1. Create kind cluster
# ---------------------------------------------------------------------------

step "1. create kind cluster '$CLUSTER_NAME' (host :${PORT_LOCAL} → node :${NODE_PORT})" \
  create_cluster

# ---------------------------------------------------------------------------
# 1b. Build the Lumen image and load it into the kind node
# ---------------------------------------------------------------------------

step "1b. docker build ${IMAGE_TAG} + kind load" build_and_load_image

# ---------------------------------------------------------------------------
# 2. Deploy lumen (overlay manifests, or the Lumen CRD + operator)
# ---------------------------------------------------------------------------

step "2. deploy lumen (mode=${E2E_MODE})" deploy_lumen

step "2b. configure Lumen-only embedded WAL (no Relay dependency)" \
  configure_lumen_only_deployment

# ---------------------------------------------------------------------------
# 3. Wait for pod Ready
# ---------------------------------------------------------------------------

step "3. wait for serving pods Ready" wait_lumen_ready 240

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

# ---------------------------------------------------------------------------
# 5. Kill all serving pods. The replacement pod uses embedded WAL, so this
#    proves k8s rollout/recovery and fresh-write readiness for the single-node
#    dogfood path.
# ---------------------------------------------------------------------------

step "5. kubectl delete pod -l $APP_LABEL (serving-pod restart)" \
  kubectl -n "$NAMESPACE" delete pod -l "$APP_LABEL" --wait=false

# ---------------------------------------------------------------------------
# 6. Wait for the replacement serving pods to come back and catch up
# ---------------------------------------------------------------------------

step "6. wait for serving pods Ready (post restart)" wait_lumen_ready 240

# The NodePort mapping survives pod churn, but the new Service endpoints need a
# moment to register; re-confirm the API is reachable before fresh writes.
step "6b. re-confirm API reachable post-recovery" expose_nodeport

# ---------------------------------------------------------------------------
# 7. Re-create the collection and assert fresh writes work after restart.
# ---------------------------------------------------------------------------

step "7a. PUT /collections/users after restart" api_put_collection
step "7b. index a probe doc after restart" api_index_probe
PROBE_HITS="$(api_search_probe | jq '.hits | length')"
echo "   probe hits: $PROBE_HITS"
if [[ "${PROBE_HITS:-0}" -lt 1 ]]; then
  echo "!! probe write never applied after serving-pod restart" >&2
  exit 1
fi

echo ">> 7. serving restart recovered + fresh writes work — PASS"

# ---------------------------------------------------------------------------
# 11. Cleanup happens via the trap.
# ---------------------------------------------------------------------------

echo ">> kind-e2e PASS"
