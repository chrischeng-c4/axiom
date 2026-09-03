#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$SCRIPT_DIR/sift-candidate.sh"
source "$SCRIPT_DIR/sift-container-boundary.sh"
source "$SCRIPT_DIR/acceptance-lock.sh"

recovery_mode=0
if [[ "$#" == "1" && "$1" == "--recover" ]]; then
  recovery_mode=1
elif [[ "$#" != "0" ]]; then
  echo "usage: run-sift-contained.sh [--recover]" >&2
  exit 2
fi

: "${PROJECT_ID:?PROJECT_ID is required}"
: "${SIFT_CANDIDATE_DIR:?SIFT_CANDIDATE_DIR is required}"
REGION="${REGION:-asia-east1}"
GKE_ZONE="${GKE_ZONE:-asia-east1-a}"
PERSISTENT_CLUSTER_NAME="${PERSISTENT_CLUSTER_NAME:-axiom-operator-acceptance}"
MAX_CLOUD_SECONDS="${MAX_CLOUD_SECONDS:-5400}"
MAX_PREFLIGHT_SECONDS="${MAX_PREFLIGHT_SECONDS:-1800}"
ACCEPTANCE_LOCAL_CLAIM_ROOT="${ACCEPTANCE_LOCAL_CLAIM_ROOT:-${TMPDIR:-/tmp}/axiom-gcp-operator-claims}"

paths_overlap() {
  local left="$1"
  local right="$2"
  [[ "$left" == "$right" \
    || "$left" == "$right/"* \
    || "$right" == "$left/"* ]]
}

canonicalize_without_creating() {
  python3 -c 'import os, sys; print(os.path.realpath(sys.argv[1]))' "$1"
}

require() {
  command -v "$1" >/dev/null 2>&1 || {
    echo "required command not found: $1" >&2
    exit 1
  }
}
for command in cp docker find gcloud grep id jq mktemp openssl python3 rm tee tr wc; do
  require "$command"
done
verify_sift_candidate_directory "$SIFT_CANDIDATE_DIR" || {
  echo "SIFT_CANDIDATE_DIR does not contain one valid immutable candidate" >&2
  exit 1
}
SIFT_CANDIDATE_DIR="$(cd "$SIFT_CANDIDATE_DIR" && pwd -P)" || {
  echo "SIFT_CANDIDATE_DIR could not be canonicalized" >&2
  exit 1
}

candidate="$SIFT_CANDIDATE_DIR/candidate.json"
candidate_project="$(jq -er '.project_id' "$candidate")"
candidate_region="$(jq -er '.region' "$candidate")"
candidate_run_id="$(jq -er '.run_id' "$candidate")"
controller_image="$(jq -er '.acceptance_runner_image' "$candidate")"
artifact_repository="$(jq -er '.artifact_registry_repository' "$candidate")"
registry="$(jq -er '.registry' "$candidate")"
image_tag="$(jq -er '.image_tag' "$candidate")"
source_prefix="$(jq -er '.source_prefix' "$candidate")"
[[ "$candidate_project" == "$PROJECT_ID" && "$candidate_region" == "$REGION" ]] || {
  echo "the candidate project or region does not match this run" >&2
  exit 1
}
if [[ -n "${RUN_ID:-}" && "$RUN_ID" != "$candidate_run_id" ]]; then
  echo "RUN_ID must match the candidate receipt" >&2
  exit 1
fi
RUN_ID="$candidate_run_id"
STATE_DIR="${STATE_DIR:-${TMPDIR:-/tmp}/axiom-gcp-operator-${RUN_ID}}"
EVIDENCE_DIR="${EVIDENCE_DIR:-${TMPDIR:-/tmp}/axiom-gcp-operator-evidence/${RUN_ID}}"
CONTAINMENT_DIR="${CONTAINMENT_DIR:-${TMPDIR:-/tmp}/axiom-gcp-operator-containment/${RUN_ID}}"
for directory in "$STATE_DIR" "$EVIDENCE_DIR" "$CONTAINMENT_DIR"; do
  [[ "$directory" == /* && ! -L "$directory" ]] || {
    echo "run directories must be absolute and cannot be symlinks" >&2
    exit 1
  }
  if [[ "$recovery_mode" == "1" ]]; then
    [[ -d "$directory" ]] || {
      echo "recovery directory is missing: $directory" >&2
      exit 1
    }
  elif [[ -e "$directory" \
    && -n "$(find "$directory" -mindepth 1 -print -quit 2>/dev/null)" ]]; then
      echo "refusing to reuse nonempty run directory: $directory" >&2
      exit 1
  fi
done
[[ "$ACCEPTANCE_LOCAL_CLAIM_ROOT" == /* \
  && ! -L "$ACCEPTANCE_LOCAL_CLAIM_ROOT" ]] || {
  echo "the local claim root must be absolute and cannot be a symlink" >&2
  exit 1
}

# Reject candidate aliases before creating or changing any writable path.
# The second check below uses the paths after mkdir and protects against a
# parent symlink changing between this check and directory creation.
planned_state_dir="$(canonicalize_without_creating "$STATE_DIR")"
planned_evidence_dir="$(canonicalize_without_creating "$EVIDENCE_DIR")"
planned_containment_dir="$(canonicalize_without_creating "$CONTAINMENT_DIR")"
planned_claim_root="$(
  canonicalize_without_creating "$ACCEPTANCE_LOCAL_CLAIM_ROOT"
)"
for planned_writable_directory in \
    "$planned_state_dir" "$planned_evidence_dir" \
    "$planned_containment_dir" "$planned_claim_root"; do
  if paths_overlap "$SIFT_CANDIDATE_DIR" "$planned_writable_directory"; then
    echo "the candidate directory must not overlap a writable run directory" >&2
    exit 1
  fi
done

mkdir -p "$STATE_DIR" "$EVIDENCE_DIR" "$CONTAINMENT_DIR" \
  "$ACCEPTANCE_LOCAL_CLAIM_ROOT"
chmod 0700 "$STATE_DIR" "$EVIDENCE_DIR" "$CONTAINMENT_DIR" \
  "$ACCEPTANCE_LOCAL_CLAIM_ROOT"
STATE_DIR="$(cd "$STATE_DIR" && pwd -P)"
EVIDENCE_DIR="$(cd "$EVIDENCE_DIR" && pwd -P)"
CONTAINMENT_DIR="$(cd "$CONTAINMENT_DIR" && pwd -P)"
ACCEPTANCE_LOCAL_CLAIM_ROOT="$(
  cd "$ACCEPTANCE_LOCAL_CLAIM_ROOT" && pwd -P
)"

if paths_overlap "$STATE_DIR" "$EVIDENCE_DIR" \
    || paths_overlap "$STATE_DIR" "$CONTAINMENT_DIR" \
    || paths_overlap "$EVIDENCE_DIR" "$CONTAINMENT_DIR"; then
  echo "state, evidence, and containment directories must not overlap" >&2
  exit 1
fi
if paths_overlap "$ACCEPTANCE_LOCAL_CLAIM_ROOT" "$CONTAINMENT_DIR" \
    || paths_overlap "$ACCEPTANCE_LOCAL_CLAIM_ROOT" "$STATE_DIR" \
    || paths_overlap "$ACCEPTANCE_LOCAL_CLAIM_ROOT" "$EVIDENCE_DIR"; then
  echo "the local claim root must not overlap run directories" >&2
  exit 1
fi
for writable_directory in \
    "$STATE_DIR" "$EVIDENCE_DIR" "$CONTAINMENT_DIR" \
    "$ACCEPTANCE_LOCAL_CLAIM_ROOT"; do
  if paths_overlap "$SIFT_CANDIDATE_DIR" "$writable_directory"; then
    echo "the candidate directory must not overlap a writable run directory" >&2
    exit 1
  fi
done

host_uid="$(id -u)"
host_gid="$(id -g)"
[[ "$host_uid" =~ ^[1-9][0-9]*$ && "$host_gid" =~ ^[0-9]+$ ]] || {
  echo "run-sift-contained.sh refuses to start the controller as root" >&2
  exit 1
}
gcloud_config="$CONTAINMENT_DIR/gcloud"
nonce_dir="$CONTAINMENT_DIR/nonce"
bootstrap_private_exit() {
  local ec=$?
  trap - EXIT INT TERM
  if [[ "$recovery_mode" == "0" ]]; then
    for private_dir in "$gcloud_config" "$nonce_dir"; do
      if [[ -d "$private_dir" && ! -L "$private_dir" ]]; then
        find "$private_dir" -depth -delete >/dev/null 2>&1 || ec=1
      fi
    done
  fi
  exit "$ec"
}
trap bootstrap_private_exit EXIT
trap 'exit 130' INT TERM
if [[ "$recovery_mode" == "1" ]]; then
  [[ -d "$gcloud_config" && ! -L "$gcloud_config" \
    && -d "$nonce_dir" && ! -L "$nonce_dir" ]] || {
    echo "contained recovery credentials or nonce directory is missing" >&2
    exit 1
  }
  if find "$gcloud_config" -type l -print -quit | grep -q .; then
    echo "contained recovery credentials contain a symlink" >&2
    exit 1
  fi
  if find "$gcloud_config" ! -type f ! -type d -print -quit | grep -q .; then
    echo "contained recovery credentials contain a special file" >&2
    exit 1
  fi
else
  gcloud_config_source="$(gcloud info --format='value(config.paths.global_config_dir)')"
  [[ "$gcloud_config_source" == /* && -d "$gcloud_config_source" \
    && ! -L "$gcloud_config_source" ]] || {
    echo "could not locate a safe gcloud configuration directory" >&2
    exit 1
  }
  if find "$gcloud_config_source" -type l -print -quit | grep -q .; then
    echo "gcloud configuration contains a symlink; refusing to copy credentials" >&2
    exit 1
  fi
  if find "$gcloud_config_source" ! -type f ! -type d -print -quit | grep -q .; then
    echo "gcloud configuration contains a special file; refusing to copy credentials" >&2
    exit 1
  fi
  mkdir -p "$gcloud_config" "$nonce_dir"
  chmod 0700 "$gcloud_config" "$nonce_dir"
  cp -R "$gcloud_config_source/." "$gcloud_config/"
  chmod -R go-rwx "$gcloud_config"
fi
CLOUDSDK_CONFIG="$gcloud_config" gcloud auth print-access-token >/dev/null || {
  echo "the active gcloud account cannot mint an access token" >&2
  exit 1
}

nonce_file="$nonce_dir/cleanup-nonce"
if [[ "$recovery_mode" == "1" ]]; then
  [[ -f "$nonce_file" && ! -L "$nonce_file" \
    && "$(wc -l < "$nonce_file" | tr -d ' ')" == "1" ]] || {
    echo "contained recovery nonce is missing or unsafe" >&2
    exit 1
  }
  cleanup_nonce="$(<"$nonce_file")"
else
  cleanup_nonce="$(openssl rand -hex 32)"
  printf '%s\n' "$cleanup_nonce" > "$nonce_file"
  chmod 0600 "$nonce_file"
fi
cleanup_digest="$(sift_container_nonce_digest "$cleanup_nonce")"
run_container_id=""
cleanup_container_id=""
wrapper_complete=0
cleanup_container_owner="$CONTAINMENT_DIR/cleanup-container-owner.json"
host_run_log="$CONTAINMENT_DIR/run.log"

stop_exact_container() {
  local container_id="$1"
  local running
  [[ "$container_id" =~ ^[0-9a-f]{64}$ ]] || return 1
  running="$(docker inspect --format '{{.State.Running}}' "$container_id" 2>/dev/null)" \
    || return 1
  if [[ "$running" == "true" ]]; then
    docker stop --time 60 "$container_id" >/dev/null 2>&1 \
      || docker kill "$container_id" >/dev/null 2>&1 \
      || return 1
  elif [[ "$running" != "false" ]]; then
    return 1
  fi
  [[ "$(docker inspect --format '{{.State.Running}}' "$container_id")" == "false" ]]
}

remove_private_copy() {
  local directory="$1"
  [[ "$directory" == "$CONTAINMENT_DIR/gcloud" \
    || "$directory" == "$CONTAINMENT_DIR/nonce" ]] || return 1
  [[ -d "$directory" && ! -L "$directory" ]] || return 0
  find "$directory" -depth -delete
}

publish_host_run_log() {
  local temporary="$EVIDENCE_DIR/.run.log.$$"
  [[ -f "$host_run_log" && ! -L "$host_run_log" ]] || return 1
  rm -f "$temporary"
  if ! cp "$host_run_log" "$temporary" \
      || ! chmod 0600 "$temporary" \
      || ! mv "$temporary" "$EVIDENCE_DIR/run.log"; then
    rm -f "$temporary"
    return 1
  fi
}

wrapper_exit() {
  local ec=$?
  trap - EXIT INT TERM
  if [[ -n "$run_container_id" ]]; then
    stop_exact_container "$run_container_id" >/dev/null 2>&1 || ec=1
  fi
  if [[ -n "$cleanup_container_id" ]]; then
    stop_exact_container "$cleanup_container_id" >/dev/null 2>&1 || ec=1
  fi
  if [[ "$wrapper_complete" == "1" \
    || ( "$recovery_mode" == "0" \
      && -z "$run_container_id" && -z "$cleanup_container_id" ) ]]; then
    remove_private_copy "$gcloud_config" || ec=1
    remove_private_copy "$nonce_dir" || ec=1
  else
    echo "containment recovery data remains at $CONTAINMENT_DIR" >&2
  fi
  exit "$ec"
}
trap wrapper_exit EXIT
trap 'exit 130' INT TERM

local_claim_path="$(acceptance_run_claim_path \
  "$ACCEPTANCE_LOCAL_CLAIM_ROOT" "$PROJECT_ID" "$RUN_ID" sift)" || exit 1

classify_run_handoff() {
  cleanup_required=0
  preflight_claim_released=0
  if jq -e '
      .schema == "axiom.gcp.sift.contained-run-exit.v1"
      and keys == ["cleanup_armed","completed","exit_code","finished_at","local_claim_released","schema"]
      and (.exit_code | type) == "number"
      and .exit_code >= 0 and .exit_code <= 255
      and (.completed | type) == "boolean"
      and (.cleanup_armed | type) == "boolean"
      and (.local_claim_released | type) == "boolean"
    ' "$EVIDENCE_DIR/run-exit.json" >/dev/null 2>&1; then
    run_status="$(jq -er '.exit_code' "$EVIDENCE_DIR/run-exit.json")"
    if jq -e '.cleanup_armed == true' "$EVIDENCE_DIR/run-exit.json" \
        >/dev/null; then
      cleanup_required=1
    elif jq -e '.local_claim_released == true' "$EVIDENCE_DIR/run-exit.json" \
        >/dev/null; then
      preflight_claim_released=1
    else
      echo "contained preflight did not release its local claim" >&2
      return 1
    fi
    return 0
  fi
  if [[ -f "$EVIDENCE_DIR/acceptance-lock-intent.json" \
    || -f "$EVIDENCE_DIR/acceptance-lock.json" ]]; then
    cleanup_required=1
    return 0
  fi
  if [[ ! -e "$local_claim_path" && ! -L "$local_claim_path" ]]; then
    preflight_claim_released=1
    return 0
  fi
  echo "contained run has no handoff receipt and still owns a local claim" >&2
  return 1
}

common_args=(
  --init
  --read-only
  --cap-drop=ALL
  --security-opt=no-new-privileges
  --user "${host_uid}:${host_gid}"
  --tmpfs "/tmp:rw,nosuid,nodev,size=2147483648,uid=${host_uid},gid=${host_gid},mode=0700"
  --mount "type=bind,src=${STATE_DIR},dst=/state"
  --mount "type=bind,src=${EVIDENCE_DIR},dst=/evidence"
  --mount "type=bind,src=${ACCEPTANCE_LOCAL_CLAIM_ROOT},dst=/claims"
  --mount "type=bind,src=${gcloud_config},dst=/gcloud"
  --mount "type=bind,src=${SIFT_CANDIDATE_DIR},dst=/candidate,readonly"
  --env HOME=/tmp/sift-acceptance-home
  --env CLOUDSDK_CONFIG=/gcloud
  --env KUBECONFIG=/state/kubeconfig
  --env ACCEPTANCE_APPS=sift
  --env PROJECT_ID="$PROJECT_ID"
  --env REGION="$REGION"
  --env GKE_ZONE="$GKE_ZONE"
  --env PERSISTENT_CLUSTER_NAME="$PERSISTENT_CLUSTER_NAME"
  --env ARTIFACT_REGISTRY_REPOSITORY="$artifact_repository"
  --env RUN_ID="$RUN_ID"
  --env STATE_DIR=/state
  --env EVIDENCE_DIR=/evidence
  --env ACCEPTANCE_LOCAL_CLAIM_ROOT=/claims
  --env SIFT_CANDIDATE_DIR=/candidate
  --env MAX_CLOUD_SECONDS="$MAX_CLOUD_SECONDS"
  --env MAX_PREFLIGHT_SECONDS="$MAX_PREFLIGHT_SECONDS"
)

run_status=125
cleanup_required=0
preflight_claim_released=0
if [[ "$recovery_mode" == "1" ]]; then
  verify_sift_container_owner \
    "$CONTAINMENT_DIR/container-owner.json" "$controller_image" \
    /state /evidence || {
    echo "contained recovery has no valid run-container owner receipt" >&2
    exit 1
  }
  [[ "$(jq -er '.cleanup_handoff_digest' \
      "$CONTAINMENT_DIR/container-owner.json")" == "$cleanup_digest" ]] || {
    echo "contained recovery nonce does not match its owner receipt" >&2
    exit 1
  }
  run_container_id="$(jq -er '.container_id' \
    "$CONTAINMENT_DIR/container-owner.json")"
  classify_run_handoff || exit 1
  if [[ -e "$cleanup_container_owner" ]]; then
    verify_sift_cleanup_container_owner \
      "$cleanup_container_owner" "$controller_image" \
      "$run_container_id" "$cleanup_digest" || {
      echo "contained recovery has an invalid cleanup-container receipt" >&2
      exit 1
    }
    previous_cleanup_id="$(jq -er '.container_id' "$cleanup_container_owner")"
    stop_exact_container "$previous_cleanup_id" || {
      echo "could not stop the previously recorded cleanup container" >&2
      exit 1
    }
    docker rm "$previous_cleanup_id" >/dev/null || exit 1
    rm -f "$cleanup_container_owner"
  fi
else
  run_container_id="$(docker create \
    "${common_args[@]}" \
    --env AXIOM_GCP_ACCEPTANCE_CONTAINER_ROLE=run \
    --env ACCEPTANCE_CONTAINER_HANDOFF_DIGEST="$cleanup_digest" \
    "$controller_image" \
    /workspace/acceptance/gcp/scripts/run.sh)"
  [[ "$run_container_id" =~ ^[0-9a-f]{64}$ ]] || {
    echo "Docker did not return one full run container ID" >&2
    exit 1
  }
  write_sift_container_owner \
    "$CONTAINMENT_DIR/container-owner.json" "$run_container_id" \
    "$controller_image" /state /evidence "$cleanup_digest" || exit 1

  : > "$host_run_log"
  chmod 0600 "$host_run_log"
  run_status=0
  set +e
  docker start --attach "$run_container_id" 2>&1 \
    | tee -a "$host_run_log"
  pipeline_status=("${PIPESTATUS[@]}")
  run_status="${pipeline_status[0]}"
  run_log_status="${pipeline_status[1]}"
  set -e
  [[ "$run_log_status" == "0" ]] || run_status=1
  classify_run_handoff || exit 1
fi
stop_exact_container "$run_container_id" || {
  echo "the exact Sift run container did not stop" >&2
  exit 1
}
docker inspect "$run_container_id" | jq -e 'length == 1' >/dev/null || exit 1
docker inspect "$run_container_id" | jq '.[0]' \
  > "$CONTAINMENT_DIR/run-container-inspect.json"
write_sift_container_stopped_receipt \
  "$CONTAINMENT_DIR/container-stopped.json" \
  "$CONTAINMENT_DIR/run-container-inspect.json" \
  "$run_container_id" "$controller_image" "$cleanup_digest" || {
  echo "could not prove that the exact run container stopped" >&2
  exit 1
}
publish_host_run_log || {
  echo "could not publish the contained Sift run log" >&2
  exit 1
}

if [[ "$cleanup_required" == "0" ]]; then
  [[ "$preflight_claim_released" == "1" ]] || {
    echo "contained Sift preflight did not release its local run claim" >&2
    exit 1
  }
  docker rm "$run_container_id" >/dev/null
  run_container_id=""
  wrapper_complete=1
  if [[ "$run_status" != "0" ]]; then
    echo "Sift acceptance preflight failed with status $run_status; no cloud cleanup was armed" >&2
    exit "$run_status"
  fi
  echo "Sift acceptance completed without arming cloud cleanup"
  exit 0
fi

cleanup_container_id="$(docker create \
  "${common_args[@]}" \
  --mount "type=bind,src=${CONTAINMENT_DIR},dst=/containment,readonly" \
  --mount "type=bind,src=${nonce_file},dst=/run-secrets/cleanup-nonce,readonly" \
  --env AXIOM_GCP_ACCEPTANCE_CONTAINER_ROLE=cleanup \
  --env ACCEPTANCE_CONTAINER_OWNER_RECEIPT=/containment/container-owner.json \
  --env ACCEPTANCE_CONTAINER_STOP_RECEIPT=/containment/container-stopped.json \
  --env ACCEPTANCE_CONTAINER_CLEANUP_NONCE_FILE=/run-secrets/cleanup-nonce \
  --env ACCEPTANCE_CONTROLLER_IMAGE="$controller_image" \
  --env REGISTRY="$registry" \
  --env IMAGE_TAG="$image_tag" \
  --env GCS_SOURCE_PREFIX="$source_prefix" \
  "$controller_image" \
  /workspace/acceptance/gcp/scripts/cleanup.sh)"
[[ "$cleanup_container_id" =~ ^[0-9a-f]{64}$ ]] || {
  echo "Docker did not return one full cleanup container ID" >&2
  exit 1
}
write_sift_cleanup_container_owner \
  "$cleanup_container_owner" "$cleanup_container_id" "$controller_image" \
  "$run_container_id" "$cleanup_digest" || exit 1
cleanup_status=0
docker start --attach "$cleanup_container_id" || cleanup_status=$?
stop_exact_container "$cleanup_container_id" || cleanup_status=1
[[ "$cleanup_status" == "0" ]] || {
  echo "contained Sift cleanup failed; recovery data remains at $CONTAINMENT_DIR" >&2
  exit 1
}

docker rm "$cleanup_container_id" >/dev/null
cleanup_container_id=""
rm -f "$cleanup_container_owner"
docker rm "$run_container_id" >/dev/null
run_container_id=""
wrapper_complete=1
if [[ "$run_status" != "0" ]]; then
  echo "Sift acceptance failed with status $run_status; cleanup passed" >&2
  exit "$run_status"
fi
echo "Sift acceptance and contained cleanup passed"
