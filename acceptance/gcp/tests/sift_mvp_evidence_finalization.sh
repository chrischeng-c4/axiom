#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ACCEPTANCE_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
source "$ACCEPTANCE_ROOT/scripts/source-prefix.sh"
FINALIZER="$ACCEPTANCE_ROOT/scripts/finalize-sift-mvp-acceptance.sh"
tmp="$(mktemp -d "${TMPDIR:-/tmp}/sift-mvp-evidence.XXXXXX")"
cleanup_tmp() {
  find "$tmp" -type f -delete
  find "$tmp" -depth -type d -empty -delete
}
trap cleanup_tmp EXIT INT TERM
candidate_digest="$(printf '0%.0s' {1..64})"
candidate_git_sha="0123456789abcdef0123456789abcdef01234567"

write_verification() {
  local status="$1"
  jq -n --arg status "$status" --arg digest "$candidate_digest" \
    --arg git_sha "$candidate_git_sha" '{
    schema:"axiom.gcp.operator.verification.v1",
    project_id:"project-1",
    region:"asia-east1",
    gke_zone:"asia-east1-a",
    run_id:"sift-test",
    backup_bucket:"project-1-sift-test-backup",
    acceptance:{sift:{
      schema:"axiom.gcp.sift.mvp.verification.v1",
      status:$status,
      candidate:{
        sift_image:("example.invalid/sift@sha256:" + $digest),
        rig_image:("example.invalid/rig@sha256:" + $digest),
        acceptance_runner_image:("example.invalid/sift-acceptance-runner@sha256:" + $digest),
        git_sha:$git_sha,
        source_bundle_sha256:$digest,
        cloud_build_id:"build-0123456789ab",
        source_object_uri:"gs://project-1-cloudbuild/source/axiom-gcp-operator-sift-test/candidate.tar.gz",
        immutable:true
      },
      topology:{
        store_voters:3,
        control_replicas:3,
        gateway_replicas:1,
        query_replicas:1,
        peer_mtls:"passed",
        pvc_restart:"passed"
      },
      protocols:{
        otlp_http_json:"passed",
        otlp_http_protobuf:"passed",
        otlp_gzip:"passed",
        otlp_grpc:"passed",
        partial_success:"passed",
        remote_write_1:"passed",
        prometheus_query_range:"passed",
        remote_write_2_rejected_415:"passed",
        mcp_read_only_tools:"passed",
        mcp_host_origin:"passed",
        cross_project_denied:"passed",
        cross_project_same_id:"passed"
      },
      load:{
        duration_seconds:1800,
        offered_items_per_second:10000,
        expected_unique_items:18000000,
        expected_logs:9000000,
        expected_metric_points:5400000,
        expected_spans:3600000,
        observed:{requests:18000,failed:0,achieved_items_per_second:10000,p95_ms:100,p99_ms:200},
        event_id_digest:{algorithm:"xor-sha256-v1",expected:$digest,observed:$digest,match:true}
      },
      latency:{query_p95_ms:100,trace_read_p95_ms:100,tail_visible_ms:100},
      failover:{
        duration_seconds:300,
        expected_unique_items:3000000,
        leader_before:"0",
        leader_after:"1",
        stopped_vm:"node-0",
        observed:{requests:3000,failed:0,achieved_items_per_second:10000,p95_ms:100,p99_ms:200},
        event_id_digest:{algorithm:"xor-sha256-v1",expected:$digest,observed:$digest,match:true},
        acknowledged_data_loss:0,
        auto_repair:"passed"
      },
      archive:{
        gcs_iam_outage:"passed",
        wal_preserved:"passed",
        quorum_recovered_after_leader_restart:true,
        manifest_uri:"gs://project-1-sift-test-backup/manifest.json"
      },
      idempotency:{
        signals:["logs","metrics","traces"],
        immediate_retry:"passed",
        after_steady_load:"passed",
        after_vm_failover:"passed",
        after_telemetry_expiration:"passed"
      },
      retention:{
        day_29:"hot-query-passed",
        day_31:"cold-query-passed",
        day_181:"non-retryable-partial-rejection-passed",
        day_180_rollover:"bounded-generation-passed",
        scan_completed:true
      },
      restore:{
        fresh_pvc:"passed",
        new_cluster_id:true,
        restored_from:"gs://project-1-sift-test-backup/manifest.json",
        source_count:18000000,
        restored_count:18000000,
        source_digest:$digest,
        restored_digest:$digest,
        source_watermark:1,
        restored_watermark:1
      },
      cleanup_evidence:null
    }}
  }' > "$tmp/sift-mvp-verification.json"
}

write_cleanup() {
  local status="$1"
  local run_id="${2:-sift-test}"
  jq -n --arg status "$status" --arg run_id "$run_id" \
    --arg digest "$candidate_digest" --arg git_sha "$candidate_git_sha" '{
    schema:"axiom.gcp.operator.cleanup.v1",
    project_id:"project-1",
    region:"asia-east1",
    gke_zone:"asia-east1-a",
    run_id:$run_id,
    verified_at:"2026-09-01T00:00:00Z",
    status:$status,
    candidate:{
      sift_image:("example.invalid/sift@sha256:" + $digest),
      rig_image:("example.invalid/rig@sha256:" + $digest),
      acceptance_runner_image:("example.invalid/sift-acceptance-runner@sha256:" + $digest),
      git_sha:$git_sha,
      source_bundle_sha256:$digest,
      cloud_build_id:"build-0123456789ab",
      source_object_uri:"gs://project-1-cloudbuild/source/axiom-gcp-operator-sift-test/candidate.tar.gz",
      immutable:true
    },
    preserved:{artifact_registry:true,preexisting_apis:true}
  }' > "$tmp/cleanup.json"
}

write_verification verification-passed
write_cleanup clean
[[ ! -e "$tmp/acceptance.json" ]]
EVIDENCE_DIR="$tmp" "$FINALIZER"
jq -e '
  .schema == "axiom.gcp.operator.acceptance.v1"
  and .acceptance.sift.schema == "axiom.gcp.sift.mvp.acceptance.v1"
  and .acceptance.sift.status == "passed"
  and .acceptance.sift.cleanup_evidence.schema == "axiom.gcp.operator.cleanup.v1"
  and .acceptance.sift.cleanup_evidence.run_id == "sift-test"
  and .acceptance.sift.cleanup_evidence.status == "clean"
  and .acceptance.sift.cleanup_evidence.candidate == .acceptance.sift.candidate
' "$tmp/acceptance.json" >/dev/null
cmp "$tmp/acceptance.json" "$tmp/sift-mvp-acceptance.json"

rm -f "$tmp/acceptance.json" "$tmp/sift-mvp-acceptance.json"
write_verification verification-passed
jq '.acceptance.sift.load.observed.reports = []' \
  "$tmp/sift-mvp-verification.json" > "$tmp/truncated.json"
mv "$tmp/truncated.json" "$tmp/sift-mvp-verification.json"
if EVIDENCE_DIR="$tmp" "$FINALIZER" >/dev/null 2>&1; then
  echo "finalizer accepted evidence outside the full schema" >&2
  exit 1
fi
[[ ! -e "$tmp/acceptance.json" ]]

rm -f "$tmp/acceptance.json" "$tmp/sift-mvp-acceptance.json"
write_verification verification-passed
write_cleanup dirty
if EVIDENCE_DIR="$tmp" "$FINALIZER" >/dev/null 2>&1; then
  echo "finalizer accepted dirty cleanup evidence" >&2
  exit 1
fi
[[ ! -e "$tmp/acceptance.json" ]]

write_cleanup clean
write_verification failed
if EVIDENCE_DIR="$tmp" "$FINALIZER" >/dev/null 2>&1; then
  echo "finalizer accepted failed verification evidence" >&2
  exit 1
fi
[[ ! -e "$tmp/acceptance.json" ]]

write_verification verification-passed
write_cleanup clean another-run
if EVIDENCE_DIR="$tmp" "$FINALIZER" >/dev/null 2>&1; then
  echo "finalizer accepted cleanup evidence from another run" >&2
  exit 1
fi
[[ ! -e "$tmp/acceptance.json" ]]

write_cleanup clean
write_verification verification-passed
jq '.candidate.git_sha = "fedcba9876543210fedcba9876543210fedcba98"' \
  "$tmp/cleanup.json" > "$tmp/mismatched-cleanup.json"
mv "$tmp/mismatched-cleanup.json" "$tmp/cleanup.json"
if EVIDENCE_DIR="$tmp" "$FINALIZER" >/dev/null 2>&1; then
  echo "finalizer accepted cleanup evidence from another candidate" >&2
  exit 1
fi
[[ ! -e "$tmp/acceptance.json" ]]

write_cleanup clean
write_verification verification-passed
jq '.acceptance.sift = {
  schema:"axiom.gcp.sift.mvp.verification.v1",
  status:"verification-passed",
  operator_reconcile_1x1:"passed",
  scheduled_backup:"passed",
  gcs_backup:"passed",
  topology_beyond_1x1:"passed",
  cleanup_evidence:null
}' "$tmp/sift-mvp-verification.json" > "$tmp/legacy-bypass.json"
mv "$tmp/legacy-bypass.json" "$tmp/sift-mvp-verification.json"
if EVIDENCE_DIR="$tmp" "$FINALIZER" >/dev/null 2>&1; then
  echo "finalizer accepted legacy Sift fields as complete MVP evidence" >&2
  exit 1
fi
[[ ! -e "$tmp/acceptance.json" ]]

receipt_dir="$tmp/cleanup-receipt"
fake_bin="$tmp/bin"
mkdir -p "$receipt_dir" "$fake_bin"
printf '%s\n' \
  '#!/usr/bin/env bash' \
  'case "$*" in' \
  '  *"artifacts docker images describe"*) echo "not found" >&2; exit 1 ;;' \
  '  *"storage ls"*) echo "matched no URLs" >&2; exit 1 ;;' \
  '  *"builds list"*) printf "[]\\n" ;;' \
  '  *) exit 0 ;;' \
  'esac' > "$fake_bin/gcloud"
chmod +x "$fake_bin/gcloud"
source_prefix="gs://project-1-cloudbuild/source/axiom-gcp-operator-sift-test"
write_source_prefix_receipt \
  "$receipt_dir/source-prefix.json" "project-1" "sift-test" "$source_prefix"
jq -n --arg git_sha "$candidate_git_sha" \
  '{git_sha:$git_sha}' > "$receipt_dir/run.json"
jq -n --arg git_sha "$candidate_git_sha" --arg digest "$candidate_digest" \
  '{git_sha:$git_sha,source_bundle_sha256:$digest}' \
  > "$receipt_dir/candidate-source.json"
jq -n --arg git_sha "$candidate_git_sha" --arg digest "$candidate_digest" '
  {
    schema:"axiom.gcp.sift.candidate-gate.v1",
    git_sha:$git_sha,
    source_bundle_sha256:$digest,
    entrypoint:"apps/sift/test.sh --candidate",
    completed_at:"2026-09-02T00:00:00Z",
    status:"passed"
  }' > "$receipt_dir/candidate-gate.json"
jq -n --arg git_sha "$candidate_git_sha" --arg digest "$candidate_digest" '
  {
    build_id:"build-0123456789ab",
    git_sha:$git_sha,
    source_uri:"gs://project-1-cloudbuild/source/axiom-gcp-operator-sift-test/candidate.tar.gz",
    source_bundle_sha256:$digest,
    staged_source_sha256:$digest
  }' > "$receipt_dir/cloud-build-source-binding.json"
jq -n '
  {
    source:{
      storageSource:{
        bucket:"project-1-cloudbuild",
        object:"source/axiom-gcp-operator-sift-test/candidate.tar.gz"
      }
    }
  }' > "$receipt_dir/cloud-build-submit.json"
jq -n --arg digest "$candidate_digest" '
  {
    sift:("example.invalid/sift@sha256:" + $digest),
    rig:("example.invalid/rig@sha256:" + $digest),
    acceptance_runner:("example.invalid/sift-acceptance-runner@sha256:" + $digest)
  }' > "$receipt_dir/images.json"
printf '{}\n' > "$receipt_dir/sift-mvp-verification.json"
PATH="$fake_bin:$PATH" \
  PROJECT_ID=project-1 REGION=asia-east1 GKE_ZONE=asia-east1-a \
  RUN_ID=sift-test REGISTRY=example.invalid IMAGE_TAG=candidate \
  GCS_SOURCE_PREFIX="$source_prefix" \
  EVIDENCE_DIR="$receipt_dir" ACCEPTANCE_APPS=sift \
  PERSISTENT_CLUSTER_CHECK_REQUIRED=0 KUBERNETES_CHECK_REQUIRED=0 \
  "$ACCEPTANCE_ROOT/scripts/verify-clean.sh" >/dev/null
jq -e --arg git_sha "$candidate_git_sha" --arg digest "$candidate_digest" '
  .status == "clean"
  and .candidate.git_sha == $git_sha
  and .candidate.source_bundle_sha256 == $digest
  and .candidate.sift_image == ("example.invalid/sift@sha256:" + $digest)
  and .candidate.rig_image == ("example.invalid/rig@sha256:" + $digest)
  and .candidate.acceptance_runner_image == ("example.invalid/sift-acceptance-runner@sha256:" + $digest)
' "$receipt_dir/cleanup.json" >/dev/null

rm -f "$receipt_dir/cleanup.json"
jq '.status = "failed"' "$receipt_dir/candidate-gate.json" \
  > "$receipt_dir/failed-candidate-gate.json"
mv "$receipt_dir/failed-candidate-gate.json" "$receipt_dir/candidate-gate.json"
if PATH="$fake_bin:$PATH" \
  PROJECT_ID=project-1 REGION=asia-east1 GKE_ZONE=asia-east1-a \
  RUN_ID=sift-test REGISTRY=example.invalid IMAGE_TAG=candidate \
  GCS_SOURCE_PREFIX="$source_prefix" \
  EVIDENCE_DIR="$receipt_dir" ACCEPTANCE_APPS=sift \
  PERSISTENT_CLUSTER_CHECK_REQUIRED=0 KUBERNETES_CHECK_REQUIRED=0 \
  "$ACCEPTANCE_ROOT/scripts/verify-clean.sh" >/dev/null 2>&1; then
  echo "verify-clean accepted a failed candidate gate" >&2
  exit 1
fi
[[ ! -e "$receipt_dir/cleanup.json" ]]
jq '.status = "passed"' "$receipt_dir/candidate-gate.json" \
  > "$receipt_dir/passed-candidate-gate.json"
mv "$receipt_dir/passed-candidate-gate.json" "$receipt_dir/candidate-gate.json"

jq '.git_sha = "fedcba9876543210fedcba9876543210fedcba98"' \
  "$receipt_dir/cloud-build-source-binding.json" \
  > "$receipt_dir/mismatched-build.json"
mv "$receipt_dir/mismatched-build.json" \
  "$receipt_dir/cloud-build-source-binding.json"
if PATH="$fake_bin:$PATH" \
  PROJECT_ID=project-1 REGION=asia-east1 GKE_ZONE=asia-east1-a \
  RUN_ID=sift-test REGISTRY=example.invalid IMAGE_TAG=candidate \
  GCS_SOURCE_PREFIX="$source_prefix" \
  EVIDENCE_DIR="$receipt_dir" ACCEPTANCE_APPS=sift \
  PERSISTENT_CLUSTER_CHECK_REQUIRED=0 KUBERNETES_CHECK_REQUIRED=0 \
  "$ACCEPTANCE_ROOT/scripts/verify-clean.sh" >/dev/null 2>&1; then
  echo "verify-clean accepted cleanup evidence from mixed candidate sources" >&2
  exit 1
fi
[[ ! -e "$receipt_dir/cleanup.json" ]]

echo "Sift MVP evidence finalization E2E: ok"
