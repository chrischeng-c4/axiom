#!/usr/bin/env bash
set -euo pipefail

# Deploy one StatefulSet app to an already-reachable cluster, run its verify
# contract, collect evidence, and ALWAYS tear the namespace down — the PVCs
# go with it, so nothing bills after the run.
#
# Nothing here (or under verify/) is GKE-specific: it needs KUBECONFIG,
# kubectl, kustomize, and an image the nodes can pull, so build-debug reuses
# it unchanged on a local kind cluster (scripts/build/debug.sh).
#
# usage: IMAGE=ghcr.io/<owner>/<app>@sha256:... EVIDENCE_DIR=... run-app.sh <app>
#   KEEP_NAMESPACE=1  skip the namespace delete and print how to remove it —
#                     a local-debug affordance; the GHA workflow never sets it.

app="${1:?usage: run-app.sh <keep|defer|relay|loom>}"
case "$app" in
  keep|defer|relay|loom) ;;
  *) echo "unsupported app '$app' (keep|defer|relay|loom)" >&2; exit 2 ;;
esac
: "${IMAGE:?IMAGE is required (digest-pinned GHCR ref, or a kind-loaded local tag)}"
: "${EVIDENCE_DIR:?EVIDENCE_DIR is required}"
: "${KUBECONFIG:?KUBECONFIG must point at a task-local file (ensure-cluster.sh or kind get kubeconfig wrote it)}"
RUN_ID="${RUN_ID:?RUN_ID is required}"

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
HARNESS_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
REPO_ROOT="$(cd "$HARNESS_ROOT/../.." && pwd)"
ns="$app"
app_evidence="$EVIDENCE_DIR/$app"
mkdir -p "$app_evidence"

render_dir="$(mktemp -d)"

collect_evidence() {
  kubectl -n "$ns" get all,pvc -o wide \
    > "$app_evidence/resources.txt" 2>&1 || true
  kubectl -n "$ns" describe statefulset "$app" \
    > "$app_evidence/statefulset-describe.txt" 2>&1 || true
  kubectl -n "$ns" describe pods \
    > "$app_evidence/pods-describe.txt" 2>&1 || true
  kubectl -n "$ns" logs "${app}-0" --all-containers --tail=400 \
    > "$app_evidence/pod-0.log" 2>&1 || true
  kubectl -n "$ns" logs "${app}-0" --all-containers --tail=400 --previous \
    > "$app_evidence/pod-0-previous.log" 2>&1 || true
}

cleanup() {
  local rc=$?
  collect_evidence
  if [ "${KEEP_NAMESPACE:-0}" = 1 ]; then
    echo "namespace $ns kept; delete with: kubectl --kubeconfig $KUBECONFIG delete ns $ns" >&2
  else
    # --wait so the PVCs (and their Persistent Disks) are actually gone before
    # this returns; a leaked PVC on the persistent cluster bills indefinitely.
    kubectl delete namespace "$ns" --ignore-not-found --wait=true --timeout=300s \
      >> "$app_evidence/teardown.log" 2>&1 || {
      echo "WARNING: namespace $ns did not delete cleanly; check for leaked PVCs" >&2
      rc=1
    }
  fi
  rm -rf "$render_dir"
  exit "$rc"
}
trap cleanup EXIT

# --- render ------------------------------------------------------------------
# The template overlay references ../../base, so copy the whole k8s tree and
# edit the copy — `kustomize edit set image` overrides the images entry by name
# match, replacing the REPLACE_ME sentinels in one move.
cp -R "$REPO_ROOT/apps/$app/k8s" "$render_dir/k8s"
(cd "$render_dir/k8s/overlays/template" && kustomize edit set image "$app=$IMAGE")
kustomize build "$render_dir/k8s/overlays/template" > "$app_evidence/manifests.yaml"

if grep -q "REPLACE_ME__" "$app_evidence/manifests.yaml"; then
  echo "rendered manifests still carry REPLACE_ME sentinels" >&2
  exit 1
fi
grep -qF "image: $IMAGE" "$app_evidence/manifests.yaml" || {
  echo "rendered manifests do not pin image to $IMAGE" >&2
  exit 1
}

# --- deploy ------------------------------------------------------------------
kubectl apply -f "$app_evidence/manifests.yaml" >&2
kubectl -n "$ns" rollout status "statefulset/$app" --timeout=600s >&2

# --- verify ------------------------------------------------------------------
APP="$app" RUN_ID="$RUN_ID" EVIDENCE_DIR="$app_evidence" \
  "$HARNESS_ROOT/verify/$app.sh"
