// <HANDWRITE gap="standardize:claim-code" tracker="projects-lumen-src-operator-reconcile-rs" reason="Existing code claimed during Score standardization until deterministic generator coverage lands.">
//! The reconcile loop. Watches `Lumen` objects cluster-wide; for each, renders
//! the child objects ([`render::render`]) and server-side-applies them as the
//! field manager `lumen-operator`, then writes back a status subresource
//! summarizing serving + broker readiness. Drift is corrected by a periodic
//! requeue.
//!
//! Child objects are applied generically as [`DynamicObject`]s keyed by GVK, so
//! the operator needs no compile-time type for every kind (Deployment, HPA,
//! StatefulSet, ServiceMonitor, …) it manages.

use std::sync::Arc;
use std::time::Duration;

use futures::StreamExt;
use kube::api::{Api, ApiResource, DynamicObject, Patch, PatchParams};
use kube::runtime::controller::{Action, Controller};
use kube::runtime::watcher;
use kube::{Client, ResourceExt};
use serde_json::{json, Value};

use super::crd::Lumen;
use super::lease::{self, Election};
use super::render;
use std::sync::atomic::Ordering;

/// Server-side-apply field manager. Owns the fields the operator sets, so it
/// can adopt and update existing objects without clobbering other managers.
const MANAGER: &str = "lumen-operator";

/// Reconcile errors. `kube` + serde failures plus a guard for malformed
/// rendered objects (which would be an operator bug, not a cluster condition).
#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("kube api error: {0}")]
    Kube(#[from] kube::Error),
    #[error("serialize error: {0}")]
    Serde(#[from] serde_json::Error),
    #[error("rendered object missing required field: {0}")]
    Missing(&'static str),
}

/// Shared reconcile context.
struct Ctx {
    client: Client,
    election: Arc<Election>,
}

/// This replica's leader-election identity (pod name when running in k8s).
fn identity() -> String {
    std::env::var("POD_NAME")
        .or_else(|_| std::env::var("HOSTNAME"))
        .unwrap_or_else(|_| "lumen-operator".to_string())
}

/// The namespace the leader-election Lease lives in (the operator's own).
fn lease_namespace() -> String {
    std::env::var("POD_NAMESPACE").unwrap_or_else(|_| "lumen-system".to_string())
}

/// Run the operator until the process is terminated. Every replica watches and
/// runs the reconcile loop, but only the Lease holder applies changes (HA-safe
/// at `replicas > 1` — see [`super::lease`]).
pub async fn run() -> anyhow::Result<()> {
    let client = Client::try_default().await?;
    let election = Election::new(identity());
    lease::spawn(client.clone(), lease_namespace(), election.clone());
    let lumens = Api::<Lumen>::all(client.clone());
    tracing::info!(identity = %election.identity, "lumen-operator starting; watching Lumen objects cluster-wide");
    Controller::new(lumens, watcher::Config::default())
        .run(reconcile, error_policy, Arc::new(Ctx { client, election }))
        .for_each(|res| async move {
            match res {
                Ok((obj, _)) => tracing::debug!(object = ?obj, "reconciled"),
                Err(e) => tracing::warn!(error = %e, "reconcile error"),
            }
        })
        .await;
    Ok(())
}

/// Plural for a kind. Covers everything the operator renders; falls back to the
/// naive `lower(kind)+"s"` for anything else.
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

/// Build the `ApiResource` (GVK + plural) for a dynamic apply, parsing
/// `apiVersion` into group/version (`""`/`v1` for the core group).
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

/// Server-side-apply one rendered object into `ns`.
async fn apply_object(client: &Client, ns: &str, value: Value) -> Result<(), Error> {
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
        &PatchParams::apply(MANAGER).force(),
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

async fn reconcile(lumen: Arc<Lumen>, ctx: Arc<Ctx>) -> Result<Action, Error> {
    // Leader-election gate: a follower replica watches but never applies, so
    // two replicas never fight over the same objects.
    if !ctx.election.is_leader.load(Ordering::Relaxed) {
        return Ok(Action::requeue(Duration::from_secs(10)));
    }
    let ns = lumen
        .namespace()
        .ok_or(Error::Missing("metadata.namespace"))?;
    let name = lumen.name_any();
    let client = &ctx.client;

    // 1. Render + apply every child object.
    for obj in render::render(&lumen) {
        apply_object(client, &ns, obj).await?;
    }

    // 2. Observe readiness.
    let ready = ready_replicas(client, &ns, "Deployment", &name).await? as i32;
    let nats_ready = if lumen.spec.nats.is_managed() {
        ready_replicas(client, &ns, "StatefulSet", &format!("{name}-nats")).await? >= 1
    } else {
        true // external broker: assumed up; serving pods' connect-retry covers blips.
    };
    let desired = lumen.spec.serving.autoscaling.min_replicas;
    let phase = if ready >= desired && nats_ready {
        "Ready"
    } else if ready > 0 {
        "Reconciling"
    } else {
        "Pending"
    };

    // 3. Write the status subresource (Merge avoids managed-field conflicts).
    let status = json!({ "status": {
        "phase": phase,
        "observedGeneration": lumen.metadata.generation.unwrap_or(0),
        "servingReadyReplicas": ready,
        "desiredReplicas": desired,
        "shardCount": lumen.spec.shard_count,
        "natsReady": nats_ready,
        "message": format!("{ready}/{desired} serving pods ready; natsReady={nats_ready}"),
    }});
    let lum_api: Api<Lumen> = Api::namespaced(client.clone(), &ns);
    lum_api
        .patch_status(&name, &PatchParams::default(), &Patch::Merge(&status))
        .await?;

    // Periodic re-reconcile corrects drift and refreshes status.
    Ok(Action::requeue(Duration::from_secs(30)))
}

fn error_policy(_lumen: Arc<Lumen>, _err: &Error, _ctx: Arc<Ctx>) -> Action {
    Action::requeue(Duration::from_secs(15))
}

// </HANDWRITE>
