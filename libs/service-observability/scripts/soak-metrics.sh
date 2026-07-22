#!/usr/bin/env bash
# HANDWRITE-BEGIN gap="missing-generator:e2e-test:shared-soak-resource-metrics" tracker="#1777" reason="Portable process-resource and latency plateau sampling is shared by service-specific bounded soak workloads."

# Shared bounded-soak measurements. Callers own the domain workload and source
# this file for portable process/resource sampling and plateau assertions.

service_soak_rss_sampler_expected_count() {
  local window_secs="$1"
  local sample_interval_secs="$2"
  [[ "${window_secs}" =~ ^[0-9]+$ ]] || return 1
  [[ "${sample_interval_secs}" =~ ^[0-9]+$ ]] || return 1
  (( window_secs > 0 )) || return 1
  (( sample_interval_secs > 0 )) || return 1
  echo $(( (window_secs + sample_interval_secs - 1) / sample_interval_secs ))
}

service_soak_rss_sampler_start() {
  local pid="$1"
  local window_secs="$2"
  local sample_interval_secs="$3"
  local dir
  local samples
  local meta
  local sampler_pid
  local expected_count
  (( ${BASH_SUBSHELL:-0} == 0 )) || {
    echo "service_soak_rss_sampler_start must be called directly, not from a subshell" >&2
    return 1
  }
  [[ "$pid" =~ ^[0-9]+$ ]] || {
    echo "invalid pid: ${pid}" >&2
    return 1
  }
  expected_count="$(service_soak_rss_sampler_expected_count "${window_secs}" "${sample_interval_secs}")" || {
    echo "invalid sampler window or cadence" >&2
    return 1
  }
  kill -0 "$pid" 2>/dev/null || {
    echo "pid ${pid} is not reachable" >&2
    return 1
  }
  dir="$(mktemp -d "${TMPDIR:-/tmp}/service-soak-rss.XXXXXX")" || return 1
  samples="${dir}/samples"
  meta="${dir}/meta"
  : >"${samples}"
  cat <<EOF >"${meta}"
target_pid=${pid}
window_secs=${window_secs}
sample_interval_secs=${sample_interval_secs}
expected_count=${expected_count}
started_at=$(date +%s)
completed=0
target_missing=0
owner_missing=0
owner_pid=$$
EOF
  _service_soak_rss_sampler_loop "$dir" &
  sampler_pid=$!
  printf 'sampler_pid=%s\n' "${sampler_pid}" >>"${meta}"
  SERVICE_SOAK_RSS_LAST_TOKEN="${dir}"
  SERVICE_SOAK_RSS_ACTIVE_TOKEN="${dir}"
  unset SERVICE_SOAK_RSS_LAST_SUMMARY
}

_service_soak_rss_sampler_loop() {
  local dir="$1"
  local meta="${dir}/meta"
  local samples="${dir}/samples"
  local pid
  local window_secs
  local sample_interval_secs
  local now
  local rss
  local started_at
  local deadline
  local owner_pid
  pid="$(_service_soak_rss_meta_get "${meta}" target_pid)" || return 1
  window_secs="$(_service_soak_rss_meta_get "${meta}" window_secs)" || return 1
  sample_interval_secs="$(_service_soak_rss_meta_get "${meta}" sample_interval_secs)" || return 1
  started_at="$(_service_soak_rss_meta_get "${meta}" started_at)" || return 1
  owner_pid="$(_service_soak_rss_meta_get "${meta}" owner_pid)" || return 1
  deadline=$(( started_at + window_secs ))
  while :; do
    now="$(date +%s)"
    if ! kill -0 "$owner_pid" 2>/dev/null; then
      _service_soak_rss_meta_set "${meta}" owner_missing 1
      rm -rf "${dir}"
      exit 0
    fi
    if ! kill -0 "$pid" 2>/dev/null; then
      _service_soak_rss_meta_set "${meta}" target_missing 1
      break
    fi
    rss="$(service_soak_rss_kb "$pid")"
    [[ "${rss}" =~ ^[0-9]+$ ]] || {
      return 1
    }
    echo "${rss}" >>"${samples}"
    if (( now + sample_interval_secs > deadline )); then
      _service_soak_rss_meta_set "${meta}" completed 1
      break
    fi
    sleep "${sample_interval_secs}"
  done
}

service_soak_rss_sampler_stop() {
  local token="$1"
  local meta="${token}/meta"
  local samples="${token}/samples"
  local pid
  local sampler_pid
  local expected_count
  local window_secs
  local started_at
  local now
  local deadline
  local summary
  (( ${BASH_SUBSHELL:-0} == 0 )) || {
    echo "service_soak_rss_sampler_stop must be called directly, not from a subshell" >&2
    return 1
  }
  [[ -d "${token}" && -f "${meta}" && -f "${samples}" ]] || {
    echo "invalid sampler token" >&2
    return 1
  }
  pid="$(_service_soak_rss_meta_get "${meta}" target_pid)" || return 1
  sampler_pid="$(_service_soak_rss_meta_get "${meta}" sampler_pid)" || return 1
  expected_count="$(_service_soak_rss_meta_get "${meta}" expected_count)" || return 1
  window_secs="$(_service_soak_rss_meta_get "${meta}" window_secs)" || return 1
  started_at="$(_service_soak_rss_meta_get "${meta}" started_at)" || return 1
  now="$(date +%s)"
  deadline=$(( started_at + window_secs ))
  if (( now < deadline )); then
    _service_soak_rss_sampler_cleanup "${token}" 0
    unset SERVICE_SOAK_RSS_ACTIVE_TOKEN
    echo "sampler window did not complete before stop" >&2
    return 1
  fi
  _service_soak_rss_sampler_wait_for_completion "${token}" || true
  _service_soak_rss_sampler_cleanup "${token}" 1
  _service_soak_rss_sampler_validate_series "${pid}" "${samples}" "${meta}" "${expected_count}" "${window_secs}" || {
    rm -rf "${token}"
    unset SERVICE_SOAK_RSS_ACTIVE_TOKEN
    return 1
  }
  summary="$(service_soak_rss_window_summary "${samples}")" || {
    rm -rf "${token}"
    unset SERVICE_SOAK_RSS_ACTIVE_TOKEN
    return 1
  }
  SERVICE_SOAK_RSS_LAST_SUMMARY="${summary}"
  rm -rf "${token}"
  unset SERVICE_SOAK_RSS_ACTIVE_TOKEN
}

service_soak_rss_sampler_abort() {
  local token="$1"
  _service_soak_rss_sampler_cleanup "${token}" 0
  if [[ "${SERVICE_SOAK_RSS_ACTIVE_TOKEN:-}" == "${token}" ]]; then
    unset SERVICE_SOAK_RSS_ACTIVE_TOKEN
  fi
}

_service_soak_rss_sampler_cleanup() {
  local token="$1"
  local keep_artifacts="${2:-0}"
  local meta="${token}/meta"
  local sampler_pid
  [[ -f "${meta}" ]] || return 0
  sampler_pid="$(_service_soak_rss_meta_get "${meta}" sampler_pid)" || return 0
  [[ -n "${sampler_pid}" ]] || return 0
  if kill -0 "${sampler_pid}" 2>/dev/null; then
    kill "${sampler_pid}" 2>/dev/null || true
  fi
  wait "${sampler_pid}" 2>/dev/null || true
  if (( keep_artifacts == 0 )); then
    rm -rf "${token}"
  fi
}

_service_soak_rss_sampler_wait_for_completion() {
  local token="$1"
  local meta="${token}/meta"
  local sample_interval_secs
  local tries
  local completed
  [[ -f "${meta}" ]] || return 1
  sample_interval_secs="$(_service_soak_rss_meta_get "${meta}" sample_interval_secs)" || return 1
  tries=$(( sample_interval_secs + 2 ))
  while (( tries > 0 )); do
    completed="$(_service_soak_rss_meta_get "${meta}" completed)" || return 1
    [[ "${completed}" == "1" ]] && return 0
    sleep 1
    tries=$((tries - 1))
  done
  return 1
}

_service_soak_rss_sampler_validate_series() {
  local pid="$1"
  local samples="$2"
  local meta="$3"
  local expected_count="$4"
  local window_secs="$5"
  local count
  local completed
  local target_missing
  local owner_missing
  [[ -f "${samples}" && -f "${meta}" ]] || {
    echo "missing sampler artifacts" >&2
    return 1
  }
  completed="$(_service_soak_rss_meta_get "${meta}" completed)" || return 1
  target_missing="$(_service_soak_rss_meta_get "${meta}" target_missing)" || return 1
  owner_missing="$(_service_soak_rss_meta_get "${meta}" owner_missing)" || return 1
  if [[ "${owner_missing}" == "1" ]]; then
    echo "sampler owner disappeared before cleanup completed" >&2
    return 1
  fi
  if [[ "${target_missing}" == "1" ]]; then
    echo "pid ${pid} disappeared before ${window_secs}s window completed" >&2
    return 1
  fi
  if [[ "${completed}" != "1" ]]; then
    echo "sampler window did not complete before stop" >&2
    return 1
  fi
  if awk '($0 !~ /^[0-9]+$/) { found=1; exit 0 } END { exit(found ? 0 : 1) }' "${samples}"; then
    echo "series contains non-numeric RSS values" >&2
    return 1
  fi
  count="$(awk 'END { print NR + 0 }' "${samples}")"
  (( count >= expected_count )) || {
    echo "sample coverage ${count} below required ${expected_count}" >&2
    return 1
  }
}

service_soak_rss_window_summary() {
  local samples="$1"
  local sorted="${samples}.sorted.$$"
  if awk '($0 !~ /^[0-9]+$/) { bad=1; exit 0 } END { exit(bad ? 0 : 1) }' "${samples}"; then
    rm -f "${sorted}"
    return 1
  fi
  LC_ALL=C sort -n "${samples}" >"${sorted}" || {
    rm -f "${sorted}"
    return 1
  }
  awk '
    { value[++count] = $1 }
    END {
      if (count == 0) exit 1
      min = value[1]
      max = value[count]
      if (count % 2 == 1) {
        median = value[(count + 1) / 2]
      } else {
        median = int((value[count / 2] + value[(count / 2) + 1]) / 2)
      }
      printf "count=%d min=%d median=%d max=%d\n", count, min, median, max
    }
  ' "${sorted}"
  local rc=$?
  rm -f "${sorted}"
  return "${rc}"
}

_service_soak_rss_meta_get() {
  local meta="$1"
  local key="$2"
  awk -F= -v want="${key}" '$1 == want { print substr($0, index($0, "=") + 1); found=1; exit } END { exit(found ? 0 : 1) }' "${meta}"
}

_service_soak_rss_meta_set() {
  local meta="$1"
  local key="$2"
  local value="$3"
  awk -F= -v want="${key}" -v value="${value}" '
    BEGIN { updated = 0 }
    $1 == want { print want "=" value; updated = 1; next }
    { print }
    END {
      if (!updated) print want "=" value
    }
  ' "${meta}" >"${meta}.tmp" && mv "${meta}.tmp" "${meta}"
}

service_soak_summary_field() {
  local summary="$1"
  local field="$2"
  awk -v key="${field}" '
    {
      for (i = 1; i <= NF; i += 1) {
        split($i, pair, "=")
        if (pair[1] == key) {
          print pair[2]
          exit 0
        }
      }
      exit 1
    }
  ' <<<"${summary}"
}

service_soak_rss_window_growth_pct() {
  local before_summary="$1"
  local after_summary="$2"
  local before_median
  local after_median
  before_median="$(service_soak_summary_field "${before_summary}" median)" || return 1
  after_median="$(service_soak_summary_field "${after_summary}" median)" || return 1
  service_soak_percent_growth "${before_median}" "${after_median}"
}

service_soak_rss_kb() {
  ps -o rss= -p "$1" 2>/dev/null | tr -d ' '
}

service_soak_fd_count() {
  local pid="$1"
  if [[ -d "/proc/${pid}/fd" ]]; then
    find "/proc/${pid}/fd" -mindepth 1 -maxdepth 1 2>/dev/null | wc -l | tr -d ' '
    return
  fi
  command -v lsof >/dev/null 2>&1 || return 1
  lsof -n -P -p "$pid" 2>/dev/null | awk 'NR > 1 { count += 1 } END { print count + 0 }'
}

service_soak_task_count() {
  local pid="$1"
  if [[ -d "/proc/${pid}/task" ]]; then
    find "/proc/${pid}/task" -mindepth 1 -maxdepth 1 2>/dev/null | wc -l | tr -d ' '
    return
  fi
  # macOS/BSD ps exposes one row per thread with -M rather than a Linux-style
  # thcount field.
  ps -M -p "$pid" 2>/dev/null | awk 'NR > 1 { count += 1 } END { print count + 0 }'
}

service_soak_p99_ms() {
  local samples="$1"
  awk 'NF { printf "%.0f\n", ($1 * 1000) }' "$samples" |
    sort -n |
    awk '
      { value[NR] = $1 }
      END {
        if (NR == 0) exit 1
        rank = int((NR * 99 + 99) / 100)
        if (rank < 1) rank = 1
        print value[rank]
      }
    '
}

service_soak_percent_growth() {
  local before="$1"
  local after="$2"
  (( before > 0 )) || return 1
  echo $(( (after - before) * 100 / before ))
}

service_soak_assert_max_growth() {
  local label="$1"
  local before="$2"
  local after="$3"
  local max_growth="$4"
  local growth=$((after - before))
  if (( growth > max_growth )); then
    echo "!! ${label} growth ${growth} exceeds ${max_growth} (${before} -> ${after})" >&2
    return 1
  fi
}

# Latency is considered unstable only when the second-window p99 exceeds both
# the absolute local-service budget and the permitted relative drift. This
# avoids turning sub-millisecond timer noise into a false failure while still
# rejecting sustained tail-latency growth.
service_soak_assert_latency_plateau() {
  local before_ms="$1"
  local after_ms="$2"
  local max_p99_ms="$3"
  local max_growth_pct="$4"
  local relative_limit=$(( before_ms * (100 + max_growth_pct) / 100 ))
  if (( after_ms > max_p99_ms && after_ms > relative_limit )); then
    echo "!! p99 latency ${after_ms}ms exceeds ${max_p99_ms}ms and ${max_growth_pct}% drift limit from ${before_ms}ms" >&2
    return 1
  fi
}

# HANDWRITE-END
