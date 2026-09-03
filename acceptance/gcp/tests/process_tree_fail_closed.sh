#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ACCEPTANCE_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
source "$ACCEPTANCE_ROOT/scripts/process-tree.sh"

test_root="$(mktemp -d "${TMPDIR:-/tmp}/sift-process-scan.XXXXXX")"
fake_bin="$test_root/bin"
records="$test_root/records.txt"
calls="$test_root/mutations.log"
real_process_helper="$ACCEPTANCE_ROOT/scripts/process-start-token.py"
bounded_pid=""
reused_pid=""
no_stop_pid=""
innocent_pid=""
exited_history_pid=""
escape_anchor_pid=""
escape_child_pid=""
late_creator_pid=""
late_child_pid=""
cleanup_test() {
  [[ -z "$bounded_pid" ]] || kill -KILL "$bounded_pid" >/dev/null 2>&1 || true
  [[ -z "$reused_pid" ]] || kill -KILL "$reused_pid" >/dev/null 2>&1 || true
  [[ -z "$no_stop_pid" ]] \
    || kill -KILL "$no_stop_pid" >/dev/null 2>&1 || true
  [[ -z "$innocent_pid" ]] \
    || kill -KILL "$innocent_pid" >/dev/null 2>&1 || true
  [[ -z "$exited_history_pid" ]] \
    || kill -KILL "$exited_history_pid" >/dev/null 2>&1 || true
  [[ -z "$escape_anchor_pid" ]] \
    || kill -KILL "$escape_anchor_pid" >/dev/null 2>&1 || true
  [[ -z "$escape_child_pid" ]] \
    || kill -KILL "$escape_child_pid" >/dev/null 2>&1 || true
  [[ -z "$late_creator_pid" ]] \
    || kill -KILL "$late_creator_pid" >/dev/null 2>&1 || true
  [[ -z "$late_child_pid" ]] \
    || kill -KILL "$late_child_pid" >/dev/null 2>&1 || true
  rm -rf "$test_root"
}
trap cleanup_test EXIT INT TERM
mkdir -p "$fake_bin"
printf '12345\told-generation\n' > "$records"
: > "$calls"

cat > "$fake_bin/process-snapshot" <<'EOF'
#!/usr/bin/env bash
if [[ "${1:-}" == "--snapshot" ]]; then
  exit 17
fi
exec "${SIFT_REAL_PROCESS_HELPER:?}" "$@"
EOF
for command in gcloud kubectl terraform; do
  cat > "$fake_bin/$command" <<'EOF'
#!/usr/bin/env bash
printf '%s %s\n' "${0##*/}" "$*" >> "${SIFT_PROCESS_SCAN_CALLS:?}"
EOF
done
chmod +x "$fake_bin/process-snapshot" "$fake_bin/gcloud" "$fake_bin/kubectl" \
  "$fake_bin/terraform"

before="$(openssl dgst -sha256 "$records" | awk '{print $NF}')"
if PATH="$fake_bin:$PATH" \
    PROCESS_SNAPSHOT_HELPER="$fake_bin/process-snapshot" \
    SIFT_REAL_PROCESS_HELPER="$real_process_helper" \
    SIFT_PROCESS_SCAN_CALLS="$calls" \
    append_process_group_members "$$" "$$" "" "$records"; then
  PATH="$fake_bin:$PATH" SIFT_PROCESS_SCAN_CALLS="$calls" \
    gcloud builds cancel unsafe
  PATH="$fake_bin:$PATH" SIFT_PROCESS_SCAN_CALLS="$calls" \
    kubectl delete namespace unsafe
  PATH="$fake_bin:$PATH" SIFT_PROCESS_SCAN_CALLS="$calls" \
    terraform destroy
  echo "process-group enumeration failure returned success" >&2
  exit 1
fi
after="$(openssl dgst -sha256 "$records" | awk '{print $NF}')"
[[ "$before" == "$after" ]] || {
  echo "a failed process scan replaced the last complete process record" >&2
  exit 1
}
[[ ! -s "$calls" ]] || {
  echo "an incomplete process scan reached a destructive command" >&2
  cat "$calls" >&2
  exit 1
}

# A partial sort or failed final rename must not replace the last complete
# exact-generation record. These commands run inside a conditional caller, so
# each write step must report its own failure.
write_failure_bin="$test_root/write-failure-bin"
real_sort="$(command -v sort)"
real_mv="$(command -v mv)"
mkdir -p "$write_failure_bin"
cat > "$write_failure_bin/sort" <<'EOF'
#!/usr/bin/env bash
if [[ "${SIFT_FAIL_PROCESS_SORT:-0}" == "1" ]]; then
  printf '99999\tpartial-generation\n'
  exit 19
fi
exec "${SIFT_REAL_SORT:?}" "$@"
EOF
cat > "$write_failure_bin/mv" <<'EOF'
#!/usr/bin/env bash
destination=""
for argument in "$@"; do
  destination="$argument"
done
if [[ -n "${SIFT_FAIL_PROCESS_MV_DEST:-}" \
  && "$destination" == "$SIFT_FAIL_PROCESS_MV_DEST" ]]; then
  exit 23
fi
exec "${SIFT_REAL_MV:?}" "$@"
EOF
chmod +x "$write_failure_bin/sort" \
  "$write_failure_bin/mv"

printf '12345\told-generation\n' > "$records"
before="$(openssl dgst -sha256 "$records" | awk '{print $NF}')"
if PATH="$write_failure_bin:$PATH" \
    SIFT_FAIL_PROCESS_SORT=1 \
    SIFT_REAL_SORT="$real_sort" \
    SIFT_REAL_MV="$real_mv" \
    append_process_group_members "999999" "$$" "" "$records"; then
  echo "a partial process-record sort returned success" >&2
  exit 1
fi
after="$(openssl dgst -sha256 "$records" | awk '{print $NF}')"
[[ "$before" == "$after" ]] || {
  echo "a failed sort replaced the last complete process record" >&2
  exit 1
}

# A failed marker rename cannot suppress the watchdog's owner-termination
# attempt. The helper must report failure after it makes that attempt.
marker_failure="$test_root/process-scan-unsafe.txt"
marker_signal_log="$test_root/process-scan-signal.txt"
marker_status=0
(
  signal_process_generation() {
    printf '%s\t%s\t%s\n' "$1" "$2" "$3" > "$marker_signal_log"
  }
  PATH="$write_failure_bin:$PATH" \
  SIFT_FAIL_PROCESS_MV_DEST="$marker_failure" \
  SIFT_REAL_MV="$real_mv" \
  report_process_scan_failure \
    "$marker_failure" "injected snapshot failure" "$$" "test-owner-token"
) || marker_status=$?
[[ "$marker_status" != "0" && -s "$marker_signal_log" ]] || {
  echo "a failed watchdog marker write suppressed owner termination" >&2
  exit 1
}

# The snapshot carries the start token in the same kernel record as PID, PPID,
# and PGID. A stale row must never be rebound to the current process generation.
stale_snapshot_bin="$test_root/stale-snapshot-bin"
stale_snapshot="$test_root/stale-snapshot.txt"
stale_records="$test_root/stale-records.txt"
mkdir -p "$stale_snapshot_bin"
sleep 30 &
innocent_pid="$!"
innocent_token="$(process_start_token "$innocent_pid")"
cat > "$stale_snapshot_bin/process-snapshot" <<'EOF'
#!/usr/bin/env bash
if [[ "${1:-}" == "--snapshot" ]]; then
  cat "${SIFT_STALE_SNAPSHOT:?}"
  exit 0
fi
exec "${SIFT_REAL_PROCESS_HELPER:?}" "$@"
EOF
chmod +x "$stale_snapshot_bin/process-snapshot"
printf '%s\t1\t777777\tstale-generation\n' "$innocent_pid" > "$stale_snapshot"
PROCESS_SNAPSHOT_HELPER="$stale_snapshot_bin/process-snapshot" \
SIFT_STALE_SNAPSHOT="$stale_snapshot" \
SIFT_REAL_PROCESS_HELPER="$real_process_helper" \
record_process_group_members "777777" "" "" "$stale_records"
if rg -q "^${innocent_pid}"$'\t' "$stale_records"; then
  echo "a stale process snapshot rebound an unrelated live generation" >&2
  exit 1
fi
kill -0 "$innocent_pid"
stop_process_generation_bounded "$innocent_pid" "$innocent_token"
wait "$innocent_pid" >/dev/null 2>&1 || true
innocent_pid=""

# An exact historical generation can exit before the freeze loop reaches it.
# That is stable progress, not a new generation that keeps the loop open.
exited_records="$test_root/exited-history-records.txt"
sleep 30 &
exited_history_pid="$!"
exited_history_token="$(process_start_token "$exited_history_pid")"
printf '%s\t%s\n' "$exited_history_pid" "$exited_history_token" > "$exited_records"
kill -KILL "$exited_history_pid"
wait "$exited_history_pid" >/dev/null 2>&1 || true
exited_history_pid=""
shutdown_process_group_members \
  "777777" "" "" "$exited_records" "" 2 0 || {
  echo "an exited historical generation prevented freeze convergence" >&2
  exit 1
}

printf '12345\told-generation\n' > "$records"
before="$(openssl dgst -sha256 "$records" | awk '{print $NF}')"
if PATH="$write_failure_bin:$PATH" \
    SIFT_FAIL_PROCESS_MV_DEST="$records" \
    SIFT_REAL_SORT="$real_sort" \
    SIFT_REAL_MV="$real_mv" \
    append_process_group_members "999999" "$$" "" "$records"; then
  echo "a failed process-record rename returned success" >&2
  exit 1
fi
after="$(openssl dgst -sha256 "$records" | awk '{print $NF}')"
[[ "$before" == "$after" ]] || {
  echo "a failed rename replaced the last complete process record" >&2
  exit 1
}

# A stopped process cannot act on TERM. The helper must escalate to KILL and
# still finish within its fixed deadline.
sleep 30 &
bounded_pid="$!"
bounded_token="$(process_start_token "$bounded_pid")"
signal_process_generation "$bounded_pid" "$bounded_token" STOP
bounded_started="$SECONDS"
stop_process_generation_bounded "$bounded_pid" "$bounded_token"
bounded_elapsed=$((SECONDS - bounded_started))
if (( bounded_elapsed >= 7 )); then
  echo "bounded process shutdown exceeded its deadline" >&2
  exit 1
fi
if process_generation_state "$bounded_pid" "$bounded_token"; then
  echo "bounded process shutdown left the recorded generation alive" >&2
  exit 1
else
  bounded_state=$?
  [[ "$bounded_state" == "1" ]] || {
    echo "bounded process shutdown left an unverifiable generation" >&2
    exit 1
  }
fi
bounded_pid=""

# A matching PID with a different birth token is a different process. It must
# not receive a signal and the helper must not wait for it.
sleep 30 &
reused_pid="$!"
reused_token="$(process_start_token "$reused_pid")"
reused_started="$SECONDS"
stop_process_generation_bounded "$reused_pid" "wrong-generation"
reused_elapsed=$((SECONDS - reused_started))
if (( reused_elapsed >= 2 )); then
  echo "stale process generation waited for a reused PID" >&2
  exit 1
fi
kill -0 "$reused_pid"
stop_process_generation_bounded "$reused_pid" "$reused_token"
reused_pid=""

# Mutation oracle: if STOP is turned into a no-op, the helper must refuse
# cleanup. A scan-and-KILL implementation without stopped-state confirmation
# would incorrectly pass this fixture.
no_stop_records="$test_root/no-stop-records.txt"
python3 -c 'import os; os.setpgid(0, 0); os.execlp("sleep", "sleep", "30")' &
no_stop_pid="$!"
no_stop_token="$(process_start_token "$no_stop_pid")"
no_stop_group_ready=0
for _ in {1..20}; do
  if [[ "$(process_group_id "$no_stop_pid")" == "$no_stop_pid" ]]; then
    no_stop_group_ready=1
    break
  fi
  sleep 0.05
done
[[ "$no_stop_group_ready" == "1" ]]
no_stop_status=0
(
  signal_process_generation() {
    [[ "$3" == "STOP" ]] && return 0
    return 2
  }
  shutdown_process_group_members \
    "$no_stop_pid" "" "" "$no_stop_records" "" 2 0 \
    || exit "$?"
) || no_stop_status=$?
[[ "$no_stop_status" == "2" ]] || {
  echo "cleanup accepted a creator that was not confirmed stopped" >&2
  exit 1
}
stop_process_generation_bounded "$no_stop_pid" "$no_stop_token"
wait "$no_stop_pid" >/dev/null 2>&1 || true
no_stop_pid=""

# A previously recorded child can leave the original group with setsid(). It
# remains owned by this run. Historical exact-generation records must still
# stop it before destructive cloud cleanup begins.
escape_trigger="$test_root/escape-trigger.txt"
escape_ready="$test_root/escape-ready.txt"
escape_child_file="$test_root/escape-child-pid.txt"
escape_done="$test_root/escape-done.txt"
escape_records="$test_root/escape-records.txt"
SIFT_ESCAPE_TRIGGER="$escape_trigger" \
SIFT_ESCAPE_READY="$escape_ready" \
SIFT_ESCAPE_CHILD="$escape_child_file" \
SIFT_ESCAPE_DONE="$escape_done" \
python3 -c '
import os
from pathlib import Path
import signal
import time

os.setpgid(0, 0)
signal.signal(signal.SIGCHLD, signal.SIG_IGN)
child = os.fork()
if child == 0:
    trigger = Path(os.environ["SIFT_ESCAPE_TRIGGER"])
    while not trigger.exists():
        time.sleep(0.005)
    os.setsid()
    Path(os.environ["SIFT_ESCAPE_DONE"]).write_text("complete", encoding="ascii")
    time.sleep(30)
    raise SystemExit(0)
Path(os.environ["SIFT_ESCAPE_CHILD"]).write_text(str(child), encoding="ascii")
Path(os.environ["SIFT_ESCAPE_READY"]).write_text("complete", encoding="ascii")
time.sleep(30)
' >/dev/null 2>&1 &
escape_anchor_pid="$!"
escape_anchor_token="$(process_start_token "$escape_anchor_pid")"
escape_group_ready=0
for _ in {1..40}; do
  if [[ -s "$escape_ready" && -s "$escape_child_file" \
    && "$(process_group_id "$escape_anchor_pid")" == "$escape_anchor_pid" ]]; then
    escape_group_ready=1
    break
  fi
  sleep 0.05
done
[[ "$escape_group_ready" == "1" ]]
escape_child_pid="$(<"$escape_child_file")"
escape_child_token="$(process_start_token "$escape_child_pid")"
record_process_group_members \
  "$escape_anchor_pid" "$escape_anchor_pid" "" "$escape_records"
rg -q "^${escape_child_pid}"$'\t'"${escape_child_token}$" "$escape_records"
: > "$escape_trigger"
for _ in {1..40}; do
  [[ -s "$escape_done" \
    && "$(process_group_id "$escape_child_pid")" == "$escape_child_pid" ]] \
    && break
  sleep 0.05
done
[[ -s "$escape_done" \
  && "$(process_group_id "$escape_child_pid")" == "$escape_child_pid" ]]
shutdown_process_group_members \
  "$escape_anchor_pid" "$escape_anchor_pid" "" "$escape_records" "" 8 0.05
if process_generation_state "$escape_child_pid" "$escape_child_token"; then
  echo "cleanup left a historically recorded escaped child alive" >&2
  exit 1
else
  escape_child_state=$?
  [[ "$escape_child_state" == "1" ]]
fi
escape_child_pid=""
stop_process_generation_bounded "$escape_anchor_pid" "$escape_anchor_token"
wait "$escape_anchor_pid" >/dev/null 2>&1 || true
escape_anchor_pid=""

# A creator forks after the first `ps` snapshot has already been captured. The
# child immediately starts a new session, so a later process-group-only scan
# cannot find it. The ancestry scan must still find and stop both generations.
late_bin="$test_root/late-bin"
late_counter="$test_root/late-scan-count.txt"
late_trigger="$test_root/late-trigger.txt"
late_ready="$test_root/late-creator-ready.txt"
late_pid_file="$test_root/late-child-pid.txt"
late_escape_file="$test_root/late-child-escaped.txt"
late_records="$test_root/late-records.txt"
mkdir -p "$late_bin"
printf '0\n' > "$late_counter"
cat > "$late_bin/process-snapshot" <<'EOF'
#!/usr/bin/env bash
set -euo pipefail
if [[ "${1:-}" != "--snapshot" ]]; then
  exec "${SIFT_REAL_PROCESS_HELPER:?}" "$@"
fi
count="$(( $(<"${SIFT_LATE_SCAN_COUNTER:?}") + 1 ))"
printf '%s\n' "$count" > "$SIFT_LATE_SCAN_COUNTER"
snapshot="${SIFT_LATE_SCAN_COUNTER}.snapshot.$$"
"${SIFT_REAL_PROCESS_HELPER:?}" --snapshot > "$snapshot"
if [[ "$count" == "1" ]]; then
  : > "$SIFT_LATE_SCAN_TRIGGER"
  for _ in {1..100}; do
    [[ -s "$SIFT_LATE_SCAN_PID_FILE" ]] && break
    sleep 0.01
  done
  [[ -s "$SIFT_LATE_SCAN_PID_FILE" ]] || exit 18
  late_pid="$(<"$SIFT_LATE_SCAN_PID_FILE")"
  late_group="$(/bin/ps -o pgid= -p "$late_pid" \
    | sed 's/^[[:space:]]*//;s/[[:space:]]*$//')"
  [[ "$late_group" == "$late_pid" ]] || exit 19
  printf 'complete\n' > "$SIFT_LATE_SCAN_ESCAPE_FILE"
fi
cat "$snapshot"
rm -f "$snapshot"
EOF
chmod +x "$late_bin/process-snapshot"
SIFT_LATE_CREATOR_TRIGGER="$late_trigger" \
SIFT_LATE_CREATOR_READY="$late_ready" \
SIFT_LATE_CREATOR_CHILD="$late_pid_file" \
python3 -c '
import os
from pathlib import Path
import subprocess
import time

os.setpgid(0, 0)
Path(os.environ["SIFT_LATE_CREATOR_READY"]).write_text(str(os.getpid()), encoding="ascii")
trigger = Path(os.environ["SIFT_LATE_CREATOR_TRIGGER"])
while not trigger.exists():
    time.sleep(0.005)
child = subprocess.Popen(["sleep", "30"], preexec_fn=os.setsid)
Path(os.environ["SIFT_LATE_CREATOR_CHILD"]).write_text(str(child.pid), encoding="ascii")
time.sleep(30)
' &
late_creator_pid="$!"
late_creator_token="$(process_start_token "$late_creator_pid")"
late_group_ready=0
for _ in 1 2 3 4 5 6 7 8 9 10 11 12 13 14 15 16 17 18 19 20; do
  if [[ -s "$late_ready" \
    && "$(process_group_id "$late_creator_pid")" == "$late_creator_pid" ]]; then
    late_group_ready=1
    break
  fi
  sleep 0.05
done
[[ "$late_group_ready" == "1" ]]
late_status=0
PROCESS_SNAPSHOT_HELPER="$late_bin/process-snapshot" \
SIFT_REAL_PROCESS_HELPER="$real_process_helper" \
SIFT_LATE_SCAN_COUNTER="$late_counter" \
SIFT_LATE_SCAN_TRIGGER="$late_trigger" \
SIFT_LATE_SCAN_PID_FILE="$late_pid_file" \
SIFT_LATE_SCAN_ESCAPE_FILE="$late_escape_file" \
shutdown_process_group_members \
  "$late_creator_pid" "" "" "$late_records" "" 8 0.05 \
  || late_status=$?
wait "$late_creator_pid" >/dev/null 2>&1 || true
[[ "$late_status" == "0" && -s "$late_pid_file" \
  && -s "$late_escape_file" ]] || {
  echo "the ancestry cleanup missed a late setsid child" >&2
  exit 1
}
late_child_pid="$(<"$late_pid_file")"
rg -q "^${late_child_pid}"$'\t' "$late_records"
if process_generation_state "$late_creator_pid" "$late_creator_token"; then
  echo "the late creator survived the frozen cleanup" >&2
  exit 1
else
  late_creator_state=$?
  [[ "$late_creator_state" == "1" ]]
fi
if process_start_token "$late_child_pid" >/dev/null 2>&1; then
  echo "the post-snapshot setsid child survived the ancestry cleanup" >&2
  exit 1
else
  late_child_state=$?
  [[ "$late_child_state" == "1" ]]
fi
late_creator_pid=""
late_child_pid=""

echo "process-tree fail-closed E2E: ok"
