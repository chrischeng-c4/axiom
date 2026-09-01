//! #3993 black-box contract for the canonical indexed-document namespace.
//!
//! Lumen indexes caller-owned fields. It does not own source records or
//! hydrate them. This target covers only the `/docs` naming migration and its
//! legacy `/index` compatibility surface. It deliberately has no merge-route
//! case.

use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use async_trait::async_trait;
use axum::http::StatusCode;
use axum_test::TestServer;
use openapi_codegen::{generate, GenOptions, HttpClient, Lang};
use serde_json::{json, Value};

use lumen::api::{router, AppState};
use lumen::auth::{AuthConfig, LumenVerifier, COLLECTIONS_RESOURCE};
use lumen::storage::Engine;
use service_auth::k8s::{
    AccessReviewOutcome, ResourceAttributes, ReviewBackend, ReviewError, ReviewedIdentity,
    TokenReviewOutcome,
};

const NAMESPACE: &str = "serving";
const COLLECTION: &str = "documents";
const WRITER: &str = "system:serviceaccount:serving:lumen-writer";
const READER: &str = "system:serviceaccount:serving:lumen-reader";
const ADMIN: &str = "system:serviceaccount:serving:lumen-admin";
const STRANGER: &str = "system:serviceaccount:serving:lumen-stranger";

fn open_server() -> TestServer {
    let engine = Arc::new(Engine::new());
    TestServer::new(router(AppState::open(engine))).expect("open test server")
}

/// A small delegated-auth oracle. It grants only the exact Kubernetes-style
/// access tuple the requested route needs.
struct AuthzOracle {
    tokens: HashMap<&'static str, &'static str>,
    grants: HashSet<(String, String, Option<String>, String)>,
}

impl AuthzOracle {
    fn new() -> Self {
        let tokens = HashMap::from([
            ("writer", WRITER),
            ("reader", READER),
            ("admin", ADMIN),
            ("stranger", STRANGER),
        ]);
        let grants = HashSet::from([
            (
                WRITER.to_string(),
                COLLECTIONS_RESOURCE.to_string(),
                Some(COLLECTION.to_string()),
                "update".to_string(),
            ),
            (
                READER.to_string(),
                COLLECTIONS_RESOURCE.to_string(),
                Some(COLLECTION.to_string()),
                "get".to_string(),
            ),
            (
                ADMIN.to_string(),
                COLLECTIONS_RESOURCE.to_string(),
                Some(COLLECTION.to_string()),
                "delete".to_string(),
            ),
        ]);
        Self { tokens, grants }
    }
}

#[async_trait]
impl ReviewBackend for AuthzOracle {
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
                error: Some("unknown token".to_string()),
            },
        })
    }

    async fn review_access(
        &self,
        identity: &ReviewedIdentity,
        attributes: &ResourceAttributes,
    ) -> Result<AccessReviewOutcome, ReviewError> {
        let grant = (
            identity.username.clone(),
            attributes.resource.clone(),
            attributes.name.clone(),
            attributes.verb.clone(),
        );
        Ok(if self.grants.contains(&grant) {
            AccessReviewOutcome::allow()
        } else {
            AccessReviewOutcome::deny("no matching RoleBinding")
        })
    }
}

fn delegated_server() -> TestServer {
    let engine = Arc::new(Engine::new());
    let verifier = Arc::new(
        LumenVerifier::delegated(NAMESPACE, Arc::new(AuthzOracle::new()))
            .expect("delegated verifier"),
    );
    let state =
        AppState::new(engine, Arc::new(AuthConfig::required_in(NAMESPACE))).with_verifier(verifier);
    TestServer::new(router(state)).expect("delegated test server")
}

#[tokio::test]
async fn canonical_delete_removes_complete_document_preserves_siblings_and_allows_rewrite() {
    let server = open_server();
    server
        .put("/collections/documents")
        .json(&json!({ "fields": {
            "title": { "type": "text" },
            "state": { "type": "keyword" }
        }}))
        .await
        .assert_status_ok();
    server
        .put("/collections/documents/docs:replace")
        .json(&json!({ "docs": [
            {
                "external_id": "remove-me",
                "fields": { "title": "remove this row", "state": "retired" }
            },
            {
                "external_id": "keep-me",
                "fields": { "title": "keep this sibling", "state": "live" }
            }
        ]}))
        .await
        .assert_status_ok();

    server
        .delete("/collections/documents/docs/remove-me")
        .await
        .assert_status(StatusCode::NO_CONTENT);

    for query in [
        json!({ "match": { "field": "title", "text": "remove" } }),
        json!({ "term": { "field": "state", "value": "retired" } }),
    ] {
        let response = server
            .post("/collections/documents/search")
            .json(&json!({ "query": query, "limit": 10 }))
            .await;
        response.assert_status_ok();
        let body: Value = response.json();
        assert_eq!(
            body["total"], 0,
            "canonical delete must remove all indexed fields: {body}"
        );
    }

    let sibling = server
        .post("/collections/documents/search")
        .json(&json!({
            "query": { "term": { "field": "state", "value": "live" } },
            "limit": 10
        }))
        .await;
    sibling.assert_status_ok();
    let sibling: Value = sibling.json();
    assert_eq!(
        sibling["total"], 1,
        "canonical delete must preserve siblings: {sibling}"
    );
    assert_eq!(sibling["hits"][0]["external_id"], "keep-me");

    server
        .delete("/collections/documents/docs/missing")
        .await
        .assert_status(StatusCode::NO_CONTENT);
    server
        .put("/collections/documents/docs/remove-me")
        .json(&json!({
            "fields": { "title": "rewritten row", "state": "active" }
        }))
        .await
        .assert_status_ok();

    let rewritten = server
        .post("/collections/documents/search")
        .json(&json!({
            "query": { "term": { "field": "state", "value": "active" } },
            "limit": 10
        }))
        .await;
    rewritten.assert_status_ok();
    let rewritten: Value = rewritten.json();
    assert_eq!(
        rewritten["total"], 1,
        "deleted ids must be writable again: {rewritten}"
    );
    assert_eq!(rewritten["hits"][0]["external_id"], "remove-me");
}

#[tokio::test]
async fn canonical_delete_requires_write_and_denies_other_grants() {
    let server = delegated_server();
    server
        .put("/collections/documents")
        .add_header("authorization", "Bearer admin")
        .json(&json!({ "fields": { "state": { "type": "keyword" } } }))
        .await
        .assert_status_ok();
    server
        .post("/collections/documents/index")
        .add_header("authorization", "Bearer writer")
        .json(&json!({ "items": [
            { "external_id": "row-1", "field": "state", "value": "live" }
        ] }))
        .await
        .assert_status_ok();

    for token in ["admin", "reader", "stranger"] {
        server
            .delete("/collections/documents/docs/row-1")
            .add_header("authorization", format!("Bearer {token}"))
            .await
            .assert_status(StatusCode::FORBIDDEN);
    }
    server
        .delete("/collections/documents/docs/row-1")
        .add_header("authorization", "Bearer writer")
        .await
        .assert_status(StatusCode::NO_CONTENT);
}

#[test]
fn openapi_publishes_canonical_delete_and_deprecated_legacy_index_operations() {
    let spec: Value = serde_json::from_str(&lumen::spec::openapi_json()).expect("OpenAPI parses");
    let canonical = &spec["paths"]["/collections/{collection_id}/docs/{external_id}"]["delete"];
    assert!(
        canonical.is_object(),
        "canonical document DELETE missing from OpenAPI: {canonical}"
    );
    assert_eq!(canonical["operationId"], "delete_doc");
    assert!(
        canonical["responses"]["204"].is_object(),
        "canonical document DELETE must declare 204: {canonical}"
    );

    for (method, path) in [
        ("post", "/collections/{collection_id}/index"),
        ("delete", "/collections/{collection_id}/index/{external_id}"),
    ] {
        let legacy = &spec["paths"][path][method];
        assert!(
            legacy.is_object(),
            "legacy {method} {path} must remain published"
        );
        assert_eq!(
            legacy["deprecated"], true,
            "legacy {method} {path} must be deprecated without a removal header: {legacy}"
        );
        assert!(
            legacy.get("x-sunset").is_none(),
            "legacy {method} {path} must not publish a Sunset removal date: {legacy}"
        );
    }
}

fn generated_client_options(lang: Lang) -> GenOptions {
    GenOptions {
        lang,
        target: None,
        spec_path: Default::default(),
        out_dir: Default::default(),
        client_name: "createLumenClient".to_string(),
        http_client: HttpClient::Fetch,
        emit_types: true,
        emit_client: true,
        emit_hooks: lang == Lang::Ts,
    }
}

#[test]
fn generated_clients_expose_canonical_delete_and_legacy_index_operations() {
    for (lang, canonical_delete, legacy_index, legacy_delete) in [
        (
            Lang::Py,
            "def delete_doc(",
            "def index(",
            "def delete_external_id(",
        ),
        (Lang::Ts, "deleteDoc(", "index(", "deleteExternalId("),
        (
            Lang::Rust,
            "pub fn delete_doc(",
            "pub fn index(",
            "pub fn delete_external_id(",
        ),
    ] {
        let generated = generate(
            &lumen::spec::openapi_json(),
            &generated_client_options(lang),
        )
        .unwrap_or_else(|error| panic!("generate {lang:?} client: {error}"));
        let output = generated
            .files
            .into_iter()
            .map(|file| file.contents)
            .collect::<Vec<_>>()
            .join("\n");
        for operation in [canonical_delete, legacy_index, legacy_delete] {
            assert!(
                output.contains(operation),
                "{lang:?} generated client must expose {operation:?}"
            );
        }
    }
}
