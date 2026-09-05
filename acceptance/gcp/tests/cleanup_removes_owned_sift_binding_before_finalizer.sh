#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ACCEPTANCE_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
source "$ACCEPTANCE_ROOT/scripts/source-prefix.sh"
source "$ACCEPTANCE_ROOT/scripts/acceptance-lock.sh"
source "$ACCEPTANCE_ROOT/scripts/kubernetes-ownership.sh"

test_root="$(mktemp -d "${TMPDIR:-/tmp}/sift-owned-binding-cleanup.XXXXXX")"
fake_bin="$test_root/bin"
state_dir="$test_root/state"
evidence_dir="$test_root/evidence"
local_claim_root="$test_root/claims"
calls="$test_root/calls.log"
cr_state="$test_root/sift-cr-present"
binding_state="$test_root/sift-binding-present"
binding_replaced="$test_root/sift-binding-replaced"
lock_state="$test_root/acceptance-lock.json"

cleanup_test() {
  find "$test_root" -type f -delete
  find "$test_root" -depth -type d -empty -delete
}
trap cleanup_test EXIT INT TERM

mkdir -p "$fake_bin" "$state_dir" "$evidence_dir"
: > "$state_dir/kube-context-ready.txt"
: > "$calls"
: > "$cr_state"
: > "$binding_state"

write_test_lock() {
  local resource
  rm -f "$evidence_dir/acceptance-lock-release.json"
  resource="$(acceptance_lock_manifest \
    "test-project" "binding-red" "lumen-sift" \
    "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa" \
    | jq '.metadata.uid="owned-lock-uid" | .metadata.resourceVersion="30"')"
  printf '%s\n' "$resource" > "$lock_state"
  write_acceptance_lock_receipt \
    "$evidence_dir/acceptance-lock.json" "$resource" \
    "test-project" "binding-red" "lumen-sift" \
    "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
}
write_test_lock
write_acceptance_run_owner \
  "$(acceptance_run_claim_path \
    "$local_claim_root" "test-project" "binding-red" "lumen-sift")" \
  "test-project" "binding-red" "lumen-sift" "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa" \
  "$state_dir" "$evidence_dir" "99999999" "99999999" "test-dead-owner" \
  "dddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddd"

ownership_root="$evidence_dir/kubernetes/ownership"
mkdir -p "$ownership_root"
write_owned_fixture() {
  local resource_type="$1"
  local api_version="$2"
  local kind="$3"
  local name="$4"
  local resource_file="$state_dir/${resource_type}-${name}.json"
  local intent="$ownership_root/${resource_type}-${name}.intent.json"
  local receipt="$ownership_root/${resource_type}-${name}.json"
  jq -n \
    --arg api_version "$api_version" --arg kind "$kind" --arg name "$name" \
    --arg resource_type "$resource_type" '
      {
        apiVersion:$api_version,
        kind:$kind,
        metadata:{
          name:$name,
          uid:("uid-" + $resource_type + "-" + $name),
          resourceVersion:"1",
          labels:{
            "axiom.axiom.dev/acceptance-owner":"gcp-operator-acceptance",
            "axiom.axiom.dev/acceptance-project":"test-project",
            "axiom.axiom.dev/acceptance-run-id":"binding-red",
            "axiom.axiom.dev/acceptance-acquisition-id":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
          }
        }
      }
    ' > "$resource_file"
  write_kubernetes_ownership_intent \
    "$intent" "$resource_type" "$name" "test-project" "binding-red" \
    "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
  write_kubernetes_ownership_receipt \
    "$receipt" "$(cat "$resource_file")" "$resource_type" "$name" \
    "test-project" "binding-red" "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
}
reset_owned_fixtures() {
  write_owned_fixture namespace v1 Namespace sift
  write_owned_fixture namespace v1 Namespace sift-system
  write_owned_fixture customresourcedefinition apiextensions.k8s.io/v1 \
    CustomResourceDefinition sifts.sift.axiom.dev
}
reset_owned_fixtures

cat > "$fake_bin/gcloud" <<'EOF'
#!/usr/bin/env bash
set -euo pipefail
printf 'gcloud %s\n' "$*" >> "${SIFT_BINDING_TEST_CALLS:?}"
case " $* " in
  *" builds list "*)
    if [[ " $* " == *" --format=json "* ]]; then
      printf '[]\n'
    fi
    ;;
  *" artifacts docker images list "*) printf '[]\n' ;;
  *" artifacts docker images describe "*|*" container node-pools describe "*)
    echo "not found" >&2
    exit 1
    ;;
  *" storage ls "*)
    echo "matched no URLs" >&2
    exit 1
    ;;
  *" storage rm "*)
    echo "no URLs matched" >&2
    exit 1
    ;;
esac
EOF

cat > "$fake_bin/kubectl" <<'EOF'
#!/usr/bin/env bash
set -euo pipefail
printf 'kubectl %s\n' "$*" >> "${SIFT_BINDING_TEST_CALLS:?}"

emit_cr() {
  jq -cn \
    --arg deletion_timestamp "${SIFT_BINDING_TEST_DELETION_TIMESTAMP-2026-09-02T00:00:00Z}" '
      {
        metadata: {
          name: "sift",
          namespace: "sift",
          uid: "owned-sift-uid",
          resourceVersion: "10",
          finalizers: ["service-k8s.axiom.dev/sift-operator-cluster-children"]
        }
      }
      | if $deletion_timestamp == "" then .
        else .metadata.deletionTimestamp = $deletion_timestamp
        end
    '
}

payload_after_flag() {
  local wanted="$1"
  shift
  while (($#)); do
    if [[ "$1" == "$wanted" ]]; then
      shift
      printf '%s' "${1-}"
      return 0
    fi
    shift
  done
  return 1
}

if [[ " $* " == *" get deployment,statefulset,cronjob,job,pod,pvc "* ]]; then
  printf '{"items":[]}\n'
  exit 0
fi
if [[ " $* " == *" patch lease axiom-gcp-operator-acceptance-lock "* ]]; then
  while [[ "$#" -gt 0 && "$1" != "-p" ]]; do shift; done
  shift
  patch="${1:?}"
  session_id="$(jq -er '.[] | select(.path | endswith("cleanup-session-id")) | .value' <<<"$patch")"
  started_at="$(jq -er '.[] | select(.path | endswith("cleanup-started-at")) | .value' <<<"$patch")"
  jq --arg session_id "$session_id" --arg started_at "$started_at" '
    .metadata.annotations["axiom.axiom.dev/cleanup-session-id"] = $session_id
    | .metadata.annotations["axiom.axiom.dev/cleanup-started-at"] = $started_at
    | .metadata.resourceVersion = (((.metadata.resourceVersion | tonumber) + 1) | tostring)
  ' "${SIFT_BINDING_TEST_LOCK_STATE:?}" > "${SIFT_BINDING_TEST_LOCK_STATE}.tmp"
  mv "${SIFT_BINDING_TEST_LOCK_STATE}.tmp" "${SIFT_BINDING_TEST_LOCK_STATE}"
  cat "${SIFT_BINDING_TEST_LOCK_STATE}"
  exit 0
fi
if [[ " $* " == *" get lease axiom-gcp-operator-acceptance-lock --namespace kube-system -o json "* ]]; then
  [[ -f "${SIFT_BINDING_TEST_LOCK_STATE:?}" ]] || {
    echo "not found" >&2
    exit 1
  }
  cat "${SIFT_BINDING_TEST_LOCK_STATE:?}"
  exit 0
fi
if [[ " $* " == *" create -f - -o json "* ]]; then
  resource="$(cat)"
  jq -e '
    .kind == "Lease"
    and (.metadata.annotations["axiom.axiom.dev/acquisition-id"] | test("^[0-9a-f]{32}$"))
  ' >/dev/null <<<"$resource"
  resource="$(jq '.metadata.uid="recovered-lock-uid" | .metadata.resourceVersion="31"' \
    <<<"$resource")"
  printf '%s\n' "$resource" > "${SIFT_BINDING_TEST_LOCK_STATE:?}"
  printf '%s\n' "$resource"
  exit 0
fi
if [[ " $* " == *" get sift.sift.axiom.dev sift --namespace sift "* ]]; then
  [[ -f "${SIFT_BINDING_TEST_CR_STATE:?}" ]] || exit 1
  emit_cr
  exit 0
fi
if [[ " $* " == *" get sift.sift.axiom.dev sift-restore --namespace sift-restore "* ]]; then
  echo "not found" >&2
  exit 1
fi
if [[ " $* " == *" delete --raw=/apis/sift.axiom.dev/v1alpha1/namespaces/sift/sifts/sift -f - "* ]]; then
  delete_options="$(cat)"
  jq -e '
    .kind == "DeleteOptions"
    and .preconditions.uid == "owned-sift-uid"
    and .preconditions.resourceVersion == "10"
  ' >/dev/null <<<"$delete_options"
  # Simulate a transport timeout after the API server accepted an earlier
  # deletion. The live object is already marked for deletion.
  exit 1
fi
if [[ " $* " == *" delete sift.sift.axiom.dev sift --namespace sift "* ]]; then
  echo "unsafe name-only CR delete" >&2
  exit 42
fi
if [[ " $* " == *" get clusterrolebinding sift.sift.sift.auth-delegator "* ]]; then
  [[ -f "${SIFT_BINDING_TEST_BINDING_STATE:?}" ]] || {
    echo "not found" >&2
    exit 1
  }
  if [[ "${SIFT_BINDING_TEST_REPLACEMENT_RACE:-0}" == "1" \
      && -f "${SIFT_BINDING_TEST_REPLACED:?}" ]]; then
    binding_uid="replacement-binding-uid"
    binding_rv="21"
  else
    binding_uid="owned-binding-uid"
    binding_rv="20"
  fi
  printf '%s\n' "{\"metadata\":{\"name\":\"sift.sift.sift.auth-delegator\",\"uid\":\"${binding_uid}\",\"resourceVersion\":\"${binding_rv}\",\"labels\":{\"app.kubernetes.io/name\":\"sift\",\"app.kubernetes.io/instance\":\"sift\",\"app.kubernetes.io/component\":\"auth-delegation\",\"sift.axiom.dev/owner-namespace\":\"sift\",\"service-k8s.axiom.dev/owner-uid\":\"${SIFT_BINDING_TEST_LABEL_UID:-owned-sift-uid}\"}}}"
  if [[ "${SIFT_BINDING_TEST_REPLACEMENT_RACE:-0}" == "1" ]]; then
    : > "${SIFT_BINDING_TEST_REPLACED:?}"
  fi
  exit 0
fi
if [[ " $* " == *" delete --raw=/apis/rbac.authorization.k8s.io/v1/clusterrolebindings/sift.sift.sift.auth-delegator -f - "* ]]; then
  delete_options="$(cat)"
  jq -e '
    .kind == "DeleteOptions"
    and .preconditions.uid == "owned-binding-uid"
    and .preconditions.resourceVersion == "20"
  ' >/dev/null <<<"$delete_options"
  if [[ "${SIFT_BINDING_TEST_REPLACEMENT_RACE:-0}" == "1" ]]; then
    echo "Conflict: binding was replaced" >&2
    exit 1
  fi
  rm -f "${SIFT_BINDING_TEST_BINDING_STATE:?}"
  exit 0
fi
if [[ " $* " == *" delete clusterrolebinding sift.sift.sift.auth-delegator "* ]]; then
  echo "unsafe name-only binding delete" >&2
  exit 43
fi
if [[ " $* " == *" wait --for=delete clusterrolebinding/sift.sift.sift.auth-delegator "* ]]; then
  [[ ! -f "${SIFT_BINDING_TEST_BINDING_STATE:?}" ]]
  exit
fi
if [[ " $* " == *" patch sift.sift.axiom.dev sift --namespace sift "* ]]; then
  [[ ! -f "${SIFT_BINDING_TEST_BINDING_STATE:?}" ]] || {
    echo "binding still exists when finalizer was removed" >&2
    exit 41
  }
  patch="$(payload_after_flag -p "$@")"
  [[ " $* " == *" --type=json "* ]] || {
    echo "finalizer patch is not an atomic JSON patch" >&2
    exit 44
  }
  jq -e '
    any(.[]; .op == "test" and .path == "/metadata/uid" and .value == "owned-sift-uid")
    and any(.[]; .op == "test" and .path == "/metadata/resourceVersion" and .value == "10")
    and any(.[]; .op == "test" and .path == "/metadata/deletionTimestamp" and .value == "2026-09-02T00:00:00Z")
    and any(.[]; .op == "replace" and .path == "/metadata/finalizers" and .value == [])
  ' >/dev/null <<<"$patch" || {
    echo "finalizer patch lost its live-object preconditions" >&2
    exit 45
  }
  rm -f "${SIFT_BINDING_TEST_CR_STATE:?}"
  exit 0
fi
if [[ " $* " == *" wait --for=delete sift.sift.axiom.dev/sift --namespace sift "* ]]; then
  [[ ! -f "${SIFT_BINDING_TEST_CR_STATE:?}" ]]
  exit
fi
if [[ " $* " == *" delete --raw=/apis/coordination.k8s.io/v1/namespaces/kube-system/leases/axiom-gcp-operator-acceptance-lock -f - "* ]]; then
  delete_options="$(cat)"
  expected_uid="$(jq -r '.metadata.uid' "${SIFT_BINDING_TEST_LOCK_STATE:?}")"
  expected_rv="$(jq -r '.metadata.resourceVersion' "${SIFT_BINDING_TEST_LOCK_STATE:?}")"
  jq -e --arg uid "$expected_uid" --arg rv "$expected_rv" '
    .preconditions.uid == $uid
    and .preconditions.resourceVersion == $rv
  ' >/dev/null <<<"$delete_options"
  rm -f "${SIFT_BINDING_TEST_LOCK_STATE:?}"
  if [[ "${SIFT_BINDING_TEST_LOCK_DELETE_UNCERTAIN:-0}" == "1" ]]; then
    echo "injected lost Lease delete response" >&2
    exit 1
  fi
  exit 0
fi
if [[ " $* " == *" wait --for=delete lease/axiom-gcp-operator-acceptance-lock --namespace kube-system "* ]]; then
  [[ ! -f "${SIFT_BINDING_TEST_LOCK_STATE:?}" ]]
  exit
fi
if [[ "${1:-}" == "get" \
    && ( "${2:-}" == "namespace" \
      || "${2:-}" == "customresourcedefinition" ) ]]; then
  resource_file="${SIFT_BINDING_TEST_KUBE_STATE:?}/${2}-${3}.json"
  if [[ -f "$resource_file" ]]; then
    cat "$resource_file"
  else
    echo "not found" >&2
    exit 1
  fi
  exit 0
fi
if [[ "${1:-}" == "delete" && "${2:-}" == --raw=* ]]; then
  raw="${2#--raw=}"
  case "$raw" in
    /api/v1/namespaces/*)
      resource_file="${SIFT_BINDING_TEST_KUBE_STATE:?}/namespace-${raw##*/}.json"
      ;;
    /apis/apiextensions.k8s.io/v1/customresourcedefinitions/*)
      resource_file="${SIFT_BINDING_TEST_KUBE_STATE:?}/customresourcedefinition-${raw##*/}.json"
      ;;
    *) exit 88 ;;
  esac
  rm -f "$resource_file"
  printf '{}\n'
  exit 0
fi
if [[ "${1:-}" == "wait" \
    && ( "${3:-}" == namespace/* \
      || "${3:-}" == customresourcedefinition/* ) ]]; then
  exit 0
fi
if [[ " $* " == *" get namespace "* ]] \
    || [[ " $* " == *" get customresourcedefinition "* ]] \
    || [[ " $* " == *" get clusterrolebinding "* ]]; then
  echo "not found" >&2
  exit 1
fi
exit 0
EOF

cat > "$fake_bin/terraform" <<'EOF'
#!/usr/bin/env bash
exit 0
EOF

cat > "$fake_bin/ps" <<'EOF'
#!/usr/bin/env bash
set -euo pipefail
if [[ " $* " == *" -o lstart= -p "* ]]; then
  last=""
  for arg in "$@"; do last="$arg"; done
  printf 'test-start-%s\n' "$last"
fi
exit 0
EOF

chmod +x "$fake_bin/gcloud" "$fake_bin/kubectl" "$fake_bin/terraform" "$fake_bin/ps"
source_prefix="gs://test-source/source/axiom-gcp-operator-binding-red"
write_source_prefix_receipt \
  "$evidence_dir/source-prefix.json" "test-project" "binding-red" "$source_prefix"
printf '[]\n' > "$evidence_dir/preexisting-lumen-images.json"
printf '[]\n' > "$evidence_dir/preexisting-sift-images.json"

set +e
PATH="$fake_bin:$PATH" \
SIFT_BINDING_TEST_CALLS="$calls" \
SIFT_BINDING_TEST_CR_STATE="$cr_state" \
SIFT_BINDING_TEST_BINDING_STATE="$binding_state" \
SIFT_BINDING_TEST_REPLACED="$binding_replaced" \
SIFT_BINDING_TEST_LOCK_STATE="$lock_state" \
SIFT_BINDING_TEST_KUBE_STATE="$state_dir" \
SIFT_BINDING_TEST_LOCK_DELETE_UNCERTAIN="1" \
ACCEPTANCE_LOCAL_CLAIM_ROOT="$local_claim_root" \
PROJECT_ID="test-project" \
REGION="asia-east1" \
GKE_ZONE="asia-east1-a" \
RUN_ID="binding-red" \
STATE_DIR="$state_dir" \
ACCEPTANCE_ROOT="$ACCEPTANCE_ROOT" \
REGISTRY="asia-east1-docker.pkg.dev/test-project/courier" \
IMAGE_TAG="acceptance-binding-red" \
GCS_SOURCE_PREFIX="$source_prefix" \
EVIDENCE_DIR="$evidence_dir" \
ACCEPTANCE_APPS="lumen sift" \
bash "$ACCEPTANCE_ROOT/scripts/cleanup.sh" >"$test_root/cleanup.log" 2>&1
status=$?
set -e

[[ "$status" == "0" ]] || {
  echo "cleanup failed while removing an exactly owned Sift binding" >&2
  cat "$test_root/cleanup.log" >&2
  cat "$calls" >&2
  exit 1
}
[[ ! -e "$binding_state" ]] || {
  echo "cleanup left the exactly owned Sift auth binding behind" >&2
  exit 1
}
[[ -f "$evidence_dir/acceptance-lock-release.json" ]] || {
  echo "cleanup did not recover the accepted Lease delete after its response was lost" >&2
  exit 1
}
delete_line="$(rg -n '^kubectl delete --raw=/apis/rbac\.authorization\.k8s\.io/v1/clusterrolebindings/sift\.sift\.sift\.auth-delegator ' "$calls" | cut -d: -f1)"
patch_line="$(rg -n '^kubectl patch sift\.sift\.axiom\.dev sift ' "$calls" | cut -d: -f1)"
[[ -n "$delete_line" && -n "$patch_line" && "$delete_line" -lt "$patch_line" ]] || {
  echo "cleanup removed the Sift finalizer before its owned auth binding" >&2
  exit 1
}

# If the Lease delete took effect but its response and release receipt were
# lost, a retry must create a fresh Lease before it resumes cleanup.
: > "$cr_state"
: > "$binding_state"
: > "$calls"
write_test_lock
reset_owned_fixtures
rm -f "$lock_state" "$evidence_dir/acceptance-lock-release.json"
set +e
PATH="$fake_bin:$PATH" \
SIFT_BINDING_TEST_CALLS="$calls" \
SIFT_BINDING_TEST_CR_STATE="$cr_state" \
SIFT_BINDING_TEST_BINDING_STATE="$binding_state" \
SIFT_BINDING_TEST_REPLACED="$binding_replaced" \
SIFT_BINDING_TEST_LOCK_STATE="$lock_state" \
SIFT_BINDING_TEST_KUBE_STATE="$state_dir" \
ACCEPTANCE_LOCAL_CLAIM_ROOT="$local_claim_root" \
PROJECT_ID="test-project" \
REGION="asia-east1" \
GKE_ZONE="asia-east1-a" \
RUN_ID="binding-red" \
STATE_DIR="$state_dir" \
ACCEPTANCE_ROOT="$ACCEPTANCE_ROOT" \
REGISTRY="asia-east1-docker.pkg.dev/test-project/courier" \
IMAGE_TAG="acceptance-binding-red" \
GCS_SOURCE_PREFIX="$source_prefix" \
EVIDENCE_DIR="$evidence_dir" \
ACCEPTANCE_APPS="lumen sift" \
bash "$ACCEPTANCE_ROOT/scripts/cleanup.sh" >"$test_root/recovery-cleanup.log" 2>&1
recovery_status=$?
set -e

[[ "$recovery_status" == "0" ]] || {
  echo "cleanup could not resume after an accepted Lease delete lost its response" >&2
  cat "$test_root/recovery-cleanup.log" >&2
  cat "$calls" >&2
  exit 1
}
rg -q '^kubectl create -f - -o json' "$calls" || {
  echo "cleanup resumed without atomically reacquiring the shared GKE Lease" >&2
  exit 1
}
[[ -f "$evidence_dir/acceptance-lock-release.json" ]] || {
  echo "cleanup did not persist the recovered Lease release" >&2
  exit 1
}

# A same-name binding with a different owner UID can belong to another live
# object. Cleanup must fail closed and leave both the grant and finalizer alone.
: > "$cr_state"
: > "$binding_state"
: > "$calls"
write_test_lock
reset_owned_fixtures
set +e
PATH="$fake_bin:$PATH" \
SIFT_BINDING_TEST_CALLS="$calls" \
SIFT_BINDING_TEST_CR_STATE="$cr_state" \
SIFT_BINDING_TEST_BINDING_STATE="$binding_state" \
SIFT_BINDING_TEST_REPLACED="$binding_replaced" \
SIFT_BINDING_TEST_LOCK_STATE="$lock_state" \
SIFT_BINDING_TEST_KUBE_STATE="$state_dir" \
SIFT_BINDING_TEST_LABEL_UID="foreign-owner-uid" \
ACCEPTANCE_LOCAL_CLAIM_ROOT="$local_claim_root" \
PROJECT_ID="test-project" \
REGION="asia-east1" \
GKE_ZONE="asia-east1-a" \
RUN_ID="binding-red" \
STATE_DIR="$state_dir" \
ACCEPTANCE_ROOT="$ACCEPTANCE_ROOT" \
REGISTRY="asia-east1-docker.pkg.dev/test-project/courier" \
IMAGE_TAG="acceptance-binding-red" \
GCS_SOURCE_PREFIX="$source_prefix" \
EVIDENCE_DIR="$evidence_dir" \
ACCEPTANCE_APPS="lumen sift" \
bash "$ACCEPTANCE_ROOT/scripts/cleanup.sh" >"$test_root/foreign-cleanup.log" 2>&1
foreign_status=$?
set -e

[[ "$foreign_status" -ne 0 && -e "$binding_state" ]] || {
  echo "cleanup did not fail closed for a foreign Sift auth binding" >&2
  cat "$test_root/foreign-cleanup.log" >&2
  exit 1
}
if rg -q '^kubectl (delete --raw=/apis/rbac\.authorization\.k8s\.io/v1/clusterrolebindings/sift\.sift\.sift\.auth-delegator|patch sift\.sift\.axiom\.dev sift )' "$calls"; then
  echo "cleanup changed a foreign Sift auth binding or its CR finalizer" >&2
  exit 1
fi

# A failed delete is not authorization to strip a live CR. The API server must
# show the same UID, an accepted deletion timestamp, and the expected finalizer.
: > "$cr_state"
: > "$binding_state"
: > "$calls"
write_test_lock
reset_owned_fixtures
set +e
PATH="$fake_bin:$PATH" \
SIFT_BINDING_TEST_CALLS="$calls" \
SIFT_BINDING_TEST_CR_STATE="$cr_state" \
SIFT_BINDING_TEST_BINDING_STATE="$binding_state" \
SIFT_BINDING_TEST_REPLACED="$binding_replaced" \
SIFT_BINDING_TEST_LOCK_STATE="$lock_state" \
SIFT_BINDING_TEST_KUBE_STATE="$state_dir" \
SIFT_BINDING_TEST_DELETION_TIMESTAMP="" \
ACCEPTANCE_LOCAL_CLAIM_ROOT="$local_claim_root" \
PROJECT_ID="test-project" \
REGION="asia-east1" \
GKE_ZONE="asia-east1-a" \
RUN_ID="binding-red" \
STATE_DIR="$state_dir" \
ACCEPTANCE_ROOT="$ACCEPTANCE_ROOT" \
REGISTRY="asia-east1-docker.pkg.dev/test-project/courier" \
IMAGE_TAG="acceptance-binding-red" \
GCS_SOURCE_PREFIX="$source_prefix" \
EVIDENCE_DIR="$evidence_dir" \
ACCEPTANCE_APPS="lumen sift" \
bash "$ACCEPTANCE_ROOT/scripts/cleanup.sh" >"$test_root/live-cleanup.log" 2>&1
live_status=$?
set -e

[[ "$live_status" -ne 0 && -e "$binding_state" && -e "$cr_state" ]] || {
  echo "cleanup stripped a live Sift CR after an unaccepted delete" >&2
  cat "$test_root/live-cleanup.log" >&2
  exit 1
}
if rg -q '^kubectl (get clusterrolebinding sift\.sift\.sift\.auth-delegator|patch sift\.sift\.axiom\.dev sift )' "$calls"; then
  echo "cleanup entered finalizer fallback for a live Sift CR" >&2
  exit 1
fi

# A same-name binding can be replaced after GET. The UID and resourceVersion
# preconditions must stop deletion and must leave the CR finalizer intact.
: > "$cr_state"
: > "$binding_state"
rm -f "$binding_replaced"
: > "$calls"
write_test_lock
reset_owned_fixtures
set +e
PATH="$fake_bin:$PATH" \
SIFT_BINDING_TEST_CALLS="$calls" \
SIFT_BINDING_TEST_CR_STATE="$cr_state" \
SIFT_BINDING_TEST_BINDING_STATE="$binding_state" \
SIFT_BINDING_TEST_REPLACED="$binding_replaced" \
SIFT_BINDING_TEST_LOCK_STATE="$lock_state" \
SIFT_BINDING_TEST_KUBE_STATE="$state_dir" \
SIFT_BINDING_TEST_REPLACEMENT_RACE="1" \
ACCEPTANCE_LOCAL_CLAIM_ROOT="$local_claim_root" \
PROJECT_ID="test-project" \
REGION="asia-east1" \
GKE_ZONE="asia-east1-a" \
RUN_ID="binding-red" \
STATE_DIR="$state_dir" \
ACCEPTANCE_ROOT="$ACCEPTANCE_ROOT" \
REGISTRY="asia-east1-docker.pkg.dev/test-project/courier" \
IMAGE_TAG="acceptance-binding-red" \
GCS_SOURCE_PREFIX="$source_prefix" \
EVIDENCE_DIR="$evidence_dir" \
ACCEPTANCE_APPS="lumen sift" \
bash "$ACCEPTANCE_ROOT/scripts/cleanup.sh" >"$test_root/race-cleanup.log" 2>&1
race_status=$?
set -e

[[ "$race_status" -ne 0 && -e "$binding_state" && -e "$cr_state" ]] || {
  echo "cleanup did not fail closed when the Sift binding was replaced" >&2
  cat "$test_root/race-cleanup.log" >&2
  exit 1
}
if rg -q '^kubectl patch sift\.sift\.axiom\.dev sift ' "$calls"; then
  echo "cleanup removed the CR finalizer after a binding replacement race" >&2
  exit 1
fi

echo "owned Sift auth binding finalizer cleanup E2E: ok"
