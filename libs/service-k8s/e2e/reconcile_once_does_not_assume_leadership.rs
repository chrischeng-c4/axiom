//! One reconcile pass runs under the caller's leadership decision, never one
//! the library invented for it.
//!
//! [`service_k8s::controller::reconcile_once`] is the crate's only observation
//! point on the convergence sequence, and it is `pub`. It used to build its own
//! [`Election`] and store `is_leader = true` into it — so every caller,
//! including a production binary that never took the Lease, got a pass that
//! walked straight through the leader gate in `reconcile_entry`. The gate is
//! the whole reason `replicas > 1` is safe; a public function that bypasses it
//! is a second, unguarded way into the same sequence.
//!
//! The fix is a parameter, not a rename: leadership is a fact the caller holds
//! and hands over. A test that wants a leader says so at its own call site, in
//! two visible lines; a caller that has no Lease cannot accidentally claim one.
//!
//! | Case | What it pins |
//! |---|---|
//! | a follower does nothing | a pass whose election says "not leader" issues no request at all and asks to come back |
//! | a leader still converges | the parameter did not turn the gate into an unconditional refusal |
//!
//! The first red for this file was a compile error — `reconcile_once` took no
//! election, so "run a pass as a follower" was not expressible. That is the
//! finding restated: the only leadership the function accepted was its own.

use std::sync::atomic::Ordering;
use std::sync::{Arc, Mutex};

use kube::client::Body;
use kube::{Client, CustomResource};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use service_k8s::controller::reconcile_once;
use service_k8s::service::{ManagedService, ReadinessTarget, ReadyFacts};
use service_k8s::Election;

const NAMESPACE: &str = "acme";
const CHILD: &str = "follower-child";

type Log = Arc<Mutex<Vec<String>>>;

/// A fake apiserver that records `METHOD path` for everything it is asked.
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
            let _ = req.into_body().collect_bytes().await;
            let (code, response) = route(&method, &path);
            seen.lock().unwrap().push(format!("{method} {path}"));
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
    kind = "FollowerService",
    namespaced
)]
struct FollowerServiceSpec {
    child: String,
}

impl ManagedService for FollowerService {
    const MANAGER: &'static str = "follower-e2e-operator";

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
}

fn subject() -> Arc<FollowerService> {
    let mut obj = FollowerService::new(
        "follower",
        FollowerServiceSpec {
            child: CHILD.to_string(),
        },
    );
    obj.metadata.namespace = Some(NAMESPACE.to_string());
    obj.metadata.uid = Some("uid-e2e-leadership".to_string());
    Arc::new(obj)
}

fn election(is_leader: bool) -> Arc<Election> {
    let election = Election::new("leadership-e2e".to_string());
    election.is_leader.store(is_leader, Ordering::Relaxed);
    election
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

fn cr_response() -> Value {
    json!({
        "apiVersion": "service-k8s.e2e/v1",
        "kind": "FollowerService",
        "metadata": { "name": "follower", "namespace": NAMESPACE, "uid": "uid-e2e-leadership" },
        "spec": { "child": CHILD },
    })
}

fn routes(method: &str, path: &str) -> (u16, Value) {
    match (method, path) {
        ("PATCH", p) if p.ends_with("/deployments/follower-child") => (200, applied_child()),
        ("GET", p) if p.ends_with("/deployments/follower-child") => (200, applied_child()),
        ("PATCH", p) if p.ends_with("/followerservices/follower/status") => (200, cr_response()),
        ("POST", p) if p.ends_with("/events") => (201, json!({ "metadata": { "name": "e" } })),
        _ => (500, json!({ "kind": "Status", "status": "Failure", "code": 500 })),
    }
}

/// The case. A pass driven by a replica that does not hold the Lease must not
/// touch the cluster — not the child apply, not the status write, nothing.
#[tokio::test]
async fn a_follower_pass_issues_no_request() {
    let (client, log) = fake_apiserver(routes);

    let action = reconcile_once(client, subject(), election(false))
        .await
        .expect("a follower has nothing to do, which is not an error");

    assert!(
        format!("{action:?}").contains("requeue"),
        "a follower comes back to check whether it has become the leader: {action:?}"
    );
    let seen = log.lock().unwrap().clone();
    assert!(
        seen.is_empty(),
        "the leader gate is what makes `replicas > 1` safe; a pass that ran anyway \
         issued: {seen:?}"
    );
}

/// Control. The parameter must gate the pass, not disable it — the leader's
/// pass still applies the child, observes readiness, and writes status.
#[tokio::test]
async fn a_leader_pass_still_converges() {
    let (client, log) = fake_apiserver(routes);

    reconcile_once(client, subject(), election(true))
        .await
        .expect("the leader converges");

    let seen = log.lock().unwrap().clone();
    assert!(
        seen.iter()
            .any(|e| e.starts_with("PATCH") && e.ends_with("/deployments/follower-child")),
        "the leader applies the child: {seen:?}"
    );
    assert!(
        seen.iter()
            .any(|e| e.starts_with("PATCH") && e.ends_with("/followerservices/follower/status")),
        "the leader writes status: {seen:?}"
    );
}
