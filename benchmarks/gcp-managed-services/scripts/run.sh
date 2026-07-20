#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
BENCH_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
REPO_ROOT="$(cd "$BENCH_ROOT/../.." && pwd)"
PROJECT_ID="${PROJECT_ID:-axiom-502607}"
REGION="${REGION:-asia-east1}"
RUN_ID="${RUN_ID:-$(date -u +%m%d%H%M)}"
BOOTSTRAP_RUN_ID="${BOOTSTRAP_RUN_ID:-$RUN_ID}"
IMAGE_TAG="${IMAGE_TAG:-$(git -C "$REPO_ROOT" rev-parse --short=12 HEAD)-${BOOTSTRAP_RUN_ID}}"
STATE_DIR="${STATE_DIR:-/tmp/axiom-gcp-bench-${RUN_ID}}"
BOOTSTRAP_STATE_DIR="${BOOTSTRAP_STATE_DIR:-$STATE_DIR}"
EVIDENCE="${EVIDENCE:-/tmp/axiom-gcp-bench-${RUN_ID}.json}"
GCS_SOURCE_PREFIX="${GCS_SOURCE_PREFIX:-gs://${PROJECT_ID}_cloudbuild/source/axiom-bench-${BOOTSTRAP_RUN_ID}}"
cleanup_started=0

require() {
  command -v "$1" >/dev/null 2>&1 || {
    echo "required command not found: $1" >&2
    exit 1
  }
}

cleanup() {
  local ec=$?
  trap - EXIT INT TERM
  if [[ "$cleanup_started" == "0" ]]; then
    cleanup_started=1
    echo ">> destroying all GCP benchmark resources"
    if ! PROJECT_ID="$PROJECT_ID" REGION="$REGION" RUN_ID="$RUN_ID" \
      STATE_DIR="$STATE_DIR" BENCH_ROOT="$BENCH_ROOT" REGISTRY="${REGISTRY:-}" \
      BOOTSTRAP_RUN_ID="$BOOTSTRAP_RUN_ID" BOOTSTRAP_STATE_DIR="$BOOTSTRAP_STATE_DIR" \
      IMAGE_TAG="$IMAGE_TAG" GCS_SOURCE_PREFIX="$GCS_SOURCE_PREFIX" "$SCRIPT_DIR/cleanup.sh"; then
      echo "!! cleanup verification failed; preserve $STATE_DIR for an immediate retry" >&2
      ec=1
    fi
  fi
  if [[ "$ec" == "0" ]]; then
    echo ">> evidence preserved at $EVIDENCE"
  fi
  exit "$ec"
}
trap cleanup EXIT
trap 'exit 130' INT TERM

for command in terraform gcloud kubectl jq git cargo; do
  require "$command"
done
mkdir -p "$STATE_DIR" "$BOOTSTRAP_STATE_DIR"

echo ">> run=$RUN_ID bootstrap_run=$BOOTSTRAP_RUN_ID project=$PROJECT_ID region=$REGION"
echo ">> hard bounds: one Autopilot cluster, three shared data nodes, Cloud Run max=1/min=0, 30 minute Job deadline"

if [[ -z "${TAPE_CLI:-}" ]]; then
  echo ">> building Tape deployment-artifact CLI before provisioning cloud resources"
  cargo build --locked --manifest-path "$REPO_ROOT/Cargo.toml" \
    -p tape --bin tape --features operator
  TAPE_CLI="$REPO_ROOT/target/debug/tape"
fi
if [[ -z "${DEFER_CLI:-}" ]]; then
  echo ">> building Defer deployment-artifact CLI before provisioning cloud resources"
  cargo build --locked --manifest-path "$REPO_ROOT/Cargo.toml" \
    -p defer --bin defer --features operator
  DEFER_CLI="$REPO_ROOT/target/debug/defer"
fi
if [[ -z "${RELAY_CLI:-}" ]]; then
  echo ">> building Relay deployment-artifact CLI before provisioning cloud resources"
  cargo build --locked --manifest-path "$REPO_ROOT/Cargo.toml" \
    -p relay --bin relay --features operator
  RELAY_CLI="$REPO_ROOT/target/debug/relay"
fi
[[ -x "$TAPE_CLI" ]] || { echo "Tape CLI is not executable: $TAPE_CLI" >&2; exit 1; }
[[ -x "$DEFER_CLI" ]] || { echo "Defer CLI is not executable: $DEFER_CLI" >&2; exit 1; }
[[ -x "$RELAY_CLI" ]] || { echo "Relay CLI is not executable: $RELAY_CLI" >&2; exit 1; }

TF_DATA_DIR="$BOOTSTRAP_STATE_DIR/.terraform-bootstrap" terraform -chdir="$BENCH_ROOT/bootstrap" init -input=false
TF_DATA_DIR="$BOOTSTRAP_STATE_DIR/.terraform-bootstrap" terraform -chdir="$BENCH_ROOT/bootstrap" apply \
  -state="$BOOTSTRAP_STATE_DIR/bootstrap.tfstate" \
  -auto-approve \
  -var="project_id=$PROJECT_ID" \
  -var="region=$REGION" \
  -var="run_id=$BOOTSTRAP_RUN_ID"
REGISTRY="$(TF_DATA_DIR="$BOOTSTRAP_STATE_DIR/.terraform-bootstrap" terraform -chdir="$BENCH_ROOT/bootstrap" output -state="$BOOTSTRAP_STATE_DIR/bootstrap.tfstate" -raw registry)"

if [[ "${SKIP_BUILD:-0}" == "1" ]]; then
  echo ">> reusing previously built images after verifying all five tags"
  for image in receiver tape defer relay client; do
    gcloud artifacts docker images describe "$REGISTRY/$image:$IMAGE_TAG" \
      --project="$PROJECT_ID" --format='value(image_summary.digest)'
  done
else
  echo ">> building current checkout images once in Cloud Build"
  gcloud builds submit "$REPO_ROOT" \
    --project="$PROJECT_ID" \
    --region="$REGION" \
    --config="$BENCH_ROOT/cloudbuild.yaml" \
    --gcs-source-staging-dir="$GCS_SOURCE_PREFIX" \
    --ignore-file="$BENCH_ROOT/gcloudignore" \
    --substitutions="_REGISTRY=$REGISTRY,_TAG=$IMAGE_TAG"
fi

TF_DATA_DIR="$STATE_DIR/.terraform-environment" terraform -chdir="$BENCH_ROOT/environment" init -input=false
TF_DATA_DIR="$STATE_DIR/.terraform-environment" terraform -chdir="$BENCH_ROOT/environment" apply \
  -state="$STATE_DIR/environment.tfstate" \
  -auto-approve \
  -var="project_id=$PROJECT_ID" \
  -var="region=$REGION" \
  -var="run_id=$RUN_ID" \
  -var="registry=$REGISTRY" \
  -var="image_tag=$IMAGE_TAG"

tf_output="$STATE_DIR/environment-output.json"
TF_DATA_DIR="$STATE_DIR/.terraform-environment" terraform -chdir="$BENCH_ROOT/environment" output \
  -state="$STATE_DIR/environment.tfstate" -json > "$tf_output"
cluster="$(jq -r '.cluster_name.value' "$tf_output")"
gcloud container clusters get-credentials "$cluster" --project="$PROJECT_ID" --region="$REGION"

export TAPE_IMAGE="$(jq -r '.images.value.tape' "$tf_output")"
export DEFER_IMAGE="$(jq -r '.images.value.defer' "$tf_output")"
export RELAY_IMAGE="$(jq -r '.images.value.relay' "$tf_output")"
export REPO_ROOT TAPE_CLI DEFER_CLI RELAY_CLI
export WORKLOAD_MANIFEST_DIR="$STATE_DIR/workload-manifests"
"$SCRIPT_DIR/deploy-workloads.sh"

export CLIENT_IMAGE="$(jq -r '.images.value.client' "$tf_output")"
export BENCHMARK_SERVICE_ACCOUNT="$(jq -r '.benchmark_service_account.value' "$tf_output")"
export RECEIVER_URL="$(jq -r '.receiver_url.value' "$tf_output")"
export RECEIVER_SECRET
RECEIVER_SECRET="$(TF_DATA_DIR="$STATE_DIR/.terraform-environment" terraform -chdir="$BENCH_ROOT/environment" output -state="$STATE_DIR/environment.tfstate" -raw receiver_secret)"
export PUBSUB_TOPIC="$(jq -r '.pubsub_topic.value' "$tf_output")"
export PUBSUB_SUBSCRIPTIONS="$(jq -r '.pubsub_subscriptions.value | join(",")' "$tf_output")"
export CLOUD_TASKS_QUEUE="$(jq -r '.cloud_tasks_queue.value' "$tf_output")"
export PROJECT_ID REGION RUN_ID

echo ">> running real managed-service comparison"
"$SCRIPT_DIR/run-client.sh" | tee "$EVIDENCE"

kubectl top pods -A --containers > "$STATE_DIR/kubectl-top.txt" 2>/dev/null || true
kubectl -n tape exec tape-0 -- du -sb /data > "$STATE_DIR/tape-disk.txt" 2>/dev/null || true
kubectl -n defer exec defer-0 -- du -sb /data > "$STATE_DIR/defer-disk.txt" 2>/dev/null || true
kubectl -n relay exec relay-0 -- du -sb /data > "$STATE_DIR/relay-disk.txt" 2>/dev/null || true

echo ">> benchmark completed; cleanup trap is now mandatory"
