//! Read-only Model Context Protocol tools backed by the public Sift API.
//!
//! The standard-input server and the HTTP server use the same tool handler.
//! HTTP requests forward their bearer token to the normal Sift API. This keeps
//! project authorization in one place.

use std::{sync::Arc, time::Duration};

use anyhow::{bail, Context, Result};
use reqwest::{Method, RequestBuilder};
use rmcp::{
    handler::server::wrapper::Parameters,
    model::{Implementation, ServerCapabilities, ServerInfo},
    tool, tool_router, ErrorData, Json,
};
use schemars1::JsonSchema;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_json::Value;
use url::Url;

use crate::api::{CorrelationRequestV1, LogTailRequestV1, QueryRequestV1, QueryResponseV1};

const MCP_TIMEOUT: Duration = Duration::from_secs(30);
const MCP_ALLOWED_HOSTS_ENV: &str = "SIFT_MCP_ALLOWED_HOSTS";
const MCP_ALLOWED_ORIGINS_ENV: &str = "SIFT_MCP_ALLOWED_ORIGINS";

/// Small client shared by the CLI and MCP tools.
#[derive(Clone)]
pub struct SiftApiClient {
    endpoint: Url,
    token: Option<Arc<str>>,
    http: reqwest::Client,
}

impl SiftApiClient {
    pub fn new(endpoint: &str, token: Option<String>, timeout: Duration) -> Result<Self> {
        let mut endpoint = Url::parse(endpoint).context("parse Sift API endpoint")?;
        if !matches!(endpoint.scheme(), "http" | "https") {
            bail!("Sift API endpoint must use http or https");
        }
        if endpoint.cannot_be_a_base()
            || endpoint.query().is_some()
            || endpoint.fragment().is_some()
        {
            bail!("Sift API endpoint must be a base URL without query or fragment");
        }
        if !endpoint.path().ends_with('/') {
            endpoint.set_path(&format!("{}/", endpoint.path()));
        }
        let http = reqwest::Client::builder()
            .timeout(timeout)
            .build()
            .context("build Sift API client")?;
        Ok(Self {
            endpoint,
            token: token.map(Arc::from),
            http,
        })
    }

    fn with_token(&self, token: Option<String>) -> Self {
        Self {
            endpoint: self.endpoint.clone(),
            token: token.map(Arc::from),
            http: self.http.clone(),
        }
    }

    pub async fn query(&self, request: &QueryRequestV1) -> Result<QueryResponseV1> {
        self.post_json("api/v1/query", &request.project, request)
            .await
    }

    async fn query_value(&self, request: &QueryRequestV1) -> Result<Value> {
        self.post_json("api/v1/query", &request.project, request)
            .await
    }

    async fn tail_logs(&self, request: &LogTailRequestV1) -> Result<Value> {
        self.post_json("api/v1/logs/tail", &request.project, request)
            .await
    }

    async fn correlate(&self, request: &CorrelationRequestV1) -> Result<Value> {
        self.post_json("api/v1/correlate", &request.project, request)
            .await
    }

    async fn get_trace(
        &self,
        project: &str,
        trace_id: &str,
        min_cursor: Option<u64>,
    ) -> Result<Value> {
        let mut url = self.endpoint.join("api/v1/traces/")?;
        url.path_segments_mut()
            .map_err(|_| anyhow::anyhow!("Sift endpoint cannot contain path segments"))?
            .pop_if_empty()
            .push(trace_id);
        let mut query = vec![("project", project.to_string())];
        if let Some(cursor) = min_cursor {
            query.push(("min_cursor", cursor.to_string()));
        }
        let request =
            self.authorize_project(self.http.request(Method::GET, url).query(&query), project);
        self.send(request).await
    }

    async fn list_services(&self, project: &str, environment: Option<&str>) -> Result<Value> {
        let url = self.endpoint.join("api/v1/services")?;
        let mut query = vec![("project", project.to_string())];
        if let Some(environment) = environment {
            query.push(("environment", environment.to_string()));
        }
        let request =
            self.authorize_project(self.http.request(Method::GET, url).query(&query), project);
        self.send(request).await
    }

    async fn post_json<T, R>(&self, path: &str, project: &str, value: &T) -> Result<R>
    where
        T: Serialize + ?Sized,
        R: DeserializeOwned,
    {
        let url = self.endpoint.join(path)?;
        let request =
            self.authorize_project(self.http.request(Method::POST, url).json(value), project);
        self.send(request).await
    }

    fn authorize_project(&self, request: RequestBuilder, project: &str) -> RequestBuilder {
        let request = request.header("x-sift-project", project);
        match &self.token {
            Some(token) => request.bearer_auth(token.as_ref()),
            None => request,
        }
    }

    async fn send<R: DeserializeOwned>(&self, request: RequestBuilder) -> Result<R> {
        let response = request.send().await.context("send Sift API request")?;
        let status = response.status();
        let body = response.bytes().await.context("read Sift API response")?;
        if !status.is_success() {
            let message = String::from_utf8_lossy(&body);
            bail!("Sift API returned {status}: {}", truncate(&message, 8_192));
        }
        serde_json::from_slice(&body).context("decode Sift API response")
    }
}

fn truncate(value: &str, max_bytes: usize) -> &str {
    if value.len() <= max_bytes {
        return value;
    }
    let mut end = max_bytes;
    while !value.is_char_boundary(end) {
        end -= 1;
    }
    &value[..end]
}

#[derive(Clone)]
struct SiftMcpServer {
    api: SiftApiClient,
}

impl SiftMcpServer {
    fn new(api: SiftApiClient) -> Self {
        Self { api }
    }
}

impl service_mcp::McpApplication for SiftMcpServer {
    fn with_bearer_token(&self, token: Option<String>) -> Self {
        Self::new(self.api.with_token(token))
    }
}

#[derive(Deserialize, JsonSchema)]
#[schemars(crate = "schemars1")]
struct QueryToolArgs {
    /// A versioned QueryRequestV1 JSON object.
    request: Value,
}

#[derive(Deserialize, JsonSchema)]
#[schemars(crate = "schemars1")]
struct TraceToolArgs {
    /// Authorized Sift project.
    project: String,
    /// Trace identifier.
    trace_id: String,
    /// Optional read-your-write watermark.
    min_cursor: Option<u64>,
}

#[derive(Deserialize, JsonSchema)]
#[schemars(crate = "schemars1")]
struct CorrelateToolArgs {
    /// A versioned CorrelationRequestV1 JSON object.
    request: Value,
}

#[derive(Deserialize, JsonSchema)]
#[schemars(crate = "schemars1")]
struct ListServicesToolArgs {
    /// Authorized Sift project.
    project: String,
    /// Optional environment filter.
    environment: Option<String>,
}

#[derive(Deserialize, JsonSchema)]
#[schemars(crate = "schemars1")]
struct TailLogsToolArgs {
    /// A versioned LogTailRequestV1 JSON object.
    request: Value,
}

#[tool_router]
impl SiftMcpServer {
    #[tool(
        name = "sift_query",
        description = "Query Sift logs, metrics, or traces with QueryRequestV1",
        annotations(read_only_hint = true)
    )]
    async fn sift_query(
        &self,
        Parameters(args): Parameters<QueryToolArgs>,
    ) -> std::result::Result<Json<Value>, ErrorData> {
        let request: QueryRequestV1 = parse_tool_request(args.request, "QueryRequestV1")?;
        request
            .validate()
            .map_err(|error| ErrorData::invalid_params(error.to_string(), None))?;
        self.api
            .query_value(&request)
            .await
            .map(Json)
            .map_err(tool_api_error)
    }

    #[tool(
        name = "sift_get_trace",
        description = "Get one trace by project and trace ID",
        annotations(read_only_hint = true)
    )]
    async fn sift_get_trace(
        &self,
        Parameters(args): Parameters<TraceToolArgs>,
    ) -> std::result::Result<Json<Value>, ErrorData> {
        if args.project.trim().is_empty() || args.trace_id.trim().is_empty() {
            return Err(ErrorData::invalid_params(
                "project and trace_id must not be empty",
                None,
            ));
        }
        self.api
            .get_trace(&args.project, &args.trace_id, args.min_cursor)
            .await
            .map(Json)
            .map_err(tool_api_error)
    }

    #[tool(
        name = "sift_correlate",
        description = "Find related logs, metrics, and traces",
        annotations(read_only_hint = true)
    )]
    async fn sift_correlate(
        &self,
        Parameters(args): Parameters<CorrelateToolArgs>,
    ) -> std::result::Result<Json<Value>, ErrorData> {
        let request: CorrelationRequestV1 =
            parse_tool_request(args.request, "CorrelationRequestV1")?;
        request
            .validate()
            .map_err(|error| ErrorData::invalid_params(error.to_string(), None))?;
        self.api
            .correlate(&request)
            .await
            .map(Json)
            .map_err(tool_api_error)
    }

    #[tool(
        name = "sift_list_services",
        description = "List services seen by Sift in one project",
        annotations(read_only_hint = true)
    )]
    async fn sift_list_services(
        &self,
        Parameters(args): Parameters<ListServicesToolArgs>,
    ) -> std::result::Result<Json<Value>, ErrorData> {
        if args.project.trim().is_empty()
            || args
                .environment
                .as_deref()
                .is_some_and(|value| value.trim().is_empty())
        {
            return Err(ErrorData::invalid_params(
                "project and environment must not be empty",
                None,
            ));
        }
        self.api
            .list_services(&args.project, args.environment.as_deref())
            .await
            .map(Json)
            .map_err(tool_api_error)
    }

    #[tool(
        name = "sift_tail_logs",
        description = "Wait for and return new Sift log records",
        annotations(read_only_hint = true)
    )]
    async fn sift_tail_logs(
        &self,
        Parameters(args): Parameters<TailLogsToolArgs>,
    ) -> std::result::Result<Json<Value>, ErrorData> {
        let request: LogTailRequestV1 = parse_tool_request(args.request, "LogTailRequestV1")?;
        request
            .validate()
            .map_err(|error| ErrorData::invalid_params(error.to_string(), None))?;
        self.api
            .tail_logs(&request)
            .await
            .map(Json)
            .map_err(tool_api_error)
    }
}

#[rmcp::tool_handler]
impl rmcp::ServerHandler for SiftMcpServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo::new(ServerCapabilities::builder().enable_tools().build())
            .with_server_info(Implementation::new("sift", env!("CARGO_PKG_VERSION")))
            .with_instructions("Read-only logs, metrics, traces, and correlation tools for Sift")
    }
}

fn parse_tool_request<T: DeserializeOwned>(
    value: Value,
    name: &str,
) -> std::result::Result<T, ErrorData> {
    serde_json::from_value(value)
        .map_err(|error| ErrorData::invalid_params(format!("invalid {name}: {error}"), None))
}

fn tool_api_error(error: anyhow::Error) -> ErrorData {
    ErrorData::internal_error(error.to_string(), None)
}

/// Names are exposed for contract tests and client discovery documentation.
pub fn tool_names() -> Vec<String> {
    let mut names = SiftMcpServer::tool_router()
        .list_all()
        .into_iter()
        .map(|tool| tool.name.into_owned())
        .collect::<Vec<_>>();
    names.sort();
    names
}

/// Serve the five read-only tools over MCP standard input and output.
pub async fn serve_stdio(endpoint: String, token: Option<String>) -> Result<()> {
    let server = SiftMcpServer::new(SiftApiClient::new(&endpoint, token, MCP_TIMEOUT)?);
    service_mcp::serve_stdio(server).await
}

/// Build the official Streamable HTTP transport at `/mcp`.
pub fn http_router(endpoint: &str) -> Result<axum::Router> {
    let base = SiftApiClient::new(endpoint, None, MCP_TIMEOUT)?;
    let default_hosts = ["localhost", "127.0.0.1", "::1"]
        .into_iter()
        .map(str::to_owned)
        .collect::<Vec<_>>();
    let default_origins = vec![base.endpoint.origin().ascii_serialization()];
    let config = service_mcp::HttpTransportConfig::from_env(
        MCP_ALLOWED_HOSTS_ENV,
        MCP_ALLOWED_ORIGINS_ENV,
        default_hosts,
        default_origins,
    )?;
    Ok(service_mcp::streamable_http_router(
        "/mcp",
        SiftMcpServer::new(base),
        config,
    ))
}
