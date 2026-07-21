#!/usr/bin/env bash
# HANDWRITE-BEGIN gap="missing-generator:unit-test:93222ae9" tracker="#2285" reason="Prove transient-spike median stability, sustained-growth breach behavior, malformed or insufficient sample failures, and real-child sampler cleanup on both normal completion and interruption."
set -euo pipefail

cd "$(git rev-parse --show-toplevel)"
source libs/service-observability/scripts/soak-metrics.sh

tmp_root="$(mktemp -d "${TMPDIR:-/tmp}/soak-metrics-test.XXXXXX")"
cleanup() {
  rm -rf "${tmp_root}"
}
trap cleanup EXIT

assert_eq() {
  local expected="$1"
  local actual="$2"
  local label="$3"
  [[ "${expected}" == "${actual}" ]] || {
    echo "assert_eq failed for ${label}: expected=${expected} actual=${actual}" >&2
    return 1
  }
}

assert_nonzero() {
  local label="$1"
  shift
  if "$@"; then
    echo "expected non-zero exit for ${label}" >&2
    return 1
  fi
}

assert_file_contains() {
  local path="$1"
  local expected="$2"
  grep -Fq "${expected}" "${path}" || {
    echo "expected ${path} to contain: ${expected}" >&2
    return 1
  }
}

assert_ordered_summary() {
  local summary="$1"
  local count
  local min
  local median
  local max
  count="$(service_soak_summary_field "${summary}" count)"
  min="$(service_soak_summary_field "${summary}" min)"
  median="$(service_soak_summary_field "${summary}" median)"
  max="$(service_soak_summary_field "${summary}" max)"
  (( count >= 2 )) || {
    echo "expected summary count >= 2, got ${count}" >&2
    return 1
  }
  (( min <= median && median <= max )) || {
    echo "expected ordered min<=median<=max, got ${summary}" >&2
    return 1
  }
}

summary_validation() {
  local stable_samples="${tmp_root}/stable.samples"
  printf '100\n100\n500\n100\n100\n' >"${stable_samples}"
  local summary
  summary="$(service_soak_rss_window_summary "${stable_samples}")"
  assert_eq "count=5 min=100 median=100 max=500" "${summary}" "stable summary"

  local bad_samples="${tmp_root}/bad.samples"
  printf '100\nabc\n100\n' >"${bad_samples}"
  assert_nonzero "non-numeric summary" service_soak_rss_window_summary "${bad_samples}"

  local short_samples="${tmp_root}/short.samples"
  printf '100\n' >"${short_samples}"
  cat <<'EOF' >"${tmp_root}/short.meta"
completed=1
target_missing=0
EOF
  assert_nonzero "insufficient coverage" \
    _service_soak_rss_sampler_validate_series 99999 "${short_samples}" "${tmp_root}/short.meta" 2 2
}

median_growth_semantics() {
  local base_samples="${tmp_root}/base.samples"
  local spike_samples="${tmp_root}/spike.samples"
  local grown_samples="${tmp_root}/grown.samples"
  printf '100\n100\n100\n100\n100\n' >"${base_samples}"
  printf '100\n100\n500\n100\n100\n' >"${spike_samples}"
  printf '120\n120\n120\n120\n120\n' >"${grown_samples}"
  local base_summary spike_summary grown_summary
  base_summary="$(service_soak_rss_window_summary "${base_samples}")"
  spike_summary="$(service_soak_rss_window_summary "${spike_samples}")"
  grown_summary="$(service_soak_rss_window_summary "${grown_samples}")"
  assert_eq "0" "$(service_soak_rss_window_growth_pct "${base_summary}" "${spike_summary}")" "transient spike median growth"
  assert_eq "20" "$(service_soak_rss_window_growth_pct "${base_summary}" "${grown_summary}")" "sustained median growth"
}

summary_stop_output() {
  local child_pid
  sleep 4 &
  child_pid=$!
  service_soak_rss_sampler_start "${child_pid}" 2 1 >/dev/null
  local token="${SERVICE_SOAK_RSS_LAST_TOKEN}"
  sleep 3
  local summary
  summary="$(service_soak_rss_sampler_stop "${token}")"
  assert_ordered_summary "${summary}"
  assert_eq "0" "$(service_soak_rss_window_growth_pct "${summary}" "${summary}")" "stop summary growth parse"
  [[ ! -e "${token}" ]] || {
    echo "token dir still exists after stop: ${token}" >&2
    return 1
  }
  wait "${child_pid}" 2>/dev/null || true
}

early_stop_fails() {
  local child_pid
  sleep 5 &
  child_pid=$!
  service_soak_rss_sampler_start "${child_pid}" 3 1 >/dev/null
  local token="${SERVICE_SOAK_RSS_LAST_TOKEN}"
  sleep 1
  assert_nonzero "early stop" service_soak_rss_sampler_stop "${token}"
  [[ ! -e "${token}" ]] || {
    echo "token dir still exists after failed early stop: ${token}" >&2
    return 1
  }
  kill "${child_pid}" 2>/dev/null || true
  wait "${child_pid}" 2>/dev/null || true
}

dead_target_fails() {
  local child_pid
  sleep 2 &
  child_pid=$!
  service_soak_rss_sampler_start "${child_pid}" 5 1 >/dev/null
  local token="${SERVICE_SOAK_RSS_LAST_TOKEN}"
  sleep 3
  wait "${child_pid}" 2>/dev/null || true
  assert_nonzero "dead target" service_soak_rss_sampler_stop "${token}"
  [[ ! -e "${token}" ]] || {
    echo "token dir still exists after dead-target failure: ${token}" >&2
    return 1
  }
}

trap_preservation_and_cleanup() {
  local state_file="${tmp_root}/trap_state"
  bash "$0" __trap_helper "${state_file}" stop
  assert_eq "$(awk -F= '/^before_exit=/{sub(/^before_exit=/, ""); print}' "${state_file}.meta")" "$(awk -F= '/^after_start_exit=/{sub(/^after_start_exit=/, ""); print}' "${state_file}.meta")" "exit trap preserved after start"
  assert_eq "$(awk -F= '/^before_int=/{sub(/^before_int=/, ""); print}' "${state_file}.meta")" "$(awk -F= '/^after_start_int=/{sub(/^after_start_int=/, ""); print}' "${state_file}.meta")" "int trap preserved after start"
  assert_eq "$(awk -F= '/^before_term=/{sub(/^before_term=/, ""); print}' "${state_file}.meta")" "$(awk -F= '/^after_start_term=/{sub(/^after_start_term=/, ""); print}' "${state_file}.meta")" "term trap preserved after start"
  assert_eq "$(awk -F= '/^before_exit=/{sub(/^before_exit=/, ""); print}' "${state_file}.meta")" "$(awk -F= '/^after_stop_exit=/{sub(/^after_stop_exit=/, ""); print}' "${state_file}.meta")" "exit trap preserved after stop"
  assert_eq "$(awk -F= '/^before_int=/{sub(/^before_int=/, ""); print}' "${state_file}.meta")" "$(awk -F= '/^after_stop_int=/{sub(/^after_stop_int=/, ""); print}' "${state_file}.meta")" "int trap preserved after stop"
  assert_eq "$(awk -F= '/^before_term=/{sub(/^before_term=/, ""); print}' "${state_file}.meta")" "$(awk -F= '/^after_stop_term=/{sub(/^after_stop_term=/, ""); print}' "${state_file}.meta")" "term trap preserved after stop"
  assert_file_contains "${state_file}" "caller_cleanup"

  local token
  local owner_pid
  local term_state="${tmp_root}/term_state"
  bash "$0" __trap_helper "${term_state}" wait &
  owner_pid=$!
  local tries=0
  while [[ ! -f "${term_state}.token" ]]; do
    tries=$((tries + 1))
    (( tries < 50 )) || {
      echo "timed out waiting for trap helper token" >&2
      return 1
    }
    sleep 0.1
  done
  token="$(cat "${term_state}.token")"
  kill -TERM "${owner_pid}"
  wait "${owner_pid}" || true
  tries=0
  while [[ -e "${token}" ]]; do
    tries=$((tries + 1))
    (( tries < 30 )) || {
      echo "token dir still exists after TERM cleanup: ${token}" >&2
      return 1
    }
    sleep 0.1
  done
  assert_file_contains "${term_state}" "caller_cleanup"
}

sampler_lifecycle_cleanup() {
  summary_stop_output
  early_stop_fails
  dead_target_fails
  trap_preservation_and_cleanup
}

contract_all() {
  summary_validation
  median_growth_semantics
  local sampler_pid
  sampler_lifecycle_cleanup
}

case "${1:-all}" in
  __trap_helper)
    state_file="$2"
    mode="$3"
    cleanup_marker() { echo "caller_cleanup" >>"${state_file}"; }
    cleanup_signal() {
      if [[ -n "${token:-}" ]]; then
        service_soak_rss_sampler_abort "${token}" || true
      fi
      if [[ -n "${child_pid:-}" ]]; then
        kill "${child_pid}" 2>/dev/null || true
        wait "${child_pid}" 2>/dev/null || true
      fi
      cleanup_marker
    }
    trap cleanup_marker EXIT
    trap 'cleanup_signal; exit 0' INT TERM
    before_exit="$(trap -p EXIT)"
    before_int="$(trap -p INT)"
    before_term="$(trap -p TERM)"
    sleep 30 &
    child_pid=$!
    service_soak_rss_sampler_start "${child_pid}" 10 1 >/dev/null
    token="${SERVICE_SOAK_RSS_LAST_TOKEN}"
    after_start_exit="$(trap -p EXIT)"
    after_start_int="$(trap -p INT)"
    after_start_term="$(trap -p TERM)"
    if [[ "${mode}" == "stop" ]]; then
      sleep 11
      service_soak_rss_sampler_stop "${token}" >"${state_file}.summary"
      after_stop_exit="$(trap -p EXIT)"
      after_stop_int="$(trap -p INT)"
      after_stop_term="$(trap -p TERM)"
      {
        printf 'before_exit=%s\n' "${before_exit}"
        printf 'before_int=%s\n' "${before_int}"
        printf 'before_term=%s\n' "${before_term}"
        printf 'after_start_exit=%s\n' "${after_start_exit}"
        printf 'after_start_int=%s\n' "${after_start_int}"
        printf 'after_start_term=%s\n' "${after_start_term}"
        printf 'after_stop_exit=%s\n' "${after_stop_exit}"
        printf 'after_stop_int=%s\n' "${after_stop_int}"
        printf 'after_stop_term=%s\n' "${after_stop_term}"
        printf 'token=%s\n' "${token}"
      } >"${state_file}.meta"
      kill "${child_pid}" 2>/dev/null || true
      wait "${child_pid}" 2>/dev/null || true
      exit 0
    fi
    printf '%s\n' "${token}" >"${state_file}.token"
    while :; do
      sleep 1
    done
    ;;
  summary_validation) summary_validation ;;
  median_growth_semantics) median_growth_semantics ;;
  summary_stop_output) summary_stop_output ;;
  early_stop_fails) early_stop_fails ;;
  dead_target_fails) dead_target_fails ;;
  trap_preservation_and_cleanup) trap_preservation_and_cleanup ;;
  sampler_lifecycle_cleanup) sampler_lifecycle_cleanup ;;
  all) contract_all ;;
  *)
    echo "unknown case: ${1}" >&2
    exit 1
    ;;
esac
# HANDWRITE-END
