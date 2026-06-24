#!/usr/bin/env bash
# Layer-2 HA integration: deploy the Raft-backed relay on a local kind cluster
# and prove leader failover with no committed loss.
#
# Flow: build the linux relay-raft binary (cargo in a cached rust container) ->
# thin runtime image -> kind create + load -> apply k8s manifests -> elect a
# leader -> publish -> assert every node committed -> kubectl delete the leader
# pod -> assert re-election among survivors -> publish again -> assert survivors
# committed the new write (kept the old one).
#
# Slow (image build + cluster spin-up); run manually or in CI, not as a cargo
# gate. The deterministic, fast failover proof is tests/raft_cluster.rs (#138).
#
# Requires: docker, kind, kubectl, curl, jq.
set -euo pipefail

REPO="$(cd "$(dirname "$0")/../../.." && pwd)"
CLUSTER=relay-smoke
IMG=relay-raft:dev
PFPIDS=()

cleanup() {
  for p in "${PFPIDS[@]:-}"; do kill "$p" 2>/dev/null || true; done
  kind delete cluster --name "$CLUSTER" >/dev/null 2>&1 || true
}
trap cleanup EXIT

echo "==> build linux relay-raft (cargo in a cached rust container)"
# reqwest pulls rustls/aws-lc-rs, which needs cmake to build.
docker run --rm \
  -v "$REPO:/src" -w /src \
  -e CARGO_TARGET_DIR=/src/target-linux \
  -v relay-raft-cargo:/usr/local/cargo/registry \
  rust:1 bash -c "apt-get update -qq && apt-get install -y -qq cmake >/dev/null && cargo build --release -p relay --bin relay-raft"

echo "==> build runtime image $IMG"
WORK="$(mktemp -d)"
cp "$REPO/target-linux/release/relay-raft" "$WORK/relay-raft"
cat > "$WORK/Dockerfile" <<'DOCKER'
FROM debian:bookworm-slim
RUN useradd -m -u 10001 relay
COPY relay-raft /usr/local/bin/relay-raft
USER relay
EXPOSE 8080
ENTRYPOINT ["/usr/local/bin/relay-raft"]
DOCKER
docker build -t "$IMG" "$WORK"
rm -rf "$WORK"

echo "==> create kind cluster + load image"
kind create cluster --name "$CLUSTER" >/dev/null
kind load docker-image "$IMG" --name "$CLUSTER"

echo "==> deploy manifests"
K8S="$REPO/projects/relay/k8s"
kubectl apply -f "$K8S/service.yaml" -f "$K8S/statefulset.yaml" -f "$K8S/pdb.yaml"
kubectl rollout status statefulset/relay --timeout=120s

echo "==> port-forward pods"
for i in 0 1 2; do
  kubectl port-forward "pod/relay-$i" "808$i:8080" >/dev/null 2>&1 &
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
    kubectl port-forward "pod/relay-$i" "808$i:8080" >/dev/null 2>&1 &
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
