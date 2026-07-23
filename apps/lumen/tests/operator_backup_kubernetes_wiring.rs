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
        admission: None,
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
    assert!(default_yaml.contains("image: lumen:latest"));

    let immutable = "asia-east1-docker.pkg.dev/axiom/lumen/lumen@sha256:0123456789abcdef";
    let immutable_yaml = render(&["--namespace", "lumen-live", "--image", immutable]);
    assert!(immutable_yaml.contains("namespace: lumen-live"));
    assert!(immutable_yaml.contains(&format!("image: {immutable}")));
    assert!(!immutable_yaml.contains("image: lumen:latest"));

    let invalid = Command::new(env!("CARGO_BIN_EXE_lumen"))
        .args(["k8s", "operator", "render", "--image", "bad\nimage"])
        .output()
        .expect("run lumen operator render with invalid image");
    assert!(!invalid.status.success());
    assert!(String::from_utf8_lossy(&invalid.stderr).contains("whitespace-free OCI image"));
}
// HANDWRITE-END
