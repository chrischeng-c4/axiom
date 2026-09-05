//! Shared Model Context Protocol transport and security shell.

use std::sync::Arc;

use anyhow::{bail, Context, Result};
use axum::{body::Body, extract::Request, response::Response, routing::any, Router};
use rmcp::{ServerHandler, ServiceExt};
use tower::ServiceExt as TowerServiceExt;

/// A product MCP handler that can receive a caller credential at the HTTP
/// transport boundary. Tool schemas and handlers remain in the product.
pub trait McpApplication: ServerHandler + Clone + Send + Sync + 'static {
    fn with_bearer_token(&self, token: Option<String>) -> Self;
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HttpTransportConfig {
    allowed_hosts: Vec<String>,
    allowed_origins: Vec<String>,
}

impl HttpTransportConfig {
    pub fn new(allowed_hosts: Vec<String>, allowed_origins: Vec<String>) -> Result<Self> {
        let config = Self {
            allowed_hosts,
            allowed_origins,
        };
        config.validate()?;
        Ok(config)
    }

    pub fn from_env(
        hosts_env: &str,
        origins_env: &str,
        default_hosts: Vec<String>,
        default_origins: Vec<String>,
    ) -> Result<Self> {
        Self::new(
            csv_env(hosts_env, default_hosts)?,
            csv_env(origins_env, default_origins)?,
        )
    }

    fn validate(&self) -> Result<()> {
        if self.allowed_hosts.is_empty()
            || self
                .allowed_hosts
                .iter()
                .any(|value| value.trim().is_empty())
        {
            bail!("MCP allowed hosts must contain at least one nonempty value");
        }
        if self.allowed_origins.is_empty()
            || self
                .allowed_origins
                .iter()
                .any(|value| value.trim().is_empty())
        {
            bail!("MCP allowed origins must contain at least one nonempty value");
        }
        Ok(())
    }

    fn into_rmcp(self) -> rmcp::transport::streamable_http_server::StreamableHttpServerConfig {
        rmcp::transport::streamable_http_server::StreamableHttpServerConfig::default()
            .with_legacy_session_mode(true)
            .with_json_response(true)
            .with_allowed_hosts(self.allowed_hosts)
            .with_allowed_origins(self.allowed_origins)
    }
}

/// Serve a product handler over MCP standard input and output.
pub async fn serve_stdio<A: McpApplication>(application: A) -> Result<()> {
    let service = application
        .serve(rmcp::transport::stdio())
        .await
        .context("start MCP stdio transport")?;
    service
        .waiting()
        .await
        .context("wait for MCP stdio transport")?;
    Ok(())
}

/// Build one Streamable HTTP route with shared sessions and browser policy.
pub fn streamable_http_router<A: McpApplication>(
    path: &str,
    application: A,
    config: HttpTransportConfig,
) -> Router {
    let sessions = Arc::new(
        rmcp::transport::streamable_http_server::session::local::LocalSessionManager::default(),
    );
    let config = config.into_rmcp();
    Router::new().route(
        path,
        any(move |request: Request| {
            let application = application.clone();
            let sessions = sessions.clone();
            let config = config.clone();
            async move { handle_http(request, application, sessions, config).await }
        }),
    )
}

async fn handle_http<A: McpApplication>(
    request: Request,
    application: A,
    sessions: Arc<rmcp::transport::streamable_http_server::session::local::LocalSessionManager>,
    config: rmcp::transport::streamable_http_server::StreamableHttpServerConfig,
) -> Response {
    let token = service_auth::bearer_token(request.headers()).map(str::to_owned);
    let handler = application.with_bearer_token(token);
    let service = rmcp::transport::streamable_http_server::StreamableHttpService::new(
        move || Ok(handler.clone()),
        sessions,
        config,
    );
    let response = match service.oneshot(request).await {
        Ok(response) => response,
        Err(never) => match never {},
    };
    let (parts, body) = response.into_parts();
    Response::from_parts(parts, Body::new(body))
}

fn csv_env(name: &str, default: Vec<String>) -> Result<Vec<String>> {
    let Ok(value) = std::env::var(name) else {
        return Ok(default);
    };
    let values = value
        .split(',')
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_owned)
        .collect::<Vec<_>>();
    if values.is_empty() {
        bail!("{name} must contain at least one comma-separated value");
    }
    Ok(values)
}
