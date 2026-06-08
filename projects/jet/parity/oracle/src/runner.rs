// SPEC-MANAGED: .aw/tech-design/projects/jet/semantic/jet-parity-oracle-src.md#schema
// CODEGEN-BEGIN

//! Runner — drives the §Logic state machine.
//!
//! The runner owns a [`BrowserSession`] trait object so unit tests can
//! substitute a [`StubBrowserSession`] without requiring a live Chromium.
//! The production implementation (`JetBrowserSession`) launches Chromium
//! directly and talks Chrome DevTools Protocol (CDP), matching the Jet Browser
//! architecture without depending on the `jet` crate and creating a cycle.

use crate::artifacts::{ArtifactBundle, ArtifactWriter};
use crate::channels::{
    a11y::A11yChannel, focus::FocusChannel, ime::ImeChannel, pixel::PixelChannel,
    pointer::PointerChannel, Channel, ChannelArtifact, ChannelCtx, ChannelError, DeterministicPrng,
};
use crate::manifest::{FixtureManifest, ManifestError};
use async_trait::async_trait;
use base64::Engine as _;
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Duration;
use thiserror::Error;
use tokio::process::{Child, Command};
use tokio::sync::{mpsc, oneshot, Mutex};
use tokio_tungstenite::tungstenite::Message;

/// @spec parity-dom-reference-runner.md#Dependency (RunnerConfig)
#[derive(Debug, Clone)]
pub struct RunnerConfig {
    pub artifact_root: PathBuf,
    pub shell_html: PathBuf,
    pub per_fixture_budget: Duration,
    pub viewport: (u32, u32),
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-parity-oracle-src.md#schema
impl Default for RunnerConfig {
    fn default() -> Self {
        Self {
            artifact_root: PathBuf::from("artifacts"),
            shell_html: PathBuf::from("fixtures/__shell__/index.html"),
            per_fixture_budget: Duration::from_secs(8),
            viewport: (1024, 768),
        }
    }
}

/// @spec parity-dom-reference-runner.md#Dependency (BrowserSession — browser kind)
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum BrowserKind {
    Chromium,
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-parity-oracle-src.md#schema
impl BrowserKind {
    pub fn as_str(self) -> &'static str {
        match self {
            BrowserKind::Chromium => "chromium",
        }
    }
}

/// @spec parity-dom-reference-runner.md#Logic (matrix entry; R4)
#[derive(Debug, Clone, Copy)]
pub struct MatrixEntry {
    pub browser: BrowserKind,
    pub dpr: f32,
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-parity-oracle-src.md#schema
impl MatrixEntry {
    pub fn dpr_label(&self) -> String {
        // Stable label like "1.0" / "2.0" for the artifact path.
        format!("{:.1}", self.dpr)
    }
}

/// @spec parity-dom-reference-runner.md#Changes (runner.rs)
#[derive(Debug, Error)]
pub enum RunnerError {
    #[error("manifest error: {0}")]
    Manifest(#[from] ManifestError),
    #[error("artifact error: {0}")]
    Artifact(#[from] crate::artifacts::ArtifactError),
    #[error("channel error in {channel}: {source}")]
    Channel {
        channel: &'static str,
        #[source]
        source: ChannelError,
    },
    #[error("browser launch failed: {0}")]
    LaunchFail(String),
    #[error("browser protocol error: {0}")]
    Browser(String),
    #[error("mount sentinel timeout after {0:?}")]
    MountTimeout(Duration),
    #[error("per-fixture wall-clock budget exceeded ({0:?})")]
    BudgetExceeded(Duration),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
}

/// @spec parity-dom-reference-runner.md#Dependency (PageHost)
///
/// The minimal page-host surface the channels need. In production this is
/// backed by a CDP page session; in tests it is stub data.
#[derive(Debug, Default, Clone)]
pub struct PageHost {
    pub url: String,
    pub viewport: (u32, u32),
    pub mounted: bool,
}

/// @spec parity-dom-reference-runner.md#Dependency (BrowserSession trait)
///
/// Abstraction over the Chromium-driving harness. The production impl
/// (`JetBrowserSession`) uses CDP. The stub impl ([`StubBrowserSession`])
/// returns canned data so tests can exercise channel orchestration without a
/// live browser.
#[async_trait]
pub trait BrowserSession: Send + Sync {
    fn browser_kind(&self) -> BrowserKind;
    fn page(&self) -> &PageHost;
    fn page_mut(&mut self) -> &mut PageHost;

    async fn launch(&mut self, dpr: f32, viewport: (u32, u32)) -> Result<(), RunnerError>;
    async fn navigate(&mut self, url: &str) -> Result<(), RunnerError>;
    async fn await_mount(&mut self, budget: Duration) -> Result<(), RunnerError>;
    async fn close(&mut self) -> Result<(), RunnerError>;

    /// @spec parity-dom-reference-runner.md#Dependency (PixelChannel -> ArtifactWriter)
    ///
    /// Take a viewport screenshot. Production: `page.screenshot()`.
    async fn screenshot(&mut self) -> Result<Vec<u8>, ChannelError>;

    /// @spec parity-dom-reference-runner.md#Dependency (A11yChannel invokes CDP)
    ///
    /// CDP `Accessibility.getFullAXTree` — verbatim response (R5).
    async fn ax_full_tree(&mut self) -> Result<serde_json::Value, ChannelError>;

    /// @spec parity-dom-reference-runner.md#Dependency (FocusChannel invokes CDP)
    ///
    /// CDP `Input.dispatchKeyEvent` for Tab × N. Returns the focus trace
    /// in the order produced by the channel (R6).
    async fn capture_focus_trace(
        &mut self,
        tab_count: u32,
    ) -> Result<Vec<crate::channels::FocusEntry>, ChannelError>;

    /// @spec parity-dom-reference-runner.md#Dependency (PointerChannel invokes PageHost)
    ///
    /// In-page `elementFromPoint` + `getComputedStyle.cursor` over the
    /// caller-supplied seeded coordinate list (R7).
    async fn capture_pointer_hits(
        &mut self,
        coords: &[(u32, u32)],
    ) -> Result<Vec<crate::channels::PointerHit>, ChannelError>;

    /// @spec parity-dom-reference-runner.md#Dependency (ImeChannel invokes CDP)
    ///
    /// CDP `Input.imeSetComposition` + `Input.insertText` for the fixed CJK
    /// script (R8). Returns the captured event list.
    async fn capture_ime_trace(&mut self) -> Result<Vec<serde_json::Value>, ChannelError>;
}

#[derive(Serialize)]
struct CdpRequest {
    id: u64,
    method: String,
    params: Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "sessionId")]
    session_id: Option<String>,
}

#[derive(Deserialize)]
struct CdpResponse {
    id: Option<u64>,
    result: Option<Value>,
    error: Option<CdpError>,
    method: Option<String>,
    params: Option<Value>,
}

#[derive(Debug, Deserialize)]
struct CdpError {
    code: i64,
    message: String,
}

enum OutgoingMessage {
    Send(String),
}

#[derive(Clone)]
struct OracleCdpSession {
    sender: mpsc::Sender<OutgoingMessage>,
    session_id: Option<String>,
    next_id: Arc<AtomicU64>,
    pending: Arc<Mutex<HashMap<u64, oneshot::Sender<anyhow::Result<Value>>>>>,
}

struct OracleCdpClient {
    root: OracleCdpSession,
    _reader_handle: tokio::task::JoinHandle<()>,
    _writer_handle: tokio::task::JoinHandle<()>,
}

/// @spec parity-dom-reference-runner.md#Dependency (BrowserSession)
impl OracleCdpClient {
    async fn connect(ws_url: &str) -> anyhow::Result<Self> {
        let (ws_stream, _) = tokio_tungstenite::connect_async(ws_url).await?;
        let (mut ws_sink, mut ws_reader) = ws_stream.split();
        let (outgoing_tx, mut outgoing_rx) = mpsc::channel::<OutgoingMessage>(64);
        let next_id = Arc::new(AtomicU64::new(1));
        let pending: Arc<Mutex<HashMap<u64, oneshot::Sender<anyhow::Result<Value>>>>> =
            Arc::new(Mutex::new(HashMap::new()));

        let writer_handle = tokio::spawn(async move {
            while let Some(OutgoingMessage::Send(text)) = outgoing_rx.recv().await {
                if ws_sink.send(Message::Text(text.into())).await.is_err() {
                    break;
                }
            }
        });

        let pending_for_reader = pending.clone();
        let reader_handle = tokio::spawn(async move {
            while let Some(Ok(msg)) = ws_reader.next().await {
                let Message::Text(text) = msg else {
                    continue;
                };
                let Ok(resp) = serde_json::from_str::<CdpResponse>(&text) else {
                    continue;
                };
                if let Some(id) = resp.id {
                    let result = if let Some(err) = resp.error {
                        Err(anyhow::anyhow!("CDP error {}: {}", err.code, err.message))
                    } else {
                        Ok(resp.result.unwrap_or(Value::Null))
                    };
                    if let Some(tx) = pending_for_reader.lock().await.remove(&id) {
                        let _ = tx.send(result);
                    }
                } else {
                    let _ = (&resp.method, &resp.params);
                }
            }
        });

        Ok(Self {
            root: OracleCdpSession {
                sender: outgoing_tx,
                session_id: None,
                next_id,
                pending,
            },
            _reader_handle: reader_handle,
            _writer_handle: writer_handle,
        })
    }

    fn root_session(&self) -> OracleCdpSession {
        self.root.clone()
    }

    async fn create_page_session(&self, url: &str) -> anyhow::Result<OracleCdpSession> {
        let target = self
            .root
            .send("Target.createTarget", serde_json::json!({ "url": url }))
            .await?;
        let target_id = target["targetId"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("missing targetId in Target.createTarget"))?;
        let attached = self
            .root
            .send(
                "Target.attachToTarget",
                serde_json::json!({
                    "targetId": target_id,
                    "flatten": true
                }),
            )
            .await?;
        let session_id = attached["sessionId"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("missing sessionId in Target.attachToTarget"))?;
        Ok(self.root.child_session(session_id.to_string()))
    }
}

/// @spec parity-dom-reference-runner.md#Dependency (BrowserSession)
impl OracleCdpSession {
    async fn send(&self, method: &str, params: Value) -> anyhow::Result<Value> {
        let id = self.next_id.fetch_add(1, Ordering::SeqCst);
        let request = CdpRequest {
            id,
            method: method.to_string(),
            params,
            session_id: self.session_id.clone(),
        };
        let (tx, rx) = oneshot::channel();
        self.pending.lock().await.insert(id, tx);
        let text = serde_json::to_string(&request)?;
        self.sender
            .send(OutgoingMessage::Send(text))
            .await
            .map_err(|_| anyhow::anyhow!("CDP outgoing channel closed while sending {method}"))?;
        rx.await
            .map_err(|_| anyhow::anyhow!("CDP response channel closed for {method}"))?
    }

    fn child_session(&self, session_id: String) -> Self {
        Self {
            sender: self.sender.clone(),
            session_id: Some(session_id),
            next_id: self.next_id.clone(),
            pending: self.pending.clone(),
        }
    }
}

/// @spec parity-dom-reference-runner.md#Dependency (BrowserSession)
///
/// Production implementation backed by Chromium CDP. It intentionally keeps a
/// small local CDP client instead of importing `jet::browser`, because the Jet
/// crate depends on this oracle crate for re-export and tests.
pub struct JetBrowserSession {
    page: PageHost,
    process: Option<Child>,
    user_data_dir: Option<tempfile::TempDir>,
    client: Option<OracleCdpClient>,
    page_session: Option<OracleCdpSession>,
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-parity-oracle-src.md#schema
impl JetBrowserSession {
    pub fn new() -> Self {
        Self {
            page: PageHost::default(),
            process: None,
            user_data_dir: None,
            client: None,
            page_session: None,
        }
    }

    fn page_session(&self) -> Result<&OracleCdpSession, ChannelError> {
        self.page_session
            .as_ref()
            .ok_or(ChannelError::Cdp("CDP page session is not attached".into()))
    }

    fn page_session_for_runner(&self) -> Result<&OracleCdpSession, RunnerError> {
        self.page_session
            .as_ref()
            .ok_or_else(|| RunnerError::Browser("CDP page session is not attached".into()))
    }
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-parity-oracle-src.md#schema
impl Default for JetBrowserSession {
    fn default() -> Self {
        Self::new()
    }
}

/// Backward-compatible type name for callers that referenced the old
/// Playwright placeholder directly.
pub type PlaywrightBrowserSession = JetBrowserSession;

/// @spec .aw/tech-design/projects/jet/semantic/jet-parity-oracle-src.md#schema
#[async_trait]
impl BrowserSession for JetBrowserSession {
    fn browser_kind(&self) -> BrowserKind {
        BrowserKind::Chromium
    }
    fn page(&self) -> &PageHost {
        &self.page
    }
    fn page_mut(&mut self) -> &mut PageHost {
        &mut self.page
    }

    async fn launch(&mut self, dpr: f32, viewport: (u32, u32)) -> Result<(), RunnerError> {
        let executable = find_chromium_executable().map_err(|err| RunnerError::LaunchFail(err))?;
        let port = find_free_port().map_err(|err| RunnerError::LaunchFail(err.to_string()))?;
        let user_data_dir =
            tempfile::tempdir().map_err(|err| RunnerError::LaunchFail(err.to_string()))?;
        let mut cmd = Command::new(executable);
        cmd.arg(format!("--remote-debugging-port={port}"))
            .arg(format!(
                "--user-data-dir={}",
                user_data_dir.path().display()
            ))
            .arg("--no-first-run")
            .arg("--no-default-browser-check")
            .arg("--disable-background-networking")
            .arg("--disable-default-apps")
            .arg("--disable-extensions")
            .arg("--disable-sync")
            .arg("--disable-translate")
            .arg("--metrics-recording-only")
            .arg("--mute-audio")
            .arg("--no-sandbox")
            .arg("--headless=new")
            .arg(format!("--window-size={},{}", viewport.0, viewport.1))
            .arg(format!("--force-device-scale-factor={dpr}"))
            .stdout(Stdio::null())
            .stderr(Stdio::piped());

        let child = cmd
            .spawn()
            .map_err(|err| RunnerError::LaunchFail(err.to_string()))?;
        let ws_url = wait_for_ws_endpoint(port)
            .await
            .map_err(|err| RunnerError::LaunchFail(err.to_string()))?;
        let client = OracleCdpClient::connect(&ws_url)
            .await
            .map_err(|err| RunnerError::LaunchFail(err.to_string()))?;
        let page_session = client
            .create_page_session("about:blank")
            .await
            .map_err(|err| RunnerError::LaunchFail(err.to_string()))?;
        page_session
            .send("Page.enable", serde_json::json!({}))
            .await
            .map_err(|err| RunnerError::Browser(err.to_string()))?;
        page_session
            .send("Runtime.enable", serde_json::json!({}))
            .await
            .map_err(|err| RunnerError::Browser(err.to_string()))?;
        page_session
            .send("Accessibility.enable", serde_json::json!({}))
            .await
            .map_err(|err| RunnerError::Browser(err.to_string()))?;

        self.page.viewport = viewport;
        self.process = Some(child);
        self.user_data_dir = Some(user_data_dir);
        self.page_session = Some(page_session);
        self.client = Some(client);
        Ok(())
    }

    async fn navigate(&mut self, url: &str) -> Result<(), RunnerError> {
        let session = self.page_session_for_runner()?;
        session
            .send("Page.navigate", serde_json::json!({ "url": url }))
            .await
            .map_err(|err| RunnerError::Browser(err.to_string()))?;
        wait_for_ready_state(session, Duration::from_secs(10)).await?;
        self.page.url = url.to_string();
        Ok(())
    }

    async fn await_mount(&mut self, budget: Duration) -> Result<(), RunnerError> {
        let session = self.page_session_for_runner()?;
        let start = std::time::Instant::now();
        loop {
            let mounted = evaluate_json(
                session,
                "Boolean(window.__jet_oracle_mounted)",
                "mount sentinel",
            )
            .await
            .map_err(|err| RunnerError::Browser(err.to_string()))?;
            if mounted.as_bool().unwrap_or(false) {
                self.page.mounted = true;
                return Ok(());
            }
            if start.elapsed() >= budget {
                return Err(RunnerError::MountTimeout(budget));
            }
            tokio::time::sleep(Duration::from_millis(50)).await;
        }
    }

    async fn close(&mut self) -> Result<(), RunnerError> {
        if let Some(client) = &self.client {
            let _ = client
                .root_session()
                .send("Browser.close", serde_json::json!({}))
                .await;
        }
        if let Some(mut child) = self.process.take() {
            let _ = child.kill().await;
        }
        self.page_session.take();
        self.client.take();
        self.user_data_dir.take();
        Ok(())
    }

    async fn screenshot(&mut self) -> Result<Vec<u8>, ChannelError> {
        let session = self.page_session()?;
        let _ = session
            .send("Page.bringToFront", serde_json::json!({}))
            .await;
        let result = session
            .send(
                "Page.captureScreenshot",
                serde_json::json!({ "format": "png" }),
            )
            .await
            .map_err(|err| ChannelError::Cdp(err.to_string()))?;
        let data = result["data"]
            .as_str()
            .ok_or_else(|| ChannelError::Cdp("missing Page.captureScreenshot data".into()))?;
        base64::engine::general_purpose::STANDARD
            .decode(data)
            .map_err(|err| ChannelError::Cdp(err.to_string()))
    }

    async fn ax_full_tree(&mut self) -> Result<serde_json::Value, ChannelError> {
        self.page_session()?
            .send("Accessibility.getFullAXTree", serde_json::json!({}))
            .await
            .map_err(|err| ChannelError::Cdp(err.to_string()))
    }

    async fn capture_focus_trace(
        &mut self,
        tab_count: u32,
    ) -> Result<Vec<crate::channels::FocusEntry>, ChannelError> {
        let session = self.page_session()?;
        let mut trace = Vec::with_capacity(tab_count as usize);
        for step in 0..tab_count {
            dispatch_key(session, "keyDown", "Tab").await?;
            dispatch_key(session, "keyUp", "Tab").await?;
            let expression = focus_snapshot_js(step);
            let value = evaluate_json(session, &expression, "focus trace").await?;
            let entry: crate::channels::FocusEntry =
                serde_json::from_value(value).map_err(ChannelError::Json)?;
            trace.push(entry);
        }
        Ok(trace)
    }

    async fn capture_pointer_hits(
        &mut self,
        coords: &[(u32, u32)],
    ) -> Result<Vec<crate::channels::PointerHit>, ChannelError> {
        let coords_json = serde_json::to_string(coords).map_err(ChannelError::Json)?;
        let expression = format!(
            r##"(() => {{
  const coords = {coords_json};
  const cssEscape = window.CSS && CSS.escape ? CSS.escape.bind(CSS) : (v) => String(v).replace(/"/g, "\\\"");
  const selectorFor = (el) => {{
    if (!el) return "";
    if (el.id) return "#" + cssEscape(el.id);
    const fixture = el.getAttribute && el.getAttribute("data-jet-fixture");
    if (fixture) return `[data-jet-fixture="${{cssEscape(fixture)}}"]`;
    const role = el.getAttribute && el.getAttribute("role");
    if (role) return `${{el.tagName.toLowerCase()}}[role="${{cssEscape(role)}}"]`;
    return el.tagName ? el.tagName.toLowerCase() : "";
  }};
  return coords.map(([x, y]) => {{
    const el = document.elementFromPoint(x, y);
    const style = el ? getComputedStyle(el) : null;
    return {{
      x,
      y,
      target_selector: selectorFor(el),
      computed_cursor: style ? style.cursor : ""
    }};
  }});
}})()"##
        );
        let value = evaluate_json(self.page_session()?, &expression, "pointer hit map").await?;
        serde_json::from_value(value).map_err(ChannelError::Json)
    }

    async fn capture_ime_trace(&mut self) -> Result<Vec<serde_json::Value>, ChannelError> {
        let session = self.page_session()?;
        evaluate_json(
            session,
            r#"(() => {
  window.__jet_oracle_ime_events = [];
  let el = document.querySelector('input, textarea, [contenteditable="true"]');
  if (!el) {
    el = document.createElement('textarea');
    el.setAttribute('data-jet-oracle-ime-probe', 'true');
    document.body.appendChild(el);
  }
  for (const type of ['compositionstart', 'compositionupdate', 'compositionend', 'beforeinput', 'input']) {
    el.addEventListener(type, (event) => {
      window.__jet_oracle_ime_events.push({
        type,
        data: event.data || '',
        inputType: event.inputType || ''
      });
    });
  }
  el.focus();
  return true;
})()"#,
            "ime setup",
        )
        .await?;
        session
            .send(
                "Input.imeSetComposition",
                serde_json::json!({
                    "text": "ni",
                    "selectionStart": 2,
                    "selectionEnd": 2,
                    "replacementStart": 0,
                    "replacementEnd": 0
                }),
            )
            .await
            .map_err(|err| ChannelError::Cdp(err.to_string()))?;
        session
            .send("Input.insertText", serde_json::json!({ "text": "你" }))
            .await
            .map_err(|err| ChannelError::Cdp(err.to_string()))?;
        let value =
            evaluate_json(session, "window.__jet_oracle_ime_events || []", "ime trace").await?;
        serde_json::from_value(value).map_err(ChannelError::Json)
    }
}

fn find_chromium_executable() -> Result<PathBuf, String> {
    if let Ok(path) = std::env::var("JET_PARITY_ORACLE_CHROME") {
        let path = PathBuf::from(path);
        if is_executable(&path) {
            return Ok(path);
        }
        return Err(format!(
            "JET_PARITY_ORACLE_CHROME points to a non-executable path: {}",
            path.display()
        ));
    }

    if let Some(home) = dirs::home_dir() {
        let cache = home.join(".jet").join("browsers");
        if let Some(path) = find_chromium_in_cache(&cache) {
            return Ok(path);
        }
    }

    for candidate in chromium_system_candidates() {
        let path = PathBuf::from(candidate);
        if is_executable(&path) {
            return Ok(path);
        }
    }
    Err("Chrome/Chromium not found for jet-parity-oracle live harness".into())
}

fn chromium_system_candidates() -> Vec<&'static str> {
    if cfg!(target_os = "macos") {
        vec![
            "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome",
            "/Applications/Chromium.app/Contents/MacOS/Chromium",
            "/Applications/Google Chrome Canary.app/Contents/MacOS/Google Chrome Canary",
        ]
    } else if cfg!(target_os = "linux") {
        vec![
            "/usr/bin/google-chrome",
            "/usr/bin/google-chrome-stable",
            "/usr/bin/chromium-browser",
            "/usr/bin/chromium",
        ]
    } else {
        vec![
            r"C:\Program Files\Google\Chrome\Application\chrome.exe",
            r"C:\Program Files (x86)\Google\Chrome\Application\chrome.exe",
        ]
    }
}

fn find_chromium_in_cache(cache_root: &Path) -> Option<PathBuf> {
    let binary_subpath = if cfg!(target_os = "macos") {
        "chrome-mac/Chromium.app/Contents/MacOS/Chromium"
    } else if cfg!(target_os = "linux") {
        "chrome-linux/chrome"
    } else {
        "chrome-win/chrome.exe"
    };
    let mut entries = std::fs::read_dir(cache_root)
        .ok()?
        .filter_map(Result::ok)
        .filter_map(|entry| {
            let name = entry.file_name();
            let name = name.to_string_lossy();
            let suffix = name.strip_prefix("chromium-")?;
            let rev = suffix.parse::<u64>().ok()?;
            Some((rev, entry.path()))
        })
        .collect::<Vec<_>>();
    entries.sort_by(|a, b| b.0.cmp(&a.0));
    entries
        .into_iter()
        .map(|(_, dir)| dir.join(binary_subpath))
        .find(|path| is_executable(path))
}

fn is_executable(path: &Path) -> bool {
    if !path.exists() || !path.is_file() {
        return false;
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::metadata(path)
            .map(|meta| meta.permissions().mode() & 0o111 != 0)
            .unwrap_or(false)
    }
    #[cfg(not(unix))]
    {
        true
    }
}

fn find_free_port() -> std::io::Result<u16> {
    let listener = std::net::TcpListener::bind("127.0.0.1:0")?;
    Ok(listener.local_addr()?.port())
}

async fn wait_for_ws_endpoint(port: u16) -> anyhow::Result<String> {
    let client = reqwest::Client::new();
    let url = format!("http://127.0.0.1:{port}/json/version");
    let mut last_error = None;
    for _ in 0..100 {
        tokio::time::sleep(Duration::from_millis(50)).await;
        match client.get(&url).send().await {
            Ok(resp) => match resp.json::<Value>().await {
                Ok(json) => {
                    if let Some(ws) = json["webSocketDebuggerUrl"].as_str() {
                        return Ok(ws.to_string());
                    }
                    last_error = Some("missing webSocketDebuggerUrl".to_string());
                }
                Err(err) => last_error = Some(format!("json parse: {err}")),
            },
            Err(err) => last_error = Some(format!("connect: {err}")),
        }
    }
    anyhow::bail!(
        "timed out waiting for Chromium CDP endpoint on {url}; last error: {}",
        last_error.unwrap_or_else(|| "none".into())
    )
}

async fn wait_for_ready_state(
    session: &OracleCdpSession,
    timeout: Duration,
) -> Result<(), RunnerError> {
    let start = std::time::Instant::now();
    loop {
        let state = evaluate_json(session, "document.readyState", "document.readyState")
            .await
            .map_err(|err| RunnerError::Browser(err.to_string()))?;
        if state.as_str() == Some("complete") {
            return Ok(());
        }
        if start.elapsed() >= timeout {
            return Err(RunnerError::Browser(format!(
                "timed out waiting for document.readyState=complete; last={state}"
            )));
        }
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
}

async fn evaluate_json(
    session: &OracleCdpSession,
    expression: &str,
    context: &str,
) -> Result<Value, ChannelError> {
    let result = session
        .send(
            "Runtime.evaluate",
            serde_json::json!({
                "expression": expression,
                "returnByValue": true,
                "awaitPromise": true
            }),
        )
        .await
        .map_err(|err| ChannelError::Cdp(format!("{context}: {err}")))?;
    if let Some(exception) = result.get("exceptionDetails") {
        return Err(ChannelError::Cdp(format!(
            "{context}: JS exception: {exception}"
        )));
    }
    Ok(result["result"]["value"].clone())
}

async fn dispatch_key(
    session: &OracleCdpSession,
    event_type: &str,
    key: &str,
) -> Result<(), ChannelError> {
    session
        .send(
            "Input.dispatchKeyEvent",
            serde_json::json!({
                "type": event_type,
                "key": key,
                "code": key,
                "windowsVirtualKeyCode": 9,
                "nativeVirtualKeyCode": 9
            }),
        )
        .await
        .map(|_| ())
        .map_err(|err| ChannelError::Cdp(err.to_string()))
}

fn focus_snapshot_js(step: u32) -> String {
    format!(
        r##"(() => {{
  const el = document.activeElement || document.body;
  const rect = el.getBoundingClientRect();
  const cssEscape = window.CSS && CSS.escape ? CSS.escape.bind(CSS) : (v) => String(v).replace(/"/g, "\\\"");
  const selector = (() => {{
    if (el.id) return "#" + cssEscape(el.id);
    const fixture = el.getAttribute && el.getAttribute("data-jet-fixture");
    if (fixture) return `[data-jet-fixture="${{cssEscape(fixture)}}"]`;
    const name = el.getAttribute && el.getAttribute("name");
    if (name) return `${{el.tagName.toLowerCase()}}[name="${{cssEscape(name)}}"]`;
    return el.tagName ? el.tagName.toLowerCase() : "";
  }})();
  return {{
    step: {step},
    selector,
    role: (el.getAttribute && el.getAttribute("role")) || (el.tagName ? el.tagName.toLowerCase() : ""),
    name: ((el.getAttribute && (el.getAttribute("aria-label") || el.getAttribute("title"))) || el.textContent || "").trim(),
    bounds: [rect.x, rect.y, rect.width, rect.height]
  }};
}})()"##
    )
}

/// @spec parity-dom-reference-runner.md#Dependency (BrowserSession — test substitution)
///
/// A canned-data implementation used by unit tests so they can exercise
/// the channel orchestration without a live Chromium.
#[derive(Debug, Clone)]
pub struct StubBrowserSession {
    page: PageHost,
    /// If false, `await_mount` will return `MountTimeout` — exercises T10.
    pub will_mount: bool,
    /// Fixed PNG bytes returned by `screenshot()`.
    pub screenshot_bytes: Vec<u8>,
    /// Fixed AX tree returned by `ax_full_tree()`.
    pub ax_tree: serde_json::Value,
    /// Fixed pointer hits (1000 entries expected per spec); the stub
    /// derives the hit list deterministically from the coord list.
    pub pointer_cursor: String,
    /// Fixed IME event payload list (empty for non-IME fixtures).
    pub ime_events: Vec<serde_json::Value>,
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-parity-oracle-src.md#schema
impl StubBrowserSession {
    pub fn new() -> Self {
        // Synthesise a deterministic 2x2 PNG so the pixel channel always
        // has something well-formed to ship.
        let img = image::RgbaImage::from_pixel(2, 2, image::Rgba([0, 0, 0, 255]));
        let mut bytes = Vec::new();
        {
            let mut cursor = std::io::Cursor::new(&mut bytes);
            image::DynamicImage::ImageRgba8(img)
                .write_to(&mut cursor, image::ImageFormat::Png)
                .unwrap();
        }
        Self {
            page: PageHost {
                url: String::new(),
                viewport: (1024, 768),
                mounted: false,
            },
            will_mount: true,
            screenshot_bytes: bytes,
            ax_tree: serde_json::json!({"nodes": []}),
            pointer_cursor: "auto".into(),
            ime_events: vec![],
        }
    }
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-parity-oracle-src.md#schema
impl Default for StubBrowserSession {
    fn default() -> Self {
        Self::new()
    }
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-parity-oracle-src.md#schema
#[async_trait]
impl BrowserSession for StubBrowserSession {
    fn browser_kind(&self) -> BrowserKind {
        BrowserKind::Chromium
    }
    fn page(&self) -> &PageHost {
        &self.page
    }
    fn page_mut(&mut self) -> &mut PageHost {
        &mut self.page
    }
    async fn launch(&mut self, _dpr: f32, viewport: (u32, u32)) -> Result<(), RunnerError> {
        self.page.viewport = viewport;
        Ok(())
    }
    async fn navigate(&mut self, url: &str) -> Result<(), RunnerError> {
        self.page.url = url.to_string();
        Ok(())
    }
    async fn await_mount(&mut self, budget: Duration) -> Result<(), RunnerError> {
        if self.will_mount {
            self.page.mounted = true;
            Ok(())
        } else {
            Err(RunnerError::MountTimeout(budget))
        }
    }
    async fn close(&mut self) -> Result<(), RunnerError> {
        Ok(())
    }
    async fn screenshot(&mut self) -> Result<Vec<u8>, ChannelError> {
        Ok(self.screenshot_bytes.clone())
    }
    async fn ax_full_tree(&mut self) -> Result<serde_json::Value, ChannelError> {
        Ok(self.ax_tree.clone())
    }
    async fn capture_focus_trace(
        &mut self,
        tab_count: u32,
    ) -> Result<Vec<crate::channels::FocusEntry>, ChannelError> {
        Ok((0..tab_count)
            .map(|i| crate::channels::FocusEntry {
                step: i,
                selector: "<body>".into(),
                role: "generic".into(),
                name: String::new(),
                bounds: [0.0, 0.0, 0.0, 0.0],
            })
            .collect())
    }
    async fn capture_pointer_hits(
        &mut self,
        coords: &[(u32, u32)],
    ) -> Result<Vec<crate::channels::PointerHit>, ChannelError> {
        Ok(coords
            .iter()
            .map(|&(x, y)| crate::channels::PointerHit {
                x,
                y,
                target_selector: "body".into(),
                computed_cursor: self.pointer_cursor.clone(),
            })
            .collect())
    }
    async fn capture_ime_trace(&mut self) -> Result<Vec<serde_json::Value>, ChannelError> {
        Ok(self.ime_events.clone())
    }
}

/// @spec parity-dom-reference-runner.md#Dependency (Runner)
pub struct Runner {
    config: RunnerConfig,
    matrix: MatrixEntry,
    session: Box<dyn BrowserSession>,
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-parity-oracle-src.md#schema
impl Runner {
    /// @spec parity-dom-reference-runner.md#Logic (init)
    ///
    /// Build a runner backed by the production [`PlaywrightBrowserSession`].
    pub fn new(config: RunnerConfig, matrix: MatrixEntry) -> Self {
        Self {
            config,
            matrix,
            session: Box::new(PlaywrightBrowserSession::new()),
        }
    }

    /// @spec parity-dom-reference-runner.md#Logic (init — test substitution)
    pub fn with_session(
        config: RunnerConfig,
        matrix: MatrixEntry,
        session: Box<dyn BrowserSession>,
    ) -> Self {
        Self {
            config,
            matrix,
            session,
        }
    }

    /// @spec parity-dom-reference-runner.md#Logic
    ///
    /// Walks the lifecycle:
    /// `load -> launch -> navigate -> wait_paint -> ch_pixel -> ch_a11y
    ///  -> ch_focus -> ch_pointer -> {ch_ime | ime_empty} -> bundle -> teardown -> done`.
    pub async fn run(&mut self, fixture_path: &Path) -> Result<ArtifactBundle, RunnerError> {
        // load
        let manifest = FixtureManifest::from_file(fixture_path)?;

        // launch
        self.session
            .launch(self.matrix.dpr, self.config.viewport)
            .await?;
        // navigate
        let url = format!(
            "file://{}?fixture={}",
            self.config.shell_html.display(),
            fixture_path.display()
        );
        self.session.navigate(&url).await?;
        // wait_paint
        self.session
            .await_mount(self.config.per_fixture_budget)
            .await?;

        // Per-fixture artifact root: artifacts/<fixture>/<browser>-<dpr>/
        let root = self.config.artifact_root.join(&manifest.name).join(format!(
            "{}-{}",
            self.matrix.browser.as_str(),
            self.matrix.dpr_label()
        ));
        let mut writer = ArtifactWriter::new(root.clone())?;

        // The five channels run in fixed order. We pre-build them with the
        // pixel filename embedded so PixelChannel knows the R4 naming
        // convention `<fixture>-<browser>-<dpr>.png`.
        let pixel_filename = format!(
            "{}-{}-{}.png",
            manifest.name,
            self.matrix.browser.as_str(),
            self.matrix.dpr_label()
        );
        let prng = DeterministicPrng::from_fixture_name(&manifest.name);
        let mut ctx = ChannelCtx {
            session: self.session.as_mut(),
            manifest: &manifest,
            matrix: self.matrix,
            prng,
        };

        // ch_pixel
        let pixel = PixelChannel::new(pixel_filename.clone());
        run_channel(&pixel, &mut ctx, &mut writer).await?;
        // ch_a11y
        run_channel(&A11yChannel, &mut ctx, &mut writer).await?;
        // ch_focus
        run_channel(&FocusChannel, &mut ctx, &mut writer).await?;
        // ch_pointer
        run_channel(&PointerChannel, &mut ctx, &mut writer).await?;
        // ch_ime (or empty trace if non-IME — handled inside channel)
        run_channel(&ImeChannel, &mut ctx, &mut writer).await?;

        // bundle
        let sha = writer.into_sha256s();
        let bundle = ArtifactBundle {
            root_dir: root.clone(),
            pixel_png: root.join(&pixel_filename),
            a11y_json: root.join("a11y-tree.json"),
            focus_json: root.join("focus-trace.json"),
            pointer_json: root.join("pointer-hitmap.json"),
            ime_json: root.join("ime-trace.json"),
            sha256s: sha,
        };
        // teardown
        self.session.close().await?;
        Ok(bundle)
    }
}

async fn run_channel(
    channel: &dyn Channel,
    ctx: &mut ChannelCtx<'_>,
    writer: &mut ArtifactWriter,
) -> Result<(), RunnerError> {
    let artifact = channel
        .capture(ctx)
        .await
        .map_err(|source| RunnerError::Channel {
            channel: channel.name(),
            source,
        })?;
    match artifact {
        ChannelArtifact::Png { filename, bytes } => {
            writer.write_png(&filename, &bytes)?;
        }
        ChannelArtifact::Json { filename, value } => {
            writer.write_json(&filename, &value)?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn matrix_dpr_label_stable() {
        let m = MatrixEntry {
            browser: BrowserKind::Chromium,
            dpr: 1.0,
        };
        assert_eq!(m.dpr_label(), "1.0");
        let m2 = MatrixEntry {
            browser: BrowserKind::Chromium,
            dpr: 2.0,
        };
        assert_eq!(m2.dpr_label(), "2.0");
    }

    #[tokio::test]
    async fn stub_session_round_trip() {
        let mut s = StubBrowserSession::new();
        s.launch(1.0, (800, 600)).await.unwrap();
        s.navigate("file:///x").await.unwrap();
        s.await_mount(Duration::from_secs(1)).await.unwrap();
        assert!(s.page().mounted);
        assert_eq!(s.page().url, "file:///x");
        let png = s.screenshot().await.unwrap();
        assert_eq!(&png[..8], &[0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A]);
    }

    #[tokio::test]
    async fn stub_mount_failure_yields_mount_timeout() {
        let mut s = StubBrowserSession::new();
        s.will_mount = false;
        let err = s.await_mount(Duration::from_millis(10)).await.unwrap_err();
        assert!(matches!(err, RunnerError::MountTimeout(_)));
    }
}
// CODEGEN-END
