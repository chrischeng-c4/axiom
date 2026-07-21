#!/usr/bin/env bash
# HANDWRITE-BEGIN gap="missing-generator:e2e-test:shared-soak-resource-metrics" tracker="#1777" reason="Portable process-resource and latency plateau sampling is shared by service-specific bounded soak workloads."

# Shared bounded-soak measurements. Callers own the domain workload and source
# this file for portable process/resource sampling and plateau assertions.

service_soak_rss_sampler_expected_count() {
  local window_secs="$1"
  local sample_interval_secs="$2"
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
  local status
  local sampler_pid
  local token
  local expected_count
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
  status="${dir}/status"
  : >"${samples}"
  : >"${status}"
  _service_soak_rss_sampler_loop "$pid" "$window_secs" "$sample_interval_secs" "$samples" "$status" &
  sampler_pid=$!
  token="${pid}:${samples}:${status}:${sampler_pid}:${window_secs}:${sample_interval_secs}:${expected_count}"
  SERVICE_SOAK_RSS_LAST_TOKEN="${token}"
  SERVICE_SOAK_RSS_ACTIVE_TOKEN="${token}"
  if (( BASH_SUBSHELL == 0 )); then
    trap '_service_soak_rss_sampler_exit_cleanup' EXIT
    trap '_service_soak_rss_sampler_signal_cleanup INT 130' INT
    trap '_service_soak_rss_sampler_signal_cleanup TERM 143' TERM
  fi
  echo "${token}"
}

_service_soak_rss_sampler_loop() {
  local pid="$1"
  local window_secs="$2"
  local sample_interval_secs="$3"
  local samples="$4"
  local status="$5"
  local started_at
  local deadline
  local now
  local rss
  started_at="$(date +%s)"
  deadline=$(( started_at + window_secs ))
  while :; do
    now="$(date +%s)"
    if ! kill -0 "$pid" 2>/dev/null; then
      echo "pid_missing=1" >>"${status}"
      break
    fi
    rss="$(service_soak_rss_kb "$pid")"
    [[ "${rss}" =~ ^[0-9]+$ ]] || {
      echo "invalid_sample=${rss}" >>"${status}"
      break
    }
    echo "${rss}" >>"${samples}"
    (( now + sample_interval_secs > deadline )) && break
    sleep "${sample_interval_secs}"
  done
}

service_soak_rss_sampler_stop() {
  local token="$1"
  local pid samples status sampler_pid window_secs sample_interval_secs expected_count
  local rc=0
  IFS=: read -r pid samples status sampler_pid window_secs sample_interval_secs expected_count <<<"${token}"
  [[ -n "${pid}" && -n "${samples}" && -n "${status}" && -n "${sampler_pid}" ]] || {
    echo "invalid sampler token" >&2
    return 1
  }
  _service_soak_rss_sampler_cleanup "${token}" 1
  _service_soak_rss_sampler_validate_series "${pid}" "${samples}" "${status}" "${expected_count}" "${window_secs}" || rc=$?
  rm -rf "$(dirname "${samples}")"
  return "${rc}"
}

_service_soak_rss_sampler_cleanup() {
  local token="$1"
  local keep_artifacts="${2:-0}"
  local pid samples status sampler_pid window_secs sample_interval_secs expected_count
  IFS=: read -r pid samples status sampler_pid window_secs sample_interval_secs expected_count <<<"${token}"
  [[ -n "${sampler_pid}" ]] || return 0
  if kill -0 "${sampler_pid}" 2>/dev/null; then
    kill "${sampler_pid}" 2>/dev/null || true
  fi
  wait "${sampler_pid}" 2>/dev/null || true
  if [[ "${SERVICE_SOAK_RSS_ACTIVE_TOKEN:-}" == "${token}" ]]; then
    unset SERVICE_SOAK_RSS_ACTIVE_TOKEN
    trap - EXIT INT TERM
  fi
  if (( keep_artifacts == 0 )); then
    rm -rf "$(dirname "${samples}")"
  fi
}

_service_soak_rss_sampler_exit_cleanup() {
  [[ -n "${SERVICE_SOAK_RSS_ACTIVE_TOKEN:-}" ]] || return 0
  _service_soak_rss_sampler_cleanup "${SERVICE_SOAK_RSS_ACTIVE_TOKEN}" 0
}

_service_soak_rss_sampler_signal_cleanup() {
  local signal="$1"
  local code="$2"
  if [[ -n "${SERVICE_SOAK_RSS_ACTIVE_TOKEN:-}" ]]; then
    _service_soak_rss_sampler_cleanup "${SERVICE_SOAK_RSS_ACTIVE_TOKEN}" 0
  fi
  exit "${code}"
}

_service_soak_rss_sampler_validate_series() {
  local pid="$1"
  local samples="$2"
  local status="$3"
  local expected_count="$4"
  local window_secs="$5"
  local count
  [[ -f "${samples}" && -f "${status}" ]] || {
    echo "missing sampler artifacts" >&2
    return 1
  }
  if [[ -s "${status}" ]]; then
    if rg -q '^pid_missing=1$' "${status}"; then
      echo "pid ${pid} disappeared before ${window_secs}s window completed" >&2
      return 1
    fi
    if rg -q '^invalid_sample=' "${status}"; then
      echo "invalid RSS sample captured" >&2
      return 1
    fi
  fi
  if rg -n -v '^[0-9]+$' "${samples}" >/dev/null; then
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
  awk '
    /^[0-9]+$/ { value[++count] = $1; next }
    NF { invalid = 1 }
    END {
      if (invalid || count == 0) exit 1
      for (i = 1; i <= count; i += 1) {
        for (j = i + 1; j <= count; j += 1) {
          if (value[j] < value[i]) {
            tmp = value[i]
            value[i] = value[j]
            value[j] = tmp
          }
        }
      }
      min = value[1]
      max = value[count]
      if (count % 2 == 1) {
        median = value[(count + 1) / 2]
      } else {
        median = int((value[count / 2] + value[(count / 2) + 1]) / 2)
      }
      printf "count=%d min=%d median=%d max=%d\n", count, min, median, max
    }
  ' "${samples}"
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
