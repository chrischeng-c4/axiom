#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ACCEPTANCE_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
REPO_ROOT="$(cd "$ACCEPTANCE_ROOT/../.." && pwd)"
source "$SCRIPT_DIR/process-tree.sh"
if [[ "${AXIOM_GCP_ACCEPTANCE_ISOLATED_SESSION:-0}" != "1" ]]; then
  command -v python3 >/dev/null 2>&1 || {
    echo "python3 is required to isolate the acceptance process group" >&2
    exit 1
  }
  export AXIOM_GCP_ACCEPTANCE_ISOLATED_SESSION=1
  exec python3 -c '
import os
import signal
import subprocess
import sys

child = None
pending_signal = None

def forward(signum, _frame):
    global pending_signal
    pending_signal = signum
    if child is not None:
        try:
            os.killpg(child.pid, signum)
        except ProcessLookupError:
            pass

signal.signal(signal.SIGINT, forward)
signal.signal(signal.SIGTERM, forward)
child = subprocess.Popen(sys.argv[1:], start_new_session=True)
if pending_signal is not None:
    forward(pending_signal, None)
status = child.wait()
try:
    os.killpg(child.pid, signal.SIGKILL)
except ProcessLookupError:
    pass
sys.exit(128 + (-status) if status < 0 else status)
' "$SCRIPT_DIR/run.sh" "$@"
fi
run_process_group="$(process_group_id "$$")"
[[ "$run_process_group" == "$$" ]] || {
  echo "acceptance run is not the leader of its isolated process group" >&2
  exit 1
}
: "${PROJECT_ID:?Set PROJECT_ID explicitly to the disposable GCP billing project}"
REGION="${REGION:-asia-east1}"
GKE_ZONE="${GKE_ZONE:-asia-east1-a}"
PERSISTENT_CLUSTER_NAME="${PERSISTENT_CLUSTER_NAME:-axiom-operator-acceptance}"
ARTIFACT_REGISTRY_REPOSITORY="${ARTIFACT_REGISTRY_REPOSITORY:-courier}"
RUN_ID="${RUN_ID:-$(date -u +%m%d%H%M%S)}"
GIT_SHA="$(git -c core.fsmonitor=false -C "$REPO_ROOT" rev-parse HEAD)"
IMAGE_TAG="${IMAGE_TAG:-${GIT_SHA}-${RUN_ID}}"
REGISTRY="${REGION}-docker.pkg.dev/${PROJECT_ID}/${ARTIFACT_REGISTRY_REPOSITORY}"
ACCEPTANCE_APPS="${ACCEPTANCE_APPS:-lumen sift}"
INPUT_LUMEN_IMAGE="${LUMEN_IMAGE:-}"
INPUT_SIFT_IMAGE="${SIFT_IMAGE:-}"
INPUT_RIG_IMAGE="${RIG_IMAGE:-}"
INPUT_TAPE_IMAGE="${TAPE_IMAGE:-}"
INPUT_SIFT_CLI="${SIFT_CLI:-}"
LUMEN_PRIOR_ACCEPTANCE="${LUMEN_PRIOR_ACCEPTANCE:-}"
LUMEN_AUTH_ISSUER_GSA="${LUMEN_AUTH_ISSUER_GSA:-}"
# Pre-declared so `export` under `set -u` never fails; each mode branch below
# fills in only the names it owns. Sift-only acceptance rejects a caller CLI;
# the other modes keep their existing local CLI override behavior. The
# *_IMAGE runtime variables are reset because their caller inputs were already
# captured into INPUT_* above.
LUMEN_CLI="${LUMEN_CLI:-}"
SIFT_CLI=""
TAPE_CLI="${TAPE_CLI:-}"
LUMEN_IMAGE=""
SIFT_IMAGE=""
TAPE_IMAGE=""
RIG_IMAGE=""
STATE_DIR="${STATE_DIR:-/tmp/axiom-gcp-operator-${RUN_ID}}"
EVIDENCE_DIR="${EVIDENCE_DIR:-/tmp/axiom-gcp-operator-evidence/${RUN_ID}}"
MANIFEST_DIR="${MANIFEST_DIR:-$STATE_DIR/manifests}"
TERRAFORM_ENVIRONMENT_DIR="${TERRAFORM_ENVIRONMENT_DIR:-$STATE_DIR/environment}"
GCS_SOURCE_PREFIX="${GCS_SOURCE_PREFIX:-gs://${PROJECT_ID}_cloudbuild/source/axiom-gcp-operator-${RUN_ID}}"
sift_cloud_cap=5400
if [[ "$ACCEPTANCE_APPS" == "sift" ]]; then
  default_cloud_seconds="$sift_cloud_cap"
else
  default_cloud_seconds=2700
fi
MAX_CLOUD_SECONDS="${MAX_CLOUD_SECONDS:-$default_cloud_seconds}"
KUBECONFIG="${KUBECONFIG:-$STATE_DIR/kubeconfig}"
cleanup_armed=0
cleanup_started=0
watchdog_pid=""
watchdog_descendants="$STATE_DIR/watchdog-descendants.txt"
watchdog_pid_file="$STATE_DIR/watchdog-pid.txt"
CANDIDATE_SOURCE_ARCHIVE=""
CANDIDATE_SOURCE_SHA256=""
CANDIDATE_CLOUD_BUILD_ID=""
CANDIDATE_SOURCE_URI=""
# Completion sentinel: bash expansion errors (set -u unbound variable)
# abort the script WITHOUT updating $?, so the EXIT trap's `local ec=$?`
# reads the PREVIOUS command's 0 — a false-green exit (runs 0724151638
# and 0724153400 both died mid-run yet exited 0). The trap therefore
# refuses ec=0 unless the last line of the script body really ran.
run_completed=0

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
  local watchdog_was_started=0
  local scan_attempt
  if [[ "$ec" -eq 0 && "$run_completed" != "1" ]]; then
    echo "run aborted before completion (likely an expansion error above) — forcing failure exit" >&2
    ec=1
  fi
  trap - EXIT INT TERM
  if [[ -n "$watchdog_pid" ]]; then
    watchdog_was_started=1
    kill "$watchdog_pid" >/dev/null 2>&1 || true
    wait "$watchdog_pid" >/dev/null 2>&1 || true
    watchdog_pid=""
  fi
  if [[ "$watchdog_was_started" == "1" || -f "$watchdog_descendants" ]]; then
    # The foreground command can exit as soon as TERM reaches it. That starts
    # this EXIT trap while the watchdog is still in its grace period. Rescan
    # the isolated group here after stopping the watchdog, so a TERM handler
    # cannot fork and reparent an unrecorded child between the two paths.
    for scan_attempt in 1 2 3 4 5; do
      append_process_group_members \
        "$run_process_group" "$$" "" "$watchdog_descendants"
      signal_recorded_processes "$watchdog_descendants" KILL
      [[ "$scan_attempt" == "5" ]] || sleep 1
    done
  fi
  if [[ "$cleanup_armed" == "1" && "$cleanup_started" == "0" ]]; then
    cleanup_started=1
    if ! PROJECT_ID="$PROJECT_ID" REGION="$REGION" GKE_ZONE="$GKE_ZONE" RUN_ID="$RUN_ID" \
      STATE_DIR="$STATE_DIR" ACCEPTANCE_ROOT="$ACCEPTANCE_ROOT" \
      TERRAFORM_ENVIRONMENT_DIR="$TERRAFORM_ENVIRONMENT_DIR" \
      REGISTRY="$REGISTRY" IMAGE_TAG="$IMAGE_TAG" \
      GCS_SOURCE_PREFIX="$GCS_SOURCE_PREFIX" EVIDENCE_DIR="$EVIDENCE_DIR" \
      ARTIFACT_REGISTRY_REPOSITORY="$ARTIFACT_REGISTRY_REPOSITORY" \
      PERSISTENT_CLUSTER_NAME="$PERSISTENT_CLUSTER_NAME" \
      ACCEPTANCE_APPS="$ACCEPTANCE_APPS" \
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
if [[ "$ACCEPTANCE_APPS" == "sift" ]]; then
  trap 'echo "90-minute Sift MVP cloud acceptance cap reached" >&2; exit 124' TERM
else
  trap 'echo "45-minute cloud acceptance cap reached" >&2; exit 124' TERM
fi

for command in awk cargo curl gcloud git gzip jq kubectl openssl ps python3 sort tar terraform; do
  require "$command"
done
python3 -c 'import jsonschema' >/dev/null 2>&1 || {
  echo "python package 'jsonschema' is required for Sift evidence validation" >&2
  exit 1
}
[[ "$RUN_ID" =~ ^[a-z0-9][a-z0-9-]{0,17}$ ]] || {
  echo "RUN_ID must be 1-18 lowercase letters, digits, or hyphens" >&2
  exit 1
}
[[ "$GKE_ZONE" =~ ^[a-z]+-[a-z]+[0-9]-[a-z]$ ]] || {
  echo "GKE_ZONE must be a zone such as asia-east1-a" >&2
  exit 1
}
[[ "$MAX_CLOUD_SECONDS" =~ ^[0-9]+$ && "$MAX_CLOUD_SECONDS" -le "$default_cloud_seconds" ]] || {
  echo "MAX_CLOUD_SECONDS must be an integer no greater than $default_cloud_seconds for this mode" >&2
  exit 1
}
case "$ACCEPTANCE_APPS" in
  "lumen sift") acceptance_mode="lumen-sift" ;;
  "lumen auth") acceptance_mode="lumen-auth" ;;
  "sift") acceptance_mode="sift" ;;
  "tape") acceptance_mode="tape" ;;
  *)
    echo "ACCEPTANCE_APPS must be exactly 'lumen sift' (default), 'lumen auth', 'sift', or 'tape'" >&2
    exit 1
    ;;
esac
if [[ "$acceptance_mode" != "tape" && "$acceptance_mode" != "sift" ]]; then
  : "${LUMEN_AUTH_ISSUER_GSA:?LUMEN_AUTH_ISSUER_GSA is required for auth-running modes}"
fi
if [[ "$acceptance_mode" == "sift" ]]; then
  [[ -z "$INPUT_SIFT_IMAGE" && -z "$INPUT_RIG_IMAGE" && -z "$INPUT_SIFT_CLI" ]] || {
    echo "ACCEPTANCE_APPS=sift builds Sift, Rig, and the local CLI from one clean candidate archive; prebuilt images are not accepted, and a caller-supplied SIFT_CLI is not accepted" >&2
    exit 1
  }
  IMAGE_PROVENANCE="cloud-build"
  [[ -z "$LUMEN_PRIOR_ACCEPTANCE" ]] || {
    echo "LUMEN_PRIOR_ACCEPTANCE is meaningless in ACCEPTANCE_APPS=sift mode" >&2
    exit 1
  }
elif [[ "$acceptance_mode" == "tape" ]]; then
  [[ -z "$INPUT_TAPE_IMAGE" || "$INPUT_TAPE_IMAGE" == *@sha256:* ]] || {
    echo "caller-supplied service images must be immutable @sha256 digest references" >&2
    exit 1
  }
  if [[ -n "$INPUT_TAPE_IMAGE" ]]; then
    IMAGE_PROVENANCE="prebuilt"
  else
    IMAGE_PROVENANCE="cloud-build"
  fi
  [[ -z "$LUMEN_PRIOR_ACCEPTANCE" ]] || {
    echo "LUMEN_PRIOR_ACCEPTANCE is meaningless in ACCEPTANCE_APPS=tape mode" >&2
    exit 1
  }
else
  if [[ "$acceptance_mode" == "lumen-auth" ]]; then
    [[ -z "$INPUT_LUMEN_IMAGE" || "$INPUT_LUMEN_IMAGE" == *@sha256:* ]] || {
      echo "caller-supplied service images must be immutable @sha256 digest references" >&2
      exit 1
    }
    IMAGE_PROVENANCE="prebuilt"
    [[ -n "$INPUT_LUMEN_IMAGE" ]] || IMAGE_PROVENANCE="cloud-build"
    [[ -z "$LUMEN_PRIOR_ACCEPTANCE" ]] || {
      echo "LUMEN_PRIOR_ACCEPTANCE is not used in ACCEPTANCE_APPS=lumen auth mode" >&2
      exit 1
    }
  elif [[ "$acceptance_mode" != "tape" ]]; then
  for input_image in "$INPUT_LUMEN_IMAGE" "$INPUT_SIFT_IMAGE"; do
    [[ -z "$input_image" || "$input_image" == *@sha256:* ]] || {
      echo "caller-supplied service images must be immutable @sha256 digest references" >&2
      exit 1
    }
  done
  if [[ -n "$INPUT_LUMEN_IMAGE" && -n "$INPUT_SIFT_IMAGE" ]]; then
    IMAGE_PROVENANCE="prebuilt"
  elif [[ -n "$INPUT_LUMEN_IMAGE" || -n "$INPUT_SIFT_IMAGE" ]]; then
    IMAGE_PROVENANCE="mixed"
  else
    IMAGE_PROVENANCE="cloud-build"
  fi
  fi
fi
if [[ -n "$LUMEN_PRIOR_ACCEPTANCE" ]]; then
  [[ -f "$LUMEN_PRIOR_ACCEPTANCE" ]] || {
    echo "LUMEN_PRIOR_ACCEPTANCE must name an existing Lumen acceptance JSON file" >&2
    exit 1
  }
  jq -e '
    .schema == "axiom.gcp.lumen.acceptance.v1"
    and .operator_reconcile_1x1 == "passed"
    and .pod_restart_data_retention == "passed"
    and .gcs_backup_before_split == "passed"
    and .auto_split_delta == 1
  ' "$LUMEN_PRIOR_ACCEPTANCE" >/dev/null || {
    echo "LUMEN_PRIOR_ACCEPTANCE is not a completed Lumen acceptance proof" >&2
    exit 1
  }
fi

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
if [[ "$acceptance_mode" != "lumen-auth" ]]; then
  require_empty_list "backup bucket" gcloud storage buckets list --project="$PROJECT_ID" \
    --filter="name=${PROJECT_ID}-axo-${RUN_ID}-backup" --format='value(name)'
  require_empty_list "backup service account" gcloud iam service-accounts list \
    --project="$PROJECT_ID" \
    --filter="email:axo-${RUN_ID}-backup@${PROJECT_ID}.iam.gserviceaccount.com" \
    --format='value(email)'
fi
if [[ "$IMAGE_PROVENANCE" != "prebuilt" ]]; then
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
fi

if [[ "$acceptance_mode" == "sift" ]]; then
  image_list=(sift rig)
elif [[ "$acceptance_mode" == "tape" ]]; then
  image_list=(tape)
elif [[ "$acceptance_mode" == "lumen-auth" ]]; then
  image_list=(lumen)
else
  image_list=(lumen sift)
fi
for image in "${image_list[@]}"; do
  inventory="$EVIDENCE_DIR/preexisting-${image}-images.json"
  list_stderr="$STATE_DIR/preexisting-${image}-images.stderr"
  if gcloud artifacts docker images list "$REGISTRY/$image" \
    --project="$PROJECT_ID" --include-tags --format=json > "$inventory" 2>"$list_stderr"; then
    :
  elif [[ "$image" == "tape" || "$image" == "rig" ]] && rg -F "NOT_FOUND" "$list_stderr" >/dev/null; then
    # A test-only package may not exist in the registry on its first run.
    printf '[]' > "$inventory"
  else
    echo "could not inventory existing $image images; refusing a destructive run" >&2
    cat "$list_stderr" >&2
    exit 1
  fi
  if jq -e --arg tag "$IMAGE_TAG" \
    'any(.[]; ((.tags // []) | index($tag)) != null)' "$inventory" >/dev/null; then
    echo "refusing to overwrite existing image tag: $REGISTRY/$image:$IMAGE_TAG" >&2
    exit 1
  fi
done

git -c core.fsmonitor=false -C "$REPO_ROOT" status --porcelain=v1 \
  > "$EVIDENCE_DIR/source-git-status.txt"
if [[ -s "$EVIDENCE_DIR/source-git-status.txt" ]]; then
  echo "refusing Cloud Build from a dirty tree; commit the exact source before GCP acceptance" >&2
  cat "$EVIDENCE_DIR/source-git-status.txt" >&2
  exit 1
fi

if [[ "$acceptance_mode" == "sift" ]]; then
  CANDIDATE_SOURCE_ARCHIVE="$STATE_DIR/sift-candidate-${GIT_SHA}.tar.gz"
  candidate_source_dir="$STATE_DIR/sift-candidate-source"
  candidate_target_dir="$STATE_DIR/sift-candidate-target"
  git -c core.fsmonitor=false -C "$REPO_ROOT" archive \
    --format=tar.gz --output="$CANDIDATE_SOURCE_ARCHIVE" "$GIT_SHA"
  chmod 0400 "$CANDIDATE_SOURCE_ARCHIVE"
  CANDIDATE_SOURCE_SHA256="$(
    openssl dgst -sha256 "$CANDIDATE_SOURCE_ARCHIVE" | awk '{print $NF}'
  )"
  [[ "$CANDIDATE_SOURCE_SHA256" =~ ^[0-9a-f]{64}$ ]] || {
    echo "could not calculate the candidate source archive SHA-256" >&2
    exit 1
  }
  mkdir -p "$candidate_source_dir" "$candidate_target_dir"
  tar -xzf "$CANDIDATE_SOURCE_ARCHIVE" -C "$candidate_source_dir"
  chmod -R a-w "$candidate_source_dir"
  jq -n \
    --arg git_sha "$GIT_SHA" \
    --arg source_archive "$CANDIDATE_SOURCE_ARCHIVE" \
    --arg source_bundle_sha256 "$CANDIDATE_SOURCE_SHA256" \
    --argjson source_bundle_bytes "$(wc -c < "$CANDIDATE_SOURCE_ARCHIVE" | tr -d ' ')" \
    '{git_sha:$git_sha,source_archive:$source_archive,source_bundle_sha256:$source_bundle_sha256,source_bundle_bytes:$source_bundle_bytes}' \
    > "$EVIDENCE_DIR/candidate-source.json"
fi

echo ">> local deployment CLI build and render-surface preflight"
if [[ "$acceptance_mode" == "sift" ]]; then
  SIFT_SOURCE_REVISION="$GIT_SHA" cargo build --locked \
    --manifest-path "$candidate_source_dir/Cargo.toml" \
    --target-dir "$candidate_target_dir" -p sift --bin sift
  SIFT_CLI="$candidate_target_dir/debug/sift"
elif [[ "$acceptance_mode" == "tape" ]]; then
  cargo build --locked --manifest-path "$REPO_ROOT/Cargo.toml" \
    -p tape --bin tape --features "operator backup"
  TAPE_CLI="${TAPE_CLI:-$REPO_ROOT/target/debug/tape}"
else
  # `delegated-auth` alongside `operator` (#2879): the auth leg drives `lumen
  # query --client-sa`, whose TokenRequest minter is behind that feature. A CLI
  # built without it refuses the flag rather than minting, so the auth leg would
  # fail on a build flag instead of on the contract it exists to check.
  cargo build --locked --manifest-path "$REPO_ROOT/Cargo.toml" \
    -p lumen --bin lumen --features "operator delegated-auth"
  LUMEN_CLI="${LUMEN_CLI:-$REPO_ROOT/target/debug/lumen}"
  if [[ "$acceptance_mode" != "lumen-auth" ]]; then
    cargo build --locked --manifest-path "$REPO_ROOT/Cargo.toml" \
      -p sift --bin sift
    SIFT_CLI="${SIFT_CLI:-$REPO_ROOT/target/debug/sift}"
  fi
fi

cleanup_armed=1

# The watchdog polls its parent between short sleeps and disarms itself the
# moment the parent is gone. A single long sleep followed by an unconditional
# `kill -TERM $$` outlives a parent that died without reaching cleanup(), and
# minutes later fires at whatever process RECYCLED the parent's pid — attempt
# 0723113842 was killed exactly that way by a prior run's leftover.
run_main_pid="$$"
(
  watchdog_self="$(wait_for_process_id_file "$watchdog_pid_file" "$run_main_pid")" \
    || exit 0
  waited=0
  while (( waited < MAX_CLOUD_SECONDS )); do
    sleep 10
    waited=$((waited + 10))
    kill -0 "$run_main_pid" >/dev/null 2>&1 || exit 0
  done
  record_process_group_members \
    "$run_process_group" "$run_main_pid" "$watchdog_self" "$watchdog_descendants"
  signal_recorded_processes "$watchdog_descendants" TERM
  sleep 10
  append_process_group_members \
    "$run_process_group" "$run_main_pid" "$watchdog_self" "$watchdog_descendants"
  signal_recorded_processes "$watchdog_descendants" KILL
  kill -TERM "$run_main_pid" >/dev/null 2>&1 || true
) &
watchdog_pid="$!"
printf '%s\n' "$watchdog_pid" > "${watchdog_pid_file}.tmp"
mv "${watchdog_pid_file}.tmp" "$watchdog_pid_file"

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

if [[ "$IMAGE_PROVENANCE" == "prebuilt" ]]; then
  echo ">> using caller-supplied immutable release or candidate images"
  if [[ "$acceptance_mode" == "sift" ]]; then
    SIFT_IMAGE="$INPUT_SIFT_IMAGE"
    RIG_IMAGE="$INPUT_RIG_IMAGE"
  elif [[ "$acceptance_mode" == "tape" ]]; then
    TAPE_IMAGE="$INPUT_TAPE_IMAGE"
  elif [[ "$acceptance_mode" == "lumen-auth" ]]; then
    LUMEN_IMAGE="$INPUT_LUMEN_IMAGE"
  else
    LUMEN_IMAGE="$INPUT_LUMEN_IMAGE"
    SIFT_IMAGE="$INPUT_SIFT_IMAGE"
  fi
else
  if [[ "$acceptance_mode" == "sift" ]]; then
    CLOUD_BUILD_CONFIG="$candidate_source_dir/acceptance/gcp/cloudbuild.sift-mvp.yaml"
  elif [[ "$acceptance_mode" == "tape" ]]; then
    CLOUD_BUILD_CONFIG="$ACCEPTANCE_ROOT/cloudbuild.tape.yaml"
  elif [[ "$acceptance_mode" == "lumen-auth" ]]; then
    CLOUD_BUILD_CONFIG="$ACCEPTANCE_ROOT/cloudbuild.lumen.yaml"
  else
    CLOUD_BUILD_CONFIG="$ACCEPTANCE_ROOT/cloudbuild.yaml"
    if [[ -n "$INPUT_LUMEN_IMAGE" ]]; then
      CLOUD_BUILD_CONFIG="$ACCEPTANCE_ROOT/cloudbuild.sift.yaml"
    elif [[ -n "$INPUT_SIFT_IMAGE" ]]; then
      CLOUD_BUILD_CONFIG="$ACCEPTANCE_ROOT/cloudbuild.lumen.yaml"
    fi
  fi
  cloud_build_substitutions="_REGISTRY=$REGISTRY,_TAG=$IMAGE_TAG,_RUN_ID=$RUN_ID"
  case "${CLOUD_BUILD_CONFIG##*/}" in
    cloudbuild.sift-mvp.yaml|cloudbuild.sift.yaml|cloudbuild.yaml)
      cloud_build_substitutions="${cloud_build_substitutions},_GIT_SHA=$GIT_SHA"
      ;;
  esac
  if [[ "$acceptance_mode" == "sift" ]]; then
    cloud_build_substitutions="${cloud_build_substitutions},_SOURCE_BUNDLE_SHA256=$CANDIDATE_SOURCE_SHA256"
  fi
  echo ">> Cloud Build: source candidate only for service image(s) not supplied by digest"
  if [[ "$acceptance_mode" == "sift" ]]; then
    build_id="$(gcloud builds submit "$CANDIDATE_SOURCE_ARCHIVE" \
      --async \
      --project="$PROJECT_ID" \
      --region="$REGION" \
      --config="$CLOUD_BUILD_CONFIG" \
      --gcs-source-staging-dir="$GCS_SOURCE_PREFIX" \
      --substitutions="$cloud_build_substitutions" \
      --format='value(id)')"
  else
    build_id="$(gcloud builds submit "$REPO_ROOT" \
      --async \
      --project="$PROJECT_ID" \
      --region="$REGION" \
      --config="$CLOUD_BUILD_CONFIG" \
      --gcs-source-staging-dir="$GCS_SOURCE_PREFIX" \
      --ignore-file="$ACCEPTANCE_ROOT/gcloudignore" \
      --substitutions="$cloud_build_substitutions" \
      --format='value(id)')"
  fi
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
  if [[ "$acceptance_mode" == "sift" ]]; then
    staged_source="$STATE_DIR/cloud-build-staged-source.tar.gz"
    gcloud storage cp "gs://${source_object_bucket}/${source_object_name}" \
      "$staged_source" >/dev/null
    staged_source_sha256="$(
      openssl dgst -sha256 "$staged_source" | awk '{print $NF}'
    )"
    [[ "$staged_source_sha256" == "$CANDIDATE_SOURCE_SHA256" ]] || {
      echo "Cloud Build staged source does not match the fixed candidate archive" >&2
      exit 1
    }
    CANDIDATE_CLOUD_BUILD_ID="$build_id"
    CANDIDATE_SOURCE_URI="gs://${source_object_bucket}/${source_object_name}"
    jq -n \
      --arg build_id "$CANDIDATE_CLOUD_BUILD_ID" \
      --arg git_sha "$GIT_SHA" \
      --arg source_uri "$CANDIDATE_SOURCE_URI" \
      --arg source_bundle_sha256 "$CANDIDATE_SOURCE_SHA256" \
      '{build_id:$build_id,git_sha:$git_sha,source_uri:$source_uri,source_bundle_sha256:$source_bundle_sha256,staged_source_sha256:$source_bundle_sha256}' \
      > "$EVIDENCE_DIR/cloud-build-source-binding.json"
  fi

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
  if [[ "$acceptance_mode" == "sift" ]]; then
    SIFT_IMAGE="$(resolve_digest sift)"
    RIG_IMAGE="$(resolve_digest rig)"
  elif [[ "$acceptance_mode" == "tape" ]]; then
    TAPE_IMAGE="$(resolve_digest tape)"
  elif [[ "$acceptance_mode" == "lumen-auth" ]]; then
    LUMEN_IMAGE="$(resolve_digest lumen)"
  else
    if [[ -n "$INPUT_LUMEN_IMAGE" ]]; then
      LUMEN_IMAGE="$INPUT_LUMEN_IMAGE"
    else
      LUMEN_IMAGE="$(resolve_digest lumen)"
    fi
    if [[ -n "$INPUT_SIFT_IMAGE" ]]; then
      SIFT_IMAGE="$INPUT_SIFT_IMAGE"
    else
      SIFT_IMAGE="$(resolve_digest sift)"
    fi
  fi
fi
if [[ "$acceptance_mode" == "sift" ]]; then
  source_object_bucket="$(jq -r '.source.storageSource.bucket' "$EVIDENCE_DIR/cloud-build-submit.json")"
  source_object_name="$(jq -r '.source.storageSource.object' "$EVIDENCE_DIR/cloud-build-submit.json")"
  jq -e \
    --arg git_sha "$GIT_SHA" \
    --arg run_id "$RUN_ID" \
    --arg source_bundle_sha256 "$CANDIDATE_SOURCE_SHA256" \
    --arg source_bucket "$source_object_bucket" \
    --arg source_object "$source_object_name" \
    --arg run_tag "axiom-run-${RUN_ID}" \
    --arg source_tag "axiom-source-${CANDIDATE_SOURCE_SHA256}" '
      .status == "SUCCESS"
      and .substitutions._GIT_SHA == $git_sha
      and .substitutions._RUN_ID == $run_id
      and .substitutions._SOURCE_BUNDLE_SHA256 == $source_bundle_sha256
      and .source.storageSource.bucket == $source_bucket
      and .source.storageSource.object == $source_object
      and ((.tags // []) | index("sift-mvp") != null)
      and ((.tags // []) | index($run_tag) != null)
      and ((.tags // []) | index($source_tag) != null)
    ' "$EVIDENCE_DIR/cloud-build-final.json" >/dev/null || {
      echo "Cloud Build final receipt is not bound to this candidate source and run" >&2
      exit 1
    }
  for built_image in sift rig; do
    if [[ "$built_image" == "sift" ]]; then
      digest_ref="$SIFT_IMAGE"
    else
      digest_ref="$RIG_IMAGE"
    fi
    jq -e \
      --arg name "$REGISTRY/$built_image:$IMAGE_TAG" \
      --arg digest "${digest_ref##*@}" '
        any(.results.images[]?; .name == $name and .digest == $digest)
      ' "$EVIDENCE_DIR/cloud-build-final.json" >/dev/null || {
      echo "Cloud Build receipt does not contain the deployed $built_image digest" >&2
      exit 1
    }
  done
fi
if [[ "$acceptance_mode" == "sift" ]]; then
  jq -n --arg sift "$SIFT_IMAGE" --arg rig "$RIG_IMAGE" \
    '{sift:$sift,rig:$rig}' > "$EVIDENCE_DIR/images.json"
elif [[ "$acceptance_mode" == "tape" ]]; then
  jq -n --arg tape "$TAPE_IMAGE" '{tape:$tape}' > "$EVIDENCE_DIR/images.json"
elif [[ "$acceptance_mode" == "lumen-auth" ]]; then
  jq -n --arg lumen "$LUMEN_IMAGE" '{lumen:$lumen}' > "$EVIDENCE_DIR/images.json"
else
  jq -n --arg lumen "$LUMEN_IMAGE" --arg sift "$SIFT_IMAGE" \
    '{lumen:$lumen,sift:$sift}' > "$EVIDENCE_DIR/images.json"
fi

# Resource names are deterministic Terraform values, so render and validate all
# app-owned Kubernetes layers before creating the cluster.
if [[ "$acceptance_mode" != "lumen-auth" ]]; then
  BACKUP_BUCKET="${PROJECT_ID}-axo-${RUN_ID}-backup"
  BACKUP_GSA_EMAIL="axo-${RUN_ID}-backup@${PROJECT_ID}.iam.gserviceaccount.com"
  export BACKUP_BUCKET BACKUP_GSA_EMAIL
fi
# The cluster this run actually uses. It was `axo-${RUN_ID}-gke` — a name left
# over from the disposable-cluster era that has not existed since the run moved
# to the persistent cluster, so anything reading it (the Sift collector's
# `clusterName`, and now the placement leg's node-pool drift check) was reading
# a cluster that is not there. Line 491 already asserts the Terraform output
# equals PERSISTENT_CLUSTER_NAME, so this is the same value, named once.
GKE_CLUSTER_NAME="$PERSISTENT_CLUSTER_NAME"
export GKE_CLUSTER_NAME GKE_ZONE PROJECT_ID REGION
export RUN_ID MANIFEST_DIR ACCEPTANCE_APPS
# Export mode-specific CLIs and images only
if [[ "$acceptance_mode" == "sift" ]]; then
  CANDIDATE_GIT_SHA="$GIT_SHA"
  export SIFT_CLI SIFT_IMAGE RIG_IMAGE CANDIDATE_GIT_SHA
  export CANDIDATE_SOURCE_SHA256 CANDIDATE_CLOUD_BUILD_ID CANDIDATE_SOURCE_URI
elif [[ "$acceptance_mode" == "tape" ]]; then
  export TAPE_CLI TAPE_IMAGE
elif [[ "$acceptance_mode" == "lumen-auth" ]]; then
  export LUMEN_CLI LUMEN_IMAGE LUMEN_AUTH_ISSUER_GSA
else
  export LUMEN_CLI LUMEN_IMAGE SIFT_CLI SIFT_IMAGE
fi
"$SCRIPT_DIR/render-manifests.sh" || {
  echo "manifest rendering failed" >&2
  exit 1
}

echo ">> persistent Standard GKE cluster bootstrap or reuse"
PROJECT_ID="$PROJECT_ID" REGION="$REGION" GKE_ZONE="$GKE_ZONE" \
  PERSISTENT_CLUSTER_NAME="$PERSISTENT_CLUSTER_NAME" \
  "$SCRIPT_DIR/bootstrap-cluster.sh" > "$EVIDENCE_DIR/persistent-cluster-name.txt"
# The whole file, not just its first line: bootstrap-cluster.sh contracts to
# emit the cluster name and nothing else, so anything extra means its stdout
# got polluted and the name we are about to trust is not the name it produced.
bootstrapped_cluster="$(cat "$EVIDENCE_DIR/persistent-cluster-name.txt")"
[[ "$bootstrapped_cluster" == "$PERSISTENT_CLUSTER_NAME" ]] || {
  echo "bootstrap-cluster.sh must emit exactly '$PERSISTENT_CLUSTER_NAME' on stdout" >&2
  echo "got $(wc -l < "$EVIDENCE_DIR/persistent-cluster-name.txt" | tr -d ' ') line(s); first and last:" >&2
  sed -n '1p;$p' "$EVIDENCE_DIR/persistent-cluster-name.txt" >&2
  exit 1
}
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
  --arg image_provenance "$IMAGE_PROVENANCE" \
  --arg cluster_name "$PERSISTENT_CLUSTER_NAME" \
  --arg started_at "$(date -u +%Y-%m-%dT%H:%M:%SZ)" \
  '{schema:$schema, project_id:$project_id, region:$region, gke_zone:$gke_zone, persistent_cluster:$cluster_name, run_id:$run_id, git_sha:$git_sha, git_dirty:false, image_tag:$image_tag, registry:$registry, image_provenance:$image_provenance, started_at:$started_at}' \
  > "$EVIDENCE_DIR/run.json"

if [[ "$acceptance_mode" == "lumen-auth" ]]; then
  echo ">> Terraform: auth-only Lumen resources on persistent Standard GKE"
else
  echo ">> Terraform: run-scoped backup bucket and workload identity on persistent Standard GKE"
fi
mkdir -p "$TERRAFORM_ENVIRONMENT_DIR"
cp "$ACCEPTANCE_ROOT/environment"/*.tf "$TERRAFORM_ENVIRONMENT_DIR/"
TF_DATA_DIR="$STATE_DIR/.terraform-environment" terraform \
  -chdir="$TERRAFORM_ENVIRONMENT_DIR" init -input=false
# Pass acceptance_apps unconditionally rather than appending to an array only in
# tape mode.  Under macOS bash 3.2 (`set -u`), expanding an *empty* array as
# "${arr[@]}" is an unbound-variable error, so the lumen-sift path — the default
# — aborted the whole run at terraform apply while the tape path, which always
# populated the array, kept passing (run 0726052225 died exactly here).  Naming
# the value outright removes the empty-array case instead of quoting around it,
# and keeps apply symmetric with cleanup.sh's destroy args.
if [[ "$acceptance_mode" == "tape" ]]; then
  terraform_acceptance_apps=tape
elif [[ "$acceptance_mode" == "sift" ]]; then
  terraform_acceptance_apps=sift
elif [[ "$acceptance_mode" == "lumen-auth" ]]; then
  terraform_acceptance_apps=lumen-auth
else
  terraform_acceptance_apps=lumen-sift
fi
TF_DATA_DIR="$STATE_DIR/.terraform-environment" terraform \
  -chdir="$TERRAFORM_ENVIRONMENT_DIR" apply \
  -state="$STATE_DIR/environment.tfstate" \
  -auto-approve \
  -var="project_id=$PROJECT_ID" \
  -var="region=$REGION" \
  -var="gke_zone=$GKE_ZONE" \
  -var="cluster_name=$PERSISTENT_CLUSTER_NAME" \
  -var="run_id=$RUN_ID" \
  -var="artifact_registry_repository=$ARTIFACT_REGISTRY_REPOSITORY" \
  -var="image_tag=$IMAGE_TAG" \
  -var="acceptance_apps=$terraform_acceptance_apps"
TF_DATA_DIR="$STATE_DIR/.terraform-environment" terraform \
  -chdir="$TERRAFORM_ENVIRONMENT_DIR" output \
  -state="$STATE_DIR/environment.tfstate" -json > "$EVIDENCE_DIR/terraform-output.json"

cluster="$(jq -r '.cluster_name.value' "$EVIDENCE_DIR/terraform-output.json")"
test "$(jq -r '.gke_zone.value' "$EVIDENCE_DIR/terraform-output.json")" = "$GKE_ZONE"
test "$(jq -r '.cluster_name.value' "$EVIDENCE_DIR/terraform-output.json")" = "$PERSISTENT_CLUSTER_NAME"
if [[ "$acceptance_mode" != "lumen-auth" ]]; then
  test "$(jq -r '.backup_bucket.value' "$EVIDENCE_DIR/terraform-output.json")" = "$BACKUP_BUCKET"
  test "$(jq -r '.backup_gsa_email.value' "$EVIDENCE_DIR/terraform-output.json")" = "$BACKUP_GSA_EMAIL"
fi
if [[ "$acceptance_mode" == "sift" ]]; then
  SIFT_NODE_POOL="$(jq -r '.sift_node_pool.value' "$EVIDENCE_DIR/terraform-output.json")"
  [[ "$SIFT_NODE_POOL" == "axo-${RUN_ID}-sift" ]] || {
    echo "Terraform did not create the exact run-scoped Sift node pool" >&2
    exit 1
  }
  export SIFT_NODE_POOL
fi
gcloud container clusters get-credentials "$cluster" \
  --project="$PROJECT_ID" --zone="$GKE_ZONE"

# App namespaces are fixed names on the shared persistent cluster, so two
# concurrent acceptance runs of the same mode would drive the SAME operator
# cell and destroy each other's expected state (runs 0723094538/0723095701
# raced exactly this way — both tampered one StatefulSet, then one run's
# cleanup deleted the other's live namespaces). Refuse to start, and do so
# BEFORE kube-context-ready.txt exists so this run's cleanup does not touch
# the other run's namespaces either.
if [[ "$acceptance_mode" == "tape" ]]; then
  mode_namespaces=(tape tape-system)
elif [[ "$acceptance_mode" == "sift" ]]; then
  mode_namespaces=(sift sift-system sift-restore)
elif [[ "$acceptance_mode" == "lumen-auth" ]]; then
  mode_namespaces=(lumen lumen-system lumen-auth-client)
else
  mode_namespaces=(lumen lumen-system sift sift-system lumen-auth-client)
fi
for namespace in "${mode_namespaces[@]}"; do
  if kubectl get namespace "$namespace" --no-headers >/dev/null 2>&1; then
    echo "namespace $namespace already exists on $cluster; another acceptance run appears active — refusing to race it" >&2
    exit 1
  fi
done
printf '%s\n' "$cluster" > "$STATE_DIR/kube-context-ready.txt"

export EVIDENCE_DIR
export PROJECT_ID REGION BACKUP_BUCKET

if [[ "$acceptance_mode" == "sift" ]]; then
  "$SCRIPT_DIR/deploy.sh" sift
  "$SCRIPT_DIR/verify-operator-cell.sh" sift
  "$SCRIPT_DIR/verify-sift-mvp.sh"
elif [[ "$acceptance_mode" == "tape" ]]; then
  # Tape-only acceptance mode: a single disposable domain-plane cell, no
  # Lumen/Sift phasing.
  "$SCRIPT_DIR/deploy.sh" tape
  "$SCRIPT_DIR/verify-operator-cell.sh" tape
  "$SCRIPT_DIR/verify-tape.sh"
elif [[ "$acceptance_mode" == "lumen-auth" ]]; then
  "$SCRIPT_DIR/deploy.sh" lumen
  "$SCRIPT_DIR/verify-operator-cell.sh" lumen
  export LUMEN_AUTH_ACCEPTANCE_EVIDENCE="$EVIDENCE_DIR/lumen-auth-acceptance.json"
  "$SCRIPT_DIR/verify-lumen-auth.sh"
  "$SCRIPT_DIR/finalize-lumen-acceptance.sh" lumen-auth
else
  # Phase 1 is a hard gate: no Sift CRD/operator/instance/collector is applied
  # until Lumen has independently reconciled, recovered, backed up to GCS, and
  # completed its bounded disk-triggered split.
  "$SCRIPT_DIR/deploy.sh" lumen
  "$SCRIPT_DIR/verify-operator-cell.sh" lumen
  if [[ -n "$LUMEN_PRIOR_ACCEPTANCE" ]]; then
    cp "$LUMEN_PRIOR_ACCEPTANCE" "$EVIDENCE_DIR/lumen-acceptance-prior.json"
    export LUMEN_ACCEPTANCE_EVIDENCE="$EVIDENCE_DIR/lumen-acceptance-prior.json"
    export LUMEN_ACCEPTANCE_PROVENANCE="prior-gke-proof"
    echo ">> current Lumen operator cell passed; reusing supplied prior persistence, backup, and split proof"
  else
    "$SCRIPT_DIR/verify-lumen.sh"
    export LUMEN_ACCEPTANCE_EVIDENCE="$EVIDENCE_DIR/lumen-acceptance.json"
    export LUMEN_ACCEPTANCE_PROVENANCE="current-run"
  fi

  # Request authorization is its own proof and runs on every pass, including
  # one reusing a prior persistence/backup/split proof: those legs all opt out
  # of auth, so nothing they established says anything about who may call.
  export LUMEN_AUTH_ACCEPTANCE_EVIDENCE="$EVIDENCE_DIR/lumen-auth-acceptance.json"
  "$SCRIPT_DIR/verify-lumen-auth.sh"

  # Only a successful Lumen phase starts the Sift data plane. The collector then
  # reads Lumen's structured stdout from Standard GKE node logs and the proof
  # queries the materialized Sift logging store.
  "$SCRIPT_DIR/deploy.sh" sift
  "$SCRIPT_DIR/verify-operator-cell.sh" sift
  "$SCRIPT_DIR/verify-sift-collection.sh"
fi

echo ">> acceptance passed; mandatory cleanup runs on EXIT"
run_completed=1
