#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ACCEPTANCE_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
source "$ACCEPTANCE_ROOT/scripts/source-prefix.sh"
source "$ACCEPTANCE_ROOT/scripts/acceptance-lock.sh"

test_root="$(mktemp -d "${TMPDIR:-/tmp}/sift-image-inventory.XXXXXX")"
fake_bin="$test_root/bin"
state_dir="$test_root/state"
evidence_dir="$test_root/evidence"
local_claim_root="$test_root/claims"
calls="$test_root/calls.log"
digest="$(printf 'a%.0s' {1..64})"
git_sha="0123456789abcdef0123456789abcdef01234567"
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
    "test-project" "image-red" "sift" "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa" \
    | jq '.metadata.uid="owned-lock-uid" | .metadata.resourceVersion="30"'
)"
printf '%s\n' "$lock_resource" > "$lock_state"
write_acceptance_lock_receipt \
  "$evidence_dir/acceptance-lock.json" "$lock_resource" \
  "test-project" "image-red" "sift" "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
write_acceptance_run_owner \
  "$(acceptance_run_claim_path \
    "$local_claim_root" "test-project" "image-red" "sift")" \
  "test-project" "image-red" "sift" "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa" \
  "$state_dir" "$evidence_dir" "99999999" "99999999" "test-dead-owner" \
  "dddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddd"

cat > "$fake_bin/gcloud" <<'EOF'
#!/usr/bin/env bash
set -euo pipefail
printf 'gcloud %s\n' "$*" >> "${SIFT_IMAGE_TEST_CALLS:?}"
if [[ " $* " == *" artifacts docker images list "* ]]; then
  if [[ "${SIFT_IMAGE_TEST_SCENARIO:-inventory-failure}" == "inventory-failure" \
      && " $* " == *"/sift "* ]]; then
    echo "injected Artifact Registry inventory failure" >&2
    exit 29
  fi
  printf '[]\n'
  exit 0
fi
case " $* " in
  *" builds list "*" --format=json "*) printf '[]\n' ;;
  *" builds list "*) ;;
  *" artifacts docker images describe "*)
    if [[ "${SIFT_IMAGE_TEST_SCENARIO:-inventory-failure}" == "tag-race" \
        && " $* " == *"/sift:"* ]]; then
      count_file="${SIFT_IMAGE_TEST_STATE:?}/sift-describe-count"
      count=0
      [[ ! -f "$count_file" ]] || count="$(cat "$count_file")"
      count=$((count + 1))
      printf '%s\n' "$count" > "$count_file"
      if [[ "$count" == "1" ]]; then
        printf 'sha256:%s\n' "${SIFT_IMAGE_TEST_DIGEST:?}"
      else
        printf 'sha256:%064d\n' 9
      fi
      exit 0
    fi
    echo "not found" >&2
    exit 1
    ;;
  *" artifacts docker tags delete "*|*" artifacts docker images delete "*) ;;
  *" storage ls "*|*" storage rm "*)
    echo "matched no URLs" >&2
    exit 1
    ;;
esac
EOF

cat > "$fake_bin/kubectl" <<'EOF'
#!/usr/bin/env bash
set -euo pipefail
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
  ' "${SIFT_IMAGE_TEST_LOCK_STATE:?}" > "${SIFT_IMAGE_TEST_LOCK_STATE}.tmp"
  mv "${SIFT_IMAGE_TEST_LOCK_STATE}.tmp" "${SIFT_IMAGE_TEST_LOCK_STATE}"
  cat "${SIFT_IMAGE_TEST_LOCK_STATE}"
  exit 0
fi
if [[ " $* " == *" get lease axiom-gcp-operator-acceptance-lock --namespace kube-system -o json "* ]]; then
  cat "${SIFT_IMAGE_TEST_LOCK_STATE:?}"
  exit 0
fi
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
source_prefix="gs://test-source/source/axiom-gcp-operator-image-red"
source_object="source/axiom-gcp-operator-image-red/source.tgz"
source_uri="gs://test-source/${source_object}"
write_source_prefix_receipt \
  "$evidence_dir/source-prefix.json" "test-project" "image-red" "$source_prefix"
jq -n --arg git_sha "$git_sha" '{git_sha:$git_sha}' > "$evidence_dir/run.json"
jq -n --arg git_sha "$git_sha" --arg digest "$digest" \
  '{git_sha:$git_sha,source_bundle_sha256:$digest}' \
  > "$evidence_dir/candidate-source.json"
jq -n --arg git_sha "$git_sha" --arg digest "$digest" --arg source_uri "$source_uri" '
  {
    build_id:"build-image-red",
    git_sha:$git_sha,
    source_uri:$source_uri,
    source_bundle_sha256:$digest,
    staged_source_sha256:$digest
  }' > "$evidence_dir/cloud-build-source-binding.json"
jq -n --arg source_object "$source_object" '
  {source:{storageSource:{bucket:"test-source",object:$source_object}}}' \
  > "$evidence_dir/cloud-build-submit.json"
jq -n --arg digest "$digest" '
  {
    sift:("example.invalid/sift@sha256:" + $digest),
    rig:("example.invalid/rig@sha256:" + $digest),
    acceptance_runner:("example.invalid/sift-acceptance-runner@sha256:" + $digest)
  }' > "$evidence_dir/images.json"
printf '[]\n' > "$evidence_dir/preexisting-sift-images.json"
printf '[]\n' > "$evidence_dir/preexisting-rig-images.json"
printf '[]\n' > "$evidence_dir/preexisting-sift-acceptance-runner-images.json"

set +e
PATH="$fake_bin:$PATH" \
SIFT_IMAGE_TEST_CALLS="$calls" \
SIFT_IMAGE_TEST_LOCK_STATE="$lock_state" \
ACCEPTANCE_LOCAL_CLAIM_ROOT="$local_claim_root" \
PROJECT_ID="test-project" \
REGION="asia-east1" \
GKE_ZONE="asia-east1-a" \
RUN_ID="image-red" \
STATE_DIR="$state_dir" \
ACCEPTANCE_ROOT="$ACCEPTANCE_ROOT" \
REGISTRY="example.invalid" \
IMAGE_TAG="acceptance-image-red" \
GCS_SOURCE_PREFIX="$source_prefix" \
EVIDENCE_DIR="$evidence_dir" \
ACCEPTANCE_APPS="sift" \
bash "$ACCEPTANCE_ROOT/scripts/cleanup.sh" >"$test_root/cleanup.log" 2>&1
status=$?
set -e

[[ "$status" -ne 0 ]] || {
  echo "cleanup hid an Artifact Registry inventory failure" >&2
  exit 1
}
rg -q '^gcloud artifacts docker images list example.invalid/sift ' "$calls" || {
  echo "cleanup did not inventory the Sift image" >&2
  cat "$calls" >&2
  cat "$test_root/cleanup.log" >&2
  exit 1
}
if [[ -e "$evidence_dir/deleted-image-sift.txt" ]]; then
  echo "cleanup wrote a deletion receipt without a valid image inventory" >&2
  exit 1
fi

# A missing before-run inventory is not evidence that the digest was new.
# Cleanup must fail before it removes even the run tag.
rm -f "$evidence_dir/preexisting-sift-images.json"
printf '%s\n' "$lock_resource" > "$lock_state"
write_acceptance_lock_receipt \
  "$evidence_dir/acceptance-lock.json" "$lock_resource" \
  "test-project" "image-red" "sift" "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
rm -f "$evidence_dir/acceptance-cleanup-session.json" \
  "$evidence_dir/acceptance-cleanup-session-intent.json"
: > "$calls"
set +e
PATH="$fake_bin:$PATH" \
SIFT_IMAGE_TEST_CALLS="$calls" \
SIFT_IMAGE_TEST_LOCK_STATE="$lock_state" \
ACCEPTANCE_LOCAL_CLAIM_ROOT="$local_claim_root" \
PROJECT_ID="test-project" \
REGION="asia-east1" \
GKE_ZONE="asia-east1-a" \
RUN_ID="image-red" \
STATE_DIR="$state_dir" \
ACCEPTANCE_ROOT="$ACCEPTANCE_ROOT" \
REGISTRY="example.invalid" \
IMAGE_TAG="acceptance-image-red" \
GCS_SOURCE_PREFIX="$source_prefix" \
EVIDENCE_DIR="$evidence_dir" \
ACCEPTANCE_APPS="sift" \
bash "$ACCEPTANCE_ROOT/scripts/cleanup.sh" >"$test_root/missing-inventory.log" 2>&1
missing_status=$?
set -e

[[ "$missing_status" -ne 0 ]] || {
  echo "cleanup accepted a missing pre-run image inventory" >&2
  exit 1
}
if rg -q '^gcloud artifacts docker images delete example.invalid/sift@' "$calls"; then
  echo "cleanup deleted the Sift digest before proving it was new" >&2
  exit 1
fi

# A live tag without one matching digest inventory record is ambiguous. Cleanup
# must retain it and must not issue an immutable delete.
printf '[]\n' > "$evidence_dir/preexisting-sift-images.json"
printf '%s\n' "$lock_resource" > "$lock_state"
write_acceptance_lock_receipt \
  "$evidence_dir/acceptance-lock.json" "$lock_resource" \
  "test-project" "image-red" "sift" "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
rm -f "$evidence_dir/acceptance-cleanup-session.json" \
  "$evidence_dir/acceptance-cleanup-session-intent.json" \
  "$state_dir/sift-describe-count"
: > "$calls"
set +e
PATH="$fake_bin:$PATH" \
SIFT_IMAGE_TEST_CALLS="$calls" \
SIFT_IMAGE_TEST_LOCK_STATE="$lock_state" \
SIFT_IMAGE_TEST_SCENARIO="tag-race" \
SIFT_IMAGE_TEST_STATE="$state_dir" \
SIFT_IMAGE_TEST_DIGEST="$digest" \
ACCEPTANCE_LOCAL_CLAIM_ROOT="$local_claim_root" \
PROJECT_ID="test-project" \
REGION="asia-east1" \
GKE_ZONE="asia-east1-a" \
RUN_ID="image-red" \
STATE_DIR="$state_dir" \
ACCEPTANCE_ROOT="$ACCEPTANCE_ROOT" \
REGISTRY="example.invalid" \
IMAGE_TAG="acceptance-image-red" \
GCS_SOURCE_PREFIX="$source_prefix" \
EVIDENCE_DIR="$evidence_dir" \
ACCEPTANCE_APPS="sift" \
bash "$ACCEPTANCE_ROOT/scripts/cleanup.sh" >"$test_root/tag-race.log" 2>&1
race_status=$?
set -e

[[ "$race_status" -ne 0 ]] || {
  echo "cleanup accepted an image tag that moved between validation reads" >&2
  exit 1
}
rg -F 'image digest inventory is ambiguous for sift' \
  "$test_root/tag-race.log" >/dev/null
if rg -q '^gcloud artifacts docker images delete example.invalid/sift@' "$calls"; then
  echo "cleanup deleted an ambiguous Sift image digest" >&2
  exit 1
fi

echo "Artifact Registry inventory failure E2E: ok"
