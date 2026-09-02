#!/usr/bin/env bash
# HANDWRITE-BEGIN gap="missing-generator:e2e-test:kind-service-journey" tracker="1646" reason="The live kind cluster/image/API/restart journey remains hand-authored until the shared stateful-service EC template can generate app-specific fixture and endpoint steps."
# lumen — kind-based end-to-end happy-path test.
#
# Implements the Lumen-only kind happy path: spin up a single-node kind
# cluster, apply the `dev` overlay or operator CR, keep the serving fleet
# on the binary's WAL auto-selection, drive the public HTTP API (:7373)
# through schema → index 10k → search → duplicates. It then checkpoints one
# unique document, replaces the serving pod, reads the old document before any
# new write, and finally proves that the replacement accepts a fresh write.
#
# Usage:  scripts/kind-e2e.sh
#         LUMEN_E2E_MODE=operator scripts/kind-e2e.sh   # deploy via the CRD
# Exit code 0 = success; any assertion failure exits non-zero.
#
# Deploy modes (LUMEN_E2E_MODE):
#   overlay  (default) — kubectl apply -k k8s/overlays/dev (hand-written manifests)
#   operator           — install the Lumen CRD + operator, then apply a Lumen CR
#                        and let the operator reconcile the serving fleet.
# Both exercise the identical Lumen-only API → restart → fresh-write path. In
# operator mode, LUMEN_E2E_SHARD_COUNT and LUMEN_E2E_REPLICAS_PER_SHARD also
# assert the StatefulSet storage topology.
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
# `lumen` → StatefulSet `lumen`), so only the label selectors and Service
# handling differ between modes.
E2E_MODE="${LUMEN_E2E_MODE:-overlay}"
OPERATOR_NS="lumen-system"
LUMEN_CR_NAME="lumen"
SHARD_COUNT="${LUMEN_E2E_SHARD_COUNT:-1}"
REPLICAS_PER_SHARD="${LUMEN_E2E_REPLICAS_PER_SHARD:-1}"
VOTER_COUNT="${LUMEN_E2E_VOTER_COUNT:-$REPLICAS_PER_SHARD}"
EXPECTED_STORAGE_PODS=$((SHARD_COUNT * REPLICAS_PER_SHARD))
SERVING_CPU="${LUMEN_E2E_SERVING_CPU:-250m}"
SERVING_MEMORY="${LUMEN_E2E_SERVING_MEMORY:-512Mi}"
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
BATCH_SIZE=1000

FIXTURE_FILE=""
INDEX_BODIES=()
IMAGE_TAG="${LUMEN_E2E_IMAGE:-lumen:latest}"
IMAGE_MODE="${LUMEN_E2E_IMAGE_MODE:-local}"

die() {
  echo "!! $*" >&2
  exit 1
}

if [[ "$IMAGE_MODE" == "prebuilt" ]]; then
  [[ "$E2E_MODE" == "operator" ]] || die "prebuilt image mode requires LUMEN_E2E_MODE=operator"
  [[ -n "${LUMEN_E2E_IMAGE:-}" ]] || die "missing LUMEN_E2E_IMAGE"
  [[ ! "$IMAGE_TAG" =~ [[:space:][:cntrl:]] ]] || die "LUMEN_E2E_IMAGE contains whitespace or control characters"
  [[ "$IMAGE_TAG" =~ ^ghcr\.io/chrischeng-c4/lumen@sha256:[0-9a-f]{64}$ ]] || die "invalid LUMEN_E2E_IMAGE: expected ghcr.io/chrischeng-c4/lumen@sha256:<64 hex>"
  [[ "${IMAGE_TAG#*@}" != *"@"* ]] || die "multiple @ in LUMEN_E2E_IMAGE"
  cargo_ver="$(grep '^version = ' "$LUMEN_DIR/Cargo.toml" | head -n1 | cut -d'"' -f2)"
  EXPECTED_VER="${LUMEN_E2E_EXPECTED_VERSION:-}"
  [[ -n "$EXPECTED_VER" && "$EXPECTED_VER" == "$cargo_ver" ]] || die "LUMEN_E2E_EXPECTED_VERSION mismatch (expected $cargo_ver)"
  EXPECTED_GIT_SHA="${LUMEN_E2E_EXPECTED_GIT_SHA:-}"
  [[ "$EXPECTED_GIT_SHA" =~ ^[0-9a-f]{8}$ ]] || die "invalid LUMEN_E2E_EXPECTED_GIT_SHA: expected 8 lowercase hex"
  EXPECTED_RUNTIME_DIGEST="${LUMEN_E2E_EXPECTED_RUNTIME_DIGEST:-}"
  [[ "$EXPECTED_RUNTIME_DIGEST" =~ ^sha256:[0-9a-f]{64}$ ]] || die "invalid LUMEN_E2E_EXPECTED_RUNTIME_DIGEST: expected sha256:<64 hex>"
  ROOT_DIGEST="${IMAGE_TAG##*@}"
  [[ "$EXPECTED_RUNTIME_DIGEST" != "$ROOT_DIGEST" ]] || die "runtime child digest must differ from root index digest"
elif [[ "$IMAGE_MODE" != "local" ]]; then
  die "invalid LUMEN_E2E_IMAGE_MODE: $IMAGE_MODE (must be local or prebuilt)"
fi

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

cleanup_fixture_files() {
  local fixture="$1"
  if [[ -n "$fixture" ]]; then
    rm -f "$fixture" "${fixture%.json}".req.*.json
  fi
}

cleanup() {
  local ec=$?
  cleanup_fixture_files "$FIXTURE_FILE"
  if [[ "${LUMEN_KEEP_CLUSTER:-0}" != "1" ]]; then
    kind delete cluster --name "$CLUSTER_NAME" >/dev/null 2>&1 || true
  fi
  exit "$ec"
}
trap cleanup EXIT INT TERM

# ---------------------------------------------------------------------------
# Helpers
# ---------------------------------------------------------------------------

discover_fixture_bodies() {
  shopt -s nullglob
  INDEX_BODIES=( "${FIXTURE_FILE%.json}".req.*.json )
  shopt -u nullglob
}

# Poll until ≥1 pod matches the label (so a subsequent `kubectl wait` does not
# error with "no matching resources" — the operator creates the workload a beat
# after the CR is applied).
wait_pods_exist() {
  local label="$1" expected="${2:-1}" timeout="${3:-60}" deadline
  deadline=$(( $(date +%s) + timeout ))
  while [[ $(date +%s) -lt $deadline ]]; do
    if [[ "$(kubectl -n "$NAMESPACE" get pod -l "$label" --no-headers 2>/dev/null | wc -l | tr -d ' ')" -ge "$expected" ]]; then
      return 0
    fi
    sleep 2
  done
  echo "!! expected at least ${expected} pod(s) for ${label} within ${timeout}s" >&2
  kubectl -n "$NAMESPACE" get pod -l "$label" >&2 || true
  return 1
}

wait_lumen_ready() {
  local timeout="${1:-180}"
  echo "   waiting up to ${timeout}s for serving pods ($APP_LABEL) Ready"
  wait_pods_exist "$APP_LABEL" "$EXPECTED_STORAGE_PODS" "$timeout"
  kubectl -n "$NAMESPACE" wait --for=condition=Ready pod -l "$APP_LABEL" \
    --timeout="${timeout}s"
}

# Build the Lumen image and load it into the kind node.
#
# Built from the WORKSPACE ROOT as context (the same pattern as
# apps/lumen/compose.yaml and conductor's CI): cargo resolves the whole
# workspace and `cargo build -p lumen` in the Dockerfile compiles only lumen's
# real dependency closure. The repo-root .dockerignore keeps the context to
# source-only (~MBs, not the 35G of target/). The deployment pins
# imagePullPolicy: IfNotPresent, so once `kind load` has injected the image
# into the node it is used without a registry pull -- but only for a container
# that actually names $IMAGE_TAG. Every deploy path below has to repoint its
# workload at it; `kind load` alone does not, because the manifests ship the
# released tag and IfNotPresent matches on the whole reference, not the digest.
build_and_load_image() {
  if [[ "$IMAGE_MODE" == "prebuilt" ]]; then
    echo "   prebuilt mode: skipping docker build and kind load for $IMAGE_TAG"
    return 0
  fi
  docker build -f "$LUMEN_DIR/Dockerfile" -t "$IMAGE_TAG" "$REPO_ROOT"
  kind load docker-image "$IMAGE_TAG" --name "$CLUSTER_NAME"
}

# Deploy lumen by the selected mode.
deploy_lumen() {
  if [[ "$E2E_MODE" == "operator" ]]; then
    deploy_via_operator
  else
    kubectl apply -k "${LUMEN_DIR}/k8s/overlays/dev"
    # The overlay inherits base/deployment.yaml verbatim, and that manifest
    # pins the released `ghcr.io/chrischeng-c4/lumen:<version>`. `kind load`
    # injected the build under test under a different name, so IfNotPresent
    # matched nothing and the node pulled the PREVIOUS RELEASE from GHCR --
    # a run that looked green while serving code this branch never built.
    # Repoint the Deployment the same way the local operator path does.
    kubectl -n "$NAMESPACE" set image deploy/lumen lumen="$IMAGE_TAG"
    assert_serving_deployment_runs_image_under_test
  fi
}

# `assert_cluster_identity` returns early unless IMAGE_MODE=prebuilt, so in the
# default local mode nothing else observes which image the serving workload
# actually names. Without this row the substitution above can silently stop
# applying -- a renamed container, a reordered patch -- and the run still reads
# as a pass.
assert_serving_deployment_runs_image_under_test() {
  local observed
  observed="$(kubectl -n "$NAMESPACE" get deploy/lumen \
    -o jsonpath='{.spec.template.spec.containers[?(@.name=="lumen")].image}')"
  [[ "$observed" == "$IMAGE_TAG" ]] || \
    die "serving Deployment runs '$observed', not the image under test $IMAGE_TAG"
}

# Operator path: install the CRD + RBAC + operator (same image as serving),
# then apply a dev-shaped Lumen CR and let the reconcile loop materialize the
# serving StatefulSet. WAL remains `auto`: one replica per shard uses embedded
# WAL, while replicated shards use raft when the image includes raft-wal.
deploy_via_operator() {
  if [[ "$IMAGE_MODE" == "prebuilt" ]]; then
    local tmp_op tmp_pinned mutable_image
    tmp_op="$(mktemp -t lumen-op.XXXXXX.yaml)"
    tmp_pinned="${tmp_op}.pinned"
    mutable_image="ghcr.io/chrischeng-c4/lumen:${cargo_ver}"
    kubectl kustomize "${LUMEN_DIR}/k8s/operator" > "$tmp_op"
    awk -v old="$mutable_image" -v new="$IMAGE_TAG" '
      function finish_document() {
        if (api_version == "apps/v1" && kind == "Deployment" && object_name == "lumen-operator" && object_namespace == "lumen-system") {
          deployments++
          if (containers != 1 || operator_names != 1 || old_images != 1) invalid = 1
        }
      }
      function reset_document() {
        api_version = ""; kind = ""; object_name = ""; object_namespace = ""
        in_metadata = 0; in_containers = 0
        containers = 0; operator_names = 0; old_images = 0
      }
      BEGIN { reset_document() }
      /^---$/ { finish_document(); reset_document(); print; next }
      /^apiVersion: / { api_version = $2 }
      /^kind: / { kind = $2 }
      /^metadata:$/ { in_metadata = 1; print; next }
      in_metadata && /^  name: / { object_name = $2 }
      in_metadata && /^  namespace: / { object_namespace = $2 }
      /^spec:$/ { in_metadata = 0 }
      in_containers && /^      [A-Za-z]/ { in_containers = 0 }
      api_version == "apps/v1" && kind == "Deployment" && object_name == "lumen-operator" && object_namespace == "lumen-system" && /^      containers:$/ { in_containers = 1 }
      in_containers && /^      - / { containers++ }
      in_containers && /^        name: operator$/ { operator_names++ }
      in_containers && $0 == "        image: " old {
          print "        image: " new
          old_images++
          changed++
          next
      }
      { print }
      END {
        finish_document()
        if (deployments != 1 || changed != 1 || invalid) exit 42
      }
    ' "$tmp_op" > "$tmp_pinned" || die "operator manifest did not contain exactly one expected mutable image scalar"
    awk -v expected="$IMAGE_TAG" '
      /^[[:space:]]*image:[[:space:]]+/ {
        image = $2
        if (image == expected) seen++
        if (image ~ /(^|\/)lumen([:@]|$)/ && image != expected) bad++
      }
      END { exit bad == 0 && seen == 1 ? 0 : 1 }
    ' "$tmp_pinned" || die "mutable Lumen image remains in rendered operator manifest"
    kubectl apply -f "$tmp_pinned"
    rm -f "$tmp_op" "$tmp_pinned"
  else
    kubectl apply -k "${LUMEN_DIR}/k8s/operator"
    kubectl -n "$OPERATOR_NS" set image deploy/lumen-operator operator="$IMAGE_TAG"
  fi
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
  shardCount: ${SHARD_COUNT}
  replicasPerShard: ${REPLICAS_PER_SHARD}
  voterCount: ${VOTER_COUNT}
  logFormat: pretty
  # #2678: auth defaults to required, so this local-only rig opts out
  # explicitly rather than shipping a data plane that never goes Ready.
  auth: disabled
  # Native compatibility placement: the default machine type plus this exact
  # selector must reconcile without the legacy capacity catalog.
  placement:
    nodeSelector:
      kubernetes.io/os: linux
  serving:
    cpu: "${SERVING_CPU}"
    memory: "${SERVING_MEMORY}"
EOF

  echo "   Lumen/${LUMEN_CR_NAME} applied; waiting for the operator to render child objects"
  local deadline=$(( $(date +%s) + 60 ))
  while [[ $(date +%s) -lt $deadline ]]; do
    if kubectl -n "$NAMESPACE" get statefulset/"${LUMEN_CR_NAME}" >/dev/null 2>&1; then
      echo "   operator reconciled StatefulSet/${LUMEN_CR_NAME}"
      assert_native_placement
      assert_legacy_missing_catalog_event
      return 0
    fi
    sleep 2
  done
  echo "!! operator did not render StatefulSet/${LUMEN_CR_NAME} within 60s" >&2
  kubectl -n "$OPERATOR_NS" logs deploy/lumen-operator --tail=60 >&2 || true
  return 1
}

assert_native_placement() {
  if [[ "$E2E_MODE" != "operator" ]]; then
    return 0
  fi
  local selector
  selector="$(kubectl -n "$NAMESPACE" get statefulset/"${LUMEN_CR_NAME}" -o json | jq -er '.spec.template.spec.nodeSelector["kubernetes.io/os"]')" \
    || die "native placement StatefulSet has no kubernetes.io/os selector"
  [[ "$selector" == "linux" ]] \
    || die "native placement selector changed: expected kubernetes.io/os=linux, got $selector"
  if kubectl -n "$OPERATOR_NS" get configmap/lumen-capacity-catalog >/dev/null 2>&1; then
    die "native placement unexpectedly depends on a fake capacity catalog"
  fi
  echo "   native placement preserved kubernetes.io/os=linux without a capacity catalog"
}

assert_legacy_missing_catalog_event() {
  if [[ "$E2E_MODE" != "operator" ]]; then
    return 0
  fi
  local legacy_namespace="${NAMESPACE}-legacy"
  local legacy_name="${LUMEN_CR_NAME}-legacy-missing-catalog"
  kubectl create namespace "$legacy_namespace" --dry-run=client -o yaml | kubectl apply -f -
  kubectl -n "$legacy_namespace" apply -f - <<EOF
apiVersion: lumen.dev/v1alpha1
kind: Lumen
metadata:
  name: ${legacy_name}
  namespace: ${legacy_namespace}
spec:
  image: ${IMAGE_TAG}
  imagePullPolicy: IfNotPresent
  shardCount: 1
  replicasPerShard: 1
  voterCount: 1
  logFormat: pretty
  auth: disabled
  serving:
    cpu: "1"
    memory: "1Gi"
EOF

  local deadline=$(( $(date +%s) + 60 ))
  while [[ $(date +%s) -lt $deadline ]]; do
    local message
    message="$(kubectl -n "$legacy_namespace" get events \
      --field-selector "involvedObject.name=${legacy_name},reason=ReconcileFailed" \
      -o json | jq -r '[.items[] | (.message // .note // "")] | join("\n")')"
    if [[ "$message" == *"capacity catalog"* ]]; then
      echo "   legacy empty-selector failure Event names missing capacity catalog"
      return 0
    fi
    sleep 2
  done
  kubectl -n "$legacy_namespace" get events --sort-by=.lastTimestamp >&2 || true
  die "legacy empty-selector CR did not publish a namespaced ReconcileFailed Event naming the capacity catalog"
}

configure_lumen_only_deployment() {
  if [[ "$E2E_MODE" == "operator" ]]; then
    echo "   operator mode: keeping LUMEN_WAL=auto from the rendered StatefulSet"
    return 0
  fi

  # The hand-written overlay is the legacy single-node dogfood path.
  # Shipped manifests intentionally default to the user-tunable 1 CPU / 4Gi
  # request-only baseline. A single-node kind cluster is a constrained test
  # fixture, so make its smaller footprint explicit instead of weakening the
  # production default or relying on whatever capacity the local VM exposes.
  kubectl -n "$NAMESPACE" set resources deploy/"${LUMEN_CR_NAME}" \
    --requests="cpu=${SERVING_CPU},memory=${SERVING_MEMORY}"
  kubectl -n "$NAMESPACE" set env deploy/"${LUMEN_CR_NAME}" LUMEN_WAL=embedded
  kubectl -n "$NAMESPACE" rollout status deploy/"${LUMEN_CR_NAME}" --timeout=180s
}

assert_operator_topology() {
  if [[ "$E2E_MODE" != "operator" ]]; then
    return 0
  fi
  local stateful_json actual_replicas
  stateful_json="$(kubectl -n "$NAMESPACE" get statefulset/"${LUMEN_CR_NAME}" -o json)"
  actual_replicas="$(jq '.spec.replicas' <<<"$stateful_json")"
  echo "   StatefulSet replicas: ${actual_replicas} (expected ${EXPECTED_STORAGE_PODS})"
  if [[ "$actual_replicas" -ne "$EXPECTED_STORAGE_PODS" ]]; then
    echo "!! expected StatefulSet/${LUMEN_CR_NAME} replicas=${EXPECTED_STORAGE_PODS}, got ${actual_replicas}" >&2
    exit 1
  fi
  if [[ "$SHARD_COUNT" -gt 1 || "$REPLICAS_PER_SHARD" -gt 1 ]]; then
    if kubectl -n "$NAMESPACE" get hpa/"${LUMEN_CR_NAME}" >/dev/null 2>&1; then
      echo "!! HPA must not own storage topology when shardCount=${SHARD_COUNT} replicasPerShard=${REPLICAS_PER_SHARD}" >&2
      exit 1
    fi
  fi
  jq -e '
    (.spec.volumeClaimTemplates | length) == 1 and
    .spec.volumeClaimTemplates[0].metadata.name == "raft" and
    ([.spec.template.spec.containers[] | select(.name == "server")] | length) == 1
  ' <<<"$stateful_json" >/dev/null \
    || die "operator StatefulSet must render exactly one raft PVC and one server container"
  if (( REPLICAS_PER_SHARD <= 1 )); then
    jq -e '
      ([.spec.template.spec.containers[] | select(.name == "server") | .volumeMounts[] | select(.name == "raft")] |
        map(if has("readOnly") then . else . + {"readOnly": false} end)) == [
        {"mountPath":"/var/lib/lumen","name":"raft","readOnly":false},
        {"mountPath":"/var/lib/lumen/data","name":"raft","readOnly":false,"subPath":"data"}
      ] and
      ([.spec.template.spec.containers[] | select(.name == "server") | .env[] | select(.name == "LUMEN_DATA_DIR" and .value == "/var/lib/lumen/data")] | length) == 1 and
      ([.spec.template.spec.containers[] | select(.name == "server") | .env[] | select(.name == "LUMEN_PERSISTENCE" and .value == "segment")] | length) == 1
    ' <<<"$stateful_json" >/dev/null \
      || die "single-replica operator StatefulSet lost the exact raft parent/data child mount contract"
  else
    jq -e '
      ([.spec.template.spec.containers[] | select(.name == "server") | .volumeMounts[] | select(.name == "raft")] |
        map(if has("readOnly") then . else . + {"readOnly": false} end)) == [
        {"mountPath":"/var/lib/lumen","name":"raft","readOnly":false}
      ] and
      ([.spec.template.spec.containers[] | select(.name == "server") | .env[] | select(.name == "LUMEN_DATA_DIR" or .name == "LUMEN_PERSISTENCE")] | length) == 0
    ' <<<"$stateful_json" >/dev/null \
      || die "replicated operator StatefulSet must not render embedded persistence mounts or env"
  fi
  local cm_shards cm_bucket_count cm_map_version
  cm_shards="$(kubectl -n "$NAMESPACE" get cm/"${LUMEN_CR_NAME}-config" -o json | jq -r '.data.SHARD_COUNT')"
  cm_bucket_count="$(kubectl -n "$NAMESPACE" get cm/"${LUMEN_CR_NAME}-config" -o json | jq -r '.data.VIRTUAL_BUCKET_COUNT')"
  cm_map_version="$(kubectl -n "$NAMESPACE" get cm/"${LUMEN_CR_NAME}-config" -o json | jq -r '.data.SHARD_MAP_VERSION')"
  echo "   ConfigMap topology: SHARD_COUNT=${cm_shards} VIRTUAL_BUCKET_COUNT=${cm_bucket_count} SHARD_MAP_VERSION=${cm_map_version}"
  if [[ "$cm_shards" != "$SHARD_COUNT" || "$cm_bucket_count" == "null" || "$cm_map_version" == "null" ]]; then
    echo "!! ConfigMap topology keys are missing or wrong" >&2
    exit 1
  fi
}

assert_operator_storage_live() {
  if [[ "$E2E_MODE" != "operator" || "$SHARD_COUNT" -ne 1 || "$REPLICAS_PER_SHARD" -ne 1 ]]; then
    return 0
  fi
  local pod_json pvc_json
  pod_json="$(kubectl -n "$NAMESPACE" get pod/"${LUMEN_CR_NAME}-0" -o json)"
  pvc_json="$(kubectl -n "$NAMESPACE" get pvc/"raft-${LUMEN_CR_NAME}-0" -o json)"
  jq -e '
    ([.spec.containers[] | select(.name == "server") | .volumeMounts[] | select(.name == "raft")] |
      map(if has("readOnly") then . else . + {"readOnly": false} end)) == [
      {"mountPath":"/var/lib/lumen","name":"raft","readOnly":false},
      {"mountPath":"/var/lib/lumen/data","name":"raft","readOnly":false,"subPath":"data"}
    ] and
    ([.spec.volumes[] | select(.name == "raft" and .persistentVolumeClaim.claimName == "raft-lumen-0")] | length) == 1
  ' <<<"$pod_json" >/dev/null || die "live serving pod lost the exact raft PVC parent/data child mounts"
  jq -e '
    .metadata.name == "raft-lumen-0" and
    .status.phase == "Bound" and
    (.spec.volumeName | type) == "string" and
    (.spec.volumeName | length) > 0
  ' <<<"$pvc_json" >/dev/null || die "raft-lumen-0 PVC is not bound"
  echo "   live pod mounts the bound raft-lumen-0 PVC at parent and exact data child paths"
}

# Create the kind cluster with a host→node port mapping so the host can
# reach the NodePort service directly (no port-forward tunnel to stall).
# One SCHEDULABLE node per storage pod. The operator renders a HARD pod
# anti-affinity -- `requiredDuringSchedulingIgnoredDuringExecution` on
# kubernetes.io/hostname, libs/service-k8s/src/render.rs:129 -- so two storage
# owners can never share a node. On a single-node cluster every
# EXPECTED_STORAGE_PODS>1 run therefore stalls at `wait_lumen_ready` with
# lumen-1 Pending and FailedScheduling, which reads as a product hang rather
# than as a cluster this harness built too small. Growing the cluster is the
# fix; relaxing the anti-affinity would delete the production rule the topology
# assertions exist to prove.
#
# Workers are EXPECTED_STORAGE_PODS, not EXPECTED_STORAGE_PODS-1, because the
# control-plane node does not count: kind drops the
# `node-role.kubernetes.io/control-plane:NoSchedule` taint only on a cluster
# that has no workers at all. Adding the first worker re-taints it, so a
# 2-shard run built as control-plane + 1 worker has exactly ONE schedulable
# node and fails identically to the single-node case it was meant to fix. Keep
# the zero-worker shape for EXPECTED_STORAGE_PODS=1 so that path stays on the
# untainted single node it already passes on.
create_cluster() {
  local workers=""
  local i
  if (( EXPECTED_STORAGE_PODS > 1 )); then
    for (( i = 0; i < EXPECTED_STORAGE_PODS; i++ )); do
      workers+="${workers:+$'\n'}  - role: worker"
    done
  fi
  kind create cluster --name "$CLUSTER_NAME" --wait 120s --config - <<EOF
kind: Cluster
apiVersion: kind.x-k8s.io/v1alpha4
nodes:
  - role: control-plane
    extraPortMappings:
      - containerPort: ${NODE_PORT}
        hostPort: ${PORT_LOCAL}
        protocol: TCP
${workers}
EOF
}

# Expose the lumen Service on a fixed NodePort that the host port mapping
# targets. The base overlay keeps the Service ClusterIP (NodePort is a
# kind-test concern); patch it here rather than in the shipped manifests.
expose_nodeport() {
  if [[ "$E2E_MODE" == "operator" ]]; then
    # The operator owns Service/lumen and would revert a NodePort patch on its
    # next reconcile. Use a SEPARATE, operator-untouched NodePort Service that
    # selects pod-0. The public service load-balances across storage owners;
    # this script is an HTTP/restart smoke after the topology assertions above,
    # so pinning one stable pod avoids cross-shard 404s between create/index.
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
    statefulset.kubernetes.io/pod-name: ${LUMEN_CR_NAME}-0
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

# Same request without --fail, so a refusal is a body to read rather than a bare
# `curl: (22)`. Prints the body, then the HTTP status on a final line of its own;
# curl's own diagnostics stay on stderr so they cannot corrupt that split.
api_duplicates_status() {
  curl -sS --max-time 30 -w '\n%{http_code}' \
    -X POST "$(base_url)/collections/users/duplicates" \
    -H 'content-type: application/json' \
    -d '{"field": "email", "min_group_size": 2, "limit": 100}'
}

# Index and search one exact document without embedding shell data into JSON.
api_index_exact() {
  local external_id="$1" value="$2"
  jq -nc --arg id "$external_id" --arg value "$value" \
    '{items:[{external_id:$id,field:"email",value:$value}]}' | \
    curl -fsS --max-time 30 -X POST "$(base_url)/collections/users/index" \
      -H 'content-type: application/json' --data-binary @-
}
api_search_exact() {
  local value="$1"
  local body
  body="$(jq -nc --arg value "$value" '{query:{term:{field:"email",value:$value}},limit:5}')"
  curl -fsS --max-time 30 -X POST "$(base_url)/collections/users/search" \
    -H 'content-type: application/json' \
    -d "$body"
}
api_checkpoint() {
  curl -fsS --max-time 120 -X POST "$(base_url)/admin/checkpoint" \
    -H 'content-type: application/json' -d '{}'
}

# This function is a release oracle. Keep its operation lines direct and in
# order so the static candidate test can reject comments, prose, dead branches,
# reordered checks, or ignored failures.
durable_restart_oracle() {
  local pre_id="pre-restart-${CLUSTER_NAME}-$$"
  local pre_value="${pre_id}@example.invalid"
  local post_id="post-restart-${CLUSTER_NAME}-$$"
  local post_value="${post_id}@example.invalid"
  local checkpoint old_hits post_hits
  api_index_exact "$pre_id" "$pre_value"
  checkpoint="$(api_checkpoint)"
  jq -e '.persisted == true' <<<"$checkpoint" >/dev/null || die "/admin/checkpoint did not return persisted=true"
  kubectl -n "$NAMESPACE" delete pod -l "$APP_LABEL" --wait=true
  wait_lumen_ready 240
  expose_nodeport
  assert_cluster_identity
  old_hits="$(api_search_exact "$pre_value" | jq --arg id "$pre_id" '[.hits[] | select(.external_id == $id)] | length')"
  [[ "$old_hits" -eq 1 ]] || die "pre-restart document was not readable before any new write"
  api_index_exact "$post_id" "$post_value"
  post_hits="$(api_search_exact "$post_value" | jq --arg id "$post_id" '[.hits[] | select(.external_id == $id)] | length')"
  [[ "$post_hits" -eq 1 ]] || die "replacement pod did not accept the post-restart write"
  echo "   durable restart preserved $pre_id and accepted $post_id"
}

# ---------------------------------------------------------------------------
# Prebuilt identity assertion helper
# ---------------------------------------------------------------------------

normalize_runtime_image_id() {
  local raw="$1"
  if [[ "$raw" =~ ^ghcr\.io/chrischeng-c4/lumen@(sha256:[0-9a-f]{64})$ ]]; then
    echo "${BASH_REMATCH[1]}"
  elif [[ "$raw" =~ ^docker-pullable://ghcr\.io/chrischeng-c4/lumen@(sha256:[0-9a-f]{64})$ ]]; then
    echo "${BASH_REMATCH[1]}"
  elif [[ "$raw" =~ ^(containerd|cri-o|docker)://(sha256:[0-9a-f]{64})$ ]]; then
    echo "${BASH_REMATCH[2]}"
  else
    return 1
  fi
}

runtime_digest_is_expected() {
  local digest="$1"
  [[ "$digest" == "$ROOT_DIGEST" || "$digest" == "$EXPECTED_RUNTIME_DIGEST" ]]
}

assert_named_pods() {
  local namespace="$1" label="$2" container="$3" desired="$4"
  local pods pod_count named_specs desired_specs named_statuses image_ids runtime_id normalized
  pods="$(kubectl -n "$namespace" get pods -l "$label" -o json)"
  pod_count="$(jq -er '.items | length' <<<"$pods")"
  named_specs="$(jq --arg name "$container" '[.items[].spec.containers[] | select(.name == $name)] | length' <<<"$pods")"
  desired_specs="$(jq --arg name "$container" --arg image "$IMAGE_TAG" '[.items[].spec.containers[] | select(.name == $name and .image == $image)] | length' <<<"$pods")"
  named_statuses="$(jq --arg name "$container" '[.items[] | (.status.containerStatuses // [])[] | select(.name == $name)] | length' <<<"$pods")"
  image_ids="$(jq --arg name "$container" '[.items[] | (.status.containerStatuses // [])[] | select(.name == $name and (.imageID | type) == "string" and (.imageID | length) > 0)] | length' <<<"$pods")"
  [[ "$pod_count" -eq "$desired" ]] || die "$container pod count $pod_count != desired replicas $desired"
  [[ "$named_specs" -eq "$desired" && "$desired_specs" -eq "$desired" ]] || die "$container desired-image container count mismatch"
  [[ "$named_statuses" -eq "$desired" && "$image_ids" -eq "$desired" ]] || die "$container runtime imageID count mismatch"
  while IFS= read -r runtime_id; do
    normalized="$(normalize_runtime_image_id "$runtime_id")" || die "unrecognized $container runtime imageID: $runtime_id"
    runtime_digest_is_expected "$normalized" || \
      die "$container runtime imageID $runtime_id is neither the pinned root digest $ROOT_DIGEST nor the expected platform child digest $EXPECTED_RUNTIME_DIGEST"
  done < <(jq -r --arg name "$container" '.items[] | .status.containerStatuses[] | select(.name == $name) | .imageID' <<<"$pods")
}

assert_cluster_identity() {
  if [[ "$IMAGE_MODE" != "prebuilt" ]]; then
    return 0
  fi
  echo "   verifying prebuilt cluster desired state and runtime image identities"
  local op_json op_replicas op_named op_image cr_img sset_json sset_replicas sset_named sset_image ver_json
  op_json="$(kubectl -n "$OPERATOR_NS" get deploy/lumen-operator -o json)"
  op_replicas="$(jq -er '.spec.replicas | select(type == "number" and . > 0 and . == floor)' <<<"$op_json")" || die "operator replicas must be a positive integer"
  op_named="$(jq '[.spec.template.spec.containers[] | select(.name == "operator")] | length' <<<"$op_json")"
  op_image="$(jq --arg image "$IMAGE_TAG" '[.spec.template.spec.containers[] | select(.name == "operator" and .image == $image)] | length' <<<"$op_json")"
  [[ "$op_named" -eq 1 && "$op_image" -eq 1 ]] || die "operator Deployment template must contain one named operator container at $IMAGE_TAG"
  assert_named_pods "$OPERATOR_NS" "app.kubernetes.io/name=lumen-operator" operator "$op_replicas"
  cr_img="$(kubectl -n "$NAMESPACE" get lumen/"${LUMEN_CR_NAME}" -o jsonpath='{.spec.image}')"
  [[ "$cr_img" == "$IMAGE_TAG" ]] || die "Lumen CR spec.image $cr_img != $IMAGE_TAG"
  sset_json="$(kubectl -n "$NAMESPACE" get statefulset/"${LUMEN_CR_NAME}" -o json)"
  sset_replicas="$(jq -er '.spec.replicas | select(type == "number" and . > 0 and . == floor)' <<<"$sset_json")" || die "serving replicas must be a positive integer"
  sset_named="$(jq '[.spec.template.spec.containers[] | select(.name == "server")] | length' <<<"$sset_json")"
  sset_image="$(jq --arg image "$IMAGE_TAG" '[.spec.template.spec.containers[] | select(.name == "server" and .image == $image)] | length' <<<"$sset_json")"
  [[ "$sset_named" -eq 1 && "$sset_image" -eq 1 ]] || die "StatefulSet template must contain one named server container at $IMAGE_TAG"
  assert_named_pods "$NAMESPACE" "$APP_LABEL" server "$sset_replicas"
  ver_json="$(curl -fsS --max-time 10 "http://127.0.0.1:${PORT_LOCAL}/version")"
  jq -e --arg version "$EXPECTED_VER" --arg sha "$EXPECTED_GIT_SHA" '
    (.version | type) == "string" and (.git_sha | type) == "string" and
    .version != "unknown" and .git_sha != "unknown" and
    .version == $version and .git_sha == $sha
  ' <<<"$ver_json" >/dev/null || die "/version identity is missing, non-string, unknown, or mismatched"
  echo "   prebuilt identity verified: version=$EXPECTED_VER git_sha=$EXPECTED_GIT_SHA runtime_digest=$EXPECTED_RUNTIME_DIGEST"
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

step "2b. configure Lumen-only WAL mode" \
  configure_lumen_only_deployment

step "2c. assert operator storage topology" assert_operator_topology

# ---------------------------------------------------------------------------
# 3. Wait for pod Ready
# ---------------------------------------------------------------------------

step "3. wait for serving pods Ready" wait_lumen_ready 240
step "3b. assert live operator PVC and mounts" assert_operator_storage_live

# ---------------------------------------------------------------------------
# 4. Drive the public HTTP API
# ---------------------------------------------------------------------------

step "4a. expose lumen on NodePort :${NODE_PORT} → host :${PORT_LOCAL}" expose_nodeport
step "4a2. assert cluster identity and /version" assert_cluster_identity
step "4b. PUT /collections/users" api_put_collection

FIXTURE_FILE="$(mktemp -t lumen-fixture.XXXXXX.json)"
# Each doc emits 2 IndexItems (bio + email); split the request bodies so no
# single POST exceeds the public HTTP cap (MAX_INDEX_BATCH_SIZE=1000).
step "4c. generate ${DOC_COUNT}-doc fixture (batched ≤${BATCH_SIZE} items/req)" \
  python3 "${SCRIPT_DIR}/load-fixture.py" \
    --count "$DOC_COUNT" \
    --items-per-batch "$BATCH_SIZE" \
    --output "$FIXTURE_FILE"

# The fixture script emits one NDJSON doc per line *and* one or more batched
# IndexRequest bodies (<fixture>.req.000.json, .001.json, …). POST each in
# order — a real bulk client batches within the per-request item cap.
discover_fixture_bodies
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

# `POST /collections/{id}/duplicates` is NOT a routed verb. In routed
# multi-shard mode the router rejects it outright with `501
# duplicates_not_routed` -- duplicate detection filters by `min_group_size`
# on one shard, before any cross-shard merge could happen, so answering it
# across shards would be answering it wrong (apps/lumen/src/spec.rs:788,
# apps/lumen/src/api.rs:1990). Asserting "at least one duplicate group"
# against a 2-shard cluster therefore demands the product break its own
# documented contract, and `curl -fsS` turned that into a bare `curl: (22)`
# with no step name attached. Assert the refusal itself instead; the single
# shard path keeps the original assertion, which is where duplicates is real.
if (( SHARD_COUNT > 1 )); then
  DUP_RAW="$(api_duplicates_status)"
  DUP_CODE="${DUP_RAW##*$'\n'}"
  DUP_BODY="${DUP_RAW%$'\n'*}"
  echo "   routed duplicates refusal: HTTP $DUP_CODE"
  if [[ "$DUP_CODE" != "501" ]] || [[ "$(echo "$DUP_BODY" | jq -r '.error')" != "duplicates_not_routed" ]]; then
    echo "!! routed mode must refuse duplicates with 501 duplicates_not_routed" >&2
    echo "   status: $DUP_CODE" >&2
    echo "   raw: $DUP_BODY" >&2
    exit 1
  fi
else
  DUP_RESP="$(api_duplicates)"
  DUP_GROUPS="$(echo "$DUP_RESP" | jq '.groups | length')"
  echo "   duplicate groups: $DUP_GROUPS"
  if [[ "$DUP_GROUPS" -le 0 ]]; then
    echo "!! expected at least one duplicate group, got $DUP_GROUPS" >&2
    echo "   raw: $DUP_RESP" >&2
    exit 1
  fi
fi

# ---------------------------------------------------------------------------
# 5. Persist one unique document. Replace the serving pod. Read the old
#    document before any new write. Then prove the replacement accepts a write.
# ---------------------------------------------------------------------------

step "5. checkpoint, replace serving pod, and prove durable recovery" durable_restart_oracle

# ---------------------------------------------------------------------------
# 11. Cleanup happens via the trap.
# ---------------------------------------------------------------------------

echo ">> kind-e2e PASS"
# HANDWRITE-END
