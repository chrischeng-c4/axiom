//! The projection contract for identity-based auth (#2764) — the oracle for the
//! operator-render slice, written by the supervisor and frozen for the duration.
//!
//! Every spec here is built with `serde_json::from_value` rather than a struct
//! literal. That is deliberate and load-bearing: a struct literal names every
//! field, so it stops compiling the moment the struct changes, and an oracle
//! that has to be edited to keep building is not an oracle — whoever edits it
//! decides what green means. Going through the CRD's own deserializer also
//! tests the thing an operator actually applies.
//!
//! What this file does NOT assert is as deliberate. There is no assertion about
//! a control-plane identity env var, because no such contract exists in the
//! source yet; inventing one here would specify a shape rather than a
//! requirement, which is how the last three specifications in this chain went
//! wrong.

#![cfg(feature = "operator")]

use kube::api::ObjectMeta;
use lumen::operator::render::render;
use lumen::operator::{Lumen, LumenSpec};
use serde_json::{json, Value};

/// Build a `Lumen` from the YAML an operator would apply, not from a struct
/// literal. `spec` is merged over a minimal valid base.
fn lumen_from(spec_overrides: Value) -> Lumen {
    let mut spec = json!({
        "image": "lumen:test",
        "shardCount": 1,
        "replicasPerShard": 1,
        "voterCount": 1,
    });
    let base = spec.as_object_mut().unwrap();
    for (k, v) in spec_overrides.as_object().unwrap() {
        base.insert(k.clone(), v.clone());
    }
    let spec: LumenSpec = serde_json::from_value(spec)
        .unwrap_or_else(|e| panic!("spec does not deserialize: {e}"));
    let mut l = Lumen::new("acct", spec);
    l.metadata = ObjectMeta {
        name: Some("acct".into()),
        namespace: Some("acme".into()),
        uid: Some("uid-1234".into()),
        generation: Some(1),
        ..Default::default()
    };
    l
}

fn objects(l: &Lumen) -> Vec<Value> {
    render(l)
}

fn kinds(objs: &[Value]) -> Vec<String> {
    objs.iter()
        .map(|o| {
            format!(
                "{}/{}",
                o["kind"].as_str().unwrap_or("?"),
                o["metadata"]["name"].as_str().unwrap_or("?")
            )
        })
        .collect()
}

fn find<'a>(objs: &'a [Value], kind: &str, name: &str) -> &'a Value {
    objs.iter()
        .find(|o| o["kind"] == kind && o["metadata"]["name"] == name)
        .unwrap_or_else(|| panic!("missing {kind}/{name}; rendered: {:?}", kinds(objs)))
}

/// The serving workload, whichever kind this topology renders. Single-member
/// and raft topologies differ (StatefulSet vs Deployment) and that choice is
/// not what this file is about — pinning one kind here would fail the oracle
/// for a reason unrelated to auth.
fn workload(objs: &[Value]) -> &Value {
    objs.iter()
        .find(|o| (o["kind"] == "StatefulSet" || o["kind"] == "Deployment") && o["metadata"]["name"] == "acct")
        .unwrap_or_else(|| panic!("no serving workload named acct; rendered: {:?}", kinds(objs)))
}

/// The serving container of that workload.
fn serving_container(objs: &[Value]) -> &Value {
    &workload(objs)["spec"]["template"]["spec"]["containers"][0]
}

fn env_value<'a>(container: &'a Value, name: &str) -> Option<&'a Value> {
    container["env"]
        .as_array()?
        .iter()
        .find(|e| e["name"] == name)
}

fn pod_volumes(objs: &[Value]) -> Vec<Value> {
    workload(objs)["spec"]["template"]["spec"]["volumes"]
        .as_array()
        .cloned()
        .unwrap_or_default()
}

/// The two identity fields, as a caller would write them.
fn identity_spec() -> Value {
    json!({
        "auth": "required",
        "identities": {
            "reader@proj.iam.gserviceaccount.com": {
                "subject": "fastapi-reader",
                "roles": { "docs": "read" }
            },
            "writer@proj.iam.gserviceaccount.com": {
                "subject": "fastapi-writer",
                "roles": { "*": "write" }
            }
        },
        "identityAudiences": ["https://lumen.acme.internal"]
    })
}

/// Identity grants become an instance-owned ConfigMap holding `identities.json`.
///
/// A ConfigMap and not a Secret: the map key is a service-account email, which
/// is an identifier and not a credential. That distinction is the whole of
/// #2764, and it has to be visible in the object kind, because the object kind
/// is what an operator's RBAC, backup tooling, and audit policy key on.
#[test]
fn identity_grants_render_an_instance_owned_configmap() {
    let objs = objects(&lumen_from(identity_spec()));
    let cm = find(&objs, "ConfigMap", "acct-identities");

    let raw = cm["data"]["identities.json"]
        .as_str()
        .unwrap_or_else(|| panic!("no identities.json key; data = {}", cm["data"]));
    let parsed: Value = serde_json::from_str(raw).expect("identities.json is valid JSON");

    let ids = &parsed["identities"];
    assert!(
        ids.is_object(),
        "identities.json must nest under an `identities` key so the loader can \
         union it with token-registry.json without either namespace capturing \
         the other; got: {parsed}"
    );
    assert_eq!(ids["reader@proj.iam.gserviceaccount.com"]["subject"], "fastapi-reader");
    assert_eq!(ids["reader@proj.iam.gserviceaccount.com"]["roles"]["docs"], "read");
    assert_eq!(ids["writer@proj.iam.gserviceaccount.com"]["roles"]["*"], "write");

    // No bearer secret may reach this object. If a future change routes token
    // material through here, the confidentiality class of the mount silently
    // changes and nothing else in the system would notice.
    assert!(
        !raw.contains("\"tokens\""),
        "identities.json must not carry a tokens namespace: {raw}"
    );
}

/// The file has to be mounted and named to the process, or the ConfigMap is
/// decoration. Asserting the env var alone would pass on a pod that never
/// mounts it — the path in the env var must resolve to a real volume.
#[test]
fn the_identities_file_is_mounted_and_named_to_the_process() {
    let objs = objects(&lumen_from(identity_spec()));
    let c = serving_container(&objs);

    let path = env_value(c, "LUMEN_IDENTITY_REGISTRY_FILE")
        .and_then(|e| e["value"].as_str().map(str::to_string))
        .expect("serving container must set LUMEN_IDENTITY_REGISTRY_FILE");
    assert!(
        path.ends_with("identities.json"),
        "LUMEN_IDENTITY_REGISTRY_FILE must name the file, not its directory: {path}"
    );

    let dir = path.rsplit_once('/').expect("an absolute path").0;
    let mount = c["volumeMounts"]
        .as_array()
        .expect("volumeMounts")
        .iter()
        .find(|m| m["mountPath"] == dir || m["mountPath"] == path)
        .unwrap_or_else(|| panic!("nothing is mounted at {dir}; mounts = {}", c["volumeMounts"]));

    let vol_name = mount["name"].as_str().expect("mount names a volume");
    let vol = pod_volumes(&objs)
        .into_iter()
        .find(|v| v["name"] == vol_name)
        .unwrap_or_else(|| panic!("volumeMount {vol_name} has no matching volume"));
    assert_eq!(
        vol["configMap"]["name"], "acct-identities",
        "the identity mount must come from the instance's own ConfigMap: {vol}"
    );
}

/// The audience list is the one auth check that cannot fail closed: a verifier
/// configured with no audience accepts every ID token Google mints, for any
/// service, and logs each as a successful authentication. It must reach the
/// process.
#[test]
fn the_audience_list_reaches_the_serving_process() {
    let objs = objects(&lumen_from(identity_spec()));
    let c = serving_container(&objs);
    let v = env_value(c, "LUMEN_AUTH_GOOGLE_AUDIENCES")
        .and_then(|e| e["value"].as_str().map(str::to_string))
        .expect("serving container must set LUMEN_AUTH_GOOGLE_AUDIENCES");
    assert!(
        v.contains("https://lumen.acme.internal"),
        "audience list must carry the CR's audiences: {v}"
    );
}

/// Bearer registry and identity registry are independent inputs. Both set means
/// two files from two Kubernetes objects, because one mount path cannot carry
/// two objects — and the union happens in the loader, not in the projection.
#[test]
fn a_bearer_secret_and_identity_grants_coexist_as_two_files() {
    let mut spec = identity_spec();
    spec.as_object_mut()
        .unwrap()
        .insert("tokensSecret".into(), json!("lumen-tokens"));
    let objs = objects(&lumen_from(spec));
    let c = serving_container(&objs);

    let tok = env_value(c, "LUMEN_TOKEN_REGISTRY_FILE")
        .and_then(|e| e["value"].as_str().map(str::to_string))
        .expect("LUMEN_TOKEN_REGISTRY_FILE when tokensSecret is set");
    let ids = env_value(c, "LUMEN_IDENTITY_REGISTRY_FILE")
        .and_then(|e| e["value"].as_str().map(str::to_string))
        .expect("LUMEN_IDENTITY_REGISTRY_FILE when identities are set");
    assert_ne!(tok, ids, "the two registries must be distinct files");

    let vols = pod_volumes(&objs);
    assert!(
        vols.iter().any(|v| v["secret"]["secretName"] == "lumen-tokens"),
        "the bearer Secret must still be projected: {vols:?}"
    );
    assert!(
        vols.iter().any(|v| v["configMap"]["name"] == "acct-identities"),
        "the identity ConfigMap must still be projected: {vols:?}"
    );
}

/// Identity-only is a complete configuration. Requiring a `tokensSecret`
/// alongside would defeat the point: the deployment that motivated #2764 has no
/// bearer secret anywhere.
#[test]
fn identity_grants_alone_need_no_secret() {
    let objs = objects(&lumen_from(identity_spec()));
    let vols = pod_volumes(&objs);
    assert!(
        !vols.iter().any(|v| v.get("secret").is_some()),
        "an identity-only instance must project no Secret volume: {vols:?}"
    );
    let c = serving_container(&objs);
    assert!(
        env_value(c, "LUMEN_TOKEN_REGISTRY_FILE").is_none(),
        "no bearer registry file without a tokensSecret"
    );
}

/// `auth: disabled` turns the whole thing off, including the projection. A
/// ConfigMap rendered for a disabled instance is a permission table that no
/// longer means anything, still sitting in the namespace.
#[test]
fn auth_disabled_projects_no_identity_objects() {
    let mut spec = identity_spec();
    spec.as_object_mut()
        .unwrap()
        .insert("auth".into(), json!("disabled"));
    let objs = objects(&lumen_from(spec));
    assert!(
        !objs
            .iter()
            .any(|o| o["kind"] == "ConfigMap" && o["metadata"]["name"] == "acct-identities"),
        "rendered: {:?}",
        kinds(&objs)
    );
    let c = serving_container(&objs);
    assert!(env_value(c, "LUMEN_IDENTITY_REGISTRY_FILE").is_none());
    assert!(env_value(c, "LUMEN_AUTH_GOOGLE_AUDIENCES").is_none());
}

/// The CSI transport is retired (#2764). Not "unused" — absent. A projection
/// path that still renders is a projection path somebody configures.
#[test]
fn nothing_renders_a_csi_projection_any_more() {
    for spec in [identity_spec(), json!({"auth": "required", "tokensSecret": "lumen-tokens"})] {
        let objs = objects(&lumen_from(spec));
        let all = serde_json::to_string(&objs).unwrap();
        assert!(
            !all.contains("SecretProviderClass"),
            "a SecretProviderClass reference survives the render: {all}"
        );
        assert!(
            !all.contains("secrets-store.csi.k8s.io")
                && !all.contains("secrets-store-gke.csi.k8s.io"),
            "a CSI driver reference survives the render"
        );
        for v in pod_volumes(&objs) {
            assert!(v.get("csi").is_none(), "a csi volume survives the render: {v}");
        }
    }
}

/// #2679's projection half: a backup runner that authenticates as its
/// ServiceAccount needs no admin bearer token.
#[test]
fn a_backup_runner_without_an_admin_secret_carries_no_bearer_token() {
    let mut spec = identity_spec();
    spec.as_object_mut().unwrap().insert(
        "serving".into(),
        json!({
            "backup": {
                "schedule": "0 3 * * *",
                "destination": "gs://acme-lumen-backups",
                "retentionSecs": 604800
            }
        }),
    );
    let objs = objects(&lumen_from(spec));
    let cron = find(&objs, "CronJob", "acct-backup");
    let all = serde_json::to_string(&cron).unwrap();
    assert!(
        !all.contains("secretKeyRef"),
        "backup CronJob still injects a bearer token secretKeyRef: {all}"
    );
}
