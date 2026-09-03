#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ACCEPTANCE_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
test_root="$(mktemp -d "${TMPDIR:-/tmp}/sift-candidate-preflight.XXXXXX")"
repo="$test_root/repo"
fake_bin="$test_root/bin"
gcloud_log="$test_root/gcloud.log"
cleanup_test() {
  if [[ "${SIFT_KEEP_TEST_TMP:-0}" == "1" ]]; then
    echo "kept test root: $test_root" >&2
    return
  fi
  find "$test_root" -depth -delete >/dev/null 2>&1 || true
}
trap cleanup_test EXIT INT TERM

mkdir -p "$repo/acceptance/gcp/scripts" "$repo/apps/sift" "$fake_bin"
cp "$ACCEPTANCE_ROOT/scripts/prepare-sift-candidate.sh" \
  "$ACCEPTANCE_ROOT/scripts/cleanup-sift-candidate.sh" \
  "$ACCEPTANCE_ROOT/scripts/sift-candidate.sh" \
  "$ACCEPTANCE_ROOT/scripts/source-prefix.sh" \
  "$repo/acceptance/gcp/scripts/"
printf 'steps: []\n' > "$repo/acceptance/gcp/cloudbuild.sift-mvp.yaml"
printf '%s\n' \
  '#!/usr/bin/env bash' \
  'set -euo pipefail' \
  '[[ "${1:-}" == "--candidate" ]]' \
  'if [[ -n "${SIFT_FAKE_GATE_READY:-}" ]]; then' \
  '  : > "$SIFT_FAKE_GATE_READY"' \
  '  while [[ ! -e "${SIFT_FAKE_GATE_RELEASE:?}" ]]; do sleep 0.02; done' \
  'fi' \
  'printf "candidate gate passed\\n"' \
  > "$repo/apps/sift/test.sh"
chmod +x "$repo/apps/sift/test.sh" "$repo/acceptance/gcp/scripts/"*.sh
git -c init.defaultBranch=main -C "$repo" init -q
git -C "$repo" add .
git -c user.name=Sift-Test -c user.email=sift-test@example.invalid \
  -C "$repo" commit -qm fixture

printf '%s\n' '#!/usr/bin/env bash' 'exit 0' > "$fake_bin/docker"
cat > "$fake_bin/gcloud" <<'EOF'
#!/usr/bin/env bash
set -euo pipefail
printf '%s\n' "$*" >> "${SIFT_FAKE_GCLOUD_LOG:?}"
failure_receipt="${SIFT_FAKE_RECOVERY_DIR:?}/candidate-preparation-failure.json"
cancelled="${SIFT_FAKE_STATE_DIR:?}/cancelled"
exact_build() {
  local build_id="$1"
  local status="WORKING"
  [[ ! -e "$cancelled" ]] || status="CANCELLED"
  jq -n \
    --argjson failure "$(cat "$failure_receipt")" \
    --arg id "$build_id" --arg status "$status" '
      {
        id:$id,
        status:$status,
        substitutions:{
          _RUN_ID:$failure.run_id,
          _GIT_SHA:$failure.git_sha,
          _SOURCE_BUNDLE_SHA256:$failure.source_bundle_sha256,
          _REGISTRY:$failure.registry,
          _TAG:$failure.image_tag,
          _CANDIDATE_ACQUISITION_ID:$failure.acquisition_id
        },
        source:{storageSource:{
          bucket:"axiom-test_cloudbuild",
          object:("source/axiom-gcp-operator-" + $failure.run_id + "/source.tgz")
        }},
        tags:[
          "sift-mvp",
          ("axiom-run-" + $failure.run_id),
          ("axiom-source-" + $failure.source_bundle_sha256),
          ("axiom-acquisition-" + $failure.acquisition_id)
        ],
        results:{images:[]}
      }
    '
}
case "${1:-} ${2:-}" in
  "artifacts repositories") printf '{}\n' ;;
  "storage buckets") printf '{}\n' ;;
  "storage ls")
    if [[ "$*" == *"/**"* ]]; then
      if [[ -e "${SIFT_FAKE_STATE_DIR:?}/source-removed" ]]; then
        echo "matched no URLs" >&2
        exit 1
      fi
      if [[ "${SIFT_FAKE_SCENARIO:?}" != "intent-denied" \
        && ! -e "${SIFT_FAKE_STATE_DIR:?}/source-object-removed" ]]; then
        printf '%s/source.tgz\n' "${SIFT_FAKE_SOURCE_PREFIX:?}"
      fi
      [[ ! -e "${SIFT_FAKE_STATE_DIR:?}/remote-submit-intent.json" ]] \
        || printf '%s/candidate-submit-intent.json\n' \
          "${SIFT_FAKE_SOURCE_PREFIX:?}"
      [[ ! -e "${SIFT_FAKE_STATE_DIR:?}/remote-reservation.json" ]] \
        || printf '%s/candidate-reservation.json\n' \
          "${SIFT_FAKE_SOURCE_PREFIX:?}"
      exit 0
    fi
    if [[ "${SIFT_FAKE_SCENARIO:?}" == "source-denied" ]]; then
      echo "PERMISSION_DENIED" >&2
      exit 1
    fi
    ;;
  "storage cp")
    source_arg="${3:?}"
    destination_arg="${4:?}"
    if [[ "$source_arg" == gs://* ]]; then
      case "$source_arg" in
        */candidate-reservation.json)
          [[ -e "${SIFT_FAKE_STATE_DIR:?}/remote-reservation.json" ]] || {
            echo "NOT_FOUND" >&2
            exit 1
          }
          cp "${SIFT_FAKE_STATE_DIR:?}/remote-reservation.json" "$destination_arg"
          ;;
        */candidate-submit-intent.json)
          [[ -e "${SIFT_FAKE_STATE_DIR:?}/remote-submit-intent.json" ]] || {
            echo "NOT_FOUND" >&2
            exit 1
          }
          cp "${SIFT_FAKE_STATE_DIR:?}/remote-submit-intent.json" "$destination_arg"
          ;;
        *)
          echo "NOT_FOUND" >&2
          exit 1
          ;;
      esac
    else
      case "$destination_arg" in
        */candidate-reservation.json)
          if [[ -e "${SIFT_FAKE_STATE_DIR:?}/remote-reservation.json" ]]; then
            echo "PRECONDITION_FAILED" >&2
            exit 1
          fi
          cp "$source_arg" "${SIFT_FAKE_STATE_DIR:?}/remote-reservation.json"
          ;;
        */candidate-submit-intent.json)
          if [[ "${SIFT_FAKE_SCENARIO:?}" == "intent-denied" ]]; then
            echo "PERMISSION_DENIED" >&2
            exit 1
          fi
          if [[ -e "${SIFT_FAKE_STATE_DIR:?}/remote-submit-intent.json" ]]; then
            echo "PRECONDITION_FAILED" >&2
            exit 1
          fi
          cp "$source_arg" "${SIFT_FAKE_STATE_DIR:?}/remote-submit-intent.json"
          ;;
        *)
          echo "unexpected fake storage upload: $*" >&2
          exit 91
          ;;
      esac
    fi
    ;;
  "artifacts docker")
    if [[ "$SIFT_FAKE_SCENARIO" == "image-denied" ]]; then
      echo "PERMISSION_DENIED" >&2
    else
      echo "NOT_FOUND" >&2
    fi
    exit 1
    ;;
  "builds submit")
    case "$SIFT_FAKE_SCENARIO" in
      lost-response|lost-empty|cleanup-denied) exit 52 ;;
      *) printf 'build-1\n' ;;
    esac
    ;;
  "builds list")
    case "$SIFT_FAKE_SCENARIO" in
      cleanup-denied)
        echo "PERMISSION_DENIED" >&2
        exit 53
        ;;
      lost-empty|intent-denied) printf '[]\n' ;;
      lost-response) printf '[{"id":"build-2"}]\n' ;;
      *) printf '[{"id":"build-1"}]\n' ;;
    esac
    ;;
  "builds cancel")
    : > "$cancelled"
    ;;
  "builds describe")
    build_id="${3:?}"
    if [[ -f "$failure_receipt" ]]; then
      exact_build "$build_id"
    else
      jq -n --arg id "$build_id" --arg run_id "${RUN_ID:?}" '
        {
          id:$id,
          status:"WORKING",
          source:{storageSource:{
            bucket:"axiom-test_cloudbuild",
            object:("source/axiom-gcp-operator-" + $run_id + "/source.tgz")
          }}
        }
      '
    fi
    ;;
  "storage objects")
    source_uri="${4:?}"
    case "$source_uri" in
      */candidate-reservation.json)
        [[ -e "${SIFT_FAKE_STATE_DIR:?}/remote-reservation.json" ]] || {
          echo "NOT_FOUND" >&2
          exit 1
        }
        jq -n '{generation:"101"}'
        ;;
      */candidate-submit-intent.json)
        [[ -e "${SIFT_FAKE_STATE_DIR:?}/remote-submit-intent.json" ]] || {
          echo "NOT_FOUND" >&2
          exit 1
        }
        jq -n '{generation:"201"}'
        ;;
      *)
        echo "injected source-object receipt failure" >&2
        exit 54
        ;;
    esac
    ;;
  "storage rm")
    source_uri="${3:?}"
    generation_match=""
    for argument in "$@"; do
      case "$argument" in
        --if-generation-match=*) generation_match="${argument#*=}" ;;
      esac
    done
    case "$source_uri" in
      */source.tgz)
        : > "${SIFT_FAKE_STATE_DIR:?}/source-object-removed"
        ;;
      */candidate-submit-intent.json)
        [[ "$generation_match" == "201" ]] || {
          echo "PRECONDITION_FAILED" >&2
          exit 1
        }
        rm -f "${SIFT_FAKE_STATE_DIR:?}/remote-submit-intent.json"
        ;;
      */candidate-reservation.json)
        [[ "$generation_match" == "101" ]] || {
          echo "PRECONDITION_FAILED" >&2
          exit 1
        }
        rm -f "${SIFT_FAKE_STATE_DIR:?}/remote-reservation.json"
        ;;
      *)
        echo "unexpected fake source delete: $source_uri" >&2
        exit 92
        ;;
    esac
    if [[ -e "${SIFT_FAKE_STATE_DIR:?}/source-object-removed" \
      && ! -e "${SIFT_FAKE_STATE_DIR:?}/remote-submit-intent.json" \
      && ! -e "${SIFT_FAKE_STATE_DIR:?}/remote-reservation.json" ]]; then
      : > "${SIFT_FAKE_STATE_DIR:?}/source-removed"
    fi
    ;;
  *)
    echo "unexpected fake gcloud call: $*" >&2
    exit 90
    ;;
esac
EOF
chmod +x "$fake_bin/docker" "$fake_bin/gcloud"

# The candidate directory is the local reservation. A second preparer cannot
# publish into it while the first preparer is still running.
atomic_candidate="$test_root/atomic-candidate"
atomic_state="$test_root/atomic-state"
atomic_ready="$test_root/atomic-ready"
atomic_release="$test_root/atomic-release"
mkdir -p "$atomic_state"
PATH="$fake_bin:$PATH" \
  SIFT_FAKE_GCLOUD_LOG="$gcloud_log" \
  SIFT_FAKE_SCENARIO=source-denied \
  SIFT_FAKE_RECOVERY_DIR="${atomic_candidate}.failed" \
  SIFT_FAKE_STATE_DIR="$atomic_state" \
  SIFT_FAKE_SOURCE_PREFIX="gs://axiom-test_cloudbuild/source/axiom-gcp-operator-atomic-claim" \
  SIFT_FAKE_GATE_READY="$atomic_ready" \
  SIFT_FAKE_GATE_RELEASE="$atomic_release" \
  PROJECT_ID=axiom-test REGION=asia-east1 RUN_ID=atomic-claim \
  CANDIDATE_DIR="$atomic_candidate" \
  "$repo/acceptance/gcp/scripts/prepare-sift-candidate.sh" \
  > "$test_root/atomic-first.log" 2>&1 &
atomic_pid=$!
for _ in $(seq 1 200); do
  [[ ! -e "$atomic_ready" ]] || break
  sleep 0.02
done
[[ -e "$atomic_ready" && -d "$atomic_candidate" ]] || {
  echo "first preparer did not acquire its local candidate directory" >&2
  exit 1
}
atomic_second_status=0
PATH="$fake_bin:$PATH" \
  SIFT_FAKE_GCLOUD_LOG="$gcloud_log" \
  SIFT_FAKE_SCENARIO=source-denied \
  SIFT_FAKE_RECOVERY_DIR="${atomic_candidate}.failed" \
  SIFT_FAKE_STATE_DIR="$atomic_state" \
  SIFT_FAKE_SOURCE_PREFIX="gs://axiom-test_cloudbuild/source/axiom-gcp-operator-atomic-claim" \
  PROJECT_ID=axiom-test REGION=asia-east1 RUN_ID=atomic-claim \
  CANDIDATE_DIR="$atomic_candidate" \
  "$repo/acceptance/gcp/scripts/prepare-sift-candidate.sh" \
  > "$test_root/atomic-second.log" 2>&1 || atomic_second_status=$?
[[ "$atomic_second_status" != "0" ]] || {
  echo "second preparer acquired an already-owned candidate directory" >&2
  exit 1
}
rg -F 'CANDIDATE_DIR must be a new absolute path' \
  "$test_root/atomic-second.log" >/dev/null
[[ ! -d "$atomic_candidate/receipts" ]] || {
  echo "concurrent candidate preparation created a nested receipt directory" >&2
  exit 1
}
: > "$atomic_release"
atomic_first_status=0
wait "$atomic_pid" || atomic_first_status=$?
[[ "$atomic_first_status" != "0" && ! -e "$atomic_candidate" ]] || {
  echo "first preparer did not release an unused local claim" >&2
  exit 1
}

run_failure() {
  local scenario="$1"
  local run_id="$2"
  local candidate_dir="$test_root/$run_id"
  local output="$test_root/${run_id}.log"
  local state_dir="$test_root/${run_id}-state"
  local status=0
  mkdir -p "$state_dir"
  if [[ "$scenario" == "reservation-conflict" ]]; then
    printf '{"owner":"another-acquisition"}\n' \
      > "$state_dir/remote-reservation.json"
  fi
  PATH="$fake_bin:$PATH" \
    SIFT_FAKE_GCLOUD_LOG="$gcloud_log" \
    SIFT_FAKE_SCENARIO="$scenario" \
    SIFT_FAKE_RECOVERY_DIR="${candidate_dir}.failed" \
    SIFT_FAKE_STATE_DIR="$state_dir" \
    SIFT_FAKE_SOURCE_PREFIX="gs://axiom-test_cloudbuild/source/axiom-gcp-operator-${run_id}" \
    PROJECT_ID=axiom-test REGION=asia-east1 RUN_ID="$run_id" \
    CANDIDATE_DIR="$candidate_dir" \
    CANDIDATE_CLEANUP_DISCOVERY_ATTEMPTS=1 \
    CANDIDATE_CLEANUP_DISCOVERY_DELAY_SECONDS=0 \
    CANDIDATE_CLEANUP_WAIT_ATTEMPTS=1 \
    CANDIDATE_CLEANUP_WAIT_DELAY_SECONDS=0 \
    "$repo/acceptance/gcp/scripts/prepare-sift-candidate.sh" \
    > "$output" 2>&1 || status=$?
  [[ "$status" != "0" ]] || {
    echo "candidate preflight unexpectedly passed: $scenario" >&2
    exit 1
  }
  printf '%s\n' "$candidate_dir"
}

: > "$gcloud_log"
source_denied_dir="$(run_failure source-denied source-denied)"
rg -F 'could not inventory the pre-existing Cloud Build source bucket' \
  "$test_root/source-denied.log" >/dev/null
if rg -F 'builds submit' "$gcloud_log" >/dev/null; then
  echo "source inventory failure reached Cloud Build" >&2
  exit 1
fi
[[ ! -e "$source_denied_dir" && ! -e "${source_denied_dir}.failed" ]]

: > "$gcloud_log"
image_denied_dir="$(run_failure image-denied image-denied)"
rg -F 'could not inventory existing sift images' \
  "$test_root/image-denied.log" >/dev/null
if rg -F 'builds submit' "$gcloud_log" >/dev/null; then
  echo "image inventory failure reached Cloud Build" >&2
  exit 1
fi
[[ ! -e "$image_denied_dir" && ! -e "${image_denied_dir}.failed" ]]

: > "$gcloud_log"
reservation_conflict_dir="$(run_failure reservation-conflict reserve-conflict)"
[[ ! -e "$reservation_conflict_dir" \
  && -f "${reservation_conflict_dir}.failed/candidate-preparation-failure.json" \
  && ! -e "${reservation_conflict_dir}.failed/candidate-cleanup.json" ]]
rg -F 'reservation is owned by another acquisition or could not be verified' \
  "$test_root/reserve-conflict.log" >/dev/null
if rg -F 'builds submit' "$gcloud_log" >/dev/null \
    || rg -F 'storage rm ' "$gcloud_log" >/dev/null \
    || rg -F 'artifacts docker images delete ' "$gcloud_log" >/dev/null; then
  echo "reservation conflict mutated another acquisition" >&2
  exit 1
fi

: > "$gcloud_log"
failed_build_dir="$(run_failure not-found build-failed)"
rg -F 'builds submit' "$gcloud_log" >/dev/null
rg -F 'builds cancel build-1' "$gcloud_log" >/dev/null || {
  cat "$test_root/build-failed.log" >&2
  cat "$gcloud_log" >&2
  exit 1
}
[[ ! -e "$failed_build_dir" \
  && -f "${failed_build_dir}.failed/candidate-preparation-failure.json" \
  && -f "${failed_build_dir}.failed/candidate-source.tar.gz" \
  && -f "${failed_build_dir}.failed/candidate-cleanup.json" \
  && ! -e "${failed_build_dir}.failed/candidate-cleanup-failures.log" ]] || {
  echo "failed Cloud Build did not retain exact recovery evidence" >&2
  exit 1
}
jq -e '
  .schema == "axiom.gcp.sift.candidate-preparation-failure.v2"
  and .cloud_build_id == "build-1"
  and .submit_response_received == true
  and .submit_intent_published == true
  and .submit_started == true
  and (.acquisition_id | test("^[0-9a-f]{32}$"))
  and .run_id == "build-failed"
  and .image_tag == (.git_sha + "-build-failed-" + .acquisition_id)
  and .source_prefix == "gs://axiom-test_cloudbuild/source/axiom-gcp-operator-build-failed"
  ' "${failed_build_dir}.failed/candidate-preparation-failure.json" >/dev/null

: > "$gcloud_log"
lost_response_dir="$(run_failure lost-response lost-response)"
rg -F 'builds cancel build-2' "$gcloud_log" >/dev/null
rg -F 'storage rm gs://axiom-test_cloudbuild/source/axiom-gcp-operator-lost-response/candidate-reservation.json --if-generation-match=101 --quiet' \
  "$gcloud_log" >/dev/null
jq -e '
  .cloud_build_id == ""
  and .submit_response_received == false
  and .submit_intent_published == true
  and .submit_started == true
  ' "${lost_response_dir}.failed/candidate-preparation-failure.json" >/dev/null
jq -e '
  .status == "clean"
  and .cloud_build_ids == ["build-2"]
  ' "${lost_response_dir}.failed/candidate-cleanup.json" >/dev/null

: > "$gcloud_log"
lost_empty_dir="$(run_failure lost-empty lost-empty)"
[[ -f "${lost_empty_dir}.failed/candidate-cleanup-failures.log" \
  && ! -e "${lost_empty_dir}.failed/candidate-cleanup.json" ]]
rg -F 'an accepted submit without a response cannot yet be excluded' \
  "${lost_empty_dir}.failed/candidate-cleanup-failures.log" >/dev/null
rg -F 'retry: bash ' "$test_root/lost-empty.log" >/dev/null
if rg -F 'storage rm ' "$gcloud_log" >/dev/null \
    || rg -F 'artifacts docker images delete ' "$gcloud_log" >/dev/null; then
  echo "uncertain candidate cleanup deleted remote data" >&2
  exit 1
fi

# If create-only submit-intent publication fails, Cloud Build was never
# invoked. The durable failure receipt must say so, and cleanup must remove the
# exact reservation instead of waiting forever for a build that cannot exist.
: > "$gcloud_log"
intent_denied_dir="$(run_failure intent-denied intent-denied)"
if rg -F 'builds submit' "$gcloud_log" >/dev/null; then
  echo "submit-intent publication failure reached Cloud Build" >&2
  exit 1
fi
jq -e '
  .schema == "axiom.gcp.sift.candidate-preparation-failure.v2"
  and .submit_intent_published == false
  and .submit_started == false
  and .submit_response_received == false
  and .cloud_build_id == ""
  and .image_tag == (.git_sha + "-intent-denied-" + .acquisition_id)
' "${intent_denied_dir}.failed/candidate-preparation-failure.json" >/dev/null
jq -e '.status == "clean" and .cloud_build_ids == []' \
  "${intent_denied_dir}.failed/candidate-cleanup.json" >/dev/null
rg -F 'storage rm gs://axiom-test_cloudbuild/source/axiom-gcp-operator-intent-denied/candidate-reservation.json --if-generation-match=101 --quiet' \
  "$gcloud_log" >/dev/null

: > "$gcloud_log"
cleanup_denied_dir="$(run_failure cleanup-denied cleanup-denied)"
[[ -f "${cleanup_denied_dir}.failed/candidate-cleanup-failures.log" \
  && ! -e "${cleanup_denied_dir}.failed/candidate-cleanup.json" ]]
rg -F 'could not inventory acquisition-tagged Cloud Builds' \
  "${cleanup_denied_dir}.failed/candidate-cleanup-failures.log" >/dev/null
if rg -F 'storage rm ' "$gcloud_log" >/dev/null \
    || rg -F 'artifacts docker images delete ' "$gcloud_log" >/dev/null; then
  echo "permission-denied candidate cleanup deleted remote data" >&2
  exit 1
fi

echo "Sift candidate preflight E2E: ok"
