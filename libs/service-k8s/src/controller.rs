// CODEGEN-BEGIN
//! The generic reconcile loop. Watches a [`ManagedService`] CR cluster-wide; for
//! each, server-side-applies the rendered child objects as the field manager
//! `S::MANAGER`, then writes back its status. Only the Lease holder applies
//! (leader-election gate), so `replicas > 1` is safe. Child objects are applied
//! generically as [`DynamicObject`]s keyed by GVK — no compile-time type per
//! kind. Lifted from lumen's `service_k8s::reconcile`, generic over `S`.

use std::collections::HashMap;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::time::{Duration, Instant};

use futures::StreamExt;
use kube::api::{Api, ApiResource, DynamicObject, Patch, PatchParams};
use kube::runtime::controller::{Action, Controller};
use kube::runtime::events::{Event, EventType, Recorder, Reporter};
use kube::runtime::watcher;
use kube::{Client, ResourceExt};
use serde_json::Value;

use crate::lease::{self, Election};
use crate::metrics::{self, ControllerMetrics};
use crate::service::{self, ManagedService, ReadyFacts};

/// Reconcile errors: `kube` + serde failures plus a guard for malformed rendered
/// objects (an operator bug, not a cluster condition).
// <HANDWRITE gap="missing-generator:logic:async-anchor" tracker="#1855" reason="AW cannot currently scaffold a hand-written region around async fn reconcile, so the planning error seam is bounded manually under the blocker.">
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
// </HANDWRITE>

struct Ctx {
    client: Client,
    election: Arc<Election>,
    metrics: Arc<ControllerMetrics>,
    recorder: Recorder,
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

    // Control-plane observability (#2620). The scrape listener runs alongside
    // the controller rather than inside it: leadership is read at scrape time,
    // so a follower replica publishes an honest `_leader 0` instead of going
    // dark, and every replica is independently scrapeable.
    let controller_metrics = Arc::new(ControllerMetrics::new(S::MANAGER));
    {
        let election = election.clone();
        tokio::spawn(metrics::serve(
            metrics::metrics_addr(),
            controller_metrics.clone(),
            move || election.is_leader.load(Ordering::Relaxed),
        ));
    }
    let recorder = Recorder::new(
        client.clone(),
        Reporter {
            controller: S::MANAGER.to_string(),
            instance: Some(election.identity.clone()),
        },
    );

    let objs = Api::<S>::all(client.clone());
    tracing::info!(identity = %election.identity, manager = S::MANAGER, "operator starting; watching CR cluster-wide");
    Controller::new(objs, watcher::Config::default())
        .run(
            reconcile_entry::<S>,
            error_policy::<S>,
            Arc::new(Ctx {
                client,
                election,
                metrics: controller_metrics,
                recorder,
            }),
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

// <HANDWRITE gap="missing-generator:logic:async-anchor" tracker="#1855" reason="AW cannot currently match async Rust functions as hand-write anchors; implement the TD-owned plan/apply/readiness/status sequence under the filed blocker.">
/// The instrumented entry point the controller actually calls (#2620).
///
/// The leader gate lives here rather than in [`reconcile`] on purpose: a
/// follower replica does no work, so counting its no-ops would inflate
/// `_reconcile_total` on the one replica that never touched the cluster and
/// dilute the error ratio of the one that did.
async fn reconcile_entry<S: ManagedService>(obj: Arc<S>, ctx: Arc<Ctx>) -> Result<Action, Error> {
    if !ctx.election.is_leader.load(Ordering::Relaxed) {
        return Ok(Action::requeue(Duration::from_secs(10)));
    }
    let started = Instant::now();
    let result = reconcile::<S>(obj, ctx.clone()).await;
    // Failures are timed too: a reconcile that fails after a 30s apiserver
    // timeout is a different problem from one that fails instantly, and a
    // histogram that only records successes cannot tell them apart.
    ctx.metrics.observe(started.elapsed());
    result
}

async fn reconcile<S: ManagedService>(obj: Arc<S>, ctx: Arc<Ctx>) -> Result<Action, Error> {
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
        let prior = obj.observed_conditions();
        let generation = obj.meta().generation.unwrap_or(0);

        // Tell the CR's owner, once, that their edit was picked up (#2620).
        //
        // The trigger is a generation the operator has not converged yet, which
        // makes the event fire on the first reconcile of a new CR and on every
        // spec change, and stay silent through the 30s steady-state requeues in
        // between. Publishing unconditionally would instead mean one apiserver
        // write per CR per requeue forever, deduplicated into an ever-counting
        // `EventSeries` that says nothing.
        let converged = prior
            .iter()
            .filter_map(|c| c.observed_generation)
            .max()
            .is_some_and(|observed| observed == generation);
        if !converged {
            publish(
                &ctx.recorder,
                obj.as_ref(),
                EventType::Normal,
                "Reconciled",
                "Reconcile",
                format!("applied spec generation {generation}"),
            )
            .await;
        }

        let projected = service::project(
            &prior,
            facts,
            generation,
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
// </HANDWRITE>

/// Publish one Event against the CR, best-effort.
///
/// Best-effort is deliberate: an operator that fails its reconcile *and* then
/// fails to say so must still requeue and retry. Losing the narration is a
/// smaller harm than a controller that stops because its own event write was
/// rejected — and the failure is not silent, because the reconcile-error
/// counter has already moved and the log carries both errors.
///
/// Requires the `events.k8s.io` / `events` `create,patch` grant in the
/// operator's ClusterRole.
async fn publish<S: ManagedService>(
    recorder: &Recorder,
    obj: &S,
    type_: EventType,
    reason: &str,
    action: &str,
    note: String,
) {
    let event = Event {
        type_,
        reason: reason.to_string(),
        note: Some(note),
        action: action.to_string(),
        secondary: None,
    };
    if let Err(error) = recorder.publish(&event, &obj.object_ref(&())).await {
        tracing::warn!(%error, reason, "failed to publish event");
    }
}

/// What the controller does with a failed reconcile.
///
/// Until #2620 this discarded the error entirely and returned a bare requeue,
/// which made a CR failing every single round externally identical to one
/// converging fine: nothing counted the failure, and the object's owner was
/// never told. Both halves of that are fixed here — the counter feeds the
/// error-rate alert, the Event feeds `kubectl describe`.
fn error_policy<S: ManagedService>(obj: Arc<S>, err: &Error, ctx: Arc<Ctx>) -> Action {
    ctx.metrics.observe_error();
    tracing::warn!(
        error = %err,
        object = %obj.name_any(),
        namespace = obj.namespace().unwrap_or_default(),
        "reconcile failed"
    );

    // `error_policy` is synchronous by the controller's contract, so the write
    // is detached. The `Recorder`'s 6-minute dedup window collapses a
    // repeatedly failing reconcile into one counted series rather than a flood.
    let recorder = ctx.recorder.clone();
    let note = err.to_string();
    tokio::spawn(async move {
        publish(
            &recorder,
            obj.as_ref(),
            EventType::Warning,
            "ReconcileFailed",
            "Reconcile",
            note,
        )
        .await;
    });

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

    #[derive(
        kube::CustomResource, Clone, Debug, serde::Deserialize, serde::Serialize, schemars::JsonSchema,
    )]
    #[kube(
        group = "service-k8s.test",
        version = "v1",
        kind = "CountedService",
        namespaced
    )]
    struct CountedServiceSpec {
        replicas: u32,
    }

    impl ManagedService for CountedService {
        const MANAGER: &'static str = "counted-operator";

        fn render(&self) -> Vec<Value> {
            vec![json!({
                "apiVersion": "apps/v1",
                "kind": "Deployment",
                "metadata": { "name": "counted" },
                "spec": { "replicas": self.spec.replicas },
            })]
        }

        fn readiness_targets(&self) -> Vec<service::ReadinessTarget> {
            Vec::new()
        }

        fn status_patch(&self, _ready: &ReadyFacts) -> Value {
            json!({ "status": {} })
        }
    }

    fn counted_ctx(client: Client, leader: bool) -> Arc<Ctx> {
        let election = Election::new("test-identity".to_string());
        election.is_leader.store(leader, Ordering::Relaxed);
        let recorder = Recorder::new(
            client.clone(),
            Reporter {
                controller: CountedService::MANAGER.to_string(),
                instance: Some("test-identity".to_string()),
            },
        );
        Arc::new(Ctx {
            client,
            election,
            metrics: Arc::new(ControllerMetrics::new(CountedService::MANAGER)),
            recorder,
        })
    }

    fn counted_obj() -> Arc<CountedService> {
        let mut obj = CountedService::new("counted", CountedServiceSpec { replicas: 1 });
        obj.metadata.namespace = Some("acme".to_string());
        Arc::new(obj)
    }

    /// The error counter is the numerator of the alert that pages a human when
    /// a control plane stops converging. Before #2620 `error_policy` discarded
    /// the error and returned the same `Action` either way — so asserting on
    /// the return value alone would pass against the unfixed code, and the
    /// counter is the only observation that actually distinguishes them.
    #[tokio::test]
    async fn a_failed_reconcile_is_counted_rather_than_discarded() {
        let (client, _) = fake_apiserver(vec![(200, json!({}))]);
        let ctx = counted_ctx(client, true);
        assert_eq!(ctx.metrics.reconcile_errors_total(), 0);

        let action = error_policy::<CountedService>(
            counted_obj(),
            &Error::Missing("metadata.namespace"),
            ctx.clone(),
        );

        assert_eq!(ctx.metrics.reconcile_errors_total(), 1);
        assert_eq!(action, Action::requeue(Duration::from_secs(15)));
    }

    /// Both operator replicas run the watch loop, but only the leader applies.
    /// If a follower's no-op counted as a reconcile, the idle replica would
    /// report a steadily climbing `_reconcile_total` with zero errors — which
    /// is exactly what a healthy working operator looks like, on the replica
    /// that has never touched the cluster.
    #[tokio::test]
    async fn a_follower_replica_records_no_reconcile_at_all() {
        let (client, seen) = fake_apiserver(vec![]);
        let ctx = counted_ctx(client, false);

        let action = reconcile_entry::<CountedService>(counted_obj(), ctx.clone())
            .await
            .expect("a follower short-circuits successfully");

        assert_eq!(action, Action::requeue(Duration::from_secs(10)));
        assert_eq!(ctx.metrics.reconcile_total(), 0);
        assert!(
            seen.lock().unwrap().is_empty(),
            "a follower must not talk to the apiserver: {:?}",
            seen.lock().unwrap()
        );
    }

    /// The denominator counts attempts, not successes. A reconcile that fails
    /// against the apiserver still took time and still happened, so it has to
    /// land in `_reconcile_total` and in the duration histogram — otherwise an
    /// operator failing everything divides by zero.
    #[tokio::test]
    async fn a_leader_counts_the_attempt_even_when_it_fails() {
        let (client, _) = fake_apiserver(vec![(
            500,
            json!({ "kind": "Status", "status": "Failure", "code": 500 }),
        )]);
        let ctx = counted_ctx(client, true);

        let result = reconcile_entry::<CountedService>(counted_obj(), ctx.clone()).await;

        assert!(result.is_err(), "the fake apiserver rejected the apply");
        assert_eq!(ctx.metrics.reconcile_total(), 1);
        assert!(ctx
            .metrics
            .render(true)
            .contains("counted_operator_reconcile_duration_seconds_count 1"));
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
// CODEGEN-END
