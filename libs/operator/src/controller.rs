//! The generic reconcile loop. Watches a [`ManagedService`] CR cluster-wide; for
//! each, server-side-applies the rendered child objects as the field manager
//! `S::MANAGER`, then writes back its status. Only the Lease holder applies
//! (leader-election gate), so `replicas > 1` is safe. Child objects are applied
//! generically as [`DynamicObject`]s keyed by GVK — no compile-time type per
//! kind. Lifted from lumen's `operator::reconcile`, generic over `S`.

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
use crate::service::{ManagedService, ReadyFacts};

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

    // 1. Render + apply every child object.
    for child in obj.render() {
        apply_object(client, &ns, S::MANAGER, child).await?;
    }

    // 2. Observe readiness for the service's declared targets.
    let mut ready = HashMap::new();
    for t in obj.readiness_targets() {
        let r = ready_replicas(client, &ns, t.kind, &t.name).await?;
        ready.insert(t.name, r);
    }

    // 3. Write the status subresource (Merge avoids managed-field conflicts).
    let status = obj.status_patch(&ReadyFacts { ready });
    let api: Api<S> = Api::namespaced(client.clone(), &ns);
    api.patch_status(&name, &PatchParams::default(), &Patch::Merge(&status))
        .await?;

    // Periodic re-reconcile corrects drift and refreshes status.
    Ok(Action::requeue(Duration::from_secs(30)))
}

fn error_policy<S: ManagedService>(_obj: Arc<S>, _err: &Error, _ctx: Arc<Ctx>) -> Action {
    Action::requeue(Duration::from_secs(15))
}
