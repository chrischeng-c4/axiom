#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ACCEPTANCE_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
source "$ACCEPTANCE_ROOT/scripts/acceptance-lock.sh"

test_root="$(mktemp -d "${TMPDIR:-/tmp}/sift-acceptance-lock.XXXXXX")"
fake_bin="$test_root/bin"
state_dir="$test_root/state"
evidence_dir="$test_root/evidence"
local_claim_root="$test_root/claims"
calls="$test_root/calls.log"
foreign_lock="$test_root/foreign-lock.json"
owned_lock="$test_root/owned-lock.json"

cleanup_test() {
  find "$test_root" -type f -delete
  find "$test_root" -depth -type d -empty -delete
}
trap cleanup_test EXIT INT TERM

mkdir -p "$fake_bin" "$state_dir" "$evidence_dir"
: > "$state_dir/kube-context-ready.txt"
: > "$calls"

owned_resource="$(
  acceptance_lock_manifest "test-project" "lock-red" "tape" \
    "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa" \
    | jq '.metadata.uid="owned-lock-uid" | .metadata.resourceVersion="30"'
)"
printf '%s\n' "$owned_resource" > "$owned_lock"
write_acceptance_lock_receipt \
  "$evidence_dir/acceptance-lock.json" "$owned_resource" \
  "test-project" "lock-red" "tape" "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
verify_acceptance_lock_receipt \
  "$evidence_dir/acceptance-lock.json" "$owned_resource" \
  "test-project" "lock-red" "tape"
write_acceptance_run_owner \
  "$(acceptance_run_claim_path \
    "$local_claim_root" "test-project" "lock-red" "tape")" \
  "test-project" "lock-red" "tape" "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa" \
  "$state_dir" "$evidence_dir" "99999999" "99999999" "test-dead-owner" \
  "dddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddd"

foreign_resource="$(
  acceptance_lock_manifest "test-project" "other-run" "tape" \
    "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb" \
    | jq '.metadata.uid="foreign-lock-uid" | .metadata.resourceVersion="41"'
)"
printf '%s\n' "$foreign_resource" > "$foreign_lock"
if verify_acceptance_lock_receipt \
    "$evidence_dir/acceptance-lock.json" "$foreign_resource" \
    "test-project" "lock-red" "tape"; then
  echo "an acceptance-lock receipt accepted another run's Lease" >&2
  exit 1
fi

cat > "$fake_bin/gcloud" <<'EOF'
#!/usr/bin/env bash
set -euo pipefail
printf 'gcloud %s\n' "$*" >> "${SIFT_LOCK_TEST_CALLS:?}"
if [[ " $* " == *" builds list "* ]]; then
  if [[ " $* " == *" --format=json "* ]]; then
    printf '[]\n'
  fi
  exit 0
fi
if [[ " $* " == *" artifacts docker images describe "* ]]; then
  echo "not found" >&2
  exit 1
fi
exit 0
EOF

cat > "$fake_bin/kubectl" <<'EOF'
#!/usr/bin/env bash
set -euo pipefail
printf 'kubectl %s\n' "$*" >> "${SIFT_LOCK_TEST_CALLS:?}"
if [[ " $* " == *" create -f - -o json "* ]]; then
  cat >/dev/null
  if [[ "${SIFT_LOCK_TEST_CREATE_RECOVERY:-0}" == "1" ]]; then
    echo "injected lost Lease create response" >&2
    exit 29
  fi
fi
if [[ " $* " == *" get lease axiom-gcp-operator-acceptance-lock --namespace kube-system -o json "* ]]; then
  if [[ "${SIFT_LOCK_TEST_CREATE_RECOVERY:-0}" == "1" ]]; then
    cat "${SIFT_LOCK_TEST_OWNED_LOCK:?}"
  else
    cat "${SIFT_LOCK_TEST_FOREIGN_LOCK:?}"
  fi
  exit 0
fi
if [[ " $* " == *" get namespace tape "* ]]; then
  printf 'tape\n'
  exit 0
fi
if [[ " $* " == *" get customresourcedefinition tapes.tape.dev "* ]]; then
  printf 'tapes.tape.dev\n'
  exit 0
fi
if [[ " $* " == *" get deployment,statefulset,cronjob,job,pod,pvc "* ]]; then
  printf '{"items":[]}\n'
  exit 0
fi
echo "not found" >&2
exit 1
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
printf '[]\n' > "$evidence_dir/preexisting-tape-images.json"

set +e
create_output="$(
  acceptance_lock_manifest \
    "test-project" "lock-red" "tape" "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa" \
    | PATH="$fake_bin:$PATH" \
      SIFT_LOCK_TEST_CALLS="$calls" \
      SIFT_LOCK_TEST_CREATE_RECOVERY="1" \
      SIFT_LOCK_TEST_OWNED_LOCK="$owned_lock" \
      SIFT_LOCK_TEST_FOREIGN_LOCK="$foreign_lock" \
      kubectl create -f - -o json
)"
create_status=$?
set -e
[[ "$create_status" -ne 0 && -z "$create_output" ]] || {
  echo "the uncertain Lease create injection did not fail as expected" >&2
  exit 1
}
recovered_resource="$(
  PATH="$fake_bin:$PATH" \
  SIFT_LOCK_TEST_CALLS="$calls" \
  SIFT_LOCK_TEST_CREATE_RECOVERY="1" \
  SIFT_LOCK_TEST_OWNED_LOCK="$owned_lock" \
  SIFT_LOCK_TEST_FOREIGN_LOCK="$foreign_lock" \
  kubectl get lease "$(acceptance_lock_name)" \
    --namespace "$(acceptance_lock_namespace)" -o json
)"
verify_acceptance_lock_json \
  "$recovered_resource" "test-project" "lock-red" "tape" \
  "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"

set +e
PATH="$fake_bin:$PATH" \
SIFT_LOCK_TEST_CALLS="$calls" \
SIFT_LOCK_TEST_FOREIGN_LOCK="$foreign_lock" \
ACCEPTANCE_LOCAL_CLAIM_ROOT="$local_claim_root" \
PROJECT_ID="test-project" \
REGION="asia-east1" \
GKE_ZONE="asia-east1-a" \
RUN_ID="lock-red" \
STATE_DIR="$state_dir" \
ACCEPTANCE_ROOT="$ACCEPTANCE_ROOT" \
REGISTRY="asia-east1-docker.pkg.dev/test-project/courier" \
IMAGE_TAG="acceptance-lock-red" \
GCS_SOURCE_PREFIX="gs://test-source/source/axiom-gcp-operator-lock-red" \
EVIDENCE_DIR="$evidence_dir" \
ACCEPTANCE_APPS="tape" \
bash "$ACCEPTANCE_ROOT/scripts/cleanup.sh" >"$test_root/cleanup.log" 2>&1
status=$?
set -e

[[ "$status" -ne 0 ]] || {
  echo "cleanup accepted another run's shared GKE Lease" >&2
  exit 1
}
if rg -q '^kubectl (delete|patch) ' "$calls"; then
  echo "cleanup changed Kubernetes state while another run owned the Lease" >&2
  cat "$calls" >&2
  exit 1
fi
if rg -q '^gcloud (builds cancel|artifacts docker tags delete|artifacts docker images delete|storage rm) ' "$calls" \
    || rg -q '^terraform .* destroy ' "$calls"; then
  echo "cleanup changed run-scoped cloud state while another run owned the Lease" >&2
  cat "$calls" >&2
  exit 1
fi

echo "shared GKE acceptance lock scope E2E: ok"
