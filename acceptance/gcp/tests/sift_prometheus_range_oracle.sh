#!/usr/bin/env bash
set -euo pipefail

test_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
acceptance_root="$(cd "$test_dir/.." && pwd)"
oracle="$acceptance_root/scripts/sift-prometheus-range-smoke.jq"
test_root="$(mktemp -d "${TMPDIR:-/tmp}/sift-prometheus-range-oracle.XXXXXX")"
cleanup() {
  local status="$1"
  trap - EXIT INT TERM
  set +e
  find "$test_root" -type f -delete
  find "$test_root" -depth -type d -empty -delete
  exit "$status"
}
trap 'cleanup "$?"' EXIT
trap 'exit 130' INT TERM

epoch=1700000000
valid="$test_root/valid.json"
old_expectation="$test_root/old-expectation.json"
missing_step="$test_root/missing-step.json"
wrong_label="$test_root/wrong-label.json"

jq -n --argjson epoch "$epoch" '
  {
    status:"success",
    data:{
      resultType:"matrix",
      result:[{
        metric:{__name__:"sift_acceptance_total",fixture:"smoke-remote-write"},
        values:[[$epoch,"0"],[$epoch + 1,"1"],[$epoch + 2,"1"]]
      }]
    }
  }
' > "$valid"
jq '.data.result[0].values[0][1] = "1"' "$valid" > "$old_expectation"
jq 'del(.data.result[0].values[2])' "$valid" > "$missing_step"
jq '.data.result[0].metric.fixture = "wrong"' "$valid" > "$wrong_label"

jq -e --argjson epoch "$epoch" -f "$oracle" "$valid" >/dev/null || {
  echo "the live Prometheus range oracle rejected the valid smoke response" >&2
  exit 1
}
for rejected in "$old_expectation" "$missing_step" "$wrong_label"; do
  if jq -e --argjson epoch "$epoch" -f "$oracle" "$rejected" >/dev/null; then
    echo "the live Prometheus range oracle accepted an invalid response" >&2
    exit 1
  fi
done

rg -F -- 'sift-prometheus-range-smoke.jq' \
  "$acceptance_root/scripts/verify-sift-mvp.sh" >/dev/null || {
  echo "the GKE verifier does not use the tested Prometheus range oracle" >&2
  exit 1
}

echo "Sift Prometheus range smoke oracle: ok"
