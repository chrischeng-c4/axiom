// SPEC-MANAGED: apps/tape/tech-design/logic/eliminate-production-ec-false-green-paths.md#logic
// HANDWRITE-BEGIN gap="missing-generator:logic:tape-openapi-route-inventory" tracker="#2159" reason="Tape-specific utoipa path/schema inventory consumed by offline generated-client verification."
//! utoipa OpenAPI document for tape's HTTP transport.
//!
//! The path operations are declared by `#[utoipa::path]` on the
//! [`crate::server`] handlers; this module collects them into one document
//! and renders it as JSON for the `/openapi.json` endpoint. Independent of
//! the existing hand-rolled [`crate::spec`] JSON contract used by
//! `tape spec`, which stays untouched.

use utoipa::OpenApi;

/// The served OpenAPI document.
#[derive(OpenApi)]
#[openapi(
    info(
        title = "tape HTTP transport",
        description = "Topic append/replay/checkpoint journal over HTTP/1.1 + h2c."
    ),
    paths(
        crate::server::append,
        crate::server::replay,
        crate::server::replay_stream,
        crate::server::checkpoint_get,
        crate::server::checkpoint_put,
        crate::server::subscription_create,
        crate::server::subscription_list,
        crate::server::subscription_get,
        crate::server::subscription_delete,
        crate::server::subscription_pull,
        crate::server::subscription_ack,
        crate::server::retention_get,
        crate::server::retention_put,
        crate::server::admin_backup,
    ),
    components(schemas(
        crate::TapeEvent,
        crate::ConsumerCheckpoint,
        crate::server::AppendRequest,
        crate::server::ReplayResponse,
        crate::server::CheckpointResponse,
        crate::server::CheckpointPutRequest,
        crate::Subscription,
        crate::PullSubscriptionBatch,
        crate::server::SubscriptionCreateRequest,
        crate::server::SubscriptionListResponse,
        crate::server::SubscriptionPullRequest,
        crate::server::SubscriptionAckRequest,
        crate::RetentionPolicy,
        crate::RetentionOutcome,
        crate::server::RetentionGetResponse,
    ))
)]
pub struct ApiDoc;

/// The tape OpenAPI document — the accessor the shared `service_http`
/// `/openapi.json` and `/docs` probe routes serve (a
/// `fn() -> utoipa::openapi::OpenApi` pointer).
pub fn openapi() -> utoipa::openapi::OpenApi {
    ApiDoc::openapi()
}

#[cfg(test)]
mod tests {
    use super::openapi;

    #[test]
    fn lists_the_public_endpoints() {
        let doc = openapi().to_pretty_json().unwrap();
        for path in [
            "/topics/{topic}/append",
            "/topics/{topic}/replay",
            "/topics/{topic}/replay/stream",
            "/topics/{topic}/consumers/{consumer}/checkpoint",
            "/topics/{topic}/subscriptions",
            "/topics/{topic}/subscriptions/{name}",
            "/topics/{topic}/subscriptions/{name}/pull",
            "/topics/{topic}/subscriptions/{name}/ack",
            "/topics/{topic}/retention",
            "/admin/backup",
        ] {
            assert!(doc.contains(path), "OpenAPI doc must list {path}");
        }
    }
}
// HANDWRITE-END
