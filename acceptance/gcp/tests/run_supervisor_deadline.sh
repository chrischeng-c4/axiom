#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ACCEPTANCE_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
source "$ACCEPTANCE_ROOT/scripts/process-tree.sh"

test_root="$(mktemp -d "${TMPDIR:-/tmp}/sift-run-supervisor.XXXXXX")"
supervisor_pid=""
descendant_pid=""
normal_ready="$test_root/normal-ready.txt"
normal_token="normal-token"
cleanup_test() {
  [[ -z "$supervisor_pid" ]] \
    || kill -KILL "$supervisor_pid" >/dev/null 2>&1 || true
  [[ -z "$descendant_pid" ]] \
    || kill -KILL "$descendant_pid" >/dev/null 2>&1 || true
  find "$test_root" -type f -delete
  find "$test_root" -depth -type d -empty -delete
}
trap cleanup_test EXIT INT TERM

set +e
python3 "$ACCEPTANCE_ROOT/scripts/run-supervisor.py" \
  --preflight-deadline-seconds 5 \
  --deadline-seconds 5 --shutdown-grace-seconds 1 \
  --ready-path "$normal_ready" --ready-token "$normal_token" -- \
  bash -c 'printf "complete\t%s\n" "$2" > "$1.tmp"; mv "$1.tmp" "$1"; sleep 0.1; exit 7' \
    _ "$normal_ready" "$normal_token"
normal_status=$?
set -e
[[ "$normal_status" == "7" ]] || {
  echo "run supervisor did not preserve a normal child failure" >&2
  exit 1
}

missing_success_ready="$test_root/missing-success-ready.txt"
set +e
python3 "$ACCEPTANCE_ROOT/scripts/run-supervisor.py" \
  --preflight-deadline-seconds 2 \
  --deadline-seconds 5 --shutdown-grace-seconds 1 \
  --ready-path "$missing_success_ready" --ready-token "missing-token" -- \
  bash -c 'exit 0' \
  >"$test_root/missing-success.stdout" 2>"$test_root/missing-success.stderr"
missing_success_status=$?
set -e
[[ "$missing_success_status" == "125" ]] || {
  echo "run supervisor accepted success without cloud readiness" >&2
  exit 1
}
rg -q 'without a valid cloud-ready receipt' "$test_root/missing-success.stderr"

ready_marker="$test_root/watchdog-was-ready.txt"
cloud_ready="$test_root/cloud-ready.txt"
cloud_token="cloud-token"
descendant_file="$test_root/descendant-pid.txt"
timeout_started="$SECONDS"
READY_MARKER="$ready_marker" DESCENDANT_FILE="$descendant_file" \
CLOUD_READY="$cloud_ready" CLOUD_TOKEN="$cloud_token" \
python3 "$ACCEPTANCE_ROOT/scripts/run-supervisor.py" \
  --preflight-deadline-seconds 5 \
  --deadline-seconds 1 --shutdown-grace-seconds 1 \
  --ready-path "$cloud_ready" --ready-token "$cloud_token" -- \
  bash -c '
    trap "" TERM
    (exit 0) &
    wait $!
    sleep 2
    printf "complete\t%s\n" "$CLOUD_TOKEN" > "$CLOUD_READY.tmp"
    mv "$CLOUD_READY.tmp" "$CLOUD_READY"
    printf "%s\n" ready > "$READY_MARKER"
    sleep 30 &
    printf "%s\n" "$!" > "$DESCENDANT_FILE"
    wait
  ' >"$test_root/timeout.stdout" 2>"$test_root/timeout.stderr" &
supervisor_pid="$!"
for _ in {1..100}; do
  [[ -s "$ready_marker" && -s "$descendant_file" ]] && break
  sleep 0.05
done
[[ -s "$ready_marker" && -s "$descendant_file" ]] || {
  echo "deadline fixture did not reach its ready state" >&2
  exit 1
}
descendant_pid="$(<"$descendant_file")"
descendant_token="$(process_start_token "$descendant_pid")"
set +e
wait "$supervisor_pid"
timeout_status=$?
set -e
supervisor_pid=""
timeout_elapsed=$((SECONDS - timeout_started))
[[ "$timeout_status" == "124" ]] || {
  echo "run supervisor did not return timeout status 124" >&2
  exit 1
}
if (( timeout_elapsed < 3 )); then
  echo "preflight time was incorrectly subtracted from the cloud deadline" >&2
  exit 1
fi
if (( timeout_elapsed >= 8 )); then
  echo "run supervisor exceeded its deadline and cleanup grace" >&2
  exit 1
fi
if process_generation_state "$descendant_pid" "$descendant_token"; then
  echo "run supervisor left a descendant alive after cleanup grace" >&2
  exit 1
else
  descendant_state=$?
  [[ "$descendant_state" == "1" ]] || {
    echo "run supervisor left an unverifiable descendant" >&2
    exit 1
  }
fi
descendant_pid=""
rg -q 'cloud deadline reached' "$test_root/timeout.stderr"
rg -q 'cleanup grace expired' "$test_root/timeout.stderr"

preflight_ready="$test_root/preflight-ready.txt"
set +e
python3 "$ACCEPTANCE_ROOT/scripts/run-supervisor.py" \
  --preflight-deadline-seconds 1 \
  --deadline-seconds 5 --shutdown-grace-seconds 1 \
  --ready-path "$preflight_ready" --ready-token "preflight-token" -- \
  bash -c 'trap "" TERM; sleep 30' \
  >"$test_root/preflight.stdout" 2>"$test_root/preflight.stderr"
preflight_status=$?
set -e
[[ "$preflight_status" == "124" ]] || {
  echo "run supervisor did not enforce the separate preflight deadline" >&2
  exit 1
}
rg -q 'preflight deadline reached' "$test_root/preflight.stderr"
if rg -q 'cloud deadline reached' "$test_root/preflight.stderr"; then
  echo "preflight timeout incorrectly started the cloud deadline" >&2
  exit 1
fi

invalid_ready="$test_root/invalid-ready.txt"
set +e
INVALID_READY="$invalid_ready" \
python3 "$ACCEPTANCE_ROOT/scripts/run-supervisor.py" \
  --preflight-deadline-seconds 5 \
  --deadline-seconds 5 --shutdown-grace-seconds 1 \
  --ready-path "$invalid_ready" --ready-token "expected-token" -- \
  bash -c 'trap "" TERM; printf "complete\twrong-token\n" > "$INVALID_READY.tmp"; mv "$INVALID_READY.tmp" "$INVALID_READY"; sleep 30' \
  >"$test_root/invalid.stdout" 2>"$test_root/invalid.stderr"
invalid_status=$?
set -e
[[ "$invalid_status" == "125" ]] || {
  echo "run supervisor accepted an invalid cloud-ready receipt" >&2
  exit 1
}
rg -q 'invalid cloud-ready receipt' "$test_root/invalid.stderr"

echo "acceptance run supervisor deadline E2E: ok"
