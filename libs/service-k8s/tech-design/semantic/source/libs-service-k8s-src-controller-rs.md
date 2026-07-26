---
id: libs-service-k8s-src-controller-rs
summary: Lossless rust-source-unit coverage for `libs/service-k8s/src/controller.rs`.
capability_refs:
  - id: shared-kubernetes-operator-scaffold
    role: primary
    claim: shared-kubernetes-operator-scaffold-contract
    coverage: full
    rationale: "The source, tests, and manifest implement the Operator library contract."
fill_sections: [overview, source, changes]
---

# Standardized libs/service-k8s/src/controller.rs

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `libs/service-k8s/src/controller.rs` captured during libs codegen standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `Error` | libs/service-k8s/src/controller.rs | enum | pub | 26 | pub enum Error { |
| `run` | libs/service-k8s/src/controller.rs | function | pub | 57 | pub async fn run<S: ManagedService>() -> anyhow::Result<()> { |


## Source
<!-- type: rust-source-unit lang: rust -->

````rust
//! The generic reconcile loop. Watches a [`ManagedService`] CR cluster-wide; for
//! each, server-side-applies the rendered child objects as the field manager
//! `S::MANAGER`, then writes back its status. Only the Lease holder applies
//! (leader-election gate), so `replicas > 1` is safe. Child objects are applied
//! generically as [`DynamicObject`]s keyed by GVK — no compile-time type per
//! kind. Lifted from lumen's `service_k8s::reconcile`, generic over `S`.

use std::collections::HashMap;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::time::Duration;

use futures::StreamExt;
use kube::api::{Api, ApiResource, DynamicObject, Patch, PatchParams};
use kube::runtime::controller::{Action, Controller};
use kube::runtime::watcher;
use kube::{Client, ResourceExt};
use serde_json::Value;

use crate::lease::{self, Election};
use crate::service::{self, ManagedService, ReadyFacts};

/// Reconcile errors: `kube` + serde failures plus a guard for malformed rendered
/// objects (an operator bug, not a cluster condition).
#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("kube api error: {0}")]
    Kube(#[from] kube::Error),
    #[error("serialize error: {0}")]
    Serde(#[from] serde_json::Error),
    #[error("rendered object missing required field: {0}")]
    Missing(&'static str),
    #[error("service reconcile plan failed: {0}")]
    Plan(String),
}

struct Ctx {
    client: Client,
    election: Arc<Election>,
}

/// This replica's leader-election identity (pod name in k8s, else the manager).
fn identity(manager: &str) -> String {
    std::env::var("POD_NAME")
        .or_else(|_| std::env::var("HOSTNAME"))
        .unwrap_or_else(|_| manager.to_string())
}

/// The namespace the leader-election Lease lives in (the operator's own).
fn lease_namespace(manager: &str) -> String {
    std::env::var("POD_NAMESPACE").unwrap_or_else(|_| format!("{manager}-system"))
}

/// Run the operator for `S` until the process is terminated. Every replica
/// watches + reconciles, but only the Lease holder applies (HA-safe at
/// `replicas > 1`).
pub async fn run<S: ManagedService>() -> anyhow::Result<()> {
    let client = Client::try_default().await?;
    let election = Election::new(identity(S::MANAGER));
    lease::spawn(
        client.clone(),
        lease_namespace(S::MANAGER),
        S::MANAGER.to_string(),
        election.clone(),
    );
    let objs = Api::<S>::all(client.clone());
    tracing::info!(identity = %election.identity, manager = S::MANAGER, "operator starting; watching CR cluster-wide");
    Controller::new(objs, watcher::Config::default())
        .run(
            reconcile::<S>,
            error_policy::<S>,
            Arc::new(Ctx { client, election }),
        )
        .for_each(|res| async move {
            match res {
                Ok((obj, _)) => tracing::debug!(object = ?obj, "reconciled"),
                Err(e) => tracing::warn!(error = %e, "reconcile error"),
            }
        })
        .await;
    Ok(())
}

/// Plural for a kind. Covers the kinds the toolkit + services render; falls back
/// to the naive `lower(kind)+"s"`.
fn plural_for(kind: &str) -> String {
    match kind {
        "Deployment" => "deployments",
        "Service" => "services",
        "ConfigMap" => "configmaps",
        "ServiceAccount" => "serviceaccounts",
        "HorizontalPodAutoscaler" => "horizontalpodautoscalers",
        "PodDisruptionBudget" => "poddisruptionbudgets",
        "StatefulSet" => "statefulsets",
        "ServiceMonitor" => "servicemonitors",
        "PrometheusRule" => "prometheusrules",
        // The fallback would yield `networkpolicys` — a plural no apiserver
        // serves, so every apply of a rendered NetworkPolicy would 404 at
        // runtime with nothing failing at build time (#2603).
        "NetworkPolicy" => "networkpolicies",
        other => return format!("{}s", other.to_lowercase()),
    }
    .to_string()
}

/// Build the `ApiResource` (GVK + plural) for a dynamic apply.
fn api_resource(api_version: &str, kind: &str) -> ApiResource {
    let (group, version) = match api_version.split_once('/') {
        Some((g, v)) => (g.to_string(), v.to_string()),
        None => (String::new(), api_version.to_string()),
    };
    ApiResource {
        group,
        version,
        api_version: api_version.to_string(),
        kind: kind.to_string(),
        plural: plural_for(kind),
    }
}

/// Server-side-apply one rendered object into `ns` as field manager `manager`.
async fn apply_object(client: &Client, ns: &str, manager: &str, value: Value) -> Result<(), Error> {
    let api_version = value["apiVersion"]
        .as_str()
        .ok_or(Error::Missing("apiVersion"))?
        .to_string();
    let kind = value["kind"]
        .as_str()
        .ok_or(Error::Missing("kind"))?
        .to_string();
    let name = value["metadata"]["name"]
        .as_str()
        .ok_or(Error::Missing("metadata.name"))?
        .to_string();

    let ar = api_resource(&api_version, &kind);
    let obj: DynamicObject = serde_json::from_value(value)?;
    let api: Api<DynamicObject> = Api::namespaced_with(client.clone(), ns, &ar);
    api.patch(
        &name,
        &PatchParams::apply(manager).force(),
        &Patch::Apply(&obj),
    )
    .await?;
    tracing::debug!(%kind, %name, "applied");
    Ok(())
}

/// Delete one object the service no longer renders (#2603) — but only if this
/// CR owns it.
///
/// The ownership re-check is the whole safety story. `prunes()` hands back a
/// name, and a name in a namespace is not proof of authorship: another
/// controller, a Helm chart, or a human could have created a NetworkPolicy at
/// exactly the CR's name. Matching the live object's controller
/// `ownerReference` UID against the CR's own UID is proof, because only the
/// apiserver writes that link and only for objects we submitted with it.
///
/// Absent object → no-op, and a 404 racing the delete → success, so a prune
/// that runs on every requeue converges once and then costs one GET.
async fn prune_object(
    client: &Client,
    ns: &str,
    owner_uid: &str,
    target: &service::PruneTarget,
) -> Result<(), Error> {
    let ar = api_resource(target.api_version, target.kind);
    let api: Api<DynamicObject> = Api::namespaced_with(client.clone(), ns, &ar);
    let Some(live) = api.get_opt(&target.name).await? else {
        return Ok(());
    };
    let owned = live
        .metadata
        .owner_references
        .iter()
        .flatten()
        .any(|r| r.uid == owner_uid && r.controller.unwrap_or(false));
    if !owned {
        tracing::warn!(
            kind = %target.kind, name = %target.name, namespace = %ns,
            "prune: an object of this kind exists at the CR's name but is not \
             controller-owned by it — leaving it alone"
        );
        return Ok(());
    }
    match api.delete(&target.name, &Default::default()).await {
        Ok(_) => {
            tracing::info!(
                kind = %target.kind, name = %target.name, namespace = %ns,
                "prune: deleted a child the spec no longer asks for"
            );
            Ok(())
        }
        Err(kube::Error::Api(e)) if e.code == 404 => Ok(()),
        Err(err) => Err(err.into()),
    }
}

/// Read `.status.readyReplicas` off a workload, or 0 if absent.
async fn ready_replicas(client: &Client, ns: &str, kind: &str, name: &str) -> Result<i64, Error> {
    let ar = api_resource("apps/v1", kind);
    let api: Api<DynamicObject> = Api::namespaced_with(client.clone(), ns, &ar);
    Ok(api
        .get_opt(name)
        .await?
        .and_then(|o| o.data["status"]["readyReplicas"].as_i64())
        .unwrap_or(0))
}

async fn reconcile<S: ManagedService>(obj: Arc<S>, ctx: Arc<Ctx>) -> Result<Action, Error> {
    // Leader-election gate: a follower watches but never applies.
    if !ctx.election.is_leader.load(Ordering::Relaxed) {
        return Ok(Action::requeue(Duration::from_secs(10)));
    }
    let ns = obj
        .namespace()
        .ok_or(Error::Missing("metadata.namespace"))?;
    let name = obj.name_any();
    let client = &ctx.client;

    // 1. Let the service perform async admission/observation, then apply the
    // planned children through the shared SSA path.
    let plan = obj
        .reconcile_plan(client.clone())
        .await
        .map_err(|error| Error::Plan(error.to_string()))?;
    for child in plan.children {
        apply_object(client, &ns, S::MANAGER, child).await?;
    }

    // 1b. Remove children a previous spec rendered and this one does not
    // (#2603). Server-side apply cannot express "this object should no longer
    // exist", so without this step a conditional child is opt-in only. Failing
    // here fails the reconcile on purpose: if the spec says an enforcement
    // object should be gone and we could not remove it, the CR has not
    // converged and its status must not claim otherwise.
    if let Some(uid) = obj.meta().uid.as_deref() {
        for target in obj.prunes() {
            prune_object(client, &ns, uid, &target).await?;
        }
    }

    // 2. Observe readiness for the service's declared targets.
    let mut ready = HashMap::new();
    for t in obj.readiness_targets() {
        let r = ready_replicas(client, &ns, t.kind, &t.name).await?;
        ready.insert(t.name, r);
    }

    // 3. Write the status subresource (Merge avoids managed-field conflicts).
    let ready = ReadyFacts { ready };
    let mut status = obj.status_patch_with_context(&ready, &plan.context);

    // 3b. `status.conditions[]` (#2601). `lastTransitionTime` is a clock read,
    // which is why it cannot live in the service's synchronous, I/O-free status
    // projection — the service hands back clock-free facts and the reconcile
    // loop, already async, stamps them. Prior transition times are carried
    // forward from the watched object rather than left to the API server:
    // `Patch::Merge` replaces the array wholesale, so nothing survives
    // server-side unless it is re-sent.
    let facts = obj.conditions(&ready, &plan.context);
    if !facts.is_empty() {
        let projected = service::project(
            &obj.observed_conditions(),
            facts,
            obj.meta().generation.unwrap_or(0),
            &service::now_rfc3339(),
        );
        status
            .as_object_mut()
            .ok_or(Error::Missing("status patch root object"))?
            .entry("status")
            .or_insert_with(|| Value::Object(Default::default()))
            .as_object_mut()
            .ok_or(Error::Missing("status patch `status` object"))?
            .insert("conditions".to_string(), serde_json::to_value(projected)?);
    }

    let api: Api<S> = Api::namespaced(client.clone(), &ns);
    api.patch_status(&name, &PatchParams::default(), &Patch::Merge(&status))
        .await?;

    // Periodic re-reconcile corrects drift and refreshes status.
    Ok(Action::requeue(Duration::from_secs(30)))
}

fn error_policy<S: ManagedService>(_obj: Arc<S>, _err: &Error, _ctx: Arc<Ctx>) -> Action {
    Action::requeue(Duration::from_secs(15))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::sync::Mutex;

    /// The naive `lower(kind) + "s"` fallback is right for every kind the
    /// toolkit rendered until now, and silently wrong for a kind ending in
    /// `-y`. A wrong plural is invisible until an apply 404s against a live
    /// apiserver — exactly the class of bug a unit test should catch instead
    /// of a cluster run.
    #[test]
    fn irregular_plurals_are_pinned_rather_than_derived() {
        assert_eq!(plural_for("NetworkPolicy"), "networkpolicies");
        assert_ne!(
            plural_for("NetworkPolicy"),
            "networkpolicys",
            "the fallback's plural for a -y kind is not served by any apiserver"
        );

        // Kinds the fallback happens to get right are still pinned, so a future
        // rewrite of the table cannot quietly drop one.
        for (kind, plural) in [
            ("PodDisruptionBudget", "poddisruptionbudgets"),
            ("HorizontalPodAutoscaler", "horizontalpodautoscalers"),
            ("StatefulSet", "statefulsets"),
        ] {
            assert_eq!(plural_for(kind), plural);
        }

        // Unlisted regular kinds must keep flowing through the fallback;
        // pinning every kind by hand is how the table goes stale.
        assert_eq!(plural_for("Secret"), "secrets");
    }

    /// A fake apiserver that replays `responses` in order and records the
    /// method+path of every request, so a test can assert on the call that was
    /// *not* made — which is the whole point of an ownership guard.
    fn fake_apiserver(responses: Vec<(u16, Value)>) -> (Client, Arc<Mutex<Vec<String>>>) {
        let seen = Arc::new(Mutex::new(Vec::new()));
        let log = seen.clone();
        let queue = Arc::new(Mutex::new(responses.into_iter()));
        let service = tower::service_fn(move |req: http::Request<kube::client::Body>| {
            let log = log.clone();
            let queue = queue.clone();
            async move {
                log.lock()
                    .unwrap()
                    .push(format!("{} {}", req.method(), req.uri().path()));
                let (code, body) = queue.lock().unwrap().next().unwrap_or((500, json!({})));
                Ok::<_, std::convert::Infallible>(
                    http::Response::builder()
                        .status(code)
                        .header("content-type", "application/json")
                        .body(kube::client::Body::from(serde_json::to_vec(&body).unwrap()))
                        .unwrap(),
                )
            }
        });
        (Client::new(service, "acme"), seen)
    }

    fn np_target() -> service::PruneTarget {
        service::PruneTarget {
            api_version: "networking.k8s.io/v1",
            kind: "NetworkPolicy",
            name: "search".to_string(),
        }
    }

    fn live_policy(owner_uid: &str, controller: bool) -> Value {
        json!({
            "apiVersion": "networking.k8s.io/v1",
            "kind": "NetworkPolicy",
            "metadata": {
                "name": "search",
                "namespace": "acme",
                "ownerReferences": [{
                    "apiVersion": "lumen.dev/v1alpha1",
                    "kind": "Lumen",
                    "name": "search",
                    "uid": owner_uid,
                    "controller": controller,
                }],
            },
        })
    }

    fn not_found() -> (u16, Value) {
        (
            404,
            json!({ "kind": "Status", "status": "Failure", "message": "not found",
                    "reason": "NotFound", "code": 404 }),
        )
    }

    /// The happy path: our own child gets deleted, so flipping a toggle off
    /// actually stops the enforcement it turned on (#2603).
    #[tokio::test]
    async fn prune_deletes_a_child_this_cr_controls() {
        let (client, seen) = fake_apiserver(vec![
            (200, live_policy("uid-1234", true)),
            (200, live_policy("uid-1234", true)),
        ]);
        prune_object(&client, "acme", "uid-1234", &np_target())
            .await
            .expect("prune succeeds");
        let seen = seen.lock().unwrap();
        assert_eq!(seen.len(), 2, "expected a GET then a DELETE: {seen:?}");
        assert!(seen[1].starts_with("DELETE"), "{seen:?}");
    }

    /// The guard: a name is not proof of authorship. Another controller, a Helm
    /// chart, or a human can own an object at exactly this CR's name, and
    /// deleting it would be this operator destroying something it never made.
    #[tokio::test]
    async fn prune_leaves_an_object_this_cr_does_not_own() {
        let (client, seen) = fake_apiserver(vec![(200, live_policy("uid-somebody-else", true))]);
        prune_object(&client, "acme", "uid-1234", &np_target())
            .await
            .expect("a foreign object is a no-op, not an error");
        let seen = seen.lock().unwrap();
        assert_eq!(seen.len(), 1, "must stop after the GET: {seen:?}");
        assert!(seen[0].starts_with("GET"), "{seen:?}");
    }

    /// A plain (non-controller) owner reference is a weaker link than the one
    /// the apiserver writes for a controlled child — Kubernetes' own garbage
    /// collector distinguishes them, and so must this.
    #[tokio::test]
    async fn prune_leaves_a_non_controller_owner_reference_alone() {
        let (client, seen) = fake_apiserver(vec![(200, live_policy("uid-1234", false))]);
        prune_object(&client, "acme", "uid-1234", &np_target())
            .await
            .expect("no-op");
        assert_eq!(seen.lock().unwrap().len(), 1);
    }

    /// Prune runs on every requeue, so the steady state after it converges is
    /// "object already gone" — that has to be a cheap no-op rather than an
    /// error that fails the reconcile forever.
    #[tokio::test]
    async fn prune_of_an_absent_object_is_a_silent_no_op() {
        let (client, seen) = fake_apiserver(vec![not_found()]);
        prune_object(&client, "acme", "uid-1234", &np_target())
            .await
            .expect("absent is success");
        assert_eq!(seen.lock().unwrap().len(), 1);
    }

    /// Losing the delete race against the CR's own garbage collection must not
    /// fail the reconcile: both wanted the object gone and it is gone.
    #[tokio::test]
    async fn prune_treats_a_404_on_delete_as_success() {
        let (client, _) = fake_apiserver(vec![(200, live_policy("uid-1234", true)), not_found()]);
        prune_object(&client, "acme", "uid-1234", &np_target())
            .await
            .expect("racing GC is success");
    }

    #[test]
    fn api_resource_splits_group_and_version_for_a_networking_child() {
        let ar = api_resource("networking.k8s.io/v1", "NetworkPolicy");
        assert_eq!(ar.group, "networking.k8s.io");
        assert_eq!(ar.version, "v1");
        assert_eq!(ar.plural, "networkpolicies");

        // Core-group kinds carry an empty group, not the literal "v1".
        let core = api_resource("v1", "ConfigMap");
        assert_eq!(core.group, "");
        assert_eq!(core.version, "v1");
    }
}
````

## Changes
<!-- type: changes lang: yaml -->

```yaml
coverage_kind: semantic
changes:
  - path: "libs/service-k8s/src/controller.rs"
    action: modify
    section: rust-source-unit
    impl_mode: codegen
    description: |
      rust-source-unit (td_ast) source for `libs/service-k8s/src/controller.rs` captured during libs codegen standardization.
```
