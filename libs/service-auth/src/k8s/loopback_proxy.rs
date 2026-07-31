// HANDWRITE-BEGIN gap="missing-generator:logic:loopback-credential-proxy" tracker="#2878" reason="A forwarding proxy whose contract is where the credential is *not* — not in the child's environment, not in its argv, not in its address space — and whose failure mode must be a refusal rather than a stale token; no generator primitive models either half."
//! Handing a credential to a program without handing it the credential.
//!
//! A CLI that wraps another command — `foo connect -- curl ...`, `foo connect
//! -- pytest` — has to get an authenticated connection to the child somehow.
//! The obvious way is an environment variable, and it is the wrong way: the
//! child's environment is inherited by every descendant it spawns, readable
//! from `/proc/<pid>/environ` by anything running as the same user, and
//! captured verbatim by most crash reporters and process supervisors. A token
//! put there has been handed to a much larger set of programs than the one
//! that was wrapped.
//!
//! So the child is given a URL instead. The token stays in the parent's
//! address space, and this proxy attaches it to each request as it passes:
//!
//! ```text
//!   child process  --http://127.0.0.1:<ephemeral>-->  LoopbackProxy
//!   (holds a URL and nothing else)                          |
//!                                        Authorization: Bearer <token from
//!                                        TokenSource, refreshed as needed>
//!                                                            v
//!                                                        upstream
//! ```
//!
//! Two properties make that worth the machinery rather than security
//! decoration:
//!
//! - **The listener is loopback-only and ephemeral.** It binds `127.0.0.1:0`,
//!   so nothing off the host can reach it and no port is predictable between
//!   runs. It is still reachable by any process on the host running as the
//!   same user — which is the same trust boundary the child itself sits in,
//!   and strictly smaller than the environment-variable boundary, which also
//!   includes anything that inherits or reads that environment later.
//! - **A refresh failure is a refusal.** When the token cannot be renewed —
//!   the grant was revoked, the kubeconfig expired — the proxy answers `503`
//!   and reports the failure on [`LoopbackProxy::next_fatal`] so the caller
//!   can tear the whole thing down. Continuing to forward the token already in
//!   hand until it expires would convert a revocation into a delay.
//!
//! Any inbound `Authorization` header is replaced, not merged: the proxy is
//! the only thing that decides what identity these requests carry, and a child
//! that sets its own must not be able to talk past it.

use std::net::SocketAddr;
use std::sync::Arc;

use axum::body::Body;
use axum::extract::State;
use axum::http::header::{HeaderMap, HeaderName, AUTHORIZATION};
use axum::http::{Request, StatusCode};
use axum::response::{IntoResponse, Response};
use tokio::sync::{mpsc, oneshot};

use super::token_request::{TokenRequestError, TokenSource};

/// The largest request or response body this will buffer.
///
/// The proxy reads each body whole rather than streaming it, which is the
/// simplification a short-lived local process can afford and a server cannot.
/// The cap exists so that "afford" stays true.
const MAX_BODY_BYTES: usize = 512 * 1024 * 1024;

/// Headers that describe the hop rather than the message, and so must not be
/// copied onto the next one. `authorization` is in this list because the proxy
/// supplies its own, and `host` because the upstream URL determines it.
const PER_HOP_HEADERS: &[&str] = &[
    "authorization",
    "connection",
    "content-length",
    "host",
    "keep-alive",
    "proxy-authenticate",
    "proxy-authorization",
    "te",
    "trailer",
    "transfer-encoding",
    "upgrade",
];

fn is_per_hop(name: &HeaderName) -> bool {
    PER_HOP_HEADERS.contains(&name.as_str())
}

/// A client that connects to `addr` while addressing — and verifying —
/// `server_name`, against `ca_pem` and nothing else.
///
/// This is what makes a forwarded local socket safe to use as a transport. A
/// port-forward's local end is `127.0.0.1`, but the thing on the far end of it
/// is a named Service holding a certificate for that name; a client that
/// verified `127.0.0.1` would be verifying the tunnel rather than what the
/// tunnel reaches, and the only certificate that could satisfy it is one
/// nobody should issue. So the URL carries the real name — SNI, hostname
/// verification, and any `Host` header all follow from it — and only address
/// resolution is redirected.
///
/// The built-in root store is switched off deliberately. A private CA that is
/// merely *added* to the public roots means any public CA can still vouch for
/// this name, which is the whole property a private trust domain buys.
pub fn verifying_client(
    ca_pem: &str,
    server_name: &str,
    addr: SocketAddr,
) -> Result<reqwest::Client, reqwest::Error> {
    reqwest::Client::builder()
        .tls_built_in_root_certs(false)
        .add_root_certificate(reqwest::Certificate::from_pem(ca_pem.as_bytes())?)
        .resolve(server_name, addr)
        .build()
}

#[derive(Clone)]
struct ProxyState {
    upstream: String,
    tokens: Arc<TokenSource>,
    client: reqwest::Client,
    fatal: mpsc::Sender<TokenRequestError>,
}

/// A running loopback listener that authenticates what passes through it.
///
/// Dropping this stops the listener: the shutdown signal is held here, and
/// `axum`'s graceful shutdown fires when it goes. So a caller that returns
/// early — including by `?` on an unrelated error — does not leave a
/// credential-bearing port open behind it.
pub struct LoopbackProxy {
    addr: SocketAddr,
    fatal: mpsc::Receiver<TokenRequestError>,
    // Held, never sent on: the receiver inside the serving task completes when
    // this is dropped, which is the shutdown edge.
    _shutdown: oneshot::Sender<()>,
    handle: tokio::task::JoinHandle<()>,
}

impl LoopbackProxy {
    /// Bind an ephemeral loopback port and start forwarding to `upstream`.
    ///
    /// Returns once the port is bound and accepting, so a caller may hand the
    /// URL to a child immediately without racing it.
    pub async fn start(
        upstream: impl Into<String>,
        tokens: Arc<TokenSource>,
    ) -> std::io::Result<Self> {
        Self::start_with_client(upstream, tokens, reqwest::Client::new()).await
    }

    /// The same, forwarding through a caller-supplied client.
    ///
    /// The client is the seam because "how the upstream is trusted" is not this
    /// module's question. A private-CA deployment builds one with
    /// [`verifying_client`]; a plaintext development deployment passes the
    /// default. Either way what happens to the credential is identical, which
    /// is why the two share this code path rather than forking it.
    pub async fn start_with_client(
        upstream: impl Into<String>,
        tokens: Arc<TokenSource>,
        client: reqwest::Client,
    ) -> std::io::Result<Self> {
        let listener = tokio::net::TcpListener::bind(("127.0.0.1", 0)).await?;
        let addr = listener.local_addr()?;
        let (fatal_tx, fatal_rx) = mpsc::channel(1);
        let (shutdown_tx, shutdown_rx) = oneshot::channel();

        let state = ProxyState {
            upstream: upstream.into().trim_end_matches('/').to_string(),
            tokens,
            client,
            fatal: fatal_tx,
        };
        let app = axum::Router::new().fallback(forward).with_state(state);

        let handle = tokio::spawn(async move {
            let _ = axum::serve(listener, app)
                .with_graceful_shutdown(async move {
                    let _ = shutdown_rx.await;
                })
                .await;
        });

        Ok(Self {
            addr,
            fatal: fatal_rx,
            _shutdown: shutdown_tx,
            handle,
        })
    }

    /// The address the child is given. Always on `127.0.0.1`.
    pub fn addr(&self) -> SocketAddr {
        self.addr
    }

    /// The URL the child is given — and, by design, the only thing it is
    /// given.
    pub fn local_url(&self) -> String {
        format!("http://{}", self.addr)
    }

    /// Resolves when the proxy can no longer authenticate a request.
    ///
    /// A caller should select on this alongside whatever else it is waiting
    /// for, and shut down when it fires: past this point every forwarded
    /// request is a `503`, and the useful thing to do is say why once rather
    /// than let the child discover it request by request.
    pub async fn next_fatal(&mut self) -> Option<TokenRequestError> {
        self.fatal.recv().await
    }

    /// Stop the listener and wait for in-flight requests to finish.
    pub async fn shutdown(self) {
        let Self {
            _shutdown, handle, ..
        } = self;
        drop(_shutdown);
        let _ = handle.await;
    }
}

async fn forward(State(state): State<ProxyState>, request: Request<Body>) -> Response {
    let token = match state.tokens.token().await {
        Ok(token) => token,
        Err(error) => {
            let message = error.to_string();
            // Best-effort: the channel holds one, and one report is the point.
            let _ = state.fatal.try_send(error);
            return (
                StatusCode::SERVICE_UNAVAILABLE,
                format!("this connection can no longer be authenticated: {message}\n"),
            )
                .into_response();
        }
    };

    let (parts, body) = request.into_parts();
    let body = match axum::body::to_bytes(body, MAX_BODY_BYTES).await {
        Ok(bytes) => bytes,
        Err(_) => {
            return (
                StatusCode::PAYLOAD_TOO_LARGE,
                format!("request body exceeds the {MAX_BODY_BYTES}-byte forwarding limit\n"),
            )
                .into_response()
        }
    };

    let target = format!(
        "{}{}",
        state.upstream,
        parts
            .uri
            .path_and_query()
            .map(|pq| pq.as_str())
            .unwrap_or("/")
    );

    let mut outbound = state.client.request(parts.method, &target);
    for (name, value) in parts.headers.iter() {
        if is_per_hop(name) {
            continue;
        }
        outbound = outbound.header(name.clone(), value.clone());
    }
    // Last, and unconditional: whatever the child sent under this name has
    // already been dropped above.
    outbound = outbound.header(AUTHORIZATION, format!("Bearer {}", token.expose()));

    let upstream = match outbound.body(body).send().await {
        Ok(response) => response,
        Err(error) => {
            return (
                StatusCode::BAD_GATEWAY,
                // `error` renders the URL and the transport failure. It cannot
                // render the header that was set, and this is the only place
                // the token and an error message are in scope together.
                format!("upstream request failed: {error}\n"),
            )
                .into_response();
        }
    };

    let status = upstream.status();
    let headers = upstream.headers().clone();
    let payload = match upstream.bytes().await {
        Ok(bytes) => bytes,
        Err(error) => {
            return (
                StatusCode::BAD_GATEWAY,
                format!("upstream response could not be read: {error}\n"),
            )
                .into_response()
        }
    };

    let mut response_headers = HeaderMap::new();
    for (name, value) in headers.iter() {
        if is_per_hop(name) {
            continue;
        }
        response_headers.insert(name.clone(), value.clone());
    }
    (status, response_headers, payload).into_response()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::sync::Mutex;
    use std::time::Duration;

    use async_trait::async_trait;

    use super::super::cache::{Clock, ManualClock};
    use super::super::token_request::{MintedToken, TokenMinter, TokenRequestTarget};

    const CANARY: &str = "canary-proxy-token-must-never-reach-the-child";

    struct Minter {
        clock: Arc<ManualClock>,
        calls: AtomicU64,
        fail_after: u64,
    }

    #[async_trait]
    impl TokenMinter for Minter {
        async fn mint(
            &self,
            target: &TokenRequestTarget,
        ) -> Result<MintedToken, TokenRequestError> {
            let n = self.calls.fetch_add(1, Ordering::SeqCst);
            if n >= self.fail_after {
                return Err(TokenRequestError::Forbidden {
                    username: Some("alice@example.com".to_string()),
                    namespace: target.namespace().to_string(),
                    service_account: target.service_account().to_string(),
                    detail: "the grant was revoked".to_string(),
                });
            }
            let now = self.clock.now_millis();
            Ok(MintedToken::new(
                format!("{CANARY}-{n}"),
                now,
                now + 600_000,
            ))
        }
    }

    fn source(clock: Arc<ManualClock>, fail_after: u64) -> Arc<TokenSource> {
        let minter = Arc::new(Minter {
            clock: clock.clone(),
            calls: AtomicU64::new(0),
            fail_after,
        });
        Arc::new(TokenSource::with_clock(
            minter,
            TokenRequestTarget::new("ops", "app-client", "callee.example.com")
                .expect("valid target"),
            clock,
        ))
    }

    /// Everything the upstream saw, so an assertion can be about the request
    /// that arrived rather than about the proxy's internal state.
    #[derive(Default)]
    struct Seen {
        authorization: Vec<String>,
        methods: Vec<String>,
        paths: Vec<String>,
        bodies: Vec<String>,
        custom: Vec<Option<String>>,
    }

    /// A minimal upstream. Deliberately not a mock library: what is being
    /// tested is the bytes on the wire.
    async fn upstream(seen: Arc<Mutex<Seen>>) -> String {
        let listener = tokio::net::TcpListener::bind(("127.0.0.1", 0))
            .await
            .expect("bind upstream");
        let addr = listener.local_addr().expect("upstream addr");
        let app = axum::Router::new().fallback(move |request: Request<Body>| {
            let seen = seen.clone();
            async move {
                let (parts, body) = request.into_parts();
                let body = axum::body::to_bytes(body, MAX_BODY_BYTES)
                    .await
                    .expect("read body");
                let mut seen = seen.lock().expect("record");
                seen.authorization.push(
                    parts
                        .headers
                        .get(AUTHORIZATION)
                        .and_then(|v| v.to_str().ok())
                        .unwrap_or("<absent>")
                        .to_string(),
                );
                seen.methods.push(parts.method.to_string());
                seen.paths.push(
                    parts
                        .uri
                        .path_and_query()
                        .map(|p| p.to_string())
                        .unwrap_or_default(),
                );
                seen.bodies.push(String::from_utf8_lossy(&body).to_string());
                seen.custom.push(
                    parts
                        .headers
                        .get("x-caller")
                        .and_then(|v| v.to_str().ok())
                        .map(str::to_string),
                );
                (StatusCode::OK, [("x-upstream", "yes")], "{\"ok\":true}")
            }
        });
        tokio::spawn(async move {
            let _ = axum::serve(listener, app).await;
        });
        format!("http://{addr}")
    }

    /// AC4: the child's request reaches the upstream carrying the minted
    /// token, and the child never had to hold it.
    #[tokio::test]
    async fn the_proxy_attaches_the_token_and_forwards_everything_else_unchanged() {
        let seen = Arc::new(Mutex::new(Seen::default()));
        let base = upstream(seen.clone()).await;
        let clock = Arc::new(ManualClock::new(0));
        let proxy = LoopbackProxy::start(base, source(clock, u64::MAX))
            .await
            .expect("start proxy");

        assert!(
            proxy.local_url().starts_with("http://127.0.0.1:"),
            "the child must be given a loopback URL: {}",
            proxy.local_url()
        );

        let response = reqwest::Client::new()
            .post(format!(
                "{}/collections/docs/search?limit=5",
                proxy.local_url()
            ))
            .header("x-caller", "the-child")
            .body("{\"query\":{}}")
            .send()
            .await
            .expect("through the proxy");
        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(
            response
                .headers()
                .get("x-upstream")
                .map(|v| v.to_str().unwrap()),
            Some("yes"),
            "the upstream's own headers come back"
        );
        assert_eq!(response.text().await.expect("body"), "{\"ok\":true}");

        let seen = seen.lock().expect("read records");
        assert_eq!(seen.authorization, vec![format!("Bearer {CANARY}-0")]);
        assert_eq!(seen.methods, vec!["POST"]);
        assert_eq!(seen.paths, vec!["/collections/docs/search?limit=5"]);
        assert_eq!(seen.bodies, vec!["{\"query\":{}}"]);
        assert_eq!(
            seen.custom,
            vec![Some("the-child".to_string())],
            "headers that are not the proxy's business pass through"
        );
    }

    /// The child does not get to choose who it is. A request that arrives with
    /// its own `Authorization` is forwarded with the proxy's, not the child's,
    /// and not both.
    #[tokio::test]
    async fn a_child_supplied_authorization_header_is_replaced_rather_than_merged() {
        let seen = Arc::new(Mutex::new(Seen::default()));
        let base = upstream(seen.clone()).await;
        let clock = Arc::new(ManualClock::new(0));
        let proxy = LoopbackProxy::start(base, source(clock, u64::MAX))
            .await
            .expect("start proxy");

        reqwest::Client::new()
            .get(proxy.local_url())
            .header(AUTHORIZATION, "Bearer a-google-access-token")
            .send()
            .await
            .expect("through the proxy");

        let seen = seen.lock().expect("read records");
        assert_eq!(seen.authorization, vec![format!("Bearer {CANARY}-0")]);
        assert!(
            !seen.authorization[0].contains("google"),
            "the child's own credential must not reach the upstream: {:?}",
            seen.authorization
        );
    }

    /// AC5 through the proxy: once the grant is gone the proxy refuses, says
    /// why, and tells the caller to shut down — rather than serving the token
    /// it still holds until that one expires too.
    #[tokio::test]
    async fn a_refresh_failure_becomes_a_refusal_and_a_shutdown_signal() {
        let seen = Arc::new(Mutex::new(Seen::default()));
        let base = upstream(seen.clone()).await;
        let clock = Arc::new(ManualClock::new(0));
        let mut proxy = LoopbackProxy::start(base, source(clock.clone(), 1))
            .await
            .expect("start proxy");
        let client = reqwest::Client::new();

        let first = client
            .get(proxy.local_url())
            .send()
            .await
            .expect("the first request works");
        assert_eq!(first.status(), StatusCode::OK);

        // Past the refresh point, with the token still nominally valid.
        clock.advance(Duration::from_secs(500));

        let refused = client
            .get(proxy.local_url())
            .send()
            .await
            .expect("the proxy answers rather than hanging");
        assert_eq!(refused.status(), StatusCode::SERVICE_UNAVAILABLE);
        let explanation = refused.text().await.expect("body");
        assert!(explanation.contains("alice@example.com"), "{explanation}");
        assert!(!explanation.contains(CANARY), "{explanation}");

        let fatal = tokio::time::timeout(Duration::from_secs(5), proxy.next_fatal())
            .await
            .expect("the caller is told, rather than left polling")
            .expect("a fatal error");
        assert!(
            matches!(fatal, TokenRequestError::Forbidden { .. }),
            "{fatal:?}"
        );

        let seen = seen.lock().expect("read records");
        assert_eq!(
            seen.authorization.len(),
            1,
            "the refused request must not reach the upstream at all"
        );
    }

    /// AC6 at this layer: the token appears in exactly one place — the
    /// `Authorization` header of the forwarded request — and in nothing the
    /// child can read back.
    #[tokio::test]
    async fn no_response_the_child_can_read_contains_the_token() {
        let seen = Arc::new(Mutex::new(Seen::default()));
        let base = upstream(seen.clone()).await;
        let clock = Arc::new(ManualClock::new(0));
        let proxy = LoopbackProxy::start(base, source(clock, u64::MAX))
            .await
            .expect("start proxy");
        let client = reqwest::Client::new();

        let ok = client.get(proxy.local_url()).send().await.expect("ok");
        let ok_headers = format!("{:?}", ok.headers());
        let ok_body = ok.text().await.expect("body");
        assert!(!ok_headers.contains(CANARY), "{ok_headers}");
        assert!(!ok_body.contains(CANARY), "{ok_body}");

        // The bad-gateway path, where an error string and the token are in
        // scope in the same function.
        let broken = LoopbackProxy::start(
            "http://127.0.0.1:1",
            source(Arc::new(ManualClock::new(0)), u64::MAX),
        )
        .await
        .expect("start proxy");
        let failed = client
            .get(broken.local_url())
            .send()
            .await
            .expect("the proxy answers");
        assert_eq!(failed.status(), StatusCode::BAD_GATEWAY);
        let body = failed.text().await.expect("body");
        assert!(
            !body.contains(CANARY),
            "an error echoed the credential: {body}"
        );
    }

    /// The listener is bound to loopback, not to every interface. Asserted on
    /// the bound address rather than by attempting an off-host connection,
    /// which a test cannot do portably.
    #[tokio::test]
    async fn the_listener_is_loopback_only_and_on_an_unpredictable_port() {
        let clock = Arc::new(ManualClock::new(0));
        let first = LoopbackProxy::start("http://127.0.0.1:1", source(clock.clone(), u64::MAX))
            .await
            .expect("start");
        let second = LoopbackProxy::start("http://127.0.0.1:1", source(clock, u64::MAX))
            .await
            .expect("start");

        assert!(first.addr().ip().is_loopback());
        assert!(second.addr().ip().is_loopback());
        assert_ne!(first.addr().port(), 0);
        assert_ne!(
            first.addr().port(),
            second.addr().port(),
            "an ephemeral port is chosen per run, not a fixed one"
        );
    }

    /// R7: when the proxy goes, the port goes. A caller that returns early
    /// must not leave an authenticating listener behind it.
    #[tokio::test]
    async fn dropping_the_proxy_closes_the_port() {
        let seen = Arc::new(Mutex::new(Seen::default()));
        let base = upstream(seen).await;
        let clock = Arc::new(ManualClock::new(0));
        let proxy = LoopbackProxy::start(base, source(clock, u64::MAX))
            .await
            .expect("start");
        let url = proxy.local_url();
        let addr = proxy.addr();

        assert!(reqwest::Client::new().get(&url).send().await.is_ok());
        proxy.shutdown().await;

        let reconnect =
            tokio::time::timeout(Duration::from_secs(5), tokio::net::TcpStream::connect(addr))
                .await
                .expect("the connect attempt itself must not hang");
        assert!(
            reconnect.is_err(),
            "the loopback port is still accepting after shutdown"
        );
    }
}
// HANDWRITE-END
