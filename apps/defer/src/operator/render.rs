// HANDWRITE-BEGIN gap="missing-generator:logic:defer-k8s-render" tracker="#766" reason="Defer-owned domain/env wiring over shared StatefulSet, Services, PDB, Secret projection, and CronJob renderers."
use serde_json::{json, Value};
use service_k8s::render::{self, RenderCtx, ServiceStatefulSet, WorkloadVolumeClaim};

use super::crd::Defer;

const APP: &str = "defer";
const MANAGER: &str = "defer-operator";
const API_VERSION: &str = "defer.dev/v1alpha1";
const KIND: &str = "Defer";
const COMPONENT: &str = "server";
const BACKUP_COMPONENT: &str = "backup";
const CLIENT_PORT: i32 = 7141;
const RAFT_PORT: i32 = 7142;
const TOKEN_MOUNT: &str = "/var/run/secrets/defer";
const TOKEN_FILE: &str = "/var/run/secrets/defer/token-registry.json";
const TARGET_MOUNT: &str = "/var/run/secrets/defer-target";
const TARGET_FILE: &str = "/var/run/secrets/defer-target/key";

fn name(defer: &Defer) -> String {
    defer.metadata.name.clone().unwrap_or_else(|| APP.into())
}

fn namespace(defer: &Defer) -> String {
    defer
        .metadata
        .namespace
        .clone()
        .unwrap_or_else(|| "default".into())
}

fn owner(defer: &Defer) -> Option<Value> {
    Some(render::owner_ref(
        API_VERSION,
        KIND,
        defer.metadata.name.as_deref()?,
        defer.metadata.uid.as_deref()?,
    ))
}

fn ctx<'a>(defer: &Defer, name: &'a str, ns: &'a str) -> RenderCtx<'a> {
    RenderCtx {
        app: APP,
        manager: MANAGER,
        api_version: API_VERSION,
        kind: KIND,
        name,
        ns,
        owner: owner(defer),
    }
}

fn token_source(defer: &Defer) -> Option<render::TokenRegistrySource<'_>> {
    if defer.spec.auth != "required" {
        return None;
    }
    if let Some(secret) = defer.spec.tokens_secret.as_deref() {
        return Some(render::TokenRegistrySource::Secret {
            name: secret,
            key: "token-registry.json",
        });
    }
    defer
        .spec
        .tokens_secret_provider_class
        .as_deref()
        .map(|provider_class| render::TokenRegistrySource::Csi {
            provider_class,
            driver: None,
        })
}

pub fn render(defer: &Defer) -> Vec<Value> {
    let name = name(defer);
    let ns = namespace(defer);
    let cx = ctx(defer, &name, &ns);
    let headless = format!("{name}-headless");
    let mut objects = vec![
        render::service_account(&cx, COMPONENT),
        statefulset(defer, &cx, &headless),
        render::headless_service_with_ports(
            &cx,
            &headless,
            COMPONENT,
            vec![
                json!({"name": "http", "port": CLIENT_PORT, "targetPort": "http", "protocol": "TCP"}),
                json!({"name": "raft", "port": RAFT_PORT, "targetPort": "raft", "protocol": "TCP"}),
            ],
        ),
        render::client_service(&cx, &name, COMPONENT, CLIENT_PORT),
        render::pdb(&cx, &name, COMPONENT, 1),
    ];
    if let Some(backup) = backup_cron_job(defer, &cx) {
        objects.push(backup);
    }
    objects
}

fn backup_cron_job(defer: &Defer, cx: &RenderCtx<'_>) -> Option<Value> {
    let backup = defer.spec.backup.as_ref()?;
    let cron_name = format!("{}-backup", cx.name);
    let mut args = vec![
        "backup".into(),
        "--url".into(),
        format!(
            "http://{}.{}.svc.cluster.local:{CLIENT_PORT}",
            cx.name, cx.ns
        ),
        "--dest".into(),
        backup.destination.clone(),
    ];
    if let Some(seconds) = backup.retention_secs {
        args.extend(["--retention-secs".into(), seconds.to_string()]);
    }
    let mut env = Vec::new();
    if let Some(secret) = &backup.admin_token_secret {
        env.push(json!({
            "name": "DEFER_TOKEN",
            "valueFrom": {"secretKeyRef": {"name": secret, "key": "token"}}
        }));
    }
    let pull = defer
        .spec
        .cluster
        .image_pull_policy
        .as_deref()
        .unwrap_or("IfNotPresent");
    Some(render::cron_job(render::CronJob {
        cx,
        name: &cron_name,
        component: BACKUP_COMPONENT,
        schedule: &backup.schedule,
        image: &defer.spec.cluster.image,
        image_pull_policy: pull,
        command: vec!["defer".into()],
        args,
        env,
        env_from: vec![],
        volumes: vec![],
        volume_mounts: vec![],
        service_account_name: Some(cx.name),
        cpu: "100m",
        memory: "128Mi",
        successful_jobs_history_limit: 3,
        failed_jobs_history_limit: 3,
    }))
}

fn statefulset(defer: &Defer, cx: &RenderCtx<'_>, headless: &str) -> Value {
    let spec = &defer.spec;
    let mut pvc = json!({
        "metadata": {"name": "data", "labels": cx.labels(COMPONENT)},
        "spec": {
            "accessModes": ["ReadWriteOnce"],
            "resources": {"requests": {"storage": spec.storage}}
        }
    });
    if let Some(class) = &spec.storage_class {
        pvc["spec"]["storageClassName"] = json!(class);
    }

    let mut env = vec![
        json!({"name": "DEFER_BIND", "value": format!("0.0.0.0:{CLIENT_PORT}")}),
        json!({"name": "DEFER_RAFT_PORT", "value": RAFT_PORT.to_string()}),
        json!({"name": "DEFER_DATA_DIR", "value": "/data"}),
        json!({"name": "DEFER_GRACE_SECS", "value": spec.grace_secs.to_string()}),
        json!({"name": "DEFER_LOG_FORMAT", "value": "json"}),
    ];
    if let Some(level) = &spec.log_level {
        env.push(json!({"name": "RUST_LOG", "value": level}));
    }
    if token_source(defer).is_some() {
        env.push(json!({"name": "DEFER_AUTH", "value": "required"}));
        env.push(json!({"name": "DEFER_TOKEN_REGISTRY_FILE", "value": TOKEN_FILE}));
    }
    if let Some(uri) = &spec.bootstrap_seed_uri {
        env.push(json!({"name": "DEFER_BOOTSTRAP_SEED_URI", "value": uri}));
    }

    let mut volumes = vec![json!({"name": "tmp", "emptyDir": {}})];
    let mut mounts = vec![json!({"name": "tmp", "mountPath": "/tmp"})];
    if let Some(source) = token_source(defer) {
        let projection = render::TokenRegistryProjection {
            volume_name: "defer-token-registry",
            mount_path: TOKEN_MOUNT,
            source,
        };
        volumes.push(render::token_registry_volume(&projection));
        mounts.push(render::token_registry_mount(&projection));
    }
    if let (Some(secret), Some(key_id)) = (
        spec.target_signing_secret.as_deref(),
        spec.target_signing_key_id.as_deref(),
    ) {
        volumes.push(json!({
            "name": "defer-target-signing",
            "secret": {"secretName": secret, "items": [{"key": "key", "path": "key"}]}
        }));
        mounts.push(json!({
            "name": "defer-target-signing", "mountPath": TARGET_MOUNT, "readOnly": true
        }));
        env.push(json!({"name": "DEFER_TARGET_SIGNING_KEY_ID", "value": key_id}));
        env.push(json!({"name": "DEFER_TARGET_SIGNING_SECRET_FILE", "value": TARGET_FILE}));
    }
    if let Some(secret) = spec.peer_tls_secret.as_deref() {
        volumes.push(json!({
            "name": "defer-peer-tls",
            "secret": {"secretName": secret, "items": [
                {"key": "tls.crt", "path": "tls.crt"},
                {"key": "tls.key", "path": "tls.key"},
                {"key": "ca.crt", "path": "ca.crt"}
            ]}
        }));
        mounts.push(json!({
            "name": "defer-peer-tls", "mountPath": "/var/run/secrets/defer-peer", "readOnly": true
        }));
        env.extend([
            json!({"name": "DEFER_PEER_MTLS", "value": "on"}),
            json!({"name": "DEFER_PEER_TLS_CERT", "value": "/var/run/secrets/defer-peer/tls.crt"}),
            json!({"name": "DEFER_PEER_TLS_KEY", "value": "/var/run/secrets/defer-peer/tls.key"}),
            json!({"name": "DEFER_PEER_TLS_CA", "value": "/var/run/secrets/defer-peer/ca.crt"}),
        ]);
    }

    render::service_statefulset(ServiceStatefulSet {
        cx,
        name: cx.name,
        component: COMPONENT,
        image: &spec.cluster.image,
        image_pull_policy: spec
            .cluster
            .image_pull_policy
            .as_deref()
            .unwrap_or("IfNotPresent"),
        command: vec!["defer".into(), "serve".into()],
        args: vec![],
        ports: vec![
            json!({"name": "http", "containerPort": CLIENT_PORT, "protocol": "TCP"}),
            json!({"name": "raft", "containerPort": RAFT_PORT, "protocol": "TCP"}),
        ],
        headless_service: headless,
        // Defer currently exposes one Raft group per instance. Replica count
        // is the HA knob; shard placement becomes a separate router slice.
        shard_count: 1,
        replicas_per_shard: spec.cluster.replicas_per_shard,
        voter_count: spec.cluster.voter_count,
        headless_env_key: "DEFER_PEER_SERVICE",
        service_account_name: Some(cx.name),
        env,
        env_from: vec![],
        resources: render::requested_resources(
            &spec.cluster.resources.cpu,
            &spec.cluster.resources.memory,
        ),
        pod_annotations: Some(json!({
            "prometheus.io/scrape": "true",
            "prometheus.io/port": CLIENT_PORT.to_string(),
            "prometheus.io/path": "/metrics"
        })),
        pod_security_context: Some(render::restricted_pod_security_context()),
        container_security_context: Some(render::restricted_container_security_context()),
        termination_grace_period_seconds: Some(spec.grace_secs),
        readiness_probe: Some(
            json!({"httpGet": {"path": "/readyz", "port": "http"}, "periodSeconds": 5}),
        ),
        liveness_probe: Some(
            json!({"httpGet": {"path": "/healthz", "port": "http"}, "periodSeconds": 15}),
        ),
        startup_probe: Some(
            json!({"httpGet": {"path": "/healthz", "port": "http"}, "periodSeconds": 5, "failureThreshold": 120}),
        ),
        volumes,
        volume_mounts: mounts,
        affinity: Some(render::dedicated_node_affinity(cx.selector(COMPONENT))),
        topology_spread_constraints: vec![],
        revision_history_limit: Some(5),
        update_strategy: Some(json!({"type": "RollingUpdate"})),
        volume_claim: Some(WorkloadVolumeClaim {
            name: "data".into(),
            template: pvc,
            mount_path: "/data",
            read_only: false,
        }),
    })
}
// HANDWRITE-END
