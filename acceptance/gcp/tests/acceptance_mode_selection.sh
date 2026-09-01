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
SIFT_VERIFY_SCRIPT="$ACCEPTANCE_ROOT/scripts/verify-sift-mvp.sh"
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

absent() { # absent <label> <pattern> <file>
  ! uncommented "$2" "$3" || fail "$1 (unexpectedly still live in ${3##*/}: $2)"
}

line_of() { # line_of <pattern> <file>
  rg -n -F -- "$1" "$2" | head -1 | cut -d: -f1
}

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
present "the false-green sentinel is gone"     'run_completed=1'                    "$RUN_SCRIPT"
present "cleanup no longer refuses ec=0 without the sentinel" 'run aborted before completion' "$RUN_SCRIPT"
present "the backup service account is no longer swept" 'wait_for_empty "backup service account"' "$VERIFY_CLEAN_SCRIPT"

# --- Sift MVP is independent, bounded, and evidence-complete --------------
present "sift-only mode does not invoke its verifier" 'verify-sift-mvp.sh"' "$RUN_SCRIPT"
present "sift-only mode lost the 90-minute cap" 'sift_cloud_cap=5400' "$RUN_SCRIPT"
present "sift-only mode no longer requires immutable Rig" 'INPUT_RIG_IMAGE' "$RUN_SCRIPT"
present "Terraform lost the sift-only enum" '"sift"' "$ENV_VARIABLES"
present "Terraform lost the run-scoped Sift node pool" 'resource "google_container_node_pool" "sift_mvp"' "$ENV_GKE"
rg -q '^\s*disk_type\s*=\s*"pd-standard"\s*$' "$ENV_GKE" \
  || fail "the Sift node pool boot disks consume the PVC SSD quota"
present "the Sift verifier lost its 30-minute load" 'LOAD_SECONDS=1800' "$SIFT_VERIFY_SCRIPT"
present "the Sift verifier lost the 10k/s rate" 'ITEMS_PER_SECOND=10000' "$SIFT_VERIFY_SCRIPT"
present "the Sift verifier lost its 18M exact count" 'EXPECTED_ITEMS=18000000' "$SIFT_VERIFY_SCRIPT"
present "the Sift verifier lost GCS outage testing" 'archive-iam-disabled' "$SIFT_VERIFY_SCRIPT"
present "the Sift verifier lost fresh-PVC restore" 'fresh-pvc-restore' "$SIFT_VERIFY_SCRIPT"
present "the Sift verifier no longer emits acceptance.json" 'acceptance.json' "$SIFT_VERIFY_SCRIPT"
present "the Sift verifier lost its authenticated non-2xx status helper" \
  'auth_curl_status() {' "$SIFT_VERIFY_SCRIPT"
present "Remote Write 2.0 rejection again aborts before its 415 assertion" \
  'remote_write_v2_status="$(auth_curl_status ' "$SIFT_VERIFY_SCRIPT"
present "the Sift verifier lost its MCP SSE decoder" \
  'extract_sse_json() {' "$SIFT_VERIFY_SCRIPT"
present "MCP initialize again sends raw SSE to jq" \
  'extract_sse_json "$init_sse" "$init_body"' "$SIFT_VERIFY_SCRIPT"
present "MCP tools/list again sends raw SSE to jq" \
  'extract_sse_json "$list_sse" "$list_body"' "$SIFT_VERIFY_SCRIPT"
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

echo "acceptance-mode oracle: ok"
# HANDWRITE-END
