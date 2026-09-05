//! Execute the real cloud-operation shell paths with local API doubles.
//! No test command can reach GCP or Kubernetes.
use serde_json::{json, Value};
use std::{fs, os::unix::fs::PermissionsExt, path::PathBuf, process::Command};

const RUN: &str = "0905071629";
const POOL: &str = "axo-0905071629-sift";
const NODE: &str = "gke-sift-run-node-a";
const BASE: &str = "https://www.googleapis.com/compute/v1/projects/test-project/zones/asia-east1-a";

const DOUBLE: &str = r##"#!/usr/bin/env bash
set -euo pipefail
tool="${0##*/}"
printf '%s %s\n' "$tool" "$*" >> "$MOCK_ROOT/calls"
reply() {
  local name="$1"
  [[ "${MOCK_FAIL:-}" != "$name" ]] || exit 17
  if [[ -f "$MOCK_ROOT/after" ]]; then
    [[ "${MOCK_FAIL_AFTER:-}" != "$name" ]] || exit 18
    cat "$MOCK_ROOT/${name}-after.json"
  else
    cat "$MOCK_ROOT/${name}.json"
  fi
}
case "$tool:$*" in
  'gcloud:container clusters describe '*) reply cluster ;;
  'gcloud:container node-pools describe '*)
    [[ "$4" == 'axo-0905071629-sift' && " $* " == *' --cluster=shared-cluster '* ]] || exit 91
    [[ " $* " == *' --project=test-project '* && " $* " == *' --zone=asia-east1-a '* ]] || exit 92
    reply pool ;;
  'gcloud:compute instance-groups managed list-instances '*)
    [[ "$5" == 'run-group' && " $* " == *' --project=test-project '* ]] || exit 93
    [[ " $* " == *' --zone=asia-east1-a '* ]] || exit 94
    reply members ;;
  'gcloud:compute instances describe '*) reply vm ;;
  'gcloud:compute instances stop '*)
    [[ "$4" == 'gke-sift-run-node-a' && " $* " == *' --project=test-project '* ]] || exit 95
    [[ " $* " == *' --zone=asia-east1-a '* ]] || exit 96
    printf 'stop %s\n' "$4" >> "$MOCK_ROOT/actions" ;;
  'kubectl:-n sift get pod/sift-store-0 '*) printf '%s\n' 'gke-sift-run-node-a' ;;
  'kubectl:get node/'*|'kubectl:get node '*|'kubectl:get nodes '*) reply node ;;
  'kubectl:-n sift get pods '*) printf '%s\n' '{"items":[]}' ;;
  'kubectl:cordon '*|'kubectl:uncordon '*) printf '%s\n' "$*" >> "$MOCK_ROOT/actions" ;;
  'kubectl:patch node/'*)
    [[ "${MOCK_FAIL:-}" != 'node_patch' ]] || exit 17
    [[ "${MOCK_REPLACE_ON_PATCH:-}" != '1' ]] || exit 18
    fixture="$MOCK_ROOT/node.json"
    [[ ! -f "$MOCK_ROOT/after" ]] || fixture="$MOCK_ROOT/node-after.json"
    if [[ "${MOCK_PATCH_RUN_DRIFT:-}" == '1' ]]; then
      jq '.metadata.labels["axiom-run-id"] = "another-run"' "$fixture" > "$MOCK_ROOT/live-node.json"
      fixture="$MOCK_ROOT/live-node.json"
    fi
    jq -e --slurpfile node "$fixture" '
      any(.[]; .op == "test" and .path == "/metadata/uid" and .value == $node[0].metadata.uid)
      and any(.[]; .op == "test" and .path == "/spec/providerID" and .value == $node[0].spec.providerID)
      and any(.[]; .op == "test" and .path == "/metadata/labels/cloud.google.com~1gke-nodepool" and .value == $node[0].metadata.labels["cloud.google.com/gke-nodepool"])
      and any(.[]; .op == "test" and .path == "/metadata/labels/axiom-run-id" and .value == $node[0].metadata.labels["axiom-run-id"])
      and ([.[] | select(.path == "/spec/unschedulable")] | length == 1)
      and any(.[]; .op == "add" and .path == "/spec/unschedulable" and (.value | type == "boolean"))
    ' <<< "$5" >/dev/null || exit 19
    action="$(jq -r 'first(.[] | select(.path == "/spec/unschedulable") | if .value then "cordon" else "uncordon" end)' <<< "$5")"
    [[ "$action" == 'cordon' || "$action" == 'uncordon' ]] || exit 20
    printf '%s gke-sift-run-node-a\n' "$action" >> "$MOCK_ROOT/actions"
    jq --argjson value "$(jq '.[] | select(.path == "/spec/unschedulable") | .value' <<< "$5")" \
      '.spec.unschedulable = $value' "$fixture" ;;
  'terraform:'*) printf '%s\n' "$*" >> "$MOCK_ROOT/terraform-calls" ;;
  'sleep:30') touch "$MOCK_ROOT/after" ;;
  'sleep:'*) ;;
  *) printf 'unexpected command: %s %s\n' "$tool" "$*" >&2; exit 99 ;;
esac
"##;

struct Harness {
    temp: tempfile::TempDir,
    root: PathBuf,
}

impl Harness {
    fn new() -> Self {
        let temp = tempfile::tempdir().unwrap();
        for name in ["gcloud", "kubectl", "terraform", "sleep"] {
            let path = temp.path().join(name);
            fs::write(&path, DOUBLE).unwrap();
            fs::set_permissions(&path, fs::Permissions::from_mode(0o755)).unwrap();
        }
        fs::create_dir_all(temp.path().join("evidence/kubernetes")).unwrap();
        let harness = Self {
            temp,
            root: PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../.."),
        };
        for (name, value) in [
            (
                "cluster",
                json!({"name":"shared-cluster","location":"asia-east1-a",
                "status":"RUNNING","autopilot":{"enabled":false},
                "networkConfig":{"datapathProvider":"ADVANCED_DATAPATH","enableFqdnNetworkPolicy":true},
                "nodePools":[{"name":"acceptance-pool","autoscaling":{"maxNodeCount":3}},
                             {"name":"data-plane-pool"}]}),
            ),
            (
                "node",
                json!({"metadata":{"name":NODE,"uid":"node-generation-1","labels":{
                "axiom-run-id":RUN,"cloud.google.com/gke-nodepool":POOL}},
                "spec":{"providerID":format!("gce://test-project/asia-east1-a/{NODE}")}}),
            ),
            (
                "pool",
                json!({"name":POOL,"initialNodeCount":3,"status":"RUNNING",
                "config":{"machineType":"e2-standard-4","labels":{"axiom-run-id":RUN}},
                "management":{"autoRepair":true},
                "instanceGroupUrls":[format!("{BASE}/instanceGroupManagers/run-group")]}),
            ),
            (
                "vm",
                json!({"name":NODE,"id":"12345","status":"RUNNING",
                "zone":"https://www.googleapis.com/compute/v1/projects/test-project/zones/asia-east1-a",
                "selfLink":format!("{BASE}/instances/{NODE}")}),
            ),
            (
                "members",
                json!([{"instance":format!("{BASE}/instances/{NODE}"),
                "id":"12345","instanceStatus":"RUNNING","currentAction":"NONE"}]),
            ),
        ] {
            harness.put(name, &value);
            harness.put(&format!("{name}-after"), &value);
        }
        harness
    }

    fn put(&self, name: &str, value: &Value) {
        fs::write(
            self.temp.path().join(format!("{name}.json")),
            value.to_string(),
        )
        .unwrap();
    }

    fn mutate(&self, name: &str, pointer: &str, value: Value) {
        let path = self.temp.path().join(format!("{name}.json"));
        let mut data: Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
        *data.pointer_mut(pointer).unwrap() = value;
        self.put(name, &data);
    }

    fn read(&self, name: &str) -> String {
        fs::read_to_string(self.temp.path().join(name)).unwrap_or_default()
    }

    fn command(&self) -> Command {
        let mut command = Command::new("bash");
        command
            .current_dir(&self.root)
            .env(
                "PATH",
                format!(
                    "{}:{}",
                    self.temp.path().display(),
                    std::env::var("PATH").unwrap()
                ),
            )
            .env("MOCK_ROOT", self.temp.path())
            .env("PROJECT_ID", "test-project")
            .env("REGION", "asia-east1")
            .env("GKE_ZONE", "asia-east1-a")
            .env("PERSISTENT_CLUSTER_NAME", "shared-cluster")
            .env("GKE_CLUSTER_NAME", "shared-cluster")
            .env("RUN_ID", RUN)
            .env("SIFT_NODE_POOL", POOL)
            .env("NAMESPACE", "sift")
            .env("FAILOVER_SECONDS", "300")
            .env("EVIDENCE_DIR", self.temp.path().join("evidence"))
            .env(
                "CLUSTER_TF_DATA_DIR",
                self.temp.path().join("terraform-data"),
            )
            .env("SCRIPT_DIR", self.root.join("acceptance/gcp/scripts"));
        command
    }

    fn failover_source(&self) -> String {
        let source =
            fs::read_to_string(self.root.join("acceptance/gcp/scripts/verify-sift-mvp.sh"))
                .unwrap();
        let start = source
            .find("failover_leader=\"$(wait_store_leader)\"")
            .unwrap();
        let end = source[start..].find("new_leader_deadline=").unwrap() + start;
        let setup = "set -euo pipefail\nwait_store_leader() { if [[ ${MOCK_CHANGE_LEADER:-0} == 1 && -f $MOCK_ROOT/after ]]; then echo 1; else echo 0; fi; }\ndie() { echo \"$*\" >&2; exit 1; }\nwait_role_ready() { :; }\nstop_forwards() { :; }\nstart_gateway_forward() { :; }\nstart_store_forwards() { :; }\nrefresh_token() { :; }\nsnapshot_restarts() { :; }\nstart_load_phase() { :; }\n";
        format!("{setup}{}", &source[start..end])
    }

    fn failover(&self, fail: &str) -> std::process::Output {
        self.command()
            .args(["-c", &self.failover_source()])
            .env("MOCK_FAIL", fail)
            .env("MOCK_FAIL_AFTER", fail.strip_prefix("after:").unwrap_or(""))
            .env(
                "MOCK_PATCH_RUN_DRIFT",
                if fail == "changed_run_during_patch" {
                    "1"
                } else {
                    "0"
                },
            )
            .env(
                "MOCK_CHANGE_LEADER",
                if fail == "changed_leader_after_wait" {
                    "1"
                } else {
                    "0"
                },
            )
            .env(
                "MOCK_REPLACE_ON_PATCH",
                if fail == "replaced_node_during_patch" {
                    "1"
                } else {
                    "0"
                },
            )
            .output()
            .unwrap()
    }
}

#[test]
fn existing_cluster_failure_never_runs_terraform() {
    let h = Harness::new();
    let output = h
        .command()
        .args([
            "acceptance/gcp/scripts/bootstrap-cluster.sh",
            "--existing-only",
        ])
        .env("MOCK_FAIL", "cluster")
        .output()
        .unwrap();
    assert!(
        !output.status.success(),
        "an unreadable cluster must fail closed"
    );
    assert!(
        h.read("terraform-calls").is_empty(),
        "must not enter cluster Terraform"
    );
}

#[test]
fn explicit_bootstrap_keeps_its_creation_contract() {
    let h = Harness::new();
    let output = h
        .command()
        .arg("acceptance/gcp/scripts/bootstrap-cluster.sh")
        .env("MOCK_FAIL", "cluster")
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&output.stdout), "shared-cluster\n");
    assert_eq!(h.read("terraform-calls").lines().count(), 2);
}

#[test]
fn valid_existing_cluster_is_read_only_and_has_exact_stdout() {
    let h = Harness::new();
    let output = h
        .command()
        .args([
            "acceptance/gcp/scripts/bootstrap-cluster.sh",
            "--existing-only",
        ])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&output.stdout), "shared-cluster\n");
    assert!(h.read("terraform-calls").is_empty());
}

#[test]
fn existing_cluster_rejects_wrong_identity_and_autopilot() {
    for (pointer, value) in [
        ("/name", json!("another-cluster")),
        ("/location", json!("another-zone")),
        ("/status", json!("ERROR")),
        ("/autopilot/enabled", json!(true)),
    ] {
        let h = Harness::new();
        h.mutate("cluster", pointer, value);
        let output = h
            .command()
            .args([
                "acceptance/gcp/scripts/bootstrap-cluster.sh",
                "--existing-only",
            ])
            .output()
            .unwrap();
        assert!(!output.status.success(), "accepted bad cluster {pointer}");
        assert!(h.read("terraform-calls").is_empty());
    }
}

#[test]
fn run_reuse_path_is_existing_only_even_when_later_describe_fails() {
    let h = Harness::new();
    let source = fs::read_to_string(h.root.join("acceptance/gcp/scripts/run.sh")).unwrap();
    let start = source.find("PROJECT_ID=\"$PROJECT_ID\" REGION=\"$REGION\" GKE_ZONE=\"$GKE_ZONE\" \\\n  PERSISTENT_CLUSTER_NAME=\"$PERSISTENT_CLUSTER_NAME\" \\\n  \"$SCRIPT_DIR/bootstrap-cluster.sh\"").unwrap();
    let end = source[start..].find("\njq -n \\\n").unwrap() + start;
    let output = h
        .command()
        .args(["-c", &format!("set -euo pipefail\n{}", &source[start..end])])
        .env("MOCK_FAIL", "cluster")
        .output()
        .unwrap();
    assert!(
        !output.status.success(),
        "run must not fall back to creating the shared cluster"
    );
    assert!(h.read("terraform-calls").is_empty());
}

#[test]
fn exact_run_vm_is_checked_twice_before_one_stop() {
    let h = Harness::new();
    let output = h.failover("");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(h.read("actions"), format!("cordon {NODE}\nstop {NODE}\n"));
    let calls = h.read("calls");
    assert_eq!(
        calls
            .matches("compute instance-groups managed list-instances")
            .count(),
        2
    );
    assert_eq!(calls.matches("container node-pools describe").count(), 2);
}

#[test]
fn forged_node_labels_and_foreign_group_members_cannot_cordon_or_stop() {
    for (file, pointer, value) in [
        ("node", "/metadata/labels/cloud.google.com~1gke-nodepool", json!("another-pool")),
        ("node", "/spec/providerID", json!(format!("gce://other-project/asia-east1-a/{NODE}"))),
        ("node", "/spec/providerID", json!(format!("gce://test-project/other-zone/{NODE}"))),
        ("node", "/metadata/uid", json!("")),
        ("pool", "/name", json!("another-pool")),
        ("pool", "/instanceGroupUrls", json!([])),
        ("pool", "/instanceGroupUrls/0", json!("https://www.googleapis.com/compute/v1/projects/other-project/zones/asia-east1-a/instanceGroupManagers/foreign-group")),
        ("members", "/0/instance", json!(format!("{BASE}/instances/another-node"))),
        ("members", "/0/id", json!("54321")),
        ("members", "/0/currentAction", json!("ABANDONING")),
        ("vm", "/selfLink", json!(format!("{BASE}/instances/another-node"))),
    ] {
        let h = Harness::new();
        h.mutate(file, pointer, value);
        let output = h.failover("");
        assert!(!output.status.success(), "accepted {file}:{pointer}");
        assert!(h.read("actions").is_empty(), "mutated before rejecting {file}:{pointer}");
    }
}

#[test]
fn unreadable_identity_never_mutates_a_node_or_vm() {
    for fail in [
        "node",
        "pool",
        "members",
        "vm",
        "node_patch",
        "replaced_node_during_patch",
        "changed_run_during_patch",
    ] {
        let h = Harness::new();
        let output = h.failover(fail);
        assert!(!output.status.success(), "accepted unreadable {fail}");
        assert!(
            h.read("actions").is_empty(),
            "mutated with unreadable {fail}"
        );
    }
}

#[test]
fn a_valid_replacement_member_is_still_not_the_cordoned_instance() {
    let h = Harness::new();
    h.mutate("vm-after", "/id", json!("99999"));
    h.mutate("members-after", "/0/id", json!("99999"));
    let output = h.failover("");
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("target changed after cordon"));
    assert_eq!(h.read("actions"), format!("cordon {NODE}\n"));
}

#[test]
fn second_membership_read_failure_does_not_stop_the_vm() {
    let h = Harness::new();
    let output = h.failover("after:members");
    assert!(!output.status.success());
    assert_eq!(h.read("actions"), format!("cordon {NODE}\n"));
    assert_eq!(
        h.read("calls")
            .matches("compute instance-groups managed list-instances")
            .count(),
        2
    );
}

#[test]
fn leader_change_during_the_load_wait_prevents_stop() {
    let h = Harness::new();
    let output = h.failover("changed_leader_after_wait");
    assert!(!output.status.success());
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("leadership changed before the VM stop")
    );
    assert_eq!(h.read("actions"), format!("cordon {NODE}\n"));
}

#[test]
fn removing_the_final_leader_check_reaches_an_invalid_failover_stop() {
    let h = Harness::new();
    let source = h.failover_source();
    let guard = "[[ \"$(wait_store_leader)\" == \"$failover_leader\" ]] \\\n  || die \"Raft leadership changed before the VM stop\"";
    assert_eq!(source.matches(guard).count(), 1);
    let output = h
        .command()
        .args(["-c", &source.replacen(guard, "", 1)])
        .env("MOCK_CHANGE_LEADER", "1")
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(h.read("actions"), format!("cordon {NODE}\nstop {NODE}\n"));
}

#[test]
fn stop_requires_an_applied_cordon_receipt() {
    let h = Harness::new();
    let output = h
        .command()
        .args(["acceptance/gcp/scripts/sift-failover-vm.sh", "stop", NODE])
        .output()
        .unwrap();
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("no applied cordon receipt"));
    assert!(h.read("actions").is_empty());
}

#[test]
fn uncordon_can_restore_a_valid_repaired_run_member() {
    let h = Harness::new();
    h.mutate("node", "/metadata/uid", json!("repaired-node"));
    h.mutate("vm", "/id", json!("99999"));
    h.mutate("members", "/0/id", json!("99999"));
    let output = h
        .command()
        .args([
            "acceptance/gcp/scripts/sift-failover-vm.sh",
            "uncordon",
            NODE,
        ])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(h.read("actions"), format!("uncordon {NODE}\n"));
}

#[test]
fn removing_either_guard_is_detected_by_the_executed_operation_oracle() {
    for action in ["cordon", "stop"] {
        let h = Harness::new();
        if action == "cordon" {
            h.put("members", &json!([]));
        } else {
            h.put("members-after", &json!([]));
        }
        let source = h.failover_source();
        let guard = format!("\"$SCRIPT_DIR/sift-failover-vm.sh\" {action} \"$failover_node\"");
        assert_eq!(source.matches(&guard).count(), 1);
        let bypass = if action == "cordon" {
            "kubectl cordon \"$failover_node\""
        } else {
            "gcloud compute instances stop \"$failover_node\" --project=\"$PROJECT_ID\" --zone=\"$GKE_ZONE\" --quiet"
        };
        let mutation = source.replacen(&guard, bypass, 1);
        let output = h.command().args(["-c", &mutation]).output().unwrap();
        let forbidden_action = format!("{action} {NODE}\n");
        assert!(
            h.read("actions").contains(&forbidden_action),
            "mutation must reach the unsafe action: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
}

#[test]
fn removing_existing_only_from_the_real_run_call_exposes_the_unsafe_fallback() {
    let h = Harness::new();
    let source = fs::read_to_string(h.root.join("acceptance/gcp/scripts/run.sh")).unwrap();
    let invocation = source
        .lines()
        .find(|line| {
            line.trim_start()
                .starts_with("\"$SCRIPT_DIR/bootstrap-cluster.sh\"")
        })
        .unwrap();
    assert!(invocation.contains(" --existing-only"));
    let mutation = invocation.replacen(" --existing-only", "", 1);
    let output = h
        .command()
        .args(["-c", &format!("set -euo pipefail\n{mutation}\n")])
        .env("MOCK_FAIL", "cluster")
        .output()
        .unwrap();
    assert!(output.status.success());
    assert_eq!(
        h.read("terraform-calls").lines().count(),
        2,
        "mutation must expose both cluster Terraform calls"
    );
}

#[test]
fn patch_double_rejects_non_mutating_or_missing_field_incompatible_operations() {
    for op in ["test", "replace"] {
        let h = Harness::new();
        let source =
            fs::read_to_string(h.root.join("acceptance/gcp/scripts/sift-failover-vm.sh")).unwrap();
        let original = "{op:\"add\",path:\"/spec/unschedulable\",value:$value}";
        assert_eq!(source.matches(original).count(), 1);
        let mutant = source.replacen(
            original,
            &format!("{{op:\"{op}\",path:\"/spec/unschedulable\",value:$value}}"),
            1,
        );
        let path = h.temp.path().join("mutated-helper.sh");
        fs::write(&path, mutant).unwrap();
        let output = h
            .command()
            .arg(path)
            .args(["cordon", NODE])
            .output()
            .unwrap();
        assert!(!output.status.success(), "fake accepted {op} as add");
        assert!(
            h.read("calls").contains("kubectl patch node/"),
            "must reach the patch boundary"
        );
        assert!(
            h.read("actions").is_empty(),
            "fake recorded a non-mutation as cordon"
        );
    }
}

#[test]
fn uncordon_also_requires_fresh_pool_membership() {
    let h = Harness::new();
    h.put("members", &json!([]));
    let source =
        fs::read_to_string(h.root.join("acceptance/gcp/scripts/verify-sift-mvp.sh")).unwrap();
    let operation = source
        .lines()
        .find(|line| line.contains("uncordon \"$failover_node\""))
        .unwrap();
    let output = h
        .command()
        .args([
            "-c",
            &format!("set -euo pipefail\nfailover_node={NODE}\n{operation}\n"),
        ])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "recovery may skip a vanished or foreign node"
    );
    assert!(
        h.read("actions").is_empty(),
        "must not uncordon an unowned node"
    );
    assert!(h
        .read("calls")
        .contains("compute instance-groups managed list-instances"));
}

#[test]
fn replacement_or_membership_drift_after_cordon_prevents_stop() {
    for (file, pointer, value) in [
        ("node-after", "/metadata/uid", json!("replacement-node")),
        (
            "node-after",
            "/metadata/labels/cloud.google.com~1gke-nodepool",
            json!("another-pool"),
        ),
        (
            "members-after",
            "/0/instance",
            json!(format!("{BASE}/instances/another-node")),
        ),
        ("vm-after", "/id", json!("99999")),
        ("pool-after", "/instanceGroupUrls", json!([])),
    ] {
        let h = Harness::new();
        h.mutate(file, pointer, value);
        let output = h.failover("");
        assert!(!output.status.success(), "accepted late {file}:{pointer}");
        assert_eq!(
            h.read("actions"),
            format!("cordon {NODE}\n"),
            "stopped after {file}:{pointer} drift"
        );
    }
}
