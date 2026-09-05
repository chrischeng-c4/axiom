#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ACCEPTANCE_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
source "$ACCEPTANCE_ROOT/scripts/source-prefix.sh"
source "$ACCEPTANCE_ROOT/scripts/acceptance-lock.sh"

test_root="$(mktemp -d "${TMPDIR:-/tmp}/sift-early-cleanup.XXXXXX")"
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
    "test-project" "early-red" "sift" "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa" \
    | jq '.metadata.uid="early-lock-uid" | .metadata.resourceVersion="30"'
)"
printf '%s\n' "$lock_resource" > "$lock_state"
write_acceptance_lock_intent \
  "$evidence_dir/acceptance-lock-intent.json" \
  "test-project" "early-red" "sift" "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
write_acceptance_run_owner \
  "$(acceptance_run_claim_path \
    "$local_claim_root" "test-project" "early-red" "sift")" \
  "test-project" "early-red" "sift" "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa" \
  "$state_dir" "$evidence_dir" "99999999" "99999999" "test-dead-owner" \
  "dddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddd"
[[ ! -e "$evidence_dir/acceptance-lock.json" ]]
source_prefix="gs://test-source/source/axiom-gcp-operator-early-red"
write_source_prefix_receipt \
  "$evidence_dir/source-prefix.json" "test-project" "early-red" "$source_prefix"
printf '[]\n' > "$evidence_dir/preexisting-sift-images.json"
printf '[]\n' > "$evidence_dir/preexisting-rig-images.json"
printf '[]\n' > "$evidence_dir/preexisting-sift-acceptance-runner-images.json"

cat > "$fake_bin/gcloud" <<'EOF'
#!/usr/bin/env bash
set -euo pipefail
printf 'gcloud %s\n' "$*" >> "${SIFT_EARLY_CLEANUP_CALLS:?}"
case " $* " in
  *" builds list "*" --format=json "*) printf '[]\n' ;;
  *" builds list "*) ;;
  *" artifacts docker images describe "*|*" container node-pools describe "*)
    echo "not found" >&2
    exit 1
    ;;
  *" artifacts docker images list "*) printf '[]\n' ;;
  *" storage ls "*|*" storage rm "*)
    echo "matched no URLs" >&2
    exit 1
    ;;
  *" storage buckets list "*|*" iam service-accounts list "*|*" compute disks list "*) ;;
esac
EOF

cat > "$fake_bin/kubectl" <<'EOF'
#!/usr/bin/env bash
set -euo pipefail
printf 'kubectl %s\n' "$*" >> "${SIFT_EARLY_CLEANUP_CALLS:?}"
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
  ' "${SIFT_EARLY_CLEANUP_LOCK_STATE:?}" > "${SIFT_EARLY_CLEANUP_LOCK_STATE}.tmp"
  mv "${SIFT_EARLY_CLEANUP_LOCK_STATE}.tmp" "${SIFT_EARLY_CLEANUP_LOCK_STATE}"
  cat "${SIFT_EARLY_CLEANUP_LOCK_STATE}"
  exit 0
fi
if [[ " $* " == *" get lease axiom-gcp-operator-acceptance-lock --namespace kube-system -o json "* ]]; then
  [[ -f "${SIFT_EARLY_CLEANUP_LOCK_STATE:?}" ]] || {
    echo "not found" >&2
    exit 1
  }
  cat "${SIFT_EARLY_CLEANUP_LOCK_STATE}"
  exit 0
fi
if [[ " $* " == *" delete --raw=/apis/coordination.k8s.io/v1/namespaces/kube-system/leases/axiom-gcp-operator-acceptance-lock -f - "* ]]; then
  options="$(cat)"
  expected_uid="$(jq -r '.metadata.uid' "${SIFT_EARLY_CLEANUP_LOCK_STATE:?}")"
  expected_rv="$(jq -r '.metadata.resourceVersion' "${SIFT_EARLY_CLEANUP_LOCK_STATE:?}")"
  jq -e --arg uid "$expected_uid" --arg rv "$expected_rv" '
    .preconditions.uid == $uid and .preconditions.resourceVersion == $rv
  ' >/dev/null <<<"$options"
  rm -f "${SIFT_EARLY_CLEANUP_LOCK_STATE}"
  exit 0
fi
if [[ " $* " == *" wait --for=delete lease/axiom-gcp-operator-acceptance-lock "* ]]; then
  [[ ! -f "${SIFT_EARLY_CLEANUP_LOCK_STATE:?}" ]]
  exit
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

PATH="$fake_bin:$PATH" \
SIFT_EARLY_CLEANUP_CALLS="$calls" \
SIFT_EARLY_CLEANUP_LOCK_STATE="$lock_state" \
ACCEPTANCE_LOCAL_CLAIM_ROOT="$local_claim_root" \
PROJECT_ID="test-project" \
REGION="asia-east1" \
GKE_ZONE="asia-east1-a" \
RUN_ID="early-red" \
STATE_DIR="$state_dir" \
ACCEPTANCE_ROOT="$ACCEPTANCE_ROOT" \
REGISTRY="example.invalid" \
IMAGE_TAG="candidate-early-red" \
GCS_SOURCE_PREFIX="$source_prefix" \
EVIDENCE_DIR="$evidence_dir" \
ACCEPTANCE_APPS="sift" \
bash "$ACCEPTANCE_ROOT/scripts/cleanup.sh" >"$test_root/cleanup.log" 2>&1

[[ ! -e "$lock_state" && -f "$evidence_dir/acceptance-lock-release.json" ]] || {
  echo "early-failure cleanup did not release the shared Lease" >&2
  cat "$test_root/cleanup.log" >&2
  exit 1
}
jq -e '.status == "clean" and has("candidate") == false' \
  "$evidence_dir/cleanup.json" >/dev/null || {
  echo "early-failure cleanup required a candidate that was never built" >&2
  exit 1
}
for absent_receipt in run.json images.json sift-mvp-verification.json; do
  [[ ! -e "$evidence_dir/$absent_receipt" ]] || exit 1
done

echo "early candidate failure cleanup E2E: ok"
