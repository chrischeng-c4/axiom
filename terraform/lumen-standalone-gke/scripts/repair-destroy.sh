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
REPAIR_STAGE=''
REPAIR_ATTEMPT=''
REPAIR_STAGE_MOVED=0

require_repair_tools() {
  local tool
  for tool in jq terraform gcloud awk cut rm mkdir mktemp chmod cp mv stat; do
    command -v "$tool" >/dev/null 2>&1 || die "required repair tool is missing: $tool"
  done
  if ! command -v sha256sum >/dev/null 2>&1 && ! command -v shasum >/dev/null 2>&1; then die 'sha256 tool is missing'; fi
}

repair_usage() {
  printf 'usage: repair-destroy.sh --state-dir %s/lumen-standalone-gke-live.XXXXXX --confirm-destroy CLUSTER\n' "$PRIVATE_TMP_ROOT" >&2
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
      "gke_zone","image","lumen_cli","node_pool_name","node_service_account_id","private_temp_root",
      "project_id","receipt_out_dir","region","run_id","schema","state_dir","storage_class_name"
    ] and
    all(.[]; type == "string") and
    .schema == "lumen.standalone-gke-live/v2" and
    (.private_temp_root == "/tmp" or .private_temp_root == "/private/tmp") and
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

  [[ "$(jq -er '.private_temp_root' "$contract")" == "$PRIVATE_TMP_ROOT" ]] || die 'repair contract uses another private temp root'
  [[ "$STATE_DIR" == "$PRIVATE_TMP_ROOT/lumen-standalone-gke-live."* ]] || die 'repair state directory is outside the private temp root'
  [[ "${STATE_DIR%/*}" == "$PRIVATE_TMP_ROOT" ]] || die 'repair state directory parent is unsafe'
  [[ -d "$STATE_DIR" && ! -L "$STATE_DIR" && "$(cd "$STATE_DIR" && pwd -P)" == "$STATE_DIR" ]] || die 'repair state directory is unsafe'
  [[ -f "$contract" && ! -L "$contract" ]] || die 'repair contract path is unsafe'

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
  local owner email
  owner=$(owner_label "$RUN_ID") || return 1
  email="${NODE_SERVICE_ACCOUNT}@${PROJECT_ID}.iam.gserviceaccount.com"
  jq -e --arg project "$PROJECT_ID" --arg zone "$GKE_ZONE" --arg cluster "$CLUSTER" \
    --arg pool "$NODE_POOL" --arg account "$NODE_SERVICE_ACCOUNT" --arg node_sa "$email" --arg owner "$owner" '
    (.resource_changes) as $changes |
    ($changes | type == "array" and length <= 4) and
    (([$changes[].address] | length) == ([$changes[].address] | unique | length)) and
    all($changes[];
      .mode == "managed" and
      (.change | type == "object" and has("before") and has("after")) and
      .change.actions == ["delete"] and
      (.change.before | type == "object") and
      .change.after == null and
      (if .address == "google_container_cluster.standalone" then
        .type == "google_container_cluster" and .name == "standalone" and
        .change.before.project == $project and .change.before.name == $cluster and
        .change.before.location == $zone and .change.before.resource_labels["lumen-owner"] == $owner
      elif .address == "google_container_node_pool.standalone" then
        .type == "google_container_node_pool" and .name == "standalone" and
        .change.before.project == $project and .change.before.name == $pool and
        .change.before.location == $zone and .change.before.cluster == $cluster and
        (.change.before.node_config | type == "array" and length == 1) and
        .change.before.node_config[0].service_account == $node_sa and
        .change.before.node_config[0].labels["lumen-owner"] == $owner
      elif .address == "google_project_iam_member.node_baseline" then
        .type == "google_project_iam_member" and .name == "node_baseline" and
        .change.before.project == $project and
        .change.before.role == "roles/container.defaultNodeServiceAccount" and
        .change.before.member == ("serviceAccount:" + $node_sa)
      elif .address == "google_service_account.nodes" then
        .type == "google_service_account" and .name == "nodes" and
        .change.before.project == $project and .change.before.account_id == $account and
        .change.before.email == $node_sa
      else false end)
    )
  ' "$1" >/dev/null
}

cleanup_repair_stage() {
  if [[ "$REPAIR_STAGE_MOVED" -eq 0 && -n "$REPAIR_STAGE" &&
    "${REPAIR_STAGE%/*}" == "$PRIVATE_TMP_ROOT" &&
    "${REPAIR_STAGE##*/}" =~ ^lumen-standalone-gke-repair\.[A-Za-z0-9]{6}$ &&
    -d "$REPAIR_STAGE" && ! -L "$REPAIR_STAGE" &&
    "$(cd "$REPAIR_STAGE" && pwd -P)" == "$REPAIR_STAGE" &&
    "$(private_mode "$REPAIR_STAGE")" == 700 ]]; then
    rm -rf -- "$REPAIR_STAGE"
  fi
}

prepare_repair_stage() {
  local source_state="$STATE_DIR/terraform.tfstate" source_digest
  REPAIR_STAGE=$(mktemp -d "$PRIVATE_TMP_ROOT/lumen-standalone-gke-repair.XXXXXX") || die 'cannot create repair staging directory'
  [[ -d "$REPAIR_STAGE" && ! -L "$REPAIR_STAGE" && "$(cd "$REPAIR_STAGE" && pwd -P)" == "$REPAIR_STAGE" ]] || die 'repair staging directory is unsafe'
  [[ "$(cd "${REPAIR_STAGE%/*}" && pwd -P)" == "$PRIVATE_TMP_ROOT" ]] || die 'repair staging parent is unsafe'
  chmod 700 "$REPAIR_STAGE"
  [[ "$(private_mode "$REPAIR_STAGE")" == 700 ]] || die 'repair staging directory mode is not 0700'
  export LUMEN_STANDALONE_GKE_REPAIR_WORK_DIR="$REPAIR_STAGE"
  mkdir -m 700 "$REPAIR_STAGE/terraform-data" "$REPAIR_STAGE/control"
  safe_private_file "$source_state" || die 'repair state file is missing or unsafe'
  source_digest=$(sha256_file "$source_state") || die 'cannot hash repair state'
  cp -- "$source_state" "$REPAIR_STAGE/terraform.tfstate" || die 'cannot copy repair state into staging'
  chmod 600 "$REPAIR_STAGE/terraform.tfstate"
  [[ -f "$REPAIR_STAGE/terraform.tfstate" && ! -L "$REPAIR_STAGE/terraform.tfstate" ]] || die 'repair staged state is unsafe'
  [[ "$(sha256_file "$REPAIR_STAGE/terraform.tfstate")" == "$source_digest" ]] || die 'repair staged state hash changed'
}

prepare_repair_plan() {
  local plan="$REPAIR_STAGE/repair-destroy.tfplan" json="$REPAIR_STAGE/control/repair-destroy-plan.json"
  export TF_DATA_DIR="$REPAIR_STAGE/terraform-data"
  terraform -chdir="$MODULE_DIR" init -backend=false -input=false -lockfile=readonly -no-color >"$REPAIR_STAGE/control/repair-init.log" 2>"$REPAIR_STAGE/control/repair-init.err" || die 'repair Terraform init failed'
  terraform -chdir="$MODULE_DIR" state list -state="$REPAIR_STAGE/terraform.tfstate" >"$REPAIR_STAGE/control/repair-state-before.txt" 2>"$REPAIR_STAGE/control/repair-state-before.err" || die 'repair cannot read Terraform state'
  terraform -chdir="$MODULE_DIR" plan -destroy -input=false -no-color \
    -state="$REPAIR_STAGE/terraform.tfstate" \
    -out="$plan" \
    -var="project_id=$PROJECT_ID" \
    -var="region=$REGION" \
    -var="gke_zone=$GKE_ZONE" \
    -var="run_id=$RUN_ID" \
    -var='storage_class_name=premium-rwo' \
    >"$REPAIR_STAGE/control/repair-plan.log" 2>"$REPAIR_STAGE/control/repair-plan.err" || die 'repair destroy plan failed'
  [[ -f "$plan" && ! -L "$plan" ]] || die 'saved repair plan path is unsafe'
  terraform -chdir="$MODULE_DIR" show -json "$plan" >"$json" 2>"$REPAIR_STAGE/control/repair-show.err" || die 'cannot inspect saved repair plan'
  [[ -d "$REPAIR_STAGE/control" && ! -L "$REPAIR_STAGE/control" && -f "$json" && ! -L "$json" ]] || die 'repair plan guard path is unsafe'
  repair_plan_is_safe "$json" || die 'saved repair plan exceeds the known delete subset'
}

promote_repair_plan() {
  local digest nonce attempt plan
  plan="$REPAIR_STAGE/repair-destroy.tfplan"
  digest=$(sha256_file "$plan") || die 'cannot hash saved repair plan'
  [[ "$digest" =~ ^[0-9a-f]{64}$ ]] || die 'saved repair plan hash is invalid'
  nonce=$(sha256_text "$digest:$$" | cut -c1-16) || die 'cannot derive repair attempt identity'
  attempt="$STATE_DIR/repair-attempt.$nonce"
  safe_new_private_path "$attempt" || die 'repair attempt evidence already exists or is unsafe'
  mv -- "$REPAIR_STAGE" "$attempt" || die 'cannot atomically retain repair attempt'
  REPAIR_ATTEMPT="$attempt"
  REPAIR_STAGE_MOVED=1
  REPAIR_STAGE=''
  [[ -d "$attempt" && ! -L "$attempt" && "$(cd "$attempt" && pwd -P)" == "$attempt" ]] || die 'retained repair attempt is unsafe'
  plan="$attempt/repair-destroy.tfplan"
  [[ "$(sha256_file "$plan")" == "$digest" ]] || die 'retained repair plan hash changed'
  export TF_DATA_DIR="$attempt/terraform-data"
  export LUMEN_STANDALONE_GKE_REPAIR_WORK_DIR="$attempt"
}

main_repair() {
  local state_output
  parse_repair_args "$@"
  [[ "$STATE_DIR" == "$PRIVATE_TMP_ROOT/lumen-standalone-gke-live."* ]] || die 'repair state directory is outside the private temp root'
  [[ "${STATE_DIR%/*}" == "$PRIVATE_TMP_ROOT" ]] || die 'repair state directory parent is unsafe'
  [[ "${STATE_DIR##*/}" =~ ^lumen-standalone-gke-live\.[A-Za-z0-9]{6}$ ]] || die 'repair state directory name is unsafe'
  safe_private_dir "$STATE_DIR" || die 'repair state directory is unsafe'
  load_contract
  require_repair_tools
  trap cleanup_repair_stage EXIT
  trap 'exit 130' INT
  trap 'exit 143' TERM
  prepare_repair_stage
  prepare_repair_plan
  promote_repair_plan
  terraform -chdir="$MODULE_DIR" apply -input=false -no-color -state="$STATE_DIR/terraform.tfstate" -backup=- "$REPAIR_ATTEMPT/repair-destroy.tfplan" >"$REPAIR_ATTEMPT/control/repair-apply.log" 2>"$REPAIR_ATTEMPT/control/repair-apply.err" || die 'saved repair destroy apply failed'
  state_output=$(terraform -chdir="$MODULE_DIR" state list -state="$STATE_DIR/terraform.tfstate" 2>"$REPAIR_ATTEMPT/control/repair-state-after.err") || die 'repair cannot prove empty Terraform state'
  [[ -z "$state_output" ]] || die 'repair Terraform state is not empty'
  cluster_is_absent "$REPAIR_ATTEMPT/control/repair-clusters.json" "$REPAIR_ATTEMPT/control/repair-clusters.err" || die 'repair cannot prove cluster absence'
  trap - EXIT INT TERM
  printf '%s\n' 'standalone GKE repair: destroy verified; private state retained'
}

if [[ "${BASH_SOURCE[0]}" == "$0" ]]; then main_repair "$@"; fi
