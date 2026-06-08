// SPEC-MANAGED: projects/cap/tech-design/semantic/cap-src.md#schema
// CODEGEN-BEGIN
//! Client-side: connect to the cap daemon, send Request, parse Response.
//!
//! `connect()` is pure. `connect_or_launch()` takes a caller-supplied closure
//! to *start* a daemon when none is up. Keeping daemon launch caller-supplied
//! lets library consumers decide whether to spawn `cap daemon run`, shell out
//! to `cap`, or degrade without a daemon.

use anyhow::{anyhow, Context, Result};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::UnixStream;

use crate::paths;
use crate::protocol::{Request, Response};

/// @spec projects/cap/tech-design/semantic/cap-src.md#schema
pub struct Client {
    stream: BufReader<UnixStream>,
}

/// @spec projects/cap/tech-design/semantic/cap-src.md#schema
impl Client {
    /// Connect to an already-running daemon. Errors if none is listening.
    pub async fn connect() -> Result<Self> {
        let socket = paths::socket_path()?;
        let stream = UnixStream::connect(&socket)
            .await
            .with_context(|| format!("connect to cap daemon at {}", socket.display()))?;
        Ok(Self {
            stream: BufReader::new(stream),
        })
    }

    /// Try connecting; if no daemon is up, run `launch` to start one, then
    /// retry. `launch` is responsible for actually spawning a daemon process
    /// (cap's CLI passes `daemon::spawn_background`); it is only invoked when
    /// [`crate::is_running`] reports no live daemon, so a parallel race still
    /// converges on one daemon (the daemon itself holds an exclusive flock).
    pub async fn connect_or_launch<F>(launch: F) -> Result<Self>
    where
        F: FnOnce() -> Result<()>,
    {
        if let Ok(c) = Self::connect().await {
            return Ok(c);
        }
        if !crate::is_running().unwrap_or(false) {
            launch()?;
        }
        // Bounded retry loop — daemon needs to bind the UDS.
        for _ in 0..50 {
            tokio::time::sleep(std::time::Duration::from_millis(40)).await;
            if let Ok(c) = Self::connect().await {
                return Ok(c);
            }
        }
        Err(anyhow!("cap daemon failed to come up within 2s"))
    }

    pub async fn send(&mut self, req: &Request) -> Result<()> {
        let mut s = serde_json::to_string(req)?;
        s.push('\n');
        self.stream.get_mut().write_all(s.as_bytes()).await?;
        Ok(())
    }

    pub async fn recv(&mut self) -> Result<Response> {
        let mut line = String::new();
        let n = self.stream.read_line(&mut line).await?;
        if n == 0 {
            return Err(anyhow!("daemon closed connection unexpectedly"));
        }
        let resp: Response = serde_json::from_str(line.trim())
            .with_context(|| format!("parsing daemon response: {line:?}"))?;
        Ok(resp)
    }

    pub async fn request(&mut self, req: &Request) -> Result<Response> {
        self.send(req).await?;
        self.recv().await
    }
}
// CODEGEN-END
