//! Full StatefulSet compatibility oracle for the shared instance adapter.
//!
//! `service-k8s` can only render the StatefulSet portion of Lumen's object
//! sets. The four cases below cover that portion for the historical single,
//! external-service, replicated-placement, and Fleet-child profiles. The
//! Lumen `operator_render` target remains the full object-set equality gate for
//! NetworkPolicy, observability, backup, and Fleet materialization.

use serde_json::{json, Value};
use service_k8s::render::{
    dedicated_node_affinity, service_statefulset, RenderCtx, ServiceStatefulSet,
    WorkloadVolumeClaim,
};

fn merge_labels(target: &mut Value, labels: &Value) {
    if !target.is_object() {
        *target = json!({});
    }
    let target = target.as_object_mut().expect("labels object");
    for (key, value) in labels.as_object().expect("labels object") {
        target.entry(key.clone()).or_insert_with(|| value.clone());
    }
}

fn template_metadata(mut template: Value, name: &str, labels: &Value) -> Value {
    if !template.is_object() {
        template = json!({});
    }
    let object = template.as_object_mut().expect("template object");
    let metadata = object.entry("metadata").or_insert_with(|| json!({}));
    if !metadata.is_object() {
        *metadata = json!({});
    }
    let metadata = metadata.as_object_mut().expect("metadata object");
    metadata.insert("name".into(), json!(name));
    let labels_value = metadata.entry("labels").or_insert_with(|| json!({}));
    merge_labels(labels_value, labels);
    template
}

/// This is the exact pre-#3925 root renderer. It is test-only, so production
/// has one renderer: the shared stateful-instance primitive.
fn baseline(p: ServiceStatefulSet<'_>) -> Value {
    let ServiceStatefulSet {
        cx,
        name,
        component,
        image,
        image_pull_policy,
        command,
        args,
        ports,
        headless_service,
        shard_count,
        replicas_per_shard,
        voter_count,
        headless_env_key,
        service_account_name,
        env: extra_env,
        env_from,
        resources,
        pod_annotations,
        pod_security_context,
        container_security_context,
        termination_grace_period_seconds,
        readiness_probe,
        liveness_probe,
        startup_probe,
        lifecycle,
        volumes,
        volume_mounts,
        affinity,
        node_selector,
        tolerations,
        topology_spread_constraints,
        revision_history_limit,
        update_strategy,
        volume_claim,
    } = p;
    let mut env = vec![
        json!({ "name": "POD_NAME", "valueFrom": { "fieldRef": { "fieldPath": "metadata.name" } } }),
        json!({ "name": "POD_NAMESPACE", "valueFrom": { "fieldRef": { "fieldPath": "metadata.namespace" } } }),
        json!({ "name": "SHARD_COUNT", "value": shard_count.to_string() }),
        json!({ "name": "REPLICAS_PER_SHARD", "value": replicas_per_shard.to_string() }),
        json!({ "name": "VOTER_COUNT", "value": voter_count.to_string() }),
        json!({ "name": headless_env_key, "value": headless_service }),
    ];
    env.extend(extra_env);
    let mut container = json!({
        "name": component, "image": image, "imagePullPolicy": image_pull_policy,
        "command": command, "ports": ports, "env": env, "resources": resources,
    });
    if !args.is_empty() {
        container["args"] = json!(args);
    }
    if !env_from.is_empty() {
        container["envFrom"] = json!(env_from);
    }
    if let Some(value) = readiness_probe {
        container["readinessProbe"] = value;
    }
    if let Some(value) = liveness_probe {
        container["livenessProbe"] = value;
    }
    if let Some(value) = startup_probe {
        container["startupProbe"] = value;
    }
    if let Some(value) = lifecycle {
        container["lifecycle"] = value;
    }
    if let Some(value) = container_security_context {
        container["securityContext"] = value;
    }
    let mut mounts = volume_mounts;
    let mut claims = Vec::new();
    if let Some(claim) = volume_claim {
        let claim_name = claim.name;
        mounts.push(json!({ "name": claim_name.clone(), "mountPath": claim.mount_path, "readOnly": claim.read_only }));
        claims.push(template_metadata(
            claim.template,
            &claim_name,
            &cx.labels(component),
        ));
    }
    if !mounts.is_empty() {
        container["volumeMounts"] = json!(mounts);
    }
    let mut pod_metadata = json!({ "labels": cx.labels(component) });
    if let Some(value) = pod_annotations {
        pod_metadata["annotations"] = value;
    }
    let mut pod_spec = json!({ "containers": [container] });
    if let Some(value) = service_account_name {
        pod_spec["serviceAccountName"] = json!(value);
    }
    if let Some(value) = termination_grace_period_seconds {
        pod_spec["terminationGracePeriodSeconds"] = json!(value);
    }
    if let Some(value) = pod_security_context {
        pod_spec["securityContext"] = value;
    }
    if !volumes.is_empty() {
        pod_spec["volumes"] = json!(volumes);
    }
    if let Some(value) = affinity {
        pod_spec["affinity"] = value;
    }
    if let Some(value) = node_selector {
        pod_spec["nodeSelector"] = value;
    }
    if !tolerations.is_empty() {
        pod_spec["tolerations"] = json!(tolerations);
    }
    if !topology_spread_constraints.is_empty() {
        pod_spec["topologySpreadConstraints"] = json!(topology_spread_constraints);
    }
    let mut spec = json!({
        "replicas": shard_count * replicas_per_shard, "serviceName": headless_service,
        "podManagementPolicy": "Parallel", "selector": { "matchLabels": cx.selector(component) },
        "template": { "metadata": pod_metadata, "spec": pod_spec },
    });
    if let Some(value) = revision_history_limit {
        spec["revisionHistoryLimit"] = json!(value);
    }
    if let Some(value) = update_strategy {
        spec["updateStrategy"] = value;
    }
    if !claims.is_empty() {
        spec["volumeClaimTemplates"] = json!(claims);
    }
    json!({ "apiVersion": "apps/v1", "kind": "StatefulSet", "metadata": cx.meta(name, component), "spec": spec })
}

fn profile<'a>(cx: &'a RenderCtx<'a>, kind: u8) -> ServiceStatefulSet<'a> {
    let (name, replicas, voter_count) = match kind {
        3 => ("fleet-child-serving", 1, 1),
        2 => ("replicated", 3, 3),
        _ => ("lumen", 1, 1),
    };
    let claim = (kind == 2 || kind == 3).then(|| WorkloadVolumeClaim {
        name: "data".into(),
        template: json!({"spec":{"resources":{"requests":{"storage":"20Gi"}}}}),
        mount_path: "/data",
        read_only: false,
    });
    ServiceStatefulSet {
        cx,
        name,
        component: "serving",
        image: "lumen:test",
        image_pull_policy: "IfNotPresent",
        command: vec!["lumen".into(), "serve".into()],
        args: vec![],
        ports: vec![json!({"name":"http","containerPort":7373,"protocol":"TCP"})],
        headless_service: "lumen-headless",
        shard_count: 1,
        replicas_per_shard: replicas,
        voter_count,
        headless_env_key: "LUMEN_HEADLESS_SERVICE",
        service_account_name: (kind == 1).then_some("external-sa"),
        env: if kind == 2 {
            vec![json!({"name":"LUMEN_PEER_MTLS","value":"on"})]
        } else {
            vec![]
        },
        env_from: vec![],
        resources: json!({"requests":{"cpu":"1","memory":"4Gi"}}),
        pod_annotations: (kind == 1).then(|| json!({"prometheus.io/scrape":"true"})),
        pod_security_context: None,
        container_security_context: None,
        termination_grace_period_seconds: None,
        readiness_probe: None,
        liveness_probe: None,
        startup_probe: None,
        lifecycle: None,
        volumes: if kind == 2 {
            vec![json!({"name":"peer-tls","secret":{"secretName":"peer-tls"}})]
        } else {
            vec![]
        },
        volume_mounts: if kind == 2 {
            vec![json!({"name":"peer-tls","mountPath":"/var/run/tls","readOnly":true})]
        } else {
            vec![]
        },
        affinity: (kind == 2).then(|| dedicated_node_affinity(cx.selector("serving"))),
        node_selector: (kind == 2).then(|| json!({"lumen.axiom.dev/capacity-profile":"high"})),
        tolerations: if kind == 2 {
            vec![
                json!({"key":"dedicated","operator":"Equal","value":"lumen","effect":"NoSchedule"}),
            ]
        } else {
            vec![]
        },
        topology_spread_constraints: if kind == 2 {
            vec![
                json!({"maxSkew":1,"topologyKey":"topology.kubernetes.io/zone","whenUnsatisfiable":"ScheduleAnyway","labelSelector":{"matchLabels":cx.selector("serving")}}),
            ]
        } else {
            vec![]
        },
        revision_history_limit: (kind == 2).then_some(3),
        update_strategy: (kind == 2).then(|| json!({"type":"RollingUpdate"})),
        volume_claim: claim,
    }
}

#[test]
fn compatibility_adapter_matches_all_lumen_statefulset_profiles() {
    let cx = RenderCtx {
        app: "lumen",
        manager: "lumen-operator",
        api_version: "lumen.axiom.dev/v1",
        kind: "Lumen",
        name: "fleet-child",
        ns: "acme",
        owner: Some(json!({"uid":"owner"})),
    };
    for kind in 0..4 {
        let expected = baseline(profile(&cx, kind));
        let actual = service_statefulset(profile(&cx, kind));
        assert_eq!(
            actual, expected,
            "profile {kind} changed its complete Value"
        );
        assert_eq!(
            serde_json::to_string(&actual).unwrap(),
            serde_json::to_string(&expected).unwrap(),
            "profile {kind} changed object order"
        );
    }
}
