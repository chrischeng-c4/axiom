//! #4025: Managed embedded index data must reach the raft PVC rather than an
//! image-declared nested volume.
//!
//! This is a public render contract. It builds a `Lumen` custom resource and
//! observes the rendered StatefulSet. It does not call the reconciler or
//! inspect renderer implementation state.

use std::collections::BTreeMap;

use kube::api::ObjectMeta;
use lumen::operator::crd::{
    AuthMode, LogFormat, PlacementSpec, ReshardPolicy, ServingSpec, ShardMapSpec,
};
use lumen::operator::render::render;
use lumen::operator::{Lumen, LumenSpec};
use serde_json::{json, Value};

const RAFT_PARENT: &str = "/var/lib/lumen";
const EMBEDDED_DATA_DIR: &str = "/var/lib/lumen/data";

fn fixture(replicas_per_shard: u32) -> Lumen {
    let voter_count = if replicas_per_shard <= 1 {
        1
    } else {
        replicas_per_shard
    };
    let spec = LumenSpec {
        image: "registry.example.invalid/lumen:test".into(),
        image_pull_policy: None,
        placement: PlacementSpec::default(),
        shard_count: 1,
        shard_map: ShardMapSpec::default(),
        replicas_per_shard,
        voter_count,
        log_format: LogFormat::Pretty,
        log_level: None,
        auth: AuthMode::Off,
        serving: ServingSpec::default(),
        reshard_policy: ReshardPolicy::default(),
        observability: false,
        network_policy: false,
        admission: None,
        service_account_name: None,
        service_account_annotations: BTreeMap::new(),
        peer_tls_secret: None,
        serving_tls_secret: None,
        body_limit_bytes: None,
    };
    let mut lumen = Lumen::new("search", spec);
    lumen.metadata = ObjectMeta {
        name: Some("search".into()),
        namespace: Some("contract".into()),
        uid: Some("report-4018-app".into()),
        ..Default::default()
    };
    lumen
}

fn serving_container(lumen: &Lumen) -> Value {
    render(lumen)
        .into_iter()
        .find(|object| object["kind"] == "StatefulSet" && object["metadata"]["name"] == "search")
        .unwrap_or_else(|| panic!("rendered objects have no serving StatefulSet"))["spec"]
        ["template"]["spec"]["containers"][0]
        .clone()
}

fn mounts(container: &Value) -> &[Value] {
    container["volumeMounts"]
        .as_array()
        .expect("serving container volumeMounts")
}

fn env_value<'a>(container: &'a Value, name: &str) -> Option<&'a str> {
    container["env"]
        .as_array()
        .expect("serving container env")
        .iter()
        .find(|entry| entry["name"] == name)
        .and_then(|entry| entry["value"].as_str())
}

#[test]
fn embedded_render_mounts_the_exact_pvc_data_subpath_and_preserves_the_parent() {
    let container = serving_container(&fixture(1));
    let rendered_mounts = mounts(&container);
    let parent = json!({
        "name": "raft",
        "mountPath": RAFT_PARENT,
        "readOnly": false,
    });
    let embedded_data = json!({
        "name": "raft",
        "mountPath": EMBEDDED_DATA_DIR,
        "subPath": "data",
        "readOnly": false,
    });
    let raft_mounts: Vec<&Value> = rendered_mounts
        .iter()
        .filter(|mount| mount["name"] == "raft")
        .collect();

    assert!(
        rendered_mounts.iter().any(|mount| mount == &parent),
        "embedded mode must preserve the raft PVC parent mount {parent}; got {rendered_mounts:#?}"
    );
    assert!(
        rendered_mounts.iter().any(|mount| mount == &embedded_data),
        "embedded mode must mount raft/data exactly at {EMBEDDED_DATA_DIR}; got {rendered_mounts:#?}"
    );
    assert_eq!(
        raft_mounts,
        vec![&parent, &embedded_data],
        "embedded mode must order its raft parent mount before the exact data subpath: {rendered_mounts:#?}"
    );
    assert_eq!(
        raft_mounts
            .last()
            .expect("embedded data child mount")
            .as_object()
            .expect("embedded data child mount object")
            .len(),
        4,
        "the child mount contract has only name, mountPath, subPath, and readOnly"
    );
    assert_eq!(
        env_value(&container, "LUMEN_DATA_DIR"),
        Some(EMBEDDED_DATA_DIR)
    );
    assert_eq!(env_value(&container, "LUMEN_PERSISTENCE"), Some("segment"));
}

#[test]
fn raft_ha_render_keeps_embedded_data_mount_and_env_absent() {
    let container = serving_container(&fixture(3));
    let rendered_mounts = mounts(&container);
    let parent = json!({
        "name": "raft",
        "mountPath": RAFT_PARENT,
        "readOnly": false,
    });

    assert!(
        rendered_mounts.iter().any(|mount| mount == &parent),
        "raft HA must retain its raft PVC parent mount: {rendered_mounts:#?}"
    );
    assert_eq!(
        rendered_mounts
            .iter()
            .filter(|mount| mount["name"] == "raft")
            .count(),
        1,
        "raft HA must not add the embedded data subpath: {rendered_mounts:#?}"
    );
    assert!(
        rendered_mounts
            .iter()
            .all(|mount| mount["mountPath"] != EMBEDDED_DATA_DIR),
        "raft HA must not mount the embedded data path: {rendered_mounts:#?}"
    );
    assert_eq!(env_value(&container, "LUMEN_DATA_DIR"), None);
    assert_eq!(env_value(&container, "LUMEN_PERSISTENCE"), None);
}
