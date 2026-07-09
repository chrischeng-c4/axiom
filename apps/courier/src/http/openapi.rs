// SPEC-MANAGED: apps/courier/tech-design/interfaces/rest/github-issues-proxy.md#rest-api
// HANDWRITE-BEGIN gap="missing-generator:rest-api:c0ur1e05" tracker="pending-tracker" reason="utoipa OpenAPI document for the four proxy endpoints, served at /openapi.json."
//! utoipa OpenAPI document for courier's GitHub-issues-proxy data plane.
//!
//! The path operations are declared by `#[utoipa::path]` on the
//! [`crate::http::routes`] handlers; this module collects them into one
//! document for the shared `/openapi.json` and `/docs` probe routes.

use utoipa::OpenApi;

/// The served OpenAPI document.
///
/// @spec apps/courier/tech-design/interfaces/rest/github-issues-proxy.md#rest-api
#[derive(OpenApi)]
#[openapi(
    info(
        title = "courier GitHub-issues proxy",
        description = "Stateless proxy that forwards issue search/view/create/comment to api.github.com with a server-held credential."
    ),
    paths(
        crate::http::routes::search_issues,
        crate::http::routes::view_issue,
        crate::http::routes::create_issue,
        crate::http::routes::comment_issue,
    )
)]
pub struct ApiDoc;

/// The courier OpenAPI document — the accessor the shared `service_http`
/// `/openapi.json` and `/docs` probe routes serve (a
/// `fn() -> utoipa::openapi::OpenApi` pointer).
pub fn openapi() -> utoipa::openapi::OpenApi {
    ApiDoc::openapi()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lists_the_public_endpoints() {
        let doc = openapi().to_pretty_json().unwrap();
        for path in [
            "/v1/issues/{owner}/{name}",
            "/v1/issues/{owner}/{name}/{number}",
            "/v1/issues/{owner}/{name}/{number}/comments",
        ] {
            assert!(doc.contains(path), "OpenAPI doc must list {path}");
        }
    }
}
// HANDWRITE-END
