#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ACCEPTANCE_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
source "$ACCEPTANCE_ROOT/scripts/source-prefix.sh"
source "$ACCEPTANCE_ROOT/scripts/acceptance-lock.sh"
source "$ACCEPTANCE_ROOT/scripts/kubernetes-ownership.sh"
test_root="$(mktemp -d "${TMPDIR:-/tmp}/sift-cleanup-continuation.XXXXXX")"
fake_bin="$test_root/bin"
state_dir="$test_root/state"
evidence_dir="$test_root/evidence"
local_claim_root="$test_root/claims"
calls="$test_root/calls.log"
lock_state="$test_root/acceptance-lock.json"

cleanup_test() {
  find "$test_root" -type f -delete
  find "$test_root" -depth -type d -empty -delete
}
trap cleanup_test EXIT INT TERM

mkdir -p "$fake_bin" "$state_dir/environment" "$evidence_dir"
: > "$state_dir/environment.tfstate"
: > "$state_dir/kube-context-ready.txt"
: > "$calls"
lock_resource="$(
  acceptance_lock_manifest "test-project" "cleanup-red" "sift" \
    "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa" \
    | jq '.metadata.uid="owned-lock-uid" | .metadata.resourceVersion="30"'
)"
printf '%s\n' "$lock_resource" > "$lock_state"
write_acceptance_lock_receipt \
  "$evidence_dir/acceptance-lock.json" "$lock_resource" \
  "test-project" "cleanup-red" "sift" "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
write_acceptance_run_owner \
  "$(acceptance_run_claim_path \
    "$local_claim_root" "test-project" "cleanup-red" "sift")" \
  "test-project" "cleanup-red" "sift" "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa" \
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
            "axiom.axiom.dev/acceptance-run-id":"cleanup-red",
            "axiom.axiom.dev/acceptance-acquisition-id":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
          }
        }
      }
    ' > "$resource_file"
  write_kubernetes_ownership_intent \
    "$intent" "$resource_type" "$name" "test-project" "cleanup-red" \
    "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
  write_kubernetes_ownership_receipt \
    "$receipt" "$(cat "$resource_file")" "$resource_type" "$name" \
    "test-project" "cleanup-red" "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
}
write_owned_fixture namespace v1 Namespace sift
write_owned_fixture namespace v1 Namespace sift-system
write_owned_fixture customresourcedefinition apiextensions.k8s.io/v1 \
  CustomResourceDefinition sifts.sift.axiom.dev

cat > "$fake_bin/gcloud" <<'EOF'
#!/usr/bin/env bash
set -euo pipefail
printf 'gcloud %s\n' "$*" >> "${SIFT_CLEANUP_TEST_CALLS:?}"
if [[ " $* " == *" builds list "* ]]; then
  echo "injected Cloud Build inventory failure" >&2
  exit 23
fi
if [[ " $* " == *" artifacts docker images list "* ]]; then
  printf '[]\n'
fi
EOF

cat > "$fake_bin/kubectl" <<'EOF'
#!/usr/bin/env bash
set -euo pipefail
printf 'kubectl %s\n' "$*" >> "${SIFT_CLEANUP_TEST_CALLS:?}"
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
  ' "${SIFT_CLEANUP_TEST_LOCK_STATE:?}" > "${SIFT_CLEANUP_TEST_LOCK_STATE}.tmp"
  mv "${SIFT_CLEANUP_TEST_LOCK_STATE}.tmp" "${SIFT_CLEANUP_TEST_LOCK_STATE}"
  cat "${SIFT_CLEANUP_TEST_LOCK_STATE}"
  exit 0
fi
if [[ " $* " == *" get lease axiom-gcp-operator-acceptance-lock --namespace kube-system -o json "* ]]; then
  cat "${SIFT_CLEANUP_TEST_LOCK_STATE:?}"
  exit 0
fi
if [[ "${1:-}" == "get" \
    && ( "${2:-}" == "namespace" \
      || "${2:-}" == "customresourcedefinition" ) ]]; then
  resource_file="${SIFT_CLEANUP_TEST_KUBE_STATE:?}/${2}-${3}.json"
  if [[ -f "$resource_file" ]]; then
    cat "$resource_file"
  else
    echo "NotFound" >&2
    exit 1
  fi
  exit 0
fi
if [[ "${1:-}" == "delete" && "${2:-}" == --raw=* ]]; then
  raw="${2#--raw=}"
  case "$raw" in
    /api/v1/namespaces/*)
      resource_file="${SIFT_CLEANUP_TEST_KUBE_STATE:?}/namespace-${raw##*/}.json"
      ;;
    /apis/apiextensions.k8s.io/v1/customresourcedefinitions/*)
      resource_file="${SIFT_CLEANUP_TEST_KUBE_STATE:?}/customresourcedefinition-${raw##*/}.json"
      ;;
    *) exit 88 ;;
  esac
  rm -f "$resource_file"
  printf '{}\n'
  exit 0
fi
if [[ "${1:-}" == "wait" ]]; then
  exit 0
fi
if [[ " $* " == *" get sift.sift.axiom.dev "* ]]; then
  echo "NotFound" >&2
  exit 1
fi
if [[ " $* " == *" get deployment,statefulset,cronjob,job,pod,pvc "* ]]; then
  printf '{"items":[]}\n'
  exit 0
fi
if [[ "${1:-}" == "get" ]]; then
  echo "NotFound" >&2
  exit 1
fi
EOF

cat > "$fake_bin/terraform" <<'EOF'
#!/usr/bin/env bash
set -euo pipefail
printf 'terraform %s\n' "$*" >> "${SIFT_CLEANUP_TEST_CALLS:?}"
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
source_prefix="gs://test-source/source/axiom-gcp-operator-cleanup-red"
write_source_prefix_receipt \
  "$evidence_dir/source-prefix.json" "test-project" "cleanup-red" "$source_prefix"
printf '[]\n' > "$evidence_dir/preexisting-sift-images.json"
printf '[]\n' > "$evidence_dir/preexisting-rig-images.json"
printf '[]\n' > "$evidence_dir/preexisting-sift-acceptance-runner-images.json"

set +e
PATH="$fake_bin:$PATH" \
SIFT_CLEANUP_TEST_CALLS="$calls" \
SIFT_CLEANUP_TEST_LOCK_STATE="$lock_state" \
SIFT_CLEANUP_TEST_KUBE_STATE="$state_dir" \
ACCEPTANCE_LOCAL_CLAIM_ROOT="$local_claim_root" \
PROJECT_ID="test-project" \
REGION="asia-east1" \
GKE_ZONE="asia-east1-a" \
RUN_ID="cleanup-red" \
STATE_DIR="$state_dir" \
ACCEPTANCE_ROOT="$ACCEPTANCE_ROOT" \
REGISTRY="asia-east1-docker.pkg.dev/test-project/courier" \
IMAGE_TAG="acceptance-cleanup-red" \
GCS_SOURCE_PREFIX="$source_prefix" \
EVIDENCE_DIR="$evidence_dir" \
ACCEPTANCE_APPS="sift" \
bash "$ACCEPTANCE_ROOT/scripts/cleanup.sh" > "$test_root/cleanup.log" 2>&1
status=$?
set -e

[[ "$status" -ne 0 ]] || {
  echo "cleanup hid the injected Cloud Build inventory failure" >&2
  exit 1
}
rg -q '^kubectl delete --raw=/api/v1/namespaces/sift -f -' "$calls" || {
  echo "cleanup stopped before Kubernetes resources after a Cloud Build failure" >&2
  exit 1
}
rg -q '^terraform .* destroy ' "$calls" || {
  echo "cleanup stopped before Terraform destroy after a Cloud Build failure" >&2
  cat "$calls" >&2
  cat "$test_root/cleanup.log" >&2
  exit 1
}

echo "cleanup continuation after Cloud Build failure E2E: ok"
