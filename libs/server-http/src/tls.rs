// HANDWRITE-BEGIN gap="missing-generator:logic:server-http-tls-listener" tracker="#3112" reason="Terminating TLS per connection against a configuration that may change between accepts is control flow over the existing accept loop; no generator primitive expresses it."
//! Serving TLS on the shared listener (#3112 R1, R4, AC1, AC6).
//!
//! The whole design is one decision: the rustls configuration is fetched *per
//! accepted connection* instead of being captured once when the listener is
//! built.
//!
//! That single change is what makes certificate rotation free. There is no
//! second accept loop, no listener rebind, no process restart, and no window
//! where the socket is unbound — connection N uses whatever was active when it
//! arrived, connection N+1 uses whatever is active when *it* arrives, and
//! connections already in flight finish on the configuration they started with,
//! inside the drain window `server-tcp` already implements.
//!
//! What lives here is exactly the listener mechanics. Deciding *whether* a
//! candidate certificate is fit to serve, and holding the last known good one,
//! is [`peer_tls::reload`]'s job; this crate never parses PEM and takes no
//! opinion on identity. The seam between them is a closure returning
//! `Option<Arc<ServerConfig>>`, which is also why `server-http` does not depend
//! on `peer-tls`: a listener should not need a certificate lifecycle to exist.

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

use server_lifecycle::BindConfig;
use tokio::net::TcpListener;

use crate::HttpServerOptions;

/// What the listener consults on every accept.
///
/// `None` means "nothing valid is active", and the connection is refused rather
/// than served. That is the runtime half of failing closed (#3112 R7): a leaf
/// that expired with no replacement stops being an identity, and the honest
/// answer to a handshake at that point is a closed connection, not a successful
/// one.
pub type ServerConfigSource =
    Arc<dyn Fn() -> Option<Arc<rustls::ServerConfig>> + Send + Sync + 'static>;

/// Build a [`ServerConfigSource`] from any closure.
pub fn config_source<F>(f: F) -> ServerConfigSource
where
    F: Fn() -> Option<Arc<rustls::ServerConfig>> + Send + Sync + 'static,
{
    Arc::new(f)
}

#[derive(Debug, Default)]
struct Counters {
    established: AtomicU64,
    handshake_failures: AtomicU64,
    refused_without_material: AtomicU64,
}

/// Bounded counters for the TLS edge of the listener.
///
/// Three numbers, no labels, and nothing derived from a certificate: a refusal
/// during a rotation is otherwise completely invisible, and the alternative —
/// logging why each handshake failed — puts attacker-controlled bytes into
/// request logs (#3112 R6).
#[derive(Clone, Debug, Default)]
pub struct TlsListenerMetrics {
    counters: Arc<Counters>,
}

/// A point-in-time read of [`TlsListenerMetrics`].
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct TlsListenerSnapshot {
    /// Connections whose handshake completed.
    pub established: u64,
    /// Connections that presented something rustls would not accept.
    pub handshake_failures: u64,
    /// Connections refused because no valid material was active.
    pub refused_without_material: u64,
}

impl TlsListenerMetrics {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn snapshot(&self) -> TlsListenerSnapshot {
        TlsListenerSnapshot {
            established: self.counters.established.load(Ordering::Relaxed),
            handshake_failures: self.counters.handshake_failures.load(Ordering::Relaxed),
            refused_without_material: self
                .counters
                .refused_without_material
                .load(Ordering::Relaxed),
        }
    }
}

/// TLS-specific listener settings, layered over [`HttpServerOptions`].
#[derive(Clone, Debug, Default)]
pub struct TlsServerOptions {
    pub http: HttpServerOptions,
    pub metrics: TlsListenerMetrics,
}

/// Serve HTTPS on `listener`, terminating TLS with whatever `config` returns at
/// the moment each connection is accepted.
///
/// ALPN comes from the supplied `ServerConfig` and is deliberately not set here:
/// a peer port offering `h2` alone and a public port offering `h2` and
/// `http/1.1` are the same listener code with different material.
pub async fn serve_tls(
    listener: TcpListener,
    app: axum::Router,
    config: ServerConfigSource,
    options: TlsServerOptions,
    shutdown: impl std::future::Future<Output = ()> + Send + 'static,
) {
    let TlsServerOptions { http, metrics } = options;
    let local_addr = listener
        .local_addr()
        .unwrap_or_else(|_| BindConfig::default().socket_addr());
    let mut tcp_config = server_tcp::TcpServerConfig::new(BindConfig {
        host: local_addr.ip(),
        port: local_addr.port(),
    })
    .with_socket_options(http.socket)
    .with_drain(http.drain)
    .with_drain_timeout(http.drain_timeout)
    .with_connection_metrics(http.connection_metrics);
    if let Some(budget) = http.connection_budget {
        tcp_config = tcp_config.with_connection_budget(budget);
    }

    let connection_options = transport_h2c::ConnectionOptions {
        max_concurrent_streams: http.max_concurrent_streams,
    };

    server_tcp::serve(
        listener,
        tcp_config,
        move |stream, _cx| {
            let app = app.clone();
            // Read once, here, at accept time. This is the activation point:
            // everything after it belongs to one connection and cannot be
            // changed out from under it.
            let active = config();
            let counters = Arc::clone(&metrics.counters);
            async move {
                let Some(active) = active else {
                    counters
                        .refused_without_material
                        .fetch_add(1, Ordering::Relaxed);
                    // Deliberately not a fallback to cleartext. A listener that
                    // downgraded here would answer the same port with the same
                    // routes and no encryption, and nothing downstream could
                    // tell the difference.
                    return Err(anyhow::anyhow!(
                        "no valid TLS material is active; refusing the connection"
                    ));
                };
                let accepted = tokio_rustls::TlsAcceptor::from(active).accept(stream).await;
                let tls_stream = match accepted {
                    Ok(stream) => {
                        counters.established.fetch_add(1, Ordering::Relaxed);
                        stream
                    }
                    Err(error) => {
                        counters.handshake_failures.fetch_add(1, Ordering::Relaxed);
                        return Err(anyhow::anyhow!("tls handshake failed: {error}"));
                    }
                };
                transport_h2c::server::serve_io_with_options(tls_stream, app, connection_options)
                    .await
                    .map_err(|error| anyhow::anyhow!(error.to_string()))
            }
        },
        shutdown,
    )
    .await;
}
// HANDWRITE-END
