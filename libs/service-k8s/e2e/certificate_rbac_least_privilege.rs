//! The declared RBAC grant is the grant the store actually needs (#3221).
//!
//! `KubernetesSecretStore` projects with Server-Side Apply, which is a `PATCH`
//! on the wire. `REQUIRED_RBAC_VERBS` was read off that wire shape and declared
//! `["get", "patch"]`.
//!
//! The apiserver does not authorize by HTTP method. An apply whose target does
//! not exist yet is authorized as **`create`** — the same request, against an
//! absent object, is a different authorization decision. Every certificate this
//! store projects is absent exactly once: the first time. A Role built from the
//! declared list therefore works for every renewal and fails the bootstrap, and
//! fails it as a 403 the store reports as `Forbidden` — an error that reads
//! like a misconfigured cluster rather than like a manifest this crate handed
//! the operator.
//!
//! | Case | Runs | What it pins |
//! |---|---|---|
//! | the declared list carries `create` | always | the constant the manifests are written from |
//! | a Role with exactly the declared verbs bootstraps | `SERVICE_K8S_LIVE_KUBE=1` | the apiserver agrees with the list, on the create *and* the renewal |
//! | the same Role minus `create` cannot bootstrap | `SERVICE_K8S_LIVE_KUBE=1` | `create` is required, not merely declared — without this the live case would pass under any superset |
//!
//! The live cases run under an *impersonated* ServiceAccount bound to a Role
//! this file builds from `REQUIRED_RBAC_VERBS` itself. The lifecycle case in
//! `certificate_kubernetes_store.rs` beside it runs as the caller's own
//! kubeconfig identity, which in every environment this has been run in is
//! cluster-admin — which is precisely why it could not see this.

use service_k8s::certificate::kubernetes_store::{RBAC_VERBS, REQUIRED_RBAC_VERBS};

/// The offline case: the declared list is what a manifest author copies, so it
/// is the artifact under test even before a cluster is involved.
#[test]
fn the_declared_verbs_include_the_one_a_first_apply_needs() {
    assert!(
        REQUIRED_RBAC_VERBS.contains(&"create"),
        "an SSA apply against an absent Secret is authorized as `create`; a grant \
         without it projects every renewal and refuses every bootstrap. Declared: \
         {REQUIRED_RBAC_VERBS:?}"
    );
    assert!(
        REQUIRED_RBAC_VERBS.contains(&"get"),
        "the store reads the live Secret before applying: {REQUIRED_RBAC_VERBS:?}"
    );
    assert!(
        REQUIRED_RBAC_VERBS.contains(&"patch"),
        "the apply itself is a PATCH: {REQUIRED_RBAC_VERBS:?}"
    );
    assert_eq!(
        RBAC_VERBS, REQUIRED_RBAC_VERBS,
        "the compatibility alias must not drift from the list it aliases"
    );
    // No `delete`, no `list`, no `watch`. The point of declaring a list at all
    // is that it is the *least* privilege that works, so a verb nobody can
    // point at a call site does not belong in it.
    assert_eq!(
        REQUIRED_RBAC_VERBS.len(),
        3,
        "least privilege: three verbs, each with a call site. Declared: \
         {REQUIRED_RBAC_VERBS:?}"
    );
}

#[cfg(feature = "certificate")]
mod live {
    use std::collections::BTreeMap;

    use k8s_openapi::api::core::v1::{Namespace, ServiceAccount};
    use k8s_openapi::api::rbac::v1::{PolicyRule, Role, RoleBinding, RoleRef, Subject};
    use k8s_openapi::apimachinery::pkg::apis::meta::v1::ObjectMeta;
    use kube::api::{Api, DeleteParams, Patch, PatchParams, PostParams};
    use kube::{Client, Config};
    use serde_json::json;

    use service_k8s::certificate::kubernetes_store::{
        KubernetesSecretStore, FIELD_MANAGER, REQUIRED_RBAC_VERBS,
    };
    use service_k8s::certificate::reconcile::{SecretStore, StoreErrorKind};

    const SA: &str = "least-privilege-projector";

    fn live_or_skip() {
        let live = std::env::var("SERVICE_K8S_LIVE_KUBE").unwrap_or_default();
        assert_eq!(
            live, "1",
            "SERVICE_K8S_LIVE_KUBE=1 must be set to run a live cluster test"
        );
    }

    /// A client that talks to the cluster as `system:serviceaccount:<ns>:<SA>`.
    /// Impersonation rather than a minted token: it needs no Secret of its own
    /// (which would itself require a grant this file is trying to measure) and
    /// it routes through the same authorizer a real pod's token would.
    async fn as_service_account(ns: &str) -> Client {
        let mut config = Config::infer().await.expect("kubeconfig");
        config.auth_info.impersonate = Some(format!("system:serviceaccount:{ns}:{SA}"));
        Client::try_from(config).expect("impersonating client")
    }

    /// Namespace + ServiceAccount + Role granting exactly `verbs` on secrets +
    /// the binding. Returns the namespace name.
    async fn stand_up(admin: &Client, verbs: &[&str], tag: &str) -> String {
        let ns = format!(
            "test-cert-rbac-{tag}-{}",
            chrono::Utc::now().timestamp_nanos_opt().unwrap_or(0)
        );
        Api::<Namespace>::all(admin.clone())
            .create(
                &PostParams::default(),
                &Namespace {
                    metadata: ObjectMeta {
                        name: Some(ns.clone()),
                        ..Default::default()
                    },
                    ..Default::default()
                },
            )
            .await
            .expect("create namespace");
        Api::<ServiceAccount>::namespaced(admin.clone(), &ns)
            .create(
                &PostParams::default(),
                &ServiceAccount {
                    metadata: ObjectMeta {
                        name: Some(SA.to_string()),
                        ..Default::default()
                    },
                    ..Default::default()
                },
            )
            .await
            .expect("create service account");
        Api::<Role>::namespaced(admin.clone(), &ns)
            .create(
                &PostParams::default(),
                &Role {
                    metadata: ObjectMeta {
                        name: Some("projector".into()),
                        ..Default::default()
                    },
                    rules: Some(vec![PolicyRule {
                        api_groups: Some(vec![String::new()]),
                        resources: Some(vec!["secrets".into()]),
                        verbs: verbs.iter().map(|v| (*v).to_string()).collect(),
                        ..Default::default()
                    }]),
                },
            )
            .await
            .expect("create role");
        Api::<RoleBinding>::namespaced(admin.clone(), &ns)
            .create(
                &PostParams::default(),
                &RoleBinding {
                    metadata: ObjectMeta {
                        name: Some("projector".into()),
                        ..Default::default()
                    },
                    role_ref: RoleRef {
                        api_group: "rbac.authorization.k8s.io".into(),
                        kind: "Role".into(),
                        name: "projector".into(),
                    },
                    subjects: Some(vec![Subject {
                        kind: "ServiceAccount".into(),
                        name: SA.into(),
                        namespace: Some(ns.clone()),
                        ..Default::default()
                    }]),
                },
            )
            .await
            .expect("create role binding");
        ns
    }

    async fn tear_down(admin: &Client, ns: &str) {
        let _ = Api::<Namespace>::all(admin.clone())
            .delete(ns, &DeleteParams::default())
            .await;
    }

    /// The SSA payload the store projects: a Secret that does not exist yet.
    fn desired(ns: &str, name: &str, value: &str) -> serde_json::Value {
        let mut data = BTreeMap::new();
        data.insert("tls.crt".to_string(), value.to_string());
        json!({
            "apiVersion": "v1",
            "kind": "Secret",
            "metadata": { "name": name, "namespace": ns },
            "type": "Opaque",
            "stringData": data,
        })
    }

    /// The case. A Role carrying exactly the declared verbs must carry the
    /// whole lifecycle: the first apply, which creates, and the second, which
    /// updates.
    #[tokio::test]
    #[ignore]
    async fn the_declared_grant_bootstraps_and_renews() {
        live_or_skip();
        let admin = Client::try_default().await.expect("admin client");
        let ns = stand_up(&admin, REQUIRED_RBAC_VERBS, "declared").await;

        let store = KubernetesSecretStore::new(as_service_account(&ns).await);
        let name = "peer-tls";

        store
            .apply(desired(&ns, name, "first"))
            .await
            .unwrap_or_else(|e| {
                panic!(
                    "the first apply creates the Secret, and the apiserver authorizes \
                     an apply-against-absent as `create`. A grant of {REQUIRED_RBAC_VERBS:?} \
                     must cover it: {e:?}"
                )
            });
        assert!(
            store.read(&ns, name).await.expect("read back").is_some(),
            "the projected Secret must be readable through the same grant"
        );
        store
            .apply(desired(&ns, name, "second"))
            .await
            .expect("the renewal apply updates an existing Secret and needs only `patch`");

        tear_down(&admin, &ns).await;
    }

    /// The negative control. Without it the case above passes under any grant
    /// that is a superset of what is needed, which is exactly the mistake the
    /// declared list made. Dropping `create` — and only `create` — must break
    /// the bootstrap and nothing else.
    #[tokio::test]
    #[ignore]
    async fn the_same_grant_without_create_cannot_bootstrap() {
        live_or_skip();
        let admin = Client::try_default().await.expect("admin client");
        let reduced: Vec<&str> = REQUIRED_RBAC_VERBS
            .iter()
            .copied()
            .filter(|v| *v != "create")
            .collect();
        assert_eq!(
            reduced.len(),
            REQUIRED_RBAC_VERBS.len() - 1,
            "the control has to actually remove something: {REQUIRED_RBAC_VERBS:?}"
        );
        let ns = stand_up(&admin, &reduced, "reduced").await;

        let store = KubernetesSecretStore::new(as_service_account(&ns).await);
        let err = store
            .apply(desired(&ns, "peer-tls", "first"))
            .await
            .expect_err("a first apply without `create` must be refused");
        assert!(
            matches!(err.kind, StoreErrorKind::Forbidden),
            "the refusal is an RBAC one: {err:?}"
        );

        // …and the reduced grant still covers a Secret that already exists, so
        // the failure above is about `create` and not about the Role being
        // broken outright.
        Api::<k8s_openapi::api::core::v1::Secret>::namespaced(admin.clone(), &ns)
            .patch(
                "peer-tls",
                &PatchParams::apply(FIELD_MANAGER).force(),
                &Patch::Apply(desired(&ns, "peer-tls", "seeded")),
            )
            .await
            .expect("admin seeds the Secret");
        store
            .apply(desired(&ns, "peer-tls", "renewed"))
            .await
            .expect("`patch` alone still covers an apply against an existing Secret");

        tear_down(&admin, &ns).await;
    }
}
