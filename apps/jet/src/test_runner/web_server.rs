// SPEC-MANAGED: .aw/tech-design/projects/jet/semantic/jet-test-runner.md#schema
// CODEGEN-BEGIN
//! Web-server supervisor for `jet test`.
//!
//! Spawns a long-running subprocess (typically a dev server) before the
//! runner dispatches specs, polls a readiness signal, and kills the process
//! when the supervisor is dropped — so a crashing runner never leaves an
//! orphan server.
//!
//! Spec: `.aw/tech-design/projects/jet/logic/web-server.md` (W1..W6).

use crate::task_runner::config::WebServerConfig;
use anyhow::{anyhow, Context, Result};
use std::path::Path;
use std::process::Stdio;
use std::time::{Duration, Instant};
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::net::TcpStream;
use tokio::process::{Child, Command};
use tokio::time::sleep;

/// RAII handle for a spawned web server. `drop` sends SIGKILL to the child
/// and waits (best-effort — if the child already exited, drop is a no-op).
// @spec web-server#W3
#[derive(Debug)]
pub struct WebServerHandle {
    child: Option<Child>,
    /// Human label for log messages.
    label: String,
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-test-runner.md#schema
impl WebServerHandle {
    /// Test-only constructor — lets tests build a "reused" handle that owns
    /// no child.
    #[cfg(test)]
    fn reused(label: String) -> Self {
        Self { child: None, label }
    }
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-test-runner.md#schema
impl Drop for WebServerHandle {
    fn drop(&mut self) {
        if let Some(mut child) = self.child.take() {
            // `start_kill` requires a runtime; it's a non-blocking signal.
            // We don't await the exit — drop can't be async — so we only
            // ensure the SIGKILL is sent. Any zombie is reaped by init.
            if let Err(e) = child.start_kill() {
                eprintln!("[jet] failed to kill web_server {}: {e}", self.label);
            }
        }
    }
}

/// Boot the configured web server and wait for it to become ready.
///
/// Contract:
/// - If `reuse_existing` is true and the probe already succeeds, returns
///   a handle that owns no child (never sends a signal on drop).
/// - If the probe never succeeds inside `timeout_ms`, the child is killed
///   and `Err` is returned.
/// - Spawning uses `sh -c <command>` so shell features (pipes, env) work.
///
/// @spec web-server#W2 W3 W4
pub async fn boot(cfg: &WebServerConfig, project_root: &Path) -> Result<WebServerHandle> {
    let label = short_label(&cfg.command);

    // Reuse-existing short-circuit.
    if cfg.reuse_existing && probe_ready(cfg).await {
        eprintln!("[jet] web_server: reusing existing instance ({label})");
        return Ok(WebServerHandle { child: None, label });
    }

    // Resolve working directory.
    let cwd = match &cfg.cwd {
        Some(rel) => project_root.join(rel),
        None => project_root.to_path_buf(),
    };

    let mut cmd = Command::new("sh");
    cmd.arg("-c")
        .arg(&cfg.command)
        .current_dir(&cwd)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .kill_on_drop(true);

    let mut child = cmd
        .spawn()
        .with_context(|| format!("Failed to spawn web_server: {}", cfg.command))?;

    // Forward stdout / stderr to the runner's stderr with a `[web_server]`
    // prefix so the user can debug why their server didn't come up.
    if let Some(stdout) = child.stdout.take() {
        spawn_log_pump("stdout", stdout);
    }
    if let Some(stderr) = child.stderr.take() {
        spawn_log_pump("stderr", stderr);
    }

    let started = Instant::now();
    let deadline = Duration::from_millis(cfg.timeout_ms);
    loop {
        // Early-exit if the child has already died — no point polling.
        if let Ok(Some(status)) = child.try_wait() {
            return Err(anyhow!(
                "web_server exited before becoming ready (status={status}, command={})",
                cfg.command
            ));
        }
        if probe_ready(cfg).await {
            eprintln!("[jet] web_server: ready ({label})");
            return Ok(WebServerHandle {
                child: Some(child),
                label,
            });
        }
        if started.elapsed() >= deadline {
            let _ = child.start_kill();
            return Err(anyhow!(
                "web_server did not become ready within {}ms (command={})",
                cfg.timeout_ms,
                cfg.command
            ));
        }
        sleep(Duration::from_millis(100)).await;
    }
}

/// Returns true when the configured readiness probe succeeds now.
/// URL wins over port — `hyper` / `reqwest` is avoided by using a tiny
/// raw-socket HTTP check (good enough to tell "accepting connections" from
/// "nothing listening").
// @spec web-server#W5
async fn probe_ready(cfg: &WebServerConfig) -> bool {
    if let Some(url) = &cfg.url {
        return probe_http(url).await;
    }
    if let Some(port) = cfg.port {
        return probe_tcp(port).await;
    }
    // No probe configured → optimistic ready.
    true
}

async fn probe_tcp(port: u16) -> bool {
    TcpStream::connect(("127.0.0.1", port)).await.is_ok()
}

/// Very small HTTP probe: opens a TCP connection to `host:port` parsed from
/// `url`, writes a minimal `GET / HTTP/1.0`, and considers any response
/// (even a 500) as ready. This avoids a `reqwest` dependency just for one
/// liveness probe.
async fn probe_http(url: &str) -> bool {
    use tokio::io::AsyncWriteExt;
    let Some((host, port, path)) = parse_http_url(url) else {
        return false;
    };
    let Ok(mut stream) = TcpStream::connect((host.as_str(), port)).await else {
        return false;
    };
    let req = format!("GET {path} HTTP/1.0\r\nHost: {host}\r\nConnection: close\r\n\r\n");
    if stream.write_all(req.as_bytes()).await.is_err() {
        return false;
    }
    let mut buf = [0u8; 16];
    // We only need to know bytes flow back — full parse would be overkill.
    tokio::io::AsyncReadExt::read(&mut stream, &mut buf)
        .await
        .is_ok()
}

/// Parse `"http://host:port/path"` or `"http://host/path"` into
/// `(host, port, path)`. Returns `None` on malformed input. HTTPS is not
/// supported by the probe — wrap an internal HTTP port instead.
fn parse_http_url(url: &str) -> Option<(String, u16, String)> {
    let rest = url.strip_prefix("http://")?;
    let (authority, path) = match rest.find('/') {
        Some(i) => (&rest[..i], &rest[i..]),
        None => (rest, "/"),
    };
    let (host, port) = match authority.rsplit_once(':') {
        Some((h, p)) => (h.to_string(), p.parse::<u16>().ok()?),
        None => (authority.to_string(), 80),
    };
    Some((host, port, path.to_string()))
}

fn short_label(command: &str) -> String {
    // First whitespace-separated token is usually the binary name.
    command
        .split_whitespace()
        .next()
        .unwrap_or("web_server")
        .to_string()
}

/// Spawn a detached task that tags every line of child output with
/// `[web_server/<stream>]` and forwards it to stderr. Stopped when the
/// underlying pipe closes (child exits or explicitly dropped).
fn spawn_log_pump<R>(stream_name: &'static str, pipe: R)
where
    R: tokio::io::AsyncRead + Unpin + Send + 'static,
{
    tokio::spawn(async move {
        let reader = BufReader::new(pipe);
        let mut lines = reader.lines();
        while let Ok(Some(line)) = lines.next_line().await {
            eprintln!("[web_server/{stream_name}] {line}");
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_http_url_with_port_and_path() {
        let (h, p, path) = parse_http_url("http://localhost:8080/ready").unwrap();
        assert_eq!(h, "localhost");
        assert_eq!(p, 8080);
        assert_eq!(path, "/ready");
    }

    #[test]
    fn parse_http_url_default_port_and_path() {
        let (h, p, path) = parse_http_url("http://example.com").unwrap();
        assert_eq!(h, "example.com");
        assert_eq!(p, 80);
        assert_eq!(path, "/");
    }

    #[test]
    fn parse_http_url_rejects_https() {
        assert!(parse_http_url("https://example.com").is_none());
    }

    #[test]
    fn short_label_takes_first_token() {
        assert_eq!(short_label("python3 -m http.server 3001"), "python3");
    }

    #[test]
    fn reused_handle_drops_without_child() {
        // Constructing + dropping a "reused" handle is a no-op; used by the
        // reuse_existing=true branch.
        drop(WebServerHandle::reused("none".into()));
    }
}
// CODEGEN-END
