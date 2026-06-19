// SPEC-MANAGED: .aw/tech-design/projects/jet/semantic/jet-browser.md#schema
// CODEGEN-BEGIN
//! Low-level Chrome DevTools Protocol (CDP) client.
//!
//! Manages a WebSocket connection to Chrome's debugging port and provides
//! typed request/response over the CDP JSON-RPC protocol.

use anyhow::{Context, Result};
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use tokio::sync::{mpsc, oneshot, Mutex};
use tokio_tungstenite::{connect_async, tungstenite::Message};

/// A CDP session — either the root browser session or a page-level session.
/// @spec .aw/tech-design/projects/jet/semantic/jet-browser.md#schema
#[derive(Clone)]
pub struct CdpSession {
    sender: mpsc::Sender<OutgoingMessage>,
    session_id: Option<String>,
    next_id: Arc<AtomicU64>,
    pending: Arc<Mutex<HashMap<u64, oneshot::Sender<Result<Value>>>>>,
}

/// CDP client connected to a browser's WebSocket debugging endpoint.
/// @spec .aw/tech-design/projects/jet/semantic/jet-browser.md#schema
pub struct CdpClient {
    session: CdpSession,
    /// Receiver for CDP events (method notifications without an id).
    events_rx: Option<mpsc::Receiver<CdpEvent>>,
    /// Handle to the background reader task.
    _reader_handle: tokio::task::JoinHandle<()>,
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-browser.md#schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CdpEvent {
    pub method: String,
    pub params: Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_id: Option<String>,
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
    #[serde(rename = "sessionId")]
    session_id: Option<String>,
}

#[derive(Debug, Deserialize)]
struct CdpError {
    code: i64,
    message: String,
}

enum OutgoingMessage {
    Send(String),
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-browser.md#schema
impl CdpClient {
    /// Connect to a CDP WebSocket endpoint.
    pub async fn connect(ws_url: &str) -> Result<Self> {
        let (ws_stream, _) = connect_async(ws_url)
            .await
            .context("Failed to connect to CDP WebSocket")?;

        let (mut ws_sink, mut ws_stream_reader) = ws_stream.split();

        let (outgoing_tx, mut outgoing_rx) = mpsc::channel::<OutgoingMessage>(64);
        let (events_tx, events_rx) = mpsc::channel::<CdpEvent>(256);

        let next_id = Arc::new(AtomicU64::new(1));
        let pending: Arc<Mutex<HashMap<u64, oneshot::Sender<Result<Value>>>>> =
            Arc::new(Mutex::new(HashMap::new()));

        let pending_clone = pending.clone();
        let events_tx_clone = events_tx.clone();

        // Background writer: sends outgoing messages to WebSocket.
        tokio::spawn(async move {
            while let Some(OutgoingMessage::Send(text)) = outgoing_rx.recv().await {
                if ws_sink.send(Message::Text(text.into())).await.is_err() {
                    break;
                }
            }
        });

        // Background reader: dispatches responses and events.
        let reader_handle = tokio::spawn(async move {
            while let Some(Ok(msg)) = ws_stream_reader.next().await {
                let text = match msg {
                    Message::Text(t) => t.to_string(),
                    Message::Close(_) => break,
                    _ => continue,
                };

                let resp: CdpResponse = match try_parse_cdp_response(&text) {
                    Some(r) => r,
                    None => continue,
                };

                if let Some(id) = resp.id {
                    // This is a response to a request.
                    let mut map = pending_clone.lock().await;
                    if let Some(tx) = map.remove(&id) {
                        let result = if let Some(err) = resp.error {
                            Err(anyhow::anyhow!("CDP error {}: {}", err.code, err.message))
                        } else {
                            Ok(resp.result.unwrap_or(Value::Null))
                        };
                        tx.send(result).ok();
                    }
                } else if let Some(method) = resp.method {
                    // This is an event notification.
                    let event = CdpEvent {
                        method,
                        params: resp.params.unwrap_or(Value::Null),
                        session_id: resp.session_id,
                    };
                    events_tx_clone.send(event).await.ok();
                }
            }
        });

        let session = CdpSession {
            sender: outgoing_tx,
            session_id: None,
            next_id,
            pending,
        };

        Ok(Self {
            session,
            events_rx: Some(events_rx),
            _reader_handle: reader_handle,
        })
    }

    /// Send a CDP command and await the response.
    pub async fn send(&self, method: &str, params: Value) -> Result<Value> {
        self.session.send(method, params).await
    }

    /// Create a new target (tab/page) and return its target ID.
    pub async fn create_target(&self, url: &str) -> Result<String> {
        let result = self
            .send("Target.createTarget", serde_json::json!({ "url": url }))
            .await?;
        result["targetId"]
            .as_str()
            .map(|s| s.to_string())
            .context("Missing targetId in createTarget response")
    }

    /// Attach to a target and return a session-scoped CdpSession.
    pub async fn attach_to_target(&self, target_id: &str) -> Result<CdpSession> {
        let result = self
            .send(
                "Target.attachToTarget",
                serde_json::json!({
                    "targetId": target_id,
                    "flatten": true,
                }),
            )
            .await?;
        let session_id = result["sessionId"]
            .as_str()
            .context("Missing sessionId in attachToTarget response")?
            .to_string();

        Ok(CdpSession {
            sender: self.session.sender.clone(),
            session_id: Some(session_id),
            next_id: self.session.next_id.clone(),
            pending: self.session.pending.clone(),
        })
    }

    /// Receive the next CDP event. Returns `None` if the connection is closed.
    pub async fn next_event(&mut self) -> Option<CdpEvent> {
        match self.events_rx.as_mut() {
            Some(rx) => rx.recv().await,
            None => None,
        }
    }

    /// Transfer the CDP event stream to a worker-owned pump.
    pub(crate) fn take_event_receiver(&mut self) -> Option<mpsc::Receiver<CdpEvent>> {
        self.events_rx.take()
    }

    /// Clone of the root (browser-level) CDP session for Target / Storage
    /// domain calls that are not scoped to a page `sessionId`.
    pub fn root_session(&self) -> CdpSession {
        self.session.clone()
    }
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-browser.md#schema
impl CdpSession {
    /// Send a CDP command within this session and await the response.
    pub async fn send(&self, method: &str, params: Value) -> Result<Value> {
        let id = self.next_id.fetch_add(1, Ordering::SeqCst);

        let request = CdpRequest {
            id,
            method: method.to_string(),
            params,
            session_id: self.session_id.clone(),
        };

        let (tx, rx) = oneshot::channel();
        {
            let mut map = self.pending.lock().await;
            map.insert(id, tx);
        }

        let json = serde_json::to_string(&request)?;
        self.sender
            .send(OutgoingMessage::Send(json))
            .await
            .map_err(|_| {
                anyhow::anyhow!(format_cdp_send_closed_err(
                    method,
                    self.session_id.as_deref()
                ))
            })?;

        rx.await
            .map_err(|_| anyhow::anyhow!("CDP response dropped"))?
    }

    /// Derive a new `CdpSession` scoped to `session_id`, sharing the transport
    /// (sender + pending map + next_id) with this session so the background
    /// reader routes responses correctly.
    pub fn child_session(&self, session_id: String) -> CdpSession {
        CdpSession {
            sender: self.sender.clone(),
            session_id: Some(session_id),
            next_id: self.next_id.clone(),
            pending: self.pending.clone(),
        }
    }

    /// Return this session's CDP sessionId if it is a sub-session.
    pub fn session_id(&self) -> Option<&str> {
        self.session_id.as_deref()
    }
}

/// Parse a single CDP WebSocket text frame as a `CdpResponse`.
///
/// Returns `None` on parse failure (the caller skips that frame and keeps the
/// reader loop alive) and emits a `tracing::warn!` under target
/// `jet::browser::cdp` tagged `GH #3333` so a stalled request id has an
/// observable cause instead of silently hanging.
// @spec GH #3333
fn try_parse_cdp_response(text: &str) -> Option<CdpResponse> {
    match serde_json::from_str::<CdpResponse>(text) {
        Ok(resp) => Some(resp),
        Err(err) => {
            let preview: String = text.chars().take(200).collect();
            tracing::warn!(
                target: "jet::browser::cdp",
                error = %err,
                text_len = text.len(),
                text_preview = %preview,
                "GH #3333 dropped a CDP frame that failed to parse as \
                 CdpResponse. The matching request id (if any) will time \
                 out — investigate the preview for protocol drift or \
                 envelope changes."
            );
            None
        }
    }
}

/// Format the error returned when `CdpSession::send` fails to push the
/// request onto the outgoing-message channel.
///
/// Names the CDP method, the session id (or `(root)` when sending on the
/// browser-level session), and a brief diagnosis pointer so the dev knows
/// the WS transport died before the request left the process. Tagged
/// `GH #3554` for grep-discovery from the user-facing error.
/// @spec .aw/tech-design/projects/jet/semantic/jet-browser.md#schema
pub(crate) fn format_cdp_send_closed_err(method: &str, session_id: Option<&str>) -> String {
    let session_label = session_id.unwrap_or("(root)");
    format!(
        "GH #3554 CDP connection closed while sending {method} (session={session_label}); the outgoing-message channel was dropped, which means the background WebSocket writer task has already exited — typically because the browser process crashed, the user closed the window, or the remote side closed the ws. Subsequent CDP calls on this session will also fail. Inspect the browser process exit status / ws close reason."
    )
}

/// Format the error returned when `CdpSession::send` enqueued the request
/// successfully but the oneshot response receiver was dropped before a
/// response arrived.
///
/// Names the CDP method + session id and explains that the pending-map
/// entry is now orphaned (the response, if it ever arrives, will be
/// dropped on the floor). Distinct wording from the connection-closed
/// variant so the dev can tell whether the request left the process at
/// all. Tagged `GH #3554`.
/// @spec .aw/tech-design/projects/jet/semantic/jet-browser.md#schema
pub(crate) fn format_cdp_response_dropped_err(method: &str, session_id: Option<&str>) -> String {
    let session_label = session_id.unwrap_or("(root)");
    format!(
        "GH #3554 CDP response receiver dropped for {method} (session={session_label}); the request was enqueued but the oneshot receiver was discarded before the background reader could deliver the response. Typical cause: the reader task died (browser disconnect mid-RPC) and tore down the pending map. The CDP call did not see a meaningful answer; assume the side-effect did NOT happen and re-issue the command after reconnecting."
    )
}

#[cfg(test)]
mod gh3333_tests {
    use super::*;

    /// Happy path: a well-formed CDP response envelope parses to Some.
    #[test]
    fn gh3333_try_parse_cdp_response_valid_returns_some() {
        let text = r#"{"id":42,"result":{"value":"ok"}}"#;
        let resp = try_parse_cdp_response(text).expect("valid envelope must parse");
        assert_eq!(resp.id, Some(42));
        assert!(resp.result.is_some());
    }

    /// Event-shape envelope (no id, with method) parses too.
    #[test]
    fn gh3333_try_parse_cdp_response_event_envelope_parses() {
        let text = r#"{"method":"Target.attachedToTarget","params":{"sessionId":"abc"}}"#;
        let resp = try_parse_cdp_response(text).expect("event envelope must parse");
        assert!(resp.id.is_none());
        assert_eq!(resp.method.as_deref(), Some("Target.attachedToTarget"));
    }

    /// Malformed JSON → None (caller skips that frame instead of stalling).
    /// The warn is emitted but not asserted here — the contract is that the
    /// reader loop stays alive and the bad frame is dropped.
    #[test]
    fn gh3333_try_parse_cdp_response_malformed_returns_none() {
        let text = "{this is not json";
        assert!(
            try_parse_cdp_response(text).is_none(),
            "malformed JSON must yield None so reader continues"
        );
    }

    /// Valid JSON but wrong shape (string instead of object) → None.
    /// Validates that envelope-shape drift is also surfaced through the
    /// same path as plain syntax errors.
    #[test]
    fn gh3333_try_parse_cdp_response_wrong_shape_returns_none() {
        let text = r#""just a string""#;
        assert!(
            try_parse_cdp_response(text).is_none(),
            "wrong-shape JSON must yield None"
        );
    }

    // ─── GH #3554: CDP send/recv errors must name the method ───────────

    /// GH #3554 — both error strings must include the GH tag, the CDP
    /// method, and the session id (or `(root)` for the browser session).
    #[test]
    fn gh3554_cdp_errors_name_method_session_and_issue() {
        let send_err = format_cdp_send_closed_err("Page.navigate", Some("ABC-123"));
        assert!(
            send_err.contains("GH #3554"),
            "send err must tag GH #3554, got: {send_err}"
        );
        assert!(
            send_err.contains("Page.navigate"),
            "send err must name method, got: {send_err}"
        );
        assert!(
            send_err.contains("ABC-123"),
            "send err must name session id, got: {send_err}"
        );

        let recv_err = format_cdp_response_dropped_err("Browser.close", None);
        assert!(
            recv_err.contains("GH #3554"),
            "recv err must tag GH #3554, got: {recv_err}"
        );
        assert!(
            recv_err.contains("Browser.close"),
            "recv err must name method, got: {recv_err}"
        );
        assert!(
            recv_err.contains("(root)"),
            "recv err must label root session as (root), got: {recv_err}"
        );
    }

    /// GH #3554 — connection-closed message must hint at likely root
    /// causes (browser crashed / user closed the window / ws closed by
    /// peer) so the dev knows where to look first.
    #[test]
    fn gh3554_cdp_send_closed_err_hints_likely_causes() {
        let msg = format_cdp_send_closed_err("DOM.focus", Some("S"));
        assert!(
            msg.contains("browser process crashed")
                || msg.contains("browser")
                || msg.contains("ws"),
            "must hint at browser/ws root causes, got: {msg}"
        );
        assert!(
            msg.contains("Subsequent") || msg.contains("subsequent"),
            "must spell out the cascade consequence, got: {msg}"
        );
    }

    /// GH #3554 — the two helpers must produce DIFFERENT messages for
    /// identical inputs so the dev can tell whether the request never
    /// left the process (send) vs. left but the answer never came back
    /// (recv). Without distinguishability the helpers collapse into one
    /// noisy string and we lose the entire reason for the split.
    #[test]
    fn gh3554_cdp_send_and_recv_errors_are_distinguishable() {
        let send_err = format_cdp_send_closed_err("Page.navigate", Some("X"));
        let recv_err = format_cdp_response_dropped_err("Page.navigate", Some("X"));

        assert_ne!(
            send_err, recv_err,
            "send and recv errors must differ so triage can tell send-side from recv-side failures"
        );
        // Send-side: emphasises the writer task / connection.
        assert!(
            send_err.contains("connection closed") || send_err.contains("writer"),
            "send err must point at the send-side, got: {send_err}"
        );
        // Recv-side: emphasises the response / oneshot / reader task.
        assert!(
            recv_err.contains("receiver dropped")
                || recv_err.contains("oneshot")
                || recv_err.contains("reader"),
            "recv err must point at the receive-side, got: {recv_err}"
        );
    }
}
// CODEGEN-END
