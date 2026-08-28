#!/usr/bin/env bash
# shellcheck disable=SC1091,SC2312
set -euo pipefail
set +x
umask 077
CDPATH=

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd -P)"
# Source only the shared identity, path, and Terraform helpers. The guard in
# live-acceptance.sh prevents its main function from running.
source "$SCRIPT_DIR/live-acceptance.sh"

STATE_DIR=''
REPAIR_CONFIRM=''
REPAIR_SEEN_OPTIONS='|'

require_repair_tools() {
  local tool
  for tool in jq terraform gcloud awk cut rm mkdir; do
    command -v "$tool" >/dev/null 2>&1 || die "required repair tool is missing: $tool"
  done
  if ! command -v sha256sum >/dev/null 2>&1 && ! command -v shasum >/dev/null 2>&1; then die 'sha256 tool is missing'; fi
}

repair_usage() {
  printf '%s\n' 'usage: repair-destroy.sh --state-dir /private/tmp/lumen-standalone-gke-live.XXXXXX --confirm-destroy CLUSTER' >&2
  exit 2
}

parse_repair_args() {
  while [[ $# -gt 0 ]]; do
    [[ $# -ge 2 && -n "$2" && "$2" != --* ]] || repair_usage
    case "$REPAIR_SEEN_OPTIONS" in *"|$1|"*) die 'duplicate option' ;; esac
    REPAIR_SEEN_OPTIONS="${REPAIR_SEEN_OPTIONS}${1}|"
    case "$1" in
      --state-dir) STATE_DIR=$2 ;;
      --confirm-destroy) REPAIR_CONFIRM=$2 ;;
      *) die 'unknown option' ;;
    esac
    shift 2
  done
  [[ -n "$STATE_DIR" && -n "$REPAIR_CONFIRM" ]] || repair_usage
}

validate_contract_shape() {
  jq -e '
    (keys | sort) == [
      "candidate_receipt_dir","cli_target","cluster_name","confirm_create","confirm_destroy",
      "expected_commit","expected_manifest_sha256","expected_run_attempt","expected_run_id",
      "gke_zone","image","lumen_cli","node_pool_name","node_service_account_id","project_id",
      "receipt_out_dir","region","run_id","schema","state_dir","storage_class_name"
    ] and
    all(.[]; type == "string") and
    .schema == "lumen.standalone-gke-live/v1" and
    (.project_id | test("^[a-z][a-z0-9-]{4,28}[a-z0-9]$")) and
    (.region | test("^[a-z]+-[a-z0-9]+[0-9]$")) and
    (.gke_zone | test("^[a-z]+-[a-z0-9]+[0-9]-[a-z]$")) and
    (.run_id | test("^[a-z0-9][a-z0-9-]{0,39}[a-z0-9]$")) and
    (.image | test("^ghcr\\.io/chrischeng-c4/lumen@sha256:[0-9a-f]{64}$")) and
    (.expected_commit | test("^[0-9a-f]{40}$")) and
    (.expected_run_id | test("^[0-9]+$")) and
    (.expected_run_attempt | test("^[0-9]+$")) and
    (.expected_manifest_sha256 | test("^[0-9a-f]{64}$")) and
    .storage_class_name == "premium-rwo"
  ' "$1" >/dev/null
}

load_contract() {
  local contract="$STATE_DIR/run-contract.json"
  [[ -f "$contract" && ! -L "$contract" ]] || die 'repair contract is missing or unsafe'
  validate_contract_shape "$contract" || die 'repair contract shape is invalid'

  PROJECT_ID=$(jq -er '.project_id' "$contract")
  REGION=$(jq -er '.region' "$contract")
  GKE_ZONE=$(jq -er '.gke_zone' "$contract")
  RUN_ID=$(jq -er '.run_id' "$contract")
  CLUSTER=$(jq -er '.cluster_name' "$contract")
  NODE_POOL=$(jq -er '.node_pool_name' "$contract")
  NODE_SERVICE_ACCOUNT=$(jq -er '.node_service_account_id' "$contract")
  CONFIRM_CREATE=$(jq -er '.confirm_create' "$contract")
  CONFIRM_DESTROY=$(jq -er '.confirm_destroy' "$contract")

  [[ "$(jq -er '.state_dir' "$contract")" == "$STATE_DIR" ]] || die 'repair contract is bound to another state directory'
  [[ "$GKE_ZONE" == "$REGION"-* ]] || die 'repair contract region and zone disagree'
  case "$PROJECT_ID" in replace-with-real-project-id|example-project|placeholder-project|your-project-id|required-project-id) die 'repair project id is a placeholder' ;; esac
  case "$RUN_ID" in run|test|placeholder) die 'repair run id is a placeholder' ;; esac
  [[ "$CLUSTER" == "$(cluster_name "$RUN_ID")" ]] || die 'repair cluster identity changed'
  [[ "$NODE_POOL" == "$(pool_name "$RUN_ID")" ]] || die 'repair node-pool identity changed'
  [[ "$NODE_SERVICE_ACCOUNT" == "$(node_service_account "$RUN_ID")" ]] || die 'repair node service-account identity changed'
  [[ "$CONFIRM_CREATE" == "$CLUSTER" && "$CONFIRM_DESTROY" == "$CLUSTER" && "$REPAIR_CONFIRM" == "$CLUSTER" ]] || die 'repair confirmation does not match derived identity'
  case "$(jq -er '.cli_target' "$contract")" in aarch64-apple-darwin|x86_64-unknown-linux-gnu|aarch64-unknown-linux-gnu|x86_64-unknown-linux-musl|aarch64-unknown-linux-musl) ;; *) die 'repair contract CLI target is invalid' ;; esac
}

repair_plan_is_safe() {
  plan_has_delete_subset "$1"
}

prepare_repair_plan() {
  local plan="$STATE_DIR/repair-destroy.tfplan" json="$STATE_DIR/control/repair-destroy-plan.json"
  if [[ -e "$plan" || -L "$plan" ]]; then
    [[ -f "$plan" && ! -L "$plan" ]] || die 'saved repair plan path is unsafe'
    rm -f -- "$plan"
  fi
  terraform -chdir="$MODULE_DIR" plan -destroy -input=false -no-color \
    -state="$STATE_DIR/terraform.tfstate" \
    -out="$plan" \
    -var="project_id=$PROJECT_ID" \
    -var="region=$REGION" \
    -var="gke_zone=$GKE_ZONE" \
    -var="run_id=$RUN_ID" \
    -var='storage_class_name=premium-rwo' \
    >"$STATE_DIR/control/repair-plan.log" 2>"$STATE_DIR/control/repair-plan.err" || die 'repair destroy plan failed'
  [[ -f "$plan" && ! -L "$plan" ]] || die 'saved repair plan path is unsafe'
  terraform -chdir="$MODULE_DIR" show -json "$plan" >"$json" 2>"$STATE_DIR/control/repair-show.err" || die 'cannot inspect saved repair plan'
  repair_plan_is_safe "$json" || die 'saved repair plan exceeds the known delete subset'
}

main_repair() {
  local state_output
  parse_repair_args "$@"
  [[ "$STATE_DIR" =~ ^/private/tmp/lumen-standalone-gke-live\.[A-Za-z0-9]{6}$ ]] || die 'repair state directory name is unsafe'
  safe_private_dir "$STATE_DIR" || die 'repair state directory is unsafe'
  load_contract
  require_repair_tools
  [[ -f "$STATE_DIR/terraform.tfstate" && ! -L "$STATE_DIR/terraform.tfstate" ]] || die 'repair state file is missing or unsafe'
  [[ -d "$STATE_DIR/terraform-data" && ! -L "$STATE_DIR/terraform-data" ]] || die 'repair Terraform data directory is missing or unsafe'
  if [[ -e "$STATE_DIR/control" || -L "$STATE_DIR/control" ]]; then
    [[ -d "$STATE_DIR/control" && ! -L "$STATE_DIR/control" ]] || die 'repair control directory is unsafe'
  else
    mkdir -m 700 "$STATE_DIR/control"
  fi
  export TF_DATA_DIR="$STATE_DIR/terraform-data"

  terraform -chdir="$MODULE_DIR" init -backend=false -input=false -lockfile=readonly -no-color >"$STATE_DIR/control/repair-init.log" 2>"$STATE_DIR/control/repair-init.err" || die 'repair Terraform init failed'
  terraform -chdir="$MODULE_DIR" state list -state="$STATE_DIR/terraform.tfstate" >"$STATE_DIR/control/repair-state-before.txt" 2>"$STATE_DIR/control/repair-state-before.err" || die 'repair cannot read Terraform state'
  prepare_repair_plan
  terraform -chdir="$MODULE_DIR" apply -input=false -no-color -state="$STATE_DIR/terraform.tfstate" -backup=- "$STATE_DIR/repair-destroy.tfplan" >"$STATE_DIR/control/repair-apply.log" 2>"$STATE_DIR/control/repair-apply.err" || die 'saved repair destroy apply failed'
  state_output=$(terraform -chdir="$MODULE_DIR" state list -state="$STATE_DIR/terraform.tfstate" 2>"$STATE_DIR/control/repair-state-after.err") || die 'repair cannot prove empty Terraform state'
  [[ -z "$state_output" ]] || die 'repair Terraform state is not empty'
  cluster_is_absent "$STATE_DIR/control/repair-clusters.json" "$STATE_DIR/control/repair-clusters.err" || die 'repair cannot prove cluster absence'
  printf '%s\n' 'standalone GKE repair: destroy verified; private state retained'
}

if [[ "${BASH_SOURCE[0]}" == "$0" ]]; then main_repair "$@"; fi
