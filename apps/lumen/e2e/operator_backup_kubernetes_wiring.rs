// HANDWRITE-BEGIN gap="missing-generator:unit-test:lumen-operator-backup-kubernetes-wiring" tracker="#2370" reason="Parse Lumen's rendered backup identity and static operator RBAC as one regression gate before live GKE operator acceptance."
#![cfg(feature = "operator")]

use kube::api::ObjectMeta;
use lumen::operator::crd::{
    AuthMode, LogFormat, ReshardPolicy, ServingBackupSpec, ServingSpec, ShardMapSpec,
};
use lumen::operator::{render, Lumen, LumenSpec};
use serde::Deserialize;
use serde_json::Value;
use std::process::Command;

fn lumen_with_backup() -> Lumen {
    let mut spec = LumenSpec {
        image: "asia-east1-docker.pkg.dev/example/lumen:sha".into(),
        image_pull_policy: Some("Always".into()),
        placement: Default::default(),
        shard_count: 1,
        shard_map: ShardMapSpec::default(),
        replicas_per_shard: 1,
        voter_count: 1,
        log_format: LogFormat::Json,
        log_level: None,
        auth: AuthMode::Off,
        serving: ServingSpec::default(),
        reshard_policy: ReshardPolicy::default(),
        observability: false,
        network_policy: false,
        admission: None,
        service_account_name: None,
        service_account_annotations: std::collections::BTreeMap::new(),
        peer_tls_secret: None,
        serving_tls_secret: None,
        body_limit_bytes: None,
    };
    spec.serving.backup = Some(ServingBackupSpec {
        policy: service_backup::ScheduledBackupPolicy {
            schedule: "*/5 * * * *".into(),
            destination: "gs://lumen-acceptance/backups".into(),
            retention_secs: Some(3600),
        },
        admin_token_secret: None,
    });

    let mut lumen = Lumen::new("search", spec);
    lumen.metadata = ObjectMeta {
        name: Some("search".into()),
        namespace: Some("lumen-acceptance".into()),
        uid: Some("uid-lumen-acceptance".into()),
        ..Default::default()
    };
    lumen
}

fn find<'a>(objects: &'a [Value], kind: &str, name: &str) -> &'a Value {
    objects
        .iter()
        .find(|object| object["kind"] == kind && object["metadata"]["name"] == name)
        .unwrap_or_else(|| panic!("missing {kind}/{name}: {objects:#?}"))
}

#[test]
fn backup_cronjob_uses_owned_cloud_neutral_service_account() {
    let objects = render::render(&lumen_with_backup());
    let service_account = find(&objects, "ServiceAccount", "search-backup");
    let cron_job = find(&objects, "CronJob", "search-backup");
    let serving = find(&objects, "StatefulSet", "search");

    assert_eq!(service_account["metadata"]["namespace"], "lumen-acceptance");
    assert_eq!(
        service_account["metadata"]["labels"]["app.kubernetes.io/instance"],
        "search"
    );
    assert_eq!(
        service_account["metadata"]["labels"]["app.kubernetes.io/component"],
        "backup"
    );
    assert!(
        service_account["metadata"]["annotations"].is_null(),
        "the app renderer must not embed a provider-specific Workload Identity annotation"
    );

    for object in [service_account, cron_job] {
        let owner = &object["metadata"]["ownerReferences"][0];
        assert_eq!(owner["apiVersion"], "lumen.dev/v1alpha1");
        assert_eq!(owner["kind"], "Lumen");
        assert_eq!(owner["name"], "search");
        assert_eq!(owner["uid"], "uid-lumen-acceptance");
        assert_eq!(owner["controller"], true);
        assert_eq!(owner["blockOwnerDeletion"], true);
    }

    assert_eq!(
        cron_job["spec"]["jobTemplate"]["spec"]["template"]["spec"]["serviceAccountName"],
        "search-backup"
    );
    assert_eq!(
        serving["spec"]["template"]["spec"]["serviceAccountName"], "search",
        "object-storage identity must not be granted to serving pods"
    );
    assert_eq!(
        cron_job["spec"]["jobTemplate"]["spec"]["template"]["spec"]["containers"][0]["args"][4],
        "gs://lumen-acceptance/backups"
    );
}

#[test]
fn backup_service_account_is_stable_when_schedule_is_disabled() {
    let mut lumen = lumen_with_backup();
    lumen.spec.serving.backup = None;
    let objects = render::render(&lumen);

    find(&objects, "ServiceAccount", "search-backup");
    assert!(
        objects.iter().all(|object| object["kind"] != "CronJob"),
        "disabling the schedule must remove the CronJob from desired state"
    );
}

fn operator_cluster_role() -> serde_yaml::Value {
    let documents = serde_yaml::Deserializer::from_str(include_str!("../k8s/operator/rbac.yaml"));
    documents
        .map(|document| serde_yaml::Value::deserialize(document).expect("RBAC document parses"))
        .find(|document| document["kind"] == "ClusterRole")
        .expect("operator ClusterRole")
}

fn rule_for<'a>(
    role: &'a serde_yaml::Value,
    api_group: &str,
    resource: &str,
) -> Option<&'a serde_yaml::Value> {
    role["rules"]
        .as_sequence()
        .expect("ClusterRole rules")
        .iter()
        .find(|rule| {
            rule["apiGroups"]
                .as_sequence()
                .is_some_and(|groups| groups.iter().any(|group| group.as_str() == Some(api_group)))
                && rule["resources"].as_sequence().is_some_and(|resources| {
                    resources
                        .iter()
                        .any(|candidate| candidate.as_str() == Some(resource))
                })
        })
}

fn verbs_for(role: &serde_yaml::Value, api_group: &str, resource: &str) -> Vec<String> {
    rule_for(role, api_group, resource)
        .unwrap_or_else(|| panic!("missing RBAC rule for {api_group:?}/{resource}"))["verbs"]
        .as_sequence()
        .expect("rule verbs")
        .iter()
        .map(|verb| verb.as_str().expect("string verb").to_string())
        .collect()
}

#[test]
fn operator_rbac_can_reconcile_cronjobs_and_holds_no_secret_grant() {
    let role = operator_cluster_role();
    let cronjob_verbs = verbs_for(&role, "batch", "cronjobs");
    for verb in [
        "get", "list", "watch", "create", "update", "patch", "delete",
    ] {
        assert!(
            cronjob_verbs.iter().any(|candidate| candidate == verb),
            "CronJob rule is missing `{verb}`: {cronjob_verbs:?}"
        );
    }

    // #2877: the reshard driver's credential is now its own projected
    // ServiceAccount token, mounted and rotated by the kubelet. Nothing in the
    // operator reads a Secret, and a cluster-wide `secrets` grant on a
    // controller that watches every namespace is the most valuable thing in
    // this file to anyone who reaches the operator's ServiceAccount.
    assert!(
        rule_for(&role, "", "secrets").is_none(),
        "the operator must hold no Secret grant at all: {:#?}",
        role["rules"]
    );

    // #2603: without this grant the operator renders a NetworkPolicy it cannot
    // apply, and the only symptom is a Forbidden buried in reconcile logs.
    let policy_verbs = verbs_for(&role, "networking.k8s.io", "networkpolicies");
    for verb in [
        "get", "list", "watch", "create", "update", "patch", "delete",
    ] {
        assert!(
            policy_verbs.iter().any(|candidate| candidate == verb),
            "NetworkPolicy rule is missing `{verb}`: {policy_verbs:?}"
        );
    }
}

/// #2603: the client API and the Raft port have different audiences. A policy
/// that admits the cluster to 7374 is worse than no policy — it reads as
/// isolation while leaving the consensus protocol open to every pod.
#[test]
fn rendered_network_policy_is_opt_in_and_never_exposes_the_raft_port() {
    let mut lumen = lumen_with_backup();
    assert!(
        !lumen.spec.network_policy,
        "isolation must stay opt-in: a NetworkPolicy is inert without an \
         enforcing CNI, and defaulting it on would silently drop traffic on \
         clusters whose clients sit outside the pod network"
    );
    assert!(
        render::render(&lumen)
            .iter()
            .all(|object| object["kind"] != "NetworkPolicy"),
        "no policy may be rendered while the field is false"
    );

    lumen.spec.network_policy = true;
    let objects = render::render(&lumen);
    let policy = find(&objects, "NetworkPolicy", "search");
    assert_eq!(policy["apiVersion"], "networking.k8s.io/v1");
    assert_eq!(policy["metadata"]["namespace"], "lumen-acceptance");
    assert_eq!(policy["metadata"]["ownerReferences"][0]["kind"], "Lumen");

    // The pods it selects are exactly the serving pods of THIS instance.
    let selected = &policy["spec"]["podSelector"]["matchLabels"];
    assert_eq!(selected["app.kubernetes.io/instance"], "search");
    assert_eq!(selected["app.kubernetes.io/component"], "server");

    let ingress = policy["spec"]["ingress"]
        .as_array()
        .expect("ingress rules")
        .clone();
    let from_cluster = ingress
        .iter()
        .find(|rule| !rule["from"][0]["namespaceSelector"].is_null())
        .expect("a rule admitting the cluster to the client API");
    let cluster_ports: Vec<i64> = from_cluster["ports"]
        .as_array()
        .expect("ports")
        .iter()
        .map(|port| port["port"].as_i64().expect("numeric port"))
        .collect();
    assert_eq!(
        cluster_ports,
        vec![7373],
        "only the search API may be cluster-reachable; 7374 carries Raft"
    );

    let from_peers = ingress
        .iter()
        .find(|rule| !rule["from"][0]["podSelector"].is_null())
        .expect("a rule admitting sibling pods to Raft");
    assert_eq!(from_peers["ports"][0]["port"], 7374);
    assert_eq!(
        from_peers["from"][0]["podSelector"]["matchLabels"]["app.kubernetes.io/instance"], "search",
        "a second Lumen sharing this namespace must not reach these Raft ports"
    );

    // The backup CronJob runs under its own component label and needs egress
    // the serving posture does not grant, so it must fall outside the selector.
    let cron_labels = &find(&objects, "CronJob", "search-backup")["spec"]["jobTemplate"]["spec"]
        ["template"]["metadata"]["labels"];
    assert_ne!(
        cron_labels["app.kubernetes.io/component"], selected["app.kubernetes.io/component"],
        "the backup job must not be selected by the serving pods' policy"
    );
}

#[test]
fn operator_cli_renders_requested_immutable_image_and_preserves_default() {
    let render = |extra: &[&str]| {
        let mut command = Command::new(env!("CARGO_BIN_EXE_lumen"));
        command.args([
            "k8s",
            "operator",
            "render",
        ]);
        command.args(extra);
        let output = command.output().expect("run lumen operator render");
        assert!(
            output.status.success(),
            "operator render failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        String::from_utf8(output.stdout).expect("operator YAML is utf8")
    };

    let default_yaml = render(&[]);
    assert!(default_yaml.contains(&format!(
        "image: ghcr.io/chrischeng-c4/lumen:{}",
        env!("CARGO_PKG_VERSION")
    )));

    let immutable = "asia-east1-docker.pkg.dev/axiom/lumen/lumen@sha256:0123456789abcdef";
    let immutable_yaml = render(&["--namespace", "lumen-live", "--image", immutable]);
    assert!(immutable_yaml.contains("namespace: lumen-live"));
    assert!(immutable_yaml.contains(&format!("image: {immutable}")));
    assert!(!immutable_yaml.contains(&format!(
        "image: ghcr.io/chrischeng-c4/lumen:{}",
        env!("CARGO_PKG_VERSION")
    )));

    let invalid = Command::new(env!("CARGO_BIN_EXE_lumen"))
        .args(["k8s", "operator", "render", "--image", "bad\nimage"])
        .output()
        .expect("run lumen operator render with invalid image");
    assert!(!invalid.status.success());
    assert!(String::from_utf8_lossy(&invalid.stderr).contains("whitespace-free OCI image"));
}

/// Every document the `k8s/operator` kustomization installs, parsed.
fn operator_manifest_documents() -> Vec<serde_yaml::Value> {
    [
        include_str!("../k8s/operator/deployment.yaml"),
        include_str!("../k8s/operator/pdb.yaml"),
    ]
    .iter()
    .flat_map(|source| {
        serde_yaml::Deserializer::from_str(source)
            .map(|document| {
                serde_yaml::Value::deserialize(document).expect("operator document parses")
            })
            .collect::<Vec<_>>()
    })
    .collect()
}

fn document_of_kind<'a>(documents: &'a [serde_yaml::Value], kind: &str) -> &'a serde_yaml::Value {
    documents
        .iter()
        .find(|document| document["kind"] == kind)
        .unwrap_or_else(|| panic!("missing operator {kind}"))
}

/// #2602 AC1: the control plane is HA by default. `replicas: 1` meant any node
/// drain, eviction, or rollout left every Lumen CR in the cluster unreconciled
/// for its duration, even though leader election (`libs/service-k8s/src/lease.rs`)
/// had been in place and unused the whole time.
#[test]
fn operator_deployment_runs_two_leader_elected_replicas() {
    let documents = operator_manifest_documents();
    let deployment = document_of_kind(&documents, "Deployment");
    assert_eq!(deployment["spec"]["replicas"], 2);

    // The standby is only safe because each pod has a distinct election
    // identity and knows which namespace holds the Lease.
    let env = deployment["spec"]["template"]["spec"]["containers"][0]["env"]
        .as_sequence()
        .expect("operator container env");
    for required in ["POD_NAME", "POD_NAMESPACE"] {
        assert!(
            env.iter().any(|entry| entry["name"] == required),
            "leader election needs {required}; have {env:?}"
        );
    }

    // Soft, not hard: a single-node kind/minikube cluster must still schedule
    // both replicas rather than park one Pending forever.
    let anti_affinity = &deployment["spec"]["template"]["spec"]["affinity"]["podAntiAffinity"];
    assert!(
        anti_affinity["requiredDuringSchedulingIgnoredDuringExecution"].is_null(),
        "required anti-affinity would make the two-replica floor unschedulable on one node"
    );
    assert_eq!(
        anti_affinity["preferredDuringSchedulingIgnoredDuringExecution"][0]["podAffinityTerm"]
            ["topologyKey"],
        "kubernetes.io/hostname"
    );
}

/// #2602 AC1: the replica floor only survives a node drain if evictions are
/// serialized — otherwise both pods can go at once and the cluster is left
/// without a reconciler until a replacement becomes ready and takes the Lease.
#[test]
fn operator_pdb_serializes_eviction_across_the_replicas() {
    let documents = operator_manifest_documents();
    let pdb = document_of_kind(&documents, "PodDisruptionBudget");
    let deployment = document_of_kind(&documents, "Deployment");

    assert_eq!(pdb["apiVersion"], "policy/v1");
    assert_eq!(pdb["spec"]["maxUnavailable"], 1);
    assert_eq!(
        pdb["metadata"]["namespace"],
        deployment["metadata"]["namespace"]
    );
    // A selector that does not match the Deployment's pods is a silently
    // inert PDB — the exact failure this assertion exists to catch.
    assert_eq!(
        pdb["spec"]["selector"]["matchLabels"],
        deployment["spec"]["selector"]["matchLabels"]
    );
}

/// #2602 R3: kustomize consumers (`kubectl apply -k k8s/operator`) and render
/// consumers (`lumen k8s operator render`) install the same operator layer.
/// A replica floor or a PDB that exists in only one of the two is a divergence
/// an integrator discovers in production.
#[test]
fn operator_render_and_static_manifest_agree_on_the_ha_shape() {
    let rendered = Command::new(env!("CARGO_BIN_EXE_lumen"))
        .args([
            "k8s",
            "operator",
            "render",
        ])
        .output()
        .expect("run lumen operator render");
    assert!(
        rendered.status.success(),
        "operator render failed: {}",
        String::from_utf8_lossy(&rendered.stderr)
    );
    let yaml = String::from_utf8(rendered.stdout).expect("operator YAML is utf8");
    let documents: Vec<serde_yaml::Value> = serde_yaml::Deserializer::from_str(&yaml)
        .map(|document| serde_yaml::Value::deserialize(document).expect("rendered document parses"))
        .collect();

    let deployment = document_of_kind(&documents, "Deployment");
    assert_eq!(deployment["spec"]["replicas"], 2);
    let pdb = document_of_kind(&documents, "PodDisruptionBudget");
    assert_eq!(pdb["spec"]["maxUnavailable"], 1);

    // `--namespace` moves the whole layer, PDB included; a PDB left in
    // `lumen-system` would not cover pods rendered elsewhere.
    let relocated = Command::new(env!("CARGO_BIN_EXE_lumen"))
        .args([
            "k8s",
            "operator",
            "render",
            "--namespace",
            "lumen-live",
        ])
        .output()
        .expect("run lumen operator render --namespace");
    let relocated_yaml = String::from_utf8(relocated.stdout).expect("operator YAML is utf8");
    let relocated_documents: Vec<serde_yaml::Value> =
        serde_yaml::Deserializer::from_str(&relocated_yaml)
            .map(|document| {
                serde_yaml::Value::deserialize(document).expect("rendered document parses")
            })
            .collect();
    assert_eq!(
        document_of_kind(&relocated_documents, "PodDisruptionBudget")["metadata"]["namespace"],
        "lumen-live"
    );

    // The kustomization must actually install what render emits, or the two
    // consumer paths silently diverge again the next time one side changes.
    let kustomization = include_str!("../k8s/operator/kustomization.yaml");
    assert!(
        kustomization.contains("- pdb.yaml"),
        "k8s/operator/kustomization.yaml does not install pdb.yaml"
    );
}

/// #2532: the operator manifest pinned `0.4.24` while the workspace shipped
/// `0.4.25` — the handout deployed one release behind, the same defect already
/// fixed in `k8s/base/deployment.yaml`. The render path now derives the pin it
/// expects from `CARGO_PKG_VERSION`, so a release bump that misses this file
/// fails the render instead of silently handing out a stale image.
#[test]
fn operator_manifest_pins_this_workspaces_version() {
    let deployment = include_str!("../k8s/operator/deployment.yaml");
    assert!(
        deployment.contains(&format!(
            "image: ghcr.io/chrischeng-c4/lumen:{}",
            env!("CARGO_PKG_VERSION")
        )),
        "k8s/operator/deployment.yaml must pin {}; the release bump missed it",
        env!("CARGO_PKG_VERSION")
    );
    // The comment is the human half of the same guard: it names the literal a
    // release bump greps for. Assert the backticked token rather than the prose
    // around it, so reflowing the comment does not break the test but deleting
    // the bump instruction does.
    assert!(
        deployment.contains("grep target") && deployment.contains("`ghcr.io/chrischeng-c4/lumen:`"),
        "the bump procedure's grep target comment must survive edits to this file"
    );
}

/// Run `lumen k8s operator render` and parse every emitted document.
fn rendered_operator_documents(extra: &[&str]) -> Vec<serde_yaml::Value> {
    let mut command = Command::new(env!("CARGO_BIN_EXE_lumen"));
    command.args([
        "k8s",
        "operator",
        "render",
    ]);
    command.args(extra);
    let output = command.output().expect("run lumen operator render");
    assert!(
        output.status.success(),
        "operator render failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let yaml = String::from_utf8(output.stdout).expect("operator YAML is utf8");
    serde_yaml::Deserializer::from_str(&yaml)
        .map(|document| serde_yaml::Value::deserialize(document).expect("rendered document parses"))
        .collect()
}

fn optional_document_of_kind<'a>(
    documents: &'a [serde_yaml::Value],
    kind: &str,
) -> Option<&'a serde_yaml::Value> {
    documents.iter().find(|document| document["kind"] == kind)
}

/// #2621 R2/R6: the metrics endpoint #2620 opens inside the pod is only
/// reachable if something selects it. The Service carries no CRD dependency, so
/// it is unconditional on both consumer paths — a cluster with no monitoring
/// stack still gets the target, and turning monitoring on later never has to
/// come back and add it.
#[test]
fn operator_metrics_service_selects_the_pods_on_the_port_they_publish() {
    let documents = rendered_operator_documents(&[]);
    let service = document_of_kind(&documents, "Service");
    let deployment = document_of_kind(&documents, "Deployment");

    assert_eq!(service["metadata"]["name"], "lumen-operator-metrics");
    assert_eq!(
        service["metadata"]["namespace"],
        deployment["metadata"]["namespace"]
    );

    // A selector that does not match the pod template is a Service with no
    // Endpoints — it applies cleanly and scrapes nothing.
    let selector = &service["spec"]["selector"];
    let pod_labels = &deployment["spec"]["template"]["metadata"]["labels"];
    for (key, value) in selector.as_mapping().expect("service selector") {
        assert_eq!(
            &pod_labels[key], value,
            "service selector {key:?}={value:?} does not match the operator pod labels"
        );
    }

    // targetPort is the container port's *name*, so the two must agree by name
    // and the container must actually declare it.
    let port = &service["spec"]["ports"][0];
    assert_eq!(port["port"], 9090);
    let container_ports = deployment["spec"]["template"]["spec"]["containers"][0]["ports"]
        .as_sequence()
        .expect("operator container declares ports");
    assert!(
        container_ports
            .iter()
            .any(|declared| declared["name"] == port["targetPort"]
                && declared["containerPort"] == port["port"]),
        "Service targetPort {:?} matches no container port: {container_ports:?}",
        port["targetPort"]
    );

    // Both consumer paths install it, or they diverge again.
    assert!(
        include_str!("../k8s/operator/kustomization.yaml").contains("- service.yaml"),
        "k8s/operator/kustomization.yaml does not install service.yaml"
    );
}

/// #2621 R3/R6: the ServiceMonitor and PrometheusRule are
/// `monitoring.coreos.com/v1` CRDs. Emitting them unconditionally would make
/// `kubectl apply` of the whole render fail on any cluster without
/// prometheus-operator — taking the operator down along with the alerts — so
/// they sit behind `--monitoring`, mirroring the opt-in kustomize component.
#[test]
fn monitoring_objects_are_opt_in_on_both_consumer_paths() {
    let default_documents = rendered_operator_documents(&[]);
    for crd_dependent in ["ServiceMonitor", "PrometheusRule"] {
        assert!(
            optional_document_of_kind(&default_documents, crd_dependent).is_none(),
            "{crd_dependent} must not be rendered without --monitoring; it needs CRDs a vanilla cluster lacks"
        );
    }

    let documents = rendered_operator_documents(&["--monitoring"]);
    let monitor = document_of_kind(&documents, "ServiceMonitor");
    let rule = document_of_kind(&documents, "PrometheusRule");
    // The gate must not drop the CRD-free layer on the way through.
    document_of_kind(&documents, "Service");
    document_of_kind(&documents, "Deployment");

    // The ServiceMonitor must select the Service that exists, in the namespace
    // that Service is in; either mismatch discovers nothing and reports no error.
    let service = document_of_kind(&documents, "Service");
    assert_eq!(
        monitor["spec"]["namespaceSelector"]["matchNames"][0],
        service["metadata"]["namespace"]
    );
    let monitor_selector = &monitor["spec"]["selector"]["matchLabels"];
    for (key, value) in monitor_selector.as_mapping().expect("monitor selector") {
        assert_eq!(
            &service["metadata"]["labels"][key], value,
            "ServiceMonitor selector {key:?}={value:?} does not match the Service labels"
        );
    }
    // Scraping by port *name* is what makes the endpoint survive a port-number
    // change; a numeric mismatch here would silently produce zero targets.
    assert_eq!(
        monitor["spec"]["endpoints"][0]["port"],
        service["spec"]["ports"][0]["name"]
    );
    assert_eq!(monitor["spec"]["endpoints"][0]["path"], "/metrics");

    assert!(
        include_str!("../k8s/components/operator-monitoring/kustomization.yaml")
            .contains("- servicemonitor.yaml"),
        "the operator-monitoring component does not install servicemonitor.yaml"
    );
    assert_eq!(rule["apiVersion"], "monitoring.coreos.com/v1");
}

/// #2621 R4: the two alerts, and specifically *what they are written against*.
///
/// The absence alert is the one that is easy to get wrong: a threshold on any
/// `lumen_operator_*` counter cannot fire when the pod is gone, because the
/// series stops existing. It has to read the scrape target's `up`, and it needs
/// an `absent()` arm for the scale-to-zero case where `up` itself disappears
/// and a plain comparison returns an empty vector forever.
#[test]
fn the_absence_alert_reads_up_and_survives_the_series_disappearing() {
    let documents = rendered_operator_documents(&["--monitoring"]);
    let rule = document_of_kind(&documents, "PrometheusRule");
    let rules = rule["spec"]["groups"][0]["rules"]
        .as_sequence()
        .expect("alert rules");

    let absent = rules
        .iter()
        .find(|entry| entry["alert"] == "LumenOperatorAbsent")
        .expect("LumenOperatorAbsent alert");
    let expr = absent["expr"].as_str().expect("absence expr");
    assert!(
        expr.contains("up{"),
        "the absence alert must read the scrape target's `up`, not a lumen counter: {expr}"
    );
    assert!(
        !expr.contains("lumen_operator_"),
        "a lumen_operator_* series cannot report its own process's death: {expr}"
    );
    assert!(
        expr.contains("absent("),
        "without an absent() arm, a scale-to-zero removes the `up` series and this alert \
         silently stops evaluating instead of firing: {expr}"
    );

    let error_rate = rules
        .iter()
        .find(|entry| entry["alert"] == "LumenOperatorReconcileErrorRate")
        .expect("LumenOperatorReconcileErrorRate alert");
    let expr = error_rate["expr"].as_str().expect("error-rate expr");
    assert!(
        expr.contains("lumen_operator_reconcile_errors_total")
            && expr.contains("lumen_operator_reconcile_total"),
        "the error alert must be a ratio, so a low-traffic operator does not page on one \
         transient failure: {expr}"
    );
    // The denominator floor is also what keeps 0/0 (NaN) out of the comparison
    // and stops this alert double-paging with LumenOperatorAbsent when the
    // attempt rate has gone to zero because the operator is dead.
    assert!(
        expr.contains("and"),
        "the ratio needs a denominator floor: {expr}"
    );
}

/// #2621 R5/AC5: a runbook link that 404s is worse than none. Every
/// `runbook_url` names a path in this repository, asserted here rather than
/// discovered by an on-call at 3am.
#[test]
fn every_runbook_url_resolves_to_a_file_in_this_repository() {
    let documents = rendered_operator_documents(&["--monitoring"]);
    let rule = document_of_kind(&documents, "PrometheusRule");
    let rules = rule["spec"]["groups"][0]["rules"]
        .as_sequence()
        .expect("alert rules");
    assert!(!rules.is_empty());

    // e2e/ -> apps/lumen -> apps -> repo root.
    let repo_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(std::path::Path::parent)
        .expect("repo root");

    for entry in rules {
        let annotations = &entry["annotations"];
        let url = annotations["runbook_url"]
            .as_str()
            .unwrap_or_else(|| panic!("{:?} has no runbook_url", entry["alert"]));
        let path = url.split('#').next().expect("runbook path");
        assert!(
            repo_root.join(path).is_file(),
            "{:?} points at {path}, which does not exist",
            entry["alert"]
        );
        // The operational half of the same requirement: lumen's house
        // convention is an inline recipe, because a link is one more hop
        // during an incident.
        assert!(
            annotations["runbook"]
                .as_str()
                .is_some_and(|r| r.contains("kubectl")),
            "{:?} has no inline kubectl recipe",
            entry["alert"]
        );
        assert!(annotations["summary"].as_str().is_some());
        assert!(annotations["description"].as_str().is_some());
    }
}

/// #2621 AC4: a malformed PromQL expression does not fail an apply — the
/// PrometheusRule is accepted and the alert simply never fires, which is
/// indistinguishable from "nothing is wrong". `promtool` is the only thing
/// that reads the expressions the way Prometheus will.
///
/// Skips when promtool is not installed (`brew install prometheus`), following
/// the repo's real-services testing convention; the content assertions above
/// hold regardless.
#[test]
fn promtool_accepts_the_rendered_expressions_and_annotation_templates() {
    if Command::new("promtool").arg("--version").output().is_err() {
        eprintln!("promtool not installed; skipping (brew install prometheus)");
        return;
    }

    // promtool reads a bare rules file, not the CRD wrapper — unwrap `spec`,
    // which is exactly what prometheus-operator hands the Prometheus pod.
    let rule: serde_yaml::Value = serde_yaml::from_str(include_str!(
        "../k8s/components/operator-monitoring/prometheusrule.yaml"
    ))
    .expect("PrometheusRule parses");
    let path = std::env::temp_dir().join("lumen-operator-rules-2621.yaml");
    std::fs::write(
        &path,
        serde_yaml::to_string(&rule["spec"]).expect("rules serialize"),
    )
    .expect("write rules file");

    let output = Command::new("promtool")
        .arg("check")
        .arg("rules")
        .arg(&path)
        .output()
        .expect("run promtool check rules");
    assert!(
        output.status.success(),
        "promtool rejected the operator alert rules:\n{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

/// #2621 AC4, the half promtool does *not* cover. An alert's `labels` become
/// labels on the fired alert series, so their keys are Prometheus label names
/// and must match `[a-zA-Z_][a-zA-Z0-9_]*` — while the surrounding
/// `metadata.labels` are Kubernetes label keys, where `app.kubernetes.io/name`
/// is not only legal but conventional. Writing the Kubernetes style in the
/// alert block is therefore a natural mistake sitting six lines from a place
/// it is correct.
///
/// It was caught by a kind cluster, not by the gate above: promtool 3.x
/// accepts UTF-8 label names, while prometheus-operator's rule validator
/// enforces the legacy regex and rejects the whole PrometheusRule — so the
/// apply fails and *no* alert is installed, including the healthy one. This
/// test exists because the tool we would reach for cannot see it.
#[test]
fn alert_label_keys_are_prometheus_label_names_not_kubernetes_ones() {
    fn legal(key: &str) -> bool {
        let mut chars = key.chars();
        chars
            .next()
            .is_some_and(|c| c.is_ascii_alphabetic() || c == '_')
            && chars.all(|c| c.is_ascii_alphanumeric() || c == '_')
    }

    let documents = rendered_operator_documents(&["--monitoring"]);
    let rule = document_of_kind(&documents, "PrometheusRule");

    // The metadata block is the contrast case: Kubernetes keys belong here.
    assert!(
        rule["metadata"]["labels"]
            .as_mapping()
            .expect("metadata labels")
            .keys()
            .any(|k| k.as_str() == Some("app.kubernetes.io/name")),
        "metadata should keep the Kubernetes-style labels"
    );

    let rules = rule["spec"]["groups"][0]["rules"]
        .as_sequence()
        .expect("alert rules");
    assert!(!rules.is_empty());
    for entry in rules {
        let labels = entry["labels"].as_mapping().unwrap_or_else(|| {
            panic!("{:?} has no labels", entry["alert"]);
        });
        for key in labels.keys() {
            let key = key.as_str().expect("label key is a string");
            assert!(
                legal(key),
                "{:?} carries alert label {key:?}, which prometheus-operator \
                 will reject — the whole PrometheusRule fails to apply, so \
                 neither alert gets installed",
                entry["alert"]
            );
        }
        assert!(
            labels.get("severity").is_some(),
            "{:?} has no severity",
            entry["alert"]
        );
    }
}

/// #2621 R6: `--namespace` has to move the *whole* control plane. The shared
/// `replace_kubernetes_namespace` helper only rewrites `name:`/`namespace:`
/// keys, so the monitoring layer's other three shapes — the ServiceMonitor's
/// `namespaceSelector`, the PromQL `namespace="..."` matchers, and the `-n`
/// in the runbook commands — need explicit handling. Each one fails silently
/// if missed: a selector that discovers nothing, an expression that can never
/// fire, and a recipe that runs against the wrong namespace.
#[test]
fn relocating_the_operator_relocates_its_monitoring_too() {
    let documents = rendered_operator_documents(&["--namespace", "lumen-live", "--monitoring"]);
    let monitor = document_of_kind(&documents, "ServiceMonitor");
    let rule = document_of_kind(&documents, "PrometheusRule");

    assert_eq!(monitor["metadata"]["namespace"], "lumen-live");
    assert_eq!(
        monitor["spec"]["namespaceSelector"]["matchNames"][0], "lumen-live",
        "a ServiceMonitor pointed at the old namespace discovers no target and reports no error"
    );
    assert_eq!(rule["metadata"]["namespace"], "lumen-live");

    for entry in rule["spec"]["groups"][0]["rules"]
        .as_sequence()
        .expect("alert rules")
    {
        let expr = entry["expr"].as_str().expect("expr");
        assert!(
            !expr.contains("lumen-system"),
            "{:?} still matches the default namespace after relocation, so it can never fire: {expr}",
            entry["alert"]
        );
        let runbook = entry["annotations"]["runbook"].as_str().expect("runbook");
        assert!(
            !runbook.contains("-n lumen-system"),
            "{:?}'s runbook sends the on-call to the wrong namespace",
            entry["alert"]
        );
    }
}

/// The token projection on one workload, whichever manifest language it came
/// from: `(audience, expirationSeconds, file path, mount is read-only)`.
///
/// Both control-plane callers are checked through this one function on
/// purpose. The operator's projection is hand-written YAML and the backup
/// runner's is rendered Rust, and the only thing that keeps those two from
/// drifting is a test that refuses to read them differently.
fn token_projection(pod_spec: &Value) -> (String, i64, String, bool) {
    let volumes = pod_spec["volumes"].as_array().expect("pod volumes");
    let projections: Vec<&Value> = volumes
        .iter()
        .filter(|volume| !volume["projected"]["sources"][0]["serviceAccountToken"].is_null())
        .collect();
    assert_eq!(
        projections.len(),
        1,
        "exactly one projected token per control-plane workload: {volumes:#?}"
    );
    let volume = projections[0];
    let sources = volume["projected"]["sources"]
        .as_array()
        .expect("projected sources");
    assert_eq!(
        sources.len(),
        1,
        "the projection carries a token and nothing else: {sources:#?}"
    );
    let token = &sources[0]["serviceAccountToken"];

    let name = volume["name"].as_str().expect("volume name");
    let mount = pod_spec["containers"][0]["volumeMounts"]
        .as_array()
        .expect("container volumeMounts")
        .iter()
        .find(|mount| mount["name"] == name)
        .unwrap_or_else(|| panic!("nothing mounts the projected volume {name:?}"));

    let path = format!(
        "{}/{}",
        mount["mountPath"].as_str().expect("mountPath"),
        token["path"].as_str().expect("token path")
    );
    (
        token["audience"].as_str().expect("audience").to_string(),
        token["expirationSeconds"]
            .as_i64()
            .expect("expirationSeconds"),
        path,
        mount["readOnly"] == Value::Bool(true),
    )
}

/// #2877 AC1/AC2: the operator and the backup runner each hold their own
/// audience-bound credential, and neither borrows the serving fleet's.
///
/// The audience is the whole point. The default token every pod gets at
/// `/var/run/secrets/kubernetes.io/serviceaccount` is minted for the API
/// server, and a Lumen instance that checks the audience answers it with a
/// bare 401 — which reads exactly like a missing RBAC binding and sends the
/// operator digging in the wrong file. Asking the kubelet for `lumen.axiom.dev`
/// instead means the credential these workloads carry is only usable against
/// the callee it was minted for, and is worthless to anyone who reads it out
/// of a compromised pod ten minutes later.
#[test]
fn each_control_plane_caller_mounts_its_own_audience_bound_token() {
    let documents = operator_manifest_documents();
    let deployment = document_of_kind(&documents, "Deployment");
    let operator_pod: Value = serde_json::to_value(&deployment["spec"]["template"]["spec"])
        .expect("operator pod spec converts to JSON");

    let mut lumen = lumen_with_backup();
    lumen.spec.auth = AuthMode::Required;
    let objects = render::render(&lumen);
    let cron_job = find(&objects, "CronJob", "search-backup");
    let backup_pod = &cron_job["spec"]["jobTemplate"]["spec"]["template"]["spec"];

    for (who, pod) in [("operator", &operator_pod), ("backup", backup_pod)] {
        let (audience, expiration, path, read_only) = token_projection(pod);
        assert_eq!(
            audience, "lumen.axiom.dev",
            "{who} must not present the API server's own audience to Lumen"
        );
        assert_eq!(
            expiration, 600,
            "{who}: 600s is the API server's floor — a smaller number is a workload that never starts"
        );
        assert_eq!(
            path, "/var/run/secrets/lumen.axiom.dev/token",
            "{who} must read the token where its own client looks for it"
        );
        assert!(read_only, "{who} has no reason to write its own credential");

        // AC1: no credential env var. The material stays a file the kubelet
        // rewrites; anything in `env` is frozen at pod creation, survives in
        // `kubectl describe`, and cannot rotate.
        let env = pod["containers"][0]["env"]
            .as_array()
            .cloned()
            .unwrap_or_default();
        for entry in &env {
            let name = entry["name"].as_str().unwrap_or_default();
            assert!(
                !name.contains("TOKEN") && !name.contains("SECRET") && !name.contains("CREDENTIAL"),
                "{who} carries a credential-shaped env var: {entry:#?}"
            );
        }
    }

    // AC2: three identities, three names. The serving fleet's ServiceAccount
    // is authorized for delegated review (#2876); handing it to a caller would
    // make "who asked for this" unanswerable at the serving side and would let
    // a compromised backup pod authenticate as the fleet itself.
    let serving = find(&objects, "StatefulSet", "search");
    let operator_sa = operator_pod["serviceAccountName"]
        .as_str()
        .expect("operator SA");
    let backup_sa = backup_pod["serviceAccountName"]
        .as_str()
        .expect("backup SA");
    let serving_sa = serving["spec"]["template"]["spec"]["serviceAccountName"]
        .as_str()
        .expect("serving SA");
    assert_eq!(operator_sa, "lumen-operator");
    assert_eq!(backup_sa, "search-backup");
    assert_ne!(operator_sa, backup_sa);
    assert_ne!(operator_sa, serving_sa);
    assert_ne!(backup_sa, serving_sa);
}

/// #2877 R4: the CronJob is told where the token is, never what it is.
///
/// A pod spec is not a secret. It is readable by anyone with `get pods` in the
/// namespace, it is checked into whatever repository drives the cluster, and it
/// is echoed by `kubectl describe`. Passing a path keeps the material inside
/// the pod's own mount for the ten minutes it is valid.
///
/// The flag is conditional and the mount is not: an instance with `auth`
/// disabled rejects a *presented* bearer (#2871), so the runner must not read
/// the file there — but the pod shape stays identical either way, because a
/// manifest that changes structure with an auth toggle is one more thing that
/// can be wrong in exactly the deployment nobody tested.
#[test]
fn the_backup_runner_is_given_a_path_and_only_when_auth_is_required() {
    let mut lumen = lumen_with_backup();

    lumen.spec.auth = AuthMode::Required;
    let objects = render::render(&lumen);
    let required = find(&objects, "CronJob", "search-backup").clone();
    let pod = &required["spec"]["jobTemplate"]["spec"]["template"]["spec"];
    let args: Vec<String> = pod["containers"][0]["args"]
        .as_array()
        .expect("container args")
        .iter()
        .map(|arg| arg.as_str().expect("string arg").to_string())
        .collect();
    let flag = args
        .iter()
        .position(|arg| arg == "--token-file")
        .expect("auth: required must tell the runner where its credential is");
    assert_eq!(args[flag + 1], "/var/run/secrets/lumen.axiom.dev/token");

    // Nothing in the rendered CronJob may be token material. Every JWT starts
    // `ey` — base64 of `{"` — and carries two dots, which is enough to catch a
    // future change that resolves the credential at render time and inlines it.
    for arg in &args {
        assert!(
            !(arg.starts_with("ey") && arg.matches('.').count() == 2),
            "argument is a JWT, not a reference to one: {arg}"
        );
    }

    lumen.spec.auth = AuthMode::Off;
    let objects = render::render(&lumen);
    let open = find(&objects, "CronJob", "search-backup");
    let open_pod = &open["spec"]["jobTemplate"]["spec"]["template"]["spec"];
    let open_args: Vec<&str> = open_pod["containers"][0]["args"]
        .as_array()
        .expect("container args")
        .iter()
        .map(|arg| arg.as_str().expect("string arg"))
        .collect();
    assert!(
        !open_args.contains(&"--token-file"),
        "an open fleet rejects a presented bearer; the runner must send none: {open_args:?}"
    );
    assert_eq!(
        open_pod["volumes"], pod["volumes"],
        "the projection itself is unconditional — one pod shape whatever `auth` says"
    );
}

/// AC6. The projected token is only an improvement if the alternatives are
/// gone. Three of them would each work today and each would be wrong:
///
/// * the GCE metadata server, which hands out a *Google* identity to anything
///   that can reach a link-local address — no audience the fleet asked for and
///   no ServiceAccount the cluster can revoke;
/// * a token passed in the environment, which every child process, crash
///   dump, and `kubectl describe pod` reproduces verbatim;
/// * a Secret read, which turns a control-plane component into a holder of
///   long-lived material and needs the cluster-wide `secrets: get` grant that
///   #2877 removed.
///
/// A reviewer cannot tell from the render tests above that none of these is
/// *also* wired somewhere in the same paths, so this reads the paths.
#[test]
fn the_control_plane_paths_reach_for_no_other_credential() {
    // Assembled from parts so this gate does not match itself if the sweep is
    // ever widened to include `apps/lumen/e2e/`.
    let forbidden: &[(&str, &str)] = &[
        (
            &["metadata", ".google.internal"].concat(),
            "the GCE metadata server issues a Google identity, not an audience-bound KSA token",
        ),
        (
            &["169.254", ".169.254"].concat(),
            "the metadata server's link-local address, reachable by any process in the pod",
        ),
        (
            &["compute", "Metadata"].concat(),
            "the metadata server's required request header",
        ),
        (
            &["LUMEN_BACKUP", "_TOKEN"].concat(),
            "R4: a credential in the environment is reproduced by every child process and by \
             `kubectl describe pod`",
        ),
        (
            &["secret", "KeyRef"].concat(),
            "R4: a Secret-backed env var is the same leak with an extra indirection",
        ),
        (
            &["core::v1::", "Secret"].concat(),
            "a control-plane component that reads Secrets needs the cluster-wide grant this \
             item removed",
        ),
    ];

    let lumen_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut scanned = 0usize;
    let mut files = Vec::new();
    collect_text_files(&lumen_root.join("src/operator"), &mut files);
    collect_text_files(&lumen_root.join("k8s/operator"), &mut files);
    files.push(lumen_root.join("src/backup.rs"));
    files.push(lumen_root.join("src/bin/lumen.rs"));
    files.sort();

    for file in &files {
        let Ok(text) = std::fs::read_to_string(file) else {
            continue;
        };
        scanned += 1;
        let rel = file.strip_prefix(lumen_root).unwrap_or(file).display();
        for (line_no, line) in text.lines().enumerate() {
            for (needle, why) in forbidden {
                assert!(
                    !line.contains(needle),
                    "apps/lumen/{rel}:{}: control-plane paths must present only their own \
                     projected KSA token, but this line names `{needle}` — {why}\n  {}",
                    line_no + 1,
                    line.trim()
                );
            }
        }
    }

    // A sweep that silently reads nothing passes every assertion in it.
    assert!(
        scanned >= 8,
        "expected the operator, backup, and CLI control-plane paths to be readable, \
         but only {scanned} files were scanned"
    );
}

fn collect_text_files(dir: &std::path::Path, out: &mut Vec<std::path::PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_text_files(&path, out);
        } else if matches!(
            path.extension().and_then(|e| e.to_str()),
            Some("rs" | "yaml" | "yml")
        ) {
            out.push(path);
        }
    }
}
// HANDWRITE-END
