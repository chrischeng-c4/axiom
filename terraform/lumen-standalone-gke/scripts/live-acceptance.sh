#!/usr/bin/env bash
# shellcheck disable=SC1091,SC2034,SC2312
set -euo pipefail
set +x
umask 077
CDPATH=

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd -P)"
MODULE_DIR="$(cd "$SCRIPT_DIR/.." && pwd -P)"
REPO_ROOT="$(cd "$MODULE_DIR/../.." && pwd -P)"
CLIENT_IMAGE='docker.io/curlimages/curl@sha256:7c12af72ceb38b7432ab85e1a265cff6ae58e06f95539d539b654f2cfa64bb13'
EXPECTED_ADDRESSES=$'google_container_cluster.standalone\ngoogle_container_node_pool.standalone\ngoogle_project_iam_member.node_baseline\ngoogle_service_account.nodes'

# Do not use TMPDIR.  macOS exposes /tmp through /private/tmp, while Linux
# uses /tmp directly.  Resolve the physical directory once and keep every
# private path below it.
PRIVATE_TMP_ROOT="$(cd -P /tmp && pwd -P)"

RUN_ROOT=''
PUBLIC_STAGING=''
PUBLIC_RECEIPT_DIGEST=''
LIFECYCLE_PHASE='precreate'
PROJECT_ID=''
REGION=''
GKE_ZONE=''
RUN_ID=''
CANDIDATE_RECEIPT_DIR=''
LUMEN_CLI=''
CLI_TARGET=''
IMAGE=''
EXPECTED_COMMIT=''
EXPECTED_RUN_ID=''
EXPECTED_RUN_ATTEMPT=''
EXPECTED_MANIFEST_SHA256=''
RECEIPT_OUT_DIR=''
CONFIRM_CREATE=''
CONFIRM_DESTROY=''
CLUSTER=''
NODE_POOL=''
NODE_SERVICE_ACCOUNT=''
OWNER_LABEL=''
CONTEXT=''
SEEN_OPTIONS='|'

die() {
  printf 'standalone GKE lifecycle: %s\n' "$*" >&2
  exit 1
}

case "$PRIVATE_TMP_ROOT" in /tmp|/private/tmp) ;; *) die 'unsupported private temp root' ;; esac

usage() {
  printf '%s\n' 'usage: live-acceptance.sh --project-id ID --region REGION --gke-zone ZONE --run-id DNS_ID --candidate-receipt-dir DIR --lumen-cli FILE --cli-target TARGET --image GHCR_DIGEST --expected-commit SHA --expected-run-id ID --expected-run-attempt ATTEMPT --expected-manifest-sha256 SHA --receipt-out-dir DIR --confirm-create CLUSTER --confirm-destroy CLUSTER' >&2
  exit 2
}

sha256_file() {
  if command -v sha256sum >/dev/null 2>&1; then sha256sum "$1" | awk '{print $1}'; else shasum -a 256 "$1" | awk '{print $1}'; fi
}

sha256_text() {
  if command -v sha256sum >/dev/null 2>&1; then printf '%s' "$1" | sha256sum | awk '{print $1}'; else printf '%s' "$1" | shasum -a 256 | awk '{print $1}'; fi
}

run_hash() { sha256_text "$1" | cut -c1-10; }
cluster_name() { printf 'lumen-sa-%s\n' "$(run_hash "$1")"; }
pool_name() { printf 'lumen-np-%s\n' "$(run_hash "$1")"; }
node_service_account() { printf 'lumen-nodes-%s\n' "$(run_hash "$1")"; }
owner_label() { printf 'lumen-standalone-%s\n' "$(run_hash "$1")"; }

safe_private_dir() {
  local path=$1
  [[ "$path" == "$PRIVATE_TMP_ROOT"/* && "$path" != *'/../'* && "$path" != */.. && "$path" != *'/./'* && "$path" != */ && -d "$path" && ! -L "$path" ]] || return 1
  [[ "$(cd "$path" && pwd -P)" == "$path" ]]
}

safe_private_file() {
  local path=$1 parent
  [[ "$path" == "$PRIVATE_TMP_ROOT"/* && "$path" != *'/../'* && "$path" != */.. && "$path" != *'/./'* && -f "$path" && ! -L "$path" ]] || return 1
  parent=${path%/*}
  safe_private_dir "$parent"
}

safe_new_private_path() {
  local path=$1 parent base
  [[ "$path" == "$PRIVATE_TMP_ROOT"/* && "$path" != */ && "$path" != *'/../'* && "$path" != */.. && "$path" != *'/./'* && ! -e "$path" && ! -L "$path" ]] || return 1
  parent=${path%/*}
  base=${path##*/}
  [[ -n "$base" && "$base" != . && "$base" != .. ]] || return 1
  safe_private_dir "$parent"
}

private_mode() {
  local path=$1 mode status
  mode=$(stat -c %a "$path" 2>/dev/null); status=$?
  if [[ "$status" -eq 0 ]]; then
    [[ "$mode" == 700 ]] || return 1
    printf '%s\n' "$mode"
    return 0
  fi
  [[ -z "$mode" ]] || return 1
  mode=$(stat -f %Lp "$path" 2>/dev/null) || return 1
  [[ "$mode" == 700 ]] || return 1
  printf '%s\n' "$mode"
}

require_tools() {
  local tool
  for tool in bash env jq terraform gcloud kubectl awk cut mktemp chmod mkdir cp mv cmp find sort wc rm rmdir stat; do
    command -v "$tool" >/dev/null 2>&1 || die "required tool is missing: $tool"
  done
  if ! command -v sha256sum >/dev/null 2>&1 && ! command -v shasum >/dev/null 2>&1; then die 'sha256 tool is missing'; fi
}

parse_args() {
  while [[ $# -gt 0 ]]; do
    [[ $# -ge 2 && -n "$2" && "$2" != --* ]] || usage
    case "$SEEN_OPTIONS" in *"|$1|"*) die 'duplicate option' ;; esac
    SEEN_OPTIONS="${SEEN_OPTIONS}${1}|"
    case "$1" in
      --project-id) PROJECT_ID=$2 ;;
      --region) REGION=$2 ;;
      --gke-zone) GKE_ZONE=$2 ;;
      --run-id) RUN_ID=$2 ;;
      --candidate-receipt-dir) CANDIDATE_RECEIPT_DIR=$2 ;;
      --lumen-cli) LUMEN_CLI=$2 ;;
      --cli-target) CLI_TARGET=$2 ;;
      --image) IMAGE=$2 ;;
      --expected-commit) EXPECTED_COMMIT=$2 ;;
      --expected-run-id) EXPECTED_RUN_ID=$2 ;;
      --expected-run-attempt) EXPECTED_RUN_ATTEMPT=$2 ;;
      --expected-manifest-sha256) EXPECTED_MANIFEST_SHA256=$2 ;;
      --receipt-out-dir) RECEIPT_OUT_DIR=$2 ;;
      --confirm-create) CONFIRM_CREATE=$2 ;;
      --confirm-destroy) CONFIRM_DESTROY=$2 ;;
      *) die 'unknown option' ;;
    esac
    shift 2
  done
  [[ -n "$PROJECT_ID" && -n "$REGION" && -n "$GKE_ZONE" && -n "$RUN_ID" && -n "$CANDIDATE_RECEIPT_DIR" && -n "$LUMEN_CLI" && -n "$CLI_TARGET" && -n "$IMAGE" && -n "$EXPECTED_COMMIT" && -n "$EXPECTED_RUN_ID" && -n "$EXPECTED_RUN_ATTEMPT" && -n "$EXPECTED_MANIFEST_SHA256" && -n "$RECEIPT_OUT_DIR" && -n "$CONFIRM_CREATE" && -n "$CONFIRM_DESTROY" ]] || usage
}

validate_inputs() {
  [[ "$PROJECT_ID" =~ ^[a-z][a-z0-9-]{4,28}[a-z0-9]$ ]] || die 'invalid project id'
  case "$PROJECT_ID" in replace-with-real-project-id|example-project|placeholder-project|your-project-id|required-project-id) die 'project id is a placeholder' ;; esac
  [[ "$REGION" =~ ^[a-z]+-[a-z0-9]+[0-9]$ ]] || die 'invalid region'
  [[ "$GKE_ZONE" =~ ^[a-z]+-[a-z0-9]+[0-9]-[a-z]$ && "$GKE_ZONE" == "$REGION"-* ]] || die 'invalid GKE zone'
  [[ "$RUN_ID" =~ ^[a-z0-9][a-z0-9-]{0,39}[a-z0-9]$ ]] || die 'invalid run id'
  case "$RUN_ID" in run|test|placeholder) die 'run id is a placeholder' ;; esac
  [[ "$IMAGE" =~ ^ghcr\.io/chrischeng-c4/lumen@sha256:[0-9a-f]{64}$ ]] || die 'image is not an immutable Lumen digest'
  [[ "$EXPECTED_COMMIT" =~ ^[0-9a-f]{40}$ ]] || die 'invalid expected commit'
  [[ "$EXPECTED_RUN_ID" =~ ^[0-9]+$ ]] || die 'invalid candidate run id'
  [[ "$EXPECTED_RUN_ATTEMPT" =~ ^[0-9]+$ ]] || die 'invalid candidate run attempt'
  [[ "$EXPECTED_MANIFEST_SHA256" =~ ^[0-9a-f]{64}$ ]] || die 'invalid candidate manifest hash'
  case "$CLI_TARGET" in aarch64-apple-darwin|x86_64-unknown-linux-gnu|aarch64-unknown-linux-gnu|x86_64-unknown-linux-musl|aarch64-unknown-linux-musl) ;; *) die 'invalid CLI target' ;; esac
  safe_private_dir "$CANDIDATE_RECEIPT_DIR" || die 'candidate receipt directory is unsafe'
  if ! safe_private_file "$LUMEN_CLI" || [[ ! -x "$LUMEN_CLI" ]]; then
    die 'Lumen CLI path is unsafe'
  fi
  safe_new_private_path "$RECEIPT_OUT_DIR" || die 'receipt output path is unsafe'
  CLUSTER=$(cluster_name "$RUN_ID")
  NODE_POOL=$(pool_name "$RUN_ID")
  NODE_SERVICE_ACCOUNT=$(node_service_account "$RUN_ID")
  OWNER_LABEL=$(owner_label "$RUN_ID")
  [[ "$CONFIRM_CREATE" == "$CLUSTER" && "$CONFIRM_DESTROY" == "$CLUSTER" ]] || die 'cluster confirmation does not match derived identity'
}

validate_repo_inputs() {
  local path
  for path in "$MODULE_DIR/scripts/check.sh" "$REPO_ROOT/kustomize/lumen-standalone-acceptance/tests/contract.sh" "$REPO_ROOT/apps/lumen/scripts/standalone-gke-acceptance.sh" "$REPO_ROOT/apps/lumen/scripts/verify-release-artifacts.sh"; do
    [[ -f "$path" && ! -L "$path" ]] || die 'required repository gate is missing or unsafe'
  done
}

write_run_contract() {
  jq -n --arg schema 'lumen.standalone-gke-live/v2' --arg private_temp_root "$PRIVATE_TMP_ROOT" --arg state_dir "$RUN_ROOT" --arg project_id "$PROJECT_ID" --arg region "$REGION" --arg gke_zone "$GKE_ZONE" --arg run_id "$RUN_ID" --arg cluster_name "$CLUSTER" --arg node_pool_name "$NODE_POOL" --arg node_service_account_id "$NODE_SERVICE_ACCOUNT" --arg storage_class_name 'premium-rwo' --arg image "$IMAGE" --arg expected_commit "$EXPECTED_COMMIT" --arg expected_run_id "$EXPECTED_RUN_ID" --arg expected_run_attempt "$EXPECTED_RUN_ATTEMPT" --arg expected_manifest_sha256 "$EXPECTED_MANIFEST_SHA256" --arg candidate_receipt_dir "$CANDIDATE_RECEIPT_DIR" --arg lumen_cli "$LUMEN_CLI" --arg cli_target "$CLI_TARGET" --arg receipt_out_dir "$RECEIPT_OUT_DIR" --arg confirm_create "$CONFIRM_CREATE" --arg confirm_destroy "$CONFIRM_DESTROY" \
    '{schema:$schema,private_temp_root:$private_temp_root,state_dir:$state_dir,project_id:$project_id,region:$region,gke_zone:$gke_zone,run_id:$run_id,cluster_name:$cluster_name,node_pool_name:$node_pool_name,node_service_account_id:$node_service_account_id,storage_class_name:$storage_class_name,image:$image,expected_commit:$expected_commit,expected_run_id:$expected_run_id,expected_run_attempt:$expected_run_attempt,expected_manifest_sha256:$expected_manifest_sha256,candidate_receipt_dir:$candidate_receipt_dir,lumen_cli:$lumen_cli,cli_target:$cli_target,receipt_out_dir:$receipt_out_dir,confirm_create:$confirm_create,confirm_destroy:$confirm_destroy}' >"$RUN_ROOT/run-contract.json"
}

prepare_run_root() {
  RUN_ROOT=$(mktemp -d "$PRIVATE_TMP_ROOT/lumen-standalone-gke-live.XXXXXX")
  [[ -d "$RUN_ROOT" && ! -L "$RUN_ROOT" && "$(cd "$RUN_ROOT" && pwd -P)" == "$RUN_ROOT" ]] || die 'run root is not canonical'
  [[ "$(cd "${RUN_ROOT%/*}" && pwd -P)" == "$PRIVATE_TMP_ROOT" ]] || die 'run root parent is not canonical'
  chmod 700 "$RUN_ROOT"
  [[ "$(private_mode "$RUN_ROOT")" == 700 ]] || die 'run root mode is not 0700'
  mkdir -m 700 "$RUN_ROOT/terraform-data" "$RUN_ROOT/private-receipt" "$RUN_ROOT/control"
  : >"$RUN_ROOT/terraform.tfstate"
  : >"$RUN_ROOT/kubeconfig"
  chmod 600 "$RUN_ROOT/terraform.tfstate" "$RUN_ROOT/kubeconfig"
  write_run_contract
  export TF_DATA_DIR="$RUN_ROOT/terraform-data"
  export KUBECONFIG="$RUN_ROOT/kubeconfig"
}

repair_command() { printf 'terraform/lumen-standalone-gke/scripts/repair-destroy.sh --state-dir %s --confirm-destroy %s\n' "$RUN_ROOT" "$CLUSTER" >&2; }

cleanup_public_staging() {
  local inventory
  [[ -n "$PUBLIC_STAGING" && -d "$PUBLIC_STAGING" && ! -L "$PUBLIC_STAGING" ]] || return 0
  inventory=$(find "$PUBLIC_STAGING" -mindepth 1 -maxdepth 1 -print | LC_ALL=C sort) || return 1
  if [[ "$inventory" == "${PUBLIC_STAGING}/lumen-standalone-gke-receipt.json"$'\n'"${PUBLIC_STAGING}/lumen-standalone-gke-receipt.json.sha256" ]]; then
    rm -f -- "$PUBLIC_STAGING/lumen-standalone-gke-receipt.json" "$PUBLIC_STAGING/lumen-standalone-gke-receipt.json.sha256" || return 1
    rmdir -- "$PUBLIC_STAGING" || return 1
    PUBLIC_STAGING=''
    return 0
  fi
  return 1
}

safe_remove_run_root() {
  local inventory file dir base cache='' backup='' backup_count=0
  [[ -n "$RUN_ROOT" && "$RUN_ROOT" == "$PRIVATE_TMP_ROOT/lumen-standalone-gke-live."* && -d "$RUN_ROOT" && ! -L "$RUN_ROOT" ]] || return 1
  [[ "${RUN_ROOT%/*}" == "$PRIVATE_TMP_ROOT" ]] || return 1
  [[ "${RUN_ROOT##*/}" =~ ^lumen-standalone-gke-live\.[A-Za-z0-9]{6}$ ]] || return 1
  [[ "$(cd "$RUN_ROOT" && pwd -P)" == "$RUN_ROOT" ]] || return 1
  inventory=$(find "$RUN_ROOT" -mindepth 1 -maxdepth 1 -print) || return 1
  while IFS= read -r file; do
    base=${file##*/}
    case "$base" in
      terraform-data|control|private-receipt) [[ -d "$file" ]] || return 1 ;;
      gke_gcloud_auth_plugin_cache)
        [[ -d "$file" && ! -L "$file" && "$(cd "$file" && pwd -P)" == "$file" ]] || return 1
        cache=$file
        ;;
      terraform.tfstate|kubeconfig|create.tfplan|destroy.tfplan|recovery-destroy.tfplan|run-contract.json) [[ -f "$file" ]] || return 1 ;;
      kubeconfig.*) [[ "$base" =~ ^kubeconfig\.[0-9]{4}-(0[1-9]|1[0-2])-(0[1-9]|[12][0-9]|3[01])T([01][0-9]|2[0-3])-[0-5][0-9]-[0-5][0-9]Z\.[1-9][0-9]*\.[0-9]{2}\.backup$ && -f "$file" && ! -L "$file" ]] || return 1; backup_count=$((backup_count + 1)); backup=$file; (( backup_count <= 1 )) || return 1 ;;
      *) return 1 ;;
    esac
    [[ -L "$file" ]] && return 1
  done <<<"$inventory"
  if [[ -n "$cache" ]]; then
    inventory=$(find "$cache" -depth -print) || return 1
    while IFS= read -r file; do
      [[ -L "$file" ]] && return 1
      if [[ -d "$file" ]]; then
        [[ "$(cd "$file" && pwd -P)" == "$file" ]] || return 1
      elif [[ ! -f "$file" ]]; then
        return 1
      fi
    done <<<"$inventory"
  fi
  for dir in "$RUN_ROOT/terraform-data" "$RUN_ROOT/control" "$RUN_ROOT/private-receipt"; do
    [[ -d "$dir" && ! -L "$dir" && "$(cd "$dir" && pwd -P)" == "$dir" ]] || return 1
    inventory=$(find "$dir" -depth -print) || return 1
    while IFS= read -r file; do
      [[ -L "$file" ]] && return 1
      [[ -f "$file" || -d "$file" ]] || return 1
    done <<<"$inventory"
  done
  inventory=$(find "$RUN_ROOT/private-receipt" -mindepth 1 -maxdepth 1 -print | LC_ALL=C sort) || return 1
  case "$inventory" in
    '') ;;
    "$RUN_ROOT/private-receipt/lumen-standalone-gke-receipt.json"|"$RUN_ROOT/private-receipt/lumen-standalone-gke-receipt.json.sha256"|"$RUN_ROOT/private-receipt/lumen-standalone-gke-receipt.json"$'\n'"$RUN_ROOT/private-receipt/lumen-standalone-gke-receipt.json.sha256")
      if [[ -e "$RUN_ROOT/private-receipt/lumen-standalone-gke-receipt.json" || -L "$RUN_ROOT/private-receipt/lumen-standalone-gke-receipt.json" ]]; then
        [[ -f "$RUN_ROOT/private-receipt/lumen-standalone-gke-receipt.json" && ! -L "$RUN_ROOT/private-receipt/lumen-standalone-gke-receipt.json" ]] || return 1
      fi
      if [[ -e "$RUN_ROOT/private-receipt/lumen-standalone-gke-receipt.json.sha256" || -L "$RUN_ROOT/private-receipt/lumen-standalone-gke-receipt.json.sha256" ]]; then
        [[ -f "$RUN_ROOT/private-receipt/lumen-standalone-gke-receipt.json.sha256" && ! -L "$RUN_ROOT/private-receipt/lumen-standalone-gke-receipt.json.sha256" ]] || return 1
      fi
      ;;
    *) return 1 ;;
  esac
  for dir in "$RUN_ROOT/terraform-data" "$RUN_ROOT/control"; do
    if [[ -e "$dir" || -L "$dir" ]]; then
      [[ -d "$dir" && ! -L "$dir" && "$(cd "$dir" && pwd -P)" == "$dir" ]] || return 1
      inventory=$(find "$dir" -depth -print) || return 1
      while IFS= read -r file; do
        [[ -L "$file" ]] && return 1
        [[ -f "$file" || -d "$file" ]] || return 1
      done <<<"$inventory"
      while IFS= read -r file; do
        [[ "$file" == "$dir" ]] && continue
        if [[ -L "$file" ]]; then return 1
        elif [[ -f "$file" ]]; then rm -f -- "$file" || return 1
        elif [[ -d "$file" ]]; then rmdir -- "$file" || return 1
        else return 1; fi
      done <<<"$inventory"
      rmdir -- "$dir" || return 1
    fi
  done
  if [[ -n "$cache" ]]; then
    inventory=$(find "$cache" -depth -print) || return 1
    while IFS= read -r file; do
      [[ "$file" == "$cache" ]] && continue
      if [[ -L "$file" ]]; then return 1
      elif [[ -f "$file" ]]; then rm -f -- "$file" || return 1
      elif [[ -d "$file" ]]; then rmdir -- "$file" || return 1
      else return 1; fi
    done <<<"$inventory"
    rmdir -- "$cache" || return 1
  fi
  [[ -z "$backup" ]] || rm -f -- "$backup" || return 1
  if [[ -d "$RUN_ROOT/private-receipt" && ! -L "$RUN_ROOT/private-receipt" ]]; then
    rm -f -- "$RUN_ROOT/private-receipt/lumen-standalone-gke-receipt.json" "$RUN_ROOT/private-receipt/lumen-standalone-gke-receipt.json.sha256" || return 1
    rmdir -- "$RUN_ROOT/private-receipt" || return 1
  elif [[ -e "$RUN_ROOT/private-receipt" || -L "$RUN_ROOT/private-receipt" ]]; then return 1; fi
  for file in terraform.tfstate kubeconfig create.tfplan destroy.tfplan recovery-destroy.tfplan run-contract.json; do
    if [[ -e "$RUN_ROOT/$file" || -L "$RUN_ROOT/$file" ]]; then [[ -f "$RUN_ROOT/$file" && ! -L "$RUN_ROOT/$file" ]] || return 1; rm -f -- "$RUN_ROOT/$file" || return 1; fi
  done
  [[ -z "$(find "$RUN_ROOT" -mindepth 1 -print -quit)" ]] || return 1
  rmdir -- "$RUN_ROOT" || return 1
  RUN_ROOT=''
}

plan_has_exact_changes() {
  local json=$1 action=$2
  jq -e --arg action "$action" --arg expected "$EXPECTED_ADDRESSES" '.resource_changes as $changes | ($changes | type == "array" and length == 4) and (([$changes[].address] | unique | sort | join("\n")) == $expected) and all($changes[]; .change.actions == [$action])' "$json" >/dev/null
}

plan_has_delete_subset() {
  jq -e '
    (.resource_changes // []) as $changes |
    ($changes | type == "array" and length <= 4) and
    (([$changes[].address] | length) == ([$changes[].address] | unique | length)) and
    all($changes[];
      (.address == "google_container_cluster.standalone" or
       .address == "google_container_node_pool.standalone" or
       .address == "google_project_iam_member.node_baseline" or
       .address == "google_service_account.nodes") and
      .change.actions == ["delete"])
  ' "$1" >/dev/null
}

state_has_exact_resources() { [[ "$1" == "$EXPECTED_ADDRESSES" ]]; }

state_is_empty() {
  local output
  output=$(terraform -chdir="$MODULE_DIR" state list -state="$RUN_ROOT/terraform.tfstate" 2>"$RUN_ROOT/control/state-empty.err") || return 1
  [[ -z "$output" ]]
}

cluster_is_absent() {
  local output=$1 error=$2
  gcloud container clusters list --project="$PROJECT_ID" --location="$GKE_ZONE" --format=json >"$output" 2>"$error" || return 1
  jq -e --arg cluster "$CLUSTER" 'type == "array" and all(.[]; (.name | type == "string") and .name != $cluster)' "$output" >/dev/null
}

terraform_plan_destroy() {
  local plan=$1 json=$2 prefix=$3
  terraform -chdir="$MODULE_DIR" plan -destroy -input=false -no-color -state="$RUN_ROOT/terraform.tfstate" -out="$plan" -var="project_id=$PROJECT_ID" -var="region=$REGION" -var="gke_zone=$GKE_ZONE" -var="run_id=$RUN_ID" -var='storage_class_name=premium-rwo' >"$RUN_ROOT/control/${prefix}-plan.log" 2>"$RUN_ROOT/control/${prefix}-plan.err" || return 1
  [[ -f "$plan" && ! -L "$plan" ]] || return 1
  terraform -chdir="$MODULE_DIR" show -json "$plan" >"$json" 2>"$RUN_ROOT/control/${prefix}-show.err" || return 1
  plan_has_exact_changes "$json" delete
}

apply_saved_destroy() {
  local plan=$1 prefix=$2
  terraform -chdir="$MODULE_DIR" apply -input=false -no-color -state="$RUN_ROOT/terraform.tfstate" -backup=- "$plan" >"$RUN_ROOT/control/${prefix}-apply.log" 2>"$RUN_ROOT/control/${prefix}-apply.err" || return 1
  state_is_empty || return 1
  cluster_is_absent "$RUN_ROOT/control/${prefix}-clusters.json" "$RUN_ROOT/control/${prefix}-clusters.err" || return 1
  LIFECYCLE_PHASE='destroyed'
}

recover_destroy() {
  local plan="$RUN_ROOT/recovery-destroy.tfplan" json="$RUN_ROOT/control/recovery-destroy-plan.json"
  terraform -chdir="$MODULE_DIR" plan -destroy -input=false -no-color -state="$RUN_ROOT/terraform.tfstate" -out="$plan" -var="project_id=$PROJECT_ID" -var="region=$REGION" -var="gke_zone=$GKE_ZONE" -var="run_id=$RUN_ID" -var='storage_class_name=premium-rwo' >"$RUN_ROOT/control/recovery-plan.log" 2>"$RUN_ROOT/control/recovery-plan.err" || return 1
  [[ -f "$plan" && ! -L "$plan" ]] || return 1
  terraform -chdir="$MODULE_DIR" show -json "$plan" >"$json" 2>"$RUN_ROOT/control/recovery-show.err" || return 1
  plan_has_delete_subset "$json" || return 1
  apply_saved_destroy "$plan" recovery
}

on_exit() {
  local status=$?
  trap - EXIT INT TERM
  set +e
  cleanup_public_staging
  if [[ "$status" -ne 0 && -n "${RUN_ROOT:-}" ]]; then
    case "$LIFECYCLE_PHASE" in
      precreate) safe_remove_run_root || printf '%s\n' 'private lifecycle state was retained' >&2 ;;
      create-uncertain) repair_command ;;
      created) if recover_destroy; then safe_remove_run_root || repair_command; else repair_command; fi ;;
      destroyed) safe_remove_run_root || repair_command ;;
    esac
  fi
  exit "$status"
}

assert_outputs() {
  jq -e --arg project "$PROJECT_ID" --arg region "$REGION" --arg zone "$GKE_ZONE" --arg run "$RUN_ID" --arg cluster "$CLUSTER" --arg pool "$NODE_POOL" --arg node_sa "$NODE_SERVICE_ACCOUNT" '(.values.outputs | keys | sort) == ["cluster_name","gke_zone","node_pool_name","node_selector","node_service_account_email","project_id","region","run_id","storage_class_name","workload_identity_pool"] and .values.outputs.project_id.value == $project and .values.outputs.region.value == $region and .values.outputs.gke_zone.value == $zone and .values.outputs.run_id.value == $run and .values.outputs.cluster_name.value == $cluster and .values.outputs.node_pool_name.value == $pool and .values.outputs.node_selector.value == {"cloud.google.com/gke-nodepool":$pool} and .values.outputs.storage_class_name.value == "premium-rwo" and .values.outputs.workload_identity_pool.value == ($project + ".svc.id.goog") and .values.outputs.node_service_account_email.value == ($node_sa + "@" + $project + ".iam.gserviceaccount.com")' "$1" >/dev/null
}

assert_cluster() {
  jq -e --arg cluster "$CLUSTER" --arg zone "$GKE_ZONE" --arg project "$PROJECT_ID" --arg owner "$OWNER_LABEL" '.name == $cluster and .location == $zone and .status == "RUNNING" and ((.autopilot.enabled // false) == false) and .ipAllocationPolicy.useIpAliases == true and .networkConfig.datapathProvider == "ADVANCED_DATAPATH" and .releaseChannel.channel == "REGULAR" and .workloadIdentityConfig.workloadPool == ($project + ".svc.id.goog") and .addonsConfig.gcePersistentDiskCsiDriverConfig.enabled == true and ((.loggingConfig.componentConfig.enableComponents | sort) == ["SYSTEM_COMPONENTS","WORKLOADS"]) and .resourceLabels["lumen-owner"] == $owner and ([.resourceLabels | to_entries[] | select(.key | startswith("lumen-"))] | length) == 1' "$1" >/dev/null
}

assert_node_pool_inventory() { jq -e --arg pool "$NODE_POOL" 'type == "array" and length == 1 and .[0].name == $pool' "$1" >/dev/null; }

assert_node_pool() {
  jq -e --arg pool "$NODE_POOL" --arg project "$PROJECT_ID" --arg node_sa "$NODE_SERVICE_ACCOUNT" --arg owner "$OWNER_LABEL" '.name == $pool and .status == "RUNNING" and .initialNodeCount == 1 and .config.machineType == "e2-standard-2" and .autoscaling.enabled == true and .autoscaling.minNodeCount == 1 and .autoscaling.maxNodeCount == 3 and ((.config.taints // []) == []) and .config.workloadMetadataConfig.mode == "GKE_METADATA" and ((.config.oauthScopes | sort) == ["https://www.googleapis.com/auth/cloud-platform"]) and .config.metadata["disable-legacy-endpoints"] == "true" and .config.labels["lumen-owner"] == $owner and ([.config.labels | to_entries[] | select(.key | startswith("lumen-"))] | length) == 1 and .config.serviceAccount == ($node_sa + "@" + $project + ".iam.gserviceaccount.com")' "$1" >/dev/null
}

assert_kubeconfig() {
  jq -e --arg endpoint "$2" '(.contexts | type == "array" and length == 1) and (.clusters | type == "array" and length == 1) and (.users | type == "array" and length == 1) and ."current-context" == .contexts[0].name and .contexts[0].context.cluster == .clusters[0].name and .contexts[0].context.user == .users[0].name and .clusters[0].cluster.server == ("https://" + $endpoint)' "$1" >/dev/null
}

assert_storage_class() {
  jq -e '.metadata.name == "premium-rwo" and .provisioner == "pd.csi.storage.gke.io" and .parameters.type == "pd-ssd" and .reclaimPolicy == "Delete" and .volumeBindingMode == "WaitForFirstConsumer" and .allowVolumeExpansion == true' "$1" >/dev/null
}

run_static_prechecks() {
  bash "$MODULE_DIR/scripts/check.sh" >"$RUN_ROOT/control/terraform-check.log" 2>"$RUN_ROOT/control/terraform-check.err" || die 'Terraform cloud-free contract failed'
  bash "$REPO_ROOT/kustomize/lumen-standalone-acceptance/tests/contract.sh" >"$RUN_ROOT/control/kustomize-check.log" 2>"$RUN_ROOT/control/kustomize-check.err" || die 'Kustomize cloud-free contract failed'
}

create_cluster() {
  local state_resources
  cluster_is_absent "$RUN_ROOT/control/pre-create-clusters.json" "$RUN_ROOT/control/pre-create-clusters.err" || die 'cannot prove cluster absence'
  terraform -chdir="$MODULE_DIR" init -backend=false -input=false -lockfile=readonly -no-color >"$RUN_ROOT/control/init.log" 2>"$RUN_ROOT/control/init.err" || die 'Terraform init failed'
  terraform -chdir="$MODULE_DIR" plan -input=false -no-color -state="$RUN_ROOT/terraform.tfstate" -out="$RUN_ROOT/create.tfplan" -var="project_id=$PROJECT_ID" -var="region=$REGION" -var="gke_zone=$GKE_ZONE" -var="run_id=$RUN_ID" -var='storage_class_name=premium-rwo' >"$RUN_ROOT/control/create-plan.log" 2>"$RUN_ROOT/control/create-plan.err" || die 'Terraform create plan failed'
  [[ -f "$RUN_ROOT/create.tfplan" && ! -L "$RUN_ROOT/create.tfplan" ]] || die 'Terraform create plan path is unsafe'
  terraform -chdir="$MODULE_DIR" show -json "$RUN_ROOT/create.tfplan" >"$RUN_ROOT/control/create-plan.json" 2>"$RUN_ROOT/control/create-show.err" || die 'cannot inspect Terraform create plan'
  plan_has_exact_changes "$RUN_ROOT/control/create-plan.json" create || die 'Terraform create plan changed the fixed resource set'
  LIFECYCLE_PHASE='create-uncertain'
  terraform -chdir="$MODULE_DIR" apply -input=false -no-color -state="$RUN_ROOT/terraform.tfstate" -backup=- "$RUN_ROOT/create.tfplan" >"$RUN_ROOT/control/create-apply.log" 2>"$RUN_ROOT/control/create-apply.err" || die 'Terraform create apply failed'
  LIFECYCLE_PHASE='created'
  state_resources=$(terraform -chdir="$MODULE_DIR" state list -state="$RUN_ROOT/terraform.tfstate" 2>"$RUN_ROOT/control/state-list.err" | LC_ALL=C sort) || die 'cannot read Terraform state'
  state_has_exact_resources "$state_resources" || die 'Terraform state changed the fixed resource set'
  terraform -chdir="$MODULE_DIR" show -json "$RUN_ROOT/terraform.tfstate" >"$RUN_ROOT/control/state.json" 2>"$RUN_ROOT/control/state-show.err" || die 'cannot inspect Terraform state'
  assert_outputs "$RUN_ROOT/control/state.json" || die 'Terraform outputs changed'
}

validate_live_infrastructure() {
  local endpoint
  gcloud container clusters describe "$CLUSTER" --project="$PROJECT_ID" --location="$GKE_ZONE" --format=json >"$RUN_ROOT/control/cluster.json" 2>"$RUN_ROOT/control/cluster.err" || die 'cannot describe created cluster'
  assert_cluster "$RUN_ROOT/control/cluster.json" || die 'created cluster violates the fixed contract'
  gcloud container node-pools list --cluster="$CLUSTER" --project="$PROJECT_ID" --location="$GKE_ZONE" --format=json >"$RUN_ROOT/control/node-pools.json" 2>"$RUN_ROOT/control/node-pools.err" || die 'cannot list created node pools'
  assert_node_pool_inventory "$RUN_ROOT/control/node-pools.json" || die 'created cluster has the wrong node-pool inventory'
  gcloud container node-pools describe "$NODE_POOL" --cluster="$CLUSTER" --project="$PROJECT_ID" --location="$GKE_ZONE" --format=json >"$RUN_ROOT/control/node-pool.json" 2>"$RUN_ROOT/control/node-pool.err" || die 'cannot describe created node pool'
  assert_node_pool "$RUN_ROOT/control/node-pool.json" || die 'created node pool violates the fixed contract'
  gcloud container clusters get-credentials "$CLUSTER" --project="$PROJECT_ID" --location="$GKE_ZONE" >"$RUN_ROOT/control/get-credentials.log" 2>"$RUN_ROOT/control/get-credentials.err" || die 'cannot create task-local kubeconfig'
  kubectl --kubeconfig "$KUBECONFIG" config view --raw -o json >"$RUN_ROOT/control/kubeconfig.json" 2>"$RUN_ROOT/control/kubeconfig.err" || die 'cannot inspect task-local kubeconfig'
  endpoint=$(jq -er '.endpoint' "$RUN_ROOT/control/cluster.json") || die 'created cluster has no endpoint'
  assert_kubeconfig "$RUN_ROOT/control/kubeconfig.json" "$endpoint" || die 'task-local kubeconfig is not bound to the created cluster'
  CONTEXT=$(jq -er '."current-context"' "$RUN_ROOT/control/kubeconfig.json") || die 'task-local kubeconfig has no current context'
  kubectl --kubeconfig "$KUBECONFIG" --context "$CONTEXT" get storageclass premium-rwo -o json >"$RUN_ROOT/control/storage-class.json" 2>"$RUN_ROOT/control/storage-class.err" || die 'cannot read premium-rwo StorageClass'
  assert_storage_class "$RUN_ROOT/control/storage-class.json" || die 'premium-rwo StorageClass violates the fixed contract'
}

run_inner_gate() {
  env -i PATH="$PATH" KUBECONFIG="$KUBECONFIG" LUMEN_STANDALONE_GKE_CONTEXT="$CONTEXT" LUMEN_STANDALONE_GKE_PROJECT_ID="$PROJECT_ID" LUMEN_STANDALONE_GKE_LOCATION="$GKE_ZONE" LUMEN_STANDALONE_GKE_CLUSTER="$CLUSTER" LUMEN_STANDALONE_GKE_CLI="$LUMEN_CLI" LUMEN_STANDALONE_GKE_IMAGE="$IMAGE" LUMEN_STANDALONE_GKE_CLIENT_IMAGE="$CLIENT_IMAGE" LUMEN_STANDALONE_GKE_CLI_TARGET="$CLI_TARGET" LUMEN_STANDALONE_GKE_STORAGE_CLASS='premium-rwo' LUMEN_STANDALONE_GKE_NODE_POOL="$NODE_POOL" LUMEN_STANDALONE_GKE_RUN_ID="$RUN_ID" LUMEN_STANDALONE_GKE_EXPECTED_COMMIT="$EXPECTED_COMMIT" LUMEN_STANDALONE_GKE_EXPECTED_RUN_ID="$EXPECTED_RUN_ID" LUMEN_STANDALONE_GKE_EXPECTED_RUN_ATTEMPT="$EXPECTED_RUN_ATTEMPT" LUMEN_STANDALONE_GKE_EXPECTED_MANIFEST_SHA256="$EXPECTED_MANIFEST_SHA256" LUMEN_STANDALONE_GKE_EVIDENCE_DIR="$RUN_ROOT/private-receipt" LUMEN_STANDALONE_GKE_MUTATION=1 LUMEN_STANDALONE_GKE_CANDIDATE_RECEIPT_DIR="$CANDIDATE_RECEIPT_DIR" bash "$REPO_ROOT/apps/lumen/scripts/standalone-gke-acceptance.sh" --mode gke >"$RUN_ROOT/control/inner.log" 2>"$RUN_ROOT/control/inner.err" || die 'inner standalone GKE gate failed'
}

validate_private_receipt() {
  local receipt="$RUN_ROOT/private-receipt/lumen-standalone-gke-receipt.json" sidecar inventory candidate_dir
  sidecar="$receipt.sha256"
  candidate_dir=$CANDIDATE_RECEIPT_DIR
  inventory=$(find "$RUN_ROOT/private-receipt" -mindepth 1 -maxdepth 1 -type f -exec basename {} \; | LC_ALL=C sort) || die 'cannot inspect private receipt'
  [[ "$inventory" == $'lumen-standalone-gke-receipt.json\nlumen-standalone-gke-receipt.json.sha256' ]] || die 'inner gate produced an unexpected receipt inventory'
  ( REPO='chrischeng-c4/axiom'; TAG='lumen@0.4.29'; COMMIT="$EXPECTED_COMMIT"; CANDIDATE_RUN_ID="$EXPECTED_RUN_ID"; CANDIDATE_RECEIPT_DIR="$candidate_dir"; STANDALONE_GKE_RECEIPT="$receipt"; STANDALONE_GKE_RECEIPT_SIDECAR="$sidecar"; source "$REPO_ROOT/apps/lumen/scripts/verify-release-artifacts.sh"; validate_receipt "$CANDIDATE_RECEIPT_DIR/final-candidate-manifest.json" "$CANDIDATE_RECEIPT_DIR/final-candidate-manifest.json.sha256" "$CANDIDATE_RECEIPT_DIR"; CANDIDATE_ATTEMPT="$EXPECTED_RUN_ATTEMPT"; validate_standalone_gke_receipt ) >"$RUN_ROOT/control/receipt-verify.log" 2>"$RUN_ROOT/control/receipt-verify.err" || die 'private standalone GKE receipt failed verification'
}

destroy_cluster() { terraform_plan_destroy "$RUN_ROOT/destroy.tfplan" "$RUN_ROOT/control/destroy-plan.json" destroy || die 'Terraform destroy plan changed the fixed resource set'; apply_saved_destroy "$RUN_ROOT/destroy.tfplan" destroy || die 'Terraform destroy or absence proof failed'; }

prepare_public_staging() {
  local receipt="$RUN_ROOT/private-receipt/lumen-standalone-gke-receipt.json" sidecar digest inventory sidecar_hash sidecar_name sidecar_extra
  sidecar="$receipt.sha256"
  digest=$(sha256_file "$receipt") || die 'cannot hash private receipt'
  read -r sidecar_hash sidecar_name sidecar_extra <"$sidecar" || die 'cannot read private receipt sidecar'
  [[ "$sidecar_hash" == "$digest" && "$sidecar_name" == 'lumen-standalone-gke-receipt.json' && -z "${sidecar_extra:-}" ]] || die 'private receipt sidecar does not bind exact bytes'
  PUBLIC_STAGING=$(mktemp -d "${RECEIPT_OUT_DIR}.tmp.XXXXXX")
  chmod 700 "$PUBLIC_STAGING"
  cp "$receipt" "$PUBLIC_STAGING/lumen-standalone-gke-receipt.json"
  cp "$sidecar" "$PUBLIC_STAGING/lumen-standalone-gke-receipt.json.sha256"
  chmod 600 "$PUBLIC_STAGING/lumen-standalone-gke-receipt.json" "$PUBLIC_STAGING/lumen-standalone-gke-receipt.json.sha256"
  [[ "$(sha256_file "$PUBLIC_STAGING/lumen-standalone-gke-receipt.json")" == "$digest" ]] || die 'staged receipt bytes changed'
  cmp -s "$sidecar" "$PUBLIC_STAGING/lumen-standalone-gke-receipt.json.sha256" || die 'staged receipt sidecar bytes changed'
  inventory=$(find "$PUBLIC_STAGING" -mindepth 1 -maxdepth 1 -type f -exec basename {} \; | LC_ALL=C sort) || die 'cannot inspect staged receipt inventory'
  [[ "$inventory" == $'lumen-standalone-gke-receipt.json\nlumen-standalone-gke-receipt.json.sha256' ]] || die 'staged receipt inventory changed'
  PUBLIC_RECEIPT_DIGEST=$digest
}

main() {
  local digest
  parse_args "$@"
  validate_inputs
  require_tools
  validate_repo_inputs
  prepare_run_root
  trap on_exit EXIT
  trap 'exit 130' INT
  trap 'exit 143' TERM
  run_static_prechecks
  create_cluster
  validate_live_infrastructure
  run_inner_gate
  validate_private_receipt
  destroy_cluster
  prepare_public_staging
  digest=$PUBLIC_RECEIPT_DIGEST
  safe_remove_run_root || die 'private lifecycle cleanup failed'
  mv "$PUBLIC_STAGING" "$RECEIPT_OUT_DIR" || die 'atomic receipt publication failed'
  PUBLIC_STAGING=''
  trap - EXIT INT TERM
  printf '%s\n%s\n' "$digest" "$RECEIPT_OUT_DIR"
}

if [[ "${BASH_SOURCE[0]}" == "$0" ]]; then main "$@"; fi
