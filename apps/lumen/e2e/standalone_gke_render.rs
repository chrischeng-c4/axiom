use serde_json::Value;
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

const MARKER: &[u8] = b"lumen-standalone-managed/v1\n";

fn init(out: &Path) -> Output {
    Command::new(env!("CARGO_BIN_EXE_lumen"))
        .args(["standalone", "gke", "init", "--out"])
        .arg(out)
        .output()
        .unwrap()
}

fn render(file: &Path, out: &Path) -> Output {
    Command::new(env!("CARGO_BIN_EXE_lumen"))
        .args(["standalone", "gke", "render", "--file"])
        .arg(file)
        .arg("--out")
        .arg(out)
        .output()
        .unwrap()
}

fn assert_success(output: &Output) {
    assert!(
        output.status.success(),
        "status={}\nstdout={}\nstderr={}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr),
    );
}

fn assert_failure(output: &Output) {
    assert!(
        !output.status.success(),
        "command unexpectedly succeeded: {}",
        String::from_utf8_lossy(&output.stdout),
    );
}

fn config(accounts: &[&str]) -> String {
    let accounts = accounts
        .iter()
        .map(|account| format!("  - {account}\n"))
        .collect::<String>();
    format!(
        "name: search\nnamespace: lumen\nnodePool: data-pool\ncpu: 1500m\nmemory: 4Gi\nstorageSize: 20Gi\nstorageClass: premium-rwo\nallowedServiceAccounts:\n{accounts}"
    )
}

fn write_config(root: &Path, text: &str) -> PathBuf {
    let path = root.join("lumen.yaml");
    fs::write(&path, text).unwrap();
    path
}

fn yaml(path: impl AsRef<Path>) -> Value {
    serde_yaml::from_slice(&fs::read(path).unwrap()).unwrap()
}

fn collect_tree(root: &Path) -> BTreeMap<String, Vec<u8>> {
    fn walk(base: &Path, current: &Path, out: &mut BTreeMap<String, Vec<u8>>) {
        let mut entries = fs::read_dir(current)
            .unwrap()
            .collect::<Result<Vec<_>, _>>()
            .unwrap();
        entries.sort_by_key(|entry| entry.file_name());
        for entry in entries {
            let path = entry.path();
            let relative = path
                .strip_prefix(base)
                .unwrap()
                .to_string_lossy()
                .into_owned();
            if entry.file_type().unwrap().is_dir() {
                walk(base, &path, out);
            } else {
                out.insert(relative, fs::read(path).unwrap());
            }
        }
    }

    let mut out = BTreeMap::new();
    walk(root, root, &mut out);
    out
}

fn env<'a>(stateful_set: &'a Value, name: &str) -> &'a Value {
    stateful_set["spec"]["template"]["spec"]["containers"][0]["env"]
        .as_array()
        .unwrap()
        .iter()
        .find(|entry| entry["name"] == name)
        .unwrap_or_else(|| panic!("missing env {name}"))
}

#[test]
fn gke_init_emits_exact_fill_in_contract_and_refuses_overwrite() {
    let dir = tempfile::tempdir().unwrap();
    let out = dir.path().join("nested/lumen.yaml");
    assert_success(&init(&out));
    assert_eq!(
        fs::read_to_string(&out).unwrap(),
        "name: lumen\nnamespace: lumen\nnodePool: REQUIRED\ncpu: REQUIRED\nmemory: REQUIRED\nstorageSize: 20Gi\nstorageClass: premium-rwo\nallowedServiceAccounts:\n  - namespace/name\n"
    );

    let before = fs::read(&out).unwrap();
    assert_failure(&init(&out));
    assert_eq!(fs::read(out).unwrap(), before);
}

#[test]
fn render_emits_one_durable_in_cluster_instance_and_split_rbac() {
    let dir = tempfile::tempdir().unwrap();
    let file = write_config(dir.path(), &config(&["zeta/worker", "apps/api"]));
    let out = dir.path().join("lumen-dist");
    assert_success(&render(&file, &out));

    assert_eq!(
        fs::read(out.join(".lumen-standalone-managed")).unwrap(),
        MARKER
    );
    let tree = collect_tree(&out);
    for required in [
        "storage/namespace.yaml",
        "storage/pvc.yaml",
        "storage/kustomization.yaml",
        "runtime/statefulset.yaml",
        "runtime/service.yaml",
        "runtime/serviceaccount.yaml",
        "runtime/admin-serviceaccount.yaml",
        "runtime/client-role.yaml",
        "runtime/admin-role.yaml",
        "runtime/client-rolebinding-000.yaml",
        "runtime/client-rolebinding-001.yaml",
        "runtime/admin-rolebinding.yaml",
        "runtime/clusterrolebinding.yaml",
        "runtime/networkpolicy.yaml",
        "runtime/kustomization.yaml",
    ] {
        assert!(tree.contains_key(required), "missing {required}");
    }

    let pvc = yaml(out.join("storage/pvc.yaml"));
    assert_eq!(pvc["kind"], "PersistentVolumeClaim");
    assert_eq!(pvc["metadata"]["name"], "search-data");
    assert_eq!(pvc["spec"]["storageClassName"], "premium-rwo");
    assert_eq!(pvc["spec"]["resources"]["requests"]["storage"], "20Gi");
    assert_eq!(pvc["metadata"]["labels"]["lumen.axiom.dev/profile"], "gke");
    assert_eq!(
        pvc["metadata"]["annotations"]["lumen.axiom.dev/instance-identity"],
        "lumen/search"
    );

    let stateful_set = yaml(out.join("runtime/statefulset.yaml"));
    assert_eq!(stateful_set["kind"], "StatefulSet");
    assert_eq!(stateful_set["spec"]["replicas"], 1);
    assert!(stateful_set["spec"]["volumeClaimTemplates"].is_null());
    let data_volume = stateful_set["spec"]["template"]["spec"]["volumes"]
        .as_array()
        .unwrap()
        .iter()
        .find(|volume| volume["name"] == "data")
        .expect("data volume");
    assert_eq!(
        data_volume["persistentVolumeClaim"]["claimName"],
        "search-data"
    );
    assert_eq!(
        stateful_set["spec"]["template"]["spec"]["nodeSelector"]["cloud.google.com/gke-nodepool"],
        "data-pool"
    );
    let container = &stateful_set["spec"]["template"]["spec"]["containers"][0];
    assert_eq!(container["image"], "ghcr.io/chrischeng-c4/lumen:0.4.29");
    assert_eq!(env(&stateful_set, "LUMEN_AUTH")["value"], "in-cluster");
    assert_eq!(env(&stateful_set, "LUMEN_AUTH_NAMESPACE")["value"], "lumen");
    assert!(container["env"]
        .as_array()
        .unwrap()
        .iter()
        .all(|entry| entry["name"] != "LUMEN_DATA_DIR"));
    assert_eq!(container["readinessProbe"]["timeoutSeconds"], 3);
    assert_eq!(container["livenessProbe"]["periodSeconds"], 30);
    assert_eq!(container["startupProbe"]["failureThreshold"], 120);
    assert_eq!(
        stateful_set["spec"]["template"]["metadata"]["annotations"]["prometheus.io/path"],
        "/metrics"
    );

    let service = yaml(out.join("runtime/service.yaml"));
    assert_eq!(service["spec"]["type"], "ClusterIP");
    assert!(service["spec"]["externalIPs"].is_null());
    assert!(service["spec"]["ports"][0]["nodePort"].is_null());
    assert_eq!(service["spec"]["ports"][0]["port"], 7373);
    assert_eq!(
        service["metadata"]["annotations"]["lumen.axiom.dev/instance-identity"],
        "lumen/search"
    );

    let client_role = yaml(out.join("runtime/client-role.yaml"));
    assert_eq!(
        client_role["rules"][0]["resources"],
        serde_json::json!(["lumencollections"])
    );
    assert!(!fs::read_to_string(out.join("runtime/client-role.yaml"))
        .unwrap()
        .contains("lumenadmin"));
    let admin_role = yaml(out.join("runtime/admin-role.yaml"));
    assert!(admin_role["rules"][0]["resources"]
        .as_array()
        .unwrap()
        .contains(&Value::String("lumenadmin".into())));

    let first_client = yaml(out.join("runtime/client-rolebinding-000.yaml"));
    let second_client = yaml(out.join("runtime/client-rolebinding-001.yaml"));
    assert_eq!(first_client["subjects"][0]["namespace"], "apps");
    assert_eq!(first_client["subjects"][0]["name"], "api");
    assert_eq!(second_client["subjects"][0]["namespace"], "zeta");
    assert_eq!(second_client["subjects"][0]["name"], "worker");
    let admin_binding = yaml(out.join("runtime/admin-rolebinding.yaml"));
    assert_eq!(admin_binding["subjects"][0]["name"], "search-admin");
    assert_eq!(admin_binding["subjects"][0]["namespace"], "lumen");

    let delegation = yaml(out.join("runtime/clusterrolebinding.yaml"));
    assert!(delegation["metadata"]["namespace"].is_null());
    assert_eq!(
        delegation["metadata"]["name"],
        "lumen.lumen.search.auth-delegator"
    );
    assert_eq!(delegation["subjects"][0]["name"], "search");

    let policy = yaml(out.join("runtime/networkpolicy.yaml"));
    assert_eq!(
        policy["spec"]["policyTypes"],
        serde_json::json!(["Ingress", "Egress"])
    );
    assert_eq!(policy["spec"]["ingress"][0]["ports"][0]["port"], 7373);
    assert!(!fs::read_to_string(out.join("runtime/networkpolicy.yaml"))
        .unwrap()
        .contains("7374"));
    assert_eq!(
        yaml(out.join("runtime/serviceaccount.yaml"))["automountServiceAccountToken"],
        true
    );
}

#[test]
fn render_is_byte_deterministic_and_canonicalizes_service_accounts() {
    let dir = tempfile::tempdir().unwrap();
    let first_file = dir.path().join("first.yaml");
    let second_file = dir.path().join("second.yaml");
    fs::write(&first_file, config(&["zeta/worker", "apps/api"])).unwrap();
    fs::write(&second_file, config(&["apps/api", "zeta/worker"])).unwrap();
    let first = dir.path().join("first");
    let second = dir.path().join("second");
    assert_success(&render(&first_file, &first));
    let first_bytes = collect_tree(&first);
    assert_success(&render(&first_file, &first));
    assert_eq!(collect_tree(&first), first_bytes);
    assert_success(&render(&second_file, &second));
    assert_eq!(collect_tree(&second), first_bytes);
}

#[test]
fn render_rejects_invalid_contracts_before_creating_output() {
    let cases = [
        (
            "unknown field",
            format!("{}unknown: true\n", config(&["apps/api"])),
        ),
        (
            "placeholder node pool",
            config(&["apps/api"]).replace("nodePool: data-pool", "nodePool: REQUIRED"),
        ),
        (
            "invalid node pool",
            config(&["apps/api"]).replace("nodePool: data-pool", "nodePool: Data_Pool"),
        ),
        (
            "invalid name",
            config(&["apps/api"]).replace("name: search", "name: Bad_Name"),
        ),
        (
            "invalid namespace",
            config(&["apps/api"]).replace("namespace: lumen", "namespace: bad_name"),
        ),
        (
            "invalid CPU",
            config(&["apps/api"]).replace("cpu: 1500m", "cpu: 1..2"),
        ),
        (
            "invalid memory",
            config(&["apps/api"]).replace("memory: 4Gi", "memory: -4Gi"),
        ),
        (
            "invalid storage size",
            config(&["apps/api"]).replace("storageSize: 20Gi", "storageSize: 0Gi"),
        ),
        (
            "invalid storage class",
            config(&["apps/api"]).replace("premium-rwo", "bad/class"),
        ),
        ("malformed KSA", config(&["apps"])),
        ("placeholder KSA", config(&["namespace/name"])),
        ("duplicate KSA", config(&["apps/api", "apps/api"])),
        (
            "empty KSA list",
            config(&["apps/api"]).replace(
                "allowedServiceAccounts:\n  - apps/api\n",
                "allowedServiceAccounts: []\n",
            ),
        ),
    ];

    for (index, (case, text)) in cases.into_iter().enumerate() {
        let dir = tempfile::tempdir().unwrap();
        let file = write_config(dir.path(), &text);
        let out = dir.path().join(format!("dist-{index}"));
        let result = render(&file, &out);
        assert_failure(&result);
        assert!(!out.exists(), "{case} created output");
    }
}

#[test]
fn render_refuses_unmanaged_and_symlink_outputs_without_changing_them() {
    let dir = tempfile::tempdir().unwrap();
    let file = write_config(dir.path(), &config(&["apps/api"]));
    let out = dir.path().join("dist");
    fs::create_dir(&out).unwrap();
    fs::write(out.join("keep.txt"), b"user bytes").unwrap();
    let before = collect_tree(&out);
    assert_failure(&render(&file, &out));
    assert_eq!(collect_tree(&out), before);

    #[cfg(unix)]
    {
        use std::os::unix::fs::symlink;
        let target = dir.path().join("target");
        fs::create_dir(&target).unwrap();
        let link = dir.path().join("linked-dist");
        symlink(&target, &link).unwrap();
        assert_failure(&render(&file, &link));
        assert!(fs::read_dir(target).unwrap().next().is_none());
    }

    let managed = dir.path().join("managed");
    fs::create_dir(&managed).unwrap();
    fs::write(managed.join(".lumen-standalone-managed"), MARKER).unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::symlink;
        let target = dir.path().join("storage-target");
        fs::create_dir(&target).unwrap();
        symlink(&target, managed.join("storage")).unwrap();
        assert_failure(&render(&file, &managed));
        assert!(managed.join("storage").is_symlink());
    }
}

#[test]
fn managed_rerender_removes_stale_rbac_and_invalid_input_preserves_old_tree() {
    let dir = tempfile::tempdir().unwrap();
    let file = write_config(dir.path(), &config(&["apps/api", "zeta/worker"]));
    let out = dir.path().join("dist");
    assert_success(&render(&file, &out));
    fs::write(out.join("runtime/stale-rolebinding.yaml"), b"stale").unwrap();

    fs::write(&file, config(&["apps/api"])).unwrap();
    assert_success(&render(&file, &out));
    assert!(!out.join("runtime/stale-rolebinding.yaml").exists());
    assert!(!out.join("runtime/client-rolebinding-001.yaml").exists());
    assert!(out.join("runtime/client-rolebinding-000.yaml").exists());
    assert!(out.join("storage/pvc.yaml").exists());

    let before = collect_tree(&out);
    fs::write(
        &file,
        config(&["apps/api"]).replace("cpu: 1500m", "cpu: broken"),
    )
    .unwrap();
    assert_failure(&render(&file, &out));
    assert_eq!(collect_tree(&out), before);

    fs::remove_dir_all(out.join("runtime")).unwrap();
    assert!(out.join("storage/pvc.yaml").exists());
}
