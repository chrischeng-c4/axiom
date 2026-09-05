#!/usr/bin/env bash
# Sift's local Kubernetes preflight.
#
# This gate proves the Kubernetes, protocol, authorization, PVC, and Raft
# paths on a disposable kind cluster. It does not claim GKE, GCS, Workload
# Identity, FQDNNetworkPolicy, or cloud-performance acceptance.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
SIFT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
REPO_ROOT="$(cd "$SIFT_DIR/../.." && pwd)"

RUN_ID="${SIFT_KIND_RUN_ID:-$(date -u +%m%d%H%M%S)}"
CLUSTER_NAME="${SIFT_KIND_CLUSTER:-sift-mvp-${RUN_ID}}"
NAMESPACE="${SIFT_KIND_NAMESPACE:-sift}"
OPERATOR_NAMESPACE="sift-system"
PROJECT="${SIFT_KIND_PROJECT:-sift-kind}"
IMAGE="${SIFT_KIND_IMAGE:-sift:kind-${RUN_ID}}"
GATEWAY_PORT="${SIFT_KIND_GATEWAY_PORT:-27380}"
STORE_HTTP_BASE="${SIFT_KIND_STORE_HTTP_BASE:-27400}"
STORE_PEER_BASE="${SIFT_KIND_STORE_PEER_BASE:-28400}"
CALICO_VERSION="${SIFT_KIND_CALICO_VERSION:-v3.32.1}"
CALICO_SHA256="${SIFT_KIND_CALICO_SHA256:-a1df919d9721cf667accdc3e72848911b0cb25cfab7d2478ad0c996302c95744}"
CALICO_URL="https://raw.githubusercontent.com/projectcalico/calico/${CALICO_VERSION}/manifests/calico.yaml"
CALICO_ROLLOUT_TIMEOUT="${SIFT_KIND_CALICO_ROLLOUT_TIMEOUT:-300s}"
POD_SUBNET="${SIFT_KIND_POD_SUBNET:-10.244.0.0/16}"
EVIDENCE_DIR="${SIFT_KIND_EVIDENCE_DIR:-/private/tmp/sift-kind-evidence/${RUN_ID}}"
WORK_DIR="$(mktemp -d "${TMPDIR:-/tmp}/sift-kind.XXXXXX")"
KUBECONFIG="$WORK_DIR/kubeconfig"
PEER_DIR="$WORK_DIR/peer-pki"
SIFT_CLI="${SIFT_KIND_CLI:-$REPO_ROOT/target/debug/sift}"

export KUBECONFIG

CLUSTER_CREATED=0
FORWARD_PIDS=()
TOKEN_FILE="$WORK_DIR/sift-rig.token"
SIFT_URL="http://127.0.0.1:${GATEWAY_PORT}"

step() {
  echo ">> $*"
}

die() {
  echo "!! Sift kind preflight failed: $*" >&2
  return 1
}

require() {
  command -v "$1" >/dev/null 2>&1 || {
    echo "!! required command not found: $1" >&2
    exit 1
  }
}

sha256_file() {
  if command -v sha256sum >/dev/null 2>&1; then
    sha256sum "$1" | awk '{print $1}'
  else
    shasum -a 256 "$1" | awk '{print $1}'
  fi
}

extract_sse_json() {
  local input="$1"
  local output="$2"
  sed -n 's/^data: //p' "$input" > "$output"
  [[ -s "$output" ]] || die "MCP response did not contain an SSE data event"
}

stop_forwards() {
  local pid
  for pid in "${FORWARD_PIDS[@]:-}"; do
    kill "$pid" >/dev/null 2>&1 || true
    wait "$pid" >/dev/null 2>&1 || true
  done
  FORWARD_PIDS=()
}

capture_diagnostics() {
  [[ "$CLUSTER_CREATED" == "1" ]] || return 0
  mkdir -p "$EVIDENCE_DIR/diagnostics"
  kubectl get nodes -o wide > "$EVIDENCE_DIR/diagnostics/nodes.txt" 2>&1 || true
  kubectl get pods -A -o wide > "$EVIDENCE_DIR/diagnostics/pods.txt" 2>&1 || true
  kubectl -n kube-system get events --sort-by=.lastTimestamp \
    > "$EVIDENCE_DIR/diagnostics/kube-system-events.txt" 2>&1 || true
  kubectl -n kube-system describe daemonset/calico-node \
    > "$EVIDENCE_DIR/diagnostics/calico-node.txt" 2>&1 || true
  kubectl -n kube-system describe pods -l k8s-app=calico-node \
    > "$EVIDENCE_DIR/diagnostics/calico-node-pods.txt" 2>&1 || true
  local calico_pod
  for calico_pod in $(kubectl -n kube-system get pods -l k8s-app=calico-node -o name 2>/dev/null); do
    kubectl -n kube-system logs "$calico_pod" --all-containers --tail=300 --prefix \
      >> "$EVIDENCE_DIR/diagnostics/calico-node.log" 2>&1 || true
  done
  kubectl -n "$NAMESPACE" get sift/sift -o yaml \
    > "$EVIDENCE_DIR/diagnostics/sift.yaml" 2>&1 || true
  kubectl -n "$NAMESPACE" get deployment,statefulset,daemonset,pod,pvc,service,networkpolicy,job -o yaml \
    > "$EVIDENCE_DIR/diagnostics/workloads.yaml" 2>&1 || true
  kubectl -n "$NAMESPACE" get events --sort-by=.lastTimestamp \
    > "$EVIDENCE_DIR/diagnostics/events.txt" 2>&1 || true
  kubectl -n "$OPERATOR_NAMESPACE" logs deployment/sift-operator --tail=400 \
    > "$EVIDENCE_DIR/diagnostics/operator.log" 2>&1 || true
  local pod
  for pod in $(kubectl -n "$NAMESPACE" get pods -o name 2>/dev/null); do
    kubectl -n "$NAMESPACE" logs "$pod" --all-containers --tail=300 --prefix \
      >> "$EVIDENCE_DIR/diagnostics/sift-pods.log" 2>&1 || true
    kubectl -n "$NAMESPACE" logs "$pod" --all-containers --tail=300 --prefix --previous \
      >> "$EVIDENCE_DIR/diagnostics/sift-pods-previous.log" 2>&1 || true
  done
}

cleanup() {
  local ec=$?
  trap - EXIT INT TERM
  stop_forwards
  if [[ "$ec" -ne 0 ]]; then
    capture_diagnostics
  fi
  if [[ "$CLUSTER_CREATED" == "1" && "${SIFT_KIND_KEEP_CLUSTER:-0}" != "1" ]]; then
    if ! kind delete cluster --name "$CLUSTER_NAME" >/dev/null 2>&1; then
      echo "!! failed to delete kind cluster $CLUSTER_NAME" >&2
      (( ec != 0 )) || ec=1
    elif kind get clusters | grep -Fxq "$CLUSTER_NAME"; then
      echo "!! kind cluster still exists after deletion: $CLUSTER_NAME" >&2
      (( ec != 0 )) || ec=1
    fi
  elif [[ "$CLUSTER_CREATED" == "1" ]]; then
    echo ">> preserving kind cluster $CLUSTER_NAME"
  fi
  case "$WORK_DIR" in
    "${TMPDIR:-/tmp}"/sift-kind.*|/tmp/sift-kind.*|/private/tmp/sift-kind.*)
      rm -rf -- "$WORK_DIR"
      ;;
    *)
      echo "!! refusing to remove unexpected work directory: $WORK_DIR" >&2
      (( ec != 0 )) || ec=1
      ;;
  esac
  echo ">> evidence: $EVIDENCE_DIR"
  exit "$ec"
}
trap cleanup EXIT
trap 'exit 130' INT TERM

for command in awk cargo curl date docker gzip jq kind kubectl openssl rg sed shasum; do
  require "$command"
done

[[ "$RUN_ID" =~ ^[a-z0-9][a-z0-9-]{0,24}$ ]] || {
  echo "SIFT_KIND_RUN_ID must be 1-25 lowercase letters, digits, or dashes" >&2
  exit 2
}
[[ "$CLUSTER_NAME" =~ ^[a-z0-9][a-z0-9-]{0,49}$ ]] || {
  echo "SIFT_KIND_CLUSTER must be 1-50 lowercase letters, digits, or dashes" >&2
  exit 2
}
[[ "$IMAGE" != *" "* && "$IMAGE" != *$'\n'* ]] || {
  echo "SIFT_KIND_IMAGE contains whitespace" >&2
  exit 2
}

mkdir -p "$EVIDENCE_DIR/kubernetes" "$EVIDENCE_DIR/protocol" "$PEER_DIR"
chmod 0700 "$PEER_DIR"

if kind get clusters | grep -Fxq "$CLUSTER_NAME"; then
  echo "refusing to reuse existing kind cluster $CLUSTER_NAME" >&2
  exit 1
fi

step "build the current Sift CLI and Linux image"
cargo build --locked -p sift --bin sift
if [[ "${SIFT_KIND_SKIP_BUILD:-0}" != "1" ]]; then
  case "$(uname -m)" in
    arm64|aarch64) platform="linux/arm64" ;;
    x86_64|amd64) platform="linux/amd64" ;;
    *) echo "unsupported host architecture: $(uname -m)" >&2; exit 1 ;;
  esac
  docker build --platform "$platform" -f "$SIFT_DIR/Dockerfile" -t "$IMAGE" "$REPO_ROOT"
fi
docker image inspect "$IMAGE" --format '{{.Id}}' > "$EVIDENCE_DIR/image-id.txt"

step "create one control plane and three kind workers"
CLUSTER_CREATED=1
kind create cluster --name "$CLUSTER_NAME" --kubeconfig "$KUBECONFIG" --config - <<EOF
kind: Cluster
apiVersion: kind.x-k8s.io/v1alpha4
networking:
  disableDefaultCNI: true
  podSubnet: ${POD_SUBNET}
nodes:
  - role: control-plane
  - role: worker
  - role: worker
  - role: worker
EOF

step "install pinned Calico NetworkPolicy enforcement"
curl -fsSL "$CALICO_URL" -o "$WORK_DIR/calico.yaml"
actual_calico_sha="$(sha256_file "$WORK_DIR/calico.yaml")"
[[ "$actual_calico_sha" == "$CALICO_SHA256" ]] || {
  echo "Calico manifest digest mismatch: $actual_calico_sha" >&2
  exit 1
}
printf '%s\n' "$actual_calico_sha" > "$EVIDENCE_DIR/calico-manifest-sha256.txt"
# Calico's raw manifest defaults to 192.168.0.0/16 with IP-in-IP. OrbStack can
# assign kind nodes from that same range, which makes pod-to-node traffic
# unroutable. Keep the downloaded bytes pinned, then make the local transport
# explicit: a non-overlapping pod CIDR and VXLAN between containerized nodes.
sed \
  -e 's/^            # - name: CALICO_IPV4POOL_CIDR$/            - name: CALICO_IPV4POOL_CIDR/' \
  -e "s|^            #   value: \"192.168.0.0/16\"\$|              value: \"${POD_SUBNET}\"|" \
  -e '/name: CALICO_IPV4POOL_IPIP/{n;s/value: "Always"/value: "Never"/;}' \
  -e '/name: CALICO_IPV4POOL_VXLAN/{n;s/value: "Never"/value: "Always"/;}' \
  "$WORK_DIR/calico.yaml" > "$WORK_DIR/calico-kind.yaml"
rg -q 'name: CALICO_IPV4POOL_CIDR' "$WORK_DIR/calico-kind.yaml" \
  || die "Calico manifest does not expose CALICO_IPV4POOL_CIDR"
rg -q 'value: "Never"' "$WORK_DIR/calico-kind.yaml" \
  || die "Calico manifest did not disable IP-in-IP"
rg -q 'value: "Always"' "$WORK_DIR/calico-kind.yaml" \
  || die "Calico manifest did not enable VXLAN"
kubectl create -f "$WORK_DIR/calico-kind.yaml"
kubectl -n kube-system rollout status daemonset/calico-node --timeout="$CALICO_ROLLOUT_TIMEOUT"
kubectl -n kube-system rollout status deployment/calico-kube-controllers --timeout="$CALICO_ROLLOUT_TIMEOUT"
kubectl wait --for=condition=Ready nodes --all --timeout=300s
kubectl get nodes -o json > "$EVIDENCE_DIR/kubernetes/nodes.json"
jq -e '[.items[] | select(.metadata.labels["node-role.kubernetes.io/control-plane"] == null)] | length == 3' \
  "$EVIDENCE_DIR/kubernetes/nodes.json" >/dev/null || die "kind did not create three workers"

step "load the exact Sift image into every kind node"
kind load docker-image "$IMAGE" --name "$CLUSTER_NAME"

step "render and install the Sift CRD and operator"
"$SIFT_CLI" k8s crd render --out "$WORK_DIR/crd.yaml"
"$SIFT_CLI" k8s operator render \
  --namespace "$OPERATOR_NAMESPACE" --image "$IMAGE" --out "$WORK_DIR/operator.yaml"
kubectl apply -f "$WORK_DIR/crd.yaml"
kubectl wait --for=condition=Established crd/sifts.sift.axiom.dev --timeout=90s
kubectl apply -f "$WORK_DIR/operator.yaml"
kubectl -n "$OPERATOR_NAMESPACE" rollout status deployment/sift-operator --timeout=300s

step "create peer PKI and scoped Kubernetes authorization"
cat > "$PEER_DIR/leaf.ext" <<'EOF'
basicConstraints=critical,CA:FALSE
keyUsage=critical,digitalSignature,keyEncipherment
extendedKeyUsage=serverAuth,clientAuth
subjectAltName=DNS:*.sift-store-headless.sift.svc.cluster.local,DNS:*.sift-control-headless.sift.svc.cluster.local,DNS:sift-store.sift.svc.cluster.local,DNS:sift-control.sift.svc.cluster.local
EOF
openssl req -x509 -newkey rsa:2048 -nodes -days 2 \
  -subj "/CN=Sift kind peer CA ${RUN_ID}" \
  -keyout "$PEER_DIR/ca.key" -out "$PEER_DIR/ca.crt" >/dev/null 2>&1
openssl req -newkey rsa:2048 -nodes -subj "/CN=sift-peer" \
  -keyout "$PEER_DIR/tls.key" -out "$PEER_DIR/tls.csr" >/dev/null 2>&1
openssl x509 -req -days 2 -sha256 -in "$PEER_DIR/tls.csr" \
  -CA "$PEER_DIR/ca.crt" -CAkey "$PEER_DIR/ca.key" -CAcreateserial \
  -extfile "$PEER_DIR/leaf.ext" -out "$PEER_DIR/tls.crt" >/dev/null 2>&1

kubectl create namespace "$NAMESPACE"
kubectl -n "$NAMESPACE" create secret generic sift-peer-tls \
  --from-file=tls.crt="$PEER_DIR/tls.crt" \
  --from-file=tls.key="$PEER_DIR/tls.key" \
  --from-file=ca.crt="$PEER_DIR/ca.crt"
kubectl -n "$NAMESPACE" create serviceaccount sift-rig
kubectl apply -f - <<EOF
apiVersion: rbac.authorization.k8s.io/v1
kind: Role
metadata:
  name: sift-rig-project
  namespace: ${NAMESPACE}
rules:
  - apiGroups: ["sift.axiom.dev"]
    resources: ["projects"]
    resourceNames: ["${PROJECT}"]
    verbs: ["get", "create", "update"]
---
apiVersion: rbac.authorization.k8s.io/v1
kind: RoleBinding
metadata:
  name: sift-rig-project
  namespace: ${NAMESPACE}
roleRef:
  apiGroup: rbac.authorization.k8s.io
  kind: Role
  name: sift-rig-project
subjects:
  - kind: ServiceAccount
    name: sift-rig
    namespace: ${NAMESPACE}
EOF

step "create the Sift three-voter instance"
kubectl apply -f - <<EOF
apiVersion: sift.axiom.dev/v1alpha1
kind: Sift
metadata:
  name: sift
  namespace: ${NAMESPACE}
spec:
  image: ${IMAGE}
  peerTlsSecret: sift-peer-tls
  replicasPerShard: 3
  voterCount: 3
  storage:
    storeSize: 1Gi
    controlSize: 1Gi
    gatewaySize: 1Gi
    querySize: 1Gi
  ingest:
    maxItemsPerMinute: 720000
    maxConcurrentRequests: 32
  auth: kubernetes
  gcpProjectId: local-kind
  gkeClusterName: ${CLUSTER_NAME}
  gkeLocation: local
EOF

wait_role_ready() {
  local kind="$1" name="$2" want="$3" deadline=$((SECONDS + 600)) ready
  while (( SECONDS < deadline )); do
    ready="$(kubectl -n "$NAMESPACE" get "$kind/$name" \
      -o jsonpath='{.status.readyReplicas}' 2>/dev/null || true)"
    [[ "$ready" == "$want" ]] && return 0
    sleep 5
  done
  die "$kind/$name did not reach $want ready replicas"
}

wait_role_ready statefulset sift-store 3
wait_role_ready statefulset sift-control 3
wait_role_ready deployment sift-gateway 1
wait_role_ready deployment sift-query 1
kubectl -n "$NAMESPACE" rollout status daemonset/sift-agent --timeout=600s

cr_deadline=$((SECONDS + 180))
while (( SECONDS < cr_deadline )); do
  [[ "$(kubectl -n "$NAMESPACE" get sift/sift -o jsonpath='{.status.phase}' 2>/dev/null || true)" == "Ready" ]] && break
  sleep 3
done
[[ "$(kubectl -n "$NAMESPACE" get sift/sift -o jsonpath='{.status.phase}')" == "Ready" ]] \
  || die "Sift CR did not reach Ready"

kubectl -n "$NAMESPACE" get deployment,statefulset,daemonset,pod,pvc,serviceaccount,networkpolicy -o json \
  > "$EVIDENCE_DIR/kubernetes/topology.json"
jq -e --arg image "$IMAGE" '
  def workload($kind; $name): .items[] | select(.kind == $kind and .metadata.name == $name);
  ([.items[] | select(.kind == "PersistentVolumeClaim")] | length) == 8
  and (workload("StatefulSet"; "sift-store").spec.replicas == 3)
  and (workload("StatefulSet"; "sift-control").spec.replicas == 3)
  and (workload("Deployment"; "sift-gateway").spec.replicas == 1)
  and (workload("Deployment"; "sift-query").spec.replicas == 1)
  and (workload("StatefulSet"; "sift-store").spec.template.spec.containers[0].image == $image)
  and ([.items[] | select(.kind == "NetworkPolicy")] | length) == 7
' "$EVIDENCE_DIR/kubernetes/topology.json" >/dev/null || die "rendered topology did not match the kind contract"

kubectl -n "$NAMESPACE" get pods -l sift.axiom.dev/role=store -o json \
  | jq -e '[.items[].spec.nodeName] | unique | length == 3' >/dev/null \
  || die "store voters do not occupy three nodes"
kubectl -n "$NAMESPACE" get pods -l sift.axiom.dev/role=control -o json \
  | jq -e '[.items[].spec.nodeName] | unique | length == 3' >/dev/null \
  || die "control voters do not occupy three nodes"

step "prove standard NetworkPolicy allow and deny behavior"
sleep 10
kubectl apply -f - <<EOF
apiVersion: batch/v1
kind: Job
metadata:
  name: network-policy-probe
  namespace: ${NAMESPACE}
spec:
  backoffLimit: 0
  activeDeadlineSeconds: 90
  template:
    spec:
      restartPolicy: Never
      containers:
        - name: probe
          image: busybox:1.36.1
          imagePullPolicy: IfNotPresent
          command: ["sh", "-ec"]
          args:
            - |
              wget -q -T 10 -O /dev/null http://sift.${NAMESPACE}.svc.cluster.local:7380/healthz
              if wget -q -T 5 -O /dev/null http://sift-store.${NAMESPACE}.svc.cluster.local:7380/healthz 2>/dev/null; then
                echo "store ingress unexpectedly allowed" >&2
                exit 42
              fi
              echo '{"gateway_ingress":"allowed","store_ingress":"denied"}'
EOF
kubectl -n "$NAMESPACE" wait --for=condition=Complete job/network-policy-probe --timeout=120s \
  || die "NetworkPolicy probe did not complete"
kubectl -n "$NAMESPACE" logs job/network-policy-probe \
  > "$EVIDENCE_DIR/kubernetes/network-policy-probe.json"
jq -e '.gateway_ingress == "allowed" and .store_ingress == "denied"' \
  "$EVIDENCE_DIR/kubernetes/network-policy-probe.json" >/dev/null \
  || die "NetworkPolicy allow/deny evidence was invalid"

start_gateway_forward() {
  kubectl -n "$NAMESPACE" port-forward service/sift "${GATEWAY_PORT}:7380" \
    >> "$EVIDENCE_DIR/kubernetes/gateway-forward.log" 2>&1 &
  FORWARD_PIDS+=("$!")
  local deadline=$((SECONDS + 120))
  while (( SECONDS < deadline )); do
    if curl -fsS --max-time 3 "${SIFT_URL}/readyz" >/dev/null 2>&1; then
      return 0
    fi
    sleep 2
  done
  die "gateway port-forward did not become ready"
}

store_http_port() { echo $((STORE_HTTP_BASE + $1)); }
store_peer_port() { echo $((STORE_PEER_BASE + $1)); }

start_store_forwards() {
  local ordinal http_port peer_port
  for ordinal in 0 1 2; do
    http_port="$(store_http_port "$ordinal")"
    peer_port="$(store_peer_port "$ordinal")"
    kubectl -n "$NAMESPACE" port-forward "pod/sift-store-${ordinal}" "${http_port}:7380" \
      >> "$EVIDENCE_DIR/kubernetes/store-${ordinal}-http-forward.log" 2>&1 &
    FORWARD_PIDS+=("$!")
    kubectl -n "$NAMESPACE" port-forward "pod/sift-store-${ordinal}" "${peer_port}:7381" \
      >> "$EVIDENCE_DIR/kubernetes/store-${ordinal}-peer-forward.log" 2>&1 &
    FORWARD_PIDS+=("$!")
  done
  sleep 4
}

refresh_token() {
  kubectl -n "$NAMESPACE" create token sift-rig \
    --audience=sift.axiom.dev --duration=1800s > "$TOKEN_FILE"
  chmod 0600 "$TOKEN_FILE"
  [[ -s "$TOKEN_FILE" ]] || die "Kubernetes did not issue a projected token"
}

auth_curl() {
  curl --silent --show-error --fail-with-body \
    -H "authorization: Bearer $(sed -n '1p' "$TOKEN_FILE")" \
    -H "x-sift-project: ${PROJECT}" "$@"
}

auth_curl_status() {
  curl --silent --show-error \
    -H "authorization: Bearer $(sed -n '1p' "$TOKEN_FILE")" \
    -H "x-sift-project: ${PROJECT}" "$@"
}

integrity_to() {
  auth_curl "${SIFT_URL}/admin/integrity?project=${PROJECT}" > "$1"
}

wait_for_integrity_count() {
  local expected="$1" output="$2" deadline=$((SECONDS + 180))
  while (( SECONDS < deadline )); do
    if integrity_to "$output" 2>/dev/null \
      && [[ "$(jq -r '.event_count' "$output")" == "$expected" ]]; then
      return 0
    fi
    sleep 2
  done
  die "project event count did not reach $expected"
}

store_raftz() {
  local ordinal="$1" port host
  port="$(store_peer_port "$ordinal")"
  host="sift-store-${ordinal}.sift-store-headless.${NAMESPACE}.svc.cluster.local"
  curl --noproxy '*' --max-time 5 --silent --show-error --fail \
    --cacert "$PEER_DIR/ca.crt" --cert "$PEER_DIR/tls.crt" --key "$PEER_DIR/tls.key" \
    --resolve "${host}:${port}:127.0.0.1" "https://${host}:${port}/raftz"
}

find_store_leader() {
  local ordinal
  for ordinal in 0 1 2; do
    if store_raftz "$ordinal" 2>/dev/null \
      | jq -e '.is_leader == true and .durability_error == null' >/dev/null 2>&1; then
      printf '%s\n' "$ordinal"
      return 0
    fi
  done
  return 1
}

wait_store_leader() {
  local excluded="${1:-}" deadline=$((SECONDS + 180)) candidate
  while (( SECONDS < deadline )); do
    candidate="$(find_store_leader 2>/dev/null || true)"
    if [[ -n "$candidate" && "$candidate" != "$excluded" ]]; then
      printf '%s\n' "$candidate"
      return 0
    fi
    sleep 2
  done
  die "three-voter store group did not expose the expected leader"
}

start_gateway_forward
start_store_forwards
refresh_token

step "verify the initial three-voter Raft group"
initial_leader="$(wait_store_leader)"
for ordinal in 0 1 2; do
  store_raftz "$ordinal" > "$EVIDENCE_DIR/kubernetes/store-${ordinal}-raft-initial.json"
done
jq -s -e '
  length == 3
  and (map(.committed_voters | length == 3) | all)
  and (map(.durability_error == null) | all)
  and (map(select(.is_leader == true)) | length == 1)
' "$EVIDENCE_DIR"/kubernetes/store-*-raft-initial.json >/dev/null \
  || die "initial Raft membership or durability was unhealthy"
store_raftz "$initial_leader" > "$EVIDENCE_DIR/kubernetes/raft-leader-before.json"

run_grpc_smoke() {
  kubectl apply -f - <<EOF
apiVersion: batch/v1
kind: Job
metadata:
  name: sift-grpc
  namespace: ${NAMESPACE}
spec:
  backoffLimit: 0
  activeDeadlineSeconds: 180
  template:
    spec:
      serviceAccountName: sift-rig
      automountServiceAccountToken: false
      restartPolicy: Never
      containers:
        - name: sift
          image: ${IMAGE}
          imagePullPolicy: IfNotPresent
          args:
            - acceptance-grpc
            - --endpoint
            - http://sift.${NAMESPACE}.svc.cluster.local:4317
            - --project
            - ${PROJECT}
            - --token-file
            - /var/run/secrets/sift/token
          volumeMounts:
            - name: token
              mountPath: /var/run/secrets/sift
              readOnly: true
      volumes:
        - name: token
          projected:
            sources:
              - serviceAccountToken:
                  audience: sift.axiom.dev
                  expirationSeconds: 600
                  path: token
EOF
  kubectl -n "$NAMESPACE" wait --for=condition=Complete job/sift-grpc --timeout=240s \
    || die "OTLP/gRPC acceptance job failed"
  kubectl -n "$NAMESPACE" logs job/sift-grpc > "$EVIDENCE_DIR/protocol/grpc.json"
  jq -e '.signal == "logs" and .accepted == 1 and .rejected == 1 and .compression == "gzip"' \
    "$EVIDENCE_DIR/protocol/grpc.json" >/dev/null || die "OTLP/gRPC contract failed"
}

run_mcp_smoke() {
  local headers="$EVIDENCE_DIR/protocol/mcp-init.headers"
  local body="$EVIDENCE_DIR/protocol/mcp-init.json"
  local init_sse="$EVIDENCE_DIR/protocol/mcp-init.sse"
  local tools="$EVIDENCE_DIR/protocol/mcp-tools.json"
  local tools_sse="$EVIDENCE_DIR/protocol/mcp-tools.sse"
  local session allowed_host allowed_origin initialize bad_status
  allowed_host="sift.${NAMESPACE}.svc.cluster.local"
  allowed_origin="http://sift.${NAMESPACE}.svc.cluster.local:7380"
  initialize='{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-11-25","capabilities":{},"clientInfo":{"name":"sift-kind","version":"1"}}}'
  curl --silent --show-error --fail-with-body -D "$headers" -o "$init_sse" \
    -X POST "${SIFT_URL}/mcp" \
    -H "host: ${allowed_host}" -H "origin: ${allowed_origin}" \
    -H "authorization: Bearer $(sed -n '1p' "$TOKEN_FILE")" \
    -H 'content-type: application/json' -H 'accept: application/json, text/event-stream' \
    --data "$initialize"
  extract_sse_json "$init_sse" "$body"
  jq -e '.jsonrpc == "2.0" and .id == 1 and .result != null' "$body" >/dev/null \
    || die "MCP initialize failed"
  session="$(awk 'tolower($1) == "mcp-session-id:" {gsub("\r", "", $2); print $2; exit}' "$headers")"
  [[ -n "$session" ]] || die "MCP session header is absent"
  curl --silent --show-error --fail-with-body -X POST "${SIFT_URL}/mcp" \
    -H "host: ${allowed_host}" -H "origin: ${allowed_origin}" \
    -H "authorization: Bearer $(sed -n '1p' "$TOKEN_FILE")" \
    -H "mcp-session-id: ${session}" -H 'mcp-protocol-version: 2025-11-25' \
    -H 'content-type: application/json' -H 'accept: application/json, text/event-stream' \
    --data '{"jsonrpc":"2.0","method":"notifications/initialized"}' >/dev/null
  curl --silent --show-error --fail-with-body -X POST "${SIFT_URL}/mcp" \
    -H "host: ${allowed_host}" -H "origin: ${allowed_origin}" \
    -H "authorization: Bearer $(sed -n '1p' "$TOKEN_FILE")" \
    -H "mcp-session-id: ${session}" -H 'mcp-protocol-version: 2025-11-25' \
    -H 'content-type: application/json' -H 'accept: application/json, text/event-stream' \
    --data '{"jsonrpc":"2.0","id":2,"method":"tools/list","params":{}}' > "$tools_sse"
  extract_sse_json "$tools_sse" "$tools"
  jq -e '[.result.tools[].name] | sort == ["sift_correlate","sift_get_trace","sift_list_services","sift_query","sift_tail_logs"]' \
    "$tools" >/dev/null || die "MCP tool list drifted"
  bad_status="$(curl --silent --output /dev/null --write-out '%{http_code}' \
    -X POST "${SIFT_URL}/mcp" -H "host: ${allowed_host}" -H 'origin: https://evil.example' \
    -H 'content-type: application/json' -H 'accept: application/json, text/event-stream' \
    --data "$initialize")"
  [[ "$bad_status" == "403" ]] || die "MCP accepted an untrusted Origin"
}

step "exercise OTLP, Remote Write, query, correlation, tail, auth, and MCP"
integrity_to "$EVIDENCE_DIR/protocol/integrity-before.json"
smoke_start="$(jq -r '.event_count' "$EVIDENCE_DIR/protocol/integrity-before.json")"
epoch_seconds="$(date -u +%s)"
timestamp_nanos="${epoch_seconds}000000000"
trace_id=aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa
span_id=bbbbbbbbbbbbbbbb

jq -nc --arg ts "$timestamp_nanos" --arg trace "$trace_id" --arg span "$span_id" '
  {resourceLogs:[{resource:{attributes:[
    {key:"service.name",value:{stringValue:"sift-acceptance"}},
    {key:"deployment.environment.name",value:{stringValue:"kind"}}
  ]},scopeLogs:[{logRecords:[
    {timeUnixNano:$ts,severityText:"ERROR",traceId:$trace,spanId:$span,
     body:{stringValue:"smoke accepted"},attributes:[{key:"sift.event_id",value:{stringValue:"smoke-log"}}]},
    {timeUnixNano:$ts,body:null}
  ]}]}]}' > "$EVIDENCE_DIR/protocol/otlp-logs-partial.json"
auth_curl -X POST "${SIFT_URL}/v1/logs" -H 'content-type: application/json' \
  --data-binary "@$EVIDENCE_DIR/protocol/otlp-logs-partial.json" \
  > "$EVIDENCE_DIR/protocol/otlp-logs-partial-response.json"
jq -e '.partialSuccess.rejectedLogRecords == 1 and (.partialSuccess.errorMessage | contains("body is required"))' \
  "$EVIDENCE_DIR/protocol/otlp-logs-partial-response.json" >/dev/null \
  || die "OTLP log partial-success was not explicit"

jq -nc --arg ts "$((timestamp_nanos + 10))" '
  {resourceMetrics:[{resource:{attributes:[
    {key:"service.name",value:{stringValue:"sift-acceptance"}},
    {key:"deployment.environment.name",value:{stringValue:"kind"}}
  ]},scopeMetrics:[{metrics:[{name:"sift.acceptance.gauge",unit:"1",gauge:{dataPoints:[{
    timeUnixNano:$ts,asDouble:42.5,attributes:[{key:"fixture",value:{stringValue:"smoke"}}],
    exemplars:[{timeUnixNano:$ts,asDouble:42.5,traceId:"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",spanId:"bbbbbbbbbbbbbbbb"}]
  }]}}]}]}]}' > "$EVIDENCE_DIR/protocol/otlp-metrics.json"
auth_curl -X POST "${SIFT_URL}/v1/metrics" -H 'content-type: application/json' \
  --data-binary "@$EVIDENCE_DIR/protocol/otlp-metrics.json" \
  > "$EVIDENCE_DIR/protocol/otlp-metrics-response.json"
jq -e 'type == "object" and (.partialSuccess == null)' \
  "$EVIDENCE_DIR/protocol/otlp-metrics-response.json" >/dev/null || die "OTLP metrics failed"

jq -nc --arg ts "$((timestamp_nanos + 20))" --arg end "$((timestamp_nanos + 1000020))" \
  --arg trace "$trace_id" --arg span "$span_id" '
  {resourceSpans:[{resource:{attributes:[
    {key:"service.name",value:{stringValue:"sift-acceptance"}},
    {key:"deployment.environment.name",value:{stringValue:"kind"}}
  ]},scopeSpans:[{spans:[{traceId:$trace,spanId:$span,name:"GET /smoke",
    startTimeUnixNano:$ts,endTimeUnixNano:$end,status:{code:2},
    attributes:[{key:"http.route",value:{stringValue:"/smoke"}}]}]}]}]}' \
  > "$EVIDENCE_DIR/protocol/otlp-traces.json"
auth_curl -X POST "${SIFT_URL}/v1/traces" -H 'content-type: application/json' \
  --data-binary "@$EVIDENCE_DIR/protocol/otlp-traces.json" \
  > "$EVIDENCE_DIR/protocol/otlp-traces-response.json"
jq -e 'type == "object" and (.partialSuccess == null)' \
  "$EVIDENCE_DIR/protocol/otlp-traces-response.json" >/dev/null || die "OTLP traces failed"

jq -nc --arg ts "$((timestamp_nanos + 30))" '
  {resourceLogs:[{resource:{attributes:[{key:"service.name",value:{stringValue:"sift-acceptance"}}]},
    scopeLogs:[{logRecords:[{timeUnixNano:$ts,body:{stringValue:"gzip accepted"},
      attributes:[{key:"sift.event_id",value:{stringValue:"smoke-gzip"}}]}]}]}]}' \
  > "$EVIDENCE_DIR/protocol/otlp-gzip.json"
gzip -c "$EVIDENCE_DIR/protocol/otlp-gzip.json" > "$EVIDENCE_DIR/protocol/otlp-gzip.json.gz"
auth_curl -X POST "${SIFT_URL}/v1/logs" -H 'content-type: application/json' \
  -H 'content-encoding: gzip' --data-binary "@$EVIDENCE_DIR/protocol/otlp-gzip.json.gz" \
  > "$EVIDENCE_DIR/protocol/otlp-gzip-response.json"
jq -e 'type == "object" and (.partialSuccess == null)' \
  "$EVIDENCE_DIR/protocol/otlp-gzip-response.json" >/dev/null || die "OTLP gzip failed"

"$SIFT_CLI" acceptance-payload --kind otlp-logs-protobuf --items 2 \
  --project "$PROJECT" --event-prefix smoke-protobuf \
  --timestamp-unix-nano "$((timestamp_nanos + 40))" > "$EVIDENCE_DIR/protocol/otlp-logs.pb"
auth_curl -X POST "${SIFT_URL}/v1/logs" -H 'content-type: application/x-protobuf' \
  --data-binary "@$EVIDENCE_DIR/protocol/otlp-logs.pb" \
  > "$EVIDENCE_DIR/protocol/otlp-logs-protobuf-response.pb"

run_grpc_smoke

"$SIFT_CLI" acceptance-payload --kind prometheus-remote-write-v1 --items 2 \
  --project "$PROJECT" --event-prefix smoke-remote-write \
  --timestamp-unix-nano "$((timestamp_nanos + 50))" > "$EVIDENCE_DIR/protocol/remote-write-v1.snappy"
rw_status="$(auth_curl --output /dev/null --write-out '%{http_code}' \
  -X POST "${SIFT_URL}/prometheus/api/v1/write" \
  -H 'content-type: application/x-protobuf' -H 'content-encoding: snappy' \
  -H 'x-prometheus-remote-write-version: 0.1.0' \
  --data-binary "@$EVIDENCE_DIR/protocol/remote-write-v1.snappy")"
[[ "$rw_status" == "204" ]] || die "Remote Write 1.0 did not return 204"

wait_for_integrity_count "$((smoke_start + 9))" "$EVIDENCE_DIR/protocol/integrity-after.json"
metric_before_v2="$(jq -r '.signals.metrics.count' "$EVIDENCE_DIR/protocol/integrity-after.json")"
rw2_status="$(auth_curl_status --output "$EVIDENCE_DIR/protocol/remote-write-v2-response.json" \
  --write-out '%{http_code}' -X POST "${SIFT_URL}/prometheus/api/v1/write" \
  -H 'content-type: application/x-protobuf;proto=io.prometheus.write.v2.Request' \
  -H 'content-encoding: snappy' -H 'x-prometheus-remote-write-version: 2.0.0' --data-binary '')"
[[ "$rw2_status" == "415" ]] || die "Remote Write 2.0 was not rejected with 415"
integrity_to "$EVIDENCE_DIR/protocol/integrity-after-rw2.json"
[[ "$(jq -r '.signals.metrics.count' "$EVIDENCE_DIR/protocol/integrity-after-rw2.json")" == "$metric_before_v2" ]] \
  || die "Remote Write 2.0 rejection wrote data"

auth_curl -X POST "${SIFT_URL}/api/v1/query" -H 'content-type: application/json' \
  --data "{\"version\":1,\"project\":\"${PROJECT}\",\"environment\":\"kind\",\"signal\":{\"kind\":\"logs\",\"filter\":{\"op\":\"regex\",\"field\":\"body_text\",\"pattern\":\"smoke.*accepted\"}},\"limit\":10,\"mode\":\"sync\"}" \
  > "$EVIDENCE_DIR/protocol/query-logs.json"
jq -e '(.data.records | map(select(.event_id == "smoke-log")) | length) == 1 and .partial == false' \
  "$EVIDENCE_DIR/protocol/query-logs.json" >/dev/null || die "log query failed"

auth_curl -X POST "${SIFT_URL}/api/v1/query" -H 'content-type: application/json' \
  --data "{\"version\":1,\"project\":\"${PROJECT}\",\"environment\":\"kind\",\"signal\":{\"kind\":\"metrics\",\"name\":\"sift.acceptance.gauge\",\"function\":\"sum\",\"step_seconds\":1,\"group_by\":[\"service.name\"]},\"limit\":10,\"mode\":\"sync\"}" \
  > "$EVIDENCE_DIR/protocol/query-metrics.json"
jq -e '(.data.series | length) == 1 and .data.series[0].aggregate == 42.5' \
  "$EVIDENCE_DIR/protocol/query-metrics.json" >/dev/null || die "metric query failed"

auth_curl -X POST "${SIFT_URL}/api/v1/query" -H 'content-type: application/json' \
  --data "{\"version\":1,\"project\":\"${PROJECT}\",\"environment\":\"kind\",\"signal\":{\"kind\":\"traces\",\"service\":\"sift-acceptance\",\"operation\":\"GET /smoke\",\"min_duration_ms\":1},\"limit\":10,\"mode\":\"sync\"}" \
  > "$EVIDENCE_DIR/protocol/query-traces.json"
jq -e --arg trace "$trace_id" '(.data.traces | map(select(.trace_id == $trace)) | length) == 1' \
  "$EVIDENCE_DIR/protocol/query-traces.json" >/dev/null || die "trace query failed"

auth_curl "${SIFT_URL}/api/v1/traces/${trace_id}?project=${PROJECT}" \
  > "$EVIDENCE_DIR/protocol/trace-read.json"
jq -e --arg trace "$trace_id" '.trace_id == $trace and (.spans | length) == 1 and (.critical_path_span_ids | length) == 1' \
  "$EVIDENCE_DIR/protocol/trace-read.json" >/dev/null || die "trace read failed"

auth_curl -X POST "${SIFT_URL}/api/v1/correlate" -H 'content-type: application/json' \
  --data "{\"version\":1,\"project\":\"${PROJECT}\",\"environment\":\"kind\",\"trace_id\":\"${trace_id}\",\"limit\":10}" \
  > "$EVIDENCE_DIR/protocol/correlate.json"
jq -e '(.logs | map(select(.event_id == "smoke-log")) | length) == 1 and (.traces | length) == 1 and .partial == false' \
  "$EVIDENCE_DIR/protocol/correlate.json" >/dev/null || die "correlation failed"

auth_curl -X POST "${SIFT_URL}/api/v1/logs/tail" -H 'content-type: application/json' \
  --data "{\"version\":1,\"project\":\"${PROJECT}\",\"environment\":\"kind\",\"filter\":{\"op\":\"eq\",\"field\":\"event_id\",\"value\":\"smoke-log\"},\"wait_ms\":1000,\"limit\":10}" \
  > "$EVIDENCE_DIR/protocol/tail.json"
jq -e '(.data.records | map(select(.event_id == "smoke-log")) | length) == 1' \
  "$EVIDENCE_DIR/protocol/tail.json" >/dev/null || die "log tail failed"

denied_status="$(curl --silent --output "$EVIDENCE_DIR/protocol/cross-project.json" --write-out '%{http_code}' \
  -X POST "${SIFT_URL}/api/v1/query" \
  -H "authorization: Bearer $(sed -n '1p' "$TOKEN_FILE")" \
  -H 'x-sift-project: denied-project' -H 'content-type: application/json' \
  --data '{"version":1,"project":"denied-project","signal":{"kind":"logs"},"limit":1,"mode":"sync"}')"
[[ "$denied_status" == "403" ]] || die "cross-project access was not denied"

run_mcp_smoke

step "prove async query state and all eight PVC identities survive pod replacement"
auth_curl -X POST "${SIFT_URL}/api/v1/query" -H 'content-type: application/json' \
  --data "{\"version\":1,\"project\":\"${PROJECT}\",\"environment\":\"kind\",\"signal\":{\"kind\":\"logs\",\"filter\":{\"op\":\"eq\",\"field\":\"event_id\",\"value\":\"smoke-log\"}},\"limit\":10,\"mode\":\"async\"}" \
  > "$EVIDENCE_DIR/kubernetes/query-job-create.json"
query_id="$(jq -r '.query_id // empty' "$EVIDENCE_DIR/kubernetes/query-job-create.json")"
[[ -n "$query_id" ]] || die "async query did not return a query ID"
kubectl -n "$NAMESPACE" get pvc -o json \
  | jq '[.items[] | {name:.metadata.name,uid:.metadata.uid}] | sort_by(.name)' \
  > "$EVIDENCE_DIR/kubernetes/pvc-before-restart.json"
integrity_to "$EVIDENCE_DIR/kubernetes/integrity-before-restart.json"
gateway_pod="$(kubectl -n "$NAMESPACE" get pod -l sift.axiom.dev/role=gateway -o jsonpath='{.items[0].metadata.name}')"
query_pod="$(kubectl -n "$NAMESPACE" get pod -l sift.axiom.dev/role=query -o jsonpath='{.items[0].metadata.name}')"
stop_forwards
kubectl -n "$NAMESPACE" delete pod/sift-store-0 "pod/${gateway_pod}" "pod/${query_pod}" \
  --wait=true --timeout=240s
wait_role_ready statefulset sift-store 3
wait_role_ready deployment sift-gateway 1
wait_role_ready deployment sift-query 1
start_gateway_forward
start_store_forwards
refresh_token
integrity_to "$EVIDENCE_DIR/kubernetes/integrity-after-restart.json"
jq -e --slurpfile before "$EVIDENCE_DIR/kubernetes/integrity-before-restart.json" '
  .event_count == $before[0].event_count
  and .event_id_sha256 == $before[0].event_id_sha256
  and .watermark == $before[0].watermark
' "$EVIDENCE_DIR/kubernetes/integrity-after-restart.json" >/dev/null \
  || die "pod replacement changed acknowledged data"
kubectl -n "$NAMESPACE" get pvc -o json \
  | jq '[.items[] | {name:.metadata.name,uid:.metadata.uid}] | sort_by(.name)' \
  > "$EVIDENCE_DIR/kubernetes/pvc-after-restart.json"
jq -e --slurpfile before "$EVIDENCE_DIR/kubernetes/pvc-before-restart.json" '. == $before[0]' \
  "$EVIDENCE_DIR/kubernetes/pvc-after-restart.json" >/dev/null || die "pod replacement changed a PVC"
query_deadline=$((SECONDS + 180))
while (( SECONDS < query_deadline )); do
  if auth_curl "${SIFT_URL}/api/v1/queries/${query_id}?project=${PROJECT}" \
      > "$EVIDENCE_DIR/kubernetes/query-job-after-restart.json" 2>/dev/null \
    && jq -e '.status == "succeeded" or .status == "failed"' \
      "$EVIDENCE_DIR/kubernetes/query-job-after-restart.json" >/dev/null 2>&1; then
    break
  fi
  sleep 2
done
jq -e '.status == "succeeded" or .status == "failed"' \
  "$EVIDENCE_DIR/kubernetes/query-job-after-restart.json" >/dev/null \
  || die "async query state disappeared after query restart"

step "delete the live Raft leader and prove acknowledged writes continue"
leader_before="$(wait_store_leader)"
store_raftz "$leader_before" > "$EVIDENCE_DIR/kubernetes/raft-leader-before-failover.json"
integrity_to "$EVIDENCE_DIR/kubernetes/integrity-before-failover.json"
before_failover_count="$(jq -r '.event_count' "$EVIDENCE_DIR/kubernetes/integrity-before-failover.json")"
kubectl -n "$NAMESPACE" delete "pod/sift-store-${leader_before}" --wait=true --timeout=120s
leader_after="$(wait_store_leader "$leader_before")"
store_raftz "$leader_after" > "$EVIDENCE_DIR/kubernetes/raft-leader-after.json"
jq -e '.is_leader == true and .durability_error == null and (.committed_voters | length) == 3' \
  "$EVIDENCE_DIR/kubernetes/raft-leader-after.json" >/dev/null || die "surviving Raft leader was unhealthy"

failover_nanos="$(date -u +%s)000000000"
jq -nc --arg ts "$failover_nanos" '
  {resourceLogs:[{resource:{attributes:[
    {key:"service.name",value:{stringValue:"sift-failover"}},
    {key:"deployment.environment.name",value:{stringValue:"kind"}}
  ]},scopeLogs:[{logRecords:[{timeUnixNano:$ts,body:{stringValue:"accepted during leader replacement"},
    attributes:[{key:"sift.event_id",value:{stringValue:"kind-failover-log"}}]}]}]}]}' \
  > "$EVIDENCE_DIR/protocol/failover-log.json"
auth_curl -X POST "${SIFT_URL}/v1/logs" -H 'content-type: application/json' \
  --data-binary "@$EVIDENCE_DIR/protocol/failover-log.json" \
  > "$EVIDENCE_DIR/protocol/failover-log-response.json"
wait_for_integrity_count "$((before_failover_count + 1))" "$EVIDENCE_DIR/kubernetes/integrity-after-failover.json"
wait_role_ready statefulset sift-store 3
stop_forwards
start_gateway_forward
start_store_forwards
refresh_token
for ordinal in 0 1 2; do
  store_raftz "$ordinal" > "$EVIDENCE_DIR/kubernetes/store-${ordinal}-raft-final.json"
done
jq -s -e 'length == 3 and (map(.durability_error == null) | all) and (map(.committed_voters | length == 3) | all)' \
  "$EVIDENCE_DIR"/kubernetes/store-*-raft-final.json >/dev/null || die "Raft group did not recover all voters"
auth_curl -X POST "${SIFT_URL}/api/v1/query" -H 'content-type: application/json' \
  --data "{\"version\":1,\"project\":\"${PROJECT}\",\"environment\":\"kind\",\"signal\":{\"kind\":\"logs\",\"filter\":{\"op\":\"eq\",\"field\":\"event_id\",\"value\":\"kind-failover-log\"}},\"limit\":10,\"mode\":\"sync\"}" \
  > "$EVIDENCE_DIR/kubernetes/failover-query.json"
jq -e '(.data.records | map(select(.event_id == "kind-failover-log")) | length) == 1' \
  "$EVIDENCE_DIR/kubernetes/failover-query.json" >/dev/null || die "failover write was not queryable"

kubectl -n "$NAMESPACE" get pods -o json > "$EVIDENCE_DIR/kubernetes/pods-final.json"
jq -e 'all(.items[];
  ([.status.containerStatuses[]?.state.waiting.reason // empty] | index("CrashLoopBackOff") | not)
  and ([.status.containerStatuses[]?.lastState.terminated.reason // empty] | index("OOMKilled") | not))' \
  "$EVIDENCE_DIR/kubernetes/pods-final.json" >/dev/null || die "a pod crashed or was OOM-killed"

jq -n \
  --arg run_id "$RUN_ID" --arg cluster "$CLUSTER_NAME" --arg image "$IMAGE" \
  --arg image_id "$(sed -n '1p' "$EVIDENCE_DIR/image-id.txt")" \
  --argjson leader_before "$leader_before" --argjson leader_after "$leader_after" \
  --argjson final_count "$(jq -r '.event_count' "$EVIDENCE_DIR/kubernetes/integrity-after-failover.json")" \
  '{version:1,result:"passed",scope:"kind-preflight-not-gke-acceptance",run_id:$run_id,
    cluster:$cluster,image:$image,image_id:$image_id,
    checks:{topology:"passed",network_policy:"passed",kubernetes_auth:"passed",
      protocols:"passed",mcp:"passed",pvc_restart:"passed",query_job_restart:"passed",
      raft_leader_replacement:{status:"passed",leader_before:$leader_before,leader_after:$leader_after}},
    final_event_count:$final_count}' > "$EVIDENCE_DIR/kind-result.json"

step "Sift kind preflight passed"
jq . "$EVIDENCE_DIR/kind-result.json"
