// SPEC-MANAGED: .aw/tech-design/projects/jet/semantic/jet-browser.md#schema
// CODEGEN-BEGIN
//! Browser process launcher — finds and starts Chrome/Chromium with CDP enabled.

use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
use tokio::process::{Child, Command};

const WS_ENDPOINT_PROBE_ATTEMPTS: usize = 200;
const WS_ENDPOINT_PROBE_INTERVAL_MS: u64 = 100;

/// Options for launching a browser.
/// @spec .aw/tech-design/projects/jet/semantic/jet-browser.md#schema
#[derive(Debug, Clone)]
pub struct LaunchOptions {
    /// Path to Chrome/Chromium binary. Auto-detected if `None`.
    pub executable: Option<PathBuf>,
    /// Whether to run headless.
    pub headless: bool,
    /// CDP debugging port. 0 = pick a free port.
    pub port: u16,
    /// Extra CLI args to pass to the browser.
    pub args: Vec<String>,
    /// User data directory. Temp dir if `None`.
    pub user_data_dir: Option<PathBuf>,
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-browser.md#schema
impl Default for LaunchOptions {
    fn default() -> Self {
        Self {
            executable: None,
            headless: true,
            port: 0,
            args: Vec::new(),
            user_data_dir: None,
        }
    }
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-browser.md#schema
pub struct BrowserLauncher;

/// @spec .aw/tech-design/projects/jet/semantic/jet-browser.md#schema
impl BrowserLauncher {
    /// Launch a browser process and return the child + CDP WebSocket URL.
    pub async fn launch(options: &LaunchOptions) -> Result<(Child, String)> {
        let executable = match &options.executable {
            Some(path) => path.clone(),
            None => Self::find_chrome()?,
        };

        let port = if options.port == 0 {
            Self::find_free_port()?
        } else {
            options.port
        };

        let temp_dir = if options.user_data_dir.is_none() {
            Some(tempfile::tempdir().context("Failed to create temp user data dir")?)
        } else {
            None
        };

        let user_data_dir = options
            .user_data_dir
            .clone()
            .unwrap_or_else(|| temp_dir.as_ref().unwrap().path().to_path_buf());

        let mut cmd = Command::new(&executable);
        cmd.arg(format!("--remote-debugging-port={}", port))
            .arg("--remote-debugging-address=127.0.0.1")
            .arg(format!("--user-data-dir={}", user_data_dir.display()))
            .arg("--no-first-run")
            .arg("--no-default-browser-check")
            .arg("--disable-background-networking")
            .arg("--disable-default-apps")
            .arg("--disable-dev-shm-usage")
            .arg("--disable-extensions")
            .arg("--disable-sync")
            .arg("--disable-translate")
            .arg("--metrics-recording-only")
            .arg("--mute-audio")
            .arg("--no-sandbox");

        if options.headless {
            cmd.arg("--headless=new");
        }

        for arg in &options.args {
            cmd.arg(arg);
        }

        // Suppress stdout/stderr from the browser process.
        cmd.stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::piped());

        let child = cmd.spawn().context("Failed to launch browser")?;

        // Wait for the CDP endpoint to become available.
        let ws_url = Self::wait_for_ws_endpoint(port).await?;

        // If we created a temp dir, leak it so it lives as long as the process.
        // The OS will clean it up when the process exits.
        if let Some(dir) = temp_dir {
            std::mem::forget(dir);
        }

        Ok((child, ws_url))
    }

    /// Detect Chrome/Chromium on the current platform.
    ///
    /// Search order:
    /// 1. `~/.jet/browsers/chromium-*/` cache (newest revision first).
    /// 2. System-installed Chrome/Chromium candidates.
    // @spec .aw/changes/enhancement-cclab-jet-browser-install-cli-pinned-chromium-down/specs/enhancement-cclab-jet-browser-install-cli-pinned-chromium-down-spec.md#R5
    fn find_chrome() -> Result<PathBuf> {
        // 1. Prefer cache — use ~/.jet/browsers/ as default cache root.
        if let Some(home) = dirs::home_dir() {
            let cache_root = home.join(".jet").join("browsers");
            if let Some(cached) = Self::find_chrome_in(&cache_root) {
                return Ok(cached);
            }
        }

        // 2. Fall back to system-installed browsers.
        Self::find_chrome_system()
    }

    /// Find a system-installed Chrome/Chromium binary without consulting
    /// Jet's pinned Chromium cache. Human visual review uses this when the
    /// operator asks for the locally installed Chrome experience.
    pub fn find_system_chrome() -> Result<PathBuf> {
        Self::find_chrome_system()
    }

    /// Scan `<cache_root>/chromium-{digits}/` for a valid cached binary.
    /// Returns the binary from the highest (newest) revision number.
    ///
    /// Public test seam used by T2 (`find_chrome_prefers_cache`).
    // @spec .aw/changes/enhancement-cclab-jet-browser-install-cli-pinned-chromium-down/specs/enhancement-cclab-jet-browser-install-cli-pinned-chromium-down-spec.md#R5
    #[doc(hidden)]
    pub fn find_chrome_in(cache_root: &Path) -> Option<PathBuf> {
        // Determine the platform-specific binary subpath under each chromium-<rev>/ dir.
        let binary_subpath: &str = if cfg!(target_os = "macos") {
            "chrome-mac/Chromium.app/Contents/MacOS/Chromium"
        } else if cfg!(target_os = "linux") {
            "chrome-linux/chrome"
        } else {
            "chrome-win/chrome.exe"
        };

        let read_dir = match std::fs::read_dir(cache_root) {
            Ok(rd) => rd,
            // Legitimate first run — cache hasn't been populated.
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => return None,
            Err(err) => {
                tracing::warn!(
                    target: "jet::browser::launcher",
                    cache_root = %cache_root.display(),
                    error = %err,
                    "GH #3328 chromium cache directory exists but is \
                     unreadable; falling back to system Chrome. Check \
                     permissions on this directory if you expected jet to \
                     use a primed cache."
                );
                return None;
            }
        };

        // Collect entries matching `chromium-{digits}`, extract numeric revision.
        // GH #3520 — surface per-entry readdir failures instead of dropping
        // them silently. When the newest `chromium-NNNN` directory is the
        // unreadable one (EACCES, stale NFS handle, FUSE i/o), the prior
        // `.filter_map(|entry| entry.ok())` silently picked an *older*
        // revision and the dev ran e2e tests against a stale Chromium
        // build with no log breadcrumb.
        let mut entries: Vec<(u64, PathBuf)> = read_dir
            .filter_map(|res| match res {
                Ok(entry) => Some(entry),
                Err(err) => {
                    tracing::warn!(
                        target: "jet::browser::launcher",
                        cache_root = %cache_root.display(),
                        error = %err,
                        "{}",
                        format_chromium_entry_warn(cache_root, &err)
                    );
                    None
                }
            })
            .filter_map(|entry| {
                let name = entry.file_name();
                let name_str = name.to_string_lossy();
                let rev_str = name_str.strip_prefix("chromium-")?;
                // Only accept pure-digit revision suffixes.
                if !rev_str.chars().all(|c| c.is_ascii_digit()) || rev_str.is_empty() {
                    return None;
                }
                let rev: u64 = rev_str.parse().ok()?;
                Some((rev, entry.path()))
            })
            .collect();

        // Sort descending by revision number (newest first).
        entries.sort_by(|a, b| b.0.cmp(&a.0));

        // Return the first entry that contains a valid, executable binary.
        for (_rev, dir) in entries {
            let bin = dir.join(binary_subpath);
            if is_executable(&bin) {
                return Some(bin);
            }
        }

        None
    }

    /// System-path fallback (original find_chrome logic).
    fn find_chrome_system() -> Result<PathBuf> {
        let candidates = if cfg!(target_os = "macos") {
            vec![
                "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome",
                "/Applications/Chromium.app/Contents/MacOS/Chromium",
                "/Applications/Google Chrome Canary.app/Contents/MacOS/Google Chrome Canary",
            ]
        } else if cfg!(target_os = "linux") {
            vec![
                "google-chrome",
                "google-chrome-stable",
                "chromium-browser",
                "chromium",
            ]
        } else {
            vec![
                r"C:\Program Files\Google\Chrome\Application\chrome.exe",
                r"C:\Program Files (x86)\Google\Chrome\Application\chrome.exe",
            ]
        };

        for candidate in &candidates {
            let path = PathBuf::from(candidate);
            if path.exists() {
                return Ok(path);
            }
            // For Linux, check if it's in PATH.
            if cfg!(target_os = "linux") {
                if let Ok(output) = std::process::Command::new("which").arg(candidate).output() {
                    if output.status.success() {
                        let p = String::from_utf8_lossy(&output.stdout).trim().to_string();
                        return Ok(PathBuf::from(p));
                    }
                }
            }
        }

        anyhow::bail!(
            "Chrome/Chromium not found. Install Chrome or set the executable path explicitly."
        )
    }

    /// Find a free TCP port.
    fn find_free_port() -> Result<u16> {
        let listener = std::net::TcpListener::bind("127.0.0.1:0")?;
        Ok(listener.local_addr()?.port())
    }

    /// Poll the CDP /json/version endpoint until the browser is ready.
    async fn wait_for_ws_endpoint(port: u16) -> Result<String> {
        let urls = [
            format!("http://127.0.0.1:{}/json/version", port),
            format!("http://localhost:{}/json/version", port),
        ];
        let client = reqwest::Client::new();

        // GH #3727 — the prior three nested `if let Ok(...)` collapsed
        // every probe failure (connection refused, malformed JSON,
        // missing `webSocketDebuggerUrl` field) into the same opaque
        // "Timed out" bail. Track the last symptom so the bail
        // message carries a breadcrumb to the actual failure.
        let mut last_symptom: Option<String> = None;
        for _ in 0..WS_ENDPOINT_PROBE_ATTEMPTS {
            tokio::time::sleep(std::time::Duration::from_millis(
                WS_ENDPOINT_PROBE_INTERVAL_MS,
            ))
            .await;
            for url in &urls {
                match client.get(url).send().await {
                    Err(err) => {
                        last_symptom = Some(format!("connect: {url}: {err}"));
                    }
                    Ok(resp) => match resp.json::<serde_json::Value>().await {
                        Err(err) => {
                            last_symptom = Some(format!("parse json: {url}: {err}"));
                        }
                        Ok(json) => match json["webSocketDebuggerUrl"].as_str() {
                            Some(ws) => return Ok(ws.to_string()),
                            None => {
                                last_symptom = Some(format!(
                                    "no `webSocketDebuggerUrl` field in {url} response"
                                ));
                            }
                        },
                    },
                }
            }
        }

        anyhow::bail!(
            "{}",
            format_browser_ws_timeout_err(port, last_symptom.as_deref())
        )
    }
}

/// GH #3727 — build the timeout-bail wording for `wait_for_ws_endpoint`
/// so the operator gets the last observed probe symptom (connect /
/// JSON-parse / missing field) instead of an opaque "Timed out". The
/// prior three nested `if let Ok(...)` swallowed every failure mode
/// into the same message; this carries the last layer that didn't
/// reach success.
/// @spec .aw/tech-design/projects/jet/semantic/jet-browser.md#schema
pub(crate) fn format_browser_ws_timeout_err(port: u16, last_symptom: Option<&str>) -> String {
    match last_symptom {
        Some(s) => format!(
            "GH #3727 jet browser launcher: timed out waiting for Chrome \
             CDP endpoint on port {port}; last probe symptom: {s}. \
             Earlier this site collapsed connection-refused, malformed \
             JSON, and missing `webSocketDebuggerUrl` into the same \
             generic timeout — the symptom above tells you which layer \
             failed last. `connect:` means Chrome never listened (binary \
             crashed, port collision, sandbox blocking loopback). \
             `parse json:` means something other than Chrome answered \
             (corporate proxy, captive portal, Chromium fork). `no \
             webSocketDebuggerUrl` means a Chromium variant whose \
             /json/version schema differs from upstream."
        ),
        None => format!(
            "GH #3727 jet browser launcher: timed out waiting for Chrome \
             CDP endpoint on port {port}; no probe symptom captured — \
             every iteration completed without recording a failure, \
             which is internally inconsistent. Likely a tokio runtime \
             issue or a panic in the request future. Re-run with \
             RUST_LOG=jet::browser=trace to capture more detail."
        ),
    }
}

/// GH #3520 — build the warn message for a per-entry readdir failure
/// in the chromium cache. Extracted so the wording is unit-testable
/// without provoking a real OS-level readdir mid-iteration error
/// (which is platform-specific and effectively non-deterministic).
/// @spec .aw/tech-design/projects/jet/semantic/jet-browser.md#schema
pub(crate) fn format_chromium_entry_warn(cache_root: &Path, err: &std::io::Error) -> String {
    format!(
        "GH #3520 chromium cache entry unreadable under {}: {err}; \
         a chromium-<rev> directory was skipped while scanning the \
         cache. If the *newest* revision is the one skipped, jet \
         will launch an older Chromium build with no diagnostic. \
         Check permissions / NFS / FUSE mount state for the cache \
         directory.",
        cache_root.display()
    )
}

/// Return `true` if `path` exists and is a regular file with an executable bit set (Unix),
/// or simply exists on non-Unix platforms.
fn is_executable(path: &Path) -> bool {
    if !path.exists() {
        return false;
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if let Ok(meta) = std::fs::metadata(path) {
            return meta.is_file() && (meta.permissions().mode() & 0o111 != 0);
        }
        return false;
    }
    #[cfg(not(unix))]
    {
        path.is_file()
    }
}

#[cfg(test)]
mod gh3328_tests {
    use super::*;
    use tempfile::tempdir;

    /// GH #3328 — happy path: a populated cache with a valid chromium-<rev>/
    /// directory yields a result (the directory is picked even if the binary
    /// doesn't exist — find_chrome_in checks for an executable per-entry, so
    /// the result is None unless a real binary is laid down; the contract
    /// here is "no panic + correct sort").
    #[test]
    fn gh3328_find_chrome_in_populated_cache_no_executable_yields_none() {
        let dir = tempdir().unwrap();
        let cache = dir.path();
        std::fs::create_dir_all(cache.join("chromium-100")).unwrap();
        std::fs::create_dir_all(cache.join("chromium-200")).unwrap();
        // No actual binary placed — is_executable returns false → None.
        let result = BrowserLauncher::find_chrome_in(cache);
        assert!(
            result.is_none(),
            "no executable in any chromium-<rev>/ → None: {:?}",
            result
        );
    }

    /// GH #3328 — legitimate missing cache root returns None silently.
    #[test]
    fn gh3328_find_chrome_in_missing_cache_root_silent_none() {
        let dir = tempdir().unwrap();
        let cache = dir.path().join("never-created");
        // path does not exist
        let result = BrowserLauncher::find_chrome_in(&cache);
        assert!(
            result.is_none(),
            "missing cache root must yield None silently"
        );
    }

    /// GH #3328 — cache root exists but is unreadable (chmod 000): must
    /// still return None (preserved behavior) AND emit a warn so the
    /// operator can chase the permissions trouble. We verify None here.
    #[cfg(unix)]
    #[test]
    fn gh3328_find_chrome_in_unreadable_cache_root_returns_none_with_warn() {
        use std::os::unix::fs::PermissionsExt;

        let dir = tempdir().unwrap();
        let cache = dir.path().join("locked-cache");
        std::fs::create_dir_all(&cache).unwrap();
        std::fs::create_dir_all(cache.join("chromium-1234567")).unwrap();
        // Chmod 0o000 removes read+exec on the dir → read_dir fails with EACCES.
        std::fs::set_permissions(&cache, std::fs::Permissions::from_mode(0o000)).unwrap();

        // Root may still be able to read — skip if so.
        if std::fs::read_dir(&cache).is_ok() {
            let _ = std::fs::set_permissions(&cache, std::fs::Permissions::from_mode(0o755));
            return;
        }

        let result = BrowserLauncher::find_chrome_in(&cache);

        // Restore perms so tempdir cleanup works.
        let _ = std::fs::set_permissions(&cache, std::fs::Permissions::from_mode(0o755));

        assert!(
            result.is_none(),
            "unreadable cache root must yield None (preserved behavior): {:?}",
            result
        );
    }
}

#[cfg(test)]
mod gh3520_tests {
    use super::*;

    /// GH #3520 — the per-entry warn message must name the cache root,
    /// name the underlying error verbatim, AND include the GH #3520
    /// tag so the dev has a searchable breadcrumb when their teammate
    /// chmod'd a chromium-NNNN directory and launch silently picked an
    /// older revision.
    #[test]
    fn gh3520_chromium_entry_warn_names_root_error_and_issue() {
        let cache = Path::new("/home/dev/.jet/browsers");
        let err = std::io::Error::new(
            std::io::ErrorKind::PermissionDenied,
            "Permission denied (os error 13)",
        );
        let msg = format_chromium_entry_warn(cache, &err);

        assert!(
            msg.contains("/home/dev/.jet/browsers"),
            "must name cache root, got: {msg}"
        );
        assert!(
            msg.contains("Permission denied"),
            "must preserve underlying error verbatim, got: {msg}"
        );
        assert!(
            msg.contains("GH #3520"),
            "must include searchable issue tag, got: {msg}"
        );
        assert!(
            msg.contains("older Chromium"),
            "must explain dev-visible symptom (stale revision), got: {msg}"
        );
    }

    /// GH #3520 — the message must point the operator at the typical
    /// fix surfaces (permissions, NFS, FUSE) so they don't waste time
    /// chasing the chromium binary itself.
    #[test]
    fn gh3520_chromium_entry_warn_hints_fix_surfaces() {
        let cache = Path::new("/tmp/cache");
        let err = std::io::Error::other("stale NFS file handle");
        let msg = format_chromium_entry_warn(cache, &err);

        assert!(
            msg.contains("permissions") || msg.contains("NFS") || msg.contains("FUSE"),
            "must hint at typical fix surfaces, got: {msg}"
        );
    }
}

#[cfg(test)]
mod gh3727_browser_ws_timeout_err_tests {
    //! GH #3727 — `wait_for_ws_endpoint` collapsed three layers of
    //! probe failures (connect, parse json, missing field) into the
    //! same opaque "Timed out" bail. `format_browser_ws_timeout_err`
    //! carries the last observed symptom so the operator can pick the
    //! right next step.
    use super::*;

    #[test]
    fn helper_tags_gh_issue() {
        let msg = format_browser_ws_timeout_err(9222, Some("connect: refused"));
        assert!(msg.contains("GH #3727"), "must carry issue tag, got: {msg}");
    }

    #[test]
    fn ws_probe_budget_covers_slow_ci_chrome_startup() {
        let budget_ms = WS_ENDPOINT_PROBE_ATTEMPTS as u64 * WS_ENDPOINT_PROBE_INTERVAL_MS;
        assert!(
            budget_ms >= 20_000,
            "GitHub Actions cold Chrome startup exceeded the old 5s budget"
        );
    }

    #[test]
    fn helper_names_port() {
        let msg = format_browser_ws_timeout_err(9333, Some("connect: refused"));
        assert!(msg.contains("9333"), "must name port, got: {msg}");
    }

    #[test]
    fn helper_forwards_last_symptom_text() {
        let msg = format_browser_ws_timeout_err(
            9222,
            Some("connect: tcp connect error: Connection refused (os error 61)"),
        );
        assert!(
            msg.contains("Connection refused"),
            "must forward last symptom, got: {msg}"
        );
        assert!(
            msg.contains("connect:"),
            "must keep the layer prefix, got: {msg}"
        );
    }

    #[test]
    fn helper_distinguishes_connect_vs_parse_vs_missing_field_layers() {
        let msg = format_browser_ws_timeout_err(9222, Some("connect: refused"));
        // The wording must explain what each layer prefix means so
        // an operator can react. Connect/parse/missing-field map to
        // very different remediation steps.
        assert!(
            msg.contains("connect:") && msg.contains("never listened"),
            "must explain connect symptom: {msg}"
        );
        assert!(
            msg.contains("parse json:") && msg.contains("proxy"),
            "must explain parse-json symptom & corporate-proxy cause: {msg}"
        );
        assert!(
            msg.contains("webSocketDebuggerUrl") && msg.contains("schema"),
            "must explain missing-field symptom & schema cause: {msg}"
        );
    }

    #[test]
    fn helper_handles_no_symptom_internal_inconsistency_branch() {
        let msg = format_browser_ws_timeout_err(9222, None);
        assert!(msg.contains("GH #3727"));
        assert!(msg.contains("9222"));
        assert!(
            msg.contains("no probe symptom") || msg.contains("inconsistent"),
            "must call out the internally-inconsistent branch: {msg}"
        );
        assert!(
            msg.contains("RUST_LOG"),
            "must tell operator how to capture more detail: {msg}"
        );
    }

    #[test]
    fn helper_no_symptom_branch_distinct_from_with_symptom_branch() {
        let with = format_browser_ws_timeout_err(9222, Some("connect: refused"));
        let without = format_browser_ws_timeout_err(9222, None);
        assert_ne!(with, without);
    }

    #[test]
    fn helper_is_deterministic_for_fixed_input() {
        let a = format_browser_ws_timeout_err(9222, Some("connect: refused"));
        let b = format_browser_ws_timeout_err(9222, Some("connect: refused"));
        assert_eq!(a, b);
    }

    #[test]
    fn helper_distinct_from_chromium_entry_warn_3520() {
        // Sibling check: distinguishable from the readdir warn so
        // operators grepping for the right GH issue don't conflate them.
        let ws = format_browser_ws_timeout_err(9222, Some("connect: refused"));
        let entry = format_chromium_entry_warn(
            Path::new("/tmp/cache"),
            &std::io::Error::other("stale handle"),
        );
        assert_ne!(ws, entry);
        assert!(!ws.contains("GH #3520"), "must not carry sibling tag: {ws}");
        assert!(
            !entry.contains("GH #3727"),
            "sibling must not carry our tag: {entry}"
        );
    }
}
// CODEGEN-END
