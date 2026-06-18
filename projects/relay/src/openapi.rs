// SPEC-MANAGED: projects/relay/tech-design/interfaces/rest/http-2-openapi-transport-client-side-sharding-streaming-subscrib.md#rest-api
// HANDWRITE-BEGIN gap="missing-generator:rest-api:44732064" tracker="pending-tracker" reason="utoipa OpenAPI document for the public endpoints, served at /openapi.json."
//! utoipa OpenAPI document for the relay HTTP/2 transport.
//!
//! The path operations are declared by `#[utoipa::path]` on the
//! [`crate::server`] handlers; this module collects them into one document and
//! renders it as JSON for the `/openapi.json` endpoint.

use utoipa::OpenApi;

/// The served OpenAPI document.
///
/// @spec projects/relay/tech-design/interfaces/rest/http-2-openapi-transport-client-side-sharding-streaming-subscrib.md#rest-api
#[derive(OpenApi)]
#[openapi(
    info(
        title = "relay HTTP/2 transport",
        description = "All protocols over HTTP/2 (h2c), no gRPC. JSON contract with an application/cbor fast path for lease/ack and a CBOR frame stream for subscribe."
    ),
    paths(
        crate::server::publish,
        crate::server::lease,
        crate::server::ack,
        crate::server::heartbeat,
        crate::server::subscribe,
    )
)]
pub struct ApiDoc;

/// Render the OpenAPI document as pretty JSON for `/openapi.json`.
pub fn api_doc_json() -> String {
    ApiDoc::openapi()
        .to_pretty_json()
        .unwrap_or_else(|_| "{}".to_string())
}

#[cfg(test)]
mod tests {
    use super::api_doc_json;

    #[test]
    fn lists_the_public_endpoints() {
        let doc = api_doc_json();
        for path in [
            "/v1/{subject}/publish",
            "/v1/{subject}/lease",
            "/v1/{subject}/ack",
            "/v1/{subject}/subscribe",
        ] {
            assert!(doc.contains(path), "OpenAPI doc must list {path}");
        }
    }
}
// HANDWRITE-END
