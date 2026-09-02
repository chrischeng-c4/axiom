#!/usr/bin/env bash

# Helpers shared by the GCP deadline watchdog and its local process-group test.
# A recorded start token prevents a recycled PID from being signalled.

process_start_token() {
  local pid="$1"
  ps -o lstart= -p "$pid" 2>/dev/null \
    | sed 's/^[[:space:]]*//;s/[[:space:]]*$//' \
    || true
}

process_group_id() {
  local pid="$1"
  ps -o pgid= -p "$pid" 2>/dev/null \
    | sed 's/^[[:space:]]*//;s/[[:space:]]*$//' \
    || true
}

# The parent knows the real PID returned by `$!`, including on Bash 3.2. The
# background process reads that atomically written value before it scans its
# group. This avoids both `$$` inheritance and the unavailable BASHPID value.
wait_for_process_id_file() {
  local input="$1"
  local parent_pid="$2"
  local attempt pid
  for attempt in 1 2 3 4 5 6 7 8 9 10; do
    if [[ -s "$input" ]]; then
      pid="$(sed -n '1p' "$input")"
      if [[ "$pid" =~ ^[0-9]+$ ]]; then
        printf '%s\n' "$pid"
        return 0
      fi
    fi
    kill -0 "$parent_pid" >/dev/null 2>&1 || return 1
    sleep 1
  done
  return 1
}

record_process_group_members() {
  local group_id="$1"
  local excluded_pid_one="$2"
  local excluded_pid_two="$3"
  local output="$4"
  local temporary="${output}.tmp"
  local pid member_group token
  : > "$temporary"
  while read -r pid member_group; do
    [[ "$member_group" == "$group_id" ]] || continue
    [[ "$pid" != "$excluded_pid_one" && "$pid" != "$excluded_pid_two" ]] \
      || continue
    token="$(process_start_token "$pid")"
    [[ -n "$token" ]] || continue
    printf '%s\t%s\n' "$pid" "$token" >> "$temporary"
  done < <(ps -axo pid=,pgid= 2>/dev/null || true)
  mv "$temporary" "$output"
}

append_process_group_members() {
  local group_id="$1"
  local excluded_pid_one="$2"
  local excluded_pid_two="$3"
  local output="$4"
  local newly_recorded="${output}.new"
  local merged="${output}.merged"
  record_process_group_members \
    "$group_id" "$excluded_pid_one" "$excluded_pid_two" "$newly_recorded"
  if [[ -f "$output" ]]; then
    sort -u "$output" "$newly_recorded" > "$merged"
  else
    sort -u "$newly_recorded" > "$merged"
  fi
  mv "$merged" "$output"
  rm -f "$newly_recorded"
}

signal_recorded_processes() {
  local input="$1"
  local signal="$2"
  local pid expected current
  [[ -f "$input" ]] || return 0
  while IFS=$'\t' read -r pid expected; do
    [[ "$pid" =~ ^[0-9]+$ && -n "$expected" ]] || continue
    current="$(process_start_token "$pid")"
    [[ -n "$current" && "$current" == "$expected" ]] || continue
    kill "-$signal" "$pid" >/dev/null 2>&1 || true
  done < "$input"
}
