use std::process::Command;

struct DeclaredCall {
    api_group: &'static str,
    resource: &'static str,
    verb: &'static str,
    resource_name: Option<&'static str>,
}

const DECLARED_CALLS: &[DeclaredCall] = &[
    // Custom resources (Lumen CRD & Fleet)
    DeclaredCall {
        api_group: "lumen.dev",
        resource: "lumens",
        verb: "get",
        resource_name: None,
    },
    DeclaredCall {
        api_group: "lumen.dev",
        resource: "lumens",
        verb: "list",
        resource_name: None,
    },
    DeclaredCall {
        api_group: "lumen.dev",
        resource: "lumens",
        verb: "watch",
        resource_name: None,
    },
    DeclaredCall {
        api_group: "lumen.dev",
        resource: "lumens",
        verb: "create",
        resource_name: None,
    },
    DeclaredCall {
        api_group: "lumen.dev",
        resource: "lumens",
        verb: "patch",
        resource_name: None,
    },
    DeclaredCall {
        api_group: "lumen.dev",
        resource: "lumens",
        verb: "delete",
        resource_name: None,
    },
    DeclaredCall {
        api_group: "lumen.dev",
        resource: "lumens/status",
        verb: "patch",
        resource_name: None,
    },
    DeclaredCall {
        api_group: "lumen.dev",
        resource: "lumenfleets",
        verb: "list",
        resource_name: None,
    },
    DeclaredCall {
        api_group: "lumen.dev",
        resource: "lumenfleets/status",
        verb: "patch",
        resource_name: None,
    },
    // Core resources (Namespace check + rendered children server-side apply)
    DeclaredCall {
        api_group: "",
        resource: "namespaces",
        verb: "get",
        resource_name: None,
    },
    DeclaredCall {
        api_group: "",
        resource: "services",
        verb: "patch",
        resource_name: None,
    },
    DeclaredCall {
        api_group: "",
        resource: "configmaps",
        verb: "patch",
        resource_name: None,
    },
    DeclaredCall {
        api_group: "",
        resource: "serviceaccounts",
        verb: "patch",
        resource_name: None,
    },
    // Workloads (StatefulSet readiness check + server-side apply)
    DeclaredCall {
        api_group: "apps",
        resource: "statefulsets",
        verb: "get",
        resource_name: None,
    },
    DeclaredCall {
        api_group: "apps",
        resource: "statefulsets",
        verb: "patch",
        resource_name: None,
    },
    // Batch (CronJob server-side apply)
    DeclaredCall {
        api_group: "batch",
        resource: "cronjobs",
        verb: "patch",
        resource_name: None,
    },
    // Autoscaling & Policy (HPA existence get & prune delete; PDB server-side apply)
    DeclaredCall {
        api_group: "autoscaling",
        resource: "horizontalpodautoscalers",
        verb: "get",
        resource_name: None,
    },
    DeclaredCall {
        api_group: "autoscaling",
        resource: "horizontalpodautoscalers",
        verb: "delete",
        resource_name: None,
    },
    DeclaredCall {
        api_group: "policy",
        resource: "poddisruptionbudgets",
        verb: "patch",
        resource_name: None,
    },
    // Networking (NetworkPolicy existence get, server-side apply patch, & prune delete)
    DeclaredCall {
        api_group: "networking.k8s.io",
        resource: "networkpolicies",
        verb: "get",
        resource_name: None,
    },
    DeclaredCall {
        api_group: "networking.k8s.io",
        resource: "networkpolicies",
        verb: "patch",
        resource_name: None,
    },
    DeclaredCall {
        api_group: "networking.k8s.io",
        resource: "networkpolicies",
        verb: "delete",
        resource_name: None,
    },
    // Monitoring (ServiceMonitor & PrometheusRule server-side apply)
    DeclaredCall {
        api_group: "monitoring.coreos.com",
        resource: "servicemonitors",
        verb: "patch",
        resource_name: None,
    },
    DeclaredCall {
        api_group: "monitoring.coreos.com",
        resource: "prometheusrules",
        verb: "patch",
        resource_name: None,
    },
    // Coordination (Leader election LeaseManager)
    DeclaredCall {
        api_group: "coordination.k8s.io",
        resource: "leases",
        verb: "get",
        resource_name: None,
    },
    DeclaredCall {
        api_group: "coordination.k8s.io",
        resource: "leases",
        verb: "create",
        resource_name: None,
    },
    DeclaredCall {
        api_group: "coordination.k8s.io",
        resource: "leases",
        verb: "update",
        resource_name: None,
    },
    // Events (Reconcile event recorder)
    DeclaredCall {
        api_group: "events.k8s.io",
        resource: "events",
        verb: "create",
        resource_name: None,
    },
    DeclaredCall {
        api_group: "events.k8s.io",
        resource: "events",
        verb: "patch",
        resource_name: None,
    },
    // RBAC (KubeAuthDelegatorControl list/patch/delete + auth-delegator bind)
    DeclaredCall {
        api_group: "rbac.authorization.k8s.io",
        resource: "clusterrolebindings",
        verb: "list",
        resource_name: None,
    },
    DeclaredCall {
        api_group: "rbac.authorization.k8s.io",
        resource: "clusterrolebindings",
        verb: "patch",
        resource_name: None,
    },
    DeclaredCall {
        api_group: "rbac.authorization.k8s.io",
        resource: "clusterrolebindings",
        verb: "delete",
        resource_name: None,
    },
    DeclaredCall {
        api_group: "rbac.authorization.k8s.io",
        resource: "clusterroles",
        verb: "bind",
        resource_name: Some("system:auth-delegator"),
    },
];

fn render_operator_manifest() -> String {
    let binary = env!("CARGO_BIN_EXE_lumen");
    let output = Command::new(binary)
        .args([
            "k8s",
            "operator",
            "render",
        ])
        .output()
        .expect("failed to execute lumen CLI");

    assert!(
        output.status.success(),
        "lumen k8s operator render failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    String::from_utf8(output.stdout).expect("CLI output is not UTF-8")
}

fn parse_cluster_role(manifest: &str) -> serde_yaml::Value {
    for doc in manifest.split("\n---") {
        if let Ok(val) = serde_yaml::from_str::<serde_yaml::Value>(doc) {
            if val.get("kind").and_then(|k| k.as_str()) == Some("ClusterRole") {
                return val;
            }
        }
    }
    panic!("ClusterRole document not found in rendered manifest");
}

fn is_call_covered_by_role(role_val: &serde_yaml::Value, call: &DeclaredCall) -> bool {
    let Some(rules) = role_val.get("rules").and_then(|r| r.as_sequence()) else {
        return false;
    };
    for rule in rules {
        let api_groups = rule.get("apiGroups").and_then(|g| g.as_sequence());
        let resources = rule.get("resources").and_then(|r| r.as_sequence());
        let verbs = rule.get("verbs").and_then(|v| v.as_sequence());
        let resource_names = rule.get("resourceNames").and_then(|rn| rn.as_sequence());

        let matches_group = api_groups.map_or(false, |groups| {
            groups
                .iter()
                .any(|g| g.as_str() == Some(call.api_group) || g.as_str() == Some("*"))
        });
        let matches_resource = resources.map_or(false, |res| {
            res.iter()
                .any(|r| r.as_str() == Some(call.resource) || r.as_str() == Some("*"))
        });
        let matches_verb = verbs.map_or(false, |v_seq| {
            v_seq
                .iter()
                .any(|v| v.as_str() == Some(call.verb) || v.as_str() == Some("*"))
        });
        let matches_resource_name = match (call.resource_name, resource_names) {
            (None, None) => true,
            (None, Some(rn_seq)) => rn_seq.is_empty(),
            (Some(_), None) => true,
            (Some(name), Some(rn_seq)) => {
                rn_seq.is_empty() || rn_seq.iter().any(|rn| rn.as_str() == Some(name))
            }
        };

        if matches_group && matches_resource && matches_verb && matches_resource_name {
            return true;
        }
    }
    false
}

#[derive(Debug)]
struct ScannerHit {
    file_path: String,
    line_number: usize,
    matched_text: String,
}

fn is_secret_client_line(line: &str) -> bool {
    let s = line.trim();
    if s.starts_with("//") || s.starts_with("///") || s.starts_with("/*") || s.starts_with("*") {
        return false;
    }
    if s.contains("Secret") {
        if s.contains("Api<") || s.contains("Api::<") || s.contains("v1::Secret") {
            return true;
        }
    }
    false
}

fn scan_lines_for_secret_client(file_path: &str, content: &str) -> Vec<ScannerHit> {
    let mut hits = Vec::new();
    for (idx, line) in content.lines().enumerate() {
        if is_secret_client_line(line) {
            hits.push(ScannerHit {
                file_path: file_path.to_string(),
                line_number: idx + 1,
                matched_text: line.trim().to_string(),
            });
        }
    }
    hits
}

fn scan_dir_recursive(dir: &std::path::Path) -> Vec<ScannerHit> {
    let mut hits = Vec::new();
    let entries = match std::fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(err) => panic!("failed to read directory {dir:?}: {err}"),
    };

    for entry in entries {
        let entry = entry.expect("valid DirEntry");
        let path = entry.path();
        if path.is_dir() {
            hits.extend(scan_dir_recursive(&path));
        } else if path.is_file() && path.extension().and_then(|e| e.to_str()) == Some("rs") {
            let content = std::fs::read_to_string(&path)
                .unwrap_or_else(|err| panic!("failed to read file {path:?}: {err}"));
            let file_hits = scan_lines_for_secret_client(path.to_str().unwrap(), &content);
            hits.extend(file_hits);
        }
    }
    hits
}

#[test]
fn test_no_secrets_grant_in_rendered_cluster_role() {
    let manifest = render_operator_manifest();
    let cluster_role = parse_cluster_role(&manifest);
    let rules = cluster_role
        .get("rules")
        .and_then(|r| r.as_sequence())
        .expect("ClusterRole must contain rules array");

    for rule in rules {
        let resources = rule.get("resources").and_then(|r| r.as_sequence());
        if let Some(res_seq) = resources {
            for res in res_seq {
                if let Some(res_str) = res.as_str() {
                    assert!(
                        res_str != "secrets" && res_str != "*",
                        "rendered ClusterRole grants secrets verb in rule: {rule:?}"
                    );
                }
            }
        }
    }
}

#[test]
fn test_no_secret_client_in_operator_source() {
    let operator_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src/operator");
    let hits = scan_dir_recursive(&operator_dir);

    if !hits.is_empty() {
        let mut msg =
            String::from("Kubernetes Secret API client construction found in operator source:\n");
        for hit in &hits {
            msg.push_str(&format!(
                "  File: {}, Line: {}, Code: {}\n",
                hit.file_path, hit.line_number, hit.matched_text
            ));
        }
        panic!("{msg}");
    }
}

#[test]
fn test_secret_client_scanner_fixture_proof() {
    let planted_fixture = r#"
        fn dummy() {
            let api: Api<Secret> = Api::namespaced(client, "default");
        }
    "#;
    let clean_fixture = r#"
        fn dummy() {
            let api: Api<Lumen> = Api::namespaced(client, "default");
        }
    "#;

    let planted_hits = scan_lines_for_secret_client("planted_fixture.rs", planted_fixture);
    assert!(
        !planted_hits.is_empty(),
        "scanner must report a hit on planted fixture"
    );
    assert_eq!(planted_hits[0].line_number, 3);
    assert!(planted_hits[0].matched_text.contains("Api<Secret>"));

    let clean_hits = scan_lines_for_secret_client("clean_fixture.rs", clean_fixture);
    assert!(
        clean_hits.is_empty(),
        "scanner must report no hits on clean fixture"
    );
}

#[test]
fn test_secret_client_scanner_recursive_fixture_proof() {
    let temp_root =
        std::env::temp_dir().join(format!("lumen_rbac_coverage_test_{}", std::process::id()));
    let nested_dir = temp_root.join("sub_module");
    std::fs::create_dir_all(&nested_dir).expect("failed to create temp nested directory");

    let nested_file = nested_dir.join("certificate.rs");
    let fixture_code = r#"
        pub async fn obtain_secret() {
            let api: Api<Secret> = Api::namespaced(client, "default");
        }
    "#;
    std::fs::write(&nested_file, fixture_code).expect("failed to write nested test file");

    let hits = scan_dir_recursive(&temp_root);

    // Clean up temporary directory
    let _ = std::fs::remove_dir_all(&temp_root);

    assert!(
        !hits.is_empty(),
        "recursive scanner must report a hit for nested .rs file containing planted Secret client"
    );
    assert_eq!(hits[0].line_number, 3);
    assert!(hits[0].matched_text.contains("Api<Secret>"));
    assert!(
        hits[0].file_path.contains("sub_module") && hits[0].file_path.contains("certificate.rs"),
        "hit must name nested path, got: {}",
        hits[0].file_path
    );
}

#[test]
fn test_declared_operator_calls_are_granted_and_ungranted_fails() {
    let manifest = render_operator_manifest();
    let cluster_role = parse_cluster_role(&manifest);

    for call in DECLARED_CALLS {
        assert!(
            is_call_covered_by_role(&cluster_role, call),
            "declared operator call ({:?}, {:?}, {:?}, {:?}) is not covered by rendered ClusterRole",
            call.api_group, call.resource, call.verb, call.resource_name
        );
    }

    let ungranted_call = DeclaredCall {
        api_group: "",
        resource: "secrets",
        verb: "get",
        resource_name: None,
    };
    assert!(
        !is_call_covered_by_role(&cluster_role, &ungranted_call),
        "scanner coverage check must fail for ungranted call: ({:?}, {:?}, {:?})",
        ungranted_call.api_group,
        ungranted_call.resource,
        ungranted_call.verb
    );
}
