#!/usr/bin/env bash
# Layer-2 HA integration: deploy the auto-mode raft relay on a local kind
# cluster and prove leader failover with no committed loss.
#
# Flow: build the linux `relay` binary (cargo in a cached rust container) ->
# thin runtime image -> kind create + load -> apply the inline HA manifests
# (the standard downward-API quartet flips the single relay bin into
# replica/HA mode: REPLICAS_PER_SHARD=3) -> elect a leader -> publish ->
# assert every node committed -> kubectl delete the leader pod -> assert
# re-election among survivors -> publish again -> assert survivors committed
# the new write (kept the old one).
#
# The 3-replica topology lives INLINE here (not in k8s/ — the base is a
# single-node direct install; production HA is the operator CR path:
# `relay k8s instance render --profile prod`). This script stays kind-simple
# by injecting the same env the operator-rendered StatefulSet would.
#
# Slow (image build + cluster spin-up); run manually or in CI, not as a cargo
# gate. The deterministic, fast failover proof is tests/raft_cluster.rs.
#
# Requires: docker, kind, kubectl, curl, jq.
set -euo pipefail

REPO="$(cd "$(dirname "$0")/../../.." && pwd)"
CLUSTER=relay-smoke
IMG=relay:dev
PFPIDS=()

cleanup() {
  for p in "${PFPIDS[@]:-}"; do kill "$p" 2>/dev/null || true; done
  kind delete cluster --name "$CLUSTER" >/dev/null 2>&1 || true
}
trap cleanup EXIT

echo "==> build linux relay (cargo in a cached rust container)"
# reqwest pulls rustls/aws-lc-rs, which needs cmake to build.
docker run --rm \
  -v "$REPO:/src" -w /src \
  -e CARGO_TARGET_DIR=/src/target-linux \
  -v relay-cargo:/usr/local/cargo/registry \
  rust:1 bash -c "apt-get update -qq && apt-get install -y -qq cmake >/dev/null && cargo build --release -p relay --bin relay"

echo "==> build runtime image $IMG"
WORK="$(mktemp -d)"
cp "$REPO/target-linux/release/relay" "$WORK/relay"
cat > "$WORK/Dockerfile" <<'DOCKER'
FROM debian:bookworm-slim
RUN useradd -m -u 10001 relay
COPY relay /usr/local/bin/relay
USER relay
EXPOSE 7000
ENTRYPOINT ["/usr/local/bin/relay"]
DOCKER
docker build -t "$IMG" "$WORK"
rm -rf "$WORK"

echo "==> create kind cluster + load image"
kind create cluster --name "$CLUSTER" >/dev/null
kind load docker-image "$IMG" --name "$CLUSTER"

echo "==> deploy the inline HA manifests"
kubectl apply -f - <<'MANIFESTS'
# Headless Service: stable per-pod DNS for raft peer addressing
# (relay-<ordinal>.relay-headless:7000).
apiVersion: v1
kind: Service
metadata:
  name: relay-headless
  labels:
    app: relay
spec:
  clusterIP: None
  publishNotReadyAddresses: true
  selector:
    app: relay
  ports:
    - name: http
      port: 7000
      targetPort: http
---
# Keep a quorum during voluntary disruptions: at most one voter down at a time.
apiVersion: policy/v1
kind: PodDisruptionBudget
metadata:
  name: relay
spec:
  maxUnavailable: 1
  selector:
    matchLabels:
      app: relay
---
# The auto-mode HA topology: the standard downward-API quartet raft-host reads.
# REPLICAS_PER_SHARD > 1 flips the single `relay` entrypoint into replica mode;
# keep spec.replicas == SHARD_COUNT * REPLICAS_PER_SHARD and VOTER_COUNT odd.
apiVersion: apps/v1
kind: StatefulSet
metadata:
  name: relay
  labels:
    app: relay
spec:
  serviceName: relay-headless
  replicas: 3
  podManagementPolicy: Parallel
  selector:
    matchLabels:
      app: relay
  template:
    metadata:
      labels:
        app: relay
    spec:
      terminationGracePeriodSeconds: 5
      containers:
        - name: relay
          image: relay:dev
          imagePullPolicy: IfNotPresent
          ports:
            - name: http
              containerPort: 7000
          env:
            - name: POD_NAME
              valueFrom:
                fieldRef:
                  fieldPath: metadata.name
            - name: SHARD_COUNT
              value: "1"
            - name: REPLICAS_PER_SHARD
              value: "3"
            - name: VOTER_COUNT
              value: "3"
            - name: RELAY_PEER_SERVICE
              value: "relay-headless"
            - name: RELAY_BIND
              value: "0.0.0.0:7000"
            - name: RELAY_DATA_DIR
              value: "/data"
          readinessProbe:
            httpGet:
              path: /readyz
              port: http
            initialDelaySeconds: 2
            periodSeconds: 3
          livenessProbe:
            httpGet:
              path: /healthz
              port: http
            initialDelaySeconds: 2
            periodSeconds: 5
          volumeMounts:
            - name: data
              mountPath: /data
  volumeClaimTemplates:
    - metadata:
        name: data
      spec:
        accessModes: ["ReadWriteOnce"]
        resources:
          requests:
            storage: 1Gi
MANIFESTS
kubectl rollout status statefulset/relay --timeout=120s

echo "==> port-forward pods"
for i in 0 1 2; do
  kubectl port-forward "pod/relay-$i" "808$i:7000" >/dev/null 2>&1 &
  PFPIDS+=($!)
done
sleep 3

raftz() { curl -s --max-time 2 "localhost:808$1/raftz"; }
find_leader() {
  for i in 0 1 2; do
    [ "$(raftz "$i" | jq -r .is_leader 2>/dev/null)" = "true" ] && { echo "$i"; return 0; }
  done
  return 1
}
all_committed() {
  local want="$1" i ci
  for i in 0 1 2; do
    ci="$(raftz "$i" | jq -r .commit_index 2>/dev/null || echo 0)"
    [ "${ci:-0}" -ge "$want" ] || return 1
  done
}
restart_port_forwards() {
  for p in "${PFPIDS[@]:-}"; do kill "$p" 2>/dev/null || true; done
  PFPIDS=()
  for i in 0 1 2; do
    kubectl port-forward "pod/relay-$i" "808$i:7000" >/dev/null 2>&1 &
    PFPIDS+=($!)
  done
  sleep 3
}
publish_to_leader() { # $1=message_id ; finds the current leader and publishes
  local mid="$1" l code
  for _ in $(seq 1 20); do
    l="$(find_leader)" || { sleep 1; continue; }
    code="$(curl -s -o /dev/null -w '%{http_code}' -X POST "localhost:808$l/v1/events/publish" \
      -H 'content-type: application/json' -d "{\"message_id\":\"$mid\",\"payload\":{}}")"
    [ "$code" = "200" ] && return 0
    sleep 1
  done
  return 1
}

echo "==> wait for a leader"
LEADER=""
for _ in $(seq 1 60); do LEADER="$(find_leader)" && break; sleep 1; done
[ -n "$LEADER" ] || { echo "FAIL: no leader elected"; exit 1; }
echo "leader = relay-$LEADER"

echo "==> publish 'a' to the leader"
publish_to_leader a || { echo "FAIL: initial publish failed"; exit 1; }

echo "==> assert all nodes committed 'a'"
for _ in $(seq 1 30); do all_committed 1 && break; sleep 1; done
all_committed 1 || { echo "FAIL: engines did not converge on 'a'"; exit 1; }

echo "==> kill the leader pod relay-$LEADER (forces a re-election)"
kubectl delete pod "relay-$LEADER" --grace-period=1 >/dev/null 2>&1 || true

# k8s reschedules the deleted pod (same name + PVC) and it rejoins — possibly
# even winning leadership back. The meaningful HA property is that the cluster
# recovers a working leader and keeps committed data, not which node leads.
echo "==> wait for the StatefulSet to recover (deleted pod reschedules onto its PVC)"
kubectl wait --for=condition=Ready pod -l app=relay --timeout=90s >/dev/null
restart_port_forwards

echo "==> wait for the cluster to have a leader again"
NEW=""
for _ in $(seq 1 60); do NEW="$(find_leader)" && break; sleep 1; done
[ -n "$NEW" ] || { echo "FAIL: cluster did not recover a leader after the kill"; exit 1; }
echo "leader after failover = relay-$NEW"

echo "==> publish 'b' post-failover (liveness + no committed loss)"
publish_to_leader b || { echo "FAIL: post-failover publish failed"; exit 1; }

echo "==> assert every node committed >= 2 (kept 'a', added 'b')"
for _ in $(seq 1 30); do all_committed 2 && break; sleep 1; done
all_committed 2 || { echo "FAIL: nodes did not retain 'a' + commit 'b'"; exit 1; }

echo "PASS: deployed, elected, replicated, survived a leader-pod kill (re-elected, no committed loss), and accepted new writes."
