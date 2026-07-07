// SPEC-MANAGED: projects/relay/tech-design/interfaces/rest/http-2-openapi-transport-client-side-sharding-work-queue-consume.md#rest-api
// HANDWRITE-BEGIN gap="missing-generator:rest-api:44732064" tracker="pending-tracker" reason="utoipa OpenAPI document for the public endpoints, served at /openapi.json."
//! utoipa OpenAPI document for the relay HTTP/2 transport.
//!
//! The path operations are declared by `#[utoipa::path]` on the
//! [`crate::server`] handlers; this module collects them into one document and
//! renders it as JSON for the `/openapi.json` endpoint.

use utoipa::OpenApi;

/// The served OpenAPI document.
///
/// @spec projects/relay/tech-design/interfaces/rest/http-2-openapi-transport-client-side-sharding-work-queue-consume.md#rest-api
#[derive(OpenApi)]
#[openapi(
    info(
        title = "relay HTTP/2 transport",
        description = "Single-cast work-queue broker over HTTP/2 (h2c), no gRPC. JSON contract with an application/cbor fast path for lease/ack and a length-prefixed frame stream for consume."
    ),
    paths(
        crate::server::publish,
        crate::server::publish_batch,
        crate::consume::consume,
        crate::server::lease,
        crate::server::ack,
        crate::server::lease_batch,
        crate::server::ack_batch,
        crate::server::heartbeat,
        crate::server::log_len,
        crate::server::admin_backup,
    )
)]
pub struct ApiDoc;

/// The relay OpenAPI document — the accessor the shared `service_http`
/// `/openapi.json` and `/docs` probe routes serve (a
/// `fn() -> utoipa::openapi::OpenApi` pointer).
pub fn openapi() -> utoipa::openapi::OpenApi {
    ApiDoc::openapi()
}

/// Render the OpenAPI document as pretty JSON (offline consumers — `relay
/// spec` and `relay spec gen`; the served route uses [`openapi`]).
pub fn api_doc_json() -> String {
    openapi()
        .to_pretty_json()
        .unwrap_or_else(|_| "{}".to_string())
}

/// Render the OpenAPI document as YAML (`relay spec --format openapi-yaml`,
/// keep's pattern) — the same document, for LLM/agent reading.
pub fn openapi_yaml() -> String {
    serde_yaml::to_string(&openapi()).expect("OpenApi serializes to YAML")
}

/// Render just the component schemas (`relay spec --format json-schema`).
/// Honest view: relay's handlers declare no named request/response schemas
/// today, so `components` serializes null — never a faked catalog (that is
/// also why relay has no keep-style `--shapes`/`--fields`).
pub fn json_schema_json() -> String {
    serde_json::to_string_pretty(&serde_json::json!({ "components": openapi().components }))
        .expect("components serialize to JSON")
}

#[cfg(test)]
mod tests {
    use super::api_doc_json;

    #[test]
    fn lists_the_public_endpoints() {
        let doc = api_doc_json();
        for path in [
            "/v1/{subject}/publish",
            "/v1/{subject}/consume",
            "/v1/{subject}/lease",
            "/v1/{subject}/ack",
            "/v1/{subject}/len",
        ] {
            assert!(doc.contains(path), "OpenAPI doc must list {path}");
        }
    }
}
// HANDWRITE-END
