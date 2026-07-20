#!/usr/bin/env bash
# HANDWRITE-BEGIN gap="missing-generator:e2e-test:shared-soak-resource-metrics" tracker="#1777" reason="Portable process-resource and latency plateau sampling is shared by service-specific bounded soak workloads."

# Shared bounded-soak measurements. Callers own the domain workload and source
# this file for portable process/resource sampling and plateau assertions.

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
