---
id: libs-compass-src-server-daemon-rs
summary: Lossless rust-source-unit coverage for `libs/compass/src/server/daemon.rs`.
capability_refs:
- id: codebase-check-and-lint-pipeline
  role: primary
  claim: multi-language-parser-and-checker-dispatch-contract
  gap: multi-language-parser-and-checker-dispatch-contract
  coverage: full
  rationale: "Multi-language parser and checker dispatch contract is implemented by the existing Compass library surface and covered by the configured smoke gate."
- id: codebase-check-and-lint-pipeline
  role: primary
  claim: agent-diagnostic-output-contract
  gap: agent-diagnostic-output-contract
  coverage: full
  rationale: "Agent diagnostic output contract is implemented by the existing Compass library surface and covered by the configured smoke gate."
- id: semantic-navigation-search-and-refactoring
  role: primary
  claim: symbol-outline-and-propagated-type-query-contract
  gap: symbol-outline-and-propagated-type-query-contract
  coverage: full
  rationale: "Symbol outline and propagated type query contract is implemented by the existing Compass library surface and covered by the configured smoke gate."
- id: semantic-navigation-search-and-refactoring
  role: primary
  claim: semantic-search-and-graph-query-contract
  gap: semantic-search-and-graph-query-contract
  coverage: full
  rationale: "Semantic search and graph query contract is implemented by the existing Compass library surface and covered by the configured smoke gate."
- id: semantic-navigation-search-and-refactoring
  role: primary
  claim: structured-refactoring-contract
  gap: structured-refactoring-contract
  coverage: full
  rationale: "Structured refactoring contract is implemented by the existing Compass library surface and covered by the configured smoke gate."
- id: spec-parsing-and-code-generation
  role: primary
  claim: spec-parser-and-state-machine-validation-contract
  gap: spec-parser-and-state-machine-validation-contract
  coverage: full
  rationale: "Spec parser and state-machine validation contract is implemented by the existing Compass library surface and covered by the configured smoke gate."
- id: spec-parsing-and-code-generation
  role: primary
  claim: python-and-rust-generator-registry-contract
  gap: python-and-rust-generator-registry-contract
  coverage: full
  rationale: "Python and Rust generator registry contract is implemented by the existing Compass library surface and covered by the configured smoke gate."
- id: daemon-watch-and-incremental-analysis
  role: primary
  claim: argus-daemon-protocol-and-request-handling-contract
  gap: argus-daemon-protocol-and-request-handling-contract
  coverage: full
  rationale: "Argus daemon protocol and request handling contract is implemented by the existing Compass library surface and covered by the configured smoke gate."
- id: daemon-watch-and-incremental-analysis
  role: primary
  claim: watch-bridge-and-incremental-dirty-file-contract
  gap: watch-bridge-and-incremental-dirty-file-contract
  coverage: full
  rationale: "Watch bridge and incremental dirty-file contract is implemented by the existing Compass library surface and covered by the configured smoke gate."
fill_sections: [overview, source, changes]
---

# Standardized libs/compass/src/server/daemon.rs

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `libs/compass/src/server/daemon.rs` captured during libs codegen standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `DaemonConfig` | libs/compass/src/server/daemon.rs | struct | pub | 18 | pub struct DaemonConfig { |
| `new` | libs/compass/src/server/daemon.rs | function | pub | 31 | pub fn new(root: PathBuf) -> Self { |
| `default_socket_path` | libs/compass/src/server/daemon.rs | function | pub | 45 | pub fn default_socket_path(root: &PathBuf) -> PathBuf { |
| `with_socket` | libs/compass/src/server/daemon.rs | function | pub | 56 | pub fn with_socket(mut self, path: PathBuf) -> Self { |
| `with_watch` | libs/compass/src/server/daemon.rs | function | pub | 62 | pub fn with_watch(mut self, enabled: bool) -> Self { |
| `AnalysisQueueStatus` | libs/compass/src/server/daemon.rs | struct | pub | 70 | pub struct AnalysisQueueStatus { |
| `ArgusDaemon` | libs/compass/src/server/daemon.rs | struct | pub | 78 | pub struct ArgusDaemon { |
| `run` | libs/compass/src/server/daemon.rs | function | pub | 141 | pub async fn run(&self) -> Result<(), String> { |
| `handler_for_file` | libs/compass/src/server/daemon.rs | function | pub | 259 | pub fn handler_for_file(&self, file_path: &std::path::Path) -> Arc<RequestHandler> { |
| `list_scopes` | libs/compass/src/server/daemon.rs | function | pub | 273 | pub fn list_scopes(&self) -> Vec<(String, PathBuf)> { |
| `shutdown` | libs/compass/src/server/daemon.rs | function | pub | 281 | pub fn shutdown(&self) { |
| `is_running` | libs/compass/src/server/daemon.rs | function | pub | 286 | pub async fn is_running(&self) -> bool { |
| `socket_path` | libs/compass/src/server/daemon.rs | function | pub | 291 | pub fn socket_path(&self) -> &PathBuf { |
| `queue_status` | libs/compass/src/server/daemon.rs | function | pub | 296 | pub async fn queue_status(&self) -> AnalysisQueueStatus { |
| `DaemonClient` | libs/compass/src/server/daemon.rs | struct | pub | 476 | pub struct DaemonClient { |
| `for_workspace` | libs/compass/src/server/daemon.rs | function | pub | 487 | pub fn for_workspace(root: &PathBuf) -> Self { |
| `is_daemon_running` | libs/compass/src/server/daemon.rs | function | pub | 493 | pub async fn is_daemon_running(&self) -> bool { |
| `request` | libs/compass/src/server/daemon.rs | function | pub | 498 | pub async fn request( |
| `check` | libs/compass/src/server/daemon.rs | function | pub | 548 | pub async fn check(&self, path: &str) -> Result<serde_json::Value, String> { |
| `type_at` | libs/compass/src/server/daemon.rs | function | pub | 553 | pub async fn type_at( |
| `symbols` | libs/compass/src/server/daemon.rs | function | pub | 570 | pub async fn symbols(&self, file: &str) -> Result<serde_json::Value, String> { |
| `diagnostics` | libs/compass/src/server/daemon.rs | function | pub | 575 | pub async fn diagnostics(&self, file: Option<&str>) -> Result<serde_json::Value, String> { |
| `index_status` | libs/compass/src/server/daemon.rs | function | pub | 580 | pub async fn index_status(&self) -> Result<serde_json::Value, String> { |
| `invalidate` | libs/compass/src/server/daemon.rs | function | pub | 593 | pub async fn invalidate(&self, files: &[&str]) -> Result<serde_json::Value, String> { |
| `hover` | libs/compass/src/server/daemon.rs | function | pub | 603 | pub async fn hover( |
| `definition` | libs/compass/src/server/daemon.rs | function | pub | 621 | pub async fn definition( |
| `references` | libs/compass/src/server/daemon.rs | function | pub | 639 | pub async fn references( |


## Source
<!-- type: rust-source-unit lang: rust -->

````rust
//! Argus Daemon - Long-running code analysis server

use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, Instant};

use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{UnixListener, UnixStream};
use tokio::sync::{broadcast, mpsc, RwLock};

use super::handler::RequestHandler;
use super::protocol::{Request, Response, RpcError};
use super::watch_bridge::{spawn_watch_bridge, BridgeEvent, WatchBridgeHandle};

/// Daemon configuration
#[derive(Debug, Clone)]
pub struct DaemonConfig {
    /// Root directory to analyze
    pub root: PathBuf,
    /// Unix socket path
    pub socket_path: PathBuf,
    /// Enable file watching
    pub watch: bool,
    /// Watch debounce duration
    pub debounce: Duration,
}

impl DaemonConfig {
    /// Create config with default socket path based on workspace hash
    pub fn new(root: PathBuf) -> Self {
        let socket_path = Self::default_socket_path(&root);
        Self {
            root,
            socket_path,
            watch: true,
            debounce: Duration::from_millis(300),
        }
    }

    /// Generate default socket path from workspace root.
    ///
    /// Uses `{root}/cclab/.index/daemon.sock` for local project storage.
    /// Falls back to `/tmp/cclab_lens-{hash}.sock` if resolution fails.
    pub fn default_socket_path(root: &PathBuf) -> PathBuf {
        crate::storage::resolve_socket_path(root).unwrap_or_else(|_| {
            use std::collections::hash_map::DefaultHasher;
            use std::hash::{Hash, Hasher};
            let mut hasher = DefaultHasher::new();
            root.hash(&mut hasher);
            PathBuf::from(format!("/tmp/cclab_lens-{:x}.sock", hasher.finish()))
        })
    }

    /// Set custom socket path
    pub fn with_socket(mut self, path: PathBuf) -> Self {
        self.socket_path = path;
        self
    }

    /// Enable/disable file watching
    pub fn with_watch(mut self, enabled: bool) -> Self {
        self.watch = enabled;
        self
    }
}

/// Status of the analysis queue
#[derive(Debug, Clone, Default)]
pub struct AnalysisQueueStatus {
    /// Number of files pending analysis
    pub pending_count: usize,
    /// Last time analysis was performed
    pub last_analysis: Option<Instant>,
}

/// Argus Daemon server
pub struct ArgusDaemon {
    config: DaemonConfig,
    handler: Arc<RequestHandler>,
    /// Per-scope handlers for scoped toolchain binding (#1127)
    scope_handlers: Arc<HashMap<String, Arc<RequestHandler>>>,
    /// Scope roots sorted by length descending (for longest prefix match)
    scope_roots: Arc<Vec<(PathBuf, String)>>,
    shutdown_tx: broadcast::Sender<()>,
    is_running: Arc<RwLock<bool>>,
    /// Analysis queue status
    queue_status: Arc<RwLock<AnalysisQueueStatus>>,
}

impl ArgusDaemon {
    /// Create a new daemon
    pub fn new(config: DaemonConfig) -> Result<Self, String> {
        let handler = RequestHandler::new(config.root.clone())?;
        let (shutdown_tx, _) = broadcast::channel(1);

        // Auto-discover scopes (#1127)
        let discovered = super::auto_discover::discover_scopes(&config.root);
        let mut scope_handlers = HashMap::new();
        let mut scope_roots = Vec::new();

        for scope in &discovered {
            let scope_root = config.root.join(&scope.root);
            match RequestHandler::new_with_scope(
                scope_root.clone(),
                &scope.id,
                &config.root,
                &scope.search_paths,
            ) {
                Ok(h) => {
                    eprintln!(
                        "[daemon] Scope '{}' ({}) at {}",
                        scope.id,
                        scope.lang,
                        scope.root.display()
                    );
                    scope_roots.push((scope.root.clone(), scope.id.clone()));
                    scope_handlers.insert(scope.id.clone(), Arc::new(h));
                }
                Err(e) => {
                    eprintln!("[daemon] Failed to init scope '{}': {}", scope.id, e);
                }
            }
        }

        // Sort by root length descending for longest prefix match
        scope_roots.sort_by(|a, b| b.0.as_os_str().len().cmp(&a.0.as_os_str().len()));

        Ok(Self {
            config,
            handler: Arc::new(handler),
            scope_handlers: Arc::new(scope_handlers),
            scope_roots: Arc::new(scope_roots),
            shutdown_tx,
            is_running: Arc::new(RwLock::new(false)),
            queue_status: Arc::new(RwLock::new(AnalysisQueueStatus::default())),
        })
    }

    /// Run the daemon
    pub async fn run(&self) -> Result<(), String> {
        // Remove existing socket if present
        if self.config.socket_path.exists() {
            std::fs::remove_file(&self.config.socket_path)
                .map_err(|e| format!("Failed to remove existing socket: {}", e))?;
        }

        // Create parent directory if needed
        if let Some(parent) = self.config.socket_path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create socket directory: {}", e))?;
        }

        // Bind to socket
        let listener = UnixListener::bind(&self.config.socket_path)
            .map_err(|e| format!("Failed to bind socket: {}", e))?;

        println!("Lens daemon listening on {:?}", self.config.socket_path);

        // Mark as running
        {
            let mut running = self.is_running.write().await;
            *running = true;
        }

        // Start file watcher if enabled
        let (watch_rx, watcher_handle) = if self.config.watch {
            let (rx, handle) = self.start_watcher().await?;
            (Some(rx), Some(handle))
        } else {
            (None, None)
        };

        // Start the background analysis task if watching
        let analysis_task = if let Some(mut watch_rx) = watch_rx {
            let handler = Arc::clone(&self.handler);
            let queue_status = Arc::clone(&self.queue_status);
            let debounce = self.config.debounce;
            let mut shutdown_rx = self.shutdown_tx.subscribe();

            Some(tokio::spawn(async move {
                run_background_analysis(
                    &mut watch_rx,
                    handler,
                    queue_status,
                    debounce,
                    &mut shutdown_rx,
                )
                .await;
            }))
        } else {
            None
        };

        // Accept connections
        let mut shutdown_rx = self.shutdown_tx.subscribe();

        loop {
            tokio::select! {
                result = listener.accept() => {
                    match result {
                        Ok((stream, _addr)) => {
                            let handler = Arc::clone(&self.handler);
                            let mut conn_shutdown_rx = self.shutdown_tx.subscribe();

                            tokio::spawn(async move {
                                if let Err(e) = handle_connection(stream, handler, &mut conn_shutdown_rx).await {
                                    eprintln!("Connection error: {}", e);
                                }
                            });
                        }
                        Err(e) => {
                            eprintln!("Accept error: {}", e);
                        }
                    }
                }
                _ = shutdown_rx.recv() => {
                    println!("Daemon shutting down...");
                    break;
                }
            }
        }

        // Flush disk cache manifest before cleanup
        self.handler.flush_cache().await;

        // Cleanup
        {
            let mut running = self.is_running.write().await;
            *running = false;
        }

        // Stop the watcher
        if let Some(mut handle) = watcher_handle {
            handle.stop();
        }

        // Wait for analysis task to finish
        if let Some(task) = analysis_task {
            let _ = task.await;
        }

        // Remove socket file
        let _ = std::fs::remove_file(&self.config.socket_path);

        Ok(())
    }

    /// Start file watcher using the async bridge
    async fn start_watcher(
        &self,
    ) -> Result<(mpsc::Receiver<BridgeEvent>, WatchBridgeHandle), String> {
        spawn_watch_bridge(self.config.root.clone(), self.config.debounce)
    }

    /// Resolve the handler for a file path (#1127).
    ///
    /// Uses longest prefix match on scope roots. Falls back to the default handler.
    pub fn handler_for_file(&self, file_path: &std::path::Path) -> Arc<RequestHandler> {
        // Try to find a scope whose root is a prefix of the file path
        for (root, scope_id) in self.scope_roots.iter() {
            if file_path.starts_with(root) {
                if let Some(h) = self.scope_handlers.get(scope_id) {
                    return Arc::clone(h);
                }
            }
        }
        // Fallback to default handler
        Arc::clone(&self.handler)
    }

    /// List discovered scopes (#1127).
    pub fn list_scopes(&self) -> Vec<(String, PathBuf)> {
        self.scope_roots
            .iter()
            .map(|(root, id)| (id.clone(), root.clone()))
            .collect()
    }

    /// Request shutdown
    pub fn shutdown(&self) {
        let _ = self.shutdown_tx.send(());
    }

    /// Check if daemon is running
    pub async fn is_running(&self) -> bool {
        *self.is_running.read().await
    }

    /// Get the socket path
    pub fn socket_path(&self) -> &PathBuf {
        &self.config.socket_path
    }

    /// Get the analysis queue status
    pub async fn queue_status(&self) -> AnalysisQueueStatus {
        self.queue_status.read().await.clone()
    }
}

/// Run background analysis loop
///
/// Consumes file watcher events, maintains a debounced queue, and
/// re-analyzes changed files in the background.
async fn run_background_analysis(
    watch_rx: &mut mpsc::Receiver<BridgeEvent>,
    handler: Arc<RequestHandler>,
    queue_status: Arc<RwLock<AnalysisQueueStatus>>,
    debounce: Duration,
    shutdown_rx: &mut broadcast::Receiver<()>,
) {
    // Pending files to analyze (debounce queue)
    let mut pending_files: HashSet<PathBuf> = HashSet::new();
    let mut last_change: Option<Instant> = None;

    // Analysis interval for debouncing
    let mut debounce_interval = tokio::time::interval(Duration::from_millis(50));

    loop {
        tokio::select! {
            // Receive watch events
            event = watch_rx.recv() => {
                match event {
                    Some(BridgeEvent::FilesChanged(paths)) => {
                        // Add files to pending queue
                        for path in paths {
                            pending_files.insert(path);
                        }
                        last_change = Some(Instant::now());

                        // Update queue status
                        {
                            let mut status = queue_status.write().await;
                            status.pending_count = pending_files.len();
                        }
                    }
                    Some(BridgeEvent::Error(e)) => {
                        eprintln!("Watch error: {}", e);
                    }
                    Some(BridgeEvent::Ready) => {
                        println!("File watcher ready");
                    }
                    Some(BridgeEvent::Stopped) | None => {
                        println!("File watcher stopped");
                        break;
                    }
                }
            }

            // Check debounce interval
            _ = debounce_interval.tick() => {
                // Check if we should flush the queue
                if let Some(last) = last_change {
                    if last.elapsed() >= debounce && !pending_files.is_empty() {
                        // Take the files and analyze them
                        let files: Vec<PathBuf> = pending_files.drain().collect();
                        last_change = None;

                        // Update queue status
                        {
                            let mut status = queue_status.write().await;
                            status.pending_count = 0;
                        }

                        // Analyze files in a blocking task
                        let handler_clone = Arc::clone(&handler);
                        let queue_status_clone = Arc::clone(&queue_status);

                        // Spawn blocking analysis
                        tokio::task::spawn_blocking(move || {
                            for path in &files {
                                // Invalidate cache for the file
                                let rt = tokio::runtime::Handle::current();
                                rt.block_on(async {
                                    handler_clone.invalidate_file(path).await;
                                });
                            }

                            // Re-analyze files by triggering a check
                            // The handler will re-analyze on next access
                            let rt = tokio::runtime::Handle::current();
                            rt.block_on(async {
                                for path in &files {
                                    // Pre-warm the cache by analyzing the file
                                    if let Some(path_str) = path.to_str() {
                                        let _ = handler_clone.analyze_file_async(path_str).await;
                                    }
                                }

                                // Update last analysis time
                                let mut status = queue_status_clone.write().await;
                                status.last_analysis = Some(Instant::now());
                            });
                        });
                    }
                }
            }

            // Handle shutdown
            _ = shutdown_rx.recv() => {
                break;
            }
        }
    }
}

/// Handle a single client connection
async fn handle_connection(
    stream: UnixStream,
    handler: Arc<RequestHandler>,
    shutdown_rx: &mut broadcast::Receiver<()>,
) -> Result<(), String> {
    let (reader, mut writer) = stream.into_split();
    let mut reader = BufReader::new(reader);
    let mut line = String::new();

    loop {
        line.clear();

        tokio::select! {
            result = reader.read_line(&mut line) => {
                match result {
                    Ok(0) => {
                        // Connection closed
                        break;
                    }
                    Ok(_) => {
                        let response = process_request(&line, &handler).await;
                        let response_json = serde_json::to_string(&response)
                            .map_err(|e| format!("Failed to serialize response: {}", e))?;

                        writer.write_all(response_json.as_bytes()).await
                            .map_err(|e| format!("Failed to write response: {}", e))?;
                        writer.write_all(b"\n").await
                            .map_err(|e| format!("Failed to write newline: {}", e))?;
                        writer.flush().await
                            .map_err(|e| format!("Failed to flush: {}", e))?;

                        // Check for shutdown request
                        if line.contains("\"method\":\"shutdown\"") {
                            break;
                        }
                    }
                    Err(e) => {
                        return Err(format!("Read error: {}", e));
                    }
                }
            }
            _ = shutdown_rx.recv() => {
                break;
            }
        }
    }

    Ok(())
}

/// Process a single request
async fn process_request(line: &str, handler: &RequestHandler) -> Response {
    // Parse request
    let request: Request = match serde_json::from_str(line.trim()) {
        Ok(r) => r,
        Err(e) => {
            return Response::error(
                super::protocol::RequestId::Number(0),
                RpcError::parse_error(format!("Invalid JSON: {}", e)),
            );
        }
    };

    // Handle request
    handler.handle(request).await
}

/// Client for connecting to daemon
pub struct DaemonClient {
    socket_path: PathBuf,
}

impl DaemonClient {
    /// Create a client for the given socket
    pub fn new(socket_path: PathBuf) -> Self {
        Self { socket_path }
    }

    /// Create a client for the default socket of a workspace
    pub fn for_workspace(root: &PathBuf) -> Self {
        let socket_path = DaemonConfig::default_socket_path(root);
        Self::new(socket_path)
    }

    /// Check if daemon is running
    pub async fn is_daemon_running(&self) -> bool {
        UnixStream::connect(&self.socket_path).await.is_ok()
    }

    /// Send a request and get response
    pub async fn request(
        &self,
        method: &str,
        params: Option<serde_json::Value>,
    ) -> Result<serde_json::Value, String> {
        let stream = UnixStream::connect(&self.socket_path)
            .await
            .map_err(|e| format!("Failed to connect to daemon: {}", e))?;

        let (reader, mut writer) = stream.into_split();

        // Build request
        let request = Request::new(1i64, method, params);
        let request_json = serde_json::to_string(&request)
            .map_err(|e| format!("Failed to serialize request: {}", e))?;

        // Send request
        writer
            .write_all(request_json.as_bytes())
            .await
            .map_err(|e| format!("Failed to send request: {}", e))?;
        writer
            .write_all(b"\n")
            .await
            .map_err(|e| format!("Failed to send newline: {}", e))?;
        writer
            .flush()
            .await
            .map_err(|e| format!("Failed to flush: {}", e))?;

        // Read response
        let mut reader = BufReader::new(reader);
        let mut response_line = String::new();
        reader
            .read_line(&mut response_line)
            .await
            .map_err(|e| format!("Failed to read response: {}", e))?;

        // Parse response
        let response: Response = serde_json::from_str(response_line.trim())
            .map_err(|e| format!("Failed to parse response: {}", e))?;

        if let Some(error) = response.error {
            Err(format!("RPC error {}: {}", error.code, error.message))
        } else {
            Ok(response.result.unwrap_or(serde_json::Value::Null))
        }
    }

    /// Convenience methods
    pub async fn check(&self, path: &str) -> Result<serde_json::Value, String> {
        self.request("check", Some(serde_json::json!({ "path": path })))
            .await
    }

    pub async fn type_at(
        &self,
        file: &str,
        line: u32,
        column: u32,
    ) -> Result<serde_json::Value, String> {
        self.request(
            "type_at",
            Some(serde_json::json!({
                "file": file,
                "line": line,
                "column": column
            })),
        )
        .await
    }

    pub async fn symbols(&self, file: &str) -> Result<serde_json::Value, String> {
        self.request("symbols", Some(serde_json::json!({ "file": file })))
            .await
    }

    pub async fn diagnostics(&self, file: Option<&str>) -> Result<serde_json::Value, String> {
        let params = file.map(|f| serde_json::json!({ "file": f }));
        self.request("diagnostics", params).await
    }

    pub async fn index_status(&self) -> Result<serde_json::Value, String> {
        self.request("index_status", None).await
    }

    pub async fn shutdown(&self) -> Result<(), String> {
        self.request("shutdown", None).await?;
        Ok(())
    }

    /// Invalidate cache for specific files
    ///
    /// This removes the cached analysis for the specified files, forcing
    /// them to be re-analyzed on the next request.
    pub async fn invalidate(&self, files: &[&str]) -> Result<serde_json::Value, String> {
        let file_list: Vec<String> = files.iter().map(|s| s.to_string()).collect();
        self.request(
            "invalidate",
            Some(serde_json::json!({ "files": file_list })),
        )
        .await
    }

    /// Request hover information at a position
    pub async fn hover(
        &self,
        file: &str,
        line: u32,
        column: u32,
    ) -> Result<serde_json::Value, String> {
        self.request(
            "hover",
            Some(serde_json::json!({
                "file": file,
                "line": line,
                "column": column
            })),
        )
        .await
    }

    /// Request definition location at a position
    pub async fn definition(
        &self,
        file: &str,
        line: u32,
        column: u32,
    ) -> Result<serde_json::Value, String> {
        self.request(
            "definition",
            Some(serde_json::json!({
                "file": file,
                "line": line,
                "column": column
            })),
        )
        .await
    }

    /// Request all references at a position
    pub async fn references(
        &self,
        file: &str,
        line: u32,
        column: u32,
        include_declaration: bool,
    ) -> Result<serde_json::Value, String> {
        self.request(
            "references",
            Some(serde_json::json!({
                "file": file,
                "line": line,
                "column": column,
                "include_declaration": include_declaration
            })),
        )
        .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_socket_path_generation() {
        let root1 = PathBuf::from("/home/user/project1");
        let root2 = PathBuf::from("/home/user/project2");

        let path1 = DaemonConfig::default_socket_path(&root1);
        let path2 = DaemonConfig::default_socket_path(&root2);

        // Different roots should produce different socket paths
        assert_ne!(path1, path2);

        // Same root should produce same path
        let path1_again = DaemonConfig::default_socket_path(&root1);
        assert_eq!(path1, path1_again);
    }

    #[test]
    fn test_daemon_config() {
        let root = PathBuf::from("/test/project");
        let config = DaemonConfig::new(root.clone());

        assert_eq!(config.root, root);
        assert!(config.watch);
        assert_eq!(config.debounce, Duration::from_millis(300));
    }
}
````

## Changes
<!-- type: changes lang: yaml -->

```yaml
coverage_kind: semantic
changes:
  - path: "libs/compass/src/server/daemon.rs"
    action: modify
    section: rust-source-unit
    impl_mode: codegen
    description: |
      rust-source-unit (td_ast) source for `libs/compass/src/server/daemon.rs` captured during libs codegen standardization.
```
