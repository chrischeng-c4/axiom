//! The retirement oracle for credential projection — what the operator must
//! *not* render, checked from the CR side.
//!
//! This file was `operator_identity_render.rs`, the #2764 oracle for projecting
//! `spec.identities` into a ConfigMap. #2870 removed that projection and #2872
//! removed the fields, so every positive assertion it carried has no subject
//! left. What survives, and is worth more than what it replaced, is the inverse:
//! a CR that still carries the old fields must produce a data plane with no
//! credential in it anywhere.
//!
//! Every spec here is built with `serde_json::from_value` rather than a struct
//! literal, and that stays deliberate for a new reason. A struct literal cannot
//! even name a removed field, so a struct-literal test would be green by
//! construction and prove nothing. Going through the CRD's own deserializer is
//! the only way to test the case that actually happens: an operator applying a
//! CR they wrote before the retirement. Serde ignores unknown keys, so these
//! specs deserialize — exactly the silent-no-op path the schema removal closes
//! at the API server, and exactly the reason the render must not be trusted to
//! fail on its own.
//!
//! The API-server half of the same claim is `kubectl apply --dry-run=server`
//! against the regenerated CRD (#2872 AC5); this is the local half.

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

/// The serving workload, whichever kind this topology renders. Single-member
/// and raft topologies differ (StatefulSet vs Deployment) and that choice is
/// not what this file is about — pinning one kind here would fail the oracle
/// for a reason unrelated to auth.
fn workload(objs: &[Value]) -> &Value {
    objs.iter()
        .find(|o| {
            (o["kind"] == "StatefulSet" || o["kind"] == "Deployment")
                && o["metadata"]["name"] == "acct"
        })
        .unwrap_or_else(|| panic!("no serving workload named acct; rendered: {:?}", kinds(objs)))
}

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

/// Every retired credential field, spelled as a CR author would have written
/// it before the retirement — including the two `spec.auth` values, because a
/// projection that only fired under `required` and one that fired regardless
/// are different bugs and only one of them is caught by testing one mode.
fn retired_specs() -> Vec<(&'static str, Value)> {
    let identities = json!({
        "reader@proj.iam.gserviceaccount.com": {
            "subject": "fastapi-reader",
            "roles": { "docs": "read" }
        },
        "writer@proj.iam.gserviceaccount.com": {
            "subject": "fastapi-writer",
            "roles": { "*": "write" }
        }
    });
    let mut out = Vec::new();
    for auth in ["required", "disabled"] {
        out.push((
            "identity registry",
            json!({
                "auth": auth,
                "identities": identities,
                "identityAudiences": ["https://lumen.acme.internal"],
            }),
        ));
        out.push((
            "bearer registry",
            json!({ "auth": auth, "tokensSecret": "lumen-tokens" }),
        ));
        out.push((
            "both registries",
            json!({
                "auth": auth,
                "tokensSecret": "lumen-tokens",
                "identities": identities,
                "identityAudiences": ["https://lumen.acme.internal"],
            }),
        ));
        out.push((
            "CSI registry",
            json!({
                "auth": auth,
                "tokensSecretProviderClass": "lumen-tokens-spc",
            }),
        ));
    }
    out
}

/// No object of any kind is rendered for a retired credential field.
///
/// A whole-object substring sweep rather than a walk over known paths: the
/// retired projections reached ConfigMap data, pod volumes, container env, and
/// CronJob pod templates, each at a different JSON path, and a check that
/// enumerates paths can only fail to notice the one it forgot.
#[test]
fn a_cr_carrying_a_retired_credential_field_renders_no_credential() {
    const RETIRED: [&str; 8] = [
        "acct-identities",
        "identities.json",
        "token-registry.json",
        "LUMEN_IDENTITY_REGISTRY_FILE",
        "LUMEN_TOKEN_REGISTRY_FILE",
        "LUMEN_AUTH_GOOGLE_AUDIENCES",
        "lumen-tokens",
        "SecretProviderClass",
    ];

    for (label, spec) in retired_specs() {
        let objs = render(&lumen_from(spec.clone()));
        let all = serde_json::to_string(&objs).expect("rendered objects serialize");
        // Positive control. Every assertion below is an absence, and an
        // absence check over an empty render passes for the wrong reason.
        assert!(
            all.contains("lumen:test"),
            "the {label} case rendered nothing recognizable: {:?}",
            kinds(&objs)
        );
        for needle in RETIRED {
            assert!(
                !all.contains(needle),
                "`{needle}` survives the render for the {label} case ({spec}); \
                 rendered: {:?}",
                kinds(&objs)
            );
        }
    }
}

/// The same claim at the pod level, where a credential would actually be
/// readable: no Secret volume, no ConfigMap volume carrying a registry, no CSI
/// volume. Asserted separately from the substring sweep because a volume can
/// reference a Secret whose name gives nothing away.
#[test]
fn no_credential_volume_reaches_the_serving_pod() {
    for (label, spec) in retired_specs() {
        let objs = render(&lumen_from(spec));
        for v in pod_volumes(&objs) {
            assert!(
                v.get("secret").is_none(),
                "a Secret volume survives the render for the {label} case: {v}"
            );
            assert!(
                v.get("csi").is_none(),
                "a CSI volume survives the render for the {label} case: {v}"
            );
        }
        let c = serving_container(&objs);
        for env in [
            "LUMEN_TOKEN_REGISTRY_FILE",
            "LUMEN_IDENTITY_REGISTRY_FILE",
            "LUMEN_AUTH_GOOGLE_AUDIENCES",
        ] {
            assert!(
                env_value(c, env).is_none(),
                "`{env}` reaches the serving process for the {label} case"
            );
        }
    }
}

/// The retired fields do not survive deserialization into a `LumenSpec` either.
///
/// This is the check the render tests cannot make: a render proves nothing was
/// *projected*, while this proves there is nothing to project from. Serde drops
/// unknown keys silently, so the evidence is that the round trip loses them —
/// which is also what the fleet's unknown-key detector reports as a rejection.
#[test]
fn a_retired_field_does_not_round_trip_through_the_spec() {
    for (label, spec) in retired_specs() {
        let l = lumen_from(spec);
        let back = serde_json::to_value(&l.spec).expect("spec re-serializes");
        // Positive control: a spec that serialized to nothing would satisfy
        // every absence below without proving anything.
        assert_eq!(back["image"], "lumen:test", "{back}");
        for retired in [
            "tokensSecret",
            "tokensSecretProviderClass",
            "identities",
            "identityAudiences",
        ] {
            assert!(
                back.get(retired).is_none(),
                "`{retired}` round-trips through LumenSpec for the {label} case: {back}"
            );
        }
    }
}
