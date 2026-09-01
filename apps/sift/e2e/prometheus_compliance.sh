#!/usr/bin/env bash
set -euo pipefail

# The upstream receiver suite primarily targets Remote Write 2.0. Sift MVP
# deliberately rejects 2.0, so run only the official RW1 compatibility cases.
# Sift's Rust prometheus_api E2E separately proves persistence and readback.
COMPLIANCE_COMMIT="67b8327a2e93dc28f64d4b21bbce00b362f565d5"
COMPLIANCE_ARCHIVE_SHA256="9c45856bef0b599445a7e72cd213d476434030fcd2e2e934e9242fedc799ccc8"
GO_IMAGE="docker.io/library/golang:1.25.0-bookworm@sha256:81dc45d05a7444ead8c92a389621fafabc8e40f8fd1a19d7e5df14e61e98bc1a"

repo_root="$(git -c core.fsmonitor=false rev-parse --show-toplevel)"
sift_bin="${SIFT_BIN:-$repo_root/target/debug/sift}"
for command in cargo curl docker tar; do
  command -v "$command" >/dev/null 2>&1 || {
    echo "required command not found: $command" >&2
    exit 1
  }
done

if [[ ! -x "$sift_bin" ]]; then
  cargo build -p sift --bin sift
fi

work_root="$(mktemp -d "${TMPDIR:-/tmp}/sift-prometheus-compliance.XXXXXX")"
sift_pid=""
cleanup() {
  local exit_code=$?
  if [[ -n "$sift_pid" ]]; then
    kill "$sift_pid" >/dev/null 2>&1 || true
    wait "$sift_pid" >/dev/null 2>&1 || true
  fi
  find "$work_root" -depth -delete >/dev/null 2>&1 || true
  exit "$exit_code"
}
trap cleanup EXIT INT TERM

archive="$work_root/prometheus-compliance.tar.gz"
curl -fsSL \
  "https://github.com/prometheus/compliance/archive/${COMPLIANCE_COMMIT}.tar.gz" \
  -o "$archive"
if command -v shasum >/dev/null 2>&1; then
  actual_archive_sha="$(shasum -a 256 "$archive" | awk '{print $1}')"
else
  command -v sha256sum >/dev/null 2>&1 || {
    echo "required command not found: shasum or sha256sum" >&2
    exit 1
  }
  actual_archive_sha="$(sha256sum "$archive" | awk '{print $1}')"
fi
[[ "$actual_archive_sha" == "$COMPLIANCE_ARCHIVE_SHA256" ]] || {
  echo "Prometheus compliance archive digest mismatch" >&2
  exit 1
}
tar -xzf "$archive" -C "$work_root"

http_port=17380
grpc_port=14317
"$sift_bin" serve \
  --host 0.0.0.0 \
  --port "$http_port" \
  --grpc-port "$grpc_port" \
  --data-dir "$work_root/data" \
  >"$work_root/sift.log" 2>&1 &
sift_pid=$!

ready=0
for _ in $(seq 1 100); do
  if curl -fsS "http://127.0.0.1:${http_port}/readyz" >/dev/null 2>&1; then
    ready=1
    break
  fi
  if ! kill -0 "$sift_pid" >/dev/null 2>&1; then
    echo "Sift stopped before the compliance test became ready" >&2
    sed -n '1,200p' "$work_root/sift.log" >&2
    exit 1
  fi
  sleep 0.1
done
[[ "$ready" == "1" ]] || {
  echo "Sift did not become ready for Prometheus compliance" >&2
  sed -n '1,200p' "$work_root/sift.log" >&2
  exit 1
}

suite_root="$work_root/compliance-${COMPLIANCE_COMMIT}"
config="$repo_root/apps/sift/e2e/prometheus_compliance.yml"

docker run --rm \
  --add-host host.docker.internal:host-gateway \
  --volume "$suite_root:/compliance:ro" \
  --volume "$config:/tmp/sift-prometheus-config.yml:ro" \
  --workdir /compliance/remotewrite/receiver \
  --env PROMETHEUS_RW2_COMPLIANCE_CONFIG_FILE=/tmp/sift-prometheus-config.yml \
  --env PROMETHEUS_RW2_COMPLIANCE_RECEIVERS=sift-rw1 \
  "$GO_IMAGE" \
  go test -count=1 -v -run '^TestRW1(BasicCompatibility|ErrorHandling)$' .

echo "Prometheus official RW1 receiver compatibility: ok"
echo "commit: $COMPLIANCE_COMMIT"
