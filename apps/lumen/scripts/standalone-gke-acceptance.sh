#!/usr/bin/env bash
# shellcheck disable=SC2329,SC2016

# Standalone-only GKE acceptance gate.
#
# This gate mutates only after LUMEN_STANDALONE_GKE_MUTATION=1 and a task-local
# kubeconfig are supplied. Evidence is sanitized and written only after all
# acceptance resources are removed.

set +x
umask 077
set -euo pipefail
CDPATH=

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd -P)"
REPO_ROOT="$(cd "$SCRIPT_DIR/../../.." && pwd -P)"
KUSTOMIZE_SOURCE_ROOT="$REPO_ROOT/kustomize/lumen-standalone-acceptance"
KUSTOMIZE_RENDERER_SOURCE="$KUSTOMIZE_SOURCE_ROOT/scripts/render.sh"
KUSTOMIZE_VALIDATOR_SOURCE="$KUSTOMIZE_SOURCE_ROOT/scripts/validate.rb"
KUSTOMIZE_RENDERER_SHA256="f83e347b5f66c6cad049595a776230ea559f80efc265129f407032bf5a93dd74"
KUSTOMIZE_VALIDATOR_SHA256="43355d4a083303c9ffadade98f4add46958d7a7e625100dea97d979a3d1d294e"

die() {
  echo "standalone GKE acceptance: $*" >&2
  exit 2
}

PRIVATE_TMP_ROOT="$(cd -P /tmp && pwd -P)"
case "$PRIVATE_TMP_ROOT" in /tmp|/private/tmp) ;; *) die 'unsupported private temp root' ;; esac

safe_private_dir() {
  local path=$1
  [[ "$path" == "$PRIVATE_TMP_ROOT"/* && "$path" != */ && "$path" != *'/../'* && "$path" != */.. && "$path" != *'/./'* && -d "$path" && ! -L "$path" ]] || return 1
  [[ "$(cd "$path" && pwd -P)" == "$path" ]]
}
safe_private_file() {
  local path=$1 parent
  [[ "$path" == "$PRIVATE_TMP_ROOT"/* && "$path" != *'/../'* && "$path" != */.. && "$path" != *'/./'* && -f "$path" && ! -L "$path" ]] || return 1
  parent=${path%/*}; safe_private_dir "$parent"
}
private_mode() {
  local path=$1 mode status
  mode=$(stat -c %a "$path" 2>/dev/null); status=$?
  if [[ "$status" -eq 0 ]]; then [[ "$mode" == 700 ]] || return 1; printf '%s\n' "$mode"; return 0; fi
  [[ -z "$mode" ]] || return 1
  mode=$(stat -f %Lp "$path" 2>/dev/null) || return 1
  [[ "$mode" == 700 ]] || return 1; printf '%s\n' "$mode"
}

if [[ "$#" -ne 2 || "$1" != "--mode" || "$2" != "gke" ]]; then
  die "usage is exactly: $0 --mode gke"
fi

require_env() {
  local name="$1"
  if [[ -z "${!name:-}" ]]; then
    die "$name is required"
  fi
}

for name in \
  KUBECONFIG \
  LUMEN_STANDALONE_GKE_CONTEXT \
  LUMEN_STANDALONE_GKE_PROJECT_ID \
  LUMEN_STANDALONE_GKE_LOCATION \
  LUMEN_STANDALONE_GKE_CLUSTER \
  LUMEN_STANDALONE_GKE_CLI \
  LUMEN_STANDALONE_GKE_IMAGE \
  LUMEN_STANDALONE_GKE_CLIENT_IMAGE \
  LUMEN_STANDALONE_GKE_CLI_TARGET \
  LUMEN_STANDALONE_GKE_STORAGE_CLASS \
  LUMEN_STANDALONE_GKE_NODE_POOL \
  LUMEN_STANDALONE_GKE_RUN_ID \
  LUMEN_STANDALONE_GKE_EXPECTED_COMMIT \
  LUMEN_STANDALONE_GKE_EXPECTED_RUN_ID \
  LUMEN_STANDALONE_GKE_EXPECTED_RUN_ATTEMPT \
  LUMEN_STANDALONE_GKE_EXPECTED_MANIFEST_SHA256 \
  LUMEN_STANDALONE_GKE_EVIDENCE_DIR; do
  require_env "$name"
done
[[ "${LUMEN_STANDALONE_GKE_MUTATION:-}" == "1" ]] ||
  die "set LUMEN_STANDALONE_GKE_MUTATION=1 to enable live GKE mutation"
require_env LUMEN_STANDALONE_GKE_CANDIDATE_RECEIPT_DIR

reject_token_env() {
  local name
  while IFS= read -r name; do
    case "$name" in
      *TOKEN*|*AUTHORIZATION*) die "$name must not be set; this gate accepts no bearer-token environment" ;;
    esac
  done < <(compgen -e)
}
reject_token_env

require_tool() {
  command -v "$1" >/dev/null 2>&1 || die "required command not found: $1"
}

for tool in \
  gcloud \
  kubectl \
  jq \
  mktemp \
  cp \
  mv \
  chmod \
  find \
  grep \
  awk \
  base64 \
  cmp \
  cut \
  mkdir \
  sed \
  tar \
  tr \
  sort \
  wc \
  rm \
  sleep \
  curl \
  ruby; do
  require_tool "$tool"
done
sha256_stdin() {
  if command -v shasum >/dev/null 2>&1; then
    shasum -a 256 | awk '{print $1}'
  else
    sha256sum | awk '{print $1}'
  fi
}
sha256_file() {
  sha256_stdin < "$1"
}

[[ -d "$KUSTOMIZE_SOURCE_ROOT" && ! -L "$KUSTOMIZE_SOURCE_ROOT" ]] ||
  die "shared Kustomize source must be a regular non-symlink directory"
[[ -f "$KUSTOMIZE_RENDERER_SOURCE" && -x "$KUSTOMIZE_RENDERER_SOURCE" && ! -L "$KUSTOMIZE_RENDERER_SOURCE" ]] ||
  die "shared Kustomize renderer source must be an executable regular non-symlink file"
[[ -f "$KUSTOMIZE_VALIDATOR_SOURCE" && -x "$KUSTOMIZE_VALIDATOR_SOURCE" && ! -L "$KUSTOMIZE_VALIDATOR_SOURCE" ]] ||
  die "shared Kustomize validator source must be an executable regular non-symlink file"
[[ -z "$(find "$KUSTOMIZE_SOURCE_ROOT" -type l -print -quit)" ]] ||
  die "shared Kustomize source must not contain symlinks"

safe_private_file "$KUBECONFIG" ||
  die "KUBECONFIG must be an existing regular non-symlink file"
[[ -f "$LUMEN_STANDALONE_GKE_CLI" && -x "$LUMEN_STANDALONE_GKE_CLI" && ! -L "$LUMEN_STANDALONE_GKE_CLI" ]] ||
  die "LUMEN_STANDALONE_GKE_CLI must be an executable regular non-symlink file"
safe_private_dir "$LUMEN_STANDALONE_GKE_EVIDENCE_DIR" ||
  die "LUMEN_STANDALONE_GKE_EVIDENCE_DIR must be an existing non-symlink directory"
[[ -z "$(find "$LUMEN_STANDALONE_GKE_EVIDENCE_DIR" -mindepth 1 -print -quit)" ]] ||
  die "LUMEN_STANDALONE_GKE_EVIDENCE_DIR must be empty"
[[ -d "$LUMEN_STANDALONE_GKE_CANDIDATE_RECEIPT_DIR" && ! -L "$LUMEN_STANDALONE_GKE_CANDIDATE_RECEIPT_DIR" ]] ||
  die "LUMEN_STANDALONE_GKE_CANDIDATE_RECEIPT_DIR must be an existing non-symlink directory"

[[ "$LUMEN_STANDALONE_GKE_CONTEXT" != *[[:space:]]* ]] ||
  die "LUMEN_STANDALONE_GKE_CONTEXT must not contain whitespace"
[[ "$LUMEN_STANDALONE_GKE_PROJECT_ID" =~ ^[a-z][a-z0-9-]{4,28}[a-z0-9]$ ]] ||
  die "LUMEN_STANDALONE_GKE_PROJECT_ID is not a safe GCP project id"
[[ "$LUMEN_STANDALONE_GKE_LOCATION" =~ ^[a-z0-9]([a-z0-9-]*[a-z0-9])?$ ]] || die "LUMEN_STANDALONE_GKE_LOCATION is not safe"
[[ "$LUMEN_STANDALONE_GKE_CLUSTER" =~ ^[a-z]([a-z0-9-]{0,38}[a-z0-9])?$ ]] || die "LUMEN_STANDALONE_GKE_CLUSTER is not safe"
validate_dns_subdomain() {
  local value="$1"
  local label
  local -a labels
  [[ "${#value}" -le 253 ]] || return 1
  IFS='.' read -r -a labels <<< "$value"
  for label in "${labels[@]}"; do
    [[ "${#label}" -le 63 ]] || return 1
  done
}
if ! [[ "$LUMEN_STANDALONE_GKE_STORAGE_CLASS" =~ ^[a-z0-9]([a-z0-9-]*[a-z0-9])?(\.[a-z0-9]([a-z0-9-]*[a-z0-9])?)*$ ]] ||
  ! validate_dns_subdomain "$LUMEN_STANDALONE_GKE_STORAGE_CLASS"; then
  die "LUMEN_STANDALONE_GKE_STORAGE_CLASS is not a safe DNS name"
fi
[[ "$LUMEN_STANDALONE_GKE_NODE_POOL" =~ ^[a-z]([a-z0-9-]{0,38}[a-z0-9])?$ ]] ||
  die "LUMEN_STANDALONE_GKE_NODE_POOL is not a safe GKE node-pool name"
[[ "$LUMEN_STANDALONE_GKE_RUN_ID" =~ ^[a-z0-9]([a-z0-9-]{0,50}[a-z0-9])?$ ]] ||
  die "LUMEN_STANDALONE_GKE_RUN_ID must be a DNS-safe lowercase name"
[[ "$LUMEN_STANDALONE_GKE_IMAGE" =~ ^ghcr\.io/chrischeng-c4/lumen@sha256:[0-9a-f]{64}$ ]] ||
  die "LUMEN_STANDALONE_GKE_IMAGE must be the exact immutable Lumen digest form"
APPROVED_CLIENT_IMAGE="docker.io/curlimages/curl@sha256:7c12af72ceb38b7432ab85e1a265cff6ae58e06f95539d539b654f2cfa64bb13"
[[ "$LUMEN_STANDALONE_GKE_CLIENT_IMAGE" == "$APPROVED_CLIENT_IMAGE" ]] ||
  die "LUMEN_STANDALONE_GKE_CLIENT_IMAGE must be the approved immutable client image"
[[ "$LUMEN_STANDALONE_GKE_EXPECTED_COMMIT" =~ ^[0-9a-f]{40}$ ]] ||
  die "LUMEN_STANDALONE_GKE_EXPECTED_COMMIT must be a landed lowercase 40-hex commit"
[[ "$LUMEN_STANDALONE_GKE_EXPECTED_RUN_ID" =~ ^[0-9]+$ ]] ||
  die "LUMEN_STANDALONE_GKE_EXPECTED_RUN_ID must be decimal"
[[ "$LUMEN_STANDALONE_GKE_EXPECTED_RUN_ATTEMPT" =~ ^[0-9]+$ ]] ||
  die "LUMEN_STANDALONE_GKE_EXPECTED_RUN_ATTEMPT must be decimal"
[[ "$LUMEN_STANDALONE_GKE_EXPECTED_MANIFEST_SHA256" =~ ^[0-9a-f]{64}$ ]] ||
  die "LUMEN_STANDALONE_GKE_EXPECTED_MANIFEST_SHA256 must be lowercase sha256"

RUNTIME_NAME="lumen"
RUNTIME_NAMESPACE="${LUMEN_STANDALONE_GKE_RUN_ID}-standalone"
CLIENT_NAMESPACE="${LUMEN_STANDALONE_GKE_RUN_ID}-client"
STATEFULSET_NAME="$RUNTIME_NAME"
PVC_NAME="${RUNTIME_NAME}-data"

TMP_ROOT=""
INITIAL_CPU=""
INITIAL_MEMORY=""
RECEIPT_SHA256=""
CANDIDATE_COMMIT=""
CANDIDATE_RUN_ID=""
CANDIDATE_ATTEMPT=""
CANDIDATE_WORKFLOW=""
ROOT_DIGEST=""
AMD64_DIGEST=""
ARM64_DIGEST=""

# Before ownership is proved, cleanup has no cluster authority. It can only
# remove the private directory that may have been made during preflight.
cleanup() {
  local status=$?
  set +e
  trap - EXIT INT TERM
  if [[ -n "${TMP_ROOT:-}" && -d "$TMP_ROOT" ]]; then
    rm -rf -- "$TMP_ROOT"
  fi
  exit "$status"
}

trap cleanup EXIT
trap 'exit 130' INT
trap 'exit 143' TERM

TMP_ROOT="$(mktemp -d "$PRIVATE_TMP_ROOT/lumen-standalone-gke.XXXXXX")"
chmod 700 "$TMP_ROOT"
[[ -d "$TMP_ROOT" && ! -L "$TMP_ROOT" && "$(cd "$TMP_ROOT" && pwd -P)" == "$TMP_ROOT" ]] || die 'private temporary root is not canonical'
[[ "${TMP_ROOT%/*}" == "$PRIVATE_TMP_ROOT" && "${TMP_ROOT##*/}" =~ ^lumen-standalone-gke\.[A-Za-z0-9]{6}$ ]] || die 'private temporary root identity is unsafe'
[[ "$(private_mode "$TMP_ROOT")" == 700 ]] || die 'private temporary root mode is not 0700'
KUBECTL_CACHE_DIR="$TMP_ROOT/kubectl-cache"
[[ ! -e "$KUBECTL_CACHE_DIR" && ! -L "$KUBECTL_CACHE_DIR" ]] || die 'kubectl cache path already exists'
mkdir -m 700 "$KUBECTL_CACHE_DIR"
[[ -d "$KUBECTL_CACHE_DIR" && ! -L "$KUBECTL_CACHE_DIR" && "$(cd "$KUBECTL_CACHE_DIR" && pwd -P)" == "$KUBECTL_CACHE_DIR" ]] || die 'kubectl cache path is not canonical'
[[ "${KUBECTL_CACHE_DIR%/*}" == "$TMP_ROOT" && "${KUBECTL_CACHE_DIR##*/}" == kubectl-cache ]] || die 'kubectl cache path identity is unsafe'
[[ "$(private_mode "$KUBECTL_CACHE_DIR")" == 700 ]] || die 'kubectl cache path mode is not 0700'
PRIVATE_REPOSITORY_ROOT="$TMP_ROOT/repository"
[[ ! -e "$PRIVATE_REPOSITORY_ROOT" && ! -L "$PRIVATE_REPOSITORY_ROOT" ]] ||
  die "private repository root already exists"
mkdir -m 700 "$PRIVATE_REPOSITORY_ROOT"
mkdir -m 700 "$PRIVATE_REPOSITORY_ROOT/kustomize"
[[ -d "$PRIVATE_REPOSITORY_ROOT" && ! -L "$PRIVATE_REPOSITORY_ROOT" && -d "$PRIVATE_REPOSITORY_ROOT/kustomize" && ! -L "$PRIVATE_REPOSITORY_ROOT/kustomize" ]] ||
  die "private repository parent is unsafe"
PRIVATE_REPOSITORY_ROOT="$(cd "$PRIVATE_REPOSITORY_ROOT" && pwd -P)" || die "private repository root cannot be canonicalized"
KUSTOMIZE_ROOT="$PRIVATE_REPOSITORY_ROOT/kustomize/lumen-standalone-acceptance"
[[ ! -e "$KUSTOMIZE_ROOT" && ! -L "$KUSTOMIZE_ROOT" ]] ||
  die "private Kustomize harness path already exists"
cp -R -- "$KUSTOMIZE_SOURCE_ROOT" "$KUSTOMIZE_ROOT"
[[ -d "$KUSTOMIZE_ROOT" && ! -L "$KUSTOMIZE_ROOT" ]] ||
  die "private Kustomize harness was not copied as a regular directory"
[[ -z "$(find "$KUSTOMIZE_ROOT" -type l -print -quit)" ]] ||
  die "private Kustomize harness must not contain symlinks"
KUSTOMIZE_RENDERER="$KUSTOMIZE_ROOT/scripts/render.sh"
KUSTOMIZE_VALIDATOR="$KUSTOMIZE_ROOT/scripts/validate.rb"
[[ -f "$KUSTOMIZE_RENDERER" && -x "$KUSTOMIZE_RENDERER" && ! -L "$KUSTOMIZE_RENDERER" ]] ||
  die "private Kustomize renderer must be an executable regular non-symlink file"
[[ -f "$KUSTOMIZE_VALIDATOR" && -x "$KUSTOMIZE_VALIDATOR" && ! -L "$KUSTOMIZE_VALIDATOR" ]] ||
  die "private Kustomize validator must be an executable regular non-symlink file"
[[ "$(sha256_file "$KUSTOMIZE_RENDERER")" == "$KUSTOMIZE_RENDERER_SHA256" ]] ||
  die "private Kustomize renderer hash differs from the controller-bound contract"
[[ "$(sha256_file "$KUSTOMIZE_VALIDATOR")" == "$KUSTOMIZE_VALIDATOR_SHA256" ]] ||
  die "private Kustomize validator hash differs from the controller-bound contract"

k() {
  kubectl \
    --kubeconfig "$KUBECONFIG" \
    --context "$LUMEN_STANDALONE_GKE_CONTEXT" \
    --cache-dir "$KUBECTL_CACHE_DIR" \
    "$@"
}

if ! context_name="$(k config get-contexts "$LUMEN_STANDALONE_GKE_CONTEXT" -o name 2>"$TMP_ROOT/context.err")"; then
  die "requested Kubernetes context could not be read"
fi
[[ "$context_name" == "$LUMEN_STANDALONE_GKE_CONTEXT" ]] ||
  die "requested Kubernetes context was not found"
if ! selected_context="$(k config view --minify -o jsonpath='{.contexts[0].name}' 2>"$TMP_ROOT/context-selected.err")"; then
  die "requested Kubernetes context could not be selected"
fi
[[ "$selected_context" == "$LUMEN_STANDALONE_GKE_CONTEXT" ]] ||
  die "kubectl wrapper did not select the requested context"
current_context="$(kubectl --kubeconfig "$KUBECONFIG" --cache-dir "$KUBECTL_CACHE_DIR" config current-context 2>"$TMP_ROOT/current-context.err")" ||
  die "task-local kubeconfig has no current context"
[[ "$current_context" == "$LUMEN_STANDALONE_GKE_CONTEXT" ]] ||
  die "task-local kubeconfig current context must equal LUMEN_STANDALONE_GKE_CONTEXT for backup and restore"
[[ "$(kubectl --kubeconfig "$KUBECONFIG" --cache-dir "$KUBECTL_CACHE_DIR" config get-contexts -o name | awk 'NF {count++; name=$0} END {if (count == 1) print name}')" == "$LUMEN_STANDALONE_GKE_CONTEXT" ]] ||
  die "task-local kubeconfig must contain exactly the requested context"

if ! gcloud container clusters describe "$LUMEN_STANDALONE_GKE_CLUSTER" \
  --project="$LUMEN_STANDALONE_GKE_PROJECT_ID" \
  --location="$LUMEN_STANDALONE_GKE_LOCATION" \
  --format=json >"$TMP_ROOT/cluster.json" 2>"$TMP_ROOT/cluster.err"; then
  die "could not describe the requested GKE cluster"
fi
jq -e '
  ((.autopilot.enabled // false) != true)
  and ((.networkPolicy.enabled == true and .networkPolicy.provider == "CALICO")
       or .networkConfig.datapathProvider == "ADVANCED_DATAPATH")
' "$TMP_ROOT/cluster.json" >/dev/null ||
  die "cluster is not existing Standard GKE with enforced NetworkPolicy"

if ! selected_server="$(k config view --minify -o jsonpath='{.clusters[0].cluster.server}' 2>"$TMP_ROOT/context-server.err")"; then
  die "requested Kubernetes context server could not be read"
fi
cluster_endpoint="$(jq -er '.endpoint // empty' "$TMP_ROOT/cluster.json")" ||
  die "GKE cluster endpoint is missing"
if [[ -z "$cluster_endpoint" ]] || {
  [[ "$selected_server" != "https://${cluster_endpoint}" ]] &&
    [[ "$selected_server" != "https://${cluster_endpoint}/" ]]
}; then
  die "Kubernetes context server does not match the described GKE cluster endpoint"
fi

if ! gcloud container node-pools describe "$LUMEN_STANDALONE_GKE_NODE_POOL" \
  --cluster="$LUMEN_STANDALONE_GKE_CLUSTER" \
  --project="$LUMEN_STANDALONE_GKE_PROJECT_ID" \
  --location="$LUMEN_STANDALONE_GKE_LOCATION" \
  --format=json >"$TMP_ROOT/node-pool.json" 2>"$TMP_ROOT/node-pool.err"; then
  die "requested node pool does not exist"
fi
jq -e --arg node_pool "$LUMEN_STANDALONE_GKE_NODE_POOL" '.name == $node_pool' "$TMP_ROOT/node-pool.json" >/dev/null ||
  die "described node pool name did not match the requested node pool"

if ! k get storageclass "$LUMEN_STANDALONE_GKE_STORAGE_CLASS" -o json >"$TMP_ROOT/storage-class.json" 2>"$TMP_ROOT/storage-class.err"; then
  die "requested StorageClass does not exist"
fi
jq -e --arg storage_class "$LUMEN_STANDALONE_GKE_STORAGE_CLASS" '.metadata.name == $storage_class and .reclaimPolicy == "Delete"' "$TMP_ROOT/storage-class.json" >/dev/null ||
  die "requested StorageClass must exist and use reclaimPolicy Delete for acceptance cleanup"

INITIAL_CPU="500m"
INITIAL_MEMORY="512Mi"
CONFIG="$TMP_ROOT/lumen.yaml"
RENDERED="$TMP_ROOT/rendered"
printf '%s\n' \
  "name: lumen" \
  "namespace: $RUNTIME_NAMESPACE" \
  "nodePool: $LUMEN_STANDALONE_GKE_NODE_POOL" \
  "cpu: $INITIAL_CPU" \
  "memory: $INITIAL_MEMORY" \
  "storageSize: 20Gi" \
  "storageClass: $LUMEN_STANDALONE_GKE_STORAGE_CLASS" \
  "allowedServiceAccounts:" \
  "  - $CLIENT_NAMESPACE/app" >"$CONFIG"

[[ "$(awk '/^[a-zA-Z][a-zA-Z0-9]*:/ {count++} END {print count+0}' "$CONFIG")" == "8" ]] ||
  die "frozen lumen.yaml shape was not written"
grep -Eq '^storageSize: 20Gi$' "$CONFIG" || die "frozen storage size is not 20Gi"
grep -Eq "^  - ${CLIENT_NAMESPACE}/app$" "$CONFIG" ||
  die "frozen allowedServiceAccounts list is not exact"
if grep -Eq '^image:' "$CONFIG"; then
  die "lumen.yaml must not contain an image configuration knob"
fi

yaml_json() {
  local source="$1"
  local output="$2"
  if ! k create --dry-run=client --validate=false -f "$source" -o json >"$output" 2>"$output.err"; then
    die "could not canonicalize rendered StatefulSet"
  fi
}

# The final candidate receipt is a release input, not an advisory note. This
# validator accepts only the exact v3 shape produced for an already-landed main
# commit. It verifies every archive pair and hashes the controller binary from
# its archive stream; it never extracts an untrusted archive.
checksum_sidecar_exact() {
  local sidecar="$1" digest="$2" filename="$3"
  [[ -f "$sidecar" && ! -L "$sidecar" ]] || die "checksum sidecar is missing"
  cmp -s "$sidecar" <(printf '%s  %s\n' "$digest" "$filename") ||
    die "checksum sidecar must contain exactly one hash and its exact filename"
}

validate_candidate_manifest_v2() {
  local manifest="$LUMEN_STANDALONE_GKE_CANDIDATE_RECEIPT_DIR/final-candidate-manifest.json"
  local sidecar="${manifest}.sha256" target archive archive_hash sidecar_hash member_hash member mode listed expected_members
  [[ -f "$manifest" && ! -L "$manifest" && -f "$sidecar" && ! -L "$sidecar" ]] || die "candidate final manifest and sidecar are required"
  RECEIPT_SHA256="$(sha256_file "$manifest")"
  checksum_sidecar_exact "$sidecar" "$RECEIPT_SHA256" "${manifest##*/}"
  jq -e '
    (keys|sort) == ["artifacts","candidate_tag","commit","image","jobs","pr","repository","run_attempt","run_id","run_url","sboms","schema","source_ref","tag","version","workflow_id","workflow_path","workflow_ref"] and
    .schema == "cclab.lumen.candidate-manifest.v3" and .repository == "chrischeng-c4/axiom" and
    .version == "0.4.29" and .tag == "lumen@0.4.29" and .source_ref == "refs/heads/main" and
    .workflow_path == ".github/workflows/lumen-release-candidate.yml" and
    .workflow_ref == "chrischeng-c4/axiom/.github/workflows/lumen-release-candidate.yml@refs/heads/main" and
    (.commit|type == "string" and test("^[0-9a-f]{40}$")) and
    (.run_id|type == "string" and test("^[0-9]+$")) and
    (.run_attempt|type == "string" and test("^[0-9]+$")) and
    .candidate_tag == ("release-candidate-" + .run_id + "-" + .run_attempt) and
    .image.repository == "ghcr.io/chrischeng-c4/lumen" and
    ([.image.root_digest,.image.amd64_digest,.image.arm64_digest] | all(.[]; type == "string" and test("^sha256:[0-9a-f]{64}$"))) and
    (.jobs|keys|sort) == ["build","ghcr-image-and-attest","identity","kind-amd64","kind-arm64","manifest","result","verify-candidate","verify-libraries"] and
    (.jobs|all(.[]; . == "success")) and
    (.artifacts|type == "array" and length == 5 and
      ([.[].target]|sort) == ["aarch64-apple-darwin","aarch64-unknown-linux-gnu","aarch64-unknown-linux-musl","x86_64-unknown-linux-gnu","x86_64-unknown-linux-musl"] and
      all(.[]; (keys|sort) == ["archive","archive_sha256","sidecar","sidecar_sha256","target"] and
        .archive == ("lumen-" + .target + ".tar.gz") and .sidecar == (.archive + ".sha256") and
        (.archive_sha256|test("^[0-9a-f]{64}$")) and (.sidecar_sha256|test("^[0-9a-f]{64}$"))))
  ' "$manifest" >/dev/null || die "candidate final manifest is not the exact v3 landed-main receipt"
  CANDIDATE_RUN_ID="$(jq -er '.run_id' "$manifest")"
  CANDIDATE_ATTEMPT="$(jq -er '.run_attempt' "$manifest")"
  CANDIDATE_WORKFLOW="$(jq -er '.workflow_ref' "$manifest")"
  CANDIDATE_COMMIT="$(jq -er '.commit' "$manifest")"
  ROOT_DIGEST="$(jq -er '.image.root_digest' "$manifest")"
  AMD64_DIGEST="$(jq -er '.image.amd64_digest' "$manifest")"
  ARM64_DIGEST="$(jq -er '.image.arm64_digest' "$manifest")"
  [[ "$RECEIPT_SHA256" == "$LUMEN_STANDALONE_GKE_EXPECTED_MANIFEST_SHA256" ]] ||
    die "candidate manifest hash differs from the controller-bound expected hash"
  [[ "$CANDIDATE_COMMIT" == "$LUMEN_STANDALONE_GKE_EXPECTED_COMMIT" ]] ||
    die "candidate manifest commit differs from the controller-bound landed commit"
  [[ "$CANDIDATE_RUN_ID" == "$LUMEN_STANDALONE_GKE_EXPECTED_RUN_ID" ]] ||
    die "candidate manifest run id differs from the controller-bound expected run"
  [[ "$CANDIDATE_ATTEMPT" == "$LUMEN_STANDALONE_GKE_EXPECTED_RUN_ATTEMPT" ]] ||
    die "candidate manifest run attempt differs from the controller-bound expected attempt"
  [[ "$LUMEN_STANDALONE_GKE_IMAGE" == "ghcr.io/chrischeng-c4/lumen@$ROOT_DIGEST" ]] || die "candidate image is not the exact receipt root digest"
  for target in aarch64-apple-darwin x86_64-unknown-linux-gnu aarch64-unknown-linux-gnu x86_64-unknown-linux-musl aarch64-unknown-linux-musl; do
    archive="$(jq -er --arg t "$target" '.artifacts[]|select(.target == $t)|.archive' "$manifest")"
    archive_hash="$(jq -er --arg t "$target" '.artifacts[]|select(.target == $t)|.archive_sha256' "$manifest")"
    sidecar_hash="$(jq -er --arg t "$target" '.artifacts[]|select(.target == $t)|.sidecar_sha256' "$manifest")"
    [[ -f "$LUMEN_STANDALONE_GKE_CANDIDATE_RECEIPT_DIR/$archive" && ! -L "$LUMEN_STANDALONE_GKE_CANDIDATE_RECEIPT_DIR/$archive" ]] || die "candidate archive is missing"
    [[ -f "$LUMEN_STANDALONE_GKE_CANDIDATE_RECEIPT_DIR/${archive}.sha256" && ! -L "$LUMEN_STANDALONE_GKE_CANDIDATE_RECEIPT_DIR/${archive}.sha256" ]] || die "candidate archive sidecar is missing"
    [[ "$(sha256_file "$LUMEN_STANDALONE_GKE_CANDIDATE_RECEIPT_DIR/$archive")" == "$archive_hash" ]] || die "candidate archive hash mismatch"
    [[ "$(sha256_file "$LUMEN_STANDALONE_GKE_CANDIDATE_RECEIPT_DIR/${archive}.sha256")" == "$sidecar_hash" ]] || die "candidate archive sidecar hash mismatch"
    checksum_sidecar_exact "$LUMEN_STANDALONE_GKE_CANDIDATE_RECEIPT_DIR/${archive}.sha256" "$archive_hash" "$archive"
  done
  target="$LUMEN_STANDALONE_GKE_CLI_TARGET"
  case "$target" in
    aarch64-apple-darwin|x86_64-unknown-linux-gnu|aarch64-unknown-linux-gnu|x86_64-unknown-linux-musl|aarch64-unknown-linux-musl) ;;
    *) die "candidate controller CLI target is not approved" ;;
  esac
  archive="lumen-${target}.tar.gz"
  member="lumen-${target}/lumen"
  listed="$(tar -tzf "$LUMEN_STANDALONE_GKE_CANDIDATE_RECEIPT_DIR/$archive" | LC_ALL=C sort)"
  expected_members="$(printf 'lumen-%s/\nlumen-%s/lumen\nlumen-%s/README.md\n' "$target" "$target" "$target" | LC_ALL=C sort)"
  [[ "$listed" == "$expected_members" ]] || die "candidate controller archive has unexpected members"
  mode="$(tar -tvzf "$LUMEN_STANDALONE_GKE_CANDIDATE_RECEIPT_DIR/$archive" | awk -v member="$member" '$NF == member {print $1; exit}')"
  [[ "$mode" == -*x* ]] || die "candidate controller binary is not executable"
  member_hash="$(tar -xOf "$LUMEN_STANDALONE_GKE_CANDIDATE_RECEIPT_DIR/$archive" "$member" | sha256_stdin)" || die "could not hash candidate CLI archive member"
  CONTROLLER_CLI_SHA256="$(sha256_file "$LUMEN_STANDALONE_GKE_CLI")"
  [[ "$member_hash" == "$CONTROLLER_CLI_SHA256" ]] || die "candidate controller CLI bytes differ from local CLI"
  VERIFIED_CLI="$TMP_ROOT/lumen-controller-verified"
  cp "$LUMEN_STANDALONE_GKE_CLI" "$VERIFIED_CLI"
  chmod 700 "$VERIFIED_CLI"
  [[ "$(sha256_file "$VERIFIED_CLI")" == "$CONTROLLER_CLI_SHA256" ]] || die "private verified controller CLI hash changed"
}

validate_candidate_manifest_v2
[[ "$LUMEN_STANDALONE_GKE_IMAGE" == "ghcr.io/chrischeng-c4/lumen@$ROOT_DIGEST" ]] ||
  die "candidate image is not the exact receipt root digest"
if ! "$VERIFIED_CLI" standalone gke render \
  --file "$CONFIG" \
  --out "$RENDERED" >"$TMP_ROOT/render.out" 2>"$TMP_ROOT/render.err"; then
  die "verified candidate CLI could not render the standalone manifests"
fi
[[ -f "$RENDERED/.lumen-standalone-managed" ]] || die "render marker is missing"
[[ -d "$RENDERED/storage" && -d "$RENDERED/runtime" ]] ||
  die "render did not produce separate storage and runtime roots"
[[ -f "$RENDERED/storage/kustomization.yaml" && -f "$RENDERED/runtime/kustomization.yaml" ]] ||
  die "rendered kustomizations are missing"
[[ -f "$RENDERED/runtime/statefulset.yaml" ]] || die "rendered StatefulSet is missing"

assert_statefulset() {
  local document="$1"
  jq -e \
    --arg name "$STATEFULSET_NAME" \
    --arg namespace "$RUNTIME_NAMESPACE" \
    --arg node_pool "$LUMEN_STANDALONE_GKE_NODE_POOL" \
    --arg cpu "$INITIAL_CPU" \
    --arg memory "$INITIAL_MEMORY" \
    --arg claim "$PVC_NAME" \
    '
      .apiVersion == "apps/v1"
      and .kind == "StatefulSet"
      and .metadata.name == $name
      and .metadata.namespace == $namespace
      and .spec.replicas == 1
      and (.spec.template.spec.nodeSelector["cloud.google.com/gke-nodepool"] == $node_pool)
      and (.spec.template.spec.containers | length == 1)
      and .spec.template.spec.containers[0].name == "serving"
      and .spec.template.spec.containers[0].image == "ghcr.io/chrischeng-c4/lumen:0.4.29"
      and .spec.template.spec.containers[0].resources.requests.cpu == $cpu
      and .spec.template.spec.containers[0].resources.requests.memory == $memory
      and (.spec.template.spec.volumes
           | map(select(.name == "data" and .persistentVolumeClaim.claimName == $claim))
           | length == 1)
      and ((.spec.volumeClaimTemplates // []) | length == 0)
    ' "$document" >/dev/null || die "canonical StatefulSet contract failed"
}

patch_statefulset_image() {
  local statefulset="$1"
  local label="$2"
  local canonical="$TMP_ROOT/${label}-image-patch-input.json"
  local patched="$TMP_ROOT/${label}-image-patch-output.json"

  yaml_json "$statefulset" "$canonical"
  jq -e '
    (.spec.template.spec.containers | length == 1)
    and .spec.template.spec.containers[0].name == "serving"
    and .spec.template.spec.containers[0].image == "ghcr.io/chrischeng-c4/lumen:0.4.29"
  ' "$canonical" >/dev/null || die "runtime image patch precondition failed"
  jq --arg image "$LUMEN_STANDALONE_GKE_IMAGE" '
    if (.spec.template.spec.containers | length) != 1
       or .spec.template.spec.containers[0].name != "serving"
       or .spec.template.spec.containers[0].image != "ghcr.io/chrischeng-c4/lumen:0.4.29"
    then error("runtime image patch precondition failed")
    else .spec.template.spec.containers[0].image = $image
    end
  ' "$canonical" >"$patched" || die "runtime image patch failed"
  jq -e --arg image "$LUMEN_STANDALONE_GKE_IMAGE" '
    (.spec.template.spec.containers | length == 1)
    and .spec.template.spec.containers[0].name == "serving"
    and .spec.template.spec.containers[0].image == $image
  ' "$patched" >/dev/null || die "runtime image patch postcondition failed"
  mv -f -- "$patched" "$statefulset"
}

ORIGINAL_STATEFULSET="$TMP_ROOT/original-statefulset.yaml"
ORIGINAL_JSON="$TMP_ROOT/original-statefulset.json"
cp "$RENDERED/runtime/statefulset.yaml" "$ORIGINAL_STATEFULSET"
yaml_json "$ORIGINAL_STATEFULSET" "$ORIGINAL_JSON"
assert_statefulset "$ORIGINAL_JSON"

prepare_runtime_copy() {
  local source_runtime="$1"
  local destination_runtime="$2"
  local label="$3"
  local original_json="$TMP_ROOT/${label}-original.json"
  local patched_json="$TMP_ROOT/${label}-patched.json"
  local original_canonical="$TMP_ROOT/${label}-original.canonical.json"
  local patched_canonical="$TMP_ROOT/${label}-patched.canonical.json"
  local original_init="$TMP_ROOT/${label}-original.init.json"
  local patched_init="$TMP_ROOT/${label}-patched.init.json"
  local statefulset="$destination_runtime/statefulset.yaml"

  [[ ! -e "$destination_runtime" && ! -L "$destination_runtime" ]] ||
    die "runtime copy destination already exists: $destination_runtime"
  cp -R "$source_runtime" "$destination_runtime"
  cp "$destination_runtime/statefulset.yaml" "$TMP_ROOT/${label}-unpatched.yaml"
  patch_statefulset_image "$statefulset" "$label"

  yaml_json "$TMP_ROOT/${label}-unpatched.yaml" "$original_json"
  yaml_json "$statefulset" "$patched_json"
  jq -S 'del(.spec.template.spec.containers[0].image)' "$original_json" >"$original_canonical"
  jq -S 'del(.spec.template.spec.containers[0].image)' "$patched_json" >"$patched_canonical"
  jq -S '.spec.template.spec.initContainers // []' "$original_json" >"$original_init"
  jq -S '.spec.template.spec.initContainers // []' "$patched_json" >"$patched_init"
  cmp -s "$original_canonical" "$patched_canonical" ||
    die "digest patch changed fields other than the serving image"
  cmp -s "$original_init" "$patched_init" ||
    die "digest patch changed init containers"
  jq -e \
    '.spec.template.spec.containers | length == 1 and .[0].name == "serving" and .[0].image == "ghcr.io/chrischeng-c4/lumen:0.4.29"' \
    "$original_json" >/dev/null || die "unpatched runtime is not the fixed serving image"
  jq -e \
    --arg image "$LUMEN_STANDALONE_GKE_IMAGE" \
    '.spec.template.spec.containers | length == 1 and .[0].name == "serving" and .[0].image == $image' \
    "$patched_json" >/dev/null || die "patched runtime does not use the exact candidate digest"
  jq -e '((.spec.template.spec.initContainers // []) | length == 0)' "$patched_json" >/dev/null ||
    die "rendered runtime unexpectedly contains init containers"
}

PATCHED_RUNTIME="$TMP_ROOT/runtime-patched"
prepare_runtime_copy "$RENDERED/runtime" "$PATCHED_RUNTIME" initial

# GKE_LIVE_GATE_V2_BEGIN
#
# The original Slice A preflight above is intentionally kept read-only. The
# live path starts here. It never uses the host as a service client and it
# owns exactly the run-scoped namespaces plus one rendered ClusterRoleBinding.
V2_RUNTIME_NAMESPACE="$RUNTIME_NAMESPACE"
V2_CLIENT_NAMESPACE="$CLIENT_NAMESPACE"
V2_CRB="lumen.${V2_RUNTIME_NAMESPACE}.lumen.auth-delegator"
V2_RUN_LABEL="lumen.axiom.dev/gke-acceptance-run"
V2_RUN_HASH="$(printf '%s' "$LUMEN_STANDALONE_GKE_RUN_ID" | sha256_stdin | cut -c1-12)"
V2_RUNTIME_ARMED=false
V2_CLIENT_ARMED=false
V2_CRB_ARMED=false
V2_CLEAN=false
V2_LIVE=false
V2_REQUIRED=false
V2_RECEIPT=false
V2_PVC_UID=""
V2_PV_NAME=""
V2_RUNTIME_NAMESPACE_UID=""
V2_CLIENT_NAMESPACE_UID=""
V2_CRB_UID=""
V2_CHILD_DIGEST=""
V2_NODE_ARCH=""
V2_OBSERVED_RUNTIME_IMAGE_DIGEST=""
V2_LAST_POD_UID=""
V2_REQUIRED_DELTAS='{}'
RECEIPT_TMP=""
RECEIPT_SIDECAR_TMP=""

v2_get_state() {
  local kind="$1" name="$2" namespace="${3:-}" response="$TMP_ROOT/v2-get-response.json" error="$TMP_ROOT/v2-get-error.txt" status
  : >"$response"
  : >"$error"
  if [[ -n "$namespace" ]]; then
    if k get "$kind" "$name" --namespace "$namespace" --ignore-not-found -o json >"$response" 2>"$error"; then status=0; else status=$?; fi
  else
    if k get "$kind" "$name" --ignore-not-found -o json >"$response" 2>"$error"; then status=0; else status=$?; fi
  fi
  [[ "$status" -eq 0 ]] || return 2
  if [[ ! -s "$response" ]]; then
    return 1
  fi
  jq -e 'type == "object"' "$response" >/dev/null 2>&1 || return 2
  return 0
}

v2_absent() {
  local state
  if [[ -n "${3:-}" ]]; then
    if v2_get_state "$1" "$2" "$3"; then return 1; else state=$?; fi
  else
    if v2_get_state "$1" "$2"; then return 1; else state=$?; fi
  fi
  [[ "$state" -eq 1 ]] && return 0
  return 2
}

v2_wait_namespace_gone() {
  local namespace="$1" deadline=$((SECONDS + 180)) state
  while :; do
    if v2_get_state namespace "$namespace"; then
      state=0
    else
      state=$?
    fi
    case "$state" in
      1) return 0 ;;
      0) (( SECONDS < deadline )) || return 1; sleep 2 ;;
      *) return 2 ;;
    esac
  done
}

v2_wait_pv_gone() {
  local pv="$1" deadline=$((SECONDS + 300)) state
  while :; do
    if v2_get_state pv "$pv"; then
      state=0
    else
      state=$?
    fi
    case "$state" in
      1) return 0 ;;
      0) (( SECONDS < deadline )) || return 1; sleep 2 ;;
      *) return 2 ;;
    esac
  done
}

v2_crb_owned() {
  [[ -n "$V2_CRB_UID" ]] || return 1
  k get clusterrolebinding "$V2_CRB" -o json >"$TMP_ROOT/v2-crb.json" || return 1
  jq -e --arg uid "$V2_CRB_UID" --arg name "$V2_CRB" --arg ns "$V2_RUNTIME_NAMESPACE" --arg run "$LUMEN_STANDALONE_GKE_RUN_ID" '
    .metadata.uid == $uid and
    .metadata.name == $name and
    .metadata.labels["app.kubernetes.io/managed-by"] == "lumen-standalone-gke-acceptance" and
    .metadata.labels["lumen.axiom.dev/owner-namespace"] == $ns and
    .metadata.labels["lumen.axiom.dev/gke-acceptance-run"] == $run and
    .roleRef == {apiGroup:"rbac.authorization.k8s.io",kind:"ClusterRole",name:"system:auth-delegator"} and
    .subjects == [{kind:"ServiceAccount",name:"lumen",namespace:$ns}]
  ' "$TMP_ROOT/v2-crb.json" >/dev/null
}

v2_assert_crb() {
  v2_crb_owned || die "runtime auth-delegator ClusterRoleBinding is not exact"
}

v2_client_namespace_owned() {
  [[ -n "$V2_CLIENT_NAMESPACE_UID" ]] || return 1
  k get namespace "$V2_CLIENT_NAMESPACE" -o json >"$TMP_ROOT/v2-client-namespace.json" || return 1
  jq -e --arg uid "$V2_CLIENT_NAMESPACE_UID" --arg ns "$V2_CLIENT_NAMESPACE" --arg run "$LUMEN_STANDALONE_GKE_RUN_ID" '
    .metadata.uid == $uid and .metadata.name == $ns and
    .metadata.labels["app.kubernetes.io/managed-by"] == "lumen-standalone-gke-acceptance" and
    .metadata.labels["lumen.axiom.dev/gke-acceptance-run"] == $run
  ' "$TMP_ROOT/v2-client-namespace.json" >/dev/null
}

v2_assert_client_namespace() {
  v2_client_namespace_owned || die "client namespace ownership is not exact"
}

v2_runtime_namespace_owned() {
  [[ -n "$V2_RUNTIME_NAMESPACE_UID" ]] || return 1
  k get namespace "$V2_RUNTIME_NAMESPACE" -o json >"$TMP_ROOT/v2-runtime-namespace.json" || return 1
  jq -e --arg uid "$V2_RUNTIME_NAMESPACE_UID" --arg ns "$V2_RUNTIME_NAMESPACE" --arg run "$LUMEN_STANDALONE_GKE_RUN_ID" '
    .metadata.uid == $uid and .metadata.name == $ns and
    .metadata.labels["app.kubernetes.io/managed-by"] == "lumen-standalone-gke-acceptance" and
    .metadata.labels["lumen.axiom.dev/gke-acceptance-run"] == $run
  ' "$TMP_ROOT/v2-runtime-namespace.json" >/dev/null
}

v2_assert_runtime_namespace() {
  v2_runtime_namespace_owned || die "runtime namespace ownership is not exact"
}

v2_recover_created_uids() {
  local recovered
  if [[ -z "$V2_RUNTIME_NAMESPACE_UID" && -f "$TMP_ROOT/v2-runtime-namespace-create.json" ]]; then
    if recovered="$(jq -er --arg name "$V2_RUNTIME_NAMESPACE" 'select(.metadata.name == $name) | .metadata.uid | select(type == "string" and length > 0)' "$TMP_ROOT/v2-runtime-namespace-create.json")"; then
      V2_RUNTIME_NAMESPACE_UID="$recovered"
    fi
  fi
  if [[ -z "$V2_CRB_UID" && -f "$TMP_ROOT/v2-crb-create.json" ]]; then
    if recovered="$(jq -er --arg name "$V2_CRB" 'select(.metadata.name == $name) | .metadata.uid | select(type == "string" and length > 0)' "$TMP_ROOT/v2-crb-create.json")"; then
      V2_CRB_UID="$recovered"
    fi
  fi
  if [[ -z "$V2_CLIENT_NAMESPACE_UID" && -f "$TMP_ROOT/v2-client-namespace-create.json" ]]; then
    if recovered="$(jq -er --arg name "$V2_CLIENT_NAMESPACE" 'select(.metadata.name == $name) | .metadata.uid | select(type == "string" and length > 0)' "$TMP_ROOT/v2-client-namespace-create.json")"; then
      V2_CLIENT_NAMESPACE_UID="$recovered"
    fi
  fi
}

v2_cleanup() {
  local state runtime_safe
  set +e
  v2_recover_created_uids
  if [[ "$V2_CRB_ARMED" == true ]]; then
    if v2_get_state clusterrolebinding "$V2_CRB"; then state=0; else state=$?; fi
    case "$state" in
      1) ;;
      0) if v2_crb_owned; then k delete clusterrolebinding "$V2_CRB" --wait=true >/dev/null 2>&1 || V2_CLEAN=false; else V2_CLEAN=false; fi ;;
      *) V2_CLEAN=false ;;
    esac
  fi
  if [[ "$V2_CLIENT_ARMED" == true ]]; then
    if v2_get_state namespace "$V2_CLIENT_NAMESPACE"; then state=0; else state=$?; fi
    case "$state" in
      1) ;;
      0) if v2_client_namespace_owned; then
           k delete namespace "$V2_CLIENT_NAMESPACE" --wait=false >/dev/null 2>&1 || V2_CLEAN=false
           v2_wait_namespace_gone "$V2_CLIENT_NAMESPACE" || V2_CLEAN=false
         else
           V2_CLEAN=false
         fi ;;
      *) V2_CLEAN=false ;;
    esac
  fi
  if [[ "$V2_RUNTIME_ARMED" == true ]]; then
    if v2_get_state namespace "$V2_RUNTIME_NAMESPACE"; then state=0; else state=$?; fi
    case "$state" in
      1) ;;
      0) if v2_runtime_namespace_owned; then
           runtime_safe=true
           if [[ -z "$V2_PV_NAME" ]]; then
             if v2_get_state pvc "$PVC_NAME" "$V2_RUNTIME_NAMESPACE"; then
               V2_PV_NAME="$(jq -er '.spec.volumeName // empty' "$TMP_ROOT/v2-get-response.json")" || V2_CLEAN=false
             else
               state=$?
               if [[ "$state" -ne 1 ]]; then V2_CLEAN=false; runtime_safe=false; fi
             fi
           fi
           if [[ "$runtime_safe" == true ]]; then
             k delete namespace "$V2_RUNTIME_NAMESPACE" --wait=false >/dev/null 2>&1 || V2_CLEAN=false
             v2_wait_namespace_gone "$V2_RUNTIME_NAMESPACE" || V2_CLEAN=false
             if [[ -n "$V2_PV_NAME" ]]; then v2_wait_pv_gone "$V2_PV_NAME" || V2_CLEAN=false; fi
           fi
         else
           V2_CLEAN=false
         fi ;;
      *) V2_CLEAN=false ;;
    esac
  fi
  v2_absent namespace "$V2_RUNTIME_NAMESPACE" || V2_CLEAN=false
  v2_absent namespace "$V2_CLIENT_NAMESPACE" || V2_CLEAN=false
  v2_absent clusterrolebinding "$V2_CRB" || V2_CLEAN=false
}

v2_metric_total() {
  local metric="$1" file="$2"
  awk -v metric="$metric" '
    $1 ~ ("^" metric "($|\\{") && $2 ~ /^[0-9]+([.][0-9]+)?$/ { sum += $2; seen = 1 }
    END { if (!seen) exit 1; printf "%.0f", sum }
  ' "$file"
}

v2_metric_deltas() {
  local before="$1" after="$2" tb ta ab aa lb la db da
  tb="$(v2_metric_total delegated_auth_token_reviews_total "$before")" || die "missing delegated_auth_token_reviews_total"
  ta="$(v2_metric_total delegated_auth_token_reviews_total "$after")" || die "missing delegated_auth_token_reviews_total"
  ab="$(v2_metric_total delegated_auth_access_reviews_total "$before")" || die "missing delegated_auth_access_reviews_total"
  aa="$(v2_metric_total delegated_auth_access_reviews_total "$after")" || die "missing delegated_auth_access_reviews_total"
  lb="$(v2_metric_total delegated_auth_allowed_total "$before")" || die "missing delegated_auth_allowed_total"
  la="$(v2_metric_total delegated_auth_allowed_total "$after")" || die "missing delegated_auth_allowed_total"
  db="$(v2_metric_total delegated_auth_denied_total "$before")" || die "missing delegated_auth_denied_total"
  da="$(v2_metric_total delegated_auth_denied_total "$after")" || die "missing delegated_auth_denied_total"
  [[ "$ta" -gt "$tb" && "$aa" -gt "$ab" && "$la" -gt "$lb" && "$da" -gt "$db" ]] || die "measured auth metric deltas are not positive"
  jq -n --argjson tokenreview_delta "$((ta-tb))" --argjson subjectaccessreview_delta "$((aa-ab))" --argjson allowed_delta "$((la-lb))" --argjson denied_delta "$((da-db))" \
    '{tokenreview_delta:$tokenreview_delta,subjectaccessreview_delta:$subjectaccessreview_delta,allowed_delta:$allowed_delta,denied_delta:$denied_delta}'
}

v2_job_name() {
  [[ "$1" =~ ^[a-z0-9-]{1,40}$ ]] || die "unsafe client job label"
  printf 'lumen-gke-%s-%s' "$V2_RUN_HASH" "$1"
}

v2_read_job_log() {
  local job="$1" log="$2" attempt
  local error="$TMP_ROOT/$job.logs.err"
  for ((attempt = 1; attempt <= 6; attempt++)); do
    if k logs "job/$job" --namespace "$V2_CLIENT_NAMESPACE" --request-timeout=10s >"$log" 2>"$error"; then
      return 0
    fi
    grep -Fq -- 'No agent available' "$error" || die "job log read failed"
    [[ "$attempt" -lt 6 ]] || die "Konnectivity agent unavailable while reading job log"
    sleep 5
  done
  die "Konnectivity agent unavailable while reading job log"
}

v2_run_client_tooling_job() {
  local job render_dir log
  job="$(v2_job_name client-tools)"
  render_dir="$TMP_ROOT/${job}-render"
  [[ ! -e "$render_dir" && ! -L "$render_dir" ]] ||
    die "shared tooling renderer output directory already exists"
  "$KUSTOMIZE_RENDERER" tooling \
    --out-dir "$render_dir" \
    --client-namespace "$V2_CLIENT_NAMESPACE" \
    --run-id "$LUMEN_STANDALONE_GKE_RUN_ID" \
    --job "$job"
  [[ -f "$render_dir/rendered.yaml" && ! -L "$render_dir/rendered.yaml" ]] ||
    die "shared tooling renderer output is missing"
  ruby "$KUSTOMIZE_VALIDATOR" tooling \
    --file "$render_dir/rendered.yaml" \
    --client-namespace "$V2_CLIENT_NAMESPACE" \
    --run-id "$LUMEN_STANDALONE_GKE_RUN_ID" \
    --job "$job" >/dev/null
  k apply -f "$render_dir/rendered.yaml" >/dev/null
  k wait --for=condition=complete "job/$job" --namespace "$V2_CLIENT_NAMESPACE" --timeout=150s >/dev/null
  log="$TMP_ROOT/${job}.log"
  v2_read_job_log "$job" "$log"
  grep -Fx 'row=client-tools status=passed' "$log" >/dev/null || die "client image tooling contract failed"
  [[ "$(wc -l < "$log" | tr -d ' ')" == 1 ]] || die "client tooling job emitted extra output"
}

v2_assert_api_job_log() {
  local label="$1" expected="$2" log="$3"
  [[ "$label" =~ ^[a-z0-9-]{1,40}$ ]] || die "unsafe client job label"
  if [[ "$expected" == 2xx ]]; then
    grep -Ex "row=$label status=2[0-9][0-9]" "$log" >/dev/null || die "client job did not prove expected status"
  else
    grep -Fx "row=$label status=$expected" "$log" >/dev/null || die "client job did not prove expected status"
  fi
  [[ "$(wc -l < "$log" | tr -d ' ')" == 1 ]] || die "client job retained request or response output"
}

v2_run_api_job() {
  local label="$1" account="$2" token_mode="$3" method="$4" path="$5" body="$6" expected="$7" need_id="$8" reject_id="$9"
  local job render_dir request_file log
  job="$(v2_job_name "$label")"
  [[ "${#job}" -le 63 ]] || die "client Job name exceeds 63 characters"
  render_dir="$TMP_ROOT/${job}-render"
  request_file="$TMP_ROOT/${job}.request.json"
  [[ ! -e "$render_dir" && ! -L "$render_dir" ]] ||
    die "shared API renderer output directory already exists"
  [[ ! -e "$request_file" && ! -L "$request_file" ]] || die "API request file already exists"
  printf '%s' "$body" >"$request_file"
  [[ -f "$request_file" && ! -L "$request_file" ]] ||
    die "API request file was not created as a regular file"
  "$KUSTOMIZE_RENDERER" api \
    --out-dir "$render_dir" \
    --client-namespace "$V2_CLIENT_NAMESPACE" \
    --runtime-namespace "$V2_RUNTIME_NAMESPACE" \
    --service lumen \
    --run-id "$LUMEN_STANDALONE_GKE_RUN_ID" \
    --job "$job" \
    --account "$account" \
    --token-mode "$token_mode" \
    --method "$method" \
    --path "$path" \
    --request-file "$request_file" \
    --expected-status "$expected" \
    --required-id "$need_id" \
    --rejected-id "$reject_id" \
    --row-label "$label"
  [[ -f "$render_dir/rendered.yaml" && ! -L "$render_dir/rendered.yaml" ]] ||
    die "shared API renderer output is missing"
  ruby "$KUSTOMIZE_VALIDATOR" api \
    --file "$render_dir/rendered.yaml" \
    --client-namespace "$V2_CLIENT_NAMESPACE" \
    --run-id "$LUMEN_STANDALONE_GKE_RUN_ID" \
    --job "$job" \
    --account "$account" \
    --token-mode "$token_mode" \
    --runtime-namespace "$V2_RUNTIME_NAMESPACE" \
    --service lumen \
    --method "$method" \
    --path "$path" \
    --request-file "$request_file" \
    --expected-status "$expected" \
    --required-id "$need_id" \
    --rejected-id "$reject_id" \
    --row-label "$label" >/dev/null
  k apply -f "$render_dir/rendered.yaml" >/dev/null
  k wait --for=condition=complete "job/$job" --namespace "$V2_CLIENT_NAMESPACE" --timeout=150s >/dev/null
  log="$TMP_ROOT/${job}.log"
  v2_read_job_log "$job" "$log"
  v2_assert_api_job_log "$label" "$expected" "$log"
}

v2_run_metrics_job() {
  local label="$1" job render_dir log
  job="$(v2_job_name "$label")"
  render_dir="$TMP_ROOT/${job}-render"
  [[ ! -e "$render_dir" && ! -L "$render_dir" ]] ||
    die "shared metrics renderer output directory already exists"
  "$KUSTOMIZE_RENDERER" metrics \
    --out-dir "$render_dir" \
    --client-namespace "$V2_CLIENT_NAMESPACE" \
    --runtime-namespace "$V2_RUNTIME_NAMESPACE" \
    --service lumen \
    --run-id "$LUMEN_STANDALONE_GKE_RUN_ID" \
    --job "$job" \
    --row-label "$label"
  [[ -f "$render_dir/rendered.yaml" && ! -L "$render_dir/rendered.yaml" ]] ||
    die "shared metrics renderer output is missing"
  ruby "$KUSTOMIZE_VALIDATOR" metrics \
    --file "$render_dir/rendered.yaml" \
    --client-namespace "$V2_CLIENT_NAMESPACE" \
    --run-id "$LUMEN_STANDALONE_GKE_RUN_ID" \
    --job "$job" \
    --runtime-namespace "$V2_RUNTIME_NAMESPACE" \
    --service lumen \
    --row-label "$label" >/dev/null
  k apply -f "$render_dir/rendered.yaml" >/dev/null
  k wait --for=condition=complete "job/$job" --namespace "$V2_CLIENT_NAMESPACE" --timeout=150s >/dev/null
  log="$TMP_ROOT/${job}.log"
  v2_read_job_log "$job" "$log"
  grep -Fx "row=$label status=200" "$log" >/dev/null || die "metrics job did not return 200"
  sed '1d' "$log" >"$TMP_ROOT/${label}.metrics"
  [[ -s "$TMP_ROOT/${label}.metrics" ]] || die "metrics text is missing from private temp"
}

v2_expected_child() {
  local pod_json="$1" node arch
  node="$(jq -er '.spec.nodeName' "$pod_json")"
  arch="$(k get node "$node" -o json | jq -er '.metadata.labels["kubernetes.io/arch"]')"
  V2_NODE_ARCH="$arch"
  case "$arch" in amd64) V2_CHILD_DIGEST="$AMD64_DIGEST" ;; arm64) V2_CHILD_DIGEST="$ARM64_DIGEST" ;; *) die "unsupported scheduled node architecture" ;; esac
}

v2_capture_pvc_identity() {
  V2_PVC_UID="$(k get pvc "$PVC_NAME" --namespace "$V2_RUNTIME_NAMESPACE" -o jsonpath='{.metadata.uid}')"
  V2_PV_NAME="$(k get pvc "$PVC_NAME" --namespace "$V2_RUNTIME_NAMESPACE" -o jsonpath='{.spec.volumeName}')"
  [[ -n "$V2_PVC_UID" && -n "$V2_PV_NAME" ]] || die "PVC did not bind after the initial pod became Ready"
}

v2_wait_pod() {
  local old_uid="$1" auth="$2" cpu="$3" memory="$4" deadline=$((SECONDS + 300)) uid image
  while :; do
    if k get pod lumen-0 --namespace "$V2_RUNTIME_NAMESPACE" -o json >"$TMP_ROOT/v2-pod.json" 2>/dev/null; then
      uid="$(jq -er '.metadata.uid' "$TMP_ROOT/v2-pod.json")"
      if [[ ( -z "$old_uid" || "$uid" != "$old_uid" ) && "$(jq -r '[.status.conditions[]? | select(.type == "Ready") | .status] | first // "False"' "$TMP_ROOT/v2-pod.json")" == True ]]; then
        image="$(jq -er '.status.containerStatuses[]|select(.name == "serving")|.imageID' "$TMP_ROOT/v2-pod.json")"
        v2_expected_child "$TMP_ROOT/v2-pod.json"
        [[ -n "$V2_NODE_ARCH" && -n "$V2_CHILD_DIGEST" ]] || die "scheduled node identity is incomplete"
        if [[ "$image" == "$LUMEN_STANDALONE_GKE_IMAGE" ]]; then
          V2_OBSERVED_RUNTIME_IMAGE_DIGEST="$ROOT_DIGEST"
        elif [[ "$image" == "ghcr.io/chrischeng-c4/lumen@$V2_CHILD_DIGEST" ]]; then
          V2_OBSERVED_RUNTIME_IMAGE_DIGEST="$V2_CHILD_DIGEST"
        else
          die "observed container imageID is not the exact candidate root or scheduled child digest"
        fi
        jq -e --arg image "$LUMEN_STANDALONE_GKE_IMAGE" --arg auth "$auth" --arg cpu "$cpu" --arg memory "$memory" '
          ([.spec.containers[] | select(.name == "serving") | .image] == [$image]) and
          .spec.enableServiceLinks == false and
          ([.spec.containers[] | select(.name == "serving") | .env[] | select(.name == "LUMEN_AUTH") | .value] == [$auth]) and
          ([.spec.containers[] | select(.name == "serving") | .resources.requests.cpu] == [$cpu]) and
          ([.spec.containers[] | select(.name == "serving") | .resources.requests.memory] == [$memory])
        ' "$TMP_ROOT/v2-pod.json" >/dev/null || die "pod auth profile or requested resources are wrong"
        if [[ -n "$V2_PVC_UID" || -n "$V2_PV_NAME" ]]; then
          [[ -n "$V2_PVC_UID" && -n "$V2_PV_NAME" ]] || die "PVC identity was only partially captured"
          [[ "$(k get pvc "$PVC_NAME" --namespace "$V2_RUNTIME_NAMESPACE" -o jsonpath='{.metadata.uid}')" == "$V2_PVC_UID" ]] || die "PVC uid changed after replacement"
          [[ "$(k get pvc "$PVC_NAME" --namespace "$V2_RUNTIME_NAMESPACE" -o jsonpath='{.spec.volumeName}')" == "$V2_PV_NAME" ]] || die "PV identity changed after replacement"
        fi
        V2_LAST_POD_UID="$uid"
        return
      fi
    fi
    (( SECONDS < deadline )) || die "StatefulSet pod did not become Ready with a new uid"
    sleep 2
  done
}

v2_prepare_private_apply() {
  V2_APPLY_ROOT="$TMP_ROOT/v2-apply"
  cp -R "$RENDERED" "$V2_APPLY_ROOT"
  patch_statefulset_image "$V2_APPLY_ROOT/runtime/statefulset.yaml" v2
  k create --dry-run=client --validate=false -f "$RENDERED/runtime/statefulset.yaml" -o json >"$TMP_ROOT/v2-public.json"
  k create --dry-run=client --validate=false -f "$V2_APPLY_ROOT/runtime/statefulset.yaml" -o json >"$TMP_ROOT/v2-private.json"
  jq -e '.spec.template.spec.containers | length == 1 and .[0].name == "serving" and .[0].image == "ghcr.io/chrischeng-c4/lumen:0.4.29"' "$TMP_ROOT/v2-public.json" >/dev/null || die "public runtime is not the fixed 0.4.29 serving image"
  jq -S 'del(.spec.template.spec.containers[0].image)' "$TMP_ROOT/v2-public.json" >"$TMP_ROOT/v2-public-no-image.json"
  jq -S 'del(.spec.template.spec.containers[0].image)' "$TMP_ROOT/v2-private.json" >"$TMP_ROOT/v2-private-no-image.json"
  cmp -s "$TMP_ROOT/v2-public-no-image.json" "$TMP_ROOT/v2-private-no-image.json" || die "private runtime changed fields other than serving image"
  jq -e --arg image "$LUMEN_STANDALONE_GKE_IMAGE" '.spec.template.spec.containers[0].image == $image' "$TMP_ROOT/v2-private.json" >/dev/null || die "private runtime image is not the root digest"
}

v2_stamp_private_ownership() {
  local file next
  for file in "$V2_APPLY_ROOT/storage/namespace.yaml" "$V2_APPLY_ROOT/runtime/clusterrolebinding.yaml"; do
    [[ -f "$file" && ! -L "$file" ]] || die "private acceptance object is missing"
    next="${file}.next"
    k label -f "$file" "$V2_RUN_LABEL=$LUMEN_STANDALONE_GKE_RUN_ID" app.kubernetes.io/managed-by=lumen-standalone-gke-acceptance --overwrite --local -o yaml >"$next"
    mv "$next" "$file"
  done
}

v2_write_client_root() {
  local output="$TMP_ROOT/v2-client" validated
  [[ ! -e "$output" && ! -L "$output" ]] ||
    die "shared client renderer output directory already exists"
  "$KUSTOMIZE_RENDERER" client \
    --out-dir "$output" \
    --client-namespace "$V2_CLIENT_NAMESPACE" \
    --run-id "$LUMEN_STANDALONE_GKE_RUN_ID"
  [[ -f "$output/rendered.yaml" && ! -L "$output/rendered.yaml" ]] ||
    die "shared client renderer output is missing"
  validated="$output/validated.json"
  ruby "$KUSTOMIZE_VALIDATOR" client \
    --file "$output/rendered.yaml" \
    --client-namespace "$V2_CLIENT_NAMESPACE" \
    --run-id "$LUMEN_STANDALONE_GKE_RUN_ID" \
    --emit-json >"$validated"
  [[ -f "$validated" && ! -L "$validated" ]] ||
    die "shared client validator output is missing"
  jq -e 'map(select(.apiVersion == "v1" and .kind == "Namespace")) | length == 1' "$validated" >/dev/null ||
    die "shared client render must contain exactly one Namespace"
  jq -e 'map(select(.apiVersion == "v1" and .kind == "Namespace")) | .[0]' "$validated" >"$output/namespace.json" ||
    die "shared client Namespace could not be extracted"
  [[ -f "$output/namespace.json" && ! -L "$output/namespace.json" ]] ||
    die "shared client Namespace output is missing"
}

v2_apply() {
  v2_absent namespace "$V2_RUNTIME_NAMESPACE" || die "runtime namespace already exists"
  v2_absent namespace "$V2_CLIENT_NAMESPACE" || die "client namespace already exists"
  v2_absent clusterrolebinding "$V2_CRB" || die "rendered ClusterRoleBinding already exists"
  v2_stamp_private_ownership
  V2_RUNTIME_ARMED=true
  k create -f "$V2_APPLY_ROOT/storage/namespace.yaml" -o json >"$TMP_ROOT/v2-runtime-namespace-create.json" || die "runtime namespace create collided or failed"
  jq -e --arg name "$V2_RUNTIME_NAMESPACE" '.metadata.name == $name and (.metadata.uid|type == "string" and length > 0)' "$TMP_ROOT/v2-runtime-namespace-create.json" >/dev/null || die "runtime namespace create response was not exact"
  V2_RUNTIME_NAMESPACE_UID="$(jq -er '.metadata.uid' "$TMP_ROOT/v2-runtime-namespace-create.json")" || die "runtime namespace uid could not be captured"
  v2_assert_runtime_namespace
  V2_CRB_ARMED=true
  k create -f "$V2_APPLY_ROOT/runtime/clusterrolebinding.yaml" -o json >"$TMP_ROOT/v2-crb-create.json" || die "auth-delegator ClusterRoleBinding create collided or failed"
  jq -e --arg name "$V2_CRB" '.metadata.name == $name and (.metadata.uid|type == "string" and length > 0)' "$TMP_ROOT/v2-crb-create.json" >/dev/null || die "ClusterRoleBinding create response was not exact"
  V2_CRB_UID="$(jq -er '.metadata.uid' "$TMP_ROOT/v2-crb-create.json")" || die "ClusterRoleBinding uid could not be captured"
  v2_assert_crb
  V2_CLIENT_ARMED=true
  if ! k create -f "$TMP_ROOT/v2-client/namespace.json" -o json >"$TMP_ROOT/v2-client-namespace-create.json"; then die "client namespace create collided or failed"; fi
  jq -e --arg name "$V2_CLIENT_NAMESPACE" '.metadata.name == $name and (.metadata.uid|type == "string" and length > 0)' "$TMP_ROOT/v2-client-namespace-create.json" >/dev/null || die "client namespace create response was not exact"
  V2_CLIENT_NAMESPACE_UID="$(jq -er '.metadata.uid' "$TMP_ROOT/v2-client-namespace-create.json")" || die "client namespace uid could not be captured"
  v2_assert_client_namespace
  if ! k apply -k "$V2_APPLY_ROOT/storage" >/dev/null; then die "storage apply failed"; fi
  v2_assert_runtime_namespace
  if ! k apply -k "$V2_APPLY_ROOT/runtime" >/dev/null; then
    die "runtime apply failed"
  fi
  v2_assert_crb
  if ! k apply -f "$TMP_ROOT/v2-client/rendered.yaml" >/dev/null; then die "client apply failed"; fi
  v2_assert_client_namespace
}

v2_assert_network() {
  local ingresses gateway_resources gateways
  k get service lumen --namespace "$V2_RUNTIME_NAMESPACE" -o json >"$TMP_ROOT/v2-service.json"
  k get networkpolicy lumen --namespace "$V2_RUNTIME_NAMESPACE" -o json >"$TMP_ROOT/v2-networkpolicy.json"
  jq -e '.spec.type == "ClusterIP" and ((.spec.externalIPs // [])|length == 0) and ((.status.loadBalancer // {})|length == 0)' "$TMP_ROOT/v2-service.json" >/dev/null || die "service is not ClusterIP-only"
  jq -e '(.spec.policyTypes|sort) == ["Egress","Ingress"]' "$TMP_ROOT/v2-networkpolicy.json" >/dev/null || die "NetworkPolicy is incomplete"
  ingresses="$(k get ingress --namespace "$V2_RUNTIME_NAMESPACE" -o name)" || die "Ingress inventory could not be read"
  [[ -z "$ingresses" ]] || die "runtime created Ingress"
  gateway_resources="$(k api-resources --api-group gateway.networking.k8s.io -o name)" || die "Gateway API inventory could not be read"
  if grep -Fxq 'gateways.gateway.networking.k8s.io' <<<"$gateway_resources"; then
    gateways="$(k get gateways.gateway.networking.k8s.io --namespace "$V2_RUNTIME_NAMESPACE" -o name)" || die "Gateway inventory could not be read"
    [[ -z "$gateways" ]] || die "runtime created Gateway"
  fi
}

v2_required_runtime() {
  local live before after
  live="$TMP_ROOT/v2-live-resized-statefulset.json"
  before="$TMP_ROOT/v2-before-required.json"
  after="$TMP_ROOT/v2-after-required.json"
  V2_REQUIRED_STATEFULSET="$TMP_ROOT/v2-required-statefulset.json"
  k get statefulset lumen --namespace "$V2_RUNTIME_NAMESPACE" -o json >"$live"
  jq 'del(.status,.metadata.uid,.metadata.resourceVersion,.metadata.generation,.metadata.creationTimestamp,.metadata.managedFields,.metadata.ownerReferences)' "$live" >"$before"
  k set env -f "$before" LUMEN_AUTH=required --local -o json >"$after"
  jq -S '(.spec.template.spec.containers[0].env |= map(select(.name != "LUMEN_AUTH")))' "$TMP_ROOT/v2-before-required.json" >"$TMP_ROOT/v2-before-required-noauth.json"
  jq -S '(.spec.template.spec.containers[0].env |= map(select(.name != "LUMEN_AUTH")))' "$TMP_ROOT/v2-after-required.json" >"$TMP_ROOT/v2-after-required-noauth.json"
  cmp -s "$TMP_ROOT/v2-before-required-noauth.json" "$TMP_ROOT/v2-after-required-noauth.json" || die "required continuity patch changed live desired fields other than LUMEN_AUTH"
  cp "$after" "$V2_REQUIRED_STATEFULSET"
  jq -e --arg image "$LUMEN_STANDALONE_GKE_IMAGE" '
    .spec.template.spec.containers[0].image == $image and
    ([.spec.template.spec.containers[0].env[]|select(.name == "LUMEN_AUTH")|.value] == ["required"]) and
    ([.spec.template.spec.containers[0].resources.requests.cpu] == ["1"]) and
    ([.spec.template.spec.containers[0].resources.requests.memory] == ["1Gi"])
  ' "$TMP_ROOT/v2-after-required.json" >/dev/null || die "required profile did not preserve exact image and resized resources"
}

v2_write_receipt_body() {
  local receipt="$LUMEN_STANDALONE_GKE_EVIDENCE_DIR/lumen-standalone-gke-receipt.json" receipt_name receipt_bytes
  [[ "$V2_LIVE" == true && "$V2_REQUIRED" == true && "$V2_CLEAN" == true ]] || return 1
  [[ -n "$RECEIPT_TMP" && -n "$RECEIPT_SIDECAR_TMP" ]] || return 1
  [[ -n "$V2_OBSERVED_RUNTIME_IMAGE_DIGEST" && -n "$V2_NODE_ARCH" && -n "$V2_CHILD_DIGEST" ]] || return 1
  rm -f -- "$receipt" "$receipt.sha256"
  jq -n --arg repository "chrischeng-c4/axiom" --arg version "0.4.29" --arg commit "$CANDIDATE_COMMIT" --arg workflow_ref "$CANDIDATE_WORKFLOW" --arg run_id "$CANDIDATE_RUN_ID" --arg run_attempt "$CANDIDATE_ATTEMPT" --arg manifest_sha256 "$RECEIPT_SHA256" --arg root_digest "$ROOT_DIGEST" --arg amd64_digest "$AMD64_DIGEST" --arg arm64_digest "$ARM64_DIGEST" --arg target "$LUMEN_STANDALONE_GKE_CLI_TARGET" --arg cli_sha256 "$CONTROLLER_CLI_SHA256" --arg observed_root "$V2_OBSERVED_RUNTIME_IMAGE_DIGEST" --arg child "$V2_CHILD_DIGEST" --arg arch "$V2_NODE_ARCH" --argjson required_deltas "$V2_REQUIRED_DELTAS" '
    {schema:"lumen.standalone-gke-receipt/v2",stage:"slice-b-live",complete:true,
     candidate:{repository:$repository,version:$version,commit:$commit,workflow_ref:$workflow_ref,run_id:$run_id,run_attempt:$run_attempt,manifest_sha256:$manifest_sha256,root_digest:$root_digest,amd64_digest:$amd64_digest,arm64_digest:$arm64_digest,controller_cli:{target:$target,sha256:$cli_sha256}},
     matrix:{clusterip_only:"passed",network_policy:"passed",allowed_ksa:"passed",unlisted_ksa:"passed",missing_token:"passed",bad_token:"passed",tokenreview:"passed",subjectaccessreview:"passed",application_admin_403:"passed",admin_backup_restore:"passed",pod_replacement:"passed",pvc_recovery:"passed",vertical_resize:"passed",cleanup:"passed",required_continuity:({profile:"LUMEN_AUTH=required",audience:"lumen.axiom.dev",observed_runtime_image_digest:$observed_root,scheduled_node_arch:$arch,scheduled_runtime_child_digest:$child,projected_allowed_2xx:"passed",same_ksa_default_token_401:"passed",projected_unlisted_403:"passed"} + $required_deltas)},
     redaction:{kubeconfig_retained:false,token_retained:false,authorization_retained:false,secret_retained:false,cluster_identity_retained:false,command_output_retained:false,canary_scan:true}}
  ' >"$RECEIPT_TMP"
  receipt_bytes="$(wc -c < "$RECEIPT_TMP" | tr -d ' ')"
  [[ "$receipt_bytes" -gt 0 && "$receipt_bytes" -le 16384 ]] || die "receipt bytes must be within the 16KiB workflow transport limit"
  [[ "$(jq -c 'keys|sort' "$RECEIPT_TMP")" == '["candidate","complete","matrix","redaction","schema","stage"]' ]] || die "receipt has unexpected keys"
  [[ "$(jq -c '.candidate|keys|sort' "$RECEIPT_TMP")" == '["amd64_digest","arm64_digest","commit","controller_cli","manifest_sha256","repository","root_digest","run_attempt","run_id","version","workflow_ref"]' ]] || die "receipt candidate has unexpected keys"
  [[ "$(jq -c '.candidate.controller_cli|keys|sort' "$RECEIPT_TMP")" == '["sha256","target"]' ]] || die "receipt controller CLI has unexpected keys"
  [[ "$(jq -c '.matrix|keys|sort' "$RECEIPT_TMP")" == '["admin_backup_restore","allowed_ksa","application_admin_403","bad_token","cleanup","clusterip_only","missing_token","network_policy","pod_replacement","pvc_recovery","required_continuity","subjectaccessreview","tokenreview","unlisted_ksa","vertical_resize"]' ]] || die "receipt matrix has unexpected keys"
  [[ "$(jq -c '.matrix.required_continuity|keys|sort' "$RECEIPT_TMP")" == '["allowed_delta","audience","denied_delta","observed_runtime_image_digest","profile","projected_allowed_2xx","projected_unlisted_403","same_ksa_default_token_401","scheduled_node_arch","scheduled_runtime_child_digest","subjectaccessreview_delta","tokenreview_delta"]' ]] || die "receipt required continuity has unexpected keys"
  [[ "$(jq -c '.redaction|keys|sort' "$RECEIPT_TMP")" == '["authorization_retained","canary_scan","cluster_identity_retained","command_output_retained","kubeconfig_retained","secret_retained","token_retained"]' ]] || die "receipt redaction has unexpected keys"
  jq -e '
    .schema == "lumen.standalone-gke-receipt/v2" and .stage == "slice-b-live" and .complete == true and
    ([.matrix|to_entries[]|select(.key != "required_continuity")|.value] | all(.[]; . == "passed")) and
    .matrix.required_continuity.profile == "LUMEN_AUTH=required" and
    .matrix.required_continuity.audience == "lumen.axiom.dev" and
    .matrix.required_continuity.projected_allowed_2xx == "passed" and
    .matrix.required_continuity.same_ksa_default_token_401 == "passed" and
    .matrix.required_continuity.projected_unlisted_403 == "passed" and
    .redaction == {authorization_retained:false,canary_scan:true,cluster_identity_retained:false,command_output_retained:false,kubeconfig_retained:false,secret_retained:false,token_retained:false} and
    .candidate.amd64_digest != .candidate.arm64_digest and
    ((.matrix.required_continuity.scheduled_node_arch == "amd64" and .matrix.required_continuity.scheduled_runtime_child_digest == .candidate.amd64_digest and (.matrix.required_continuity.observed_runtime_image_digest == .candidate.root_digest or .matrix.required_continuity.observed_runtime_image_digest == .candidate.amd64_digest)) or (.matrix.required_continuity.scheduled_node_arch == "arm64" and .matrix.required_continuity.scheduled_runtime_child_digest == .candidate.arm64_digest and (.matrix.required_continuity.observed_runtime_image_digest == .candidate.root_digest or .matrix.required_continuity.observed_runtime_image_digest == .candidate.arm64_digest))) and
    ([.matrix.required_continuity.tokenreview_delta,.matrix.required_continuity.subjectaccessreview_delta,.matrix.required_continuity.allowed_delta,.matrix.required_continuity.denied_delta] | all(.[]; type == "number" and floor == . and . > 0))
  ' "$RECEIPT_TMP" >/dev/null || die "receipt required continuity has invalid runtime identity or non-positive deltas"
  if grep -Eiq 'Bearer[[:space:]]|eyJ[A-Za-z0-9_-]{20,}|-----BEGIN|lumen-standalone|gke-acceptance' "$RECEIPT_TMP"; then die "receipt redaction scan failed"; fi
  printf '%s  %s\n' "$(sha256_file "$RECEIPT_TMP")" "${receipt##*/}" >"$RECEIPT_SIDECAR_TMP"
  receipt_name="${receipt##*/}"
  checksum_sidecar_exact "$RECEIPT_SIDECAR_TMP" "$(sha256_file "$RECEIPT_TMP")" "$receipt_name"
  [[ -f "$RECEIPT_TMP" && ! -L "$RECEIPT_TMP" ]] || die "receipt temporary file is missing"
  [[ -f "$RECEIPT_SIDECAR_TMP" && ! -L "$RECEIPT_SIDECAR_TMP" ]] || die "receipt sidecar temporary file is missing"
  mv -f -- "$RECEIPT_SIDECAR_TMP" "$receipt.sha256" || die "receipt sidecar commit failed"
  mv -f -- "$RECEIPT_TMP" "$receipt" || die "receipt commit failed"
  [[ -f "$receipt.sha256" && ! -L "$receipt.sha256" && -f "$receipt" && ! -L "$receipt" ]] || die "receipt atomic commit is incomplete"
  [[ "$(find "$LUMEN_STANDALONE_GKE_EVIDENCE_DIR" -mindepth 1 -maxdepth 1 ! -type f -print -quit)" == "" ]] || die "evidence output has an unexpected non-file"
  [[ "$(find "$LUMEN_STANDALONE_GKE_EVIDENCE_DIR" -mindepth 1 -maxdepth 1 -type f ! -name "$receipt_name" ! -name "${receipt_name}.sha256" -print -quit)" == "" ]] || die "evidence output has unexpected files"
}

v2_write_receipt() {
  RECEIPT_TMP="$(mktemp "$LUMEN_STANDALONE_GKE_EVIDENCE_DIR/.receipt.XXXXXX")" || return 1
  if ! RECEIPT_SIDECAR_TMP="$(mktemp "$LUMEN_STANDALONE_GKE_EVIDENCE_DIR/.receipt-sidecar.XXXXXX")"; then
    rm -f -- "$RECEIPT_TMP"
    RECEIPT_TMP=""
    return 1
  fi
  if ! (v2_write_receipt_body); then
    rm -f -- "$LUMEN_STANDALONE_GKE_EVIDENCE_DIR/lumen-standalone-gke-receipt.json" "$LUMEN_STANDALONE_GKE_EVIDENCE_DIR/lumen-standalone-gke-receipt.json.sha256" "$RECEIPT_TMP" "$RECEIPT_SIDECAR_TMP"
    RECEIPT_TMP=""
    RECEIPT_SIDECAR_TMP=""
    return 1
  fi
  RECEIPT_TMP=""
  RECEIPT_SIDECAR_TMP=""
  V2_RECEIPT=true
}

# Redefine the EXIT hook before any live mutation. The trap resolves this
# function at exit, so every fail path below performs the same exact cleanup.
cleanup() {
  local status=$?
  set +e
  trap - EXIT INT TERM
  V2_CLEAN=true
  v2_cleanup
  if [[ -n "$TMP_ROOT" && -d "$TMP_ROOT" ]]; then rm -rf -- "$TMP_ROOT" || V2_CLEAN=false; fi
  if [[ "$status" -eq 0 && "$V2_CLEAN" == true ]]; then v2_write_receipt || status=1; fi
  if [[ "$V2_RECEIPT" != true ]]; then
    rm -f -- "$LUMEN_STANDALONE_GKE_EVIDENCE_DIR/lumen-standalone-gke-receipt.json" "$LUMEN_STANDALONE_GKE_EVIDENCE_DIR/lumen-standalone-gke-receipt.json.sha256" "$RECEIPT_TMP" "$RECEIPT_SIDECAR_TMP"
  fi
  [[ "$V2_RECEIPT" == true ]] || status=1
  exit "$status"
}

run_live_acceptance_v2() {
  local initial_uid replacement_uid resized_uid required_uid
  v2_prepare_private_apply
  v2_write_client_root
  v2_apply
  v2_run_client_tooling_job
  v2_wait_pod '' in-cluster 500m 512Mi
  v2_capture_pvc_identity
  initial_uid="$V2_LAST_POD_UID"
  v2_assert_network
  v2_run_metrics_job metrics-before-incluster
  v2_run_api_job create app default PUT /collections/gke '{"fields":{"tag":{"type":"keyword"}}}' 2xx none none
  v2_run_api_job index-first app default POST /collections/gke/index '{"items":[{"external_id":"durable-first","field":"tag","value":"first"}]}' 2xx none none
  v2_run_api_job search-first app default POST /collections/gke/search '{"query":{"term":{"field":"tag","value":"first"}},"limit":10}' 2xx durable-first none
  v2_run_api_job unlisted unlisted default POST /collections/gke/search '{"query":{"term":{"field":"tag","value":"first"}},"limit":10}' 403 none none
  v2_run_api_job missing default missing POST /collections/gke/search '{"query":{"term":{"field":"tag","value":"first"}},"limit":10}' 401 none none
  v2_run_api_job bad unlisted bad POST /collections/gke/search '{"query":{"term":{"field":"tag","value":"first"}},"limit":10}' 401 none none
  v2_run_api_job application-admin app default GET /admin/backup '' 403 none none
  "$VERIFIED_CLI" standalone backup --gke "$CONFIG" --out "$TMP_ROOT/v2-backup.json" >/dev/null
  [[ -s "$TMP_ROOT/v2-backup.json" ]] || die "formal admin backup did not create a snapshot"
  sha256_file "$TMP_ROOT/v2-backup.json" >"$TMP_ROOT/v2-backup.sha256"
  v2_run_api_job index-after-backup app default POST /collections/gke/index '{"items":[{"external_id":"after-backup","field":"tag","value":"after"}]}' 2xx none none
  v2_run_api_job see-after-backup app default POST /collections/gke/search '{"query":{"term":{"field":"tag","value":"after"}},"limit":10}' 2xx after-backup none
  "$VERIFIED_CLI" standalone restore --gke "$CONFIG" --file "$TMP_ROOT/v2-backup.json" --replace >/dev/null
  v2_run_api_job restore-first app default POST /collections/gke/search '{"query":{"term":{"field":"tag","value":"first"}},"limit":10}' 2xx durable-first none
  v2_run_api_job restore-no-after app default POST /collections/gke/search '{"query":{"term":{"field":"tag","value":"after"}},"limit":10}' 2xx none after-backup
  k delete pod lumen-0 --namespace "$V2_RUNTIME_NAMESPACE" --wait=true >/dev/null
  v2_wait_pod "$initial_uid" in-cluster 500m 512Mi
  replacement_uid="$V2_LAST_POD_UID"
  v2_run_api_job marker-after-replace app default POST /collections/gke/search '{"query":{"term":{"field":"tag","value":"first"}},"limit":10}' 2xx durable-first none
  k patch statefulset lumen --namespace "$V2_RUNTIME_NAMESPACE" --type=json -p='[{"op":"replace","path":"/spec/template/spec/containers/0/resources/requests/cpu","value":"1"},{"op":"replace","path":"/spec/template/spec/containers/0/resources/requests/memory","value":"1Gi"}]' >/dev/null
  v2_wait_pod "$replacement_uid" in-cluster 1 1Gi
  resized_uid="$V2_LAST_POD_UID"
  v2_run_api_job marker-after-resize app default POST /collections/gke/search '{"query":{"term":{"field":"tag","value":"first"}},"limit":10}' 2xx durable-first none
  v2_run_metrics_job metrics-after-incluster
  v2_metric_deltas "$TMP_ROOT/metrics-before-incluster.metrics" "$TMP_ROOT/metrics-after-incluster.metrics" >/dev/null
  v2_required_runtime
  k apply -f "$V2_REQUIRED_STATEFULSET" >/dev/null
  v2_wait_pod "$resized_uid" required 1 1Gi
  required_uid="$V2_LAST_POD_UID"
  [[ "$required_uid" != "$resized_uid" ]] || die "required profile did not replace pod"
  v2_run_metrics_job metrics-before-required
  v2_run_api_job required-projected-app app projected POST /collections/gke/search '{"query":{"term":{"field":"tag","value":"first"}},"limit":10}' 2xx durable-first none
  v2_run_api_job required-default-app app default POST /collections/gke/search '{"query":{"term":{"field":"tag","value":"first"}},"limit":10}' 401 none none
  v2_run_api_job required-projected-unlisted unlisted projected POST /collections/gke/search '{"query":{"term":{"field":"tag","value":"first"}},"limit":10}' 403 none none
  v2_run_metrics_job metrics-after-required
  V2_REQUIRED_DELTAS="$(v2_metric_deltas "$TMP_ROOT/metrics-before-required.metrics" "$TMP_ROOT/metrics-after-required.metrics")"
  V2_LIVE=true
  V2_REQUIRED=true
}

run_live_acceptance() {
  run_live_acceptance_v2
}

run_live_acceptance

exit 0
