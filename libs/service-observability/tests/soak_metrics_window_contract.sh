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
  : >"${tmp_root}/short.status"
  assert_nonzero "insufficient coverage" \
    _service_soak_rss_sampler_validate_series 99999 "${short_samples}" "${tmp_root}/short.status" 2 2
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

sampler_lifecycle_cleanup() {
  local child_pid
  sleep 5 &
  child_pid=$!
  service_soak_rss_sampler_start "${child_pid}" 2 1 >/dev/null
  local token="${SERVICE_SOAK_RSS_LAST_TOKEN}"
  sleep 2
  service_soak_rss_sampler_stop "${token}" >/dev/null
  local sampler_pid
  sampler_pid="$(awk -F: '{ print $4 }' <<<"${token}")"
  if kill -0 "${sampler_pid}" 2>/dev/null; then
    echo "sampler pid ${sampler_pid} still alive after stop" >&2
    return 1
  fi
  kill "${child_pid}" 2>/dev/null || true
  wait "${child_pid}" 2>/dev/null || true

  local signal_script="${tmp_root}/signal_cleanup.sh"
  cat >"${signal_script}" <<'EOF'
#!/usr/bin/env bash
set -euo pipefail
cd "$(git rev-parse --show-toplevel)"
source libs/service-observability/scripts/soak-metrics.sh
sleep 30 &
child_pid=$!
service_soak_rss_sampler_start "${child_pid}" 10 1 >/dev/null
token="${SERVICE_SOAK_RSS_LAST_TOKEN}"
echo "${token}" >"$1"
echo "ready" >"$2"
while :; do
  sleep 1
done
EOF
  chmod +x "${signal_script}"
  local token_file="${tmp_root}/signal.token"
  local ready_file="${tmp_root}/signal.ready"
  "${signal_script}" "${token_file}" "${ready_file}" &
  local owner_pid=$!
  local tries=0
  while [[ ! -f "${ready_file}" ]]; do
    tries=$((tries + 1))
    (( tries < 50 )) || {
      echo "timed out waiting for signal cleanup helper" >&2
      return 1
    }
    sleep 0.1
  done
  local signal_token signal_sampler_pid
  signal_token="$(cat "${token_file}")"
  signal_sampler_pid="$(awk -F: '{ print $4 }' <<<"${signal_token}")"
  kill -TERM "${owner_pid}"
  wait "${owner_pid}" || true
  if kill -0 "${signal_sampler_pid}" 2>/dev/null; then
    echo "sampler pid ${signal_sampler_pid} still alive after TERM cleanup" >&2
    return 1
  fi
}

case "${1:-all}" in
  summary_validation) summary_validation ;;
  median_growth_semantics) median_growth_semantics ;;
  sampler_lifecycle_cleanup) sampler_lifecycle_cleanup ;;
  all)
    summary_validation
    median_growth_semantics
    sampler_lifecycle_cleanup
    ;;
  *)
    echo "unknown case: ${1}" >&2
    exit 1
    ;;
esac
# HANDWRITE-END
