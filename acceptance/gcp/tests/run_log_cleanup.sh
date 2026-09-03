#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ACCEPTANCE_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
source "$ACCEPTANCE_ROOT/scripts/process-tree.sh"
source "$ACCEPTANCE_ROOT/scripts/run-log.sh"

test_root="$(mktemp -d "${TMPDIR:-/tmp}/sift-run-log.XXXXXX")"
cleanup_test() {
  finish_run_log >/dev/null 2>&1 || true
  find "$test_root" -type f -delete
  find "$test_root" -depth -type d -empty -delete
}
trap cleanup_test EXIT INT TERM

log_path="$test_root/run.log"
record_path="$test_root/run-log-process.txt"
pipe_path="$test_root/run-log.pipe"
cleanup_marker="$test_root/cleanup-complete.txt"

start_run_log "$log_path" "$record_path" "$pipe_path"
tee_pid="$RUN_LOG_TEE_PID"
tee_token="$RUN_LOG_TEE_TOKEN"
process_record_is_excluded "$record_path" "$tee_pid" "$tee_token"
! process_record_is_excluded "$record_path" "$tee_pid" "wrong-generation"
echo "before cleanup"
printf '%s\n' "complete" > "$cleanup_marker"
echo "cleanup complete"
finish_run_log

[[ -s "$cleanup_marker" ]]
rg -q '^before cleanup$' "$log_path"
rg -q '^cleanup complete$' "$log_path"
if kill -0 "$tee_pid" >/dev/null 2>&1; then
  echo "run log sink remained alive after cleanup completed" >&2
  exit 1
fi

# A background child inherits the FIFO writer. Parent detachment must return
# immediately, keep the sink alive, and let cleanup stop the child first.
writer_log="$test_root/background-writer.log"
writer_record="$test_root/background-writer-process.txt"
writer_pipe="$test_root/background-writer.pipe"
writer_marker="$test_root/background-writer-cleanup-complete.txt"
start_run_log "$writer_log" "$writer_record" "$writer_pipe"
writer_tee_pid="$RUN_LOG_TEE_PID"
sleep 30 &
writer_pid="$!"
writer_token="$(process_start_token "$writer_pid")"
detach_started="$SECONDS"
detach_run_log_for_cleanup
detach_elapsed=$((SECONDS - detach_started))
if (( detach_elapsed >= 3 )); then
  echo "parent run-log detachment waited for a descendant FIFO writer" >&2
  exit 1
fi
run_log_generation_state
printf '%s\n' "complete" > "$writer_marker"
echo "cleanup detached before stopping the background writer"
stop_process_generation_bounded "$writer_pid" "$writer_token"
close_run_log_sink
finish_run_log
[[ -s "$writer_marker" ]]
rg -q '^cleanup detached before stopping the background writer$' "$writer_log"
if kill -0 "$writer_tee_pid" >/dev/null 2>&1; then
  echo "run log sink remained alive after the background writer stopped" >&2
  exit 1
fi

# A deleted exclusion record must fail the run without exposing cleanup to the
# FIFO. The exact sink generation still stops, and later output uses run.log.
missing_log="$test_root/missing-record.log"
missing_record="$test_root/missing-record-process.txt"
missing_pipe="$test_root/missing-record.pipe"
missing_marker="$test_root/missing-record-cleanup-complete.txt"
start_run_log "$missing_log" "$missing_record" "$missing_pipe"
missing_tee_pid="$RUN_LOG_TEE_PID"
rm -f "$missing_record"
if detach_run_log_for_cleanup; then
  echo "deleted run-log exclusion record did not fail closed" >&2
  exit 1
fi
printf '%s\n' "complete" > "$missing_marker"
echo "cleanup survived a deleted exclusion record"
finish_run_log
[[ -s "$missing_marker" ]]
rg -q '^cleanup survived a deleted exclusion record$' "$missing_log"
if kill -0 "$missing_tee_pid" >/dev/null 2>&1; then
  echo "run log sink remained alive after its exclusion record was deleted" >&2
  exit 1
fi

# If the sink dies after a successful liveness check, cleanup must restore the
# saved output before it prints or mutates anything else.
fallback_log="$test_root/fallback.log"
fallback_record="$test_root/fallback-process.txt"
fallback_pipe="$test_root/fallback.pipe"
fallback_marker="$test_root/fallback-cleanup-complete.txt"
start_run_log "$fallback_log" "$fallback_record" "$fallback_pipe"
fallback_tee_pid="$RUN_LOG_TEE_PID"
run_log_sink_is_current
signal_process_generation "$fallback_tee_pid" "$RUN_LOG_TEE_TOKEN" TERM
fallback_detach_status=0
detach_run_log_for_cleanup || fallback_detach_status=$?
printf '%s\n' "complete" > "$fallback_marker"
echo "cleanup survived an early sink exit"
fallback_finish_status=0
finish_run_log || fallback_finish_status=$?
if [[ "$fallback_detach_status" == "0" && "$fallback_finish_status" == "0" ]]; then
  echo "early run-log sink exit did not fail closed" >&2
  exit 1
fi
[[ -s "$fallback_marker" ]]
rg -q '^cleanup survived an early sink exit$' "$fallback_log"

# A reader that ignores EOF and TERM must not make finish_run_log wait without
# a deadline. The exact recorded generation is killed after the grace period.
stalled_log="$test_root/stalled.log"
stalled_record="$test_root/stalled-process.txt"
stalled_pipe="$test_root/stalled.pipe"
stalled_marker="$test_root/stalled-cleanup-complete.txt"
start_run_log "$stalled_log" "$stalled_record" "$stalled_pipe" \
  python3 -c 'import signal,time; signal.signal(signal.SIGTERM, signal.SIG_IGN); time.sleep(60)'
stalled_tee_pid="$RUN_LOG_TEE_PID"
finish_started="$SECONDS"
if finish_run_log; then
  echo "stalled run-log reader did not fail the run" >&2
  exit 1
fi
finish_elapsed=$((SECONDS - finish_started))
if (( finish_elapsed >= 10 )); then
  echo "stalled run-log reader exceeded its shutdown deadline" >&2
  exit 1
fi
printf '%s\n' "complete" > "$stalled_marker"
[[ -s "$stalled_marker" ]]
if kill -0 "$stalled_tee_pid" >/dev/null 2>&1; then
  echo "stalled run-log reader remained alive after bounded shutdown" >&2
  exit 1
fi

# A live PID with the wrong birth token belongs to another generation. The
# stale record must fail quickly, must not signal that process, and must not
# wait for it.
sleep 30 &
reused_pid="$!"
reused_token="$(process_start_token "$reused_pid")"
RUN_LOG_TEE_PID="$reused_pid"
RUN_LOG_TEE_TOKEN="wrong-generation"
RUN_LOG_RECEIPT="$test_root/stale-generation.complete"
RUN_LOG_RECEIPT_NONCE="expected-nonce"
RUN_LOG_RECEIPT_REQUIRED=1
printf '%s\n' $'complete\tdifferent-nonce' > "$RUN_LOG_RECEIPT"
stale_started="$SECONDS"
if stop_run_log_tee; then
  echo "stale run-log generation was accepted" >&2
  exit 1
fi
stale_elapsed=$((SECONDS - stale_started))
if (( stale_elapsed >= 3 )); then
  echo "stale run-log generation waited for a reused PID" >&2
  exit 1
fi
kill -0 "$reused_pid"
stop_process_generation_bounded "$reused_pid" "$reused_token"
reset_run_log_state

echo "acceptance run log cleanup E2E: ok"
