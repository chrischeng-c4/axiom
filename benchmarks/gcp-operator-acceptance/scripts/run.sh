#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ACCEPTANCE_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
REPO_ROOT="$(cd "$ACCEPTANCE_ROOT/../.." && pwd)"
: "${PROJECT_ID:?Set PROJECT_ID explicitly to the disposable GCP billing project}"
REGION="${REGION:-asia-east1}"
GKE_ZONE="${GKE_ZONE:-asia-east1-a}"
PERSISTENT_CLUSTER_NAME="${PERSISTENT_CLUSTER_NAME:-axiom-operator-acceptance}"
ARTIFACT_REGISTRY_REPOSITORY="${ARTIFACT_REGISTRY_REPOSITORY:-courier}"
RUN_ID="${RUN_ID:-$(date -u +%m%d%H%M%S)}"
GIT_SHA="$(git -C "$REPO_ROOT" rev-parse --short=12 HEAD)"
IMAGE_TAG="${IMAGE_TAG:-${GIT_SHA}-${RUN_ID}}"
REGISTRY="${REGION}-docker.pkg.dev/${PROJECT_ID}/${ARTIFACT_REGISTRY_REPOSITORY}"
STATE_DIR="${STATE_DIR:-/tmp/axiom-gcp-operator-${RUN_ID}}"
EVIDENCE_DIR="${EVIDENCE_DIR:-/tmp/axiom-gcp-operator-evidence/${RUN_ID}}"
MANIFEST_DIR="${MANIFEST_DIR:-$STATE_DIR/manifests}"
GCS_SOURCE_PREFIX="${GCS_SOURCE_PREFIX:-gs://${PROJECT_ID}_cloudbuild/source/axiom-gcp-operator-${RUN_ID}}"
MAX_CLOUD_SECONDS="${MAX_CLOUD_SECONDS:-2700}"
KUBECONFIG="${KUBECONFIG:-$STATE_DIR/kubeconfig}"
cleanup_armed=0
cleanup_started=0
watchdog_pid=""

require() {
  command -v "$1" >/dev/null 2>&1 || {
    echo "required command not found: $1" >&2
    exit 1
  }
}

require_empty_list() {
  local label="$1"
  shift
  local output
  if ! output="$("$@" 2>/dev/null)"; then
    echo "could not inventory existing $label; refusing a destructive run" >&2
    exit 1
  fi
  if [[ -n "$output" ]]; then
    echo "refusing to reuse RUN_ID=$RUN_ID; existing $label:" >&2
    echo "$output" >&2
    exit 1
  fi
}

cleanup() {
  local ec=$?
  trap - EXIT INT TERM
  if [[ -n "$watchdog_pid" ]]; then
    kill "$watchdog_pid" >/dev/null 2>&1 || true
    wait "$watchdog_pid" >/dev/null 2>&1 || true
  fi
  if [[ "$cleanup_armed" == "1" && "$cleanup_started" == "0" ]]; then
    cleanup_started=1
    if ! PROJECT_ID="$PROJECT_ID" REGION="$REGION" GKE_ZONE="$GKE_ZONE" RUN_ID="$RUN_ID" \
      STATE_DIR="$STATE_DIR" ACCEPTANCE_ROOT="$ACCEPTANCE_ROOT" \
      REGISTRY="$REGISTRY" IMAGE_TAG="$IMAGE_TAG" \
      GCS_SOURCE_PREFIX="$GCS_SOURCE_PREFIX" EVIDENCE_DIR="$EVIDENCE_DIR" \
      ARTIFACT_REGISTRY_REPOSITORY="$ARTIFACT_REGISTRY_REPOSITORY" \
      PERSISTENT_CLUSTER_NAME="$PERSISTENT_CLUSTER_NAME" \
      "$SCRIPT_DIR/cleanup.sh"; then
      echo "cleanup failed; Terraform state remains at $STATE_DIR" >&2
      ec=1
    fi
  fi
  echo "evidence: $EVIDENCE_DIR"
  exit "$ec"
}
trap cleanup EXIT
trap 'exit 130' INT
trap 'echo "45-minute cloud acceptance cap reached" >&2; exit 124' TERM

for command in cargo curl gcloud git jq kubectl terraform; do
  require "$command"
done
[[ "$RUN_ID" =~ ^[a-z0-9][a-z0-9-]{0,17}$ ]] || {
  echo "RUN_ID must be 1-18 lowercase letters, digits, or hyphens" >&2
  exit 1
}
[[ "$GKE_ZONE" =~ ^[a-z]+-[a-z]+[0-9]-[a-z]$ ]] || {
  echo "GKE_ZONE must be a zone such as asia-east1-a" >&2
  exit 1
}
[[ "$MAX_CLOUD_SECONDS" =~ ^[0-9]+$ && "$MAX_CLOUD_SECONDS" -le 2700 ]] || {
  echo "MAX_CLOUD_SECONDS must be an integer no greater than 2700" >&2
  exit 1
}

for run_dir in "$STATE_DIR" "$EVIDENCE_DIR"; do
  if [[ -e "$run_dir" && -n "$(find "$run_dir" -mindepth 1 -print -quit 2>/dev/null)" ]]; then
    echo "refusing to reuse nonempty run directory: $run_dir" >&2
    exit 1
  fi
done

mkdir -p "$STATE_DIR" "$EVIDENCE_DIR/kubernetes" "$EVIDENCE_DIR/gcs"
export KUBECONFIG
exec > >(tee -a "$EVIDENCE_DIR/run.log") 2>&1

required_apis=(
  artifactregistry.googleapis.com
  cloudbuild.googleapis.com
  compute.googleapis.com
  container.googleapis.com
  iam.googleapis.com
  iamcredentials.googleapis.com
  storage.googleapis.com
)
: > "$EVIDENCE_DIR/preexisting-apis.txt"
for api in "${required_apis[@]}"; do
  enabled="$(gcloud services list --enabled --project="$PROJECT_ID" \
    --filter="config.name=${api}" --format='value(config.name)')"
  if [[ "$enabled" != "$api" ]]; then
    echo "required API is not already enabled: $api" >&2
    echo "The harness never changes project API state; enable it explicitly before retrying." >&2
    exit 1
  fi
  printf '%s\n' "$api" >> "$EVIDENCE_DIR/preexisting-apis.txt"
done
gcloud artifacts repositories describe "$ARTIFACT_REGISTRY_REPOSITORY" \
  --project="$PROJECT_ID" --location="$REGION" --format=json \
  > "$EVIDENCE_DIR/preexisting-artifact-registry.json"
require_empty_list "backup bucket" gcloud storage buckets list --project="$PROJECT_ID" \
  --filter="name=${PROJECT_ID}-axo-${RUN_ID}-backup" --format='value(name)'
require_empty_list "backup service account" gcloud iam service-accounts list \
  --project="$PROJECT_ID" \
  --filter="email:axo-${RUN_ID}-backup@${PROJECT_ID}.iam.gserviceaccount.com" \
  --format='value(email)'
source_bucket_path="${GCS_SOURCE_PREFIX#gs://}"
source_bucket="${source_bucket_path%%/*}"
[[ -n "$source_bucket" && "$source_bucket" != "$GCS_SOURCE_PREFIX" ]] || {
  echo "GCS_SOURCE_PREFIX must be a gs://bucket/prefix URI" >&2
  exit 1
}
if ! gcloud storage buckets describe "gs://${source_bucket}" --project="$PROJECT_ID" \
  --format=json > "$EVIDENCE_DIR/preexisting-cloud-build-source-bucket.json"; then
  echo "Cloud Build source bucket must already exist; the harness will not create or leak one" >&2
  exit 1
fi
if ! gcloud storage ls --recursive "gs://${source_bucket}" \
  > "$EVIDENCE_DIR/preexisting-cloud-build-source-objects.txt"; then
  echo "could not inventory the pre-existing Cloud Build source bucket" >&2
  exit 1
fi
if rg -F "${GCS_SOURCE_PREFIX}/" "$EVIDENCE_DIR/preexisting-cloud-build-source-objects.txt" >/dev/null; then
  echo "refusing to reuse Cloud Build source prefix: $GCS_SOURCE_PREFIX" >&2
  exit 1
fi

for image in lumen sift; do
  inventory="$EVIDENCE_DIR/preexisting-${image}-images.json"
  if ! gcloud artifacts docker images list "$REGISTRY/$image" \
    --project="$PROJECT_ID" --include-tags --format=json > "$inventory"; then
    echo "could not inventory existing $image images; refusing a destructive run" >&2
    exit 1
  fi
  if jq -e --arg tag "$IMAGE_TAG" \
    'any(.[]; ((.tags // []) | index($tag)) != null)' "$inventory" >/dev/null; then
    echo "refusing to overwrite existing image tag: $REGISTRY/$image:$IMAGE_TAG" >&2
    exit 1
  fi
done

git -C "$REPO_ROOT" status --porcelain=v1 > "$EVIDENCE_DIR/source-git-status.txt"
if [[ -s "$EVIDENCE_DIR/source-git-status.txt" ]]; then
  echo "refusing Cloud Build from a dirty tree; commit the exact source before GCP acceptance" >&2
  cat "$EVIDENCE_DIR/source-git-status.txt" >&2
  exit 1
fi

echo ">> persistent Standard GKE cluster bootstrap or reuse"
PROJECT_ID="$PROJECT_ID" REGION="$REGION" GKE_ZONE="$GKE_ZONE" \
  PERSISTENT_CLUSTER_NAME="$PERSISTENT_CLUSTER_NAME" \
  "$SCRIPT_DIR/bootstrap-cluster.sh" > "$EVIDENCE_DIR/persistent-cluster-name.txt"
test "$(sed -n '1p' "$EVIDENCE_DIR/persistent-cluster-name.txt")" = "$PERSISTENT_CLUSTER_NAME"
gcloud container clusters describe "$PERSISTENT_CLUSTER_NAME" \
  --project="$PROJECT_ID" --zone="$GKE_ZONE" --format=json \
  > "$EVIDENCE_DIR/persistent-cluster.json"

jq -n \
  --arg schema "axiom.gcp.operator.run.v1" \
  --arg project_id "$PROJECT_ID" \
  --arg region "$REGION" \
  --arg gke_zone "$GKE_ZONE" \
  --arg run_id "$RUN_ID" \
  --arg git_sha "$GIT_SHA" \
  --arg image_tag "$IMAGE_TAG" \
  --arg registry "$REGISTRY" \
  --arg cluster_name "$PERSISTENT_CLUSTER_NAME" \
  --arg started_at "$(date -u +%Y-%m-%dT%H:%M:%SZ)" \
  '{schema:$schema, project_id:$project_id, region:$region, gke_zone:$gke_zone, persistent_cluster:$cluster_name, run_id:$run_id, git_sha:$git_sha, git_dirty:false, image_tag:$image_tag, registry:$registry, started_at:$started_at}' \
  > "$EVIDENCE_DIR/run.json"

echo ">> local deployment CLI build and render-surface preflight"
cargo build --locked --manifest-path "$REPO_ROOT/Cargo.toml" \
  -p lumen --bin lumen --features operator
cargo build --locked --manifest-path "$REPO_ROOT/Cargo.toml" \
  -p sift --bin sift
LUMEN_CLI="${LUMEN_CLI:-$REPO_ROOT/target/debug/lumen}"
SIFT_CLI="${SIFT_CLI:-$REPO_ROOT/target/debug/sift}"

cleanup_armed=1
(
  sleep "$MAX_CLOUD_SECONDS"
  kill -TERM "$$" >/dev/null 2>&1 || true
) &
watchdog_pid="$!"

echo ">> Cloud Build: one shared Rust stage, two immutable runtime images"
build_id="$(gcloud builds submit "$REPO_ROOT" \
  --async \
  --project="$PROJECT_ID" \
  --region="$REGION" \
  --config="$ACCEPTANCE_ROOT/cloudbuild.yaml" \
  --gcs-source-staging-dir="$GCS_SOURCE_PREFIX" \
  --ignore-file="$ACCEPTANCE_ROOT/gcloudignore" \
  --substitutions="_REGISTRY=$REGISTRY,_TAG=$IMAGE_TAG" \
  --format='value(id)')"
[[ -n "$build_id" && "$build_id" != "null" ]] || {
  echo "Cloud Build did not return a build id" >&2
  exit 1
}
printf '%s\n' "$build_id" > "$STATE_DIR/cloud-build-id.txt"
gcloud builds describe "$build_id" --project="$PROJECT_ID" --region="$REGION" \
  --format=json > "$EVIDENCE_DIR/cloud-build-submit.json"
source_object_bucket="$(jq -r '.source.storageSource.bucket' "$EVIDENCE_DIR/cloud-build-submit.json")"
source_object_name="$(jq -r '.source.storageSource.object' "$EVIDENCE_DIR/cloud-build-submit.json")"
[[ -n "$source_object_bucket" && "$source_object_bucket" != "null" && -n "$source_object_name" && "$source_object_name" != "null" ]] || {
  echo "Cloud Build did not expose its exact staged source object" >&2
  exit 1
}
gcloud storage objects describe "gs://${source_object_bucket}/${source_object_name}" \
  --format=json > "$EVIDENCE_DIR/cloud-build-source-object.json"

while true; do
  build_status="$(gcloud builds describe "$build_id" --project="$PROJECT_ID" \
    --region="$REGION" --format='value(status)')"
  printf '%s %s\n' "$(date -u +%Y-%m-%dT%H:%M:%SZ)" "$build_status" \
    >> "$EVIDENCE_DIR/cloud-build-status.log"
  case "$build_status" in
    SUCCESS) break ;;
    FAILURE|INTERNAL_ERROR|TIMEOUT|CANCELLED|EXPIRED)
      echo "Cloud Build ended with $build_status" >&2
      exit 1
      ;;
  esac
  sleep 10
done
gcloud builds describe "$build_id" --project="$PROJECT_ID" --region="$REGION" \
  --format=json > "$EVIDENCE_DIR/cloud-build-final.json"

resolve_digest() {
  local image="$1"
  local digest
  digest="$(gcloud artifacts docker images describe "$REGISTRY/$image:$IMAGE_TAG" \
    --project="$PROJECT_ID" --format='value(image_summary.digest)')"
  [[ "$digest" == sha256:* ]] || {
    echo "could not resolve immutable digest for $image:$IMAGE_TAG" >&2
    return 1
  }
  printf '%s@%s\n' "$REGISTRY/$image" "$digest"
}

LUMEN_IMAGE="$(resolve_digest lumen)"
SIFT_IMAGE="$(resolve_digest sift)"
jq -n --arg lumen "$LUMEN_IMAGE" --arg sift "$SIFT_IMAGE" \
  '{lumen:$lumen,sift:$sift}' > "$EVIDENCE_DIR/images.json"

# Resource names are deterministic Terraform values, so render and validate all
# app-owned Kubernetes layers before creating the cluster.
BACKUP_BUCKET="${PROJECT_ID}-axo-${RUN_ID}-backup"
BACKUP_GSA_EMAIL="axo-${RUN_ID}-backup@${PROJECT_ID}.iam.gserviceaccount.com"
GKE_CLUSTER_NAME="axo-${RUN_ID}-gke"
export LUMEN_CLI SIFT_CLI LUMEN_IMAGE SIFT_IMAGE BACKUP_BUCKET BACKUP_GSA_EMAIL
export GKE_CLUSTER_NAME GKE_ZONE PROJECT_ID REGION
export RUN_ID MANIFEST_DIR
"$SCRIPT_DIR/render-manifests.sh"

echo ">> Terraform: run-scoped backup bucket and workload identity on persistent Standard GKE"
TF_DATA_DIR="$STATE_DIR/.terraform-environment" terraform \
  -chdir="$ACCEPTANCE_ROOT/environment" init -input=false
TF_DATA_DIR="$STATE_DIR/.terraform-environment" terraform \
  -chdir="$ACCEPTANCE_ROOT/environment" apply \
  -state="$STATE_DIR/environment.tfstate" \
  -auto-approve \
  -var="project_id=$PROJECT_ID" \
  -var="region=$REGION" \
  -var="gke_zone=$GKE_ZONE" \
  -var="cluster_name=$PERSISTENT_CLUSTER_NAME" \
  -var="run_id=$RUN_ID" \
  -var="artifact_registry_repository=$ARTIFACT_REGISTRY_REPOSITORY" \
  -var="image_tag=$IMAGE_TAG"
TF_DATA_DIR="$STATE_DIR/.terraform-environment" terraform \
  -chdir="$ACCEPTANCE_ROOT/environment" output \
  -state="$STATE_DIR/environment.tfstate" -json > "$EVIDENCE_DIR/terraform-output.json"

cluster="$(jq -r '.cluster_name.value' "$EVIDENCE_DIR/terraform-output.json")"
test "$(jq -r '.gke_zone.value' "$EVIDENCE_DIR/terraform-output.json")" = "$GKE_ZONE"
test "$(jq -r '.cluster_name.value' "$EVIDENCE_DIR/terraform-output.json")" = "$PERSISTENT_CLUSTER_NAME"
test "$(jq -r '.backup_bucket.value' "$EVIDENCE_DIR/terraform-output.json")" = "$BACKUP_BUCKET"
test "$(jq -r '.backup_gsa_email.value' "$EVIDENCE_DIR/terraform-output.json")" = "$BACKUP_GSA_EMAIL"
gcloud container clusters get-credentials "$cluster" \
  --project="$PROJECT_ID" --zone="$GKE_ZONE"
printf '%s\n' "$cluster" > "$STATE_DIR/kube-context-ready.txt"

export EVIDENCE_DIR

# Phase 1 is a hard gate: no Sift CRD/operator/instance/collector is applied
# until Lumen has independently reconciled, recovered, backed up to GCS, and
# completed its bounded disk-triggered split.
"$SCRIPT_DIR/deploy.sh" lumen
"$SCRIPT_DIR/verify-operator-cell.sh" lumen
export PROJECT_ID REGION BACKUP_BUCKET
"$SCRIPT_DIR/verify-lumen.sh"

# Only a successful Lumen phase starts the Sift data plane. The collector then
# reads Lumen's structured stdout from Standard GKE node logs and the proof
# queries the materialized Sift logging store.
"$SCRIPT_DIR/deploy.sh" sift
"$SCRIPT_DIR/verify-operator-cell.sh" sift
"$SCRIPT_DIR/verify-sift-collection.sh"

echo ">> acceptance passed; mandatory cleanup runs on EXIT"
