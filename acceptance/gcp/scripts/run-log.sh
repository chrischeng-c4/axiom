#!/usr/bin/env bash

# Preserve acceptance output without letting a failed log pipe interrupt
# destructive cleanup. The caller must source process-tree.sh first.

RUN_LOG_SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
RUN_LOG_SINK_HELPER="${RUN_LOG_SINK_HELPER:-$RUN_LOG_SCRIPT_DIR/run-log-sink.py}"

RUN_LOG_TEE_PID="${RUN_LOG_TEE_PID:-}"
RUN_LOG_TEE_TOKEN="${RUN_LOG_TEE_TOKEN:-}"
RUN_LOG_PROCESS_RECORD="${RUN_LOG_PROCESS_RECORD:-}"
RUN_LOG_PIPE="${RUN_LOG_PIPE:-}"
RUN_LOG_PATH="${RUN_LOG_PATH:-}"
RUN_LOG_OUTPUT_ACTIVE="${RUN_LOG_OUTPUT_ACTIVE:-0}"
RUN_LOG_DIRECT_OUTPUT_ACTIVE="${RUN_LOG_DIRECT_OUTPUT_ACTIVE:-0}"
RUN_LOG_FDS_SAVED="${RUN_LOG_FDS_SAVED:-0}"
RUN_LOG_RECEIPT="${RUN_LOG_RECEIPT:-}"
RUN_LOG_RECEIPT_NONCE="${RUN_LOG_RECEIPT_NONCE:-}"
RUN_LOG_RECEIPT_REQUIRED="${RUN_LOG_RECEIPT_REQUIRED:-0}"

run_log_generation_state() {
  process_generation_state "$RUN_LOG_TEE_PID" "$RUN_LOG_TEE_TOKEN"
}

run_log_wait_until_absent() {
  local attempts="$1"
  wait_for_process_generation_absent \
    "$RUN_LOG_TEE_PID" "$RUN_LOG_TEE_TOKEN" "$attempts"
}

run_log_signal_current_generation() {
  local signal="$1"
  signal_process_generation "$RUN_LOG_TEE_PID" "$RUN_LOG_TEE_TOKEN" "$signal"
}

# Return 0 after a normal EOF shutdown. Return 1 if the exact generation is gone
# but failed or needed a signal. Return 2 if it remains live or unverifiable.
# This function never waits without a deadline.
stop_run_log_tee() {
  local pid="$RUN_LOG_TEE_PID"
  local state forced=0 receipt_status=0
  local graceful_attempts=20 forced_attempts=5
  [[ -n "$pid" ]] || return 0

  if run_log_wait_until_absent "$graceful_attempts"; then
    :
  else
    state=$?
    if [[ "$state" == "2" ]]; then
      return 2
    fi
    forced=1
    run_log_signal_current_generation TERM || return 2
    if run_log_wait_until_absent "$forced_attempts"; then
      :
    else
      state=$?
      if [[ "$state" == "2" ]]; then
        return 2
      fi
      run_log_signal_current_generation KILL || return 2
      run_log_wait_until_absent "$forced_attempts" || return 2
    fi
  fi

  if [[ "$RUN_LOG_RECEIPT_REQUIRED" == "1" ]]; then
    if [[ -f "$RUN_LOG_RECEIPT" && ! -L "$RUN_LOG_RECEIPT" ]] \
        && [[ "$(sed -n '1p' "$RUN_LOG_RECEIPT")" \
          == $'complete\t'"$RUN_LOG_RECEIPT_NONCE" ]]; then
      receipt_status=1
    fi
  fi
  RUN_LOG_TEE_PID=""
  RUN_LOG_TEE_TOKEN=""
  [[ "$forced" == "0" && "$receipt_status" == "1" ]]
}

run_log_restore_original_output() {
  local restore_status=0
  if [[ "$RUN_LOG_OUTPUT_ACTIVE" == "1" \
    || "$RUN_LOG_DIRECT_OUTPUT_ACTIVE" == "1" ]]; then
    if exec 1>&8 2>&9; then
      :
    else
      restore_status=1
      exec >/dev/null 2>&1 || true
    fi
    RUN_LOG_OUTPUT_ACTIVE=0
    RUN_LOG_DIRECT_OUTPUT_ACTIVE=0
  fi
  return "$restore_status"
}

reset_run_log_state() {
  RUN_LOG_TEE_PID=""
  RUN_LOG_TEE_TOKEN=""
  RUN_LOG_PROCESS_RECORD=""
  RUN_LOG_PIPE=""
  RUN_LOG_PATH=""
  RUN_LOG_OUTPUT_ACTIVE=0
  RUN_LOG_DIRECT_OUTPUT_ACTIVE=0
  RUN_LOG_RECEIPT=""
  RUN_LOG_RECEIPT_NONCE=""
  RUN_LOG_RECEIPT_REQUIRED=0
  if [[ "$RUN_LOG_FDS_SAVED" == "1" ]]; then
    exec 8>&- 9>&- || true
  fi
  RUN_LOG_FDS_SAVED=0
}

rollback_run_log_start() {
  local cleanup_status=0 stop_status=0
  run_log_restore_original_output || cleanup_status=1
  exec 7>&- >/dev/null 2>&1 || true
  if stop_run_log_tee; then
    :
  else
    stop_status=$?
    cleanup_status=1
  fi
  if [[ "$stop_status" != "2" ]]; then
    rm -f "$RUN_LOG_PROCESS_RECORD" "$RUN_LOG_PIPE" "$RUN_LOG_RECEIPT" \
      >/dev/null 2>&1 \
      || cleanup_status=1
  fi
  reset_run_log_state
  return "$cleanup_status"
}

start_run_log() {
  local log_path="$1"
  local process_record="$2"
  local pipe_path="$3"
  local record_tmp="${process_record}.tmp"
  local receipt_path="${process_record}.complete"
  shift 3

  [[ "$RUN_LOG_OUTPUT_ACTIVE" == "0" \
    && "$RUN_LOG_DIRECT_OUTPUT_ACTIVE" == "0" \
    && "$RUN_LOG_FDS_SAVED" == "0" \
    && -z "$RUN_LOG_TEE_PID" ]] || return 1
  RUN_LOG_RECEIPT_REQUIRED=0
  RUN_LOG_RECEIPT_NONCE=""
  if (( $# == 0 )); then
    RUN_LOG_RECEIPT_NONCE="$(python3 -c 'import secrets; print(secrets.token_hex(32))')" \
      || return 1
    [[ "$RUN_LOG_RECEIPT_NONCE" =~ ^[0-9a-f]{64}$ ]] || return 1
    RUN_LOG_RECEIPT_REQUIRED=1
    set -- python3 "$RUN_LOG_SINK_HELPER" "$log_path" "$receipt_path" \
      "$RUN_LOG_RECEIPT_NONCE"
  fi
  command -v "$1" >/dev/null 2>&1 || return 1
  rm -f "$pipe_path" "$record_tmp" "$process_record" "$receipt_path" || return 1
  : >> "$log_path" || return 1
  mkfifo "$pipe_path" || return 1
  if ! exec 8>&1 9>&2; then
    rm -f "$pipe_path" "$record_tmp" "$process_record" >/dev/null 2>&1 || true
    exec 8>&- 9>&- || true
    return 1
  fi
  RUN_LOG_FDS_SAVED=1
  RUN_LOG_PATH="$log_path"
  RUN_LOG_PROCESS_RECORD="$process_record"
  RUN_LOG_PIPE="$pipe_path"
  RUN_LOG_RECEIPT="$receipt_path"

  # This guard opens both FIFO ends. It prevents the reader and writer opens
  # from deadlocking during startup. It closes after both real ends exist.
  if ! exec 7<>"$pipe_path"; then
    rollback_run_log_start || true
    return 1
  fi
  (exec 7>&-; exec "$@") < "$pipe_path" >&8 2>&9 &
  RUN_LOG_TEE_PID="$!"
  if ! RUN_LOG_TEE_TOKEN="$(process_start_token "$RUN_LOG_TEE_PID")"; then
    rollback_run_log_start || true
    return 1
  fi
  if ! printf '%s\t%s\n' "$RUN_LOG_TEE_PID" "$RUN_LOG_TEE_TOKEN" \
      > "$record_tmp"; then
    rollback_run_log_start || true
    return 1
  fi
  if ! mv "$record_tmp" "$process_record"; then
    rollback_run_log_start || true
    return 1
  fi
  if ! process_record_is_excluded \
      "$process_record" "$RUN_LOG_TEE_PID" "$RUN_LOG_TEE_TOKEN"; then
    rollback_run_log_start || true
    return 1
  fi
  if exec > "$pipe_path" 2>&1; then
    RUN_LOG_OUTPUT_ACTIVE=1
  else
    rollback_run_log_start || true
    return 1
  fi
  if ! exec 7>&-; then
    rollback_run_log_start || true
    return 1
  fi
  if ! rm -f "$pipe_path" >/dev/null 2>&1; then
    rollback_run_log_start || true
    return 1
  fi
  if ! run_log_sink_is_current; then
    rollback_run_log_start || true
    return 1
  fi
  return 0
}

run_log_sink_is_current() {
  [[ "$RUN_LOG_OUTPUT_ACTIVE" == "1" ]] || return 1
  run_log_generation_state
}

# Switch the parent away from the FIFO before any process signal or destructive
# cleanup. Descendants can still hold FIFO writers, so this function does not
# stop the sink. Later output appends directly to the regular log file.
detach_run_log_for_cleanup() {
  local status=0
  local saved_record="$RUN_LOG_PROCESS_RECORD"

  if [[ "$RUN_LOG_OUTPUT_ACTIVE" != "1" ]] || ! run_log_sink_is_current; then
    status=1
  fi
  if ! process_record_is_excluded \
      "$saved_record" "$RUN_LOG_TEE_PID" "$RUN_LOG_TEE_TOKEN"; then
    status=1
  fi

  # This is the first mutation. No later cleanup decision depends on a FIFO.
  run_log_restore_original_output || status=1
  if [[ -n "$RUN_LOG_PATH" ]]; then
    if exec >> "$RUN_LOG_PATH" 2>&1; then
      RUN_LOG_DIRECT_OUTPUT_ACTIVE=1
    else
      status=1
    fi
  else
    status=1
  fi
  return "$status"
}

# Stop the sink only after all other recorded descendants are gone. The process
# record remains in place when the exact generation cannot be proved absent.
close_run_log_sink() {
  local status=0
  local saved_record="$RUN_LOG_PROCESS_RECORD"
  local saved_pipe="$RUN_LOG_PIPE"
  local saved_receipt="$RUN_LOG_RECEIPT"

  if [[ -n "$RUN_LOG_TEE_PID" ]]; then
    if stop_run_log_tee; then
      :
    else
      status=$?
    fi
  fi
  if [[ "$status" != "2" ]]; then
    rm -f "$saved_record" "$saved_pipe" "$saved_receipt" >/dev/null 2>&1 \
      || status=1
    RUN_LOG_PROCESS_RECORD=""
    RUN_LOG_PIPE=""
  fi
  return "$status"
}

finish_run_log() {
  local status=0
  local close_status=0

  if [[ "$RUN_LOG_OUTPUT_ACTIVE" == "1" ]]; then
    detach_run_log_for_cleanup || status=1
  fi
  if [[ -n "$RUN_LOG_TEE_PID" ]]; then
    if close_run_log_sink; then
      :
    else
      close_status=$?
      status="$close_status"
    fi
  fi
  run_log_restore_original_output || status=1
  reset_run_log_state
  return "$status"
}
