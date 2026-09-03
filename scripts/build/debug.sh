#!/usr/bin/env bash
# Debug route of /build:debug for keep / defer / relay / loom: build the app's
# image locally from the working tree (cargo debug profile), load it into a
# persistent kind cluster, and run the same deploy -> verify -> evidence ->
# teardown harness that the gke-acceptance workflow runs on GKE
# (acceptance/gke-harness/scripts/run-app.sh + verify/<app>.sh).
#
# usage: scripts/build/debug.sh <app> [--fresh] [--keep] [--image <ref>] [--out <dir>]
#   app      one of: keep defer relay loom
#   --fresh  delete and recreate the kind cluster before the run
#   --keep   leave the app namespace in place after the run, for inspection
#            (KUBECONFIG=<out>/kubeconfig kubectl -n <app> ...)
#   --image  skip the local build and deploy this image instead; the kind
#            nodes pull it. A linux/amd64 GHCR image on an arm64 host runs
#            under emulation and proves that image, not this tree.
#   --out    evidence directory
#            (default ${TMPDIR:-/tmp}/build-debug-<app>-<timestamp>)
# exit: 0 verify PASS and the namespace is gone (or kept on request);
#   1 build, load, deploy, or verify failed (evidence collected once the run
#   reached the cluster); 2 refused — uncovered app, two apps, unknown flag,
#   missing tool, docker daemon down.
# Unlike release.sh this runs whatever is in the working tree: a dirty tree is
# allowed, and the image tag says so (-dirty). A cold debug build of one app
# takes minutes; run it in the background from a session.
set -euo pipefail
GIT=(git -c core.fsmonitor=false)
COVERED="keep defer relay loom"
CLUSTER=axiom-build-debug

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
HARNESS="$REPO_ROOT/acceptance/gke-harness/scripts/run-app.sh"

app='' fresh=false keep=false image='' out=''
while [ $# -gt 0 ]; do
  case "$1" in
    --fresh) fresh=true; shift ;;
    --keep) keep=true; shift ;;
    --image) image=${2:?--image needs a reference}; shift 2 ;;
    --out) out=${2:?--out needs a directory}; shift 2 ;;
    -*) echo "refused: unknown flag $1" >&2; exit 2 ;;
    *)
      [ -z "$app" ] || { echo "refused: one app per run, got $app and $1" >&2; exit 2; }
      app=$1; shift ;;
  esac
done
[ -n "$app" ] || { echo "refused: name one app (covered: $COVERED)" >&2; exit 2; }
case " $COVERED " in
  *" $app "*) ;;
  *) echo "refused: debug route not wired for $app (covered: $COVERED)" >&2; exit 2 ;;
esac
for tool in docker kind kubectl kustomize jq curl; do
  command -v "$tool" >/dev/null || { echo "refused: $tool is required" >&2; exit 2; }
done
docker info >/dev/null 2>&1 || { echo "refused: docker daemon is not reachable" >&2; exit 2; }

ts=$(date -u +%Y%m%dT%H%M%SZ)
tmp=${TMPDIR:-/tmp}
out=${out:-${tmp%/}/build-debug-$app-$ts}
mkdir -p "$out"
run_id="debug-$ts"

built_locally=false
if [ -z "$image" ]; then
  built_locally=true
  sha=$(cd "$REPO_ROOT" && "${GIT[@]}" rev-parse --short=12 HEAD)
  dirty=''
  [ -z "$(cd "$REPO_ROOT" && "${GIT[@]}" status --porcelain)" ] || dirty='-dirty'
  image="axiom/$app:debug-$sha$dirty"
fi

# The four report lines print on every exit past this point, so a failed
# build or a red verify still says where the evidence is. Refusals above
# print their one line only.
report() {
  local rc=$?
  [ "$rc" -eq 0 ] || rc=1
  echo "image: $image"
  echo "cluster: $CLUSTER (kept; kind delete cluster --name $CLUSTER to remove)"
  if [ -f "$out/$app/verdict.txt" ]; then
    echo "verdict: $(cat "$out/$app/verdict.txt")"
  else
    echo "verdict: FAIL (no verdict written — the run stopped before verify)"
  fi
  echo "evidence: $out"
  exit "$rc"
}
trap report EXIT

# --- cluster -----------------------------------------------------------------
if [ "$fresh" = true ]; then
  kind delete cluster --name "$CLUSTER" >&2
fi
if ! kind get clusters 2>/dev/null | grep -qx "$CLUSTER"; then
  kind create cluster --name "$CLUSTER" --wait 120s >&2
fi
kind get kubeconfig --name "$CLUSTER" > "$out/kubeconfig"
export KUBECONFIG="$out/kubeconfig"

# --- image -------------------------------------------------------------------
if [ "$built_locally" = true ]; then
  # Build context is the repo root: every app Dockerfile is a cargo-workspace
  # build. --progress=plain keeps the saved log readable.
  docker build --progress=plain --build-arg CARGO_PROFILE=debug \
    -f "$REPO_ROOT/apps/$app/Dockerfile" -t "$image" "$REPO_ROOT" \
    2>&1 | tee "$out/docker-build.log" >&2
  kind load docker-image "$image" --name "$CLUSTER" >&2
fi

# --- deploy + verify ---------------------------------------------------------
# A namespace left behind by an earlier --keep run would turn `kubectl apply`
# into an in-place update and the rollout wait into a no-op; start from none.
if kubectl get namespace "$app" >/dev/null 2>&1; then
  echo "deleting namespace $app left by an earlier run" >&2
  kubectl delete namespace "$app" --wait=true --timeout=300s >&2
fi

keep_ns=0
[ "$keep" = false ] || keep_ns=1
IMAGE="$image" EVIDENCE_DIR="$out" RUN_ID="$run_id" KEEP_NAMESPACE="$keep_ns" \
  "$HARNESS" "$app"
