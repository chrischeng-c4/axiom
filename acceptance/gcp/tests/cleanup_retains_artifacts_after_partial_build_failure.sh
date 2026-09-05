#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ACCEPTANCE_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
source "$ACCEPTANCE_ROOT/scripts/source-prefix.sh"
source "$ACCEPTANCE_ROOT/scripts/acceptance-lock.sh"

test_root="$(mktemp -d "${TMPDIR:-/tmp}/sift-cleanup-partial-build.XXXXXX")"
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

mkdir -p "$fake_bin" "$state_dir" "$evidence_dir"
: > "$calls"
lock_resource="$(
  acceptance_lock_manifest \
    "test-project" "partial-build" "sift" "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa" \
    | jq '.metadata.uid="owned-lock-uid" | .metadata.resourceVersion="30"'
)"
printf '%s\n' "$lock_resource" > "$lock_state"
write_acceptance_lock_receipt \
  "$evidence_dir/acceptance-lock.json" "$lock_resource" \
  "test-project" "partial-build" "sift" "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
write_acceptance_run_owner \
  "$(acceptance_run_claim_path \
    "$local_claim_root" "test-project" "partial-build" "sift")" \
  "test-project" "partial-build" "sift" "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa" \
  "$state_dir" "$evidence_dir" "99999999" "99999999" "test-dead-owner" \
  "dddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddd"

cat > "$fake_bin/gcloud" <<'EOF'
#!/usr/bin/env bash
set -euo pipefail
printf 'gcloud %s\n' "$*" >> "${SIFT_PARTIAL_BUILD_CALLS:?}"
case " $* " in
  *" builds list "*" --format=value(id) "*)
    printf 'bad-build\ngood-build\n'
    ;;
  *" builds list "*" --format=json "*)
    printf '[{"id":"bad-build","status":"WORKING"},{"id":"good-build","status":"SUCCESS"}]\n'
    ;;
  *" builds describe bad-build "*)
    echo "injected bad-build status failure" >&2
    exit 23
    ;;
  *" builds describe good-build "*) printf 'SUCCESS\n' ;;
  *" artifacts docker images list "*) printf '[]\n' ;;
  *" artifacts docker images describe "*)
    echo "not found" >&2
    exit 1
    ;;
  *" storage ls "*)
    echo "matched no URLs" >&2
    exit 1
    ;;
esac
EOF

cat > "$fake_bin/kubectl" <<'EOF'
#!/usr/bin/env bash
set -euo pipefail
printf 'kubectl %s\n' "$*" >> "${SIFT_PARTIAL_BUILD_CALLS:?}"
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
  ' "${SIFT_PARTIAL_BUILD_LOCK_STATE:?}" > "${SIFT_PARTIAL_BUILD_LOCK_STATE}.tmp"
  mv "${SIFT_PARTIAL_BUILD_LOCK_STATE}.tmp" "${SIFT_PARTIAL_BUILD_LOCK_STATE}"
  cat "${SIFT_PARTIAL_BUILD_LOCK_STATE}"
  exit 0
fi
if [[ " $* " == *" get lease axiom-gcp-operator-acceptance-lock --namespace kube-system -o json "* ]]; then
  cat "${SIFT_PARTIAL_BUILD_LOCK_STATE:?}"
  exit 0
fi
if [[ " $* " == *" get deployment,statefulset,cronjob,job,pod,pvc "* ]]; then
  printf '{"items":[]}\n'
elif [[ " $* " == *" get "* ]]; then
  echo "not found" >&2
  exit 1
fi
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
source_prefix="gs://test-source/source/axiom-gcp-operator-partial-build"
write_source_prefix_receipt \
  "$evidence_dir/source-prefix.json" "test-project" "partial-build" "$source_prefix"

set +e
PATH="$fake_bin:$PATH" \
SIFT_PARTIAL_BUILD_CALLS="$calls" \
SIFT_PARTIAL_BUILD_LOCK_STATE="$lock_state" \
ACCEPTANCE_LOCAL_CLAIM_ROOT="$local_claim_root" \
PROJECT_ID="test-project" \
REGION="asia-east1" \
GKE_ZONE="asia-east1-a" \
RUN_ID="partial-build" \
STATE_DIR="$state_dir" \
ACCEPTANCE_ROOT="$ACCEPTANCE_ROOT" \
REGISTRY="asia-east1-docker.pkg.dev/test-project/courier" \
IMAGE_TAG="acceptance-partial-build" \
GCS_SOURCE_PREFIX="$source_prefix" \
EVIDENCE_DIR="$evidence_dir" \
ACCEPTANCE_APPS="sift" \
bash "$ACCEPTANCE_ROOT/scripts/cleanup.sh" >"$test_root/cleanup.log" 2>&1
status=$?
set -e

[[ "$status" -ne 0 ]] || {
  echo "cleanup hid one Cloud Build failure behind a later successful build" >&2
  exit 1
}
rg -q '^gcloud builds describe bad-build ' "$calls"
rg -q '^gcloud builds describe good-build ' "$calls" || {
  echo "cleanup stopped before checking the second Cloud Build" >&2
  exit 1
}
if rg -q '^gcloud (artifacts docker (tags|images) delete|storage rm) ' "$calls"; then
  echo "cleanup deleted candidate artifacts while one Cloud Build state was unknown" >&2
  exit 1
fi
if rg -q '^gcloud builds cancel ' "$calls"; then
  echo "cleanup cancelled a run-tagged build before proving ownership" >&2
  exit 1
fi

echo "partial Cloud Build cleanup failure E2E: ok"
