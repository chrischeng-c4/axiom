#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ACCEPTANCE_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
source "$ACCEPTANCE_ROOT/scripts/acceptance-lock.sh"

test_root="$(mktemp -d "${TMPDIR:-/tmp}/sift-lock-writers.XXXXXX")"
fake_bin="$test_root/bin"
real_jq="$(command -v jq)"
acquisition_id="aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
session_id="bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"

cleanup_test() {
  find "$test_root" -type f -delete
  find "$test_root" -depth -type d -empty -delete
}
trap cleanup_test EXIT INT TERM

mkdir -p "$fake_bin"
resource="$(
  acceptance_lock_manifest \
    "test-project" "writer-red" "sift" "$acquisition_id" \
    | jq '.metadata.uid="writer-lock-uid" | .metadata.resourceVersion="30"'
)"
session_resource="$(
  jq --arg session_id "$session_id" '
    .metadata.annotations["axiom.axiom.dev/cleanup-session-id"] = $session_id
    | .metadata.annotations["axiom.axiom.dev/cleanup-started-at"] = "2026-09-02T00:00:00Z"
    | .metadata.resourceVersion = "31"
  ' <<<"$resource"
)"
write_acceptance_lock_receipt \
  "$test_root/valid-lock.json" "$resource" \
  "test-project" "writer-red" "sift" "$acquisition_id"

cat > "$fake_bin/jq" <<'EOF'
#!/usr/bin/env bash
set -euo pipefail
mode="${SIFT_WRITER_FAIL_MODE:-}"
case "$mode" in
  generate)
    [[ "${1:-}" != "-n" ]] || exit 73
    ;;
  release)
    [[ "$*" != *"axiom.gcp.operator.acceptance-lock-release.v1"* ]] || exit 74
    ;;
  verify-after-rename)
    if [[ "${1:-}" == "-n" ]]; then
      "${SIFT_WRITER_REAL_JQ:?}" "$@"
      status=$?
      [[ "$status" == "0" ]] || exit "$status"
      : > "${SIFT_WRITER_GENERATED_MARKER:?}"
      exit 0
    fi
    if [[ "${1:-}" == "-e" && -f "${SIFT_WRITER_GENERATED_MARKER:?}" ]]; then
      exit 75
    fi
    ;;
esac
exec "${SIFT_WRITER_REAL_JQ:?}" "$@"
EOF
chmod +x "$fake_bin/jq"

expect_generation_failure() {
  local destination="$1"
  shift
  rm -f "$destination"
  set +e
  SIFT_WRITER_FAIL_MODE=generate \
  SIFT_WRITER_REAL_JQ="$real_jq" \
  PATH="$fake_bin:$PATH" \
    "$@"
  status=$?
  set -e
  [[ "$status" -ne 0 && ! -e "$destination" ]] || {
    echo "writer installed $destination after jq generation failed" >&2
    exit 1
  }
}

expect_generation_failure "$test_root/run-owner.json" \
  write_acceptance_run_owner "$test_root/run-owner.json" \
    "test-project" "writer-red" "sift" "$acquisition_id" \
    "$test_root/state" "$test_root/evidence" \
    "99999999" "99999999" "test-owner" \
    "dddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddd"
expect_generation_failure "$test_root/lock-intent.json" \
  write_acceptance_lock_intent "$test_root/lock-intent.json" \
    "test-project" "writer-red" "sift" "$acquisition_id"
expect_generation_failure "$test_root/lock.json" \
  write_acceptance_lock_receipt "$test_root/lock.json" "$resource" \
    "test-project" "writer-red" "sift" "$acquisition_id"
expect_generation_failure "$test_root/session-intent.json" \
  write_acceptance_cleanup_session_intent "$test_root/session-intent.json" \
    "test-project" "writer-red" "sift" "$acquisition_id" "$session_id" \
    "99999998" "test-cleanup-owner"
expect_generation_failure "$test_root/session.json" \
  write_acceptance_cleanup_session_receipt "$test_root/session.json" \
    "$session_resource" "test-project" "writer-red" "sift" \
    "$acquisition_id" "$session_id"

set +e
SIFT_WRITER_FAIL_MODE=release \
SIFT_WRITER_REAL_JQ="$real_jq" \
PATH="$fake_bin:$PATH" \
  write_acceptance_lock_release_receipt \
    "$test_root/release.json" "$test_root/valid-lock.json"
release_status=$?
set -e
[[ "$release_status" -ne 0 && ! -e "$test_root/release.json" ]] || {
  echo "release writer installed a receipt after jq failed" >&2
  exit 1
}

verification_marker="$test_root/generated.marker"
set +e
SIFT_WRITER_FAIL_MODE=verify-after-rename \
SIFT_WRITER_REAL_JQ="$real_jq" \
SIFT_WRITER_GENERATED_MARKER="$verification_marker" \
PATH="$fake_bin:$PATH" \
  write_acceptance_lock_intent "$test_root/post-verify-intent.json" \
    "test-project" "writer-red" "sift" "$acquisition_id"
verification_status=$?
set -e
[[ "$verification_status" -ne 0 \
  && ! -e "$test_root/post-verify-intent.json" ]] || {
  echo "writer retained a receipt that failed post-rename verification" >&2
  exit 1
}

temporary_files="$(find "$test_root" -type f \
  \( -name '.acceptance-lock.*' \
    -o -name '.acceptance-lock-intent.*' \
    -o -name '.acceptance-cleanup-session.*' \
    -o -name '.acceptance-cleanup-session-intent.*' \
    -o -name '.acceptance-run-owner.*' \
    -o -name '.acceptance-lock-release.*' \) -print)"
[[ -z "$temporary_files" ]] || {
  echo "writer left temporary files after failure:" >&2
  echo "$temporary_files" >&2
  exit 1
}

echo "acceptance lock writer failure E2E: ok"
