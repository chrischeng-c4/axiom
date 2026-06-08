// SPEC-MANAGED: .aw/tech-design/projects/jet/semantic/jet-browser.md#schema
// CODEGEN-BEGIN
//! Browser control via Chrome DevTools Protocol (CDP).
//!
//! Provides a native CDP client for controlling Chromium-based browsers,
//! replacing the need for external tools like Playwright or Puppeteer.
//!
//! # Architecture
//!
//! ```text
//! ┌─────────────┐     WebSocket      ┌──────────────────┐
//! │  CdpClient   │ ◄──────────────► │  Chrome/Chromium   │
//! │  (this mod)  │   CDP JSON-RPC    │  --remote-debug    │
//! └──────┬──────┘                    └──────────────────┘
//!        │
//!   ┌────┴─────────────────────────┐
//!   │  Page    Navigate, evaluate  │
//!   │  Dom     querySelector, etc  │
//!   │  Network Intercept, wait     │
//!   │  Runtime JS evaluation       │
//!   └─────────────────────────────┘
//! ```

pub mod cdp;
pub mod context;
pub mod install;
pub mod launcher;
pub mod locator;
pub mod page;

use anyhow::{Context, Result};

pub use cdp::CdpClient;
pub use context::BrowserContext;
pub use install::{install_chromium, DEFAULT_CHROMIUM_REVISION};
pub use launcher::{BrowserLauncher, LaunchOptions};
pub use locator::{Actionability, Locator, LocatorError, LocatorOptions, SelectorExpr};
pub use page::Page;

/// High-level browser handle. Owns the CDP connection and child process.
/// @spec .aw/tech-design/projects/jet/semantic/jet-browser.md#schema
pub struct Browser {
    client: CdpClient,
    process: Option<tokio::process::Child>,
    ws_url: String,
    /// Implicit default context created at launch / connect time. All pages
    /// opened via `Browser::new_page()` route through this context.
    // @spec .aw/issues/open/enhancement-browsercontext-refactor-multi-context-isolation-fo.md#R2
    default_context: Option<BrowserContext>,
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-browser.md#schema
impl Browser {
    /// Launch a new browser instance and connect via CDP.
    pub async fn launch(options: LaunchOptions) -> Result<Self> {
        let (process, ws_url) = BrowserLauncher::launch(&options).await?;
        let client = CdpClient::connect(&ws_url).await?;
        let default_context = create_browser_context(&client, true).await?;
        Ok(Self {
            client,
            process: Some(process),
            ws_url,
            default_context: Some(default_context),
        })
    }

    /// Connect to an already-running browser at the given CDP WebSocket URL.
    pub async fn connect(ws_url: &str) -> Result<Self> {
        let client = CdpClient::connect(ws_url).await?;
        let default_context = create_browser_context(&client, true).await?;
        Ok(Self {
            client,
            process: None,
            ws_url: ws_url.to_string(),
            default_context: Some(default_context),
        })
    }

    /// Open a new page (tab) in the default context.
    ///
    /// Backward-compatible entry point — delegates to the implicit default
    /// `BrowserContext` so the 50-test Page API parity surface and all
    /// existing callers continue to work without any API changes.
    // @spec .aw/issues/open/enhancement-browsercontext-refactor-multi-context-isolation-fo.md#R2
    pub async fn new_page(&self) -> Result<Page> {
        let ctx = self
            .default_context
            .as_ref()
            .context("Browser has no default context (already closed)")?;
        ctx.new_page().await
    }

    /// Create a new, isolated `BrowserContext`.
    ///
    /// Pages, cookies, and storage state inside the returned context are
    /// isolated from the default context and from every other context
    /// created via this method.
    // @spec .aw/issues/open/enhancement-browsercontext-refactor-multi-context-isolation-fo.md#R3
    pub async fn new_context(&self) -> Result<BrowserContext> {
        create_browser_context(&self.client, false).await
    }

    /// Borrow the implicit default context, if the browser is still open.
    pub fn default_context(&self) -> Option<&BrowserContext> {
        self.default_context.as_ref()
    }

    /// Close the browser. Kills the child process if we launched it.
    pub async fn close(mut self) -> Result<()> {
        // Drop the default context handle first; the actual `browserContextId`
        // is disposed by `Browser.close` below alongside the process exit.
        self.default_context.take();
        self.client
            .send("Browser.close", serde_json::json!({}))
            .await
            .ok();
        if let Some(mut proc) = self.process.take() {
            let _ = try_kill_browser_process(&mut proc).await;
        }
        Ok(())
    }

    /// OS process id for a browser launched by this handle.
    pub fn process_id(&self) -> Option<u32> {
        self.process.as_ref().and_then(|proc| proc.id())
    }

    /// Release ownership of a launched browser process without closing it.
    ///
    /// Used by agent-first CLI commands that launch a browser, persist the CDP
    /// session metadata, then exit while follow-up commands reattach through
    /// the stored WebSocket endpoint.
    pub fn detach(mut self) {
        self.default_context.take();
        if let Some(proc) = self.process.take() {
            std::mem::forget(proc);
        }
    }

    /// The WebSocket URL this browser is connected on.
    pub fn ws_url(&self) -> &str {
        &self.ws_url
    }
}

/// Create a `BrowserContext` via `Target.createBrowserContext`. The
/// `is_default` flag controls whether `BrowserContext::close()` skips
/// `Target.disposeBrowserContext` (default contexts are owned by the
/// `Browser` and disposed on `Browser::close`).
// @spec .aw/issues/open/enhancement-browsercontext-refactor-multi-context-isolation-fo.md#R2
// @spec .aw/issues/open/enhancement-browsercontext-refactor-multi-context-isolation-fo.md#R3
async fn create_browser_context(client: &CdpClient, is_default: bool) -> Result<BrowserContext> {
    let root = client.root_session();
    let res = root
        .send("Target.createBrowserContext", serde_json::json!({}))
        .await?;
    let context_id = res["browserContextId"]
        .as_str()
        .context("Missing browserContextId in createBrowserContext response")?
        .to_string();
    Ok(BrowserContext::new(root, context_id, is_default))
}

/// GH #3488 — attempt the graceful `Browser.close` CDP RPC. On failure, log
/// a structured warn so an operator can chase the underlying transport
/// issue, and propagate the error so the caller knows the graceful path
/// did not run. The caller is expected to still attempt to kill the child
/// process — graceful-degrade, not abort.
/// @spec .aw/tech-design/projects/jet/semantic/jet-browser.md#schema
pub(crate) async fn try_close_browser_rpc(client: &CdpClient) -> Result<()> {
    if let Err(err) = client.send("Browser.close", serde_json::json!({})).await {
        tracing::warn!(
            target: "jet::browser",
            error = %err,
            "GH #3488 Browser.close CDP RPC failed; the browser may not \
             have exited gracefully. The caller will still try to kill the \
             child process. Check for orphaned Chromium processes if the \
             child kill also fails."
        );
        return Err(err);
    }
    Ok(())
}

/// GH #3488 — attempt to kill the browser child process. On failure, log a
/// structured warn so an operator can chase the leaked PID, and propagate
/// the error so the caller can decide whether to escalate. Tokio returns
/// `Ok(())` if the child has already exited, so a real `Err` here means
/// the OS refused the signal (permission denied, ESRCH on a process whose
/// PID has been recycled, etc).
/// @spec .aw/tech-design/projects/jet/semantic/jet-browser.md#schema
pub(crate) async fn try_kill_browser_process(
    proc: &mut tokio::process::Child,
) -> std::io::Result<()> {
    let pid = proc.id();
    if let Err(err) = proc.kill().await {
        tracing::warn!(
            target: "jet::browser",
            pid = ?pid,
            error_kind = ?err.kind(),
            error = %err,
            "GH #3488 failed to kill browser child process; the process is \
             likely leaked. Check `ps` for orphaned Chromium and kill \
             manually. Repeated occurrences point to permission / namespace \
             issues on the host."
        );
        return Err(err);
    }
    Ok(())
}

#[cfg(test)]
mod gh3488_tests {
    use super::*;

    /// Killing a freshly-spawned, still-running child must succeed and the
    /// helper must return `Ok(())`. The post-kill `try_wait` must report
    /// `Some(_)` — i.e. the child actually exited.
    #[cfg(unix)]
    #[tokio::test]
    async fn gh3488_try_kill_browser_process_kills_running_child() {
        let mut child = tokio::process::Command::new("sleep")
            .arg("30")
            .spawn()
            .expect("spawn sleep");

        try_kill_browser_process(&mut child)
            .await
            .expect("kill of a running child must succeed");

        // The kill above also reaps; try_wait must show a final status.
        let status = child.wait().await.expect("wait after kill");
        assert!(
            !status.success(),
            "sleep was SIGKILLed; exit status must be non-success"
        );
    }

    /// Tokio's `Child::kill` is documented as idempotent — calling it on a
    /// child that has already exited returns `Ok(())`. Pin that contract so
    /// `close()` on a Browser whose process exited on its own does not
    /// surface a spurious warn.
    #[cfg(unix)]
    #[tokio::test]
    async fn gh3488_try_kill_browser_process_already_exited_is_idempotent() {
        let mut child = tokio::process::Command::new("true")
            .spawn()
            .expect("spawn true");
        let _ = child.wait().await.expect("wait for true to exit");

        try_kill_browser_process(&mut child)
            .await
            .expect("kill of an already-exited child must be Ok per Tokio contract");
    }
}
// CODEGEN-END
