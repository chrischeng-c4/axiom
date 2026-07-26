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
        shard_count: 1,
        shard_map: ShardMapSpec::default(),
        replicas_per_shard: 1,
        voter_count: 1,
        log_format: LogFormat::Json,
        log_level: None,
        auth: AuthMode::Off,
        tokens_secret: None,
        tokens_secret_provider_class: None,
        tokens_secret_csi_driver: None,
        serving: ServingSpec::default(),
        reshard_policy: ReshardPolicy::default(),
        observability: false,
        network_policy: false,
        admission: None,
        service_account_name: None,
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

fn verbs_for(role: &serde_yaml::Value, api_group: &str, resource: &str) -> Vec<String> {
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
        .unwrap_or_else(|| panic!("missing RBAC rule for {api_group:?}/{resource}"))["verbs"]
        .as_sequence()
        .expect("rule verbs")
        .iter()
        .map(|verb| verb.as_str().expect("string verb").to_string())
        .collect()
}

#[test]
fn operator_rbac_can_reconcile_cronjobs_and_read_reshard_secrets() {
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

    let secret_verbs = verbs_for(&role, "", "secrets");
    assert_eq!(
        secret_verbs,
        vec!["get"],
        "reshard requires read-only Secret access; mutation is unnecessary"
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
        command.args(["k8s", "operator", "render"]);
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
        .args(["k8s", "operator", "render"])
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
        .args(["k8s", "operator", "render", "--namespace", "lumen-live"])
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
// HANDWRITE-END
