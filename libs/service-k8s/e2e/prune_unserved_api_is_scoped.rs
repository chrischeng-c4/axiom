//! An unserved prune API is scoped to the prune, not to the whole reconcile
//! (#3079).
//!
//! Black box on purpose. This file stands up its own fake apiserver, drives one
//! full pass through the crate's public
//! [`service_k8s::controller::reconcile_once`], and asserts only on what that
//! apiserver saw: which requests arrived, in what order, and what the status
//! write carried. Nothing here reads a controller internal, so the case cannot
//! pass because of the shape the fix took — only because the writes it demands
//! were actually issued.
//!
//! The situation it pins: a `ManagedService` naming a prune target whose API
//! the cluster does not serve. That GET is answered by the apiserver mux rather
//! than by a resource handler, so its body is not a `Status` object, and
//! `kube`'s `get_opt` — whose one swallow is a `Status` reading `NotFound` —
//! hands it back as an error. Before this change that error left `reconcile`
//! through `?`, ahead of readiness observation and ahead of the status write,
//! so the CR received no status subresource at all and nobody was told why.
//!
//! The apiserver here answers by *route*, not by queue position. A full
//! reconcile's request count is part of what is under test, and a queue would
//! turn one extra or missing request into a cascade of mismatched responses
//! instead of one failed assertion.

use std::sync::{Arc, Mutex};

use kube::client::Body;
use kube::{Client, CustomResource};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use service_k8s::controller::reconcile_once;
use service_k8s::service::{ManagedService, PruneTarget, ReadinessTarget, ReadyFacts};

const NAMESPACE: &str = "acme";
const CHILD: &str = "pruned-child";
const PRUNE_TARGET: &str = "pruned-policy";

/// One request the fake apiserver answered.
#[derive(Clone, Debug)]
struct Recorded {
    method: String,
    path: String,
    body: Value,
}

type Log = Arc<Mutex<Vec<Recorded>>>;

/// A fake apiserver that answers `route(method, path)` and records every
/// request it was asked, body included.
fn fake_apiserver(route: impl Fn(&str, &str) -> (u16, Value) + Send + Sync + 'static) -> (Client, Log) {
    let log: Log = Arc::new(Mutex::new(Vec::new()));
    let seen = log.clone();
    let route = Arc::new(route);
    let service = tower::service_fn(move |req: http::Request<Body>| {
        let seen = seen.clone();
        let route = route.clone();
        async move {
            let method = req.method().to_string();
            let path = req.uri().path().to_string();
            let bytes = req.into_body().collect_bytes().await.unwrap_or_default();
            let body = serde_json::from_slice(&bytes).unwrap_or(Value::Null);
            let (code, response) = route(&method, &path);
            seen.lock().unwrap().push(Recorded {
                method,
                path,
                body,
            });
            Ok::<_, std::convert::Infallible>(
                http::Response::builder()
                    .status(code)
                    .header("content-type", "application/json")
                    .body(Body::from(serde_json::to_vec(&response).unwrap()))
                    .unwrap(),
            )
        }
    });
    (Client::new(service, NAMESPACE), log)
}

#[derive(CustomResource, Clone, Debug, Deserialize, Serialize, JsonSchema)]
#[kube(
    group = "service-k8s.e2e",
    version = "v1",
    kind = "PruningService",
    namespaced
)]
struct PruningServiceSpec {
    child: String,
}

impl ManagedService for PruningService {
    const MANAGER: &'static str = "pruning-e2e-operator";

    fn render(&self) -> Vec<Value> {
        vec![json!({
            "apiVersion": "apps/v1",
            "kind": "Deployment",
            "metadata": { "name": self.spec.child },
            "spec": { "replicas": 2 },
        })]
    }

    fn readiness_targets(&self) -> Vec<ReadinessTarget> {
        vec![ReadinessTarget {
            kind: "Deployment",
            name: self.spec.child.clone(),
        }]
    }

    fn status_patch(&self, ready: &ReadyFacts) -> Value {
        json!({ "status": { "readyReplicas": ready.get(&self.spec.child) } })
    }

    /// The whole subject of this file: a target the spec no longer wants.
    fn prunes(&self) -> Vec<PruneTarget> {
        vec![PruneTarget {
            api_version: "networking.k8s.io/v1",
            kind: "NetworkPolicy",
            name: PRUNE_TARGET.to_string(),
        }]
    }
}

fn subject() -> Arc<PruningService> {
    let mut obj = PruningService::new(
        "pruned",
        PruningServiceSpec {
            child: CHILD.to_string(),
        },
    );
    obj.metadata.namespace = Some(NAMESPACE.to_string());
    // Pruning is gated on the CR's own UID: the controller re-checks the live
    // object's controller `ownerReference` against it before deleting.
    obj.metadata.uid = Some("uid-e2e-3079".to_string());
    Arc::new(obj)
}

/// The CR body the status write is answered with, so `patch_status` has
/// something of the right type to deserialize.
fn cr_response() -> Value {
    json!({
        "apiVersion": "service-k8s.e2e/v1",
        "kind": "PruningService",
        "metadata": { "name": "pruned", "namespace": NAMESPACE, "uid": "uid-e2e-3079" },
        "spec": { "child": CHILD },
    })
}

fn applied_child() -> Value {
    json!({
        "apiVersion": "apps/v1",
        "kind": "Deployment",
        "metadata": { "name": CHILD, "namespace": NAMESPACE },
        "spec": { "replicas": 2 },
        "status": { "readyReplicas": 2 },
    })
}

/// A 404 from the apiserver mux rather than from a resource handler: nothing
/// routed the request, so the body is plain text and not a `Status` object.
/// This is what a cluster that does not serve an API group returns.
fn unserved_api() -> (u16, Value) {
    (404, json!("404 page not found"))
}

/// A 404 from the resource handler: the API is served and the object is not
/// there. `get_opt` maps exactly this shape to `Ok(None)`.
fn served_but_absent() -> (u16, Value) {
    (
        404,
        json!({ "kind": "Status", "status": "Failure", "message": "not found",
                "reason": "NotFound", "code": 404 }),
    )
}

fn methods_and_paths(log: &Log) -> Vec<String> {
    log.lock()
        .unwrap()
        .iter()
        .map(|r| format!("{} {}", r.method, r.path))
        .collect()
}

fn position(log: &[String], method: &str, suffix: &str) -> Option<usize> {
    log.iter()
        .position(|entry| entry.starts_with(method) && entry.ends_with(suffix))
}

fn status_write(log: &Log) -> Recorded {
    let writes: Vec<Recorded> = log
        .lock()
        .unwrap()
        .iter()
        .filter(|r| r.method == "PATCH" && r.path.ends_with("/pruningservices/pruned/status"))
        .cloned()
        .collect();
    assert_eq!(
        writes.len(),
        1,
        "expected exactly one status write; the whole reconcile was: {:?}",
        methods_and_paths(log)
    );
    writes.into_iter().next().unwrap()
}

/// The case. A prune target whose API the cluster does not serve must cost the
/// prune and nothing else: the children still get applied, readiness still gets
/// observed, and the status subresource still gets written — carrying a
/// condition that names the target nobody could remove.
#[tokio::test]
async fn an_unserved_prune_api_still_reaches_the_status_write() {
    let (client, log) = fake_apiserver(|method, path| match (method, path) {
        ("PATCH", p) if p.ends_with("/deployments/pruned-child") => (200, applied_child()),
        ("GET", p) if p.ends_with("/deployments/pruned-child") => (200, applied_child()),
        ("GET", p) if p.ends_with("/networkpolicies/pruned-policy") => unserved_api(),
        ("PATCH", p) if p.ends_with("/pruningservices/pruned/status") => (200, cr_response()),
        // Event publication is best-effort narration and not what this case is
        // about; answering it 201 keeps it out of the way without hiding it
        // from the request log.
        ("POST", p) if p.ends_with("/events") => (201, json!({ "metadata": { "name": "e" } })),
        _ => (500, json!({ "kind": "Status", "status": "Failure", "code": 500 })),
    });

    // The reconcile's own return value is checked at the end, not here. What
    // this case is about is the writes the pass issued, and asserting the
    // result first would report "the reconcile returned an error" for a
    // regression whose actual symptom is a CR left with no status at all.
    let action = reconcile_once(client, subject()).await;

    let seen = methods_and_paths(&log);

    // The status write is the observation the whole case turns on: before
    // #3079 the reconcile left through `?` at the prune GET and this request
    // was never issued.
    let status = status_write(&log);

    // …and it happened *after* the prune GET, which is what says the reconcile
    // carried on past the failure rather than never having attempted it.
    let prune_get = position(&seen, "GET", "/networkpolicies/pruned-policy")
        .unwrap_or_else(|| panic!("the prune GET was never issued: {seen:?}"));
    let status_patch = position(&seen, "PATCH", "/pruningservices/pruned/status")
        .unwrap_or_else(|| panic!("the status write was never issued: {seen:?}"));
    let child_apply = position(&seen, "PATCH", "/deployments/pruned-child")
        .unwrap_or_else(|| panic!("the child was never applied: {seen:?}"));
    let readiness_get = position(&seen, "GET", "/deployments/pruned-child")
        .unwrap_or_else(|| panic!("readiness was never observed: {seen:?}"));
    assert!(
        child_apply < prune_get && prune_get < readiness_get && readiness_get < status_patch,
        "apply → prune → readiness → status is the sequence under test: {seen:?}"
    );

    // Readiness reached the status body, so the pass observed the cluster
    // rather than short-circuiting to a bare condition.
    assert_eq!(
        status.body["status"]["readyReplicas"], 2,
        "status write: {}",
        status.body
    );

    let conditions = status.body["status"]["conditions"]
        .as_array()
        .unwrap_or_else(|| panic!("status carried no conditions array: {}", status.body));
    let blocked = conditions
        .iter()
        .find(|c| c["type"] == "PruneBlocked")
        .unwrap_or_else(|| panic!("no PruneBlocked condition in {conditions:?}"));
    assert_eq!(blocked["status"], "True", "{blocked}");
    let message = blocked["message"].as_str().unwrap_or_default();
    assert!(
        message.contains(PRUNE_TARGET) && message.contains("networking.k8s.io/v1"),
        "the condition has to name the target nobody could prune: {blocked}"
    );

    // The reconcile converged everything it could, so it asks to come back
    // rather than reporting an operator failure a restart could fix.
    let action = action
        .expect("an API the cluster does not serve is a cluster fact, not an operator error");
    assert!(
        format!("{action:?}").contains("requeue"),
        "an unavailable prune target requeues: {action:?}"
    );
}

/// The control. Every assertion above would also pass against a change that
/// simply appends `PruneBlocked` on every round, so the served case has to
/// pin the other side: an API the cluster does serve, whose object is already
/// gone, converges silently and writes no condition at all.
#[tokio::test]
async fn a_served_prune_api_writes_no_block() {
    let (client, log) = fake_apiserver(|method, path| match (method, path) {
        ("PATCH", p) if p.ends_with("/deployments/pruned-child") => (200, applied_child()),
        ("GET", p) if p.ends_with("/deployments/pruned-child") => (200, applied_child()),
        ("GET", p) if p.ends_with("/networkpolicies/pruned-policy") => served_but_absent(),
        ("PATCH", p) if p.ends_with("/pruningservices/pruned/status") => (200, cr_response()),
        ("POST", p) if p.ends_with("/events") => (201, json!({ "metadata": { "name": "e" } })),
        _ => (500, json!({ "kind": "Status", "status": "Failure", "code": 500 })),
    });

    reconcile_once(client, subject())
        .await
        .expect("an absent object at a served API is the converged steady state");

    let status = status_write(&log);
    assert!(
        status.body["status"].get("conditions").is_none(),
        "a service declaring no conditions of its own, with nothing blocked, \
         must write no conditions array: {}",
        status.body
    );
}
