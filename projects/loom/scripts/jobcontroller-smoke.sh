#!/usr/bin/env bash
# loom job-controller LIVE smoke (#164) — requires a real k8s cluster.
#
# The job-controller's relay→Job translation + lease loop are unit-tested with a
# fake KubeApi (cargo test -p loom), and the in-Job `run-task` entrypoint is
# verified end-to-end without k8s (scripts, manual run-task). This script
# exercises the ONLY cluster-gated piece: `kubectl create` of a real Job.
#
# Prereqs: a kube context (kind/minikube/GKE), `kubectl` on PATH, a relay + keep
# reachable from the cluster, and a worker image published as $LOOM_JOB_IMAGE.
#
#   export LOOM_RELAY=http://relay.default.svc:7400
#   export LOOM_KEEP=http://keep.default.svc:7379
#   export LOOM_JOB_IMAGE=ghcr.io/you/loom:latest
#   export LOOM_JOB_NAMESPACE=default            # optional
#   ./jobcontroller-smoke.sh
#
# Then: submit a run whose node has "runner":"k8s-job" to the loom controller;
# the controller publishes it to the loom.k8s-job relay subject, this controller
# leases it and `kubectl create`s a Job that runs `loom run-task`, which executes
# the task and reports completion back to loom. Watch with:
#   kubectl get jobs -l app=loom -w
set -euo pipefail
command -v kubectl >/dev/null || { echo "kubectl not found — needs a k8s cluster"; exit 1; }
kubectl cluster-info >/dev/null 2>&1 || { echo "no reachable kube context"; exit 1; }
: "${LOOM_RELAY:?set LOOM_RELAY}"; : "${LOOM_KEEP:?set LOOM_KEEP}"
cd "$(dirname "$0")/../../.."
echo "starting job-controller against $LOOM_RELAY (image ${LOOM_JOB_IMAGE:-loom:latest})"
exec ./target/debug/loom job-controller
