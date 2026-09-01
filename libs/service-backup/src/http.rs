//! Shared HTTP transport for a service's standard admin snapshot endpoint.

use std::{
    fmt,
    path::Path,
    time::{Duration, SystemTime},
};

use anyhow::{bail, Context, Result};

const CONNECT_TIMEOUT: Duration = Duration::from_secs(10);
const OPERATION_TIMEOUT: Duration = Duration::from_secs(2 * 60 * 60);
const BACKUP_IDLE_TIMEOUT: Duration = Duration::from_secs(30);

/// The standard operation performed by [`AdminSnapshotTransport`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AdminSnapshotOperation {
    Backup,
    Restore,
}

/// Redacted errors from the strict admin snapshot transport.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AdminSnapshotTransportError {
    ClientBuildFailed,
    RequestFailed {
        operation: AdminSnapshotOperation,
    },
    UnexpectedStatus {
        operation: AdminSnapshotOperation,
        status: reqwest::StatusCode,
    },
    ResponseReadFailed {
        operation: AdminSnapshotOperation,
    },
}

impl fmt::Display for AdminSnapshotTransportError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ClientBuildFailed => f.write_str("admin snapshot client build failed"),
            Self::RequestFailed { operation } => {
                write!(f, "admin snapshot {operation:?} request failed")
            }
            Self::UnexpectedStatus { operation, status } => {
                write!(
                    f,
                    "admin snapshot {operation:?} returned unexpected status {status}"
                )
            }
            Self::ResponseReadFailed { operation } => {
                write!(f, "admin snapshot {operation:?} response read failed")
            }
        }
    }
}

impl std::error::Error for AdminSnapshotTransportError {}

impl AdminSnapshotOperation {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Backup => "backup",
            Self::Restore => "restore",
        }
    }
}

/// Shared transport limits. Products may lower these values but do not need to
/// own request, redirect, retry, idle-read, or diagnostic-body control flow.
#[derive(Clone, Copy, Debug)]
pub struct AdminSnapshotTransportConfig {
    pub connect_timeout: Duration,
    pub operation_timeout: Duration,
    pub response_idle_timeout: Duration,
    pub max_diagnostic_bytes: usize,
}

impl Default for AdminSnapshotTransportConfig {
    fn default() -> Self {
        Self {
            connect_timeout: CONNECT_TIMEOUT,
            operation_timeout: OPERATION_TIMEOUT,
            response_idle_timeout: BACKUP_IDLE_TIMEOUT,
            max_diagnostic_bytes: 8 * 1024,
        }
    }
}

enum AdminCredential {
    None,
    Static(String),
    Projected(service_auth::k8s::ProjectedTokenFile),
}

/// Per-request product policy for a shared admin snapshot call.
pub struct AdminSnapshotRequest {
    credential: AdminCredential,
    headers: reqwest::header::HeaderMap,
}

impl AdminSnapshotRequest {
    pub fn new() -> Self {
        Self {
            credential: AdminCredential::None,
            headers: reqwest::header::HeaderMap::new(),
        }
    }

    pub fn with_static_bearer(mut self, token: impl Into<String>) -> Self {
        self.credential = AdminCredential::Static(token.into());
        self
    }

    /// Store only the file descriptor policy. The file is opened and checked
    /// immediately before every request so kubelet token rotation is observed.
    pub fn with_projected_bearer(
        mut self,
        path: impl AsRef<Path>,
        audience: impl Into<String>,
    ) -> Self {
        self.credential = AdminCredential::Projected(
            service_auth::k8s::ProjectedTokenFile::new(path.as_ref(), audience),
        );
        self
    }

    pub fn with_header(
        mut self,
        name: &str,
        value: &str,
    ) -> std::result::Result<Self, AdminSnapshotRequestError> {
        let name = reqwest::header::HeaderName::from_bytes(name.as_bytes())
            .map_err(|_| AdminSnapshotRequestError::InvalidHeader)?;
        let value = reqwest::header::HeaderValue::from_str(value)
            .map_err(|_| AdminSnapshotRequestError::InvalidHeader)?;
        self.headers.insert(name, value);
        Ok(self)
    }
}

impl Default for AdminSnapshotRequest {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AdminSnapshotDiagnostic {
    pub operation: AdminSnapshotOperation,
    pub status: u16,
    pub body: String,
    pub truncated: bool,
}

#[derive(Debug)]
pub enum AdminSnapshotRequestError {
    InvalidHeader,
    CredentialFailed { operation: AdminSnapshotOperation },
    RequestFailed { operation: AdminSnapshotOperation },
    UnexpectedStatus { diagnostic: AdminSnapshotDiagnostic },
    ResponseReadFailed { operation: AdminSnapshotOperation },
}

impl AdminSnapshotRequestError {
    pub fn diagnostic(&self) -> Option<&AdminSnapshotDiagnostic> {
        match self {
            Self::UnexpectedStatus { diagnostic } => Some(diagnostic),
            _ => None,
        }
    }
}

impl fmt::Display for AdminSnapshotRequestError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidHeader => f.write_str("admin snapshot request header is invalid"),
            Self::CredentialFailed { operation } => write!(
                f,
                "admin snapshot {} credential could not be read or validated",
                operation.as_str()
            ),
            Self::RequestFailed { operation } => {
                write!(f, "admin snapshot {} request failed", operation.as_str())
            }
            Self::UnexpectedStatus { diagnostic } => write!(
                f,
                "admin snapshot {} returned status {}: {}{}",
                diagnostic.operation.as_str(),
                diagnostic.status,
                diagnostic.body,
                if diagnostic.truncated { " [truncated]" } else { "" }
            ),
            Self::ResponseReadFailed { operation } => write!(
                f,
                "admin snapshot {} response read failed",
                operation.as_str()
            ),
        }
    }
}

impl std::error::Error for AdminSnapshotRequestError {}

fn redacted_error(error: AdminSnapshotRequestError) -> AdminSnapshotTransportError {
    match error {
        AdminSnapshotRequestError::UnexpectedStatus { diagnostic } => {
            AdminSnapshotTransportError::UnexpectedStatus {
                operation: diagnostic.operation,
                status: reqwest::StatusCode::from_u16(diagnostic.status)
                    .unwrap_or(reqwest::StatusCode::INTERNAL_SERVER_ERROR),
            }
        }
        AdminSnapshotRequestError::ResponseReadFailed { operation } => {
            AdminSnapshotTransportError::ResponseReadFailed { operation }
        }
        AdminSnapshotRequestError::InvalidHeader
        | AdminSnapshotRequestError::CredentialFailed { .. }
        | AdminSnapshotRequestError::RequestFailed { .. } => {
            AdminSnapshotTransportError::RequestFailed {
                operation: AdminSnapshotOperation::Backup,
            }
        }
    }
}

/// Strict, byte-preserving transport for the standard admin snapshot endpoints.
pub struct AdminSnapshotTransport {
    client: reqwest::Client,
    config: AdminSnapshotTransportConfig,
}

impl AdminSnapshotTransport {
    /// Construct a transport with production timeouts and no redirect policy.
    pub fn new() -> std::result::Result<Self, AdminSnapshotTransportError> {
        Self::with_config(AdminSnapshotTransportConfig::default())
    }

    pub fn with_config(
        config: AdminSnapshotTransportConfig,
    ) -> std::result::Result<Self, AdminSnapshotTransportError> {
        let client = reqwest::Client::builder()
            .connect_timeout(config.connect_timeout)
            .redirect(reqwest::redirect::Policy::none())
            .retry(reqwest::retry::never())
            .build()
            .map_err(|_| AdminSnapshotTransportError::ClientBuildFailed)?;
        Ok(Self { client, config })
    }

    #[cfg(test)]
    fn for_tests(operation: Duration, backup_idle: Duration) -> Self {
        Self::with_config(AdminSnapshotTransportConfig {
            operation_timeout: operation,
            response_idle_timeout: backup_idle,
            ..Default::default()
        })
        .expect("test client builds")
    }

    /// Fetch exact snapshot bytes with a product-supplied credential and
    /// metadata policy. Status diagnostics are bounded by the transport config.
    pub async fn fetch(
        &self,
        base_url: &str,
        policy: &AdminSnapshotRequest,
    ) -> std::result::Result<Vec<u8>, AdminSnapshotRequestError> {
        let operation = AdminSnapshotOperation::Backup;
        let result = tokio::time::timeout(self.config.operation_timeout, async {
            let url = format!("{}/admin/backup", base_url.trim_end_matches('/'));
            let mut request = self.client.get(url).headers(policy.headers.clone());
            match &policy.credential {
                AdminCredential::None => {}
                AdminCredential::Static(token) => {
                    request = request.bearer_auth(token);
                }
                AdminCredential::Projected(file) => {
                    let token = file
                        .read()
                        .map_err(|_| AdminSnapshotRequestError::CredentialFailed { operation })?;
                    request = request.bearer_auth(token.expose());
                }
            }
            let mut response = request
                .send()
                .await
                .map_err(|_| AdminSnapshotRequestError::RequestFailed { operation })?;
            if response.status() != reqwest::StatusCode::OK {
                let status = response.status().as_u16();
                let mut body = Vec::new();
                let limit = self.config.max_diagnostic_bytes;
                let mut truncated = false;
                loop {
                    let chunk = tokio::time::timeout(
                        self.config.response_idle_timeout,
                        response.chunk(),
                    )
                    .await
                    .map_err(|_| AdminSnapshotRequestError::ResponseReadFailed { operation })?
                    .map_err(|_| AdminSnapshotRequestError::ResponseReadFailed { operation })?;
                    let Some(chunk) = chunk else { break };
                    let remaining = limit.saturating_sub(body.len());
                    if chunk.len() > remaining {
                        body.extend_from_slice(&chunk[..remaining]);
                        truncated = true;
                        break;
                    }
                    body.extend_from_slice(&chunk);
                    if body.len() == limit {
                        let more = tokio::time::timeout(
                            self.config.response_idle_timeout,
                            response.chunk(),
                        )
                        .await
                        .map_err(|_| AdminSnapshotRequestError::ResponseReadFailed { operation })?
                        .map_err(|_| AdminSnapshotRequestError::ResponseReadFailed { operation })?;
                        truncated = more.is_some();
                        break;
                    }
                }
                return Err(AdminSnapshotRequestError::UnexpectedStatus {
                    diagnostic: AdminSnapshotDiagnostic {
                        operation,
                        status,
                        body: String::from_utf8_lossy(&body).into_owned(),
                        truncated,
                    },
                });
            }
            let mut payload = Vec::new();
            loop {
                let chunk = tokio::time::timeout(
                    self.config.response_idle_timeout,
                    response.chunk(),
                )
                .await
                .map_err(|_| AdminSnapshotRequestError::ResponseReadFailed { operation })?
                .map_err(|_| AdminSnapshotRequestError::ResponseReadFailed { operation })?;
                let Some(chunk) = chunk else { break };
                payload.extend_from_slice(&chunk);
            }
            Ok(payload)
        })
        .await;
        result.map_err(|_| AdminSnapshotRequestError::RequestFailed { operation })?
    }

    /// Fetch the exact bytes returned by one `GET /admin/backup` request.
    pub async fn fetch_exact(
        &self,
        base_url: &str,
        bearer: Option<&str>,
    ) -> std::result::Result<Vec<u8>, AdminSnapshotTransportError> {
        let mut policy = AdminSnapshotRequest::new();
        if let Some(bearer) = bearer {
            policy = policy.with_static_bearer(bearer);
        }
        self.fetch(base_url, &policy).await.map_err(redacted_error)
    }

    /// Restore exact snapshot bytes with one `POST /admin/restore` request.
    pub async fn restore_exact(
        &self,
        base_url: &str,
        bearer: Option<&str>,
        snapshot: &[u8],
    ) -> std::result::Result<(), AdminSnapshotTransportError> {
        let operation = AdminSnapshotOperation::Restore;
        tokio::time::timeout(self.config.operation_timeout, async {
            let url = format!("{}/admin/restore", base_url.trim_end_matches('/'));
            let mut request = self
                .client
                .post(url)
                .header(reqwest::header::CONTENT_TYPE, "application/json")
                .body(snapshot.to_vec());
            if let Some(token) = bearer {
                request = request.bearer_auth(token);
            }
            let response = request
                .send()
                .await
                .map_err(|_| AdminSnapshotTransportError::RequestFailed { operation })?;
            if response.status() != reqwest::StatusCode::NO_CONTENT {
                return Err(AdminSnapshotTransportError::UnexpectedStatus {
                    operation,
                    status: response.status(),
                });
            }
            Ok(())
        })
        .await
        .map_err(|_| AdminSnapshotTransportError::RequestFailed { operation })?
    }
}

use crate::{
    run_backup_once, sink_from_destination, BackupDestination, BackupRunResult, RetentionPolicy,
};

/// Fetch exact snapshot bytes from the standard `GET /admin/backup` endpoint.
///
/// Services retain the domain-specific snapshot encoding and restore policy.
/// This helper owns the common Bearer request, non-success diagnostic, and
/// byte-preserving response read used by service backup CLIs and CronJobs.
pub async fn fetch_admin_snapshot(base_url: &str, token: Option<&str>) -> Result<Vec<u8>> {
    let url = format!("{}/admin/backup", base_url.trim_end_matches('/'));
    let client = reqwest::Client::new();
    let mut request = client.get(&url);
    if let Some(token) = token {
        request = request.bearer_auth(token);
    }
    let response = request.send().await.with_context(|| format!("GET {url}"))?;
    let status = response.status();
    if !status.is_success() {
        let body = response.text().await.unwrap_or_default();
        bail!("GET {url} returned {status}: {body}");
    }
    let payload = response
        .bytes()
        .await
        .with_context(|| format!("read response body from {url}"))?;
    Ok(payload.to_vec())
}

/// Fetch an admin snapshot and ship the exact bytes to `dest`.
///
/// `file://` always works. `s3://` requires the crate's `s3` feature; `gs://`
/// remains schema-compatible and fails loudly until a GCS sink exists.
pub async fn run_admin_snapshot_backup(
    base_url: &str,
    token: Option<&str>,
    dest: &BackupDestination,
    retention: &RetentionPolicy,
) -> Result<BackupRunResult> {
    let payload = fetch_admin_snapshot(base_url, token).await?;
    let sink = sink_from_destination(dest)?;
    run_backup_once(sink.as_ref(), SystemTime::now(), &payload, retention)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    };
    use tokio::{
        io::{AsyncReadExt, AsyncWriteExt},
        net::TcpListener,
    };
    use wiremock::matchers::{body_bytes, header, method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    #[tokio::test]
    async fn fetches_exact_snapshot_bytes_with_bearer_auth() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/admin/backup"))
            .and(header("authorization", "Bearer registry-token"))
            .respond_with(ResponseTemplate::new(200).set_body_bytes(b"snapshot".to_vec()))
            .mount(&server)
            .await;

        let bytes = fetch_admin_snapshot(&server.uri(), Some("registry-token"))
            .await
            .expect("admin snapshot fetch succeeds");
        assert_eq!(bytes, b"snapshot");
    }

    #[tokio::test]
    async fn keeps_non_success_status_and_body_in_the_diagnostic() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/admin/backup"))
            .respond_with(ResponseTemplate::new(503).set_body_string("not ready"))
            .mount(&server)
            .await;

        let error = fetch_admin_snapshot(&server.uri(), None)
            .await
            .expect_err("non-success status must fail");
        assert!(error.to_string().contains("503"));
        assert!(error.to_string().contains("not ready"));
    }

    #[tokio::test]
    async fn strict_backup_accepts_only_200_and_has_no_bearer_path() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/admin/backup"))
            .respond_with(ResponseTemplate::new(201).set_body_bytes(b"secret-payload".to_vec()))
            .expect(1)
            .mount(&server)
            .await;
        let transport = AdminSnapshotTransport::for_tests(
            Duration::from_millis(200),
            Duration::from_millis(50),
        );
        let error = transport
            .fetch_exact(&server.uri(), None)
            .await
            .expect_err("201 is not exact 200");
        assert_eq!(
            error,
            AdminSnapshotTransportError::UnexpectedStatus {
                operation: AdminSnapshotOperation::Backup,
                status: reqwest::StatusCode::CREATED,
            }
        );
        assert!(!error.to_string().contains("secret-payload"));
    }

    #[tokio::test]
    async fn strict_backup_preserves_exact_200_bytes_without_bearer() {
        let server = MockServer::start().await;
        let payload = b"bytes\x00with\xffopaque".to_vec();
        Mock::given(method("GET"))
            .and(path("/admin/backup"))
            .respond_with(ResponseTemplate::new(200).set_body_bytes(payload.clone()))
            .expect(1)
            .mount(&server)
            .await;
        let transport = AdminSnapshotTransport::for_tests(
            Duration::from_millis(200),
            Duration::from_millis(50),
        );
        assert_eq!(
            transport.fetch_exact(&server.uri(), None).await.unwrap(),
            payload
        );
    }

    #[tokio::test]
    async fn strict_restore_preserves_body_and_content_type_and_accepts_only_204() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/admin/restore"))
            .and(header("content-type", "application/json"))
            .and(header("authorization", "Bearer restore-token"))
            .and(body_bytes(b"{\"opaque\":true}"))
            .respond_with(ResponseTemplate::new(204))
            .expect(1)
            .mount(&server)
            .await;
        let transport = AdminSnapshotTransport::for_tests(
            Duration::from_millis(200),
            Duration::from_millis(50),
        );
        transport
            .restore_exact(&server.uri(), Some("restore-token"), b"{\"opaque\":true}")
            .await
            .expect("restore succeeds");
    }

    #[tokio::test]
    async fn strict_restore_rejects_other_success_status_without_body_leak() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/admin/restore"))
            .respond_with(ResponseTemplate::new(200).set_body_bytes(b"canary-body".to_vec()))
            .mount(&server)
            .await;
        let transport = AdminSnapshotTransport::for_tests(
            Duration::from_millis(200),
            Duration::from_millis(50),
        );
        let error = transport
            .restore_exact(&server.uri(), Some("canary-token"), b"canary-payload")
            .await
            .expect_err("200 is not exact 204");
        assert_eq!(
            error,
            AdminSnapshotTransportError::UnexpectedStatus {
                operation: AdminSnapshotOperation::Restore,
                status: reqwest::StatusCode::OK,
            }
        );
        let text = error.to_string();
        assert!(!text.contains("canary"));
    }

    #[tokio::test]
    async fn strict_backup_does_not_follow_redirects() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/admin/backup"))
            .respond_with(
                ResponseTemplate::new(302)
                    .insert_header("Location", "http://127.0.0.1:1/admin/backup"),
            )
            .expect(1)
            .mount(&server)
            .await;
        let transport = AdminSnapshotTransport::for_tests(
            Duration::from_millis(200),
            Duration::from_millis(50),
        );
        let error = transport
            .fetch_exact(&server.uri(), Some("redirect-token"))
            .await
            .expect_err("redirect is not 200");
        assert!(
            matches!(error, AdminSnapshotTransportError::UnexpectedStatus { status, .. } if status == reqwest::StatusCode::FOUND)
        );
        assert!(!error.to_string().contains("redirect-token"));
    }

    #[tokio::test]
    async fn strict_request_timeout_is_redacted() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/admin/backup"))
            .respond_with(ResponseTemplate::new(200).set_delay(Duration::from_millis(150)))
            .mount(&server)
            .await;
        let transport =
            AdminSnapshotTransport::for_tests(Duration::from_millis(40), Duration::from_millis(20));
        let error = transport
            .fetch_exact(&server.uri(), Some("timeout-token"))
            .await
            .expect_err("request times out");
        assert_eq!(
            error,
            AdminSnapshotTransportError::RequestFailed {
                operation: AdminSnapshotOperation::Backup
            }
        );
        assert!(!error.to_string().contains("timeout-token"));
    }

    #[tokio::test]
    async fn strict_backup_idle_timeout_maps_stalled_body_read() {
        let listener = TcpListener::bind(("127.0.0.1", 0))
            .await
            .expect("bind test listener");
        let address = listener.local_addr().expect("listener address");
        let task = tokio::spawn(async move {
            let (mut socket, _) = listener.accept().await.expect("accept test request");
            let mut request = [0_u8; 2048];
            let _ = socket.read(&mut request).await;
            socket
                .write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 2\r\nConnection: close\r\n\r\na")
                .await
                .expect("write initial chunk");
            tokio::time::sleep(Duration::from_millis(120)).await;
            let _ = socket.write_all(b"b").await;
        });
        let transport = AdminSnapshotTransport::for_tests(
            Duration::from_millis(500),
            Duration::from_millis(40),
        );
        let error = transport
            .fetch_exact(&format!("http://{address}"), None)
            .await
            .expect_err("stalled body must fail");
        assert_eq!(
            error,
            AdminSnapshotTransportError::ResponseReadFailed {
                operation: AdminSnapshotOperation::Backup
            }
        );
        task.await.expect("server task exits");
    }

    #[tokio::test]
    async fn strict_backup_idle_timeout_resets_for_each_chunk() {
        let listener = TcpListener::bind(("127.0.0.1", 0))
            .await
            .expect("bind test listener");
        let address = listener.local_addr().expect("listener address");
        let task = tokio::spawn(async move {
            let (mut socket, _) = listener.accept().await.expect("accept test request");
            let mut request = [0_u8; 2048];
            let _ = socket.read(&mut request).await;
            socket
                .write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 3\r\nConnection: close\r\n\r\na")
                .await
                .expect("write first chunk");
            tokio::time::sleep(Duration::from_millis(45)).await;
            socket.write_all(b"b").await.expect("write second chunk");
            tokio::time::sleep(Duration::from_millis(45)).await;
            socket.write_all(b"c").await.expect("write final chunk");
        });
        let transport = AdminSnapshotTransport::for_tests(
            Duration::from_millis(500),
            Duration::from_millis(80),
        );
        assert_eq!(
            transport
                .fetch_exact(&format!("http://{address}"), None)
                .await
                .expect("chunks arrive within idle timeout"),
            b"abc"
        );
        task.await.expect("server task exits");
    }

    #[tokio::test]
    async fn strict_transport_failure_uses_one_connection_without_retry() {
        let listener = TcpListener::bind(("127.0.0.1", 0))
            .await
            .expect("bind test listener");
        let address = listener.local_addr().expect("listener address");
        let connections = Arc::new(AtomicUsize::new(0));
        let seen = Arc::clone(&connections);
        let task = tokio::spawn(async move {
            let (mut socket, _) =
                tokio::time::timeout(Duration::from_millis(150), listener.accept())
                    .await
                    .expect("first connection arrives")
                    .expect("first accept succeeds");
            seen.fetch_add(1, Ordering::SeqCst);
            let mut request = [0_u8; 2048];
            let _ = socket.read(&mut request).await;
            let _ = socket.shutdown().await;
            if let Ok(Ok((mut retry_socket, _))) =
                tokio::time::timeout(Duration::from_millis(150), listener.accept()).await
            {
                seen.fetch_add(1, Ordering::SeqCst);
                let _ = retry_socket.shutdown().await;
            }
        });
        let transport = AdminSnapshotTransport::for_tests(
            Duration::from_millis(300),
            Duration::from_millis(30),
        );
        let error = transport
            .fetch_exact(&format!("http://{address}"), None)
            .await
            .expect_err("closed response must fail");
        assert!(matches!(
            error,
            AdminSnapshotTransportError::RequestFailed {
                operation: AdminSnapshotOperation::Backup
            } | AdminSnapshotTransportError::ResponseReadFailed {
                operation: AdminSnapshotOperation::Backup
            }
        ));
        task.await.expect("server task exits");
        assert_eq!(connections.load(Ordering::SeqCst), 1);
    }
}
