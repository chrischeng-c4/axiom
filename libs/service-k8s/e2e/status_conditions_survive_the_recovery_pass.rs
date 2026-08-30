//! A stale `PruneBlocked` must not change what happens to anyone else's
//! conditions.
//!
//! `PruneBlocked` is the controller's own (#3079): no service can report it,
//! because the prune GET is the controller's call. Everything else in
//! `status.conditions[]` belongs to the service.
//!
//! The gate that writes the array is
//! `!facts.is_empty() || <a prior PruneBlocked>`. The second disjunct is what
//! #3079 added, and it made a stale block change the meaning of a pass that has
//! nothing to say: `project` returns exactly the facts it is handed, so an
//! empty-facts pass wrote `conditions: []`, and `Patch::Merge` replaces an
//! array wholesale. The same pass without a stale block writes nothing and
//! leaves the array alone.
//!
//! So the invariant is differential: **whether a stale `PruneBlocked` is
//! present must not decide whether the service's own conditions survive.** The
//! controller authors exactly one condition, so it may remove exactly one.
//!
//! A service declares nothing on a pass whenever its conditions are
//! conditional — one reported only while degraded, say, on a pass where it is
//! healthy. "The service declared nothing" is not "the service declared that
//! nothing is true".
//!
//! | Case | What it pins |
//! |---|---|
//! | recovery keeps the service's conditions | only `PruneBlocked` leaves; the rest of the array is what the pass found, transition times and observed generations included |
//! | no block, no facts, no write | the other half of the differential: this is the behaviour the case above has to match |
//! | a declared set still replaces wholesale | a pass that *does* declare facts keeps `project`'s documented behaviour — this is not a change to it |
//! | a block still lands beside the service's own | subtraction did not turn into "never write `PruneBlocked` again" |
//!
//! Black box, like `prune_unserved_api_is_scoped.rs` beside it: the assertions
//! read the body the fake apiserver received, never a controller internal.

use std::sync::{Arc, Mutex};

use kube::client::Body;
use kube::{Client, CustomResource};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use service_k8s::controller::reconcile_once;
use service_k8s::service::{
    Condition, ConditionFact, ConditionStatus, ManagedService, PruneTarget, ReadinessTarget,
    ReadyFacts,
};
use service_k8s::Election;

const NAMESPACE: &str = "acme";
const CHILD: &str = "recovering-child";
const PRUNE_TARGET: &str = "recovering-policy";
/// The instant the service's own condition last changed. A recovery pass that
/// preserves the condition but restamps this has still lost the fact the field
/// carries, so the cases assert the value and not just the presence.
const SINCE: &str = "2026-01-01T00:00:00Z";

#[derive(Clone, Debug)]
struct Recorded {
    method: String,
    path: String,
    body: Value,
}

type Log = Arc<Mutex<Vec<Recorded>>>;

fn fake_apiserver(
    route: impl Fn(&str, &str) -> (u16, Value) + Send + Sync + 'static,
) -> (Client, Log) {
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
            seen.lock().unwrap().push(Recorded { method, path, body });
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
    kind = "RecoveringService",
    namespaced,
    status = "RecoveringServiceStatus"
)]
struct RecoveringServiceSpec {
    child: String,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize, JsonSchema)]
struct RecoveringServiceStatus {
    #[serde(default)]
    conditions: Vec<Condition>,
}

impl ManagedService for RecoveringService {
    const MANAGER: &'static str = "recovering-e2e-operator";

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

    /// Conditional, which is the shape the whole file is about: a healthy pass
    /// has nothing to declare. Services that report a fixed set every pass —
    /// lumen's and tape's do — never reach the branch under test, which is why
    /// the crate's own controller e2e could not see this.
    fn conditions(&self, ready: &ReadyFacts, _context: &Value) -> Vec<ConditionFact> {
        if ready.get(&self.spec.child) > 0 {
            Vec::new()
        } else {
            vec![ConditionFact::new(
                "Degraded",
                ConditionStatus::True,
                "NoReadyReplicas",
                "the child reports no ready replicas",
            )]
        }
    }

    fn observed_conditions(&self) -> Vec<Condition> {
        self.status
            .as_ref()
            .map(|s| s.conditions.clone())
            .unwrap_or_default()
    }

    fn prunes(&self) -> Vec<PruneTarget> {
        vec![PruneTarget {
            api_version: "networking.k8s.io/v1",
            kind: "NetworkPolicy",
            name: PRUNE_TARGET.to_string(),
        }]
    }
}

fn condition(type_: &str, status: &str, since: &str) -> Condition {
    Condition {
        type_: type_.to_string(),
        status: status.to_string(),
        reason: "Recorded".to_string(),
        message: format!("{type_} as a previous pass left it"),
        last_transition_time: since.to_string(),
        observed_generation: Some(4),
    }
}

/// A CR whose persisted status already carries `prior`.
fn subject(prior: Vec<Condition>) -> Arc<RecoveringService> {
    let mut obj = RecoveringService::new(
        "recovering",
        RecoveringServiceSpec {
            child: CHILD.to_string(),
        },
    );
    obj.metadata.namespace = Some(NAMESPACE.to_string());
    obj.metadata.uid = Some("uid-e2e-recovery".to_string());
    obj.metadata.generation = Some(4);
    obj.status = Some(RecoveringServiceStatus { conditions: prior });
    Arc::new(obj)
}

/// An election that holds the lease. These cases measure what a pass writes to
/// the apiserver, so they have to be a leader's passes; leadership is
/// `reconcile_once`'s parameter, stated here rather than assumed by it.
fn leader() -> Arc<Election> {
    let election = Election::new("status-conditions-e2e".to_string());
    election
        .is_leader
        .store(true, std::sync::atomic::Ordering::Relaxed);
    election
}

fn cr_response() -> Value {
    json!({
        "apiVersion": "service-k8s.e2e/v1",
        "kind": "RecoveringService",
        "metadata": { "name": "recovering", "namespace": NAMESPACE, "uid": "uid-e2e-recovery" },
        "spec": { "child": CHILD },
    })
}

/// The child as the apiserver reports it once it is up: the service is healthy
/// this pass and therefore declares nothing.
fn healthy_child() -> Value {
    json!({
        "apiVersion": "apps/v1",
        "kind": "Deployment",
        "metadata": { "name": CHILD, "namespace": NAMESPACE },
        "spec": { "replicas": 2 },
        "status": { "readyReplicas": 2 },
    })
}

/// The child with no ready replicas: the service declares `Degraded`.
fn degraded_child() -> Value {
    json!({
        "apiVersion": "apps/v1",
        "kind": "Deployment",
        "metadata": { "name": CHILD, "namespace": NAMESPACE },
        "spec": { "replicas": 2 },
        "status": { "readyReplicas": 0 },
    })
}

/// A 404 from the apiserver mux: the API group is not served at all.
fn unserved_api() -> (u16, Value) {
    (404, json!("404 page not found"))
}

/// A 404 from the resource handler: served, and the object is gone.
fn served_but_absent() -> (u16, Value) {
    (
        404,
        json!({ "kind": "Status", "status": "Failure", "message": "not found",
                "reason": "NotFound", "code": 404 }),
    )
}

fn status_write(log: &Log) -> Value {
    let writes: Vec<Value> = log
        .lock()
        .unwrap()
        .iter()
        .filter(|r| {
            r.method == "PATCH" && r.path.ends_with("/recoveringservices/recovering/status")
        })
        .map(|r| r.body.clone())
        .collect();
    assert_eq!(writes.len(), 1, "expected exactly one status write");
    writes.into_iter().next().unwrap()
}

fn conditions_of(body: &Value) -> Vec<Value> {
    body["status"]["conditions"]
        .as_array()
        .unwrap_or_else(|| panic!("the status write carried no conditions array: {body}"))
        .clone()
}

fn types_of(conditions: &[Value]) -> Vec<String> {
    conditions
        .iter()
        .map(|c| c["type"].as_str().unwrap_or_default().to_string())
        .collect()
}

/// The case. The prune API is served again, so `PruneBlocked` has to go — and
/// the child is healthy, so the service declares nothing this pass. The write
/// must subtract the controller's condition and leave the service's untouched.
#[tokio::test]
async fn a_recovery_pass_removes_only_the_condition_the_controller_authored() {
    let (client, log) = fake_apiserver(|method, path| match (method, path) {
        ("PATCH", p) if p.ends_with("/deployments/recovering-child") => (200, healthy_child()),
        ("GET", p) if p.ends_with("/deployments/recovering-child") => (200, healthy_child()),
        // The prune API is back, so nothing is blocked any more.
        ("GET", p) if p.ends_with("/networkpolicies/recovering-policy") => served_but_absent(),
        ("PATCH", p) if p.ends_with("/recoveringservices/recovering/status") => {
            (200, cr_response())
        }
        ("POST", p) if p.ends_with("/events") => (201, json!({ "metadata": { "name": "e" } })),
        _ => (500, json!({ "kind": "Status", "status": "Failure", "code": 500 })),
    });

    reconcile_once(
        client,
        subject(vec![
            condition("Degraded", "True", SINCE),
            condition("PruneBlocked", "True", "2026-02-02T00:00:00Z"),
        ]),
        leader(),
    )
    .await
    .expect("a served, absent prune target is the converged steady state");

    let conditions = conditions_of(&status_write(&log));

    assert_eq!(
        types_of(&conditions),
        vec!["Degraded".to_string()],
        "the recovery pass withdraws `PruneBlocked` and nothing else; the service's \
         own conditions are not the controller's to delete. Got: {conditions:?}"
    );
    // Carried forward as found — not re-projected. A pass with no facts has no
    // opinion about this condition, so none of its fields may move.
    assert_eq!(
        conditions[0],
        json!({
            "type": "Degraded",
            "status": "True",
            "reason": "Recorded",
            "message": "Degraded as a previous pass left it",
            "lastTransitionTime": SINCE,
            "observedGeneration": 4,
        }),
        "a condition the pass carried forward untouched must arrive byte for byte"
    );
}

/// The other half of the differential, and the behaviour the case above has to
/// match. Same pass, same healthy child, same empty facts — only the stale
/// block is gone. This one writes no conditions at all, which is why the one
/// above may not write an empty array.
#[tokio::test]
async fn without_a_stale_block_the_same_pass_writes_no_conditions_at_all() {
    let (client, log) = fake_apiserver(|method, path| match (method, path) {
        ("PATCH", p) if p.ends_with("/deployments/recovering-child") => (200, healthy_child()),
        ("GET", p) if p.ends_with("/deployments/recovering-child") => (200, healthy_child()),
        ("GET", p) if p.ends_with("/networkpolicies/recovering-policy") => served_but_absent(),
        ("PATCH", p) if p.ends_with("/recoveringservices/recovering/status") => {
            (200, cr_response())
        }
        ("POST", p) if p.ends_with("/events") => (201, json!({ "metadata": { "name": "e" } })),
        _ => (500, json!({ "kind": "Status", "status": "Failure", "code": 500 })),
    });

    reconcile_once(client, subject(vec![condition("Degraded", "True", SINCE)]), leader())
        .await
        .expect("nothing blocked, nothing declared");

    let body = status_write(&log);
    assert!(
        body["status"]["conditions"].is_null(),
        "a pass with no facts and no block leaves the array alone, so the persisted \
         `Degraded` survives. Whether a stale `PruneBlocked` was present must not \
         change that: {body}"
    );
}

/// Control. A pass that *does* declare facts still replaces the array
/// wholesale — a service declares its whole set every pass, and a prior
/// condition it stopped declaring is meant to disappear. Green on both sides
/// of the change; without it, "preserve the prior array" would look like the
/// fix and would freeze every condition a service ever wrote.
#[tokio::test]
async fn a_declared_set_still_replaces_the_array_wholesale() {
    let (client, log) = fake_apiserver(|method, path| match (method, path) {
        ("PATCH", p) if p.ends_with("/deployments/recovering-child") => (200, degraded_child()),
        ("GET", p) if p.ends_with("/deployments/recovering-child") => (200, degraded_child()),
        ("GET", p) if p.ends_with("/networkpolicies/recovering-policy") => served_but_absent(),
        ("PATCH", p) if p.ends_with("/recoveringservices/recovering/status") => {
            (200, cr_response())
        }
        ("POST", p) if p.ends_with("/events") => (201, json!({ "metadata": { "name": "e" } })),
        _ => (500, json!({ "kind": "Status", "status": "Failure", "code": 500 })),
    });

    reconcile_once(
        client,
        subject(vec![
            condition("Degraded", "True", SINCE),
            condition("Stale", "True", "2026-03-03T00:00:00Z"),
        ]),
        leader(),
    )
    .await
    .expect("readiness observed and nothing blocked");

    let conditions = conditions_of(&status_write(&log));
    assert_eq!(
        types_of(&conditions),
        vec!["Degraded".to_string()],
        "`Stale` is not in this pass's declared set, so it leaves: {conditions:?}"
    );
    assert_eq!(
        conditions[0]["lastTransitionTime"], SINCE,
        "`Degraded` was re-declared with the same status, so `project` still carries \
         its transition time forward: {conditions:?}"
    );
}

/// Control. Subtraction must not become "the controller never writes its own
/// condition again": an API that is still unserved still has to be reported,
/// beside whatever the service declared.
#[tokio::test]
async fn a_block_still_lands_beside_the_service_conditions() {
    let (client, log) = fake_apiserver(|method, path| match (method, path) {
        ("PATCH", p) if p.ends_with("/deployments/recovering-child") => (200, degraded_child()),
        ("GET", p) if p.ends_with("/deployments/recovering-child") => (200, degraded_child()),
        ("GET", p) if p.ends_with("/networkpolicies/recovering-policy") => unserved_api(),
        ("PATCH", p) if p.ends_with("/recoveringservices/recovering/status") => {
            (200, cr_response())
        }
        ("POST", p) if p.ends_with("/events") => (201, json!({ "metadata": { "name": "e" } })),
        _ => (500, json!({ "kind": "Status", "status": "Failure", "code": 500 })),
    });

    reconcile_once(client, subject(Vec::new()), leader())
        .await
        .expect("an unserved prune API is a cluster fact, not an operator error");

    let conditions = conditions_of(&status_write(&log));
    let mut types = types_of(&conditions);
    types.sort();
    assert_eq!(
        types,
        vec!["Degraded".to_string(), "PruneBlocked".to_string()],
        "the service's declared set and the controller's own condition both belong \
         in the write: {conditions:?}"
    );
}
