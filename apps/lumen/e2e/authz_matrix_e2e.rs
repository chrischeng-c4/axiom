// SPEC-MANAGED: apps/lumen/tech-design/semantic/lumen-tests.md#unit-test
// CODEGEN-BEGIN
//! Authorization matrix (TEST-STRATEGY security gate) over delegated auth (#2869).
//!
//! Every check runs through the real router with a scripted `ReviewBackend`
//! standing in for kube-apiserver, so what is under test is the whole path a
//! request actually takes: middleware authenticates once, the handler asks
//! `SubjectAccessReview` about the operation it is about to perform.
//!
//! The matrix has three columns, and each one catches a different hole:
//!
//! - **401** — no credential, and a credential kube-apiserver verified but that
//!   is not a ServiceAccount (a Google user, a GSA). Those principals
//!   authenticate to the apiserver, never to Lumen.
//! - **403** — an authenticated ServiceAccount without the grant for *that*
//!   operation. This column is per-verb on purpose: `delete` does not imply
//!   `get`, because RBAC has no such hierarchy and neither does Lumen.
//! - **2xx** — the same endpoint with exactly its own grant, so the 403 column
//!   cannot pass by accident on a route that is simply broken.
//!
//! Plus the two cases that are neither: a collection listing is *filtered*, not
//! denied, and an apiserver that cannot answer yields 503 — never 200, never a
//! 403 that would look like a settled policy decision.

use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use axum_test::TestServer;
use serde_json::{json, Value};

use lumen::api::{router, AppState};
use lumen::auth::{AuthConfig, LumenVerifier, ADMIN_RESOURCE, COLLECTIONS_RESOURCE};
use lumen::storage::Engine;
use service_auth::k8s::{
    AccessReviewOutcome, ResourceAttributes, ReviewBackend, ReviewError, ReviewedIdentity,
    TokenReviewOutcome,
};

const NAMESPACE: &str = "serving";
const COLLECTION: &str = "users";

const ADMIN: &str = "system:serviceaccount:serving:lumen-admin";
const WRITER: &str = "system:serviceaccount:serving:lumen-writer";
const READER: &str = "system:serviceaccount:serving:lumen-reader";
/// A ServiceAccount kube-apiserver knows and Lumen grants nothing to.
const STRANGER: &str = "system:serviceaccount:serving:lumen-stranger";
/// A human kube-apiserver authenticates. Never a Lumen caller.
const GOOGLE_USER: &str = "alice@example.com";

/// A scripted apiserver: a token table, an explicit grant list, and a switch
/// for the outage case.
struct Cluster {
    tokens: HashMap<&'static str, &'static str>,
    /// `(username, resource, name, verb)` — the exact tuple a
    /// `SubjectAccessReview` would be asked about.
    grants: HashSet<(String, String, Option<String>, String)>,
    access_reachable: bool,
    asked: Mutex<Vec<ResourceAttributes>>,
}

impl Cluster {
    fn new() -> Self {
        let tokens = HashMap::from([
            ("t-admin", ADMIN),
            ("t-writer", WRITER),
            ("t-reader", READER),
            ("t-stranger", STRANGER),
            ("t-google", GOOGLE_USER),
        ]);
        let mut cluster = Self {
            tokens,
            grants: HashSet::new(),
            access_reachable: true,
            asked: Mutex::new(Vec::new()),
        };
        // One grant per (identity, verb) — deliberately not a hierarchy.
        cluster.grant(ADMIN, COLLECTIONS_RESOURCE, Some(COLLECTION), "delete");
        cluster.grant(ADMIN, ADMIN_RESOURCE, None, "delete");
        cluster.grant(WRITER, COLLECTIONS_RESOURCE, Some(COLLECTION), "update");
        cluster.grant(READER, COLLECTIONS_RESOURCE, Some(COLLECTION), "get");
        cluster
    }

    fn grant(&mut self, user: &str, resource: &str, name: Option<&str>, verb: &str) {
        self.grants.insert((
            user.to_string(),
            resource.to_string(),
            name.map(str::to_string),
            verb.to_string(),
        ));
    }

    fn unreachable_authorizer(mut self) -> Self {
        self.access_reachable = false;
        self
    }

    fn asked(&self) -> Vec<ResourceAttributes> {
        self.asked.lock().expect("asked").clone()
    }
}

#[async_trait]
impl ReviewBackend for Cluster {
    async fn review_token(
        &self,
        token: &str,
        audiences: &[String],
    ) -> Result<TokenReviewOutcome, ReviewError> {
        Ok(match self.tokens.get(token) {
            Some(username) => TokenReviewOutcome {
                authenticated: true,
                identity: ReviewedIdentity {
                    username: (*username).to_string(),
                    ..Default::default()
                },
                audiences: audiences.to_vec(),
                error: None,
            },
            None => TokenReviewOutcome {
                authenticated: false,
                identity: ReviewedIdentity::default(),
                audiences: Vec::new(),
                error: Some("unknown token".into()),
            },
        })
    }

    async fn review_access(
        &self,
        identity: &ReviewedIdentity,
        attributes: &ResourceAttributes,
    ) -> Result<AccessReviewOutcome, ReviewError> {
        self.asked.lock().expect("asked").push(attributes.clone());
        if !self.access_reachable {
            return Err(ReviewError::Transport("apiserver unreachable".into()));
        }
        let key = (
            identity.username.clone(),
            attributes.resource.clone(),
            attributes.name.clone(),
            attributes.verb.clone(),
        );
        Ok(if self.grants.contains(&key) {
            AccessReviewOutcome::allow()
        } else {
            AccessReviewOutcome::deny("no RoleBinding grants this")
        })
    }
}

fn delegated_server(cluster: Arc<Cluster>) -> TestServer {
    let engine = Arc::new(Engine::new());
    let verifier =
        Arc::new(LumenVerifier::delegated(NAMESPACE, cluster).expect("delegated verifier"));
    let state =
        AppState::new(engine, Arc::new(AuthConfig::required_in(NAMESPACE))).with_verifier(verifier);
    TestServer::new(router(state)).expect("server")
}

/// One row of the matrix: the endpoint, and the single identity whose grant
/// covers it.
struct Row {
    method: &'static str,
    path: &'static str,
    body: Option<Value>,
    /// The token that must succeed. Every other ServiceAccount must be 403.
    allowed: &'static str,
}

/// Ordered on purpose: the collection is created, written, read, then dropped,
/// so every row runs against a server where the row before it succeeded.
fn rows() -> Vec<Row> {
    let schema = json!({ "fields": { "email": { "type": "keyword" } } });
    let index = json!({ "items": [{ "external_id": "u1", "field": "email", "value": "a@x.com" }] });
    let search =
        json!({ "query": { "term": { "field": "email", "value": "a@x.com" } }, "limit": 5 });
    vec![
        Row {
            method: "PUT",
            path: "/collections/users",
            body: Some(schema),
            allowed: "t-admin",
        },
        Row {
            method: "POST",
            path: "/collections/users/index",
            body: Some(index),
            allowed: "t-writer",
        },
        Row {
            method: "POST",
            path: "/collections/users/search",
            body: Some(search),
            allowed: "t-reader",
        },
        Row {
            method: "GET",
            path: "/collections/users/stats",
            body: None,
            allowed: "t-reader",
        },
        Row {
            method: "DELETE",
            path: "/collections/users",
            body: None,
            allowed: "t-admin",
        },
    ]
}

async fn status(s: &TestServer, row: &Row, token: Option<&str>) -> u16 {
    let mut r = match row.method {
        "GET" => s.get(row.path),
        "PUT" => s.put(row.path),
        "POST" => s.post(row.path),
        "DELETE" => s.delete(row.path),
        other => panic!("unhandled method {other}"),
    };
    if let Some(b) = &row.body {
        r = r.json(b);
    }
    if let Some(t) = token {
        r = r.add_header("authorization", format!("Bearer {t}"));
    }
    r.await.status_code().as_u16()
}

/// The whole matrix in one pass: for every endpoint, the three identities that
/// hold a *different* verb are denied, and only the one holding this
/// endpoint's verb gets through.
///
/// The denials run before the success so the 403s are answered by a server in
/// the same state the success will see — a 403 that only happened because the
/// collection did not exist yet would prove nothing.
#[tokio::test]
async fn each_endpoint_admits_exactly_its_own_grant_and_denies_every_other() {
    let cluster = Arc::new(Cluster::new());
    let s = delegated_server(cluster.clone());

    for row in rows() {
        for token in ["t-admin", "t-writer", "t-reader", "t-stranger"] {
            if token == row.allowed {
                continue;
            }
            assert_eq!(
                status(&s, &row, Some(token)).await,
                403,
                "{} {} must deny {token}: no RoleBinding grants it that verb",
                row.method,
                row.path
            );
        }
        let code = status(&s, &row, Some(row.allowed)).await;
        assert!(
            (200..300).contains(&code),
            "{} {} with its own grant returned {code}",
            row.method,
            row.path
        );
    }
}

/// The 401 column. Both cases are "not an accepted identity", and keeping them
/// distinct from 403 matters: a 403 would tell a caller their credential was
/// accepted and merely under-privileged.
#[tokio::test]
async fn unauthenticated_and_non_serviceaccount_callers_are_401_not_403() {
    let cluster = Arc::new(Cluster::new());
    let s = delegated_server(cluster.clone());

    for row in rows() {
        assert_eq!(
            status(&s, &row, None).await,
            401,
            "{} {} with no credential",
            row.method,
            row.path
        );
        assert_eq!(
            status(&s, &row, Some("t-unknown")).await,
            401,
            "{} {} with a token the apiserver does not know",
            row.method,
            row.path
        );
        assert_eq!(
            status(&s, &row, Some("t-google")).await,
            401,
            "{} {} with a Google user the apiserver *did* authenticate",
            row.method,
            row.path
        );
    }

    assert!(
        cluster.asked().is_empty(),
        "a rejected identity must never reach SubjectAccessReview: {:?}",
        cluster.asked()
    );
}

/// Listing is filtered, not denied: a caller sees the collections it may read
/// and no others, and never a 403 for the ones it may not.
#[tokio::test]
async fn the_collection_listing_is_filtered_per_caller() {
    let cluster = Arc::new(Cluster::new());
    let s = delegated_server(cluster.clone());

    s.put("/collections/users")
        .add_header("authorization", "Bearer t-admin")
        .json(&json!({ "fields": { "email": { "type": "keyword" } } }))
        .await
        .assert_status_ok();

    let listed: Value = s
        .get("/collections")
        .add_header("authorization", "Bearer t-reader")
        .await
        .json();
    assert_eq!(
        listed,
        json!(["users"]),
        "a reader with `get` on `users` should see exactly it"
    );

    let listed: Value = s
        .get("/collections")
        .add_header("authorization", "Bearer t-stranger")
        .await
        .json();
    assert_eq!(
        listed,
        json!([]),
        "a ServiceAccount with no grant sees an empty list, not a 403"
    );
}

/// An apiserver that cannot answer is 503. Never 200 — that would be a
/// fail-open — and never 403, which would report a policy decision nobody made.
#[tokio::test]
async fn an_authorization_outage_is_503() {
    let cluster = Arc::new(Cluster::new().unreachable_authorizer());
    let s = delegated_server(cluster);

    for row in rows() {
        assert_eq!(
            status(&s, &row, Some(row.allowed)).await,
            503,
            "{} {} during an apiserver outage",
            row.method,
            row.path
        );
    }
}
// CODEGEN-END
