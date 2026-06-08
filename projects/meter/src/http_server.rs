// SPEC-MANAGED: projects/meter/tech-design/semantic/source/projects-meter-src-http-server-rs.md#source
// CODEGEN-BEGIN
//! Minimal HTTP test server for benchmarking
//!
//! Provides a lightweight HTTP server with configurable static JSON responses.
//! Used for HTTP client benchmarks to eliminate external network latency.
//!
//! Also supports spawning Python applications as subprocesses for integration testing.

use axum::{
    extract::{Path, State},
    http::{Method, StatusCode},
    response::{IntoResponse, Response},
    routing::get,
    Json, Router,
};
use serde_json::Value as JsonValue;
use std::collections::HashMap;
use std::process::{Child, Command};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::net::TcpListener;
use tokio::sync::oneshot;

/// Configuration for a test server route
#[derive(Debug, Clone)]
/// @spec projects/meter/tech-design/semantic/source/projects-meter-src-http-server-rs.md#source
pub struct RouteConfig {
    /// HTTP method (GET, POST, etc.)
    pub method: String,
    /// Response JSON
    pub response: JsonValue,
    /// Response status code
    pub status: u16,
    /// Whether to echo POST body in response
    pub echo_body: bool,
}

/// @spec projects/meter/tech-design/semantic/source/projects-meter-src-http-server-rs.md#source
impl Default for RouteConfig {
    fn default() -> Self {
        Self {
            method: "GET".to_string(),
            response: JsonValue::Object(serde_json::Map::new()),
            status: 200,
            echo_body: false,
        }
    }
}

/// Shared state for the test server
#[derive(Debug, Clone)]
struct ServerState {
    routes: Arc<HashMap<String, RouteConfig>>,
}

/// Configuration for a Python application test server
#[derive(Debug, Clone)]
/// @spec projects/meter/tech-design/semantic/source/projects-meter-src-http-server-rs.md#source
pub struct TestServerConfig {
    /// Python module to import (e.g., "tests.fixtures.test_app")
    pub app_module: String,
    /// Application callable name (e.g., "app")
    pub app_callable: String,
    /// Port to bind to
    pub port: u16,
    /// Startup timeout in seconds
    pub startup_timeout: f64,
    /// Health check endpoint (e.g., "/health")
    pub health_endpoint: Option<String>,
}

/// @spec projects/meter/tech-design/semantic/source/projects-meter-src-http-server-rs.md#source
impl Default for TestServerConfig {
    fn default() -> Self {
        Self {
            app_module: String::new(),
            app_callable: "app".to_string(),
            port: 18765,
            startup_timeout: 10.0,
            health_endpoint: Some("/health".to_string()),
        }
    }
}

/// Handle for a running test server
/// @spec projects/meter/tech-design/semantic/source/projects-meter-src-http-server-rs.md#source
pub struct TestServerHandle {
    /// Server address (e.g., "http://127.0.0.1:8765")
    pub url: String,
    /// Actual port the server is listening on
    pub port: u16,
    /// Shutdown channel (for Axum servers)
    shutdown_tx: Option<oneshot::Sender<()>>,
    /// Process handle (for Python subprocess servers)
    process: Option<Child>,
    /// HTTP client for making requests
    client: Option<reqwest::Client>,
}

/// @spec projects/meter/tech-design/semantic/source/projects-meter-src-http-server-rs.md#source
impl TestServerHandle {
    /// Get the base URL for this server
    pub fn url(&self) -> &str {
        &self.url
    }

    /// Get an HTTP client for making requests to this server
    pub fn client(&self) -> reqwest::Client {
        self.client.clone().unwrap_or_default()
    }

    /// Stop the server
    pub fn stop(mut self) {
        // Stop Axum server
        if let Some(tx) = self.shutdown_tx.take() {
            let _ = tx.send(());
        }
        // Stop subprocess
        if let Some(mut process) = self.process.take() {
            // Try graceful shutdown first
            let _ = process.kill();
        }
    }
}

/// @spec projects/meter/tech-design/semantic/source/projects-meter-src-http-server-rs.md#source
impl Drop for TestServerHandle {
    fn drop(&mut self) {
        if let Some(tx) = self.shutdown_tx.take() {
            let _ = tx.send(());
        }
        if let Some(mut process) = self.process.take() {
            let _ = process.kill();
        }
    }
}

/// Test HTTP server for benchmarking
/// @spec projects/meter/tech-design/semantic/source/projects-meter-src-http-server-rs.md#source
pub struct TestServer {
    routes: HashMap<String, RouteConfig>,
    port: Option<u16>,
    /// Configuration for Python app mode
    app_config: Option<TestServerConfig>,
}

/// @spec projects/meter/tech-design/semantic/source/projects-meter-src-http-server-rs.md#source
impl TestServer {
    /// Create a new test server builder
    pub fn new() -> Self {
        Self {
            routes: HashMap::new(),
            port: None,
            app_config: None,
        }
    }

    /// Create a test server from a Python application
    pub fn from_app(config: TestServerConfig) -> Self {
        Self {
            routes: HashMap::new(),
            port: Some(config.port),
            app_config: Some(config),
        }
    }

    /// Set the port to listen on (default: auto-select)
    pub fn port(mut self, port: u16) -> Self {
        self.port = Some(port);
        self
    }

    /// Add a GET route with static JSON response
    pub fn get(mut self, path: &str, response: JsonValue) -> Self {
        self.routes.insert(
            path.to_string(),
            RouteConfig {
                method: "GET".to_string(),
                response,
                status: 200,
                echo_body: false,
            },
        );
        self
    }

    /// Add a POST route with static JSON response
    pub fn post(mut self, path: &str, response: JsonValue) -> Self {
        self.routes.insert(
            path.to_string(),
            RouteConfig {
                method: "POST".to_string(),
                response,
                status: 200,
                echo_body: false,
            },
        );
        self
    }

    /// Add a POST route that echoes the request body
    pub fn post_echo(mut self, path: &str) -> Self {
        self.routes.insert(
            path.to_string(),
            RouteConfig {
                method: "POST".to_string(),
                response: JsonValue::Null,
                status: 200,
                echo_body: true,
            },
        );
        self
    }

    /// Add a route with full configuration
    pub fn route(mut self, path: &str, config: RouteConfig) -> Self {
        self.routes.insert(path.to_string(), config);
        self
    }

    /// Add multiple routes from a HashMap
    pub fn routes(mut self, routes: HashMap<String, JsonValue>) -> Self {
        for (path, response) in routes {
            self.routes.insert(
                path,
                RouteConfig {
                    method: "GET".to_string(),
                    response,
                    status: 200,
                    echo_body: false,
                },
            );
        }
        self
    }

    /// Start the server and return a handle
    pub async fn start(mut self) -> anyhow::Result<TestServerHandle> {
        // Check if this is a Python app server or Axum server
        if let Some(app_config) = self.app_config.take() {
            Self::start_python_app_static(app_config).await
        } else {
            self.start_axum_server().await
        }
    }

    /// Start an Axum server with static routes
    async fn start_axum_server(self) -> anyhow::Result<TestServerHandle> {
        let state = ServerState {
            routes: Arc::new(self.routes),
        };

        // Build the router with a catch-all handler
        let app = Router::new()
            .route("/{*path}", get(handle_request).post(handle_request))
            .route("/", get(handle_root).post(handle_root))
            .with_state(state);

        // Bind to port
        let addr = if let Some(port) = self.port {
            format!("127.0.0.1:{}", port)
        } else {
            "127.0.0.1:0".to_string()
        };

        let listener = TcpListener::bind(&addr).await?;
        let actual_addr = listener.local_addr()?;
        let port = actual_addr.port();
        let url = format!("http://127.0.0.1:{}", port);

        // Create shutdown channel
        let (shutdown_tx, shutdown_rx) = oneshot::channel::<()>();

        // Spawn the server
        tokio::spawn(async move {
            axum::serve(listener, app)
                .with_graceful_shutdown(async {
                    let _ = shutdown_rx.await;
                })
                .await
                .ok();
        });

        let client = reqwest::Client::new();

        Ok(TestServerHandle {
            url,
            port,
            shutdown_tx: Some(shutdown_tx),
            process: None,
            client: Some(client),
        })
    }

    /// Start a Python application as a subprocess (static method)
    async fn start_python_app_static(config: TestServerConfig) -> anyhow::Result<TestServerHandle> {
        // Build Python command to start the app
        // Import module and get the callable, then run it
        let python_cmd = format!(
            "python3 -c 'from {} import {}; {}.run(host=\"127.0.0.1\", port={})'",
            config.app_module, config.app_callable, config.app_callable, config.port
        );

        // Spawn the subprocess
        let mut child = Command::new("sh")
            .arg("-c")
            .arg(&python_cmd)
            .spawn()
            .map_err(|e| anyhow::anyhow!("Failed to spawn Python app: {}", e))?;

        let url = format!("http://127.0.0.1:{}", config.port);
        let client = reqwest::Client::new();

        // Wait for server to be ready
        let start = Instant::now();
        let timeout_duration = Duration::from_secs_f64(config.startup_timeout);

        loop {
            // Check if process is still running
            match child.try_wait() {
                Ok(Some(status)) => {
                    return Err(anyhow::anyhow!(
                        "Python app exited prematurely with status: {}",
                        status
                    ));
                }
                Ok(None) => {
                    // Process is still running, continue
                }
                Err(e) => {
                    return Err(anyhow::anyhow!("Failed to check process status: {}", e));
                }
            }

            // Try health check
            if let Some(health_endpoint) = &config.health_endpoint {
                let health_url = format!("{}{}", url, health_endpoint);
                match client.get(&health_url).send().await {
                    Ok(resp) if resp.status().is_success() => {
                        // Server is ready
                        break;
                    }
                    _ => {
                        // Not ready yet, continue
                    }
                }
            } else {
                // No health endpoint, just try to connect to the port
                match tokio::net::TcpStream::connect(format!("127.0.0.1:{}", config.port)).await {
                    Ok(_) => {
                        // Port is open, server is ready
                        break;
                    }
                    Err(_) => {
                        // Port not ready yet
                    }
                }
            }

            // Check timeout
            if start.elapsed() > timeout_duration {
                // Kill the process on timeout
                let _ = child.kill();
                return Err(anyhow::anyhow!(
                    "Server startup timeout after {:.1}s",
                    config.startup_timeout
                ));
            }

            // Wait a bit before retrying
            tokio::time::sleep(Duration::from_millis(100)).await;
        }

        Ok(TestServerHandle {
            url,
            port: config.port,
            shutdown_tx: None,
            process: Some(child),
            client: Some(client),
        })
    }
}

/// @spec projects/meter/tech-design/semantic/source/projects-meter-src-http-server-rs.md#source
impl Default for TestServer {
    fn default() -> Self {
        Self::new()
    }
}

/// Handle requests to the root path
async fn handle_root(
    method: Method,
    State(state): State<ServerState>,
    body: Option<Json<JsonValue>>,
) -> Response {
    handle_path("/".to_string(), method, State(state), body).await
}

/// Handle requests to any path
async fn handle_request(
    Path(path): Path<String>,
    method: Method,
    State(state): State<ServerState>,
    body: Option<Json<JsonValue>>,
) -> Response {
    let full_path = format!("/{}", path);
    handle_path(full_path, method, State(state), body).await
}

/// Common handler for all paths
async fn handle_path(
    path: String,
    method: Method,
    State(state): State<ServerState>,
    body: Option<Json<JsonValue>>,
) -> Response {
    // Look up the route
    if let Some(config) = state.routes.get(&path) {
        // Check method
        if (method == Method::GET && config.method == "GET")
            || (method == Method::POST && config.method == "POST")
            || config.method == "*"
        {
            if config.echo_body {
                // Echo the request body
                if let Some(Json(body)) = body {
                    let response = serde_json::json!({
                        "received": body,
                        "status": "ok"
                    });
                    return (
                        StatusCode::from_u16(config.status).unwrap_or(StatusCode::OK),
                        Json(response),
                    )
                        .into_response();
                }
            }
            return (
                StatusCode::from_u16(config.status).unwrap_or(StatusCode::OK),
                Json(config.response.clone()),
            )
                .into_response();
        }
    }

    // Route not found
    (
        StatusCode::NOT_FOUND,
        Json(serde_json::json!({
            "error": "Not found",
            "path": path
        })),
    )
        .into_response()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_server_basic() {
        let server = TestServer::new()
            .get("/test", serde_json::json!({"status": "ok"}))
            .start()
            .await
            .unwrap();

        // Make a request
        let client = reqwest::Client::new();
        let resp = client
            .get(format!("{}/test", server.url()))
            .send()
            .await
            .unwrap();

        assert_eq!(resp.status(), 200);
        let body: JsonValue = resp.json().await.unwrap();
        assert_eq!(body["status"], "ok");

        server.stop();
    }

    #[tokio::test]
    async fn test_server_post() {
        let server = TestServer::new().post_echo("/echo").start().await.unwrap();

        let client = reqwest::Client::new();
        let resp = client
            .post(format!("{}/echo", server.url()))
            .json(&serde_json::json!({"name": "test"}))
            .send()
            .await
            .unwrap();

        assert_eq!(resp.status(), 200);
        let body: JsonValue = resp.json().await.unwrap();
        assert_eq!(body["received"]["name"], "test");

        server.stop();
    }
}
// CODEGEN-END
