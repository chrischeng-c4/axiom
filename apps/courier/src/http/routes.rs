// SPEC-MANAGED: apps/courier/tech-design/interfaces/rest/github-issues-proxy.md#logic
// HANDWRITE-BEGIN gap="missing-generator:logic:c0ur1e04" tracker="pending-tracker" reason="axum handlers for the four GitHub-issues-proxy endpoints: search/view/create/comment, each authorizing on the {owner}/{name} repo resource then forwarding through GithubClient."
//! axum handlers for the GitHub-issues-proxy data plane.
//!
//! Each handler authorizes the injected [`RoleMapPrincipal`] on the
//! `{owner}/{name}` repo resource (search/view = read, create/comment =
//! write — [`crate::http::auth::authorize`]), rejects repos outside
//! [`crate::http::github::GithubClient::is_allowed`], then forwards through
//! [`crate::http::github::GithubClient`] and relays the JSON response
//! verbatim.

use axum::extract::{Extension, Path, Query, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::Deserialize;
use serde_json::Value;
use service_auth::{Role, RoleMapPrincipal};
use service_http::ApiErr;

use crate::http::auth::authorize;
use crate::http::github::GithubError;
use crate::http::AppState;

fn resource(owner: &str, name: &str) -> String {
    format!("{owner}/{name}")
}

fn forbidden_repo(owner: &str, name: &str) -> Response {
    ApiErr::new(
        StatusCode::FORBIDDEN,
        "repo_not_allowed",
        format!("{owner}/{name} is not in COURIER_ALLOWED_REPOS"),
    )
    .into_response()
}

fn github_err(e: GithubError) -> Response {
    match e {
        GithubError::Upstream(message) => {
            ApiErr::new(StatusCode::BAD_GATEWAY, "github_upstream", message).into_response()
        }
        GithubError::Github { status, message } => {
            ApiErr::new(status, "github_error", message).into_response()
        }
    }
}

/// `GET /v1/issues/{owner}/{name}` query params.
#[derive(Debug, Deserialize, utoipa::IntoParams)]
pub struct SearchQuery {
    #[serde(default = "default_state")]
    state: String,
    q: Option<String>,
    #[serde(default = "default_limit")]
    limit: u32,
}

fn default_state() -> String {
    "open".to_string()
}

fn default_limit() -> u32 {
    20
}

/// `GET /v1/issues/{owner}/{name}` — forwards to `GET
/// api.github.com/search/issues`, scoped to the repo.
#[utoipa::path(
    get,
    path = "/v1/issues/{owner}/{name}",
    params(
        ("owner" = String, Path, description = "GitHub repo owner"),
        ("name" = String, Path, description = "GitHub repo name"),
        SearchQuery,
    ),
    responses((status = 200, description = "GitHub search/issues response, forwarded verbatim"))
)]
pub async fn search_issues(
    State(st): State<AppState>,
    Extension(principal): Extension<RoleMapPrincipal>,
    Path((owner, name)): Path<(String, String)>,
    Query(q): Query<SearchQuery>,
) -> Response {
    if let Err(deny) = authorize(&principal, &resource(&owner, &name), Role::Read) {
        return deny.into_response();
    }
    if !st.github().is_allowed(&owner, &name) {
        return forbidden_repo(&owner, &name);
    }
    match st
        .github()
        .search_issues(&owner, &name, &q.state, q.q.as_deref(), q.limit)
        .await
    {
        Ok(v) => Json(v).into_response(),
        Err(e) => github_err(e),
    }
}

/// `GET /v1/issues/{owner}/{name}/{number}` — forwards to `GET
/// api.github.com/repos/{owner}/{name}/issues/{number}`.
#[utoipa::path(
    get,
    path = "/v1/issues/{owner}/{name}/{number}",
    params(
        ("owner" = String, Path, description = "GitHub repo owner"),
        ("name" = String, Path, description = "GitHub repo name"),
        ("number" = u64, Path, description = "Issue number"),
    ),
    responses((status = 200, description = "GitHub issue resource, forwarded verbatim"))
)]
pub async fn view_issue(
    State(st): State<AppState>,
    Extension(principal): Extension<RoleMapPrincipal>,
    Path((owner, name, number)): Path<(String, String, u64)>,
) -> Response {
    if let Err(deny) = authorize(&principal, &resource(&owner, &name), Role::Read) {
        return deny.into_response();
    }
    if !st.github().is_allowed(&owner, &name) {
        return forbidden_repo(&owner, &name);
    }
    match st.github().view_issue(&owner, &name, number).await {
        Ok(v) => Json(v).into_response(),
        Err(e) => github_err(e),
    }
}

/// `POST /v1/issues/{owner}/{name}` — forwards to `POST
/// api.github.com/repos/{owner}/{name}/issues`. `body` is the GitHub
/// issue-creation payload (`title`/`body`/`labels`), forwarded verbatim.
#[utoipa::path(
    post,
    path = "/v1/issues/{owner}/{name}",
    params(
        ("owner" = String, Path, description = "GitHub repo owner"),
        ("name" = String, Path, description = "GitHub repo name"),
    ),
    request_body = Object,
    responses((status = 200, description = "Created GitHub issue, forwarded verbatim"))
)]
pub async fn create_issue(
    State(st): State<AppState>,
    Extension(principal): Extension<RoleMapPrincipal>,
    Path((owner, name)): Path<(String, String)>,
    Json(payload): Json<Value>,
) -> Response {
    if let Err(deny) = authorize(&principal, &resource(&owner, &name), Role::Write) {
        return deny.into_response();
    }
    if !st.github().is_allowed(&owner, &name) {
        return forbidden_repo(&owner, &name);
    }
    match st.github().create_issue(&owner, &name, &payload).await {
        Ok(v) => Json(v).into_response(),
        Err(e) => github_err(e),
    }
}

/// `POST /v1/issues/{owner}/{name}/{number}/comments` — reopens the issue
/// then forwards to `POST
/// api.github.com/repos/{owner}/{name}/issues/{number}/comments`. `body` is
/// the GitHub comment payload (`{"body": "..."}`), forwarded verbatim.
#[utoipa::path(
    post,
    path = "/v1/issues/{owner}/{name}/{number}/comments",
    params(
        ("owner" = String, Path, description = "GitHub repo owner"),
        ("name" = String, Path, description = "GitHub repo name"),
        ("number" = u64, Path, description = "Issue number"),
    ),
    request_body = Object,
    responses((status = 200, description = "Created GitHub comment, forwarded verbatim"))
)]
pub async fn comment_issue(
    State(st): State<AppState>,
    Extension(principal): Extension<RoleMapPrincipal>,
    Path((owner, name, number)): Path<(String, String, u64)>,
    Json(payload): Json<Value>,
) -> Response {
    if let Err(deny) = authorize(&principal, &resource(&owner, &name), Role::Write) {
        return deny.into_response();
    }
    if !st.github().is_allowed(&owner, &name) {
        return forbidden_repo(&owner, &name);
    }
    match st
        .github()
        .comment_issue(&owner, &name, number, &payload)
        .await
    {
        Ok(v) => Json(v).into_response(),
        Err(e) => github_err(e),
    }
}
// HANDWRITE-END
