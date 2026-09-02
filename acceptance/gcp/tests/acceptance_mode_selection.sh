#!/usr/bin/env bash
set -euo pipefail

# HANDWRITE-BEGIN gap="missing-generator:unit-test:a87c6c67" tracker="2370" reason="The GKE harness needs a static regression oracle for acceptance-mode phase selection until shell-control-flow generation exists."
#
# This oracle replaces the LUMEN_ONLY one. That mode was deleted by the
# tape-mode refactor (ce6635f57a) and this file kept asserting it existed, so
# check.sh had been failing ever since -- and failing SILENTLY, because every
# assertion was a bare `rg ... >/dev/null` whose only output under `set -e` is
# the exit status. A gate nobody can read is a gate nobody runs. Every
# assertion below therefore names itself on failure.
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ACCEPTANCE_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
RUN_SCRIPT="$ACCEPTANCE_ROOT/scripts/run.sh"
RENDER_SCRIPT="$ACCEPTANCE_ROOT/scripts/render-manifests.sh"
CLEANUP_SCRIPT="$ACCEPTANCE_ROOT/scripts/cleanup.sh"
VERIFY_CLEAN_SCRIPT="$ACCEPTANCE_ROOT/scripts/verify-clean.sh"
CELL_SCRIPT="$ACCEPTANCE_ROOT/scripts/verify-operator-cell.sh"
DEPLOY_SCRIPT="$ACCEPTANCE_ROOT/scripts/deploy.sh"
BOOTSTRAP_SCRIPT="$ACCEPTANCE_ROOT/scripts/bootstrap-cluster.sh"
SIFT_VERIFY_SCRIPT="${SIFT_VERIFY_SCRIPT_OVERRIDE:-$ACCEPTANCE_ROOT/scripts/verify-sift-mvp.sh}"
SIFT_LOAD_DIGEST_SCRIPT="$ACCEPTANCE_ROOT/scripts/sift-load-digest.py"
SIFT_FINALIZER="$ACCEPTANCE_ROOT/scripts/finalize-sift-mvp-acceptance.sh"
SIFT_EVIDENCE_VALIDATOR="$ACCEPTANCE_ROOT/scripts/validate-sift-mvp-evidence.py"
PROCESS_TREE_SCRIPT="$ACCEPTANCE_ROOT/scripts/process-tree.sh"
SIFT_AUTH_DELEGATOR_FILTER="$ACCEPTANCE_ROOT/scripts/sift-auth-delegator.jq"
SIFT_ARCHIVE_FQDN_FILTER="$ACCEPTANCE_ROOT/scripts/sift-archive-fqdn-policy.jq"
ENV_VARIABLES="$ACCEPTANCE_ROOT/environment/variables.tf"
ENV_GKE="$ACCEPTANCE_ROOT/environment/gke.tf"
CLUSTER_TF="$ACCEPTANCE_ROOT/cluster/main.tf"
SCHEMA="$ACCEPTANCE_ROOT/evidence/schema.json"

fail() {
  echo "acceptance-mode oracle: $1" >&2
  exit 1
}

# Both helpers ignore comment lines, and that is not pedantry: the first draft
# of this oracle used a bare substring match, so commenting out `trap cleanup
# EXIT` -- i.e. disarming the mandatory GCP teardown -- still satisfied it. An
# assertion a `#` can defeat is not an assertion.
uncommented() { # uncommented <pattern> <file>
  rg -F -- "$1" "$2" 2>/dev/null | rg -v '^\s*#' >/dev/null
}

present() { # present <label> <pattern> <file>
  uncommented "$2" "$3" || fail "$1 (expected live in ${3##*/}: $2)"
}

present_re() { # present_re <label> <anchored-regex> <file>
  rg -- "$2" "$3" 2>/dev/null | rg -v '^\s*#' >/dev/null \
    || fail "$1 (expected exact live code in ${3##*/}: $2)"
}

absent() { # absent <label> <pattern> <file>
  ! uncommented "$2" "$3" || fail "$1 (unexpectedly still live in ${3##*/}: $2)"
}

line_of() { # line_of <pattern> <file>
  rg -n -F -- "$1" "$2" | head -1 | cut -d: -f1
}

run_process_group_exit_race_child() {
  source "$PROCESS_TREE_SCRIPT"
  root_pid="$$"
  group_id=""
  local worker_pid
  watchdog_pid=""
  local watchdog_self
  scan_attempt=""
  records="${SIFT_PROCESS_GROUP_RACE_RECORDS:?}"
  watchdog_pid_file="${records}.watchdog-pid"
  local child_pid_file="${SIFT_PROCESS_GROUP_RACE_CHILD_PID:?}"
  group_id="$(process_group_id "$root_pid")"
  [[ "$group_id" == "$root_pid" ]] || return 1

  python3 -c '
import os
import signal
import subprocess
import time

def stop(_signum, _frame):
    child = subprocess.Popen(
        ["bash", "-c", "trap : TERM; while :; do sleep 30; done"],
        stdout=subprocess.DEVNULL,
        stderr=subprocess.DEVNULL,
    )
    with open(os.environ["SIFT_PROCESS_GROUP_RACE_CHILD_PID"], "w", encoding="utf-8") as output:
        output.write(str(child.pid))
    os._exit(0)

signal.signal(signal.SIGTERM, stop)
while True:
    time.sleep(30)
' >/dev/null 2>&1 &
  worker_pid="$!"

  process_group_race_cleanup() {
    trap - EXIT INT TERM
    kill "$watchdog_pid" >/dev/null 2>&1 || true
    wait "$watchdog_pid" >/dev/null 2>&1 || true
    for scan_attempt in 1 2 3 4 5; do
      append_process_group_members "$group_id" "$root_pid" "" "$records"
      signal_recorded_processes "$records" KILL
      [[ "$scan_attempt" == "5" ]] || sleep 1
    done
  }
  trap process_group_race_cleanup EXIT INT TERM

  (
    watchdog_self="$(wait_for_process_id_file "$watchdog_pid_file" "$root_pid")" \
      || exit 1
    record_process_group_members \
      "$group_id" "$root_pid" "$watchdog_self" "$records"
    signal_recorded_processes "$records" TERM
    sleep 10
    append_process_group_members \
      "$group_id" "$root_pid" "$watchdog_self" "$records"
    signal_recorded_processes "$records" KILL
  ) &
  watchdog_pid="$!"
  printf '%s\n' "$watchdog_pid" > "${watchdog_pid_file}.tmp"
  mv "${watchdog_pid_file}.tmp" "$watchdog_pid_file"

  wait "$worker_pid" || true
  [[ -s "$child_pid_file" ]]
}

if [[ "${SIFT_PROCESS_GROUP_EXIT_RACE_CHILD:-0}" == "1" ]]; then
  run_process_group_exit_race_child
  exit
fi

# Every script, by glob, not by name. This loop used to enumerate six of the
# twelve, which meant the three verify-*.sh scripts -- where the actual per-leg
# assertions live and where the file keeps growing -- were the only ones a
# syntax error could reach production in. A new script is covered the day it
# lands, not the day someone remembers to add it here.
shopt -s nullglob
acceptance_scripts=("$ACCEPTANCE_ROOT"/scripts/*.sh)
shopt -u nullglob
[[ ${#acceptance_scripts[@]} -ge 6 ]] || fail "found ${#acceptance_scripts[@]} acceptance scripts; the glob is not matching"
for script in "${acceptance_scripts[@]}"; do
  bash -n "$script" || fail "shell syntax error in ${script##*/}"
done
jq empty "$SCHEMA" || fail "evidence schema is not valid JSON"

# Start run.sh itself as a process-group leader. The outer wrapper must fork a
# new session child instead of calling setsid() in that leader and failing with
# EPERM. A missing PROJECT_ID is the first expected error after isolation.
if ! python3 -c '
import os
import subprocess
import sys

environment = os.environ.copy()
environment.pop("AXIOM_GCP_ACCEPTANCE_ISOLATED_SESSION", None)
environment.pop("PROJECT_ID", None)
result = subprocess.run(
    [sys.argv[1]],
    env=environment,
    stdout=subprocess.PIPE,
    stderr=subprocess.PIPE,
    text=True,
    start_new_session=True,
)
if result.returncode == 0:
    raise SystemExit(1)
if "Set PROJECT_ID explicitly" not in result.stderr:
    sys.stderr.write(result.stderr)
    raise SystemExit(1)
' "$RUN_SCRIPT"; then
  fail "isolated-session wrapper failed when run.sh was already a process-group leader"
fi

# Prove the real EXIT-cleanup race. The watchdog sends TERM and starts its
# grace period. The foreground worker then exits, which starts main cleanup
# and kills the watchdog before its second scan. Cleanup must rescan the group
# itself and kill the reparented TERM-handler child.
if [[ "${SIFT_ORACLE_MUTATION_CHILD:-0}" != "1" ]] && ! (
  source "$PROCESS_TREE_SCRIPT"
  process_test_dir="$(mktemp -d "${TMPDIR:-/tmp}/sift-process-tree.XXXXXX")"
  process_test_records="$process_test_dir/records.txt"
  process_test_child_pid="$process_test_dir/child-pid.txt"
  process_test_alive() {
    local pid expected current
    while IFS=$'\t' read -r pid expected; do
      current="$(process_start_token "$pid")"
      [[ -n "$current" && "$current" == "$expected" ]] && return 0
    done < "$process_test_records"
    return 1
  }
  process_test_cleanup() {
    signal_recorded_processes "$process_test_records" KILL || true
    find "$process_test_dir" -type f -delete
    find "$process_test_dir" -depth -type d -empty -delete
  }
  trap process_test_cleanup EXIT INT TERM
  SIFT_PROCESS_GROUP_EXIT_RACE_CHILD=1 \
    SIFT_PROCESS_GROUP_RACE_RECORDS="$process_test_records" \
    SIFT_PROCESS_GROUP_RACE_CHILD_PID="$process_test_child_pid" \
    python3 -c '
import os
import subprocess
import sys

result = subprocess.run([sys.argv[1]], env=os.environ.copy(), start_new_session=True)
raise SystemExit(result.returncode)
' "$SCRIPT_DIR/acceptance_mode_selection.sh" || exit 1
  [[ -s "$process_test_child_pid" && -s "$process_test_records" ]] || exit 1
  for _ in 1 2 3 4 5 6 7 8 9 10; do
    process_test_alive || exit 0
    sleep 1
  done
  exit 1
); then
  fail "process-group watchdog did not stop a reparented TERM-handler child"
fi

# --- the mode enum is closed, and both scripts close it the same way --------
# The two scripts branch independently. When they disagreed about which modes
# exist, the harness rendered one app's manifests and verified another's.
present "run.sh lost the lumen-sift mode"      '"lumen sift") acceptance_mode="lumen-sift" ;;' "$RUN_SCRIPT"
present "run.sh lost the lumen-auth mode"       '"lumen auth") acceptance_mode="lumen-auth" ;;' "$RUN_SCRIPT"
present "run.sh lost the tape mode"            '"tape") acceptance_mode="tape" ;;'             "$RUN_SCRIPT"
present "run.sh lost the sift-only MVP mode"   '"sift") acceptance_mode="sift" ;;'             "$RUN_SCRIPT"
present "run.sh accepts an unknown mode"       "lumen auth" "$RUN_SCRIPT"
present "render-manifests lost lumen modes"   '"lumen sift"|"lumen auth")' "$RENDER_SCRIPT"
present "render-manifests lost tape"           '"tape")'       "$RENDER_SCRIPT"
present "render-manifests lost sift-only mode" '"sift")'       "$RENDER_SCRIPT"
present "render-manifests accepts an unknown mode" "ACCEPTANCE_APPS must be 'lumen sift', 'lumen auth', 'sift', or 'tape'" "$RENDER_SCRIPT"
present "cleanup lost lumen-auth"              '"lumen auth") acceptance_mode="lumen-auth" ;;' "$CLEANUP_SCRIPT"
present "verify-clean lost lumen-auth"         '"lumen auth") acceptance_mode="lumen-auth" ;;' "$VERIFY_CLEAN_SCRIPT"
present "cleanup lost sift-only mode"          '"sift") acceptance_mode="sift" ;;' "$CLEANUP_SCRIPT"
present "verify-clean lost sift-only mode"     '"sift") acceptance_mode="sift" ;;' "$VERIFY_CLEAN_SCRIPT"
present "auth-only invokes finalizer"          'finalize-lumen-acceptance.sh" lumen-auth' "$RUN_SCRIPT"
absent  "the deleted LUMEN_ONLY mode came back in run.sh"    'LUMEN_ONLY' "$RUN_SCRIPT"
absent  "the deleted LUMEN_ONLY mode came back in cleanup"   'LUMEN_ONLY' "$CLEANUP_SCRIPT"

# --- cleanup is armed on every exit path -----------------------------------
# The standing requirement is that GCP resources are released whether the run
# passes or fails, so the trap and its completion sentinel are contract, not
# housekeeping. `run_completed` exists because a `set -u` expansion error
# aborts without updating $?, which made two real runs exit 0 mid-flight.
present "cleanup is no longer trapped on EXIT" 'trap cleanup EXIT'                  "$RUN_SCRIPT"
present "the interrupt trap is gone"           "trap 'exit 130' INT"                "$RUN_SCRIPT"
present "the cloud-time cap trap is gone"      '45-minute cloud acceptance cap reached' "$RUN_SCRIPT"
present "the acceptance run is no longer isolated in its own process group" \
  'start_new_session=True' "$RUN_SCRIPT"
present "the outer isolation wrapper no longer kills surviving group members" \
  'os.killpg(child.pid, signal.SIGKILL)' "$RUN_SCRIPT"
absent "the watchdog again depends on Bash 4 BASHPID" 'BASHPID' "$RUN_SCRIPT"
present "the cloud-time cap no longer stops descendant processes" \
  'signal_recorded_processes "$watchdog_descendants" KILL' "$RUN_SCRIPT"
present "the cloud-time cap no longer records the isolated process group" \
  'record_process_group_members' "$RUN_SCRIPT"
present "the cloud-time cap misses group members created during TERM grace" \
  'append_process_group_members' "$RUN_SCRIPT"
present "EXIT cleanup no longer rescans after stopping the watchdog" \
  'for scan_attempt in 1 2 3 4 5; do' "$RUN_SCRIPT"
present "EXIT cleanup no longer kills the watchdog process tree" \
  'signal_recorded_processes "$watchdog_descendants" KILL' "$RUN_SCRIPT"
present "the false-green sentinel is gone"     'run_completed=1'                    "$RUN_SCRIPT"
present "cleanup no longer refuses ec=0 without the sentinel" 'run aborted before completion' "$RUN_SCRIPT"
present "the backup service account is no longer swept" 'wait_for_empty "backup service account"' "$VERIFY_CLEAN_SCRIPT"

# --- Sift MVP is independent, bounded, and evidence-complete --------------
present "sift-only mode does not invoke its verifier" 'verify-sift-mvp.sh"' "$RUN_SCRIPT"
present "sift-only mode lost the 90-minute cap" 'sift_cloud_cap=5400' "$RUN_SCRIPT"
present "sift-only mode accepts a caller-supplied image again" \
  'prebuilt images are not accepted' "$RUN_SCRIPT"
present "sift-only mode accepts a caller-supplied CLI again" \
  'caller-supplied SIFT_CLI is not accepted' "$RUN_SCRIPT"
present "sift-only mode no longer builds Sift and Rig in one receipt" \
  'cloudbuild.sift-mvp.yaml' "$RUN_SCRIPT"
present "Sift candidate source is no longer archived from the fixed Git commit" \
  'git -c core.fsmonitor=false -C "$REPO_ROOT" archive' "$RUN_SCRIPT"
present "Sift Cloud Build again submits the mutable worktree" \
  'gcloud builds submit "$CANDIDATE_SOURCE_ARCHIVE"' "$RUN_SCRIPT"
present "Sift local CLI is no longer built from the fixed candidate archive" \
  '--manifest-path "$candidate_source_dir/Cargo.toml"' "$RUN_SCRIPT"
present "Sift Cloud Build no longer checks the uploaded source bytes" \
  'Cloud Build staged source does not match the fixed candidate archive' "$RUN_SCRIPT"
present "Sift Cloud Build receipts are no longer bound to the candidate SHA" \
  '.substitutions._GIT_SHA == $git_sha' "$RUN_SCRIPT"
present "Sift Cloud Build receipts are no longer bound to the candidate source digest" \
  '.substitutions._SOURCE_BUNDLE_SHA256 == $source_bundle_sha256' "$RUN_SCRIPT"
present "Sift Cloud Build receipts are no longer bound to the run" \
  '.substitutions._RUN_ID == $run_id' "$RUN_SCRIPT"
present "Sift Cloud Build receipts no longer bind the deployed image digests" \
  'any(.results.images[]?; .name == $name and .digest == $digest)' "$RUN_SCRIPT"
candidate_digest_line="$(line_of 'SIFT_IMAGE="$(resolve_digest sift)"' "$RUN_SCRIPT")"
gke_bootstrap_line="$(line_of 'echo ">> persistent Standard GKE cluster bootstrap or reuse"' "$RUN_SCRIPT")"
[[ -n "$candidate_digest_line" && -n "$gke_bootstrap_line" \
  && "$candidate_digest_line" -lt "$gke_bootstrap_line" ]] \
  || fail "GKE starts before the Sift and Rig candidate digests are fixed"
present "cleanup no longer discovers every run-tagged Cloud Build" \
  '--filter="tags=axiom-run-${RUN_ID}"' "$CLEANUP_SCRIPT"
present "cleanup no longer waits for Cloud Build cancellation" \
  'Cloud Build $build_id did not reach a terminal state after cancellation' "$CLEANUP_SCRIPT"
present "verify-clean no longer rejects active run-tagged Cloud Builds" \
  'a run-tagged Cloud Build is still active or has an unknown status' "$VERIFY_CLEAN_SCRIPT"
present_re "the Sift verifier accepts a mutable Sift image" \
  '^\[\[ "\$SIFT_IMAGE" == \*@sha256:\* \]\] \|\| \{$' "$SIFT_VERIFY_SCRIPT"
present_re "the Sift verifier accepts a mutable Rig image" \
  '^\[\[ "\$RIG_IMAGE" == \*@sha256:\* \]\] \|\| \{$' "$SIFT_VERIFY_SCRIPT"
present "terminal candidate evidence lost its source bundle digest" \
  'source_bundle_sha256:$source_bundle_sha256' "$SIFT_VERIFY_SCRIPT"
present "terminal candidate evidence lost its Cloud Build id" \
  'cloud_build_id:$cloud_build_id' "$SIFT_VERIFY_SCRIPT"
present "terminal candidate evidence lost its staged source object" \
  'source_object_uri:$source_object_uri' "$SIFT_VERIFY_SCRIPT"
present "the paired Sift build lost its source digest tag" \
  'axiom-source-${_SOURCE_BUNDLE_SHA256}' "$ACCEPTANCE_ROOT/cloudbuild.sift-mvp.yaml"
present "Terraform lost the sift-only enum" '"sift"' "$ENV_VARIABLES"
present "Terraform lost the run-scoped Sift node pool" 'resource "google_container_node_pool" "sift_mvp"' "$ENV_GKE"
rg -q '^\s*disk_type\s*=\s*"pd-standard"\s*$' "$ENV_GKE" \
  || fail "the Sift node pool boot disks consume the PVC SSD quota"
present_re "the Sift verifier lost its 30-minute load" \
  '^LOAD_SECONDS=1800$' "$SIFT_VERIFY_SCRIPT"
present_re "the Sift verifier lost the 10k/s rate" \
  '^ITEMS_PER_SECOND=10000$' "$SIFT_VERIFY_SCRIPT"
present_re "the Sift verifier lost its 18M exact count" \
  '^EXPECTED_ITEMS=18000000$' "$SIFT_VERIFY_SCRIPT"
present "the Sift verifier no longer requires the run-scoped node-pool name" \
  'SIFT_NODE_POOL is required' "$SIFT_VERIFY_SCRIPT"
present "the Sift verifier no longer validates the run-scoped node-pool shape" \
  'sift-node-pool.json' "$SIFT_VERIFY_SCRIPT"
present "the Sift verifier no longer checks actual pod node placement" \
  'verify_pods_on_run_nodes() {' "$SIFT_VERIFY_SCRIPT"
absent "the gRPC probe escaped to the shared acceptance pool" \
  'cloud.google.com/gke-nodepool: acceptance-pool' "$SIFT_VERIFY_SCRIPT"
present "the Sift verifier lost GCS outage testing" 'archive-iam-disabled' "$SIFT_VERIFY_SCRIPT"
present "the Sift verifier does not timestamp the real GCS outage" \
  'archive_outage_started_at=' "$SIFT_VERIFY_SCRIPT"
present "the Sift verifier does not restrict logs to the real outage" \
  '--since-time="$archive_outage_started_at"' "$SIFT_VERIFY_SCRIPT"
absent "the Sift verifier can reuse a stale archive failure" \
  '--since=7m' "$SIFT_VERIFY_SCRIPT"
present "the Sift verifier lost fresh-PVC restore" 'fresh-pvc-restore' "$SIFT_VERIFY_SCRIPT"
present "the Sift verifier no longer requires archive V10" \
  '.format_version == 10' "$SIFT_VERIFY_SCRIPT"
present "the Sift verifier no longer checks the paged archive catalog" \
  '.catalog_root.entry_count == (.segment_count + .blob_count + .dedupe_receipt_count)' "$SIFT_VERIFY_SCRIPT"
present "the Sift verifier no longer requires archived segments" \
  '.segment_count > 0' "$SIFT_VERIFY_SCRIPT"
absent "the Sift verifier still expects the removed inline segment array" \
  '(.segments | length) > 0' "$SIFT_VERIFY_SCRIPT"
present "the Sift verifier no longer compares every voter after failover" \
  'wait_for_store_convergence' "$SIFT_VERIFY_SCRIPT"
present "the Sift verifier no longer checks bounded resident Raft logs" \
  '.resident_log_bytes <= 536870912' "$SIFT_VERIFY_SCRIPT"
present "the Sift verifier no longer checks every restored voter" \
  'restore_voters_converged=true' "$SIFT_VERIFY_SCRIPT"
absent "the Sift verifier emits terminal evidence before cleanup" \
  'acceptance.json' "$SIFT_VERIFY_SCRIPT"
present "the Sift verifier lost pre-cleanup evidence" \
  'sift-mvp-verification.json' "$SIFT_VERIFY_SCRIPT"
present "cleanup no longer finalizes Sift evidence after verify-clean" \
  'sift-mvp-verification.json' "$CLEANUP_SCRIPT"
present "cleanup no longer requires a clean cleanup receipt" \
  '$cleanup[0].status != "clean"' "$SIFT_FINALIZER"
present "cleanup no longer invokes the Sift evidence finalizer" \
  'finalize-sift-mvp-acceptance.sh' "$CLEANUP_SCRIPT"
present "the finalizer no longer creates terminal Sift evidence" \
  'mv "$output_tmp" "$output"' "$SIFT_FINALIZER"
[[ -f "$SIFT_EVIDENCE_VALIDATOR" ]] \
  || fail "the full Sift evidence schema validator is missing"
present "the verifier no longer validates pre-cleanup evidence" \
  'validate-sift-mvp-evidence.py' "$SIFT_VERIFY_SCRIPT"
present "the finalizer no longer validates terminal evidence" \
  'validate-sift-mvp-evidence.py' "$SIFT_FINALIZER"
present "cleanup queries can again hide API failures as empty results" \
  'inventory_output() {' "$VERIFY_CLEAN_SCRIPT"
present "cleanup no longer verifies newly deleted image digests" \
  'deleted-image-*.txt' "$VERIFY_CLEAN_SCRIPT"
present "the Sift verifier lost its authenticated non-2xx status helper" \
  'auth_curl_status() {' "$SIFT_VERIFY_SCRIPT"
present "Remote Write 2.0 rejection again aborts before its 415 assertion" \
  'remote_write_v2_status="$(auth_curl_status ' "$SIFT_VERIFY_SCRIPT"
present "the Sift verifier lost its same-ID retry probe" \
  'smoke-idempotency' "$SIFT_VERIFY_SCRIPT"
present "the Sift verifier no longer proves retry count and digest stability" \
  'idempotency retry changed durable project identity' "$SIFT_VERIFY_SCRIPT"
present "the Sift verifier lost delayed idempotency retries" \
  'verify_idempotency_retry after-steady-load' "$SIFT_VERIFY_SCRIPT"
present "the Sift verifier lost failover idempotency retries" \
  'verify_idempotency_retry after-vm-failover' "$SIFT_VERIFY_SCRIPT"
present "the Sift verifier no longer retries every signal" \
  'for signal in logs metrics traces; do' "$SIFT_VERIFY_SCRIPT"
present "the Sift load no longer rejects OTLP partial-success bodies" \
  '"$.partialSuccess" = "absent"' "$SIFT_VERIFY_SCRIPT"
present "the steady phase lost its independent event-ID digest" \
  'verify_load_digest steady "$LOAD_SECONDS"' "$SIFT_VERIFY_SCRIPT"
present "the failover phase lost its independent event-ID digest" \
  'verify_load_digest failover "$FAILOVER_SECONDS"' "$SIFT_VERIFY_SCRIPT"
present "steady and failover trace IDs no longer use separate prefixes" \
  'trace_prefix=57ead0000000' "$SIFT_VERIFY_SCRIPT"
present "failover trace IDs lost their phase prefix" \
  'trace_prefix=fa1100000000' "$SIFT_VERIFY_SCRIPT"
present "latency query samples no longer validate response bodies" \
  'query-response-${sample}.json' "$SIFT_VERIFY_SCRIPT"
present "latency trace samples no longer validate response bodies" \
  'trace-response-${sample}.json' "$SIFT_VERIFY_SCRIPT"
present "the verifier lost the authorized same-ID project proof" \
  'cross-project-same-id' "$SIFT_VERIFY_SCRIPT"
present "the acceptance Role no longer authorizes the second test project" \
  'resourceNames: ["sift-mvp", "sift-mvp-alt"]' "$RENDER_SCRIPT"
present_re "the GCS outage no longer invokes quorum recovery" \
  '^verify_outage_quorum_recovery "\$archive_leader" \\$' "$SIFT_VERIFY_SCRIPT"
present "the GCS outage no longer restarts the current leader" \
  'pod/sift-store-${stopped_leader}' "$SIFT_VERIFY_SCRIPT"
present_re "the failover leg no longer invokes its pod identity comparison" \
  '^assert_failover_restart_evidence [\\]$' "$SIFT_VERIFY_SCRIPT"
present_re "the peer mTLS negative gate is no longer invoked" \
  '^verify_peer_mtls_rejection$' "$SIFT_VERIFY_SCRIPT"
present "peer mTLS no longer rejects clients without a certificate" \
  'peer-mtls-no-client.stderr' "$SIFT_VERIFY_SCRIPT"
present "peer mTLS no longer rejects an untrusted CA" \
  'peer-mtls-wrong-ca.stderr' "$SIFT_VERIFY_SCRIPT"
for tool in sift_query sift_get_trace sift_correlate sift_list_services sift_tail_logs; do
  present_re "MCP no longer calls ${tool}" \
    "^  mcp_call [0-9]+ ${tool} " "$SIFT_VERIFY_SCRIPT"
done
present "MCP query no longer proves the known log content" \
  'MCP sift_query did not return the known smoke log' "$SIFT_VERIFY_SCRIPT"
present "MCP trace no longer proves the known span content" \
  'MCP sift_get_trace did not return the known smoke span' "$SIFT_VERIFY_SCRIPT"
present "MCP correlation no longer proves related signal content" \
  'MCP sift_correlate did not return known correlated signals' "$SIFT_VERIFY_SCRIPT"
present "MCP service listing no longer proves all three signals" \
  'MCP sift_list_services did not return the three-signal smoke service' "$SIFT_VERIFY_SCRIPT"
present "MCP tail no longer proves the known log content" \
  'MCP sift_tail_logs did not return the known smoke log' "$SIFT_VERIFY_SCRIPT"
present "MCP query no longer proves cross-project isolation" \
  'MCP sift_query leaked the smoke log into another project' "$SIFT_VERIFY_SCRIPT"
present "MCP bad-Origin probe no longer uses valid authentication" \
  'bad_origin_status="$(curl' "$SIFT_VERIFY_SCRIPT"
present "cross-project allowed-header/denied-body mismatch is no longer tested" \
  'cross-project-allowed-header-denied-body.json' "$SIFT_VERIFY_SCRIPT"
present "cross-project denied-header/allowed-body mismatch is no longer tested" \
  'cross-project-denied-header-allowed-body.json' "$SIFT_VERIFY_SCRIPT"
present "Prometheus acceptance can again pass with an empty vector" \
  'and (.data.result | length) == 1' "$SIFT_VERIFY_SCRIPT"
present "restore convergence no longer requires commit indexes" \
  'and (map(.commit_index) | unique | length == 1)' "$SIFT_VERIFY_SCRIPT"
present_re "candidate image provenance gate is no longer invoked" \
  '^verify_sift_image_provenance$' "$SIFT_VERIFY_SCRIPT"
[[ -f "$SIFT_LOAD_DIGEST_SCRIPT" ]] \
  || fail "the independent Sift load digest helper is missing"
digest_golden="$(python3 "$SIFT_LOAD_DIGEST_SCRIPT" phase \
  --name steady --duration 1 --batch-items 2)"
[[ "$digest_golden" == "201abc9eb7f9d2270d82d26bb334d11d0a7f435effe582d1474f8e59eebec57e" ]] \
  || fail "the independent Sift load digest helper changed its golden result"
present "the Sift verifier lost the real 180-day rollover" \
  'retention-rollover' "$SIFT_VERIFY_SCRIPT"
present "the Sift verifier no longer waits for bounded retention completion" \
  '.storage.archive.retention_scan_pending == false' "$SIFT_VERIFY_SCRIPT"
present "the Sift verifier lost the active receipt after telemetry expiration" \
  'active six-hour receipt did not win over telemetry retention' "$SIFT_VERIFY_SCRIPT"
present "the Sift verifier lost its MCP SSE decoder" \
  'extract_sse_json() {' "$SIFT_VERIFY_SCRIPT"
present "MCP initialize again sends raw SSE to jq" \
  'extract_sse_json "$init_sse" "$init_body"' "$SIFT_VERIFY_SCRIPT"
present "MCP tools/list again sends raw SSE to jq" \
  'extract_sse_json "$list_sse" "$list_body"' "$SIFT_VERIFY_SCRIPT"
present "nested load evidence directories are no longer prepared" \
  'mkdir -p "$(dirname "$output")"' "$SIFT_VERIFY_SCRIPT"
present "large Sift load ConfigMaps no longer use create" \
  'kubectl create -f "$EVIDENCE_DIR/load/${phase}/${signal}/configmap.yaml"' "$SIFT_VERIFY_SCRIPT"
absent "large Sift load ConfigMaps still use apply and overflow the annotation limit" \
  'kubectl apply -f "$EVIDENCE_DIR/load/${phase}/${signal}/configmap.yaml"' "$SIFT_VERIFY_SCRIPT"
present "Sift deployment no longer requires GKE FQDN policy enforcement" \
  'fqdnnetworkpolicies.networking.gke.io' "$DEPLOY_SCRIPT"
present "Sift verifier lost the operator-managed auth binding proof" \
  'auth-delegator-binding.json' "$SIFT_VERIFY_SCRIPT"
[[ -f "$SIFT_AUTH_DELEGATOR_FILTER" ]] \
  || fail "Sift auth-delegator contract filter is missing"
auth_binding="$(jq -n '{
  roleRef:{
    apiGroup:"rbac.authorization.k8s.io",
    kind:"ClusterRole",
    name:"system:auth-delegator"
  },
  subjects:[
    {kind:"ServiceAccount", name:"sift", namespace:"sift"},
    {kind:"ServiceAccount", name:"sift-store", namespace:"sift"}
  ],
  metadata:{labels:{
    "app.kubernetes.io/name":"sift",
    "app.kubernetes.io/instance":"sift",
    "sift.axiom.dev/owner-namespace":"sift",
    "service-k8s.axiom.dev/owner-uid":"owner-uid"
  }}
}')"
jq -e --arg namespace sift --arg instance sift \
  -f "$SIFT_AUTH_DELEGATOR_FILTER" >/dev/null <<<"$auth_binding" \
  || fail "Sift auth-delegator contract rejected the exact runtime and store subjects"
missing_store="$(jq '.subjects = [.subjects[0]]' <<<"$auth_binding")"
if jq -e --arg namespace sift --arg instance sift \
  -f "$SIFT_AUTH_DELEGATOR_FILTER" >/dev/null <<<"$missing_store"; then
  fail "Sift auth-delegator contract accepted a binding without the store subject"
fi
extra_backup="$(jq '.subjects += [{kind:"ServiceAccount", name:"sift-backup", namespace:"sift"}]' \
  <<<"$auth_binding")"
if jq -e --arg namespace sift --arg instance sift \
  -f "$SIFT_AUTH_DELEGATOR_FILTER" >/dev/null <<<"$extra_backup"; then
  fail "Sift auth-delegator contract accepted an extra backup subject"
fi
wrong_namespace="$(jq '.subjects[1].namespace = "other"' <<<"$auth_binding")"
if jq -e --arg namespace sift --arg instance sift \
  -f "$SIFT_AUTH_DELEGATOR_FILTER" >/dev/null <<<"$wrong_namespace"; then
  fail "Sift auth-delegator contract accepted the store subject from another namespace"
fi
[[ -f "$SIFT_ARCHIVE_FQDN_FILTER" ]] \
  || fail "Sift archive FQDN policy contract filter is missing"
archive_fqdn="$(jq -n '{items:[{
  metadata:{
    name:"sift-store-google-apis",
    namespace:"sift",
    labels:{
      "app.kubernetes.io/name":"sift",
      "app.kubernetes.io/instance":"sift",
      "app.kubernetes.io/component":"store"
    }
  },
  spec:{
    podSelector:{matchLabels:{
      "app.kubernetes.io/name":"sift",
      "app.kubernetes.io/instance":"sift",
      "app.kubernetes.io/component":"store",
      "sift.axiom.dev/role":"store"
    }},
    egress:[{
      matches:[{name:"storage.googleapis.com"}],
      ports:[{port:443,protocol:"TCP"}]
    }]
  }
}]}')"
jq -e --arg namespace sift --arg instance sift \
  -f "$SIFT_ARCHIVE_FQDN_FILTER" >/dev/null <<<"$archive_fqdn" \
  || fail "Sift archive FQDN contract rejected the exact store policy"
extra_backup_policy="$(jq '.items += [{metadata:{name:"sift-backup-google-apis"},spec:{}}]' \
  <<<"$archive_fqdn")"
if jq -e --arg namespace sift --arg instance sift \
  -f "$SIFT_ARCHIVE_FQDN_FILTER" >/dev/null <<<"$extra_backup_policy"; then
  fail "Sift archive FQDN contract accepted a policy for a disabled backup role"
fi
wrong_fqdn="$(jq '.items[0].spec.egress[0].matches[0].name = "*.googleapis.com"' \
  <<<"$archive_fqdn")"
if jq -e --arg namespace sift --arg instance sift \
  -f "$SIFT_ARCHIVE_FQDN_FILTER" >/dev/null <<<"$wrong_fqdn"; then
  fail "Sift archive FQDN contract accepted broad Google API egress"
fi
wrong_selector="$(jq '.items[0].spec.podSelector.matchLabels["app.kubernetes.io/instance"] = "other"' \
  <<<"$archive_fqdn")"
if jq -e --arg namespace sift --arg instance sift \
  -f "$SIFT_ARCHIVE_FQDN_FILTER" >/dev/null <<<"$wrong_selector"; then
  fail "Sift archive FQDN contract accepted another Sift instance"
fi
present "Sift verifier lost instance-scoped FQDN policy proof" \
  'fqdn-network-policies.json' "$SIFT_VERIFY_SCRIPT"
rg -q '^\s*datapath_provider\s*=\s*"ADVANCED_DATAPATH"\s*$' "$CLUSTER_TF" \
  || fail "persistent cluster lost Dataplane V2"
rg -q '^\s*enable_fqdn_network_policy\s*=\s*true\s*$' "$CLUSTER_TF" \
  || fail "persistent cluster lost FQDN Network Policy"
present "cluster reuse no longer rejects an incompatible dataplane" \
  'requires Dataplane V2 and FQDN Network Policy' "$BOOTSTRAP_SCRIPT"
present "cluster reuse reads Dataplane V2 from GKE networkConfig" \
  '.networkConfig.datapathProvider // ""' "$BOOTSTRAP_SCRIPT"
present "cluster reuse reads FQDN policy from GKE networkConfig" \
  '.networkConfig.enableFqdnNetworkPolicy // false' "$BOOTSTRAP_SCRIPT"

# --- phase ordering ---------------------------------------------------------
# Lumen's bundle must be materialized before the Sift branch consumes the
# shared manifest tree; a reordering here silently renders Sift against a
# half-built directory.
lumen_bundle_line="$(line_of 'kubectl kustomize "$MANIFEST_DIR/lumen/operator"' "$RENDER_SCRIPT")"
sift_manifest_line="$(line_of 'cat > "$MANIFEST_DIR/sift/operator/kustomization.yaml"' "$RENDER_SCRIPT")"
[[ "$lumen_bundle_line" =~ ^[0-9]+$ && "$sift_manifest_line" =~ ^[0-9]+$ ]] \
  || fail "could not locate the lumen/sift render phases"
(( lumen_bundle_line < sift_manifest_line )) \
  || fail "lumen must bundle before the sift branch (lumen@$lumen_bundle_line, sift@$sift_manifest_line)"

# --- the control-plane observability leg stays lumen-gated (#2621) ----------
# sift and tape render no metrics Service, so an ungated assertion would fail
# their cells on an endpoint that was never supposed to exist.
present "the observability leg lost its app guard" 'if [[ "$app" == "lumen" ]]; then' "$CELL_SCRIPT"
present "the leader gauge is no longer cross-checked against the lease" \
  'require_leader_gauge_agrees' "$CELL_SCRIPT"
# The port counter must stay file-backed: callers read these helpers through
# `$(...)`, so a plain variable increment is discarded and every scrape reuses
# one port -- two pods then return byte-identical metrics.
present "the metrics port counter stopped being file-backed" \
  'printf '"'"'%s\n'"'"' "$p" > "$metrics_port_state"' "$CELL_SCRIPT"

# --- bootstrap-cluster.sh's stdout is a one-line contract ------------------
# Its two branches are asymmetric: reuse prints the name and returns, create
# runs terraform first. When terraform's chatter went to stdout, the create
# branch wrote ~19KB of plan output into the file the caller asserts on, and
# the run died mute AFTER paying for the cluster. Every earlier run reused an
# existing cluster, so the create branch had never been exercised -- which is
# exactly the shape of bug a static oracle has to hold, because reproducing it
# costs ten minutes of GKE.
bootstrap_terraform_lines="$(rg -c -F 'terraform \' "$BOOTSTRAP_SCRIPT" || echo 0)"
(( bootstrap_terraform_lines == 2 )) \
  || fail "expected 2 terraform invocations in bootstrap-cluster.sh, found $bootstrap_terraform_lines"
redirected="$(rg -c -F '>&2' "$BOOTSTRAP_SCRIPT" || echo 0)"
(( redirected >= 2 )) \
  || fail "bootstrap-cluster.sh must send terraform output to stderr; only $redirected redirect(s) found — its stdout is the cluster name and nothing else"
present "the cluster-name check went back to a mute bare test" \
  "bootstrap-cluster.sh must emit exactly" "$RUN_SCRIPT"
absent "the cluster-name check is back to inspecting only line 1" \
  "test \"\$(sed -n '1p' \"\$EVIDENCE_DIR/persistent-cluster-name.txt\")\"" "$RUN_SCRIPT"

present "the reuse branch stopped warning about data-plane pool drift" \
  "node pool; the spec.placement leg will fail" "$BOOTSTRAP_SCRIPT"

# --- every namespace the run can create is swept, and checked (#2462) --------
# The fleet leg materializes data planes into namespaces of its own and tears
# them down itself on the happy path. A leg that fails midway does not, and
# this cluster is persistent -- so a leaked StatefulSet PVC is a Persistent
# Disk that bills indefinitely with nothing left pointing at it. Cleanup must
# name those namespaces even though a passing run never needs it to, and the
# no-leftovers gate must look for them, or the leak is invisible by design.
for fleet_ns in lumen-fleet-a lumen-fleet-b; do
  present "cleanup no longer sweeps the $fleet_ns data-plane namespace" \
    "$fleet_ns" "$CLEANUP_SCRIPT"
  present "the no-leftovers gate stopped checking $fleet_ns" \
    "$fleet_ns" "$VERIFY_CLEAN_SCRIPT"
done
present "cleanup no longer removes the cluster-scoped LumenFleet CRD" \
  "lumenfleets.lumen.dev" "$CLEANUP_SCRIPT"
present "the no-leftovers gate stopped checking the LumenFleet CRD" \
  "lumenfleets.lumen.dev" "$VERIFY_CLEAN_SCRIPT"
# Ordering, not just presence: the fleet controller reconciles cluster-wide, so
# its CRD must go before its target namespaces start terminating. Reversed, a
# reconcile pass landing between two deletes re-materializes a Lumen into a
# namespace on its way out and the gate trips on what cleanup just removed.
fleet_crd_line="$(line_of 'kubectl delete customresourcedefinition lumenfleets.lumen.dev' "$CLEANUP_SCRIPT")"
ns_delete_line="$(line_of 'kubectl delete namespace "$namespace" --ignore-not-found' "$CLEANUP_SCRIPT")"
[[ "$fleet_crd_line" =~ ^[0-9]+$ && "$ns_delete_line" =~ ^[0-9]+$ ]] \
  || fail "could not locate cleanup's fleet-CRD and namespace deletes"
(( fleet_crd_line < ns_delete_line )) \
  || fail "the LumenFleet CRD must be deleted before the namespace sweep (crd@$fleet_crd_line, namespaces@$ns_delete_line)"

# A Sift CR owns cluster-scoped children through an operator finalizer. Delete
# that CR while the operator namespace is still live. Otherwise the namespace
# and CRD stay Terminating with nobody left to remove the finalizer.
sift_cr_delete_line="$(line_of 'kubectl delete sift.sift.axiom.dev "$name" --namespace "$namespace"' "$CLEANUP_SCRIPT")"
[[ "$sift_cr_delete_line" =~ ^[0-9]+$ ]] \
  || fail "could not locate cleanup's ordered Sift CR delete"
(( sift_cr_delete_line < ns_delete_line )) \
  || fail "Sift CRs must be deleted before the namespace sweep (cr@$sift_cr_delete_line, namespaces@$ns_delete_line)"
present "cleanup lost the orphaned Sift finalizer fallback" \
  'patch sift.sift.axiom.dev "$name" --namespace "$namespace"' "$CLEANUP_SCRIPT"

present "the lumen-sift pre-flight list names lumen-auth-client" \
  'mode_namespaces=(lumen lumen-system sift sift-system lumen-auth-client)' "$RUN_SCRIPT"

if [[ "${SIFT_ORACLE_MUTATION_CHILD:-0}" != "1" ]]; then
  mutation_dir="$(mktemp -d "${TMPDIR:-/tmp}/sift-acceptance-oracle.XXXXXX")"
  cleanup_mutations() {
    find "$mutation_dir" -type f -delete
    find "$mutation_dir" -depth -type d -empty -delete
  }
  trap cleanup_mutations EXIT INT TERM

  mutation_must_fail() {
    local label="$1"
    local fixture="$2"
    if SIFT_ORACLE_MUTATION_CHILD=1 SIFT_VERIFY_SCRIPT_OVERRIDE="$fixture" \
      bash "$0" >/dev/null 2>&1; then
      fail "the static oracle accepted mutation: $label"
    fi
  }

  quoted_load="$mutation_dir/quoted-load.sh"
  sed 's/^LOAD_SECONDS=1800$/LOAD_SECONDS=1799/' \
    "$SIFT_VERIFY_SCRIPT" > "$quoted_load"
  printf '%s\n' "echo 'LOAD_SECONDS=1800' >/dev/null" >> "$quoted_load"
  mutation_must_fail "quoted prose replaced the 30-minute load" "$quoted_load"

  divergent_outage="$mutation_dir/divergent-outage.sh"
  sed 's/^verify_outage_quorum_recovery /skip_outage_quorum_recovery /' \
    "$SIFT_VERIFY_SCRIPT" > "$divergent_outage"
  printf '%s\n' "echo 'verify_outage_quorum_recovery \"\$archive_leader\" \\' >/dev/null" \
    >> "$divergent_outage"
  mutation_must_fail "quoted prose hid a divergent outage invocation" "$divergent_outage"

  missing_failover_call="$mutation_dir/missing-failover-call.sh"
  awk '
    /^assert_failover_restart_evidence \\$/ { skip=3; next }
    skip > 0 { skip--; next }
    { print }
  ' "$SIFT_VERIFY_SCRIPT" > "$missing_failover_call"
  mutation_must_fail "the failover pod comparison was defined but never called" \
    "$missing_failover_call"

  mutable_pin="$mutation_dir/mutable-pin.sh"
  sed 's/== \*@sha256:\*/== *:*/' \
    "$SIFT_VERIFY_SCRIPT" > "$mutable_pin"
  rg -F '[[ "$SIFT_IMAGE" == *:* ]]' "$mutable_pin" >/dev/null \
    || fail "could not build the mutable-image mutation"
  printf '%s\n' "echo '[[ \"\$SIFT_IMAGE\" == *@sha256:* ]]' >/dev/null" >> "$mutable_pin"
  mutation_must_fail "quoted prose hid a mutable Sift image" "$mutable_pin"

  early_acceptance="$mutation_dir/early-acceptance.sh"
  cp "$SIFT_VERIFY_SCRIPT" "$early_acceptance"
  printf '%s\n' 'cp "$EVIDENCE_DIR/sift-mvp-verification.json" "$EVIDENCE_DIR/acceptance.json"' \
    >> "$early_acceptance"
  mutation_must_fail "the verifier wrote acceptance.json before cleanup" "$early_acceptance"
fi

echo "acceptance-mode oracle: ok"
# HANDWRITE-END
