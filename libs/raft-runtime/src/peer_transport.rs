// HANDWRITE-BEGIN gap="missing-generator:logic:508c7859" tracker="#1643" reason="Reloadable last-known-good mTLS client/server snapshot, raw TLS connect/accept seams, HTTPS client, and TLS HTTP/2 listener."
//! Reloadable mutually authenticated transport for Raft peer traffic.

use std::fmt;
use std::future::Future;
use std::sync::{Arc, RwLock};

use anyhow::{bail, Context, Result};
use axum::Router;
use rustls::pki_types::ServerName;
use tokio::net::{TcpListener, TcpStream};
use tokio_rustls::{TlsAcceptor, TlsConnector};

#[derive(Clone)]
struct PeerTransportSnapshot {
    generation: u64,
    client_config: Arc<rustls::ClientConfig>,
    server_config: Arc<rustls::ServerConfig>,
    http_client: reqwest::Client,
}

struct PeerTransportInner {
    snapshot: RwLock<PeerTransportSnapshot>,
}

/// One atomic client/server mTLS generation shared by a Raft host and its
/// peer listener. Reloads affect new connections only; existing HTTP/2
/// sessions drain naturally on their old generation.
#[derive(Clone)]
pub struct PeerTransport(Arc<PeerTransportInner>);

impl fmt::Debug for PeerTransport {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PeerTransport")
            .field("generation", &self.generation())
            .finish_non_exhaustive()
    }
}

impl PeerTransport {
    pub fn from_config(config: &peer_tls::PeerTlsConfig) -> Result<Self> {
        let snapshot = build_snapshot(config, 1)?;
        Ok(Self(Arc::new(PeerTransportInner {
            snapshot: RwLock::new(snapshot),
        })))
    }

    pub fn generation(&self) -> u64 {
        self.read_snapshot().generation
    }

    /// Build and validate a complete replacement before atomically publishing
    /// it. Any error leaves the current client, acceptor, and generation intact.
    pub fn reload(&self, config: &peer_tls::PeerTlsConfig) -> Result<u64> {
        let next = self.generation().saturating_add(1);
        let replacement = build_snapshot(config, next)?;
        *self
            .0
            .snapshot
            .write()
            .unwrap_or_else(|poisoned| poisoned.into_inner()) = replacement;
        Ok(next)
    }

    pub fn http_client(&self) -> reqwest::Client {
        self.read_snapshot().http_client
    }

    /// Connect one already-established TCP stream and validate the server's
    /// DNS identity plus certificate chain before returning decrypted bytes.
    pub async fn connect(
        &self,
        stream: TcpStream,
        expected_identity: &str,
    ) -> Result<tokio_rustls::client::TlsStream<TcpStream>> {
        let server_name = ServerName::try_from(expected_identity.to_owned())
            .context("invalid expected peer TLS identity")?;
        let connector = TlsConnector::from(self.read_snapshot().client_config);
        connector
            .connect(server_name, stream)
            .await
            .context("peer TLS client handshake")
    }

    /// Accept one peer and require a trusted client certificate before any
    /// HTTP request reaches the Raft router.
    pub async fn accept(
        &self,
        stream: TcpStream,
    ) -> Result<tokio_rustls::server::TlsStream<TcpStream>> {
        let acceptor = TlsAcceptor::from(self.read_snapshot().server_config);
        acceptor
            .accept(stream)
            .await
            .context("peer TLS server handshake")
    }

    /// Serve a dedicated authenticated peer listener. The public service port
    /// remains owned by `service-http`; only the Raft router is passed here.
    pub async fn serve(
        &self,
        listener: TcpListener,
        app: Router,
        shutdown: impl Future<Output = ()> + Send,
    ) -> Result<()> {
        tokio::pin!(shutdown);
        loop {
            tokio::select! {
                _ = &mut shutdown => return Ok(()),
                accepted = listener.accept() => {
                    let (stream, peer_addr) = accepted.context("accept peer TCP connection")?;
                    let transport = self.clone();
                    let app = app.clone();
                    tokio::spawn(async move {
                        let tls = match transport.accept(stream).await {
                            Ok(tls) => tls,
                            Err(error) => {
                                tracing::warn!(%peer_addr, %error, "rejected unauthenticated raft peer");
                                return;
                            }
                        };
                        if let Err(error) = transport_h2c::server::serve_io(tls, app).await {
                            tracing::debug!(%peer_addr, %error, "raft peer HTTP/2 connection ended");
                        }
                    });
                }
            }
        }
    }

    fn read_snapshot(&self) -> PeerTransportSnapshot {
        self.0
            .snapshot
            .read()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .clone()
    }
}

fn build_snapshot(
    config: &peer_tls::PeerTlsConfig,
    generation: u64,
) -> Result<PeerTransportSnapshot> {
    if !config.required {
        bail!("raft peer transport requires mutual TLS; set the peer mTLS posture to on");
    }
    let client_config = Arc::new(config.rustls_client_config()?);
    let server_config = Arc::new(config.rustls_server_config()?);
    let http_client = reqwest::Client::builder()
        .https_only(true)
        .use_preconfigured_tls((*client_config).clone())
        .build()
        .context("build peer HTTPS client")?;
    Ok(PeerTransportSnapshot {
        generation,
        client_config,
        server_config,
        http_client,
    })
}
// HANDWRITE-END
