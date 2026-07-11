//! Shared TCP accept/runtime layer.
//! @spec apps/agentic-workflow/tech-design/logic/shared-server-substrate-performance-layers.md#logic
//!
//! Protocol crates should implement [`TcpHandler`] and let this crate own the
//! listener loop, budget admission, task supervision, and drain behavior. HTTP
//! sits above this layer in `http-server`; raw protocol products such as a
//! Postgres pooler can use it directly.

use std::future::Future;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;

use anyhow::Result;
use server_core::{BindConfig, ConnectionBudget, DrainController, DrainSignal};
use socket2::{Domain, Protocol, Socket, Type};
use tokio::net::{TcpListener, TcpStream};
use tokio::task::JoinSet;

#[derive(Debug, Clone)]
pub struct TcpServerConfig {
    pub bind: BindConfig,
    pub connection_budget: Option<ConnectionBudget>,
    pub drain: DrainController,
    pub socket: TcpSocketOptions,
    pub drain_timeout: Duration,
}

impl TcpServerConfig {
    pub fn new(bind: BindConfig) -> Self {
        Self {
            bind,
            connection_budget: None,
            drain: DrainController::new(),
            socket: TcpSocketOptions::default(),
            drain_timeout: Duration::from_secs(5),
        }
    }

    pub fn with_connection_budget(mut self, budget: ConnectionBudget) -> Self {
        self.connection_budget = Some(budget);
        self
    }

    pub fn with_socket_options(mut self, socket: TcpSocketOptions) -> Self {
        self.socket = socket;
        self
    }

    pub fn with_drain_timeout(mut self, drain_timeout: Duration) -> Self {
        self.drain_timeout = drain_timeout;
        self
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TcpSocketOptions {
    pub backlog: i32,
    pub reuse_addr: bool,
    pub nodelay: bool,
}

impl Default for TcpSocketOptions {
    fn default() -> Self {
        Self {
            backlog: 1024,
            reuse_addr: true,
            nodelay: true,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ConnectionContext {
    pub local_addr: SocketAddr,
    pub peer_addr: SocketAddr,
    pub drain: DrainSignal,
}

/// Zero-boxing TCP protocol handler.
/// @spec apps/agentic-workflow/tech-design/logic/shared-server-substrate-performance-layers.md#logic
///
/// A blanket impl for closures keeps call sites terse while avoiding the
/// boxed-future cost of `async_trait` on every accepted connection.
pub trait TcpHandler: Send + Sync + 'static {
    type Future: Future<Output = Result<()>> + Send + 'static;

    fn handle(&self, stream: TcpStream, cx: ConnectionContext) -> Self::Future;
}

impl<F, Fut> TcpHandler for F
where
    F: Fn(TcpStream, ConnectionContext) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = Result<()>> + Send + 'static,
{
    type Future = Fut;

    fn handle(&self, stream: TcpStream, cx: ConnectionContext) -> Self::Future {
        self(stream, cx)
    }
}

pub async fn bind(config: &TcpServerConfig) -> std::io::Result<TcpListener> {
    // @spec apps/agentic-workflow/tech-design/logic/shared-server-substrate-performance-layers.md#logic
    let addr = config.bind.socket_addr();
    let domain = if addr.is_ipv6() {
        Domain::IPV6
    } else {
        Domain::IPV4
    };
    let socket = Socket::new(domain, Type::STREAM, Some(Protocol::TCP))?;
    if config.socket.reuse_addr {
        socket.set_reuse_address(true)?;
    }
    socket.set_nonblocking(true)?;
    socket.bind(&addr.into())?;
    socket.listen(config.socket.backlog)?;
    TcpListener::from_std(std::net::TcpListener::from(socket))
}

pub async fn serve<H, S>(listener: TcpListener, config: TcpServerConfig, handler: H, shutdown: S)
where
    H: TcpHandler,
    S: Future<Output = ()> + Send + 'static,
{
    serve_arc(listener, config, Arc::new(handler), shutdown).await;
}

pub async fn serve_arc<H, S>(
    listener: TcpListener,
    config: TcpServerConfig,
    handler: Arc<H>,
    shutdown: S,
) where
    H: TcpHandler,
    S: Future<Output = ()> + Send + 'static,
{
    // @spec apps/agentic-workflow/tech-design/logic/shared-server-substrate-performance-layers.md#logic
    let local_addr = listener.local_addr().ok();
    let mut shutdown = std::pin::pin!(shutdown);
    let mut tasks = JoinSet::new();

    loop {
        tokio::select! {
            joined = tasks.join_next(), if !tasks.is_empty() => {
                if let Some(Err(error)) = joined {
                    tracing::debug!(%error, "tcp connection task join failed");
                }
            }
            accept = listener.accept() => {
                let (stream, peer_addr) = match accept {
                    Ok(accepted) => accepted,
                    Err(error) => {
                        tracing::warn!(%error, "tcp accept failed");
                        continue;
                    }
                };

                if let Err(error) = stream.set_nodelay(config.socket.nodelay) {
                    tracing::debug!(%error, %peer_addr, "failed to set tcp nodelay");
                }

                let permit = match config.connection_budget.as_ref() {
                    Some(budget) => match budget.try_acquire() {
                        Ok(permit) => Some(permit),
                        Err(error) => {
                            tracing::warn!(%error, %peer_addr, "tcp connection rejected");
                            drop(stream);
                            continue;
                        }
                    },
                    None => None,
                };

                let handler = handler.clone();
                let cx = ConnectionContext {
                    local_addr: local_addr.unwrap_or_else(|| stream.local_addr().unwrap_or(config.bind.socket_addr())),
                    peer_addr,
                    drain: config.drain.signal(),
                };
                tasks.spawn(async move {
                    let _permit = permit;
                    handler.handle(stream, cx).await
                });
            }
            _ = &mut shutdown => {
                config.drain.start_drain();
                tracing::info!("tcp server stopped accepting connections");
                break;
            }
        }
    }

    drop(listener);

    let drain = async {
        while let Some(joined) = tasks.join_next().await {
            match joined {
                Ok(Ok(())) => {}
                Ok(Err(error)) => tracing::debug!(%error, "tcp connection handler failed"),
                Err(error) => tracing::debug!(%error, "tcp connection task join failed"),
            }
        }
    };

    if tokio::time::timeout(config.drain_timeout, drain)
        .await
        .is_err()
    {
        tracing::warn!(
            drain_timeout_ms = config.drain_timeout.as_millis(),
            "tcp drain timeout; abandoning remaining connection tasks"
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Result;
    use server_core::ConnectionBudget;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::sync::oneshot;

    #[tokio::test]
    async fn bind_uses_configured_socket_options() {
        // @spec apps/agentic-workflow/tech-design/logic/shared-server-substrate-performance-layers.md#unit-test
        let cfg =
            TcpServerConfig::new(BindConfig::localhost(0)).with_socket_options(TcpSocketOptions {
                backlog: 128,
                reuse_addr: true,
                nodelay: true,
            });
        let listener = bind(&cfg).await.expect("bind");
        assert!(listener.local_addr().unwrap().port() > 0);
    }

    #[tokio::test]
    async fn serve_accepts_closure_handler_without_async_trait_boxing() {
        // @spec apps/agentic-workflow/tech-design/logic/shared-server-substrate-performance-layers.md#unit-test
        let cfg = TcpServerConfig::new(BindConfig::localhost(0));
        let listener = bind(&cfg).await.expect("bind");
        let addr = listener.local_addr().unwrap();
        let (shutdown_tx, shutdown_rx) = oneshot::channel();

        let server = tokio::spawn(serve(
            listener,
            cfg,
            |mut stream: TcpStream, _cx: ConnectionContext| async move {
                let mut buf = [0_u8; 4];
                stream.read_exact(&mut buf).await?;
                stream.write_all(&buf).await?;
                Ok(())
            },
            async move {
                let _ = shutdown_rx.await;
            },
        ));

        let mut client = TcpStream::connect(addr).await.expect("connect");
        client.write_all(b"ping").await.expect("write");
        let mut out = [0_u8; 4];
        client.read_exact(&mut out).await.expect("read");
        assert_eq!(&out, b"ping");

        let _ = shutdown_tx.send(());
        server.await.expect("server task");
    }

    #[tokio::test]
    async fn connection_budget_releases_after_handler_finishes() {
        // @spec apps/agentic-workflow/tech-design/logic/shared-server-substrate-performance-layers.md#unit-test
        let budget = ConnectionBudget::new(1);
        let cfg =
            TcpServerConfig::new(BindConfig::localhost(0)).with_connection_budget(budget.clone());
        let listener = bind(&cfg).await.expect("bind");
        let addr = listener.local_addr().unwrap();
        let (shutdown_tx, shutdown_rx) = oneshot::channel();

        let server = tokio::spawn(serve(
            listener,
            cfg,
            |stream: TcpStream, _cx: ConnectionContext| async move {
                drop(stream);
                Result::<()>::Ok(())
            },
            async move {
                let _ = shutdown_rx.await;
            },
        ));

        let _client = TcpStream::connect(addr).await.expect("connect");
        tokio::time::timeout(Duration::from_secs(2), async {
            loop {
                if budget.active() == 0 {
                    break;
                }
                tokio::task::yield_now().await;
            }
        })
        .await
        .expect("permit released");

        let _ = shutdown_tx.send(());
        server.await.expect("server task");
    }
}
