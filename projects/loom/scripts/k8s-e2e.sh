#!/usr/bin/env bash
# loom FULL k8s e2e (#164): deploy relay + keep + loom-controller + job-controller
# into a kind cluster (one image, loom:verify), submit a k8s-job run, and verify
# the job-controller `kubectl create`s a real Job that runs `loom run-task`, which
# executes the task and reports back so the run reaches succeeded.
#
# Prereqs: docker (OrbStack/Docker Desktop), kind, kubectl. ~10-15min first run
# (the in-container build). bash 3.2-safe.
set -euo pipefail
cd "$(dirname "$0")/../../.."
CLUSTER=${KIND_CLUSTER:-loom-verify}
CTX="kind-$CLUSTER"

kind get clusters 2>/dev/null | grep -qx "$CLUSTER" || kind create cluster --name "$CLUSTER" --wait 90s
docker image inspect loom:verify >/dev/null 2>&1 || docker build -t loom:verify -f projects/loom/Dockerfile .
echo "=== loading image into kind ==="
kind load docker-image loom:verify --name "$CLUSTER"
echo "=== applying deploy ==="
kubectl --context "$CTX" apply -f projects/loom/deploy/k8s.yaml
for d in relay keep loom-controller job-controller; do
  kubectl --context "$CTX" rollout status deploy/$d --timeout=120s
done

echo "=== port-forward the controller ==="
kubectl --context "$CTX" port-forward svc/loom 18474:7474 >/tmp/loom-pf.log 2>&1 & PF=$!
trap 'kill $PF 2>/dev/null' EXIT
for i in $(seq 1 30); do curl -sf http://127.0.0.1:18474/healthz >/dev/null 2>&1 && break; sleep 1; done

echo "=== submit a k8s-job run ==="
curl -s -X POST http://127.0.0.1:18474/runs -H 'content-type: application/json' \
  -d '{"run_id":"k8sjob","nodes":[{"id":"a","task_name":"echo","runner":"k8s-job"}]}' -w ' [%{http_code}]'; echo

echo "=== watch for a real Job + the run reaching succeeded ==="
ok=0
for i in $(seq 1 60); do
  jobs=$(kubectl --context "$CTX" get jobs -l app=loom --no-headers 2>/dev/null | wc -l | tr -d ' ')
  st=$(curl -s http://127.0.0.1:18474/runs/k8sjob 2>/dev/null | python3 -c 'import json,sys;print(json.load(sys.stdin).get("status",""))' 2>/dev/null || true)
  echo "  t=$i jobs=$jobs run=$st"
  if [ "$st" = "succeeded" ]; then ok=1; break; fi
  sleep 2
done
echo "=== created Jobs ==="; kubectl --context "$CTX" get jobs -l app=loom 2>/dev/null || true
[ "$ok" = "1" ] && echo "PASS: job-controller created a real k8s Job that ran loom run-task → run succeeded" || { echo "FAIL"; kubectl --context "$CTX" get pods; exit 1; }
