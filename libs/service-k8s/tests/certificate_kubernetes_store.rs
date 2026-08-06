//! Unit and live integration tests for `KubernetesSecretStore` (#3221).

use std::time::Duration;

use k8s_openapi::api::core::v1::Secret;
use k8s_openapi::apimachinery::pkg::apis::meta::v1::{ObjectMeta, OwnerReference};
use k8s_openapi::ByteString;

use service_k8s::certificate::ephemeral::{instant, EphemeralIssuer};
use service_k8s::certificate::kubernetes_store::{
    classify_kube_error, prepare_ssa_patch, KubernetesSecretStore, FIELD_MANAGER, RBAC_VERBS,
    REQUIRED_RBAC_VERBS,
};
use service_k8s::certificate::profile::{
    CertificateIdentity, CertificateProfile, InstanceScope, Purpose,
};
use service_k8s::certificate::projection::{
    material_secret, parse_leaf, trust_bundle_secret, Owner, TrustBundle,
};
use service_k8s::certificate::reconcile::{
    Reconciler, RuntimeReport, SecretStore, StoreError, StoreErrorKind,
};
use service_k8s::certificate::state::{Action, IssueReason};
use service_k8s::certificate::Issuer;

fn scope(ns: &str) -> InstanceScope {
    InstanceScope::new(ns, "lumen-test", "lumen-prod.svc.id.goog")
}

fn owner(uid: &str) -> Owner {
    Owner {
        api_version: "v1".into(),
        kind: "ConfigMap".into(),
        name: "lumen-owner".into(),
        uid: uid.into(),
    }
}

fn peer_profile(ns: &str) -> CertificateProfile {
    CertificateProfile::new(
        &scope(ns),
        Purpose::Peer,
        format!("lumen-0.lumen-headless.{ns}.svc.cluster.local"),
        CertificateIdentity {
            dns_names: vec![format!("lumen-0.lumen-headless.{ns}.svc.cluster.local")],
            spiffe_uri: Some(format!("spiffe://lumen-prod.svc.id.goog/ns/{ns}/sa/lumen")),
        },
        Duration::from_secs(12 * 3600),
        Duration::from_secs(2 * 3600),
        Duration::ZERO,
    )
    .expect("peer profile")
}

fn activated(fingerprint: Option<String>) -> RuntimeReport {
    RuntimeReport {
        activated_fingerprint: fingerprint,
        consecutive_failures: 0,
    }
}

#[test]
fn rbac_verbs_are_exactly_get_and_patch() {
    assert_eq!(FIELD_MANAGER, "service-k8s-certificate");
    assert_eq!(REQUIRED_RBAC_VERBS, &["get", "patch"]);
    assert_eq!(RBAC_VERBS, &["get", "patch"]);
}

#[test]
fn error_classification_and_retryability() {
    let make_api_err = |code: u16, reason: &str, message: &str| {
        kube::Error::Api(kube::error::ErrorResponse {
            status: "Failure".into(),
            message: message.into(),
            reason: reason.into(),
            code,
        })
    };

    // 403 Forbidden -> not retryable
    let err_403 = classify_kube_error(&make_api_err(403, "Forbidden", "RBAC denied"));
    assert_eq!(err_403.kind, StoreErrorKind::Forbidden);
    assert!(!err_403.retryable());
    assert_eq!(err_403.to_string(), "forbidden: RBAC denied");

    // 409 Conflict -> retryable
    let err_409 = classify_kube_error(&make_api_err(
        409,
        "Conflict",
        "Operation cannot be fulfilled",
    ));
    assert_eq!(err_409.kind, StoreErrorKind::Conflict);
    assert!(err_409.retryable());
    assert_eq!(
        err_409.to_string(),
        "conflict: Operation cannot be fulfilled"
    );

    // 503 Unavailable -> retryable
    let err_503 = classify_kube_error(&make_api_err(
        503,
        "ServiceUnavailable",
        "Service unavailable",
    ));
    assert_eq!(err_503.kind, StoreErrorKind::Unavailable);
    assert!(err_503.retryable());
    assert_eq!(err_503.to_string(), "unavailable: Service unavailable");

    // Malformed object -> not retryable
    let malformed = StoreError::malformed("missing metadata");
    assert_eq!(malformed.kind, StoreErrorKind::Malformed);
    assert!(!malformed.retryable());
    assert_eq!(malformed.to_string(), "malformed object: missing metadata");
}

#[test]
fn error_redaction_prevents_leaking_secrets_and_tokens() {
    let raw_err = kube::Error::Api(kube::error::ErrorResponse {
        status: "Failure".into(),
        message: "Failed for Bearer secrettoken123 and -----BEGIN PRIVATE KEY-----\nkeydata\n-----END PRIVATE KEY-----".into(),
        reason: "Forbidden".into(),
        code: 403,
    });
    let classified = classify_kube_error(&raw_err);
    assert_eq!(
        classified,
        StoreError::forbidden("Failed for Bearer [redacted] and [redacted pem]")
    );
    assert!(!classified.retryable());
}

#[test]
fn prepare_ssa_patch_merges_live_leaf_and_detects_unchanged() {
    let now = instant(2026, 7, 1, 12);
    let issuer = EphemeralIssuer::new("pool-a", now);
    let scope = scope("lumen");
    let owner = owner("0f7d1f4e-0000-4000-8000-000000000000");
    let profile = peer_profile("lumen");

    let mut bundle = TrustBundle::new();
    bundle.insert(
        service_k8s::certificate::IssuerId::new("pool-a"),
        "ANCHOR_A_PEM",
    );

    let (req, key) = service_k8s::certificate::IssuanceRequest::build(&scope, &profile).unwrap();
    let mat = futures::executor::block_on(issuer.issue(req)).unwrap();

    let mat_secret = material_secret(
        &scope,
        Purpose::Peer,
        &owner,
        &mat,
        &key.into_pem(),
        &bundle,
        &profile.identity_digest(),
    );

    // Initial apply against no live secret -> not unchanged, returns typed Secret with data
    let (patch1, unchanged1) = prepare_ssa_patch(&mat_secret, None).unwrap();
    assert!(!unchanged1);
    assert!(patch1.string_data.is_none());
    let data1 = patch1.data.as_ref().unwrap();

    // Prove each desired stringData value becomes the exact UTF-8 byte sequence in typed Secret data
    let string_data = mat_secret["stringData"].as_object().unwrap();
    for (k, v) in string_data {
        let expected_bytes = v.as_str().unwrap().as_bytes();
        assert_eq!(data1.get(k).unwrap().0.as_slice(), expected_bytes);
    }

    // Construct live Secret matching patch1
    let live_secret = Secret {
        type_: Some("Opaque".into()),
        metadata: patch1.metadata.clone(),
        data: patch1.data.clone(),
        ..Default::default()
    };

    // Re-apply identical material_secret -> unchanged is true
    let (_, unchanged2) = prepare_ssa_patch(&mat_secret, Some(&live_secret)).unwrap();
    assert!(unchanged2);

    // Prove a live Secret with external data key, external annotation, and external label is still unchanged
    let mut live_with_external = live_secret.clone();
    live_with_external.data.as_mut().unwrap().insert(
        "external-key.txt".into(),
        ByteString(b"external_data".to_vec()),
    );
    live_with_external
        .metadata
        .annotations
        .as_mut()
        .unwrap()
        .insert("external.annotation.domain/foo".into(), "bar".into());
    live_with_external
        .metadata
        .labels
        .as_mut()
        .unwrap()
        .insert("external.label.domain/env".into(), "prod".into());

    let (patch_ext, unchanged_ext) =
        prepare_ssa_patch(&mat_secret, Some(&live_with_external)).unwrap();
    assert!(
        unchanged_ext,
        "external fields must not force a write when lifecycle-owned fields match"
    );
    let patch_ext_data = patch_ext.data.as_ref().unwrap();
    assert!(
        !patch_ext_data.contains_key("external-key.txt"),
        "external data key must not be claimed by this manager"
    );
    let patch_ext_ann = patch_ext.metadata.annotations.as_ref().unwrap();
    assert!(
        !patch_ext_ann.contains_key("external.annotation.domain/foo"),
        "external annotation must not be claimed by this manager"
    );
    let patch_ext_lbl = patch_ext.metadata.labels.as_ref().unwrap();
    assert!(
        !patch_ext_lbl.contains_key("external.label.domain/env"),
        "external label must not be claimed by this manager"
    );

    // Apply trust_bundle_secret (contains only ca.crt and trust-bundle annotation)
    let mut bundle2 = bundle.clone();
    bundle2.insert(
        service_k8s::certificate::IssuerId::new("pool-b"),
        "ANCHOR_B_PEM",
    );
    let trust_secret = trust_bundle_secret(&scope, Purpose::Peer, &owner, &bundle2);

    let (merged_trust, unchanged3) = prepare_ssa_patch(&trust_secret, Some(&live_secret)).unwrap();
    assert!(!unchanged3);

    // Verify leaf material (tls.crt, tls.key) and leaf annotations were preserved in merged patch
    let merged_data = merged_trust.data.as_ref().unwrap();
    assert!(merged_data.contains_key("tls.crt"));
    assert!(merged_data.contains_key("tls.key"));
    assert!(merged_data.contains_key("ca.crt"));

    let merged_ann = merged_trust.metadata.annotations.as_ref().unwrap();
    assert!(merged_ann.contains_key("service-k8s.axiom.dev/leaf-issuer"));
    assert!(merged_ann.contains_key("service-k8s.axiom.dev/identity-digest"));
    assert!(merged_ann.contains_key("service-k8s.axiom.dev/trust-bundle"));
}

#[test]
fn prepare_ssa_patch_detects_changed_or_missing_owner_uid() {
    let now = instant(2026, 7, 1, 12);
    let issuer = EphemeralIssuer::new("pool-a", now);
    let scope = scope("lumen");
    let owner_a = owner("0f7d1f4e-0000-4000-8000-000000000000");
    let profile = peer_profile("lumen");

    let mut bundle = TrustBundle::new();
    bundle.insert(
        service_k8s::certificate::IssuerId::new("pool-a"),
        "ANCHOR_A_PEM",
    );

    let (req, key) = service_k8s::certificate::IssuanceRequest::build(&scope, &profile).unwrap();
    let mat = futures::executor::block_on(issuer.issue(req)).unwrap();

    let mat_secret = material_secret(
        &scope,
        Purpose::Peer,
        &owner_a,
        &mat,
        &key.into_pem(),
        &bundle,
        &profile.identity_digest(),
    );

    let (patch1, _) = prepare_ssa_patch(&mat_secret, None).unwrap();

    // 1. Live secret with missing ownerReferences -> not unchanged
    let mut live_no_owner = Secret {
        type_: Some("Opaque".into()),
        metadata: patch1.metadata.clone(),
        data: patch1.data.clone(),
        ..Default::default()
    };
    live_no_owner.metadata.owner_references = None;

    let (_, unchanged_no_owner) = prepare_ssa_patch(&mat_secret, Some(&live_no_owner)).unwrap();
    assert!(
        !unchanged_no_owner,
        "identical data with missing ownerReference must not be considered unchanged"
    );

    // 2. Live secret with different owner UID -> not unchanged
    let mut live_diff_owner = Secret {
        type_: Some("Opaque".into()),
        metadata: patch1.metadata.clone(),
        data: patch1.data.clone(),
        ..Default::default()
    };
    live_diff_owner.metadata.owner_references = Some(vec![OwnerReference {
        api_version: "v1".into(),
        kind: "ConfigMap".into(),
        name: "lumen-owner".into(),
        uid: "different-uid-1234".into(),
        controller: Some(true),
        block_owner_deletion: Some(true),
    }]);

    let (_, unchanged_diff_owner) = prepare_ssa_patch(&mat_secret, Some(&live_diff_owner)).unwrap();
    assert!(
        !unchanged_diff_owner,
        "identical data with changed owner UID must not be considered unchanged"
    );
}

#[test]
#[ignore]
fn live_kubernetes_secret_store_lifecycle() {
    let live_env = std::env::var("SERVICE_K8S_LIVE_KUBE").unwrap_or_default();
    assert_eq!(
        live_env, "1",
        "SERVICE_K8S_LIVE_KUBE=1 environment variable must be set to run live cluster test"
    );

    let rt = tokio::runtime::Runtime::new().expect("tokio runtime");
    rt.block_on(async {
        use k8s_openapi::api::core::v1::{ConfigMap, Namespace};
        use kube::api::{Api, DeleteParams, PostParams};

        let client = kube::Client::try_default()
            .await
            .expect("kube client default");

        let ns_name = format!(
            "test-cert-store-{}",
            chrono::Utc::now().timestamp_nanos_opt().unwrap_or(0)
        );
        let ns_api: Api<Namespace> = Api::all(client.clone());

        // 1. Create unique namespace
        let ns_obj = Namespace {
            metadata: ObjectMeta {
                name: Some(ns_name.clone()),
                ..Default::default()
            },
            ..Default::default()
        };
        ns_api
            .create(&PostParams::default(), &ns_obj)
            .await
            .expect("create namespace");

        // 2. Create ConfigMap owner
        let cm_api: Api<ConfigMap> = Api::namespaced(client.clone(), &ns_name);
        let cm_obj = ConfigMap {
            metadata: ObjectMeta {
                name: Some("lumen-owner".into()),
                ..Default::default()
            },
            ..Default::default()
        };
        let created_cm = cm_api
            .create(&PostParams::default(), &cm_obj)
            .await
            .expect("create owner configmap");
        let owner_uid = created_cm.metadata.uid.expect("configmap uid");

        let scope = scope(&ns_name);
        let owner = owner(&owner_uid);
        let store = KubernetesSecretStore::new(client.clone());
        let secret_name = scope.secret_name(Purpose::Peer);

        // Prove pre-create store read is None
        let pre_read = store.read(&ns_name, &secret_name).await.expect("pre-read");
        assert!(pre_read.is_none(), "pre-create store read must return None");

        // 3 & 4. Drive real lifecycle with EphemeralIssuer and Reconciler
        let start_time = instant(2026, 7, 1, 12);
        let issuer_a = EphemeralIssuer::new("pool-a", start_time);
        let profile = peer_profile(&ns_name);
        let reconciler_a = Reconciler::new(&scope, &owner, &store, &issuer_a);

        // Step 1: PublishTrustBundle
        let outcome1 = reconciler_a
            .reconcile(&profile, &RuntimeReport::default(), start_time)
            .await
            .expect("reconcile step 1");
        assert!(
            matches!(outcome1.action, Action::PublishTrustBundle { .. }),
            "expected PublishTrustBundle, got {:?}",
            outcome1.action
        );

        // Step 2: Issue (Bootstrap)
        let outcome2 = reconciler_a
            .reconcile(&profile, &RuntimeReport::default(), start_time)
            .await
            .expect("reconcile step 2");
        assert!(
            matches!(
                outcome2.action,
                Action::Issue {
                    reason: IssueReason::Bootstrap,
                    ..
                }
            ),
            "expected Issue(Bootstrap), got {:?}",
            outcome2.action
        );

        // Step 3: Wait / AwaitActivation
        let outcome3 = reconciler_a
            .reconcile(&profile, &RuntimeReport::default(), start_time)
            .await
            .expect("reconcile step 3");
        assert!(
            matches!(
                outcome3.action,
                Action::Wait { .. } | Action::AwaitActivation { .. }
            ),
            "expected Wait or AwaitActivation, got {:?}",
            outcome3.action
        );

        // 5. Assert API Secret ownerReference equals real ConfigMap UID
        let secret_api: Api<Secret> = Api::namespaced(client.clone(), &ns_name);
        let live1 = secret_api
            .get(&secret_name)
            .await
            .expect("get secret from API");
        let owner_refs = live1
            .metadata
            .owner_references
            .as_ref()
            .expect("ownerReferences");
        assert_eq!(owner_refs[0].uid, owner_uid);

        // 6. Record bytes & resourceVersion; run settled reconcile with activated fingerprint; prove unchanged
        let rv1 = live1
            .metadata
            .resource_version
            .clone()
            .expect("resourceVersion");
        let data1 = live1.data.clone().expect("data");

        let stored1 = store
            .read(&ns_name, &secret_name)
            .await
            .expect("read")
            .expect("stored");
        let pem1 = String::from_utf8(stored1.data["tls.crt"].clone()).expect("pem string");
        let facts1 = parse_leaf(&pem1).expect("parse leaf");

        let settled_outcome = reconciler_a
            .reconcile(
                &profile,
                &activated(Some(facts1.fingerprint.clone())),
                start_time,
            )
            .await
            .expect("settled reconcile");
        assert!(
            matches!(settled_outcome.action, Action::Wait { .. }),
            "expected Wait, got {:?}",
            settled_outcome.action
        );

        let live2 = secret_api
            .get(&secret_name)
            .await
            .expect("get secret after settled reconcile");
        let rv2 = live2.metadata.resource_version.expect("resourceVersion");
        let data2 = live2.data.expect("data");

        assert_eq!(
            rv1, rv2,
            "settled reconcile must produce no write and leave resourceVersion unchanged"
        );
        assert_eq!(data1, data2, "settled reconcile must leave bytes unchanged");

        // 7. Create second real EphemeralIssuer, run PublishTrustBundle rotation step, prove 2 valid PEM anchors coexist & leaf/key exact
        let issuer_b = EphemeralIssuer::new("pool-b", start_time);
        let reconciler_b = Reconciler::new(&scope, &owner, &store, &issuer_b);

        let rot_outcome = reconciler_b
            .reconcile(
                &profile,
                &activated(Some(facts1.fingerprint.clone())),
                start_time,
            )
            .await
            .expect("rotation reconcile step");
        assert!(
            matches!(rot_outcome.action, Action::PublishTrustBundle { .. }),
            "expected PublishTrustBundle during rotation, got {:?}",
            rot_outcome.action
        );

        let stored_rotated = store
            .read(&ns_name, &secret_name)
            .await
            .expect("read rotated secret")
            .expect("rotated secret exists");

        // Assert leaf and key bytes remain exact
        assert_eq!(stored_rotated.data["tls.crt"], stored1.data["tls.crt"]);
        assert_eq!(stored_rotated.data["tls.key"], stored1.data["tls.key"]);

        // Assert ca.crt contains 2 valid PEM anchors
        let ca_crt_pem =
            String::from_utf8(stored_rotated.data["ca.crt"].clone()).expect("ca.crt pem");
        let bundle_parsed = TrustBundle::parse(
            &ca_crt_pem,
            stored_rotated
                .annotations
                .get("service-k8s.axiom.dev/trust-bundle")
                .map(String::as_str),
        );
        assert_eq!(
            bundle_parsed.issuers().len(),
            2,
            "both anchors must coexist in ca.crt"
        );

        // 8. Delete owner ConfigMap and poll until read returns None
        cm_api
            .delete("lumen-owner", &DeleteParams::default())
            .await
            .expect("delete owner configmap");

        let mut gc_successful = false;
        for _ in 0..30 {
            tokio::time::sleep(Duration::from_secs(1)).await;
            if store.read(&ns_name, &secret_name).await.unwrap().is_none() {
                gc_successful = true;
                break;
            }
        }
        assert!(
            gc_successful,
            "Secret was not garbage-collected after owner deletion"
        );

        // 9. Clean up namespace
        let _ = ns_api.delete(&ns_name, &DeleteParams::default()).await;
    });
}
