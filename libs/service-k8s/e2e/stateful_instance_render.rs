use serde_json::json;
use service_k8s::render::common::ServicePodTemplate;
use service_k8s::render::stateful_instance::{
    stateful_instance, ExistingClaim, StatefulInstanceError, StatefulInstancePlan,
    StatefulStorageAttachment, VolumeClaimTemplate,
};
use service_k8s::render::RenderCtx;

fn pod<'a>(cx: &'a RenderCtx<'a>) -> ServicePodTemplate<'a> {
    ServicePodTemplate {
        cx,
        component: "serving",
        image: "lumen",
        image_pull_policy: "IfNotPresent",
        command: vec!["lumen".into()],
        args: vec![],
        ports: vec![],
        env: vec![],
        env_from: vec![],
        resources: json!({}),
        readiness_probe: None,
        liveness_probe: None,
        startup_probe: None,
        lifecycle: None,
        container_security_context: None,
        pod_security_context: None,
        service_account_name: None,
        termination_grace_period_seconds: None,
        volumes: vec![],
        volume_mounts: vec![],
        pod_annotations: None,
        topology_spread_constraints: vec![],
    }
}

fn plan<'a>(cx: &'a RenderCtx<'a>, storage: StatefulStorageAttachment) -> StatefulInstancePlan<'a> {
    StatefulInstancePlan::new(cx, "lumen", 1, pod(cx), storage)
}

fn plan_without_storage<'a>(cx: &'a RenderCtx<'a>) -> StatefulInstancePlan<'a> {
    StatefulInstancePlan::without_storage(cx, "lumen", 1, pod(cx))
}

fn template_claim(name: &str, mount: &str) -> VolumeClaimTemplate {
    VolumeClaimTemplate::new(
        name,
        json!({"spec": {"resources": {"requests": {"storage": "20Gi"}}}}),
        mount,
    )
}

fn existing_claim(name: &str, mount: &str) -> ExistingClaim {
    ExistingClaim::new(
        name,
        format!("{name}-existing"),
        json!({"spec": {"resources": {"requests": {"storage": "20Gi"}}}}),
        mount,
    )
}

#[test]
fn both_storage_shapes_and_identity_are_complete() {
    let cx = RenderCtx {
        app: "lumen",
        manager: "test",
        api_version: "v1",
        kind: "Lumen",
        name: "lumen",
        ns: "lumen",
        owner: Some(json!({"uid":"u"})),
    };
    let template = stateful_instance(plan(
        &cx,
        StatefulStorageAttachment::VolumeClaimTemplate(template_claim("data", "/data")),
    ))
    .unwrap();
    assert!(template.storage.is_none());
    assert_eq!(
        template.workload["spec"]["volumeClaimTemplates"][0]["metadata"]["name"],
        "data"
    );
    assert_eq!(
        template.workload["metadata"]["ownerReferences"][0]["uid"],
        "u"
    );
    let existing = stateful_instance(plan(
        &cx,
        StatefulStorageAttachment::ExistingClaim(existing_claim("data", "/data")),
    ))
    .unwrap();
    assert!(existing.storage.is_some());
    assert!(existing.workload["spec"]["volumeClaimTemplates"].is_null());
}

#[test]
fn collisions_and_zero_replicas_fail_closed() {
    let cx = RenderCtx {
        app: "lumen",
        manager: "test",
        api_version: "v1",
        kind: "Lumen",
        name: "lumen",
        ns: "lumen",
        owner: None,
    };
    let mut p = plan(
        &cx,
        StatefulStorageAttachment::VolumeClaimTemplate(template_claim("data", "/data")),
    );
    p.replicas = 0;
    assert_eq!(
        stateful_instance(p),
        Err(StatefulInstanceError::ZeroReplicas)
    );
    let mut p = plan(
        &cx,
        StatefulStorageAttachment::VolumeClaimTemplate(template_claim("data", "/data")),
    );
    p.pod
        .volume_mounts
        .push(json!({"name":"other","mountPath":"/data"}));
    assert_eq!(
        stateful_instance(p),
        Err(StatefulInstanceError::VolumeMountCollision)
    );
}

#[test]
fn no_storage_has_no_claim_template_mount_or_volume() {
    let cx = RenderCtx {
        app: "lumen",
        manager: "test",
        api_version: "v1",
        kind: "Lumen",
        name: "lumen",
        ns: "lumen",
        owner: None,
    };
    let rendered = stateful_instance(plan_without_storage(&cx)).unwrap();
    assert!(rendered.storage.is_none());
    assert!(rendered.workload["spec"]["volumeClaimTemplates"].is_null());
    assert!(rendered.workload["spec"]["template"]["spec"]["volumes"].is_null());
    assert!(
        rendered.workload["spec"]["template"]["spec"]["containers"][0]["volumeMounts"].is_null()
    );
}

#[test]
fn selector_extras_follow_the_pod_label_and_core_divergence_fails() {
    let cx = RenderCtx {
        app: "lumen",
        manager: "test",
        api_version: "v1",
        kind: "Lumen",
        name: "lumen",
        ns: "lumen",
        owner: None,
    };
    let mut normalized = plan_without_storage(&cx);
    normalized
        .selector
        .insert("app.kubernetes.io/managed-by".into(), "test".into());
    normalized.selector.insert("tier".into(), "data".into());
    let rendered = stateful_instance(normalized).unwrap();
    assert_eq!(
        rendered.workload["spec"]["selector"]["matchLabels"]["tier"],
        "data"
    );
    assert_eq!(
        rendered.workload["spec"]["template"]["metadata"]["labels"]["tier"],
        "data"
    );
    assert!(
        rendered.workload["spec"]["selector"]["matchLabels"]["app.kubernetes.io/managed-by"]
            .is_null()
    );

    let mut divergent = plan_without_storage(&cx);
    divergent
        .selector
        .insert("app.kubernetes.io/name".into(), "other".into());
    assert_eq!(
        stateful_instance(divergent),
        Err(StatefulInstanceError::SelectorCoreIdentityOverride)
    );
}

#[test]
fn same_pvc_direct_child_mount_is_exact_and_ordered() {
    let cx = RenderCtx {
        app: "lumen",
        manager: "test",
        api_version: "v1",
        kind: "Lumen",
        name: "lumen",
        ns: "lumen",
        owner: None,
    };
    let mut p = plan(
        &cx,
        StatefulStorageAttachment::VolumeClaimTemplate(template_claim("raft", "/var/lib/lumen")),
    );
    p.pod
        .volume_mounts
        .push(json!({"name":"config","mountPath":"/etc/lumen","readOnly":true}));
    p.pod.volume_mounts.push(json!({
        "name":"raft",
        "mountPath":"/var/lib/lumen/data",
        "subPath":"data",
        "readOnly":false,
    }));

    let rendered = stateful_instance(p).expect("exact direct child mount is safe");
    assert_eq!(
        rendered.workload["spec"]["template"]["spec"]["containers"][0]["volumeMounts"],
        json!([
            {"name":"config","mountPath":"/etc/lumen","readOnly":true},
            {"name":"raft","mountPath":"/var/lib/lumen","readOnly":false},
            {"name":"raft","mountPath":"/var/lib/lumen/data","subPath":"data","readOnly":false},
        ])
    );
}

#[test]
fn same_pvc_child_mount_rejects_every_other_overlap_fail_closed() {
    let cx = RenderCtx {
        app: "lumen",
        manager: "test",
        api_version: "v1",
        kind: "Lumen",
        name: "lumen",
        ns: "lumen",
        owner: None,
    };
    let unsafe_children = vec![
        json!({"name":"raft","mountPath":"/var/lib/lumen/.","subPath":".","readOnly":false}),
        json!({"name":"raft","mountPath":"/var/lib/lumen/..","subPath":"..","readOnly":false}),
        json!({"name":"raft","mountPath":"/var/lib/lumen/data/child","subPath":"data/child","readOnly":false}),
        json!({"name":"raft","mountPath":"/var/lib/lumen/data","subPath":"data","readOnly":false,"mountPropagation":"None"}),
        json!({"name":"raft","mountPath":"/var/lib/lumen/data","subPathExpr":"data","readOnly":false}),
        json!({"name":"raft","mountPath":"/var/lib/lumen/data","subPath":"data","readOnly":true}),
        json!({"name":"raft","mountPath":"/var/lib/lumen/./data","subPath":"data","readOnly":false}),
        json!({"name":"raft","mountPath":"/var/lib/lumen/data/cache","subPath":"cache","readOnly":false}),
    ];

    for mount in unsafe_children {
        let mut p = plan(
            &cx,
            StatefulStorageAttachment::VolumeClaimTemplate(template_claim(
                "raft",
                "/var/lib/lumen",
            )),
        );
        p.pod.volume_mounts.push(mount);
        assert_eq!(
            stateful_instance(p),
            Err(StatefulInstanceError::VolumeMountCollision),
            "unsafe same-PVC mount must fail closed"
        );
    }

    let mut p = plan(
        &cx,
        StatefulStorageAttachment::VolumeClaimTemplate(template_claim("raft", "/var/lib/lumen")),
    );
    p.pod.volume_mounts.extend([
        json!({"name":"raft","mountPath":"/var/lib/lumen/data","subPath":"data","readOnly":false}),
        json!({"name":"raft","mountPath":"/var/lib/lumen/aof","subPath":"aof","readOnly":false}),
    ]);
    assert_eq!(
        stateful_instance(p),
        Err(StatefulInstanceError::VolumeMountCollision),
        "only one direct child is allowed"
    );
}
