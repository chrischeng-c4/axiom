#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ACCEPTANCE_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
source "$ACCEPTANCE_ROOT/scripts/kubernetes-ownership.sh"

test_root="$(mktemp -d "${TMPDIR:-/tmp}/sift-kubernetes-ownership.XXXXXX")"
fake_bin="$test_root/bin"
state_dir="$test_root/state"
receipt_root="$test_root/evidence/kubernetes/ownership"
calls="$test_root/kubectl.log"
cleanup_test() {
  find "$test_root" -depth -delete >/dev/null 2>&1 || true
}
trap cleanup_test EXIT INT TERM
mkdir -p "$fake_bin" "$state_dir" "$receipt_root"
: > "$calls"

project_id="axiom-test"
run_id="ownership"
acquisition_id="0123456789abcdef0123456789abcdef"

cat > "$fake_bin/kubectl" <<'EOF'
#!/usr/bin/env bash
set -euo pipefail
printf '%s\n' "$*" >> "${SIFT_KUBE_OWNERSHIP_CALLS:?}"
if [[ " $* " == *" delete clusterrolebinding -l app.kubernetes.io/component=auth-delegation,app.kubernetes.io/name=lumen "* ]]; then
  exit 0
fi
state="${SIFT_KUBE_OWNERSHIP_STATE:?}"
resource_file() {
  local resource_type="$1"
  local name="$2"
  printf '%s/%s-%s.json\n' "$state" "$resource_type" "$name"
}
case "${1:-}" in
  get)
    resource_type="${2:?}"
    name="${3:?}"
    if [[ "${SIFT_KUBE_GET_DENIED:-}" == "$resource_type/$name" ]]; then
      echo "Forbidden" >&2
      exit 41
    fi
    file="$(resource_file "$resource_type" "$name")"
    if [[ "${SIFT_KUBE_RECREATE_AFTER_DELETE:-}" == "$resource_type/$name" \
        && -e "$state/deleted-${resource_type}-${name}" \
        && ! -e "$state/recreated-${resource_type}-${name}" ]]; then
      : > "$state/recreated-${resource_type}-${name}"
      jq -n \
        --arg owner "${SIFT_KUBE_OWNER:?}" \
        --arg project_id "${SIFT_KUBE_PROJECT:?}" \
        --arg run_id "${SIFT_KUBE_RUN:?}" \
        --arg acquisition_id "${SIFT_KUBE_ACQUISITION:?}" \
        --arg name "$name" '
          {
            apiVersion:"v1",kind:"Namespace",
            metadata:{name:$name,uid:("replacement-" + $name),resourceVersion:"99",
              labels:{
                "axiom.axiom.dev/acceptance-owner":$owner,
                "axiom.axiom.dev/acceptance-project":$project_id,
                "axiom.axiom.dev/acceptance-run-id":$run_id,
                "axiom.axiom.dev/acceptance-acquisition-id":$acquisition_id
              }}
          }
        ' > "$file"
    fi
    if [[ "${SIFT_KUBE_LATE_CREATE:-}" == "$resource_type/$name" \
        && ! -e "$state/late-create-fired" ]]; then
      : > "$state/late-create-fired"
      jq -n \
        --arg owner "${SIFT_KUBE_OWNER:?}" \
        --arg project_id "${SIFT_KUBE_PROJECT:?}" \
        --arg run_id "${SIFT_KUBE_RUN:?}" \
        --arg acquisition_id "${SIFT_KUBE_ACQUISITION:?}" \
        --arg name "$name" '
          {
            apiVersion:"v1",
            kind:"Namespace",
            metadata:{
              name:$name,
              uid:("uid-late-" + $name),
              resourceVersion:"9",
              labels:{
                "axiom.axiom.dev/acceptance-owner":$owner,
                "axiom.axiom.dev/acceptance-project":$project_id,
                "axiom.axiom.dev/acceptance-run-id":$run_id,
                "axiom.axiom.dev/acceptance-acquisition-id":$acquisition_id
              }
            }
          }
        ' > "$file"
      echo "NotFound" >&2
      exit 1
    fi
    if [[ -f "$file" ]]; then
      cat "$file"
    else
      echo "NotFound" >&2
      exit 1
    fi
    ;;
  label)
    [[ "${2:-}" == "--local" && "${3:-}" == "--overwrite" \
      && "${4:-}" == "-f" && "${6:-}" == axiom.axiom.dev/acceptance-owner=* ]]
    manifest="${5:?}"
    jq \
      --arg owner "${SIFT_KUBE_OWNER:?}" \
      --arg project_id "${SIFT_KUBE_PROJECT:?}" \
      --arg run_id "${SIFT_KUBE_RUN:?}" \
      --arg acquisition_id "${SIFT_KUBE_ACQUISITION:?}" '
        .metadata.labels = ((.metadata.labels // {}) + {
          "axiom.axiom.dev/acceptance-owner":$owner,
          "axiom.axiom.dev/acceptance-project":$project_id,
          "axiom.axiom.dev/acceptance-run-id":$run_id,
          "axiom.axiom.dev/acceptance-acquisition-id":$acquisition_id
        })
      ' "$manifest"
    ;;
  create)
    [[ "${2:-}" == --request-timeout=* \
      && "${3:-}" == "-f" && "${5:-}" == "-o" && "${6:-}" == "json" ]]
    manifest="${4:?}"
    kind="$(jq -er '.kind' "$manifest")"
    name="$(jq -er '.metadata.name' "$manifest")"
    case "$kind" in
      Namespace) resource_type=namespace ;;
      CustomResourceDefinition) resource_type=customresourcedefinition ;;
      *) exit 42 ;;
    esac
    file="$(resource_file "$resource_type" "$name")"
    [[ ! -e "$file" ]] || {
      echo "AlreadyExists" >&2
      exit 1
    }
    jq --arg uid "uid-${resource_type}-${name}" \
      '.metadata.uid=$uid | .metadata.resourceVersion="1"' \
      "$manifest" > "$file"
    if [[ "${SIFT_KUBE_CREATE_RESPONSE_LOST:-}" == "$resource_type/$name" ]]; then
      echo "injected lost create response" >&2
      exit 43
    fi
    cat "$file"
    ;;
  delete)
    raw="${2#--raw=}"
    body="$(cat)"
    case "$raw" in
      /api/v1/namespaces/*)
        resource_type=namespace
        name="${raw##*/}"
        ;;
      /apis/apiextensions.k8s.io/v1/customresourcedefinitions/*)
        resource_type=customresourcedefinition
        name="${raw##*/}"
        ;;
      *) exit 44 ;;
    esac
    file="$(resource_file "$resource_type" "$name")"
    [[ -f "$file" ]] || {
      echo "NotFound" >&2
      exit 1
    }
    live_uid="$(jq -er '.metadata.uid' "$file")"
    live_rv="$(jq -er '.metadata.resourceVersion' "$file")"
    [[ "$(jq -er '.preconditions.uid' <<<"$body")" == "$live_uid" \
      && "$(jq -er '.preconditions.resourceVersion' <<<"$body")" == "$live_rv" ]] \
      || {
        echo "Conflict" >&2
        exit 1
      }
    rm -f "$file"
    : > "$state/deleted-${resource_type}-${name}"
    printf '{}\n'
    ;;
  wait)
    reference="${3:?}"
    resource_type="${reference%%/*}"
    name="${reference#*/}"
    [[ ! -e "$(resource_file "$resource_type" "$name")" ]]
    ;;
  *)
    echo "unexpected fake kubectl call: $*" >&2
    exit 90
    ;;
esac
EOF
chmod +x "$fake_bin/kubectl"

export PATH="$fake_bin:$PATH"
export SIFT_KUBE_OWNERSHIP_CALLS="$calls"
export SIFT_KUBE_OWNERSHIP_STATE="$state_dir"
export SIFT_KUBE_OWNER="$KUBERNETES_ACCEPTANCE_OWNER"
export SIFT_KUBE_PROJECT="$project_id"
export SIFT_KUBE_RUN="$run_id"
export SIFT_KUBE_ACQUISITION="$acquisition_id"

: > "$calls"
cleanup_lumen_auth_delegation_bindings_for_mode sift
[[ ! -s "$calls" ]] || {
  echo "Sift-only cleanup deleted an unrelated Lumen auth binding" >&2
  exit 1
}
cleanup_lumen_auth_delegation_bindings_for_mode lumen-sift
rg -F 'delete clusterrolebinding -l app.kubernetes.io/component=auth-delegation,app.kubernetes.io/name=lumen ' \
  "$calls" >/dev/null || {
  echo "Lumen cleanup lost its cluster-scoped auth binding cleanup" >&2
  exit 1
}
: > "$calls"

denied_status=0
SIFT_KUBE_GET_DENIED="namespace/blocked" \
  require_kubernetes_resource_absent namespace blocked \
  > "$test_root/denied.log" 2>&1 || denied_status=$?
[[ "$denied_status" != "0" ]]
rg -F 'could not prove that namespace blocked is absent' \
  "$test_root/denied.log" >/dev/null
require_kubernetes_resource_absent namespace sift

SIFT_KUBE_CREATE_RESPONSE_LOST="namespace/sift" \
KUBERNETES_OWNERSHIP_CREATE_TIMEOUT_SECONDS=1 \
KUBERNETES_OWNERSHIP_CREATE_GRACE_SECONDS=0 \
  create_owned_namespace \
    sift "$receipt_root" "$project_id" "$run_id" "$acquisition_id"
namespace_receipt="$receipt_root/namespace-sift.json"
namespace_state="$state_dir/namespace-sift.json"
[[ -f "$namespace_receipt" && -f "$namespace_state" ]]
verify_kubernetes_ownership_receipt \
  "$namespace_receipt" "$(cat "$namespace_state")" namespace sift \
  "$project_id" "$run_id" "$acquisition_id"

crd_manifest="$test_root/sift-crd.json"
jq -n '
  {
    apiVersion:"apiextensions.k8s.io/v1",
    kind:"CustomResourceDefinition",
    metadata:{name:"sifts.sift.axiom.dev"},
    spec:{group:"sift.axiom.dev",names:{kind:"Sift",plural:"sifts"},scope:"Namespaced",versions:[]}
  }
' > "$crd_manifest"
create_owned_kubernetes_resource \
  customresourcedefinition sifts.sift.axiom.dev "$crd_manifest" \
  "$receipt_root" "$project_id" "$run_id" "$acquisition_id"

cp "$namespace_state" "$test_root/original-namespace.json"
jq '.metadata.uid="replacement-uid" | .metadata.resourceVersion="2"' \
  "$namespace_state" > "$namespace_state.tmp"
mv "$namespace_state.tmp" "$namespace_state"
: > "$calls"
replacement_status=0
delete_owned_kubernetes_resource \
  namespace sift "$receipt_root" "$project_id" "$run_id" "$acquisition_id" 5 \
  > "$test_root/replacement.log" 2>&1 || replacement_status=$?
[[ "$replacement_status" != "0" && -f "$namespace_state" ]]
rg -F 'was replaced; refusing deletion' "$test_root/replacement.log" >/dev/null
if rg -F 'delete --raw=' "$calls" >/dev/null; then
  echo "ownership cleanup sent a delete for a replacement namespace" >&2
  exit 1
fi

cp "$test_root/original-namespace.json" "$namespace_state"
: > "$calls"
delete_owned_kubernetes_resource \
  namespace sift "$receipt_root" "$project_id" "$run_id" "$acquisition_id" 5
[[ ! -e "$namespace_state" \
  && -f "$receipt_root/deleted-namespace-sift.json" ]]
rg -F 'delete --raw=/api/v1/namespaces/sift -f -' "$calls" >/dev/null
delete_owned_kubernetes_resource \
  namespace sift "$receipt_root" "$project_id" "$run_id" "$acquisition_id" 5

# Simulate a process that saved the intent, received no create response, and
# stopped before it could save the UID receipt. Cleanup must recover only the
# exact labeled object and then use UID/resourceVersion preconditions.
restore_intent="$receipt_root/namespace-sift-restore.intent.json"
write_kubernetes_ownership_intent \
  "$restore_intent" namespace sift-restore "$project_id" "$run_id" \
  "$acquisition_id"
jq -n \
  --arg owner "$KUBERNETES_ACCEPTANCE_OWNER" \
  --arg project_id "$project_id" --arg run_id "$run_id" \
  --arg acquisition_id "$acquisition_id" '
    {
      apiVersion:"v1",
      kind:"Namespace",
      metadata:{
        name:"sift-restore",
        uid:"uid-restore",
        resourceVersion:"7",
        labels:{
          "axiom.axiom.dev/acceptance-owner":$owner,
          "axiom.axiom.dev/acceptance-project":$project_id,
          "axiom.axiom.dev/acceptance-run-id":$run_id,
          "axiom.axiom.dev/acceptance-acquisition-id":$acquisition_id
        }
      }
    }
  ' > "$state_dir/namespace-sift-restore.json"
delete_owned_kubernetes_resource \
  namespace sift-restore "$receipt_root" "$project_id" "$run_id" \
  "$acquisition_id" 5 > "$test_root/recovered.log" 2>&1
rg -F 'recovered the ownership receipt for namespace sift-restore' \
  "$test_root/recovered.log" >/dev/null
[[ -f "$receipt_root/namespace-sift-restore.json" \
  && -f "$receipt_root/deleted-namespace-sift-restore.json" \
  && ! -e "$state_dir/namespace-sift-restore.json" ]]

late_intent="$receipt_root/namespace-sift-late.intent.json"
late_manifest="$receipt_root/namespace-sift-late.manifest.json"
jq -n \
  --arg owner "$KUBERNETES_ACCEPTANCE_OWNER" \
  --arg project_id "$project_id" --arg run_id "$run_id" \
  --arg acquisition_id "$acquisition_id" '
    {
      apiVersion:"v1",kind:"Namespace",metadata:{name:"sift-late",labels:{
        "axiom.axiom.dev/acceptance-owner":$owner,
        "axiom.axiom.dev/acceptance-project":$project_id,
        "axiom.axiom.dev/acceptance-run-id":$run_id,
        "axiom.axiom.dev/acceptance-acquisition-id":$acquisition_id
      }}
    }
  ' > "$late_manifest"
KUBERNETES_OWNERSHIP_CREATE_GRACE_SECONDS=0 \
  write_kubernetes_ownership_create_intent \
    "$late_intent" namespace sift-late "$project_id" "$run_id" \
    "$acquisition_id" "$late_manifest" 1
SIFT_KUBE_LATE_CREATE="namespace/sift-late" \
KUBERNETES_OWNERSHIP_CREATE_TIMEOUT_SECONDS=1 \
  delete_owned_kubernetes_resource \
    namespace sift-late "$receipt_root" "$project_id" "$run_id" \
    "$acquisition_id" 5 > "$test_root/late-create.log" 2>&1
[[ -f "$receipt_root/namespace-sift-late.json" \
  && -f "$receipt_root/deleted-namespace-sift-late.json" \
  && ! -e "$state_dir/namespace-sift-late.json" ]]
rg -F 'recovered the ownership receipt for namespace sift-late' \
  "$test_root/late-create.log" >/dev/null

# A replacement that appears after the preconditioned DELETE must prevent a
# clean deletion receipt. `kubectl wait` only tracks the old UID.
create_owned_namespace \
  sift-recreated "$receipt_root" "$project_id" "$run_id" "$acquisition_id"
recreated_status=0
SIFT_KUBE_RECREATE_AFTER_DELETE="namespace/sift-recreated" \
  delete_owned_kubernetes_resource \
    namespace sift-recreated "$receipt_root" "$project_id" "$run_id" \
    "$acquisition_id" 5 > "$test_root/recreated.log" 2>&1 \
    || recreated_status=$?
[[ "$recreated_status" != "0" \
  && -f "$state_dir/namespace-sift-recreated.json" \
  && ! -e "$receipt_root/deleted-namespace-sift-recreated.json" ]]
rg -F 'was recreated after preconditioned deletion' \
  "$test_root/recreated.log" >/dev/null

delete_owned_kubernetes_resource \
  customresourcedefinition sifts.sift.axiom.dev "$receipt_root" \
  "$project_id" "$run_id" "$acquisition_id" 5
[[ ! -e "$state_dir/customresourcedefinition-sifts.sift.axiom.dev.json" \
  && -f "$receipt_root/deleted-customresourcedefinition-sifts.sift.axiom.dev.json" ]]

echo "Kubernetes ownership receipt E2E: ok"
