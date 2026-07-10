---
id: libs-cli-std-src-lib-rs
summary: Lossless rust-source-unit coverage for `libs/cli-std/src/lib.rs`.
capability_refs:
  - id: standard-agent-cli-commands
    role: primary
    claim: standard-agent-cli-commands-contract
    coverage: full
    rationale: "The source, tests, and manifest implement the Cli Std library contract."
fill_sections: [overview, source, changes]
---

# Standardized libs/cli-std/src/lib.rs

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `libs/cli-std/src/lib.rs` captured during libs codegen standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `chainable` | libs/cli-std/src/lib.rs | module | pub | 31 | pub mod chainable; |
| `connect` | libs/cli-std/src/lib.rs | module | pub | 33 | pub mod connect; (feature `k8s`) |
| `issue` | libs/cli-std/src/lib.rs | module | pub | 34 | pub mod issue; |
| `llm` | libs/cli-std/src/lib.rs | module | pub | 35 | pub mod llm; |
| `report_issue` | libs/cli-std/src/lib.rs | module | pub | 37 | pub mod report_issue; |
| `upgrade` | libs/cli-std/src/lib.rs | module | pub | 38 | pub mod upgrade; |
| `ToolInfo` | libs/cli-std/src/lib.rs | struct | pub | 61 | pub struct ToolInfo { |
| `tag_prefix` | libs/cli-std/src/lib.rs | function | pub | 85 | pub fn tag_prefix(&self) -> String { |
| `asset_name` | libs/cli-std/src/lib.rs | function | pub | 90 | pub fn asset_name(&self) -> String { |
| `inner_binary_path` | libs/cli-std/src/lib.rs | function | pub | 96 | pub fn inner_binary_path(&self) -> String { |
| `resolve_github_token` | libs/cli-std/src/lib.rs | function | pub | 112 | pub(crate) fn resolve_github_token() -> Option<String> { |
| `github_get` | libs/cli-std/src/lib.rs | function | pub | 151 | pub(crate) async fn github_get( |
| `download_bytes` | libs/cli-std/src/lib.rs | function | pub | 170 | pub(crate) async fn download_bytes(client: &reqwest::Client, url: &str) -> anyhow::Result<Vec<u8>> { |
| `download_text` | libs/cli-std/src/lib.rs | function | pub | 183 | pub(crate) async fn download_text(client: &reqwest::Client, url: &str) -> anyhow::Result<String> { |
| `confirm` | libs/cli-std/src/lib.rs | function | pub | 199 | pub(crate) fn confirm(prompt: &str) -> anyhow::Result<bool> { |
| `install_over_self` | libs/cli-std/src/lib.rs | function | pub | 219 | pub(crate) fn install_over_self(bin: &[u8], tmp_label: &str) -> anyhow::Result<()> { |


## Source
<!-- type: rust-source-unit lang: rust -->

````rust
// SPEC-MANAGED: libs/cli-std/tech-design/semantic/source/libs-cli-std-src-lib-rs.md#rust-source-unit
// CODEGEN-BEGIN
//! `cli-std` — the standard agent-facing CLI commands every axiom tool
//! ships, per the convention in `CONTRIBUTING.md` ("every CLI ships `llm`,
//! `upgrade`, `issue`"):
//!
//! - [`llm`] — offline self-documentation (how do I drive this?)
//! - [`upgrade`] — self-update from the tool's GitHub releases (am I current?)
//! - [`issue`] — search, view, file, and comment on diagnostics-rich GitHub
//!   issues; comments automatically reopen issues for post-closure verification
//!   failures.
//!
//! The logic is parameterized by a [`ToolInfo`] the calling binary fills from
//! its own build stamps, and is **clap-agnostic**: each CLI keeps its own clap
//! registration (derive or builder) and calls these `run` functions. The
//! network paths (`upgrade` install, `issue` search/view/create/comment`) live behind the
//! `online` feature; the offline paths (`llm`, `upgrade --check` messaging,
//! `issue create --dry-run` / pre-filled-URL fallback, `issue comment --dry-run`
//! / manual-comment fallback) always build.
//!
//! One more module rides along that is not a CLI subcommand: [`chainable`] —
//! a test harness (not a `run` function a binary calls) backing the
//! `chainable_output` archetype trait's gate: `CONTRIBUTING.md` § "CLI
//! convention: stdout tells the agent the next step".
//!
//! A fourth verb, [`connect`], rides behind the `k8s` feature: the
//! port-forward lifecycle + token-registry Secret resolution every
//! k8s-native service CLI's `<cli> connect` wants (extracted from `lumen
//! connect`, #1321/#1376 — see `CONTRIBUTING.md` § "Deploy artifacts").

pub mod chainable;
#[cfg(feature = "k8s")]
pub mod connect;
pub mod issue;
pub mod llm;
/// Deprecated alias of [`issue`] — kept until keep/loom/lumen adopt `issue`.
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
/// const TOOL: cli_std::ToolInfo = cli_std::ToolInfo {
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
/// @spec libs/cli-std/tech-design/semantic/source/libs-cli-std-src-lib-rs.md#source
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

/// @spec libs/cli-std/tech-design/semantic/source/libs-cli-std-src-lib-rs.md#source
impl ToolInfo {
    /// Default tracker label for this tool's issue surface.
    pub fn issue_label(&self) -> String {
        format!("app:{}", self.project)
    }

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

/// Resolve a GitHub token the way `gh` itself does, in order: `$GH_TOKEN`,
/// then `$GITHUB_TOKEN`, then the `gh` CLI credential store (`gh auth token`,
/// which reads the OS keyring / `hosts.yml`). Returns `None` when no credential
/// is available. This makes the standard CLI ops "just work" for anyone already
/// authenticated via `gh`, which does not export a `GITHUB_TOKEN` env var.
#[cfg(feature = "online")]
/// @spec libs/cli-std/tech-design/semantic/source/libs-cli-std-src-lib-rs.md#source
pub(crate) fn resolve_github_token() -> Option<String> {
    resolve_github_token_from(|var| std::env::var(var).ok(), gh_auth_token)
}

/// Pure resolution order (env lookup + `gh` fallback injected for testing):
/// first non-empty of `$GH_TOKEN`, `$GITHUB_TOKEN`, then `gh()`.
#[cfg(feature = "online")]
fn resolve_github_token_from(
    env: impl Fn(&str) -> Option<String>,
    gh: impl Fn() -> Option<String>,
) -> Option<String> {
    for var in ["GH_TOKEN", "GITHUB_TOKEN"] {
        if let Some(token) = env(var) {
            let token = token.trim().to_string();
            if !token.is_empty() {
                return Some(token);
            }
        }
    }
    gh()
}

/// Shell out to `gh auth token` — the de-facto GitHub auth on developer
/// machines (token in the OS keyring). Returns `None` if `gh` is missing,
/// unauthenticated, or prints nothing.
#[cfg(feature = "online")]
fn gh_auth_token() -> Option<String> {
    let output = std::process::Command::new("gh")
        .args(["auth", "token"])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let token = String::from_utf8_lossy(&output.stdout).trim().to_string();
    (!token.is_empty()).then_some(token)
}

#[cfg(feature = "online")]
pub(crate) async fn github_get(
    client: &reqwest::Client,
    url: &str,
) -> anyhow::Result<reqwest::Response> {
    use anyhow::Context;
    let mut req = client
        .get(url)
        .header("Accept", "application/vnd.github+json");
    if let Some(token) = resolve_github_token() {
        req = req.bearer_auth(token);
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
/// @spec libs/cli-std/tech-design/semantic/source/libs-cli-std-src-lib-rs.md#source
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
/// @spec libs/cli-std/tech-design/semantic/source/libs-cli-std-src-lib-rs.md#source
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

#[cfg(all(test, feature = "online"))]
mod token_tests {
    use super::resolve_github_token_from;

    #[test]
    fn gh_token_takes_precedence() {
        let got = resolve_github_token_from(
            |v| match v {
                "GH_TOKEN" => Some("from-gh-env".to_string()),
                "GITHUB_TOKEN" => Some("from-github".to_string()),
                _ => None,
            },
            || Some("from-gh-cli".to_string()),
        );
        assert_eq!(got.as_deref(), Some("from-gh-env"));
    }

    #[test]
    fn github_token_is_second() {
        let got = resolve_github_token_from(
            |v| (v == "GITHUB_TOKEN").then(|| "from-github".to_string()),
            || Some("from-gh-cli".to_string()),
        );
        assert_eq!(got.as_deref(), Some("from-github"));
    }

    #[test]
    fn falls_back_to_gh_cli_when_env_absent_or_blank() {
        // Blank env values are skipped, so the gh CLI store is consulted.
        let got = resolve_github_token_from(
            |v| (v == "GH_TOKEN").then(|| "   ".to_string()),
            || Some("from-gh-cli".to_string()),
        );
        assert_eq!(got.as_deref(), Some("from-gh-cli"));
    }

    #[test]
    fn none_when_no_credential_anywhere() {
        let got = resolve_github_token_from(|_| None, || None);
        assert_eq!(got, None);
    }
}
// CODEGEN-END

// SPEC-MANAGED: libs/cli-std/tech-design/interfaces/cli/courier-proxy-mode-client-for-the-issue-triad.md#logic
// HANDWRITE-BEGIN gap="cli-std-logic-flowchart-patch-fn" tracker="#1320" reason="the generic Mermaid-flowchart-to-Rust generator only synthesizes one brand-new function per diagram with todo!() bodies; it cannot target insertions into existing named functions, so resolve_courier_url()/resolve_courier_token() are hand-written to mirror resolve_github_token()'s env-resolution pattern exactly."
/// Resolve the courier proxy URL from `$AXIOM_COURIER_URL`. Returns `None`
/// when unset or blank (mirrors `resolve_github_token()`'s blank-counts-as-
/// unset convention). When `Some`, `issue::{search,view,create,comment}`
/// route through courier's `/v1/issues/...` endpoints instead of calling
/// `api.github.com` directly.
#[cfg(feature = "online")]
/// @spec libs/cli-std/tech-design/interfaces/cli/courier-proxy-mode-client-for-the-issue-triad.md#logic
pub(crate) fn resolve_courier_url() -> Option<String> {
    resolve_courier_url_from(|var| std::env::var(var).ok())
}

/// Pure resolution (env lookup injected for testing): trim `$AXIOM_COURIER_URL`,
/// `None` when unset or blank.
#[cfg(feature = "online")]
fn resolve_courier_url_from(env: impl Fn(&str) -> Option<String>) -> Option<String> {
    env("AXIOM_COURIER_URL").and_then(|v| {
        let v = v.trim().to_string();
        (!v.is_empty()).then_some(v)
    })
}

/// Resolve the courier proxy bearer token from `$AXIOM_COURIER_TOKEN`.
/// Returns `None` when unset or blank. Sent as `Authorization: Bearer
/// <token>` to courier -- this is the courier-issued client credential, not
/// a personal GitHub token.
#[cfg(feature = "online")]
/// @spec libs/cli-std/tech-design/interfaces/cli/courier-proxy-mode-client-for-the-issue-triad.md#logic
pub(crate) fn resolve_courier_token() -> Option<String> {
    resolve_courier_token_from(|var| std::env::var(var).ok())
}

/// Pure resolution (env lookup injected for testing): trim `$AXIOM_COURIER_TOKEN`,
/// `None` when unset or blank.
#[cfg(feature = "online")]
fn resolve_courier_token_from(env: impl Fn(&str) -> Option<String>) -> Option<String> {
    env("AXIOM_COURIER_TOKEN").and_then(|v| {
        let v = v.trim().to_string();
        (!v.is_empty()).then_some(v)
    })
}

#[cfg(all(test, feature = "online"))]
mod courier_tests {
    use super::{resolve_courier_token_from, resolve_courier_url_from};

    #[test]
    fn resolve_courier_url_returns_some_when_env_set() {
        let got = resolve_courier_url_from(|v| {
            (v == "AXIOM_COURIER_URL").then(|| "  https://courier.internal  ".to_string())
        });
        assert_eq!(got.as_deref(), Some("https://courier.internal"));
    }

    #[test]
    fn resolve_courier_url_returns_none_when_env_unset_or_blank() {
        assert_eq!(resolve_courier_url_from(|_| None), None);
        let blank =
            resolve_courier_url_from(|v| (v == "AXIOM_COURIER_URL").then(|| "   ".to_string()));
        assert_eq!(blank, None);
    }

    #[test]
    fn resolve_courier_token_returns_some_when_env_set() {
        let got = resolve_courier_token_from(|v| {
            (v == "AXIOM_COURIER_TOKEN").then(|| " secret-token ".to_string())
        });
        assert_eq!(got.as_deref(), Some("secret-token"));
    }

    #[test]
    fn resolve_courier_token_returns_none_when_env_unset_or_blank() {
        assert_eq!(resolve_courier_token_from(|_| None), None);
        let blank = resolve_courier_token_from(|v| {
            (v == "AXIOM_COURIER_TOKEN").then(|| "".to_string())
        });
        assert_eq!(blank, None);
    }
}
// HANDWRITE-END
````

## Changes
<!-- type: changes lang: yaml -->

```yaml
coverage_kind: semantic
changes:
  - path: "libs/cli-std/src/lib.rs"
    action: modify
    section: rust-source-unit
    impl_mode: codegen
    description: |
      rust-source-unit (td_ast) source for `libs/cli-std/src/lib.rs` captured during libs codegen standardization.
      Updated (#1376) to declare the new `connect` module (feature `k8s`) alongside `chainable`/`issue`/`llm`/`report_issue`/`upgrade`.
```
