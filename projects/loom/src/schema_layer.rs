//! loom **schema layer** (#432) — the lean, pgbouncer-class worker edge.
//!
//! Its job: **guarantee the message format/schema** so any hand-written, any-
//! language worker conforms or gets a clear error — no per-language SDK. Lean:
//! validate frames, **own the keep key schema** (the worker never builds a key),
//! multiplex workers onto the backends, forward. NOT app logic — the DAG fold
//! stays in the controller.
//!
//! Worker ⟷ layer is a **bidi h2 stream**: the layer pushes [`TaskEnvelope`]
//! frames down, the worker sends [`UpFrame`]s up. Frames are length-prefixed JSON
//! ([`encode_frame`] / [`FrameDecoder`]). The transport-agnostic core (frames +
//! envelope) is unit-tested; a [`TaskSource`] feeds it (relay lease-batch in #440).

use serde::{Deserialize, Serialize};

use crate::scheduler::TaskMessage;

/// Encode a value as a length-prefixed (4-byte big-endian length) JSON frame —
/// the wire framing for both directions of the bidi stream.
pub fn encode_frame<T: Serialize>(v: &T) -> Vec<u8> {
    let body = serde_json::to_vec(v).expect("encode frame");
    let mut buf = Vec::with_capacity(4 + body.len());
    buf.extend_from_slice(&(body.len() as u32).to_be_bytes());
    buf.extend_from_slice(&body);
    buf
}

/// Incremental frame decoder: feed arbitrary byte chunks from the stream, pull
/// complete frames (handles partial + coalesced reads).
#[derive(Default)]
pub struct FrameDecoder {
    buf: Vec<u8>,
}

impl FrameDecoder {
    pub fn push(&mut self, bytes: &[u8]) {
        self.buf.extend_from_slice(bytes);
    }
    /// The next complete frame's raw JSON body, if one is fully buffered.
    pub fn next_frame(&mut self) -> Option<Vec<u8>> {
        if self.buf.len() < 4 {
            return None;
        }
        let len = u32::from_be_bytes([self.buf[0], self.buf[1], self.buf[2], self.buf[3]]) as usize;
        if self.buf.len() < 4 + len {
            return None;
        }
        let frame = self.buf[4..4 + len].to_vec();
        self.buf.drain(..4 + len);
        Some(frame)
    }
}

/// Frames the worker sends UP the bidi stream. Strict deserialization *is* the
/// format guarantee — a malformed frame fails to parse and is rejected.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum UpFrame {
    /// First frame: join a work `group` with a `prefetch` (credit) window.
    Subscribe { group: String, prefetch: u32 },
    /// Task finished OK. Small results come back inline; large results are already
    /// in keep (the worker PUT them to the envelope's `result_put_url`).
    Done {
        id: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        result_inline: Option<Vec<u8>>,
    },
    /// Task could not be handled — redeliver to another consumer.
    Nack { id: String },
}

impl UpFrame {
    /// Parse + validate an up-frame. Errors on any malformed/unknown frame.
    pub fn parse(bytes: &[u8]) -> anyhow::Result<UpFrame> {
        Ok(serde_json::from_slice(bytes)?)
    }
}

/// Where a task's input lives (claim-check): inline for small payloads, a keep
/// URL for large, or empty.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum InputSource {
    Inline { bytes: Vec<u8> },
    KeepUrl { url: String },
    Empty,
}

/// Frame the proxy pushes DOWN: a fully self-describing task. The worker never
/// constructs a keep key — the proxy owns the schema — so it cannot get it wrong.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TaskEnvelope {
    /// Ack/dedup id = `run:node:attempt` (matches `UpFrame::Done.id`).
    pub id: String,
    pub run_id: String,
    pub node_id: String,
    pub attempt: u32,
    pub task_name: String,
    pub args: serde_json::Value,
    pub input: InputSource,
    /// Pre-resolved keep URL the worker PUTs a (large) result to.
    pub result_put_url: String,
    /// Scoped keep token issued per task (keep tokens #434/#445).
    pub token: String,
}

/// Build the self-describing envelope for a leased task. **Owns the keep key
/// schema**: input/result keys are constructed here, so the worker only does dumb
/// GET/PUT against given URLs.
pub fn build_envelope(task: &TaskMessage, keep_base: &str, token: String) -> TaskEnvelope {
    let result_key = format!("{}:{}:result", task.run_id, task.node_id);
    let input = if let Some(bytes) = &task.input_inline {
        InputSource::Inline { bytes: bytes.clone() }
    } else if let Some(first) = task.input_refs.first() {
        InputSource::KeepUrl { url: format!("{keep_base}/v1/inputs/{}", first.0) }
    } else {
        InputSource::Empty
    };
    TaskEnvelope {
        id: task.message_id(),
        run_id: task.run_id.clone(),
        node_id: task.node_id.clone(),
        attempt: task.attempt,
        task_name: task.task_name.clone(),
        args: task.args.clone(),
        input,
        result_put_url: format!("{keep_base}/v1/results/{result_key}"),
        token,
    }
}

// ---- bidi h2 worker endpoint (#439) -----------------------------------------

use std::sync::Arc;

use async_trait::async_trait;
use axum::body::{Body, BodyDataStream, Bytes};
use axum::extract::State;
use axum::response::Response;
use axum::routing::{get, post};
use axum::Router;
use futures::StreamExt;
use tokio::sync::mpsc;

/// Source of tasks for the schema layer + sink for their outcomes. The relay
/// lease-batch implementation is #440; tests use a fake.
#[async_trait]
pub trait TaskSource: Send + Sync {
    /// Long-poll: block until ≥1 task is available for `group`, return up to
    /// `max` ready envelopes. An empty return means the source is closed.
    async fn lease(&self, group: &str, max: u32) -> Vec<TaskEnvelope>;
    /// A task finished OK — forward the completion + ack the source.
    async fn done(&self, id: &str, result_inline: Option<Vec<u8>>);
    /// A task could not be handled — release it for redelivery.
    async fn nack(&self, id: &str);
}

/// Router for `loom schema-layer`: the worker-facing bidi work stream + health.
pub fn router(source: Arc<dyn TaskSource>) -> Router {
    Router::new()
        .route("/v1/work/stream", post(work_stream))
        .route("/healthz", get(|| async { "ok" }))
        .with_state(source)
}

/// POST /v1/work/stream — one bidi h2 stream per worker: TaskEnvelopes down,
/// UpFrames up. Both half-streams stay open for the session's life.
async fn work_stream(State(source): State<Arc<dyn TaskSource>>, req: Body) -> Response {
    let (down_tx, mut down_rx) = mpsc::channel::<Vec<u8>>(64);
    tokio::spawn(drive_session(source, req.into_data_stream(), down_tx));
    let body = Body::from_stream(async_stream::stream! {
        while let Some(frame) = down_rx.recv().await {
            yield Ok::<Bytes, std::convert::Infallible>(Bytes::from(frame));
        }
    });
    Response::new(body)
}

/// Read + strictly parse the next up-frame; None on stream end or a malformed
/// frame (which closes the session — the format guarantee).
async fn read_up(stream: &mut BodyDataStream, dec: &mut FrameDecoder) -> Option<UpFrame> {
    loop {
        if let Some(raw) = dec.next_frame() {
            return UpFrame::parse(&raw).ok();
        }
        match stream.next().await {
            Some(Ok(chunk)) => dec.push(&chunk),
            _ => return None,
        }
    }
}

/// Drive one worker session: Subscribe, then keep ≤ `prefetch` tasks in flight
/// (a credit freed per Ack/Done); redeliver via the source on Nack / disconnect.
async fn drive_session(source: Arc<dyn TaskSource>, mut up: BodyDataStream, down: mpsc::Sender<Vec<u8>>) {
    let mut dec = FrameDecoder::default();
    let (group, prefetch) = match read_up(&mut up, &mut dec).await {
        Some(UpFrame::Subscribe { group, prefetch }) => (group, prefetch.max(1)),
        _ => return, // first frame must be Subscribe
    };
    let mut inflight: u32 = 0;
    loop {
        if inflight < prefetch {
            tokio::select! {
                tasks = source.lease(&group, prefetch - inflight) => {
                    if tasks.is_empty() { break; }
                    for t in tasks {
                        if down.send(encode_frame(&t)).await.is_err() { return; }
                        inflight += 1;
                    }
                }
                up_frame = read_up(&mut up, &mut dec) => {
                    if !handle_up(&source, up_frame, &mut inflight).await { return; }
                }
            }
        } else if !handle_up(&source, read_up(&mut up, &mut dec).await, &mut inflight).await {
            return;
        }
    }
}

/// Apply one up-frame; returns false when the session should end (disconnect).
async fn handle_up(source: &Arc<dyn TaskSource>, frame: Option<UpFrame>, inflight: &mut u32) -> bool {
    match frame {
        Some(UpFrame::Done { id, result_inline }) => {
            source.done(&id, result_inline).await;
            *inflight = inflight.saturating_sub(1);
            true
        }
        Some(UpFrame::Nack { id }) => {
            source.nack(&id).await;
            *inflight = inflight.saturating_sub(1);
            true
        }
        Some(UpFrame::Subscribe { .. }) => true, // ignore re-subscribe
        None => false,
    }
}

/// Serve the schema-layer router over h2c on `addr` (mirror of the controller).
pub async fn serve(addr: &str, app: Router) -> anyhow::Result<()> {
    use hyper_util::rt::{TokioExecutor, TokioIo};
    use hyper_util::server::conn::auto;
    use tokio::net::TcpListener;
    use tower::ServiceExt;

    let listener = TcpListener::bind(addr).await?;
    eprintln!("loom schema-layer listening (h2c) on {addr}");
    let mut builder = auto::Builder::new(TokioExecutor::new());
    builder.http2().max_concurrent_streams(4096);
    loop {
        let (stream, _peer) = listener.accept().await?;
        let io = TokioIo::new(stream);
        let app = app.clone();
        let svc = hyper::service::service_fn(move |req| app.clone().oneshot(req));
        let conn = builder.serve_connection_with_upgrades(io, svc).into_owned();
        tokio::spawn(async move {
            let _ = conn.await;
        });
    }
}

// ---- relay-backed task source (#440) ----------------------------------------

use crate::relay_client::shard_of;
use crate::scheduler::CompletionMsg;
use std::collections::HashMap;

/// Parse an envelope/Done id ("run:node:attempt") back to its parts (run may
/// itself contain ':').
fn parse_id(id: &str) -> Option<(String, String, u32)> {
    let mut it = id.rsplitn(3, ':');
    let attempt: u32 = it.next()?.parse().ok()?;
    let node = it.next()?.to_string();
    let run = it.next()?.to_string();
    Some((run, node, attempt))
}

/// Build the completion to forward to the controller for a finished task. Small
/// results came back inline; large ones are at the conventional result key the
/// envelope told the worker to PUT to.
fn completion_for(id: &str, result_inline: Option<Vec<u8>>) -> Option<CompletionMsg> {
    let (run_id, node_id, attempt) = parse_id(id)?;
    let result_ref =
        if result_inline.is_some() { None } else { Some(format!("{run_id}:{node_id}:result")) };
    Some(CompletionMsg { run_id, node_id, attempt, result_ref, result_inline, failed: false, fan_out: vec![] })
}

/// A [`TaskSource`] over relay's work-queue (phase 1, relay UNCHANGED). lease-batch
/// returns only leases (no #166 entry bodies), so this leases singly — the single
/// lease returns the body — up to the credit window. Completions are forwarded to
/// the controller via the `loom.completions` subject; ack releases the relay lease.
pub struct RelayTaskSource {
    client: reqwest::Client,
    relay: String,
    keep: String,
    shards: u32,
    /// Optional HMAC secret: when set, each envelope gets a scoped keep token (#444).
    secret: Option<Vec<u8>>,
    inflight: tokio::sync::Mutex<HashMap<String, (String, u64, String)>>, // id -> (lease_id, epoch, subject)
}

impl RelayTaskSource {
    pub fn new(
        relay: impl Into<String>,
        keep: impl Into<String>,
        shards: u32,
        secret: Option<Vec<u8>>,
    ) -> anyhow::Result<Self> {
        Ok(Self {
            client: reqwest::Client::builder().http2_prior_knowledge().build()?,
            relay: relay.into(),
            keep: keep.into(),
            shards: shards.max(1),
            secret,
            inflight: tokio::sync::Mutex::new(HashMap::new()),
        })
    }

    /// Sign a scoped keep token for one task (readable input key + writable result
    /// key, 5-min TTL) — so the worker hits keep directly but only within scope.
    fn token_for(&self, task: &TaskMessage) -> String {
        let Some(secret) = &self.secret else { return String::new() };
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        let scope = claimtoken::Scope {
            r: task.input_refs.first().map(|k| k.0.clone()).unwrap_or_default(),
            w: format!("{}:{}:result", task.run_id, task.node_id),
            exp: now + 300,
        };
        claimtoken::sign(secret, &scope)
    }

    async fn lease_one(&self, group: &str) -> Option<TaskEnvelope> {
        let resp = self
            .client
            .post(format!("{}/v1/{group}/lease", self.relay))
            .json(&serde_json::json!({ "consumer_id": "loom-schema-layer" }))
            .send()
            .await
            .ok()?;
        let body: serde_json::Value = resp.json().await.ok()?;
        let lease = body.get("lease")?.as_object()?;
        let lease_id = lease.get("lease_id")?.as_str()?.to_string();
        let epoch = lease.get("epoch")?.as_u64()?;
        let payload = body.get("entry")?.get("payload")?.clone();
        let task: TaskMessage = serde_json::from_value(payload).ok()?;
        let env = build_envelope(&task, &self.keep, self.token_for(&task));
        self.inflight.lock().await.insert(env.id.clone(), (lease_id, epoch, group.to_string()));
        Some(env)
    }
}

#[async_trait]
impl TaskSource for RelayTaskSource {
    async fn lease(&self, group: &str, max: u32) -> Vec<TaskEnvelope> {
        loop {
            let mut out = Vec::new();
            for _ in 0..max {
                match self.lease_one(group).await {
                    Some(e) => out.push(e),
                    None => break,
                }
            }
            if !out.is_empty() {
                return out;
            }
            tokio::time::sleep(std::time::Duration::from_millis(50)).await; // long-poll
        }
    }

    async fn done(&self, id: &str, result_inline: Option<Vec<u8>>) {
        let Some((lease_id, epoch, subject)) = self.inflight.lock().await.remove(id) else { return };
        if let Some(cm) = completion_for(id, result_inline) {
            let subj = format!("loom.completions.{}", shard_of(&cm.run_id, self.shards));
            let _ = self
                .client
                .post(format!("{}/v1/{subj}/publish", self.relay))
                .json(&serde_json::json!({ "message_id": format!("{id}:done"), "payload": cm }))
                .send()
                .await;
        }
        let _ = self
            .client
            .post(format!("{}/v1/{subject}/ack", self.relay))
            .json(&serde_json::json!({ "lease_id": lease_id, "epoch": epoch }))
            .send()
            .await;
    }

    async fn nack(&self, id: &str) {
        // Drop tracking; the relay lease expires → redeliver (relay has no nack).
        self.inflight.lock().await.remove(id);
    }
}

/// Entry point for `loom schema-layer`: the worker bidi edge over a relay
/// work-queue source. Needs LOOM_RELAY + LOOM_KEEP; LOOM_ADDR (default
/// 0.0.0.0:7475); LOOM_COMPLETION_SHARDS (default 8, must match the controller).
pub fn run() -> anyhow::Result<()> {
    let relay = std::env::var("LOOM_RELAY")
        .map_err(|_| anyhow::anyhow!("loom schema-layer requires LOOM_RELAY"))?;
    let keep = std::env::var("LOOM_KEEP")
        .map_err(|_| anyhow::anyhow!("loom schema-layer requires LOOM_KEEP"))?;
    let addr = std::env::var("LOOM_ADDR").unwrap_or_else(|_| "0.0.0.0:7475".to_string());
    let shards = std::env::var("LOOM_COMPLETION_SHARDS")
        .ok()
        .and_then(|s| s.parse::<u32>().ok())
        .unwrap_or(8);
    // Scoped keep tokens (#444): sign per-task tokens when set (must match keep's
    // KEEP_TOKEN_SECRET). Absent → no tokens (open claim-check).
    let secret =
        std::env::var("LOOM_KEEP_TOKEN_SECRET").ok().filter(|s| !s.is_empty()).map(String::into_bytes);
    let tokens_on = secret.is_some();
    let rt = tokio::runtime::Runtime::new()?;
    rt.block_on(async move {
        let source = Arc::new(RelayTaskSource::new(&relay, &keep, shards, secret)?);
        eprintln!(
            "loom schema-layer: relay {relay}, keep {keep}, shards {shards} (keep tokens {})",
            if tokens_on { "on" } else { "off" }
        );
        serve(&addr, router(source)).await
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::KeepRef;
    use crate::runner::RunnerClass;

    fn task(input_refs: Vec<KeepRef>, inline: Option<Vec<u8>>) -> TaskMessage {
        TaskMessage {
            run_id: "run1".into(),
            node_id: "nodeA".into(),
            attempt: 2,
            task_name: "crunch".into(),
            args: serde_json::json!({"k": 1}),
            input_refs,
            input_inline: inline,
            runner: RunnerClass::Resident,
        }
    }

    #[test]
    fn envelope_owns_keys_and_resolves_input() {
        // input ref → keep GET url; result → keep PUT url; id = run:node:attempt
        let e = build_envelope(&task(vec![KeepRef("in:7".into())], None), "http://keep", "tok".into());
        assert_eq!(e.id, "run1:nodeA:2");
        assert_eq!(e.input, InputSource::KeepUrl { url: "http://keep/v1/inputs/in:7".into() });
        assert_eq!(e.result_put_url, "http://keep/v1/results/run1:nodeA:result");
        assert_eq!(e.token, "tok");

        // inline input wins over refs
        let e2 = build_envelope(&task(vec![], Some(b"hi".to_vec())), "http://keep", "t".into());
        assert_eq!(e2.input, InputSource::Inline { bytes: b"hi".to_vec() });

        // no input → empty
        let e3 = build_envelope(&task(vec![], None), "http://keep", "t".into());
        assert_eq!(e3.input, InputSource::Empty);
    }

    #[test]
    fn up_frames_round_trip_and_reject_malformed() {
        for f in [
            UpFrame::Subscribe { group: "w".into(), prefetch: 4 },
            UpFrame::Done { id: "run1:nodeA:2".into(), result_inline: Some(b"r".to_vec()) },
            UpFrame::Done { id: "x".into(), result_inline: None },
            UpFrame::Nack { id: "x".into() },
        ] {
            let bytes = serde_json::to_vec(&f).unwrap();
            assert_eq!(UpFrame::parse(&bytes).unwrap(), f);
        }
        // malformed / unknown frames are rejected (the format guarantee)
        assert!(UpFrame::parse(b"{}").is_err());
        assert!(UpFrame::parse(br#"{"type":"bogus"}"#).is_err());
        assert!(UpFrame::parse(b"not json").is_err());
    }

    #[test]
    fn frame_codec_handles_partial_and_coalesced() {
        let a = encode_frame(&UpFrame::Nack { id: "a".into() });
        let b = encode_frame(&UpFrame::Subscribe { group: "g".into(), prefetch: 2 });
        let mut d = FrameDecoder::default();

        // partial: header only, then the rest → one frame
        d.push(&a[..3]);
        assert!(d.next_frame().is_none());
        d.push(&a[3..]);
        let f1 = d.next_frame().unwrap();
        assert_eq!(UpFrame::parse(&f1).unwrap(), UpFrame::Nack { id: "a".into() });
        assert!(d.next_frame().is_none());

        // coalesced: two frames in one push → both decode in order
        let mut both = b.clone();
        both.extend_from_slice(&a);
        d.push(&both);
        assert_eq!(
            UpFrame::parse(&d.next_frame().unwrap()).unwrap(),
            UpFrame::Subscribe { group: "g".into(), prefetch: 2 }
        );
        assert_eq!(
            UpFrame::parse(&d.next_frame().unwrap()).unwrap(),
            UpFrame::Nack { id: "a".into() }
        );
        assert!(d.next_frame().is_none());
    }

    // ---- live bidi round-trip (#439) ----------------------------------------

    struct FakeSource {
        queue: tokio::sync::Mutex<Vec<TaskEnvelope>>,
        done: std::sync::Mutex<Vec<String>>,
    }
    #[async_trait]
    impl TaskSource for FakeSource {
        async fn lease(&self, _group: &str, _max: u32) -> Vec<TaskEnvelope> {
            let mut q = self.queue.lock().await;
            if q.is_empty() {
                drop(q);
                futures::future::pending::<()>().await; // no more tasks → block (only handle acks)
                unreachable!()
            }
            std::mem::take(&mut *q)
        }
        async fn done(&self, id: &str, _r: Option<Vec<u8>>) {
            self.done.lock().unwrap().push(id.to_string());
        }
        async fn nack(&self, _id: &str) {}
    }

    async fn spawn(app: Router) -> std::net::SocketAddr {
        use hyper_util::rt::{TokioExecutor, TokioIo};
        use hyper_util::server::conn::auto;
        use tower::ServiceExt;
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        tokio::spawn(async move {
            let mut b = auto::Builder::new(TokioExecutor::new());
            b.http2().max_concurrent_streams(64);
            loop {
                let (s, _) = listener.accept().await.unwrap();
                let io = TokioIo::new(s);
                let app = app.clone();
                let svc = hyper::service::service_fn(move |req| app.clone().oneshot(req));
                let conn = b.serve_connection_with_upgrades(io, svc).into_owned();
                tokio::spawn(async move {
                    let _ = conn.await;
                });
            }
        });
        addr
    }

    #[tokio::test]
    async fn bidi_round_trip_lease_push_done() {
        let env = build_envelope(&task(vec![], None), "http://keep", "tok".into());
        let src = Arc::new(FakeSource {
            queue: tokio::sync::Mutex::new(vec![env.clone()]),
            done: std::sync::Mutex::new(vec![]),
        });
        let addr = spawn(router(src.clone())).await;

        // worker opens a bidi stream: request body = up-frames, response = down-frames
        let (up_tx, up_rx) = mpsc::channel::<Bytes>(8);
        let body = reqwest::Body::wrap_stream(async_stream::stream! {
            let mut rx = up_rx;
            while let Some(b) = rx.recv().await { yield Ok::<Bytes, std::io::Error>(b); }
        });
        let client = reqwest::Client::builder().http2_prior_knowledge().build().unwrap();
        let resp = client
            .post(format!("http://{addr}/v1/work/stream"))
            .body(body)
            .send()
            .await
            .unwrap();
        let mut down = resp.bytes_stream();

        up_tx
            .send(Bytes::from(encode_frame(&UpFrame::Subscribe { group: "w".into(), prefetch: 4 })))
            .await
            .unwrap();

        // read the pushed task envelope
        let mut dec = FrameDecoder::default();
        let got: TaskEnvelope = loop {
            if let Some(raw) = dec.next_frame() {
                break serde_json::from_slice(&raw).unwrap();
            }
            let chunk = down.next().await.expect("down stream ended early").unwrap();
            dec.push(&chunk);
        };
        assert_eq!(got.id, env.id);

        // ack it back up the same stream
        up_tx
            .send(Bytes::from(encode_frame(&UpFrame::Done {
                id: got.id.clone(),
                result_inline: None,
            })))
            .await
            .unwrap();

        // the server folds it via the source
        let mut ok = false;
        for _ in 0..100 {
            if src.done.lock().unwrap().contains(&env.id) {
                ok = true;
                break;
            }
            tokio::time::sleep(std::time::Duration::from_millis(20)).await;
        }
        assert!(ok, "server did not receive Done over the bidi stream");
        drop(up_tx);
    }

    #[test]
    fn id_parse_and_completion() {
        assert_eq!(parse_id("run1:nodeA:2"), Some(("run1".into(), "nodeA".into(), 2)));
        assert_eq!(parse_id("a:b:node:5"), Some(("a:b".into(), "node".into(), 5))); // run keeps colons
        assert!(parse_id("bad").is_none());
        // small result → inline (no ref); large → ref at the conventional key
        let c = completion_for("r:n:1", Some(b"x".to_vec())).unwrap();
        assert_eq!((c.result_ref, c.result_inline, c.failed), (None, Some(b"x".to_vec()), false));
        assert_eq!(completion_for("r:n:1", None).unwrap().result_ref, Some("r:n:result".into()));
    }

    #[test]
    fn signed_token_scopes_to_input_and_result_keys() {
        let src = RelayTaskSource::new("r", "http://keep", 8, Some(b"secret".to_vec())).unwrap();
        let tok = src.token_for(&task(vec![KeepRef("in:1".into())], None));
        let scope = claimtoken::verify(b"secret", &tok, 0).unwrap();
        assert_eq!(scope.r, "in:1");
        assert_eq!(scope.w, "run1:nodeA:result");
        assert!(claimtoken::verify(b"wrong", &tok, 0).is_none());
        // no secret → empty token
        let src2 = RelayTaskSource::new("r", "http://keep", 8, None).unwrap();
        assert_eq!(src2.token_for(&task(vec![], None)), "");
    }
}
