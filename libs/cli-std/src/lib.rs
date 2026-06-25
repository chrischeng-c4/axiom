//! `cclab-cli-std` — the standard agent-facing CLI commands every axiom tool
//! ships, per the convention in `CONTRIBUTING.md` ("every CLI ships `llm`,
//! `upgrade`, `report-issue`"):
//!
//! - [`llm`] — offline self-documentation (how do I drive this?)
//! - [`upgrade`] — self-update from the tool's GitHub releases (am I current?)
//! - [`report_issue`] — file a diagnostics-rich GitHub issue (it's broken — how
//!   do I file it?)
//!
//! The logic is parameterized by a [`ToolInfo`] the calling binary fills from
//! its own build stamps, and is **clap-agnostic**: each CLI keeps its own clap
//! registration (derive or builder) and calls these `run` functions. The
//! network paths (`upgrade` install, `report-issue` submit) live behind the
//! `online` feature; the offline paths (`llm`, `upgrade --check` messaging,
//! `report-issue --dry-run` / pre-filled-URL fallback) always build.

pub mod llm;
pub mod report_issue;
pub mod upgrade;

/// Identity + build provenance of the calling binary. Construct it once in the
/// binary (filling the fields from `env!`/build-script stamps) and pass it to
/// the `run` functions.
///
/// In a real binary the stamps come from `env!`/build-script values; here they
/// are literals so the example compiles standalone:
///
/// ```
/// const TOOL: cclab_cli_std::ToolInfo = cclab_cli_std::ToolInfo {
///     project: "lumen",                      // env-free; the tool name
///     repo: "chrischeng-c4/axiom",
///     target: "aarch64-apple-darwin",        // env!("LUMEN_TARGET") in lumen
///     version: env!("CARGO_PKG_VERSION"),
///     git_sha: "unknown",                    // env!("LUMEN_GIT_SHA") in lumen
///     built_at: "unknown",                   // env!("LUMEN_BUILT_AT") in lumen
/// };
/// assert_eq!(TOOL.tag_prefix(), "lumen@");
/// assert_eq!(TOOL.asset_name(), "lumen-aarch64-apple-darwin.tar.gz");
/// ```
#[derive(Clone, Copy, Debug)]
pub struct ToolInfo {
    /// Short tool name — also the release-tag prefix (`<project>@X.Y.Z`), the
    /// asset stem (`<project>-<target>.tar.gz`) and the inner binary name.
    pub project: &'static str,
    /// GitHub `owner/name` that owns releases + the issue tracker.
    pub repo: &'static str,
    /// The exact target triple this binary was built for (build-script stamp).
    pub target: &'static str,
    /// This binary's version (`env!("CARGO_PKG_VERSION")`).
    pub version: &'static str,
    /// Short git sha stamped at build time (or "unknown").
    pub git_sha: &'static str,
    /// Build timestamp stamped at build time (or "unknown").
    pub built_at: &'static str,
}

impl ToolInfo {
    /// Release-tag prefix, e.g. `lumen@`.
    pub fn tag_prefix(&self) -> String {
        format!("{}@", self.project)
    }

    /// Release-asset filename, e.g. `lumen-aarch64-apple-darwin.tar.gz`.
    pub fn asset_name(&self) -> String {
        format!("{}-{}.tar.gz", self.project, self.target)
    }

    /// Path of the binary inside the release tarball, e.g.
    /// `lumen-aarch64-apple-darwin/lumen`.
    pub fn inner_binary_path(&self) -> String {
        format!("{}-{}/{}", self.project, self.target, self.project)
    }
}

// ---------------------------------------------------------------------------
// Shared helpers for the `online` (network + self-install) paths.
// ---------------------------------------------------------------------------

#[cfg(feature = "online")]
pub(crate) async fn github_get(
    client: &reqwest::Client,
    url: &str,
) -> anyhow::Result<reqwest::Response> {
    use anyhow::Context;
    let mut req = client
        .get(url)
        .header("Accept", "application/vnd.github+json");
    if let Ok(token) = std::env::var("GITHUB_TOKEN") {
        if !token.is_empty() {
            req = req.bearer_auth(token);
        }
    }
    req.send()
        .await
        .with_context(|| format!("GET {url}"))?
        .error_for_status()
        .with_context(|| format!("GitHub API error for {url}"))
}

#[cfg(feature = "online")]
pub(crate) async fn download_bytes(client: &reqwest::Client, url: &str) -> anyhow::Result<Vec<u8>> {
    use anyhow::Context;
    let resp = client
        .get(url)
        .send()
        .await
        .with_context(|| format!("download {url}"))?
        .error_for_status()
        .with_context(|| format!("download {url}"))?;
    Ok(resp.bytes().await.context("read download body")?.to_vec())
}

#[cfg(feature = "online")]
pub(crate) async fn download_text(client: &reqwest::Client, url: &str) -> anyhow::Result<String> {
    use anyhow::Context;
    let resp = client
        .get(url)
        .send()
        .await
        .with_context(|| format!("download {url}"))?
        .error_for_status()
        .with_context(|| format!("download {url}"))?;
    resp.text().await.context("read download body")
}

/// Prompt on an interactive terminal; non-interactive sessions return `true`
/// (callers gate on `--yes` first).
#[cfg(feature = "online")]
pub(crate) fn confirm(prompt: &str) -> anyhow::Result<bool> {
    use anyhow::Context;
    use std::io::{IsTerminal, Write};
    if !std::io::stdin().is_terminal() {
        return Ok(true);
    }
    print!("{prompt} [y/N] ");
    std::io::stdout().flush().ok();
    let mut line = String::new();
    std::io::stdin()
        .read_line(&mut line)
        .context("read confirmation")?;
    Ok(matches!(line.trim(), "y" | "Y" | "yes" | "Yes"))
}

/// Atomically replace the running executable: write a sibling temp file (same
/// dir ⇒ same filesystem), make it executable, then `rename` over self. A
/// permission failure leaves the existing binary intact.
#[cfg(feature = "online")]
pub(crate) fn install_over_self(bin: &[u8], tmp_label: &str) -> anyhow::Result<()> {
    use anyhow::{anyhow, Context};
    let exe = std::env::current_exe().context("locate current executable")?;
    let exe = exe.canonicalize().unwrap_or(exe);
    let dir = exe
        .parent()
        .ok_or_else(|| anyhow!("current executable has no parent directory"))?;
    let tmp = dir.join(format!(".{tmp_label}-{}.tmp", std::process::id()));

    let write = || -> std::io::Result<()> {
        std::fs::write(&tmp, bin)?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&tmp, std::fs::Permissions::from_mode(0o755))?;
        }
        Ok(())
    };
    if let Err(e) = write() {
        let _ = std::fs::remove_file(&tmp);
        return Err(install_error(e, &exe));
    }
    if let Err(e) = std::fs::rename(&tmp, &exe) {
        let _ = std::fs::remove_file(&tmp);
        return Err(install_error(e, &exe));
    }
    Ok(())
}

#[cfg(feature = "online")]
fn install_error(e: std::io::Error, exe: &std::path::Path) -> anyhow::Error {
    if e.kind() == std::io::ErrorKind::PermissionDenied {
        anyhow::anyhow!(
            "cannot replace {}: permission denied. Re-run with elevated permissions (e.g. sudo) or reinstall manually.",
            exe.display()
        )
    } else {
        anyhow::anyhow!("failed to install new binary at {}: {e}", exe.display())
    }
}
