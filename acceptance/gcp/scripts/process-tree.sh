#!/usr/bin/env bash

# Helpers shared by the GCP deadline watchdog and its local process-group test.
# A recorded start token prevents a recycled PID from being signalled.

PROCESS_TREE_SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROCESS_START_TOKEN_HELPER="${PROCESS_START_TOKEN_HELPER:-$PROCESS_TREE_SCRIPT_DIR/process-start-token.py}"
PROCESS_SNAPSHOT_HELPER="${PROCESS_SNAPSHOT_HELPER:-$PROCESS_START_TOKEN_HELPER}"
PROCESS_TOKEN_UNVERIFIABLE="unverifiable"

process_start_token() {
  local pid="$1"
  "$PROCESS_START_TOKEN_HELPER" "$pid" 2>/dev/null
}

process_snapshot() {
  "$PROCESS_SNAPSHOT_HELPER" --snapshot 2>/dev/null
}

persist_process_scan_failure() {
  local output="$1"
  local message="$2"
  local temporary
  temporary="$(mktemp "${output}.tmp.XXXXXX")" || return 1
  if ! printf '%s\n' "$message" > "$temporary"; then
    rm -f "$temporary"
    return 1
  fi
  chmod 0600 "$temporary" || {
    rm -f "$temporary"
    return 1
  }
  mv "$temporary" "$output" || {
    rm -f "$temporary"
    return 1
  }
}

# A marker failure must never suppress the attempt to stop the owner. Return a
# failure when either action fails so the watchdog cannot report success.
report_process_scan_failure() {
  local output="$1"
  local message="$2"
  local owner_pid="$3"
  local owner_token="$4"
  local marker_status=0
  local signal_status=0
  persist_process_scan_failure "$output" "$message" || marker_status=$?
  signal_process_generation "$owner_pid" "$owner_token" TERM \
    >/dev/null 2>&1 || signal_status=$?
  [[ "$marker_status" == "0" && "$signal_status" == "0" ]]
}

# Return 0 only while the exact recorded process generation is live.
# Return 1 when the PID is absent or belongs to a newer generation.
# Return 2 when the operating system cannot prove either state.
process_generation_state() {
  local pid="$1"
  local expected="$2"
  local current token_status
  [[ "$pid" =~ ^[0-9]+$ && -n "$expected" \
    && "$expected" != "$PROCESS_TOKEN_UNVERIFIABLE" ]] || return 2
  if current="$(process_start_token "$pid")"; then
    [[ "$current" == "$expected" ]] && return 0
    return 1
  else
    token_status=$?
  fi
  [[ "$token_status" == "1" ]] && return 1
  return 2
}

# Print `stopped` or `running` only while the exact generation is live.
# Return 1 when it is absent or replaced. Return 2 when it is unverifiable.
process_generation_run_state() {
  local pid="$1"
  local expected="$2"
  local state status
  [[ "$pid" =~ ^[0-9]+$ && -n "$expected" \
    && "$expected" != "$PROCESS_TOKEN_UNVERIFIABLE" ]] || return 2
  if state="$("$PROCESS_START_TOKEN_HELPER" --status "$pid" "$expected" 2>/dev/null)"; then
    [[ "$state" == "stopped" || "$state" == "running" ]] || return 2
    printf '%s\n' "$state"
    return 0
  else
    status=$?
  fi
  [[ "$status" == "1" ]] && return 1
  return 2
}

# Return 0 after the exact generation is stopped. Return 1 if it is gone.
# Return 2 if its identity is unverifiable. Return 3 if it stays runnable.
wait_for_process_generation_stopped() {
  local pid="$1"
  local expected="$2"
  local attempts="$3"
  local attempt state status
  for ((attempt = 0; attempt < attempts; attempt += 1)); do
    if state="$(process_generation_run_state "$pid" "$expected")"; then
      [[ "$state" == "stopped" ]] && return 0
      sleep 0.05
      continue
    else
      status=$?
    fi
    [[ "$status" == "1" ]] && return 1
    return 2
  done
  return 3
}

wait_for_process_generation_absent() {
  local pid="$1"
  local expected="$2"
  local attempts="$3"
  local attempt state
  for ((attempt = 0; attempt < attempts; attempt += 1)); do
    if process_generation_state "$pid" "$expected"; then
      sleep 0.1
      continue
    else
      state=$?
    fi
    [[ "$state" == "1" ]] && return 0
    return 2
  done
  return 1
}

signal_process_generation() {
  local pid="$1"
  local expected="$2"
  local signal="$3"
  local state
  if process_generation_state "$pid" "$expected"; then
    kill "-$signal" "$pid" >/dev/null 2>&1 || {
      if process_generation_state "$pid" "$expected"; then
        return 1
      else
        state=$?
        [[ "$state" == "1" ]] && return 0
        return 2
      fi
    }
    return 0
  else
    state=$?
  fi
  [[ "$state" == "1" ]] && return 0
  return 2
}

# Stop one exact process generation. Every wait has a fixed deadline.
# Return 0 after the generation is absent. Return 1 if it survives SIGKILL.
# Return 2 if its identity becomes unverifiable.
stop_process_generation_bounded() {
  local pid="$1"
  local expected="$2"
  local state

  if process_generation_state "$pid" "$expected"; then
    :
  else
    state=$?
    [[ "$state" == "1" ]] && return 0
    return 2
  fi
  signal_process_generation "$pid" "$expected" TERM || return $?
  if wait_for_process_generation_absent "$pid" "$expected" 20; then
    return 0
  else
    state=$?
    [[ "$state" == "2" ]] && return 2
  fi
  signal_process_generation "$pid" "$expected" KILL || return $?
  wait_for_process_generation_absent "$pid" "$expected" 20
}

process_group_id() {
  local pid="$1"
  ps -o pgid= -p "$pid" 2>/dev/null \
    | sed 's/^[[:space:]]*//;s/[[:space:]]*$//' \
    || true
}

process_record_is_excluded() {
  local input="$1"
  local candidate_pid="$2"
  local candidate_token="$3"
  local pid token
  [[ -n "$input" && -f "$input" && ! -L "$input" ]] || return 1
  while IFS=$'\t' read -r pid token; do
    [[ "$pid" =~ ^[0-9]+$ && -n "$token" ]] || return 1
    if [[ "$pid" == "$candidate_pid" && "$token" == "$candidate_token" ]]; then
      return 0
    fi
  done < "$input"
  return 1
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
  local exclusion_record="${5:-}"
  local ancestry_record="${6:-}"
  local temporary process_list candidates ancestry_seeds
  local pid token token_status state
  [[ "$group_id" =~ ^[0-9]+$ ]] || return 1
  if [[ -e "$output" || -L "$output" ]]; then
    [[ -f "$output" && ! -L "$output" ]] || return 1
  fi
  temporary="$(mktemp "${output}.tmp.XXXXXX")" || return 1
  process_list="$(mktemp "${output}.processes.XXXXXX")" || {
    rm -f "$temporary"
    return 1
  }
  candidates="$(mktemp "${output}.candidates.XXXXXX")" || {
    rm -f "$temporary" "$process_list"
    return 1
  }
  ancestry_seeds="$(mktemp "${output}.seeds.XXXXXX")" || {
    rm -f "$temporary" "$process_list" "$candidates"
    return 1
  }

  # The direct run process and each live historical generation are ancestry
  # roots. This keeps ownership after a child changes its process group or
  # creates a new session with setsid(). Validate every historical generation
  # before and after the process snapshot so PID reuse cannot change a root.
  if [[ -n "$excluded_pid_one" ]]; then
    [[ "$excluded_pid_one" =~ ^[0-9]+$ ]] || {
      rm -f "$temporary" "$process_list" "$candidates" "$ancestry_seeds"
      return 1
    }
    if token="$(process_start_token "$excluded_pid_one")"; then
      printf '%s\t%s\n' "$excluded_pid_one" "$token" >> "$ancestry_seeds" || {
        rm -f "$temporary" "$process_list" "$candidates" "$ancestry_seeds"
        return 1
      }
    else
      token_status=$?
      [[ "$token_status" == "1" ]] || {
        rm -f "$temporary" "$process_list" "$candidates" "$ancestry_seeds"
        return 1
      }
    fi
  fi
  if [[ -e "$ancestry_record" || -L "$ancestry_record" ]]; then
    [[ -f "$ancestry_record" && ! -L "$ancestry_record" ]] || {
      rm -f "$temporary" "$process_list" "$candidates" "$ancestry_seeds"
      return 1
    }
    while IFS=$'\t' read -r pid token; do
      [[ "$pid" =~ ^[0-9]+$ && -n "$token" \
        && "$token" != "$PROCESS_TOKEN_UNVERIFIABLE" ]] || {
        rm -f "$temporary" "$process_list" "$candidates" "$ancestry_seeds"
        return 1
      }
      if process_generation_state "$pid" "$token"; then
        printf '%s\t%s\n' "$pid" "$token" >> "$ancestry_seeds" || {
          rm -f "$temporary" "$process_list" "$candidates" "$ancestry_seeds"
          return 1
        }
      else
        state=$?
        [[ "$state" == "1" ]] || {
          rm -f "$temporary" "$process_list" "$candidates" "$ancestry_seeds"
          return 1
        }
      fi
    done < "$ancestry_record"
  fi

  if ! process_snapshot > "$process_list"; then
    rm -f "$temporary" "$process_list" "$candidates" "$ancestry_seeds"
    return 1
  fi
  if ! awk \
      -v group_id="$group_id" \
      -v seed_file="$ancestry_seeds" '
        BEGIN {
          while ((getline seed_line < seed_file) > 0) {
            split(seed_line, seed_fields, /[[:space:]]+/)
            if (seed_fields[1] !~ /^[0-9]+$/ || seed_fields[2] == "") {
              exit 2
            }
            owned[seed_fields[1]] = 1
            seed_token[seed_fields[1]] = seed_fields[2]
          }
          close(seed_file)
        }
        NF != 4 || $1 !~ /^[0-9]+$/ || $2 !~ /^[0-9]+$/ \
          || $3 !~ /^[0-9]+$/ || $4 == "" { invalid = 1; next }
        {
          pid[NR] = $1
          parent[NR] = $2
          member_group[NR] = $3
          token[NR] = $4
          present[$1] = 1
          present_token[$1] = $4
          if ($3 == group_id) {
            owned[$1] = 1
          }
        }
        END {
          if (invalid) {
            exit 2
          }
          changed = 1
          while (changed) {
            changed = 0
            for (row = 1; row <= NR; row += 1) {
              if (!(pid[row] in owned) && (parent[row] in owned)) {
                owned[pid[row]] = 1
                changed = 1
              }
            }
          }
          for (row = 1; row <= NR; row += 1) {
            if (pid[row] in owned) {
              print pid[row] "\t" token[row]
            }
          }
          for (seed in owned) {
            if ((seed in seed_token) \
                && (!present[seed] || present_token[seed] != seed_token[seed])) {
              exit 3
            }
          }
        }
      ' "$process_list" > "$candidates"; then
    rm -f "$temporary" "$process_list" "$candidates" "$ancestry_seeds"
    return 1
  fi
  while IFS=$'\t' read -r pid token; do
    if process_generation_state "$pid" "$token"; then
      :
    else
      state=$?
      rm -f "$temporary" "$process_list" "$candidates" "$ancestry_seeds"
      [[ "$state" == "1" ]] && return 1
      return 1
    fi
  done < "$ancestry_seeds"

  while IFS=$'\t' read -r pid token; do
    [[ "$pid" != "$excluded_pid_one" && "$pid" != "$excluded_pid_two" ]] \
      || continue
    [[ "$pid" =~ ^[0-9]+$ && -n "$token" ]] || {
      rm -f "$temporary" "$process_list" "$candidates" "$ancestry_seeds"
      return 1
    }
    if process_generation_state "$pid" "$token"; then
      :
    else
      token_status=$?
      if [[ "$token_status" == "1" ]]; then
        continue
      fi
      rm -f "$temporary" "$process_list" "$candidates" "$ancestry_seeds"
      return 1
    fi
    process_record_is_excluded "$exclusion_record" "$pid" "$token" && continue
    printf '%s\t%s\n' "$pid" "$token" >> "$temporary" || {
      rm -f "$temporary" "$process_list" "$candidates" "$ancestry_seeds"
      return 1
    }
  done < "$candidates"
  rm -f "$process_list" "$candidates" "$ancestry_seeds" || {
    rm -f "$temporary"
    return 1
  }
  mv "$temporary" "$output" || {
    rm -f "$temporary"
    return 1
  }
}

append_process_group_members() {
  local group_id="$1"
  local excluded_pid_one="$2"
  local excluded_pid_two="$3"
  local output="$4"
  local exclusion_record="${5:-}"
  local newly_recorded merged
  newly_recorded="$(mktemp "${output}.new.XXXXXX")" || return 1
  merged="$(mktemp "${output}.merged.XXXXXX")" || {
    rm -f "$newly_recorded"
    return 1
  }
  if ! record_process_group_members \
      "$group_id" "$excluded_pid_one" "$excluded_pid_two" "$newly_recorded" \
      "$exclusion_record" "$output"; then
    rm -f "$newly_recorded" "$merged"
    return 1
  fi
  if [[ -f "$output" ]]; then
    sort -u "$output" "$newly_recorded" > "$merged" || {
      rm -f "$newly_recorded" "$merged"
      return 1
    }
  else
    sort -u "$newly_recorded" > "$merged" || {
      rm -f "$newly_recorded" "$merged"
      return 1
    }
  fi
  mv "$merged" "$output" || {
    rm -f "$newly_recorded" "$merged"
    return 1
  }
  rm -f "$newly_recorded" || {
    return 1
  }
}

# A process can exit after the snapshot records it and before the exact-token
# check reads it. That race is safe to retry because every failed attempt keeps
# the last complete record unchanged. A fixed retry bound still fails closed
# when the kernel snapshot or identity checks remain unavailable.
append_process_group_members_with_retry() {
  local group_id="$1"
  local excluded_pid_one="$2"
  local excluded_pid_two="$3"
  local output="$4"
  local exclusion_record="${5:-}"
  local attempts="${6:-3}"
  local attempt
  [[ "$attempts" =~ ^[1-9][0-9]*$ && "$attempts" -le 10 ]] || return 1
  for ((attempt = 1; attempt <= attempts; attempt += 1)); do
    if append_process_group_members \
        "$group_id" "$excluded_pid_one" "$excluded_pid_two" "$output" \
        "$exclusion_record"; then
      return 0
    fi
  done
  return 1
}

merge_process_records() {
  local output="$1"
  local new_records="$2"
  local merged
  [[ -f "$new_records" && ! -L "$new_records" ]] || return 1
  if [[ -e "$output" || -L "$output" ]]; then
    [[ -f "$output" && ! -L "$output" ]] || return 1
  fi
  merged="$(mktemp "${output}.merged.XXXXXX")" || return 1
  if [[ -e "$output" ]]; then
    sort -u "$output" "$new_records" > "$merged" || {
      rm -f "$merged"
      return 1
    }
  else
    sort -u "$new_records" > "$merged" || {
      rm -f "$merged"
      return 1
    }
  fi
  mv "$merged" "$output" || {
    rm -f "$merged"
    return 1
  }
}

signal_recorded_processes() {
  local input="$1"
  local signal="$2"
  local pid expected current
  [[ -f "$input" ]] || return 0
  while IFS=$'\t' read -r pid expected; do
    [[ "$pid" =~ ^[0-9]+$ && -n "$expected" ]] || continue
    [[ "$expected" != "$PROCESS_TOKEN_UNVERIFIABLE" ]] || continue
    if ! current="$(process_start_token "$pid")"; then
      continue
    fi
    [[ -n "$current" && "$current" == "$expected" ]] || continue
    kill "-$signal" "$pid" >/dev/null 2>&1 || true
  done < "$input"
}

process_record_contains_generation() {
  local input="$1"
  local candidate_pid="$2"
  local candidate_token="$3"
  local pid token
  [[ -f "$input" && ! -L "$input" ]] || return 1
  while IFS=$'\t' read -r pid token; do
    if [[ "$pid" == "$candidate_pid" && "$token" == "$candidate_token" ]]; then
      return 0
    fi
  done < "$input"
  return 1
}

recorded_processes_have_live_member() {
  local input="$1"
  local pid expected current token_status
  if [[ ! -e "$input" && ! -L "$input" ]]; then
    return 1
  fi
  [[ -f "$input" && ! -L "$input" ]] || return 0
  while IFS=$'\t' read -r pid expected; do
    if [[ ! "$pid" =~ ^[0-9]+$ || -z "$expected" ]]; then
      return 0
    fi
    if current="$(process_start_token "$pid")"; then
      if [[ "$expected" == "$PROCESS_TOKEN_UNVERIFIABLE" \
        || "$current" == "$expected" ]]; then
        return 0
      fi
    else
      token_status=$?
      # Exit 1 means the kernel proved that the PID is absent. Every other
      # failure is unsafe because the process can still be alive.
      [[ "$token_status" == "1" ]] || return 0
    fi
  done < "$input"
  return 1
}

# Freeze every visible creator to a fixed point before the first KILL. A
# process can fork after a `ps` snapshot, but it cannot fork after its exact
# generation is confirmed stopped. Repeated scans therefore close the
# fork-after-snapshot window. Any scan, identity, stop, or drain uncertainty
# makes the caller refuse destructive cleanup.
shutdown_process_group_members() {
  local group_id="$1"
  local excluded_pid_one="$2"
  local excluded_pid_two="$3"
  local output="$4"
  local exclusion_record="$5"
  local attempts="$6"
  local delay_seconds="$7"
  local attempt fresh frozen pid token state new_generation stable

  [[ "$attempts" =~ ^[1-9][0-9]*$ ]] || return 2
  fresh="${output}.fresh"
  frozen="${output}.frozen"
  : > "$frozen" || return 2
  stable=0
  for ((attempt = 1; attempt <= attempts; attempt += 1)); do
    record_process_group_members \
      "$group_id" "$excluded_pid_one" "$excluded_pid_two" "$fresh" \
      "$exclusion_record" "$output" || return 2
    merge_process_records "$output" "$fresh" || return 2
    new_generation=0
    while IFS=$'\t' read -r pid token; do
      [[ "$pid" =~ ^[0-9]+$ && -n "$token" \
        && "$token" != "$PROCESS_TOKEN_UNVERIFIABLE" ]] || return 2
      if process_record_contains_generation "$frozen" "$pid" "$token"; then
        if state="$(process_generation_run_state "$pid" "$token")"; then
          if [[ "$state" == "running" ]]; then
            new_generation=1
            signal_process_generation "$pid" "$token" STOP || return 2
            wait_for_process_generation_stopped "$pid" "$token" 40 \
              || return 2
          fi
        else
          state=$?
          [[ "$state" == "1" ]] || return 2
        fi
        continue
      fi
      if state="$(process_generation_run_state "$pid" "$token")"; then
        [[ "$state" == "running" ]] || {
          printf '%s\t%s\n' "$pid" "$token" >> "$frozen" || return 2
          continue
        }
      else
        state=$?
        # An already exited historical generation cannot create more work.
        [[ "$state" == "1" ]] && continue
        return 2
      fi
      new_generation=1
      signal_process_generation "$pid" "$token" STOP || return 2
      if wait_for_process_generation_stopped "$pid" "$token" 40; then
        printf '%s\t%s\n' "$pid" "$token" >> "$frozen" || return 2
      else
        state=$?
        # A generation that exited before STOP cannot create more children.
        [[ "$state" == "1" ]] || return 2
      fi
    done < "$output"
    if [[ "$new_generation" == "0" ]]; then
      stable=1
      break
    fi
    sort -u "$frozen" -o "$frozen" || return 2
    [[ "$attempt" == "$attempts" ]] || sleep "$delay_seconds"
  done
  [[ "$stable" == "1" ]] || return 1

  signal_recorded_processes "$frozen" KILL
  for ((attempt = 1; attempt <= attempts; attempt += 1)); do
    record_process_group_members \
      "$group_id" "$excluded_pid_one" "$excluded_pid_two" "$fresh" \
      "$exclusion_record" "$output" || return 2
    merge_process_records "$output" "$fresh" || return 2
    if ! recorded_processes_have_live_member "$fresh" \
      && ! recorded_processes_have_live_member "$output"; then
      rm -f "$fresh" "$frozen"
      return 0
    fi
    # All creators were confirmed stopped before KILL. A new generation here
    # cannot be attributed safely to this run, so do not continue cleanup.
    while IFS=$'\t' read -r pid token; do
      if process_generation_state "$pid" "$token"; then
        process_record_contains_generation "$frozen" "$pid" "$token" \
          || return 1
      else
        state=$?
        [[ "$state" == "1" ]] || return 2
      fi
    done < "$output"
    signal_recorded_processes "$frozen" KILL
    [[ "$attempt" == "$attempts" ]] || sleep "$delay_seconds"
  done
  return 1
}
