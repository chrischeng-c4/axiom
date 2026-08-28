#!/usr/bin/env bash
# shellcheck disable=SC1090,SC1091,SC2016,SC2034,SC2312
set -euo pipefail
set +x
umask 077
CDPATH=

MODULE_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd -P)"
REPO_ROOT="$(cd "$MODULE_DIR/../.." && pwd -P)"
LIVE="$MODULE_DIR/scripts/live-acceptance.sh"
REPAIR="$MODULE_DIR/scripts/repair-destroy.sh"
SYSTEM_PATH=$PATH

bash -n "$LIVE"
bash -n "$REPAIR"
shellcheck "$LIVE" "$REPAIR"

# shellcheck source=/dev/null
source "$LIVE"
LIVE_ROOTS_BEFORE=$(find "$PRIVATE_TMP_ROOT" -maxdepth 1 -type d -name 'lumen-standalone-gke-live.??????' -print | LC_ALL=C sort)
if command -v sha256sum >/dev/null 2>&1; then
  CONTRACT_DIGEST=$(printf '%s' contract-run | sha256sum | awk '{print $1}')
else
  CONTRACT_DIGEST=$(printf '%s' contract-run | shasum -a 256 | awk '{print $1}')
fi
HASH=${CONTRACT_DIGEST:0:10}
[[ "$HASH" == 1ab6cf668e ]] || { printf '%s\n' 'live lifecycle contract: independent identity oracle changed' >&2; exit 1; }
CLUSTER_ID="lumen-sa-$HASH"
POOL_ID="lumen-np-$HASH"
NODE_SA_ID="lumen-nodes-$HASH"
OWNER_ID="lumen-standalone-$HASH"

TMP=$(mktemp -d "$PRIVATE_TMP_ROOT/lumen-live-lifecycle-contract.XXXXXX")
FIXTURE="$TMP/repo"
STATE="$FIXTURE/test-state"
CANDIDATE="$TMP/candidate"
CLI="$TMP/lumen"
RETAINED_ROOTS=''

cleanup() {
  local root
  trap - EXIT
  while IFS= read -r root; do
    [[ -z "$root" ]] && continue
    if [[ "$root" == "$PRIVATE_TMP_ROOT/lumen-standalone-gke-live."* && -d "$root" && ! -L "$root" ]]; then rm -rf -- "$root"; fi
  done <<<"$RETAINED_ROOTS"
  if [[ "$TMP" == "$PRIVATE_TMP_ROOT/lumen-live-lifecycle-contract."* && -d "$TMP" && ! -L "$TMP" ]]; then rm -rf -- "$TMP"; fi
}
trap cleanup EXIT

fail() { printf 'live lifecycle contract: %s\n' "$*" >&2; exit 1; }
expect_reject() {
  local label=$1
  shift
  if ( "$@" ) >/dev/null 2>"$TMP/reject.err"; then fail "accepted forbidden case: $label"; fi
  [[ -s "$TMP/reject.err" ]] || fail "rejection had no evidence: $label"
}
expect_predicate_reject() {
  local label=$1
  shift
  if ( "$@" ); then fail "predicate accepted forbidden mutation: $label"; fi
}

mkdir -p "$TMP/stat-fixture" "$TMP/path-real"
chmod 700 "$TMP/path-real"
printf '%s\n' sentinel >"$TMP/path-real/file"
cat >"$TMP/stat-fixture/stat-polluted" <<'EOF'
#!/usr/bin/env bash
if [[ "$1" == -c ]]; then printf '%s\n' polluted; exit 1; fi
printf '%s\n' 700
EOF
cat >"$TMP/stat-fixture/stat-gnu" <<'EOF'
#!/usr/bin/env bash
if [[ "$1" == -c && "$2" == %a && "$#" -eq 3 ]]; then printf '%s\n' 700; exit 0; fi
if [[ "$1" == -f && "$2" == %Lp && "$#" -eq 3 ]]; then printf '%s\n' fake-filesystem-data; exit 1; fi
exit 1
EOF
cat >"$TMP/stat-fixture/stat-bsd" <<'EOF'
#!/usr/bin/env bash
if [[ "$1" == -c ]]; then exit 1; fi
printf '%s\n' 700
EOF
cat >"$TMP/stat-fixture/stat-bad" <<'EOF'
#!/usr/bin/env bash
if [[ "$1" == -c ]]; then printf '700\n701\n'; exit 0; fi
EOF
chmod 755 "$TMP/stat-fixture/stat-gnu" "$TMP/stat-fixture/stat-polluted" "$TMP/stat-fixture/stat-bsd" "$TMP/stat-fixture/stat-bad"
ln -s stat-polluted "$TMP/stat-fixture/stat"
if PATH="$TMP/stat-fixture:$SYSTEM_PATH" private_mode "$TMP/path-real"; then fail 'GNU stat pollution was accepted'; fi
rm -f "$TMP/stat-fixture/stat"; ln -s stat-gnu "$TMP/stat-fixture/stat"
[[ "$(PATH="$TMP/stat-fixture:$SYSTEM_PATH" private_mode "$TMP/path-real")" == 700 ]] || fail 'GNU stat oracle failed'
rm -f "$TMP/stat-fixture/stat"; ln -s stat-bsd "$TMP/stat-fixture/stat"
[[ "$(PATH="$TMP/stat-fixture:$SYSTEM_PATH" private_mode "$TMP/path-real")" == 700 ]] || fail 'BSD stat fallback oracle failed'
rm -f "$TMP/stat-fixture/stat"; ln -s stat-bad "$TMP/stat-fixture/stat"
if PATH="$TMP/stat-fixture:$SYSTEM_PATH" private_mode "$TMP/path-real"; then fail 'multi-line stat was accepted'; fi

# Private path helpers must reject another root, dot segments, and symlinked
# parents before a lifecycle can create or remove any state.
mkdir -p "$TMP/path-real"
ln -s "$TMP/path-real" "$TMP/path-link"
expect_predicate_reject private-other-root safe_private_dir /
expect_predicate_reject private-dot safe_private_dir "$TMP/./path-real"
expect_predicate_reject private-symlink-dir safe_private_dir "$TMP/path-link"
expect_predicate_reject private-symlink-file safe_private_file "$TMP/path-link/file"
expect_predicate_reject private-new-symlink-parent safe_new_private_path "$TMP/path-link/new"
ln -s "$TMP/path-real/file" "$TMP/path-link-leaf"
expect_predicate_reject private-symlink-leaf safe_private_file "$TMP/path-link-leaf"

mkdir -p "$FIXTURE/terraform/lumen-standalone-gke/scripts" \
  "$FIXTURE/kustomize/lumen-standalone-acceptance/tests" \
  "$FIXTURE/apps/lumen/scripts" "$FIXTURE/bin" "$STATE" "$CANDIDATE"
cp "$LIVE" "$FIXTURE/terraform/lumen-standalone-gke/scripts/live-acceptance.sh"
cp "$REPAIR" "$FIXTURE/terraform/lumen-standalone-gke/scripts/repair-destroy.sh"
chmod 755 "$FIXTURE/terraform/lumen-standalone-gke/scripts/live-acceptance.sh" "$FIXTURE/terraform/lumen-standalone-gke/scripts/repair-destroy.sh"
printf '%s\n' "$CLUSTER_ID" >"$STATE/cluster"
printf '%s\n' "$POOL_ID" >"$STATE/pool"
printf '%s\n' "$NODE_SA_ID" >"$STATE/node-sa"
printf '%s\n' "$OWNER_ID" >"$STATE/owner"
printf '%s\n' absent >"$STATE/resource-state"
: >"$STATE/events"
printf '%s\n' '{}' >"$CANDIDATE/final-candidate-manifest.json"
manifest_hash=$(sha256_file "$CANDIDATE/final-candidate-manifest.json")
printf '%s  final-candidate-manifest.json\n' "$manifest_hash" >"$CANDIDATE/final-candidate-manifest.json.sha256"
printf '%s\n' '#!/usr/bin/env bash' 'exit 0' >"$CLI"
chmod 755 "$CLI"
printf '%s\n' "$FIXTURE/bin:$SYSTEM_PATH" >"$STATE/expected-path"
printf '%s\n' "$CANDIDATE" >"$STATE/expected-candidate"
printf '%s\n' "$CLI" >"$STATE/expected-cli"

cat >"$FIXTURE/terraform/lumen-standalone-gke/scripts/check.sh" <<'EOF'
#!/usr/bin/env bash
set -euo pipefail
repo=$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd -P)
printf '%s\n' check >>"$repo/test-state/events"
[[ "$(<"$repo/test-state/mode")" != check-fail ]]
EOF

cat >"$FIXTURE/kustomize/lumen-standalone-acceptance/tests/contract.sh" <<'EOF'
#!/usr/bin/env bash
set -euo pipefail
repo=$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd -P)
printf '%s\n' kustomize >>"$repo/test-state/events"
[[ "$(<"$repo/test-state/mode")" != kustomize-fail ]]
EOF

cat >"$FIXTURE/apps/lumen/scripts/standalone-gke-acceptance.sh" <<'EOF'
#!/usr/bin/env bash
set -euo pipefail
repo=$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd -P)
state="$repo/test-state"
mode=$(<"$state/mode"); private_tmp_root=$(cd -P /tmp && pwd -P)
printf '%s\n' inner >>"$state/events"
[[ "$#" -eq 2 && "$1" == --mode && "$2" == gke ]]
while IFS='=' read -r name _; do case "$name" in *TOKEN*|*AUTHORIZATION*) exit 81 ;; esac; done < <(env)
actual_env=$(env | cut -d= -f1 | grep -Ev '^(PWD|SHLVL|_)$' | LC_ALL=C sort)
expected_env=$(printf '%s\n' KUBECONFIG LUMEN_STANDALONE_GKE_CANDIDATE_RECEIPT_DIR LUMEN_STANDALONE_GKE_CLI LUMEN_STANDALONE_GKE_CLIENT_IMAGE LUMEN_STANDALONE_GKE_CLI_TARGET LUMEN_STANDALONE_GKE_CLUSTER LUMEN_STANDALONE_GKE_CONTEXT LUMEN_STANDALONE_GKE_EVIDENCE_DIR LUMEN_STANDALONE_GKE_EXPECTED_COMMIT LUMEN_STANDALONE_GKE_EXPECTED_MANIFEST_SHA256 LUMEN_STANDALONE_GKE_EXPECTED_RUN_ATTEMPT LUMEN_STANDALONE_GKE_EXPECTED_RUN_ID LUMEN_STANDALONE_GKE_IMAGE LUMEN_STANDALONE_GKE_LOCATION LUMEN_STANDALONE_GKE_MUTATION LUMEN_STANDALONE_GKE_NODE_POOL LUMEN_STANDALONE_GKE_PROJECT_ID LUMEN_STANDALONE_GKE_RUN_ID LUMEN_STANDALONE_GKE_STORAGE_CLASS PATH | LC_ALL=C sort)
[[ "$actual_env" == "$expected_env" ]] || exit 80
for name in KUBECONFIG LUMEN_STANDALONE_GKE_CONTEXT LUMEN_STANDALONE_GKE_PROJECT_ID LUMEN_STANDALONE_GKE_LOCATION LUMEN_STANDALONE_GKE_CLUSTER LUMEN_STANDALONE_GKE_CLI LUMEN_STANDALONE_GKE_IMAGE LUMEN_STANDALONE_GKE_CLIENT_IMAGE LUMEN_STANDALONE_GKE_CLI_TARGET LUMEN_STANDALONE_GKE_STORAGE_CLASS LUMEN_STANDALONE_GKE_NODE_POOL LUMEN_STANDALONE_GKE_RUN_ID LUMEN_STANDALONE_GKE_EXPECTED_COMMIT LUMEN_STANDALONE_GKE_EXPECTED_RUN_ID LUMEN_STANDALONE_GKE_EXPECTED_RUN_ATTEMPT LUMEN_STANDALONE_GKE_EXPECTED_MANIFEST_SHA256 LUMEN_STANDALONE_GKE_EVIDENCE_DIR LUMEN_STANDALONE_GKE_MUTATION LUMEN_STANDALONE_GKE_CANDIDATE_RECEIPT_DIR; do [[ -n "${!name:-}" ]] || exit 82; done
[[ "$LUMEN_STANDALONE_GKE_CLUSTER" == "$(<"$state/cluster")" ]]
[[ "$LUMEN_STANDALONE_GKE_NODE_POOL" == "$(<"$state/pool")" ]]
[[ "$PATH" == "$(<"$state/expected-path")" ]]
[[ "$KUBECONFIG" == "$private_tmp_root"/lumen-standalone-gke-live.*/kubeconfig && "${KUBECONFIG%/*}" =~ ^.*lumen-standalone-gke-live\.[A-Za-z0-9]{6}$ ]]
[[ "$LUMEN_STANDALONE_GKE_EVIDENCE_DIR" == "${KUBECONFIG%/kubeconfig}/private-receipt" ]]
[[ "$LUMEN_STANDALONE_GKE_CONTEXT" == contract-context ]]
[[ "$LUMEN_STANDALONE_GKE_PROJECT_ID" == abcde1 && "$LUMEN_STANDALONE_GKE_LOCATION" == us-central1-a ]]
[[ "$LUMEN_STANDALONE_GKE_CLI" == "$(<"$state/expected-cli")" ]]
[[ "$LUMEN_STANDALONE_GKE_IMAGE" == "ghcr.io/chrischeng-c4/lumen@sha256:$(printf '%064d' 0)" ]]
[[ "$LUMEN_STANDALONE_GKE_CLI_TARGET" == aarch64-apple-darwin ]]
[[ "$LUMEN_STANDALONE_GKE_RUN_ID" == contract-run ]]
[[ "$LUMEN_STANDALONE_GKE_EXPECTED_RUN_ID" == 123 && "$LUMEN_STANDALONE_GKE_EXPECTED_RUN_ATTEMPT" == 2 ]]
[[ "$LUMEN_STANDALONE_GKE_EXPECTED_COMMIT" == "$(printf '%040d' 0)" ]]
[[ "$LUMEN_STANDALONE_GKE_EXPECTED_MANIFEST_SHA256" == "$(printf '%064d' 0)" ]]
[[ "$LUMEN_STANDALONE_GKE_STORAGE_CLASS" == premium-rwo && "$LUMEN_STANDALONE_GKE_MUTATION" == 1 ]]
[[ "$LUMEN_STANDALONE_GKE_CLIENT_IMAGE" == docker.io/curlimages/curl@sha256:7c12af72ceb38b7432ab85e1a265cff6ae58e06f95539d539b654f2cfa64bb13 ]]
[[ "$LUMEN_STANDALONE_GKE_CANDIDATE_RECEIPT_DIR" == "$(<"$state/expected-candidate")" ]]
[[ "$mode" != inner-fail ]] || exit 83
receipt="$LUMEN_STANDALONE_GKE_EVIDENCE_DIR/lumen-standalone-gke-receipt.json"
printf '%s\n' '{"schema":"fake-live-receipt"}' >"$receipt"
if command -v sha256sum >/dev/null 2>&1; then hash=$(sha256sum "$receipt" | awk '{print $1}'); else hash=$(shasum -a 256 "$receipt" | awk '{print $1}'); fi
if [[ "$mode" == inner-sidecar-bad ]]; then hash=$(printf '%064d' 0); fi
printf '%s  lumen-standalone-gke-receipt.json\n' "$hash" >"$receipt.sha256"
EOF

cat >"$FIXTURE/apps/lumen/scripts/verify-release-artifacts.sh" <<'EOF'
#!/usr/bin/env bash
set -euo pipefail
fake_repo=$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd -P)
fake_state="$fake_repo/test-state"
fake_sha() { if command -v sha256sum >/dev/null 2>&1; then sha256sum "$1" | awk '{print $1}'; else shasum -a 256 "$1" | awk '{print $1}'; fi; }
validate_receipt() {
  printf '%s\n' verifier:receipt >>"$fake_state/events"
  [[ -z "${CANDIDATE_ATTEMPT:-}" && "$1" == "$CANDIDATE_RECEIPT_DIR/final-candidate-manifest.json" && "$2" == "$CANDIDATE_RECEIPT_DIR/final-candidate-manifest.json.sha256" && "$3" == "$CANDIDATE_RECEIPT_DIR" ]]
}
validate_standalone_gke_receipt() {
  local expected name extra actual
  printf '%s\n' verifier:gke >>"$fake_state/events"
  [[ "$CANDIDATE_ATTEMPT" == 2 && "$COMMIT" == 0000000000000000000000000000000000000000 && "$CANDIDATE_RUN_ID" == 123 ]]
  read -r expected name extra <"$STANDALONE_GKE_RECEIPT_SIDECAR"
  actual=$(fake_sha "$STANDALONE_GKE_RECEIPT")
  [[ "$expected" == "$actual" && "$name" == lumen-standalone-gke-receipt.json && -z "${extra:-}" ]]
  [[ "$(<"$fake_state/mode")" != verify-fail ]]
}
return 0
EOF

cat >"$FIXTURE/bin/terraform" <<'EOF'
#!/usr/bin/env bash
set -euo pipefail
repo=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd -P)
state="$repo/test-state"; mode=$(<"$state/mode"); private_tmp_root=$(cd -P /tmp && pwd -P)
[[ "${1:-}" == "-chdir=$repo/terraform/lumen-standalone-gke" ]] || exit 89
shift
cmd=${1:-}; [[ -n "$cmd" ]] || exit 90; shift
event() { printf '%s\n' "$1" >>"$state/events"; }
plan_json() {
  local action=$1 kind=$2
  if [[ "$kind" == missing ]]; then jq -nc --arg a "$action" '{resource_changes:["google_container_cluster.standalone","google_container_node_pool.standalone","google_project_iam_member.node_baseline"]|map({address:.,change:{actions:[$a]}})}'
  elif [[ "$kind" == subset ]]; then jq -nc --arg a "$action" '{resource_changes:["google_service_account.nodes"]|map({address:.,change:{actions:[$a]}})}'
  elif [[ "$kind" == extra ]]; then jq -nc --arg a "$action" '{resource_changes:["google_container_cluster.standalone","google_container_node_pool.standalone","google_project_iam_member.node_baseline","google_service_account.nodes","google_compute_network.extra"]|map({address:.,change:{actions:[$a]}})}'
  elif [[ "$kind" == unknown ]]; then jq -nc --arg a "$action" '{resource_changes:["google_container_cluster.standalone","google_container_node_pool.standalone","google_project_iam_member.node_baseline","google_compute_network.unknown"]|map({address:.,change:{actions:[$a]}})}'
  elif [[ "$kind" == wrong-action ]]; then jq -nc '{resource_changes:["google_container_cluster.standalone","google_container_node_pool.standalone","google_project_iam_member.node_baseline","google_service_account.nodes"]|map({address:.,change:{actions:["update"]}})}'
  else jq -nc --arg a "$action" '{resource_changes:["google_container_cluster.standalone","google_container_node_pool.standalone","google_project_iam_member.node_baseline","google_service_account.nodes"]|map({address:.,change:{actions:[$a]}})}'; fi
}
case "$cmd" in
  init)
    [[ "$#" -eq 4 && "$1" == -backend=false && "$2" == -input=false && "$3" == -lockfile=readonly && "$4" == -no-color ]] || exit 95
    event terraform:init; [[ "$mode" != init-fail ]]
    ;;
  plan)
    destroy=0
    if [[ "${1:-}" == -destroy ]]; then destroy=1; shift; fi
    [[ "$#" -eq 9 && "$1" == -input=false && "$2" == -no-color && "$3" == -state=* && "$4" == -out=* ]] || exit 96
    state_file=${3#-state=}; out=${4#-out=}; run_root=${state_file%/terraform.tfstate}
    [[ "$state_file" == "$private_tmp_root"/lumen-standalone-gke-live.??????/terraform.tfstate && "${out%/*}" == "$run_root" ]] || exit 97
    [[ "$5" == '-var=project_id=abcde1' && "$6" == '-var=region=us-central1' && "$7" == '-var=gke_zone=us-central1-a' && "$8" == '-var=run_id=contract-run' && "$9" == '-var=storage_class_name=premium-rwo' ]] || exit 91
    if [[ "$destroy" -eq 0 ]]; then [[ "${out##*/}" == create.tfplan ]] || exit 98
    else case "${out##*/}" in destroy.tfplan|recovery-destroy.tfplan|repair-destroy.tfplan) ;; *) exit 98 ;; esac; fi
    if [[ "$destroy" -eq 1 ]]; then event terraform:plan:destroy; printf '%s\n' destroy >"$out"; else event terraform:plan:create; printf '%s\n' create >"$out"; fi
    ;;
  show)
    [[ "$#" -eq 2 && "$1" == -json ]] || exit 99
    shift
    file=$1; marker=$(<"$file")
    if [[ "$marker" == create ]]; then
      event terraform:show:create; kind=valid
      [[ "$mode" == create-plan-missing ]] && kind=missing
      [[ "$mode" == create-plan-extra ]] && kind=extra
      [[ "$mode" == create-plan-unknown ]] && kind=unknown
      [[ "$mode" == create-plan-action ]] && kind=wrong-action
      plan_json create "$kind"
    elif [[ "$marker" == destroy ]]; then
      event terraform:show:destroy; kind=valid
      if [[ "$mode" == destroy-plan-bad && "$file" == */destroy.tfplan ]]; then kind=extra; fi
      [[ "$mode" == repair-plan-action ]] && kind=wrong-action
      [[ "$mode" == repair-plan-unknown ]] && kind=unknown
      [[ "$mode" == repair-plan-five ]] && kind=extra
      [[ "$mode" == repair-plan-subset ]] && kind=subset
      if [[ "$kind" == valid && "$(<"$state/resource-state")" == destroyed ]]; then printf '%s\n' '{"resource_changes":[]}'; else plan_json delete "$kind"; fi
    else
      event terraform:show:state
      cluster=$(<"$state/cluster"); pool=$(<"$state/pool"); node=$(<"$state/node-sa"); storage=premium-rwo
      [[ "$mode" == output-bad ]] && storage=standard-rwo
      jq -nc --arg c "$cluster" --arg p "$pool" --arg n "$node" --arg storage "$storage" '{values:{outputs:{project_id:{value:"abcde1"},region:{value:"us-central1"},gke_zone:{value:"us-central1-a"},cluster_name:{value:$c},node_pool_name:{value:$p},node_selector:{value:{"cloud.google.com/gke-nodepool":$p}},storage_class_name:{value:$storage},workload_identity_pool:{value:"abcde1.svc.id.goog"},node_service_account_email:{value:($n+"@abcde1.iam.gserviceaccount.com")},run_id:{value:"contract-run"}}}}'
    fi
    ;;
  apply)
    [[ "$#" -eq 5 && "$1" == -input=false && "$2" == -no-color && "$3" == -state=* && "$4" == -backup=- ]] || exit 100
    state_file=${3#-state=}; file=$5; run_root=${state_file%/terraform.tfstate}
    [[ "$state_file" == "$private_tmp_root"/lumen-standalone-gke-live.??????/terraform.tfstate && "${file%/*}" == "$run_root" ]] || exit 101
    case "${file##*/}" in create.tfplan|destroy.tfplan|recovery-destroy.tfplan|repair-destroy.tfplan) ;; *) exit 102 ;; esac
    marker=$(<"$file")
    if [[ "$marker" == create ]]; then
      event terraform:apply:create; printf '%s\n' partial >"$state/resource-state"
      [[ "$mode" != create-apply-fail ]] || exit 92
      printf '%s\n' created >"$state/resource-state"
    else
      event terraform:apply:destroy
      [[ "$mode" != destroy-apply-fail ]] || exit 93
      printf '%s\n' destroyed >"$state/resource-state"
      if [[ "$mode" == receipt-after-verify-tamper ]]; then [[ -n "$state_file" ]]; printf ' ' >>"${state_file%/*}/private-receipt/lumen-standalone-gke-receipt.json"; fi
    fi
    ;;
  state)
    [[ "$#" -eq 2 && "$1" == list && "$2" == -state=* ]] || exit 103
    state_file=${2#-state=}
    [[ "$state_file" == "$private_tmp_root"/lumen-standalone-gke-live.??????/terraform.tfstate ]] || exit 104
    event terraform:state-list
    resource=$(<"$state/resource-state")
    if [[ "$resource" == created ]]; then
      if [[ "$mode" == state-bad ]]; then printf '%s\n' google_container_cluster.standalone google_container_node_pool.standalone google_service_account.nodes
      else printf '%s\n' google_container_cluster.standalone google_container_node_pool.standalone google_project_iam_member.node_baseline google_service_account.nodes; fi
    elif [[ "$resource" == partial || ( "$resource" == destroyed && "$mode" == repair-state-nonempty ) ]]; then printf '%s\n' google_service_account.nodes; fi
    ;;
  *) exit 94 ;;
esac
EOF

cat >"$FIXTURE/bin/gcloud" <<'EOF'
#!/usr/bin/env bash
set -euo pipefail
repo=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd -P); state="$repo/test-state"; mode=$(<"$state/mode"); cluster=$(<"$state/cluster"); pool=$(<"$state/pool"); node=$(<"$state/node-sa"); owner=$(<"$state/owner")
event() { printf '%s\n' "$1" >>"$state/events"; }
if [[ "$#" -eq 6 && "$1" == container && "$2" == clusters && "$3" == list && "$4" == --project=abcde1 && "$5" == --location=us-central1-a && "$6" == --format=json ]]; then
  event gcloud:list; resource=$(<"$state/resource-state")
  [[ "$mode" != pre-list-fail ]] || exit 71
  [[ ! ( "$mode" == post-list-fail && "$resource" == destroyed ) ]] || exit 72
  if [[ "$mode" == cluster-exists || ( "$mode" == post-list-present && "$resource" == destroyed ) ]]; then jq -nc --arg n "$cluster" '[{name:$n}]'; else printf '%s\n' '[]'; fi
elif [[ "$#" -eq 7 && "$1" == container && "$2" == clusters && "$3" == describe && "$4" == "$cluster" && "$5" == --project=abcde1 && "$6" == --location=us-central1-a && "$7" == --format=json ]]; then
  event gcloud:cluster-describe; datapath=ADVANCED_DATAPATH; [[ "$mode" == cluster-bad ]] && datapath=LEGACY_DATAPATH
  jq -nc --arg c "$cluster" --arg d "$datapath" --arg o "$owner" '{name:$c,location:"us-central1-a",status:"RUNNING",autopilot:{},endpoint:"203.0.113.8",ipAllocationPolicy:{useIpAliases:true},networkConfig:{datapathProvider:$d},releaseChannel:{channel:"REGULAR"},workloadIdentityConfig:{workloadPool:"abcde1.svc.id.goog"},addonsConfig:{gcePersistentDiskCsiDriverConfig:{enabled:true}},loggingConfig:{componentConfig:{enableComponents:["SYSTEM_COMPONENTS","WORKLOADS"]}},resourceLabels:{"lumen-owner":$o,"goog-terraform-provisioned":"true"}}'
elif [[ "$#" -eq 7 && "$1" == container && "$2" == node-pools && "$3" == list && "$4" == --cluster="$cluster" && "$5" == --project=abcde1 && "$6" == --location=us-central1-a && "$7" == --format=json ]]; then
  event gcloud:node-pools-list; if [[ "$mode" == node-list-bad ]]; then jq -nc --arg p "$pool" '[{name:$p},{name:"extra"}]'; else jq -nc --arg p "$pool" '[{name:$p}]'; fi
elif [[ "$#" -eq 8 && "$1" == container && "$2" == node-pools && "$3" == describe && "$4" == "$pool" && "$5" == --cluster="$cluster" && "$6" == --project=abcde1 && "$7" == --location=us-central1-a && "$8" == --format=json ]]; then
  event gcloud:node-pool-describe; machine=e2-standard-2; [[ "$mode" == node-bad ]] && machine=e2-small
  jq -nc --arg p "$pool" --arg n "$node" --arg o "$owner" --arg m "$machine" '{name:$p,status:"RUNNING",initialNodeCount:1,autoscaling:{enabled:true,minNodeCount:1,maxNodeCount:3},config:{machineType:$m,serviceAccount:($n+"@abcde1.iam.gserviceaccount.com"),oauthScopes:["https://www.googleapis.com/auth/cloud-platform"],metadata:{"disable-legacy-endpoints":"true"},labels:{"lumen-owner":$o},taints:[],workloadMetadataConfig:{mode:"GKE_METADATA"}}}'
elif [[ "$#" -eq 6 && "$1" == container && "$2" == clusters && "$3" == get-credentials && "$4" == "$cluster" && "$5" == --project=abcde1 && "$6" == --location=us-central1-a ]]; then event gcloud:get-credentials
else exit 73; fi
EOF

cat >"$FIXTURE/bin/kubectl" <<'EOF'
#!/usr/bin/env bash
set -euo pipefail
repo=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd -P); state="$repo/test-state"; mode=$(<"$state/mode"); private_tmp_root=$(cd -P /tmp && pwd -P)
  if [[ "$#" -eq 7 && "$1" == --kubeconfig && "$2" == "$private_tmp_root"/lumen-standalone-gke-live.??????/kubeconfig && "$3" == config && "$4" == view && "$5" == --raw && "$6" == -o && "$7" == json ]]; then
  printf '%s\n' "$2" >"$state/kubeconfig-path"
  printf '%s\n' kubectl:config >>"$state/events"; server=https://203.0.113.8; [[ "$mode" == kube-bad ]] && server=https://wrong.invalid
  jq -nc --arg s "$server" '{"current-context":"contract-context",contexts:[{name:"contract-context",context:{cluster:"contract-cluster",user:"contract-user"}}],clusters:[{name:"contract-cluster",cluster:{server:$s}}],users:[{name:"contract-user",user:{exec:{command:"gke-gcloud-auth-plugin"}}}]}'
elif [[ "$#" -eq 9 && "$1" == --kubeconfig && "$2" == "$(<"$state/kubeconfig-path")" && "$3" == --context && "$4" == contract-context && "$5" == get && "$6" == storageclass && "$7" == premium-rwo && "$8" == -o && "$9" == json ]]; then
  printf '%s\n' kubectl:storage >>"$state/events"; provisioner=pd.csi.storage.gke.io; [[ "$mode" == storage-bad ]] && provisioner=kubernetes.io/gce-pd
  jq -nc --arg p "$provisioner" '{metadata:{name:"premium-rwo"},provisioner:$p,parameters:{type:"pd-ssd"},reclaimPolicy:"Delete",volumeBindingMode:"WaitForFirstConsumer",allowVolumeExpansion:true}'
else exit 74; fi
EOF

cat >"$FIXTURE/bin/cp" <<'EOF'
#!/usr/bin/env bash
set -euo pipefail
/bin/cp "$@"
repo=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd -P)
if [[ "$(<"$repo/test-state/mode")" == publication-extra ]]; then destination=''; for arg in "$@"; do destination=$arg; done; : >"${destination%/*}/extra"; fi
EOF

chmod 755 "$FIXTURE/terraform/lumen-standalone-gke/scripts/check.sh" "$FIXTURE/kustomize/lumen-standalone-acceptance/tests/contract.sh" "$FIXTURE/apps/lumen/scripts/standalone-gke-acceptance.sh" "$FIXTURE/apps/lumen/scripts/verify-release-artifacts.sh" "$FIXTURE/bin/terraform" "$FIXTURE/bin/gcloud" "$FIXTURE/bin/kubectl" "$FIXTURE/bin/cp"

LIVE_FIXTURE="$FIXTURE/terraform/lumen-standalone-gke/scripts/live-acceptance.sh"
REPAIR_FIXTURE="$FIXTURE/terraform/lumen-standalone-gke/scripts/repair-destroy.sh"
COMMON_ARGS=(--project-id abcde1 --region us-central1 --gke-zone us-central1-a --run-id contract-run --candidate-receipt-dir "$CANDIDATE" --lumen-cli "$CLI" --cli-target aarch64-apple-darwin --image "ghcr.io/chrischeng-c4/lumen@sha256:$(printf '%064d' 0)" --expected-commit "$(printf '%040d' 0)" --expected-run-id 123 --expected-run-attempt 2 --expected-manifest-sha256 "$(printf '%064d' 0)")

reset_case() {
  local name=$1 mode=$2
  CASE_OUT="$TMP/out-$name"; CASE_STDOUT="$TMP/$name.stdout"; CASE_STDERR="$TMP/$name.stderr"
  rm -rf -- "$CASE_OUT"
  : >"$STATE/events"
  printf '%s\n' absent >"$STATE/resource-state"
  printf '%s\n' "$mode" >"$STATE/mode"
}

run_live() {
  PATH="$FIXTURE/bin:$SYSTEM_PATH" "$LIVE_FIXTURE" "${COMMON_ARGS[@]}" --receipt-out-dir "$CASE_OUT" --confirm-create "$CLUSTER_ID" --confirm-destroy "$CLUSTER_ID" >"$CASE_STDOUT" 2>"$CASE_STDERR"
}

expect_live_reject() {
  local label=$1
  if run_live; then fail "accepted forbidden live case: $label"; fi
  [[ -s "$CASE_STDERR" ]] || fail "live rejection had no evidence: $label"
}

assert_no_new_live_roots() {
  local label=$1 root
  for root in "$PRIVATE_TMP_ROOT"/lumen-standalone-gke-live.*; do
    [[ -d "$root" ]] || continue
    if ! grep -Fqx "$root" <<<"$LIVE_ROOTS_BEFORE" && ! grep -Fqx "$root" <<<"$RETAINED_ROOTS"; then
      fail "$label left an unexpected private root"
    fi
  done
}

remember_repair_root() {
  local root
  root=$(sed -n 's|^terraform/lumen-standalone-gke/scripts/repair-destroy.sh --state-dir \([^ ]*\) --confirm-destroy .*$|\1|p' "$CASE_STDERR" | tail -n 1)
  [[ "$root" == "$PRIVATE_TMP_ROOT/lumen-standalone-gke-live."* && "${root##*/}" =~ ^lumen-standalone-gke-live\.[A-Za-z0-9]{6}$ && -d "$root" ]] || fail 'missing exact repair command and retained root'
  printf '%s' "$root"
}

reset_case happy happy
run_live || fail 'happy lifecycle failed'
[[ -d "$CASE_OUT" && ! -L "$CASE_OUT" ]] || fail 'happy receipt output is missing'
actual_inventory=$(find "$CASE_OUT" -mindepth 1 -maxdepth 1 -type f -exec basename {} \; | LC_ALL=C sort)
[[ "$actual_inventory" == $'lumen-standalone-gke-receipt.json\nlumen-standalone-gke-receipt.json.sha256' ]] || fail 'happy publication inventory changed'
happy_hash=$(sha256_file "$CASE_OUT/lumen-standalone-gke-receipt.json")
[[ "$(<"$CASE_STDOUT")" == "$happy_hash"$'\n'"$CASE_OUT" ]] || fail 'happy stdout is not exactly hash and path'
expected_events=$'check\nkustomize\ngcloud:list\nterraform:init\nterraform:plan:create\nterraform:show:create\nterraform:apply:create\nterraform:state-list\nterraform:show:state\ngcloud:cluster-describe\ngcloud:node-pools-list\ngcloud:node-pool-describe\ngcloud:get-credentials\nkubectl:config\nkubectl:storage\ninner\nverifier:receipt\nverifier:gke\nterraform:plan:destroy\nterraform:show:destroy\nterraform:apply:destroy\nterraform:state-list\ngcloud:list'
[[ "$(<"$STATE/events")" == "$expected_events" ]] || fail 'happy call order changed'

: >"$STATE/events"
expect_reject missing "$LIVE_FIXTURE" --project-id abcde1
expect_reject empty "$LIVE_FIXTURE" --project-id ''
expect_reject unknown "$LIVE_FIXTURE" --unknown value
expect_reject duplicate "$LIVE_FIXTURE" --project-id abcde1 --project-id abcde1
[[ ! -s "$STATE/events" ]] || fail 'CLI rejection reached a dependency'

for pair in 'check check-fail' 'kustomize kustomize-fail' 'init init-fail'; do
  read -r name mode <<<"$pair"; reset_case "$name" "$mode"; expect_live_reject "$name"
  [[ ! -e "$CASE_OUT" ]] || fail "$name published a receipt"
  case "$name" in
    check) expected=$'check' ;;
    kustomize) expected=$'check\nkustomize' ;;
    init) expected=$'check\nkustomize\ngcloud:list\nterraform:init' ;;
  esac
  [[ "$(<"$STATE/events")" == "$expected" ]] || fail "$name reached a forbidden dependency"
done

for pair in 'pre-list pre-list-fail' 'cluster-exists cluster-exists' 'plan-missing create-plan-missing' 'plan-extra create-plan-extra' 'plan-unknown create-plan-unknown' 'plan-action create-plan-action'; do
  read -r name mode <<<"$pair"; reset_case "$name" "$mode"; expect_live_reject "$name"
  [[ ! -e "$CASE_OUT" ]] || fail "$name published a receipt"
  if [[ "$name" == pre-list || "$name" == cluster-exists ]]; then ! grep -q 'terraform:apply:create' "$STATE/events" || fail "$name mutated Terraform"; else ! grep -q 'terraform:apply:create' "$STATE/events" || fail "$name applied a rejected plan"; fi
done

reset_case create-uncertain create-apply-fail
expect_live_reject create-uncertain
uncertain_root=$(remember_repair_root)
RETAINED_ROOTS="${RETAINED_ROOTS}${RETAINED_ROOTS:+$'\n'}$uncertain_root"
[[ ! -e "$CASE_OUT" ]] || fail 'uncertain create published a receipt'
! grep -q 'terraform:plan:destroy' "$STATE/events" || fail 'uncertain create guessed a destroy plan'

: >"$STATE/events"; printf '%s\n' repair-happy >"$STATE/mode"
PATH="$FIXTURE/bin:$SYSTEM_PATH" "$REPAIR_FIXTURE" --state-dir "$uncertain_root" --confirm-destroy "$CLUSTER_ID" >"$TMP/repair.stdout" 2>"$TMP/repair.stderr" || fail 'repair happy path failed'
[[ -d "$uncertain_root" && "$(<"$STATE/resource-state")" == destroyed ]] || fail 'repair did not retain empty private state'
[[ "$(<"$TMP/repair.stdout")" == 'standalone GKE repair: destroy verified; private state retained' ]] || fail 'repair stdout changed'
if ! grep -q 'terraform:plan:destroy' "$STATE/events" || ! grep -q 'terraform:apply:destroy' "$STATE/events"; then fail 'repair did not apply a saved destroy plan'; fi
[[ ! -e "$TMP/out-create-uncertain" ]] || fail 'repair wrote a public receipt'

for mode in state-bad output-bad cluster-bad node-list-bad node-bad kube-bad storage-bad inner-fail verify-fail inner-sidecar-bad receipt-after-verify-tamper; do
  reset_case "$mode" "$mode"; expect_live_reject "$mode"
  [[ ! -e "$CASE_OUT" ]] || fail "$mode published a receipt"
  ! grep -q '^terraform/lumen-standalone-gke/scripts/repair-destroy.sh ' "$CASE_STDERR" || fail "$mode unexpectedly retained repair state"
  ! grep -q '^private lifecycle state was retained$' "$CASE_STDERR" || fail "$mode unexpectedly retained private state"
  assert_no_new_live_roots "$mode"
  if ! grep -q 'terraform:plan:destroy' "$STATE/events" || ! grep -q 'terraform:apply:destroy' "$STATE/events"; then fail "$mode did not run failure cleanup"; fi
done

reset_case destroy-plan-bad destroy-plan-bad
expect_live_reject destroy-plan-bad
[[ ! -e "$CASE_OUT" ]] || fail 'bad destroy plan published a receipt'
[[ "$(grep -c 'terraform:plan:destroy' "$STATE/events")" -eq 2 ]] || fail 'bad destroy plan did not create an independent recovery plan'
grep -q 'terraform:apply:destroy' "$STATE/events" || fail 'recovery destroy plan was not applied'

for mode in destroy-apply-fail post-list-fail post-list-present; do
  reset_case "$mode" "$mode"; expect_live_reject "$mode"
  retained_root=$(remember_repair_root)
  RETAINED_ROOTS="${RETAINED_ROOTS}${RETAINED_ROOTS:+$'\n'}$retained_root"
  [[ ! -e "$CASE_OUT" ]] || fail "$mode published a receipt"
done

reset_case publication-extra publication-extra
expect_live_reject publication-extra
[[ ! -e "$CASE_OUT" ]] || fail 'unexpected staging inventory was published'

# Unit-level negative mutations cover every field in the live cluster, pool,
# kubeconfig, and StorageClass predicates. These helpers are the same bytes
# used by the executed lifecycle above.
PROJECT_ID=abcde1; REGION=us-central1; GKE_ZONE=us-central1-a; RUN_ID=contract-run; CLUSTER=$CLUSTER_ID; NODE_POOL=$POOL_ID; NODE_SERVICE_ACCOUNT=$NODE_SA_ID; OWNER_LABEL=$OWNER_ID
printf '%s\n' happy >"$STATE/mode"; printf '%s\n' created >"$STATE/resource-state"
UNIT_KUBE_ROOT=$(mktemp -d "$PRIVATE_TMP_ROOT/lumen-standalone-gke-live.XXXXXX")
RETAINED_ROOTS="${RETAINED_ROOTS}${RETAINED_ROOTS:+$'\n'}$UNIT_KUBE_ROOT"
PATH="$FIXTURE/bin:$SYSTEM_PATH" gcloud container clusters describe "$CLUSTER" --project=abcde1 --location=us-central1-a --format=json >"$TMP/valid-cluster.json"
PATH="$FIXTURE/bin:$SYSTEM_PATH" gcloud container node-pools describe "$NODE_POOL" --cluster="$CLUSTER" --project=abcde1 --location=us-central1-a --format=json >"$TMP/valid-pool.json"
PATH="$FIXTURE/bin:$SYSTEM_PATH" kubectl --kubeconfig "$UNIT_KUBE_ROOT/kubeconfig" config view --raw -o json >"$TMP/valid-kube.json"
PATH="$FIXTURE/bin:$SYSTEM_PATH" kubectl --kubeconfig "$UNIT_KUBE_ROOT/kubeconfig" --context contract-context get storageclass premium-rwo -o json >"$TMP/valid-storage.json"
assert_cluster "$TMP/valid-cluster.json" || fail 'valid cluster fixture rejected'
assert_node_pool "$TMP/valid-pool.json" || fail 'valid pool fixture rejected'
assert_kubeconfig "$TMP/valid-kube.json" 203.0.113.8 || fail 'valid kubeconfig fixture rejected'
assert_storage_class "$TMP/valid-storage.json" || fail 'valid StorageClass fixture rejected'

for filter in '.name="wrong"' '.location="us-central1-b"' '.status="ERROR"' '.autopilot.enabled=true' '.ipAllocationPolicy.useIpAliases=false' '.networkConfig.datapathProvider="LEGACY_DATAPATH"' '.releaseChannel.channel="RAPID"' '.workloadIdentityConfig.workloadPool="wrong"' '.addonsConfig.gcePersistentDiskCsiDriverConfig.enabled=false' '.loggingConfig.componentConfig.enableComponents=["SYSTEM_COMPONENTS"]' '.resourceLabels["lumen-owner"]="wrong"' '.resourceLabels["lumen-extra"]="bad"'; do jq "$filter" "$TMP/valid-cluster.json" >"$TMP/mutated.json"; expect_predicate_reject "cluster $filter" assert_cluster "$TMP/mutated.json"; done
for filter in '.name="wrong"' '.status="ERROR"' '.initialNodeCount=2' '.config.machineType="e2-small"' '.autoscaling.enabled=false' '.autoscaling.minNodeCount=0' '.autoscaling.maxNodeCount=4' '.config.taints=[{}]' '.config.workloadMetadataConfig.mode="GCE_METADATA"' '.config.oauthScopes=[]' '.config.metadata["disable-legacy-endpoints"]="false"' '.config.labels["lumen-owner"]="wrong"' '.config.labels["lumen-extra"]="bad"' '.config.serviceAccount="wrong"'; do jq "$filter" "$TMP/valid-pool.json" >"$TMP/mutated.json"; expect_predicate_reject "pool $filter" assert_node_pool "$TMP/mutated.json"; done
for filter in '.contexts=[]' '.clusters=[]' '.users=[]' '."current-context"="wrong"' '.contexts[0].context.cluster="wrong"' '.contexts[0].context.user="wrong"' '.clusters[0].cluster.server="https://wrong"'; do jq "$filter" "$TMP/valid-kube.json" >"$TMP/mutated.json"; expect_predicate_reject "kubeconfig $filter" assert_kubeconfig "$TMP/mutated.json" 203.0.113.8; done
for filter in '.metadata.name="wrong"' '.provisioner="wrong"' '.parameters.type="pd-balanced"' '.reclaimPolicy="Retain"' '.volumeBindingMode="Immediate"' '.allowVolumeExpansion=false'; do jq "$filter" "$TMP/valid-storage.json" >"$TMP/mutated.json"; expect_predicate_reject "storage $filter" assert_storage_class "$TMP/mutated.json"; done

jq -nc --arg c "$CLUSTER" --arg p "$NODE_POOL" --arg n "$NODE_SERVICE_ACCOUNT" '{values:{outputs:{project_id:{value:"abcde1"},region:{value:"us-central1"},gke_zone:{value:"us-central1-a"},cluster_name:{value:$c},node_pool_name:{value:$p},node_selector:{value:{"cloud.google.com/gke-nodepool":$p}},storage_class_name:{value:"premium-rwo"},workload_identity_pool:{value:"abcde1.svc.id.goog"},node_service_account_email:{value:($n+"@abcde1.iam.gserviceaccount.com")},run_id:{value:"contract-run"}}}}' >"$TMP/valid-outputs.json"
assert_outputs "$TMP/valid-outputs.json" || fail 'valid Terraform output fixture rejected'
for filter in '.values.outputs.extra={value:"bad"}' 'del(.values.outputs.project_id)' '.values.outputs.project_id.value="wrong"' '.values.outputs.region.value="wrong"' '.values.outputs.gke_zone.value="wrong"' '.values.outputs.run_id.value="wrong"' '.values.outputs.cluster_name.value="wrong"' '.values.outputs.node_pool_name.value="wrong"' '.values.outputs.node_selector.value={"cloud.google.com/gke-nodepool":"wrong"}' '.values.outputs.storage_class_name.value="wrong"' '.values.outputs.workload_identity_pool.value="wrong"' '.values.outputs.node_service_account_email.value="wrong"'; do jq "$filter" "$TMP/valid-outputs.json" >"$TMP/mutated.json"; expect_predicate_reject "outputs $filter" assert_outputs "$TMP/mutated.json"; done

# Repair refuses parser, contract, confirmation, and plan mutations without
# deleting the retained state directory or creating a receipt.
expect_reject repair-missing "$REPAIR_FIXTURE" --state-dir "$uncertain_root"
expect_reject repair-duplicate "$REPAIR_FIXTURE" --state-dir "$uncertain_root" --state-dir "$uncertain_root" --confirm-destroy "$CLUSTER_ID"
expect_reject repair-unknown "$REPAIR_FIXTURE" --unknown x
expect_reject repair-confirm "$REPAIR_FIXTURE" --state-dir "$uncertain_root" --confirm-destroy wrong
cp "$uncertain_root/run-contract.json" "$TMP/run-contract.good"
for entry in \
  'extra|.extra="bad"' \
  'missing|del(.project_id)' \
  'type|.expected_run_id=123' \
  'identity|.cluster_name="wrong"' \
  'storage|.storage_class_name="standard-rwo"' \
  'state-dir|.state_dir="/private/tmp/wrong"' \
  'confirmation|.confirm_create="wrong"'; do
  label=${entry%%|*}; filter=${entry#*|}
  jq "$filter" "$TMP/run-contract.good" >"$uncertain_root/run-contract.json"
  expect_reject "repair-contract-$label" "$REPAIR_FIXTURE" --state-dir "$uncertain_root" --confirm-destroy "$CLUSTER_ID"
done
cp "$TMP/run-contract.good" "$uncertain_root/run-contract.json"

# A nested direct-looking state directory must be rejected before contract
# loading or any Terraform command.
mkdir -p "$uncertain_root/nested/lumen-standalone-gke-live.ABC123"
cp "$uncertain_root/run-contract.json" "$uncertain_root/nested/lumen-standalone-gke-live.ABC123/run-contract.json"
: >"$STATE/events"
expect_reject repair-nested-state "$REPAIR_FIXTURE" --state-dir "$uncertain_root/nested/lumen-standalone-gke-live.ABC123" --confirm-destroy "$CLUSTER_ID"
[[ -z "$(<"$STATE/events")" ]] || fail 'nested repair state reached Terraform'
[[ -d "$uncertain_root" && -f "$uncertain_root/run-contract.json" ]] || fail 'nested repair rejected state lost evidence'

# Contract paths and repair inputs are untrusted evidence.  Each mutation is
# rejected before Terraform destroy, and the retained evidence must stay byte
# for byte unchanged.
repair_inventory() { find "$uncertain_root" -mindepth 1 -print | LC_ALL=C sort; }
repair_sentinel_hash() { sha256_file "$uncertain_root/terraform.tfstate"; }
assert_repair_unchanged() {
  local label=$1 before_inventory=$2 before_hash=$3
  [[ "$(repair_inventory)" == "$before_inventory" ]] || fail "$label changed retained inventory"
  [[ "$(repair_sentinel_hash)" == "$before_hash" ]] || fail "$label changed retained state"
  ! grep -q 'terraform:apply:destroy' "$STATE/events" || fail "$label applied destroy"
  [[ ! -e "$TMP/out-create-uncertain" ]] || fail "$label wrote a public receipt"
}
run_repair_contract_mutation() {
  local label=$1 filter=$2
  cp "$TMP/run-contract.good" "$uncertain_root/run-contract.json"
  jq "$filter" "$TMP/run-contract.good" >"$TMP/mutated-contract.json"
  cp "$TMP/mutated-contract.json" "$uncertain_root/run-contract.json"
  local before_inventory before_hash
  before_inventory=$(repair_inventory); before_hash=$(repair_sentinel_hash)
  : >"$STATE/events"
  expect_reject "repair-contract-$label" env PATH="$FIXTURE/bin:$SYSTEM_PATH" "$REPAIR_FIXTURE" --state-dir "$uncertain_root" --confirm-destroy "$CLUSTER_ID"
  assert_repair_unchanged "repair-contract-$label" "$before_inventory" "$before_hash"
}
run_repair_contract_mutation private-root '.private_temp_root = (if .private_temp_root == "/private/tmp" then "/tmp" else "/private/tmp" end)'
run_repair_contract_mutation direct-root '.state_dir = (if .private_temp_root == "/private/tmp" then "/private/tmp/lumen-standalone-gke-live.ZYX987" else "/tmp/lumen-standalone-gke-live.ZYX987" end)'
mkdir -p "$uncertain_root/nested/lumen-standalone-gke-live.ABC123"
run_repair_contract_mutation nested-root '.state_dir = (.state_dir + "/nested/lumen-standalone-gke-live.ABC123")'
cp "$TMP/run-contract.good" "$uncertain_root/run-contract.json"

run_repair_path_type_mutation() {
  local label=$1 target=$2
  cp "$TMP/run-contract.good" "$uncertain_root/run-contract.json"
  rm -rf -- "${uncertain_root:?}/$target"
  case "$target" in
    run-contract.json|terraform.tfstate) ln -s "$TMP/path-real/file" "$uncertain_root/$target" ;;
    terraform-data|control) ln -s "$TMP/path-real" "$uncertain_root/$target" ;;
  esac
  local before_inventory before_hash
  before_inventory=$(repair_inventory); before_hash=$(repair_sentinel_hash)
  : >"$STATE/events"
  expect_reject "repair-path-$label" env PATH="$FIXTURE/bin:$SYSTEM_PATH" "$REPAIR_FIXTURE" --state-dir "$uncertain_root" --confirm-destroy "$CLUSTER_ID"
  [[ -L "$uncertain_root/$target" ]] || fail "repair-path-$label removed offending entry"
  assert_repair_unchanged "repair-path-$label" "$before_inventory" "$before_hash"
  rm -f -- "$uncertain_root/$target"
  case "$target" in
    run-contract.json) cp "$TMP/run-contract.good" "$uncertain_root/run-contract.json" ;;
    terraform.tfstate) : >"$uncertain_root/terraform.tfstate" ;;
    terraform-data|control) mkdir -m 700 "$uncertain_root/$target" ;;
  esac
}
for pair in 'run-contract-leaf run-contract.json' 'terraform-state-leaf terraform.tfstate' 'terraform-data-directory terraform-data' 'control-directory control'; do
  read -r label target <<<"$pair"
  run_repair_path_type_mutation "$label" "$target"
done

# safe_remove_run_root must refuse every unexpected shape without deleting
# any sentinel or the offending path.
make_removal_negative_root() {
  local label=$1
  local root
  root=$(mktemp -d "$PRIVATE_TMP_ROOT/lumen-standalone-gke-live.XXXXXX")
  chmod 700 "$root"
  mkdir -m 700 "$root/terraform-data" "$root/control" "$root/private-receipt"
  : >"$root/terraform.tfstate"; : >"$root/kubeconfig"; : >"$root/run-contract.json"
  printf '%s\n' "$label-state" >"$root/terraform-data/sentinel"
  printf '%s\n' "$label-control" >"$root/control/sentinel"
  printf '%s\n' "$label-state" >"$root/private-receipt/lumen-standalone-gke-receipt.json"
  printf '%s\n' "$label-sidecar" >"$root/private-receipt/lumen-standalone-gke-receipt.json.sha256"
  RETAINED_ROOTS="${RETAINED_ROOTS}${RETAINED_ROOTS:+$'\n'}$root"
  printf '%s' "$root"
}
run_removal_negative() {
  local label=$1 mutation=$2 root
  root=$(make_removal_negative_root "$label")
  RETAINED_ROOTS="${RETAINED_ROOTS}${RETAINED_ROOTS:+$'\n'}$root"
  local state_hash control_hash receipt_hash
  state_hash=$(sha256_file "$root/terraform-data/sentinel")
  control_hash=$(sha256_file "$root/control/sentinel")
  receipt_hash=$(sha256_file "$root/private-receipt/lumen-standalone-gke-receipt.json")
  case "$mutation" in
    unknown) : >"$root/unknown" ;;
    leaf) rm -f -- "$root/terraform.tfstate"; mkdir -m 700 "$root/terraform.tfstate" ;;
    subtree) ln -s "$TMP/path-real/file" "$root/control/link" ;;
    receipt) : >"$root/private-receipt/extra" ;;
    receipt-leaf) rm -rf -- "${root:?}/private-receipt/lumen-standalone-gke-receipt.json.sha256"; mkdir -m 700 "$root/private-receipt/lumen-standalone-gke-receipt.json.sha256" ;;
  esac
  RUN_ROOT="$root"
  : >"$STATE/events"
  expect_predicate_reject "safe-remove-$label" safe_remove_run_root
  ! grep -q 'terraform:apply:destroy' "$STATE/events" || fail "safe-remove-$label applied destroy"
  [[ ! -e "$TMP/out-create-uncertain" ]] || fail "safe-remove-$label wrote a public receipt"
  [[ -e "$root" && -e "$root/terraform-data/sentinel" && -e "$root/control/sentinel" ]] || fail "safe-remove-$label deleted evidence"
  if [[ "$mutation" == receipt-leaf ]]; then [[ -d "$root/private-receipt/lumen-standalone-gke-receipt.json.sha256" ]] || fail "safe-remove-$label deleted offending entry"; else [[ -f "$root/private-receipt/lumen-standalone-gke-receipt.json" ]] || fail "safe-remove-$label deleted evidence"; fi
  [[ "$(sha256_file "$root/terraform-data/sentinel")" == "$state_hash" && "$(sha256_file "$root/control/sentinel")" == "$control_hash" ]] || fail "safe-remove-$label changed sentinel"
  if [[ "$mutation" != receipt-leaf ]]; then [[ "$(sha256_file "$root/private-receipt/lumen-standalone-gke-receipt.json")" == "$receipt_hash" ]] || fail "safe-remove-$label changed receipt"; fi
  case "$mutation" in
    unknown) [[ -e "$root/unknown" ]] ;;
    leaf) [[ -d "$root/terraform.tfstate" ]] ;;
    subtree) [[ -L "$root/control/link" ]] ;;
    receipt) [[ -e "$root/private-receipt/extra" ]] ;;
    receipt-leaf) [[ -d "$root/private-receipt/lumen-standalone-gke-receipt.json.sha256" ]] ;;
  esac
}
for pair in 'unknown unknown' 'named-leaf leaf' 'control-subtree subtree' 'receipt-extra receipt' 'receipt-leaf receipt-leaf'; do
  read -r label mutation <<<"$pair"
  run_removal_negative "$label" "$mutation"
done
RUN_ROOT=''

# A fresh repair plan may contain a strict known delete subset when a prior
# partial operation already removed other resources.
: >"$STATE/events"; printf '%s\n' created >"$STATE/resource-state"; printf '%s\n' repair-plan-subset >"$STATE/mode"; rm -f "$uncertain_root/repair-destroy.tfplan"
PATH="$FIXTURE/bin:$SYSTEM_PATH" "$REPAIR_FIXTURE" --state-dir "$uncertain_root" --confirm-destroy "$CLUSTER_ID" >"$TMP/repair-subset.stdout" 2>"$TMP/repair-subset.stderr" || fail 'repair rejected a known delete subset'
grep -q 'terraform:apply:destroy' "$STATE/events" || fail 'repair did not apply a known delete subset'

for pair in 'action repair-plan-action' 'unknown repair-plan-unknown' 'five repair-plan-five'; do
  read -r label mode <<<"$pair"
  : >"$STATE/events"; printf '%s\n' created >"$STATE/resource-state"; printf '%s\n' "$mode" >"$STATE/mode"; rm -f "$uncertain_root/repair-destroy.tfplan"
  expect_reject "repair-plan-$label" env PATH="$FIXTURE/bin:$SYSTEM_PATH" "$REPAIR_FIXTURE" --state-dir "$uncertain_root" --confirm-destroy "$CLUSTER_ID"
  ! grep -q 'terraform:apply:destroy' "$STATE/events" || fail "repair applied rejected $label plan"
done

for mode in repair-state-nonempty post-list-fail post-list-present; do
  : >"$STATE/events"; printf '%s\n' created >"$STATE/resource-state"; printf '%s\n' "$mode" >"$STATE/mode"; rm -f "$uncertain_root/repair-destroy.tfplan"
  expect_reject "$mode" env PATH="$FIXTURE/bin:$SYSTEM_PATH" "$REPAIR_FIXTURE" --state-dir "$uncertain_root" --confirm-destroy "$CLUSTER_ID"
  [[ -d "$uncertain_root" && ! -e "$TMP/out-create-uncertain" ]] || fail "$mode deleted evidence or wrote a public receipt"
done
[[ -d "$uncertain_root" ]] || fail 'repair rejection deleted retained state'

cleanup
cleanup

LIVE_ROOTS_AFTER=$(find "$PRIVATE_TMP_ROOT" -maxdepth 1 -type d -name 'lumen-standalone-gke-live.??????' -print | LC_ALL=C sort)
[[ "$LIVE_ROOTS_AFTER" == "$LIVE_ROOTS_BEFORE" ]] || fail 'cleanup changed live root inventory'

printf '%s\n' 'live lifecycle cloud-free contract passed'
