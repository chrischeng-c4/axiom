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
DEAD="x"

cleanup() {
  for p in "${PFPIDS[@]:-}"; do kill "$p" 2>/dev/null || true; done
  kind delete cluster --name "$CLUSTER" >/dev/null 2>&1 || true
}
trap cleanup EXIT

echo "==> build linux relay-raft (cargo in a cached rust container)"
docker run --rm \
  -v "$REPO:/src" -w /src \
  -e CARGO_TARGET_DIR=/src/target-linux \
  -v relay-raft-cargo:/usr/local/cargo/registry \
  rust:1 cargo build --release -p relay --bin relay-raft

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

raftz() { curl -s "localhost:808$1/raftz"; }
find_leader() {
  for i in 0 1 2; do
    [ "$i" = "$DEAD" ] && continue
    if [ "$(raftz "$i" | jq -r .is_leader 2>/dev/null)" = "true" ]; then echo "$i"; return 0; fi
  done
  return 1
}
all_committed() {
  local want="$1" i ci
  for i in 0 1 2; do
    [ "$i" = "$DEAD" ] && continue
    ci="$(raftz "$i" | jq -r .commit_index 2>/dev/null || echo 0)"
    [ "${ci:-0}" -ge "$want" ] || return 1
  done
}

echo "==> wait for a leader"
LEADER=""
for _ in $(seq 1 30); do LEADER="$(find_leader)" && break; sleep 1; done
[ -n "$LEADER" ] || { echo "FAIL: no leader elected"; exit 1; }
echo "leader = relay-$LEADER"

echo "==> publish to the leader"
curl -s -X POST "localhost:808$LEADER/v1/events/publish" \
  -H 'content-type: application/json' -d '{"message_id":"a","payload":{"n":1}}' >/dev/null

echo "==> assert all nodes committed"
for _ in $(seq 1 30); do all_committed 1 && break; sleep 1; done
all_committed 1 || { echo "FAIL: engines did not converge"; exit 1; }

echo "==> kill leader (kubectl delete pod relay-$LEADER)"
kubectl delete pod "relay-$LEADER" --grace-period=0 --force >/dev/null 2>&1 || true
DEAD="$LEADER"

echo "==> wait for re-election among survivors"
NEW=""
for _ in $(seq 1 60); do NEW="$(find_leader)" && [ "$NEW" != "$LEADER" ] && break; sleep 1; done
[ -n "$NEW" ] && [ "$NEW" != "$LEADER" ] || { echo "FAIL: no re-election"; exit 1; }
echo "new leader = relay-$NEW"

echo "==> publish to the new leader (no committed loss + accepts new writes)"
curl -s -X POST "localhost:808$NEW/v1/events/publish" \
  -H 'content-type: application/json' -d '{"message_id":"b","payload":{"n":2}}' >/dev/null
for _ in $(seq 1 30); do all_committed 2 && break; sleep 1; done
all_committed 2 || { echo "FAIL: survivors did not commit the post-failover write"; exit 1; }

echo "PASS: elected, replicated, failed over, and kept committed data."
