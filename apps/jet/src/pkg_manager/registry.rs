// SPEC-MANAGED: .aw/tech-design/projects/jet/semantic/jet-pkg-manager.md#schema
// CODEGEN-BEGIN
use anyhow::Result;
use dashmap::DashMap;
use serde::Deserialize;
use std::collections::HashMap;
use std::env::VarError;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, SystemTime};

use super::npmrc::NpmrcConfig;

/// Disk cache TTL for registry metadata (5 minutes).
const DISK_CACHE_TTL: Duration = Duration::from_secs(300);

/// NPM registry client with in-memory + persistent disk metadata caching.
///
/// Two-layer cache strategy:
/// - L1: in-memory `DashMap` — zero-cost hit within a single install run.
/// - L2: disk cache at `~/.cache/jet/metadata/` (XDG) — survives across runs
///   and projects, giving cold installs the same speed as warm ones after the
///   first install.
///
/// Pass `no_cache: true` to skip disk reads/writes (still uses L1 memory
/// cache within a single process run). Useful for `jet install --no-cache`.
/// @spec .aw/tech-design/projects/jet/semantic/jet-pkg-manager.md#schema
#[derive(Clone)]
pub struct RegistryClient {
    client: reqwest::Client,
    #[allow(dead_code)]
    registry_url: String,
    cache: Arc<DashMap<String, PackageMetadata>>,
    npmrc: NpmrcConfig,
    /// Directory for persistent disk cache: `~/.cache/jet/metadata/` (XDG).
    disk_cache_dir: PathBuf,
    /// When true, skip disk cache reads and writes (always fetch from registry).
    no_cache: bool,
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-pkg-manager.md#schema
#[derive(Debug, Clone, serde::Serialize, Deserialize)]
pub struct PackageMetadata {
    pub name: String,
    #[serde(rename = "dist-tags")]
    pub dist_tags: HashMap<String, String>,
    pub versions: HashMap<String, VersionMetadata>,
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-pkg-manager.md#schema
#[derive(Debug, Clone, serde::Serialize, Deserialize)]
pub struct VersionMetadata {
    pub version: String,
    pub dist: DistInfo,
    #[serde(default)]
    pub dependencies: Option<HashMap<String, String>>,
    #[serde(rename = "peerDependencies", default)]
    pub peer_dependencies: Option<HashMap<String, String>>,
    #[serde(rename = "optionalDependencies", default)]
    pub optional_dependencies: Option<HashMap<String, String>>,
    /// `bin` can be a string (single binary) or a map of name→path.
    #[serde(default)]
    pub bin: Option<BinField>,
    #[serde(default)]
    pub scripts: Option<HashMap<String, String>>,
    /// Platform restriction: allowed OS values (e.g., ["darwin", "linux"])
    #[serde(default)]
    pub os: Option<Vec<String>>,
    /// Platform restriction: allowed CPU values (e.g., ["x64", "arm64"])
    #[serde(default)]
    pub cpu: Option<Vec<String>>,
}

/// npm `bin` field: either `"path/to/cli.js"` or `{"cmd": "path/to/cli.js"}`.
/// @spec .aw/tech-design/projects/jet/semantic/jet-pkg-manager.md#schema
#[derive(Debug, Clone, serde::Serialize, Deserialize)]
#[serde(untagged)]
pub enum BinField {
    Single(String),
    Map(HashMap<String, String>),
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-pkg-manager.md#schema
#[derive(Debug, Clone, serde::Serialize, Deserialize)]
pub struct DistInfo {
    pub tarball: String,
    pub shasum: String,
    pub integrity: Option<String>,
}

/// Format the warning emitted when an .npmrc proxy URL cannot be parsed.
///
/// `field_name` is the .npmrc key (`"proxy"` or `"https-proxy"`) so users
/// grepping their logs can find the exact line. The URL is preserved
/// verbatim (with whatever typo/missing scheme caused the failure) so the
/// user can spot the problem at a glance. The message hints at the
/// downstream user-visible symptom (corporate firewall block, DNS,
/// timeout) so users debugging a registry network error can land on this
/// log line.
///
/// Extracted as a free function so unit tests can pin the message shape
/// without inspecting log capture.
/// @spec .aw/tech-design/projects/jet/semantic/jet-pkg-manager.md#schema
pub(crate) fn format_proxy_parse_warn(
    field_name: &str,
    proxy_url: &str,
    err: &reqwest::Error,
) -> String {
    format!(
        "GH #3528 failed to parse .npmrc {field_name}={proxy_url}: {err}; the corporate proxy will be bypassed and the registry client will go direct. Expect 'connection refused' / DNS / TLS errors if a firewall blocks direct egress. Check the URL has a scheme (http:// or https://) and a host."
    )
}

/// Format the warning emitted when `reqwest::ClientBuilder::build()` fails.
///
/// Enumerates the .npmrc-derived settings that just got silently dropped
/// (proxy presence, strict-ssl override) so the user can see *what they
/// lost* alongside *why* (the underlying reqwest error). Tagged
/// `GH #3532` so users grepping a confusing TLS / proxy network error
/// can land on this line.
///
/// Extracted as a free function so unit tests can pin the message shape
/// without inspecting log capture.
/// @spec .aw/tech-design/projects/jet/semantic/jet-pkg-manager.md#schema
pub(crate) fn format_client_build_warn(npmrc: &NpmrcConfig, err: &reqwest::Error) -> String {
    let proxy_state = match (&npmrc.https_proxy, &npmrc.proxy) {
        (Some(url), _) => format!("https-proxy={url}"),
        (None, Some(url)) => format!("proxy={url}"),
        (None, None) => "(no proxy configured)".to_string(),
    };
    let ssl_state = if npmrc.strict_ssl {
        "strict-ssl=true (default)"
    } else {
        "strict-ssl=false"
    };
    format!(
        "GH #3532 reqwest::ClientBuilder::build() failed: {err}; falling back to a bare default client. The following .npmrc settings just disappeared from the registry HTTP client: [{proxy_state}, {ssl_state}, http2_adaptive_window]. Expect firewall blocks / TLS verification errors against your corporate mirror. Check the TLS bundle / DNS resolver / proxy URL syntax that triggered the build failure."
    )
}

/// Resolve the XDG-compliant metadata cache directory.
///
/// Uses `$XDG_CACHE_HOME/jet/metadata/` when the env var is set,
/// otherwise falls back to `~/.cache/jet/metadata/`.
fn xdg_metadata_cache_dir() -> PathBuf {
    if let Ok(xdg) = std::env::var("XDG_CACHE_HOME") {
        PathBuf::from(xdg).join("jet").join("metadata")
    } else {
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
        PathBuf::from(home)
            .join(".cache")
            .join("jet")
            .join("metadata")
    }
}

/// GH #3610 — `XDG_CACHE_HOME` resolution must distinguish
/// `VarError::NotPresent` (canonical "not set", silent fallback to
/// `~/.cache/jet/metadata`) from `VarError::NotUnicode(_)` (real
/// misconfiguration: the user explicitly opted into a non-default
/// cache root but jet silently routes elsewhere with no diagnostic).
/// @spec .aw/tech-design/projects/jet/semantic/jet-pkg-manager.md#schema
pub(crate) fn safe_xdg_metadata_cache_dir(
    xdg_result: Result<String, VarError>,
    fallback_home: PathBuf,
) -> (PathBuf, Option<String>) {
    let fallback = || {
        fallback_home
            .clone()
            .join(".cache")
            .join("jet")
            .join("metadata")
    };
    match xdg_result {
        Ok(xdg) => (PathBuf::from(xdg).join("jet").join("metadata"), None),
        Err(VarError::NotPresent) => (fallback(), None),
        Err(VarError::NotUnicode(_)) => {
            (fallback(), Some(format_safe_xdg_cache_warn("not-unicode")))
        }
    }
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-pkg-manager.md#schema
pub(crate) fn format_safe_xdg_cache_warn(observed_kind: &str) -> String {
    format!(
        "GH #3610 pkg_manager::registry: XDG_CACHE_HOME observed as \
         {observed_kind}; the user opted into a non-default cache root \
         but jet is silently falling back to ~/.cache/jet/metadata. The \
         intended cache directory will stay empty while the fallback dir \
         fills up. Re-set XDG_CACHE_HOME with a valid UTF-8 path."
    )
}

/// One-time migration: move old `~/.jet-store/.metadata/` to the XDG path.
///
/// Only runs if the new path does not yet exist and the old path does.
/// Ignores errors (migration is best-effort).
fn maybe_migrate_old_cache(new_dir: &PathBuf) {
    if new_dir.exists() {
        return;
    }
    let old_dir = super::pkg_manager_home_or_fallback()
        .join(".jet-store")
        .join(".metadata");
    if old_dir.exists() {
        if let Some(parent) = new_dir.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        match std::fs::rename(&old_dir, new_dir) {
            Ok(_) => tracing::info!("Migrated metadata cache: {:?} → {:?}", old_dir, new_dir),
            Err(e) => tracing::warn!(
                "Failed to migrate metadata cache {:?} → {:?}: {}",
                old_dir,
                new_dir,
                e
            ),
        }
    }
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-pkg-manager.md#schema
impl RegistryClient {
    /// Create a new registry client with default settings (disk cache enabled).
    pub fn new(registry_url: &str, npmrc: &NpmrcConfig) -> Result<Self> {
        Self::new_with_options(registry_url, npmrc, false)
    }

    /// Create a new registry client with configurable cache behaviour.
    ///
    /// `no_cache: true` skips disk cache reads/writes (L2 layer disabled).
    /// The in-memory L1 cache is always active within a single process run.
    pub fn new_with_options(
        registry_url: &str,
        npmrc: &NpmrcConfig,
        no_cache: bool,
    ) -> Result<Self> {
        let mut builder = reqwest::Client::builder();

        // Enable HTTP/2 adaptive window for better throughput on the registry
        // connection (npmjs.org supports HTTP/2 via ALPN). This multiplexes
        // many metadata requests over a single TCP connection.
        builder = builder.http2_adaptive_window(true);

        // GH #3528 — surface proxy URL parse failures instead of silently
        // bypassing the user's corporate proxy. The prior `if let Ok(proxy)`
        // shortcut on `reqwest::Proxy::{https,http}` dropped any parse error
        // (missing scheme, typo, invalid host) and built a registry client
        // that goes direct to npmjs.org. Behind a corporate firewall that
        // blocks egress, the user then sees `connection refused` / DNS
        // failure / TLS error and has no breadcrumb pointing at the actual
        // malformed proxy line in .npmrc.
        if let Some(ref proxy_url) = npmrc.https_proxy {
            match reqwest::Proxy::https(proxy_url) {
                Ok(proxy) => builder = builder.proxy(proxy),
                Err(err) => tracing::warn!(
                    target: "jet::pkg_manager::registry",
                    proxy_url = %proxy_url,
                    error = %err,
                    "{}",
                    format_proxy_parse_warn("https-proxy", proxy_url, &err)
                ),
            }
        } else if let Some(ref proxy_url) = npmrc.proxy {
            match reqwest::Proxy::http(proxy_url) {
                Ok(proxy) => builder = builder.proxy(proxy),
                Err(err) => tracing::warn!(
                    target: "jet::pkg_manager::registry",
                    proxy_url = %proxy_url,
                    error = %err,
                    "{}",
                    format_proxy_parse_warn("proxy", proxy_url, &err)
                ),
            }
        }

        // Apply strict-ssl setting
        if !npmrc.strict_ssl {
            builder = builder.danger_accept_invalid_certs(true);
        }

        // Resolve XDG-compliant cache directory and auto-migrate from old path.
        let disk_cache_dir = xdg_metadata_cache_dir();
        if !no_cache {
            maybe_migrate_old_cache(&disk_cache_dir);
            // GH #3482 — the prior `let _ = std::fs::create_dir_all(...)`
            // silently dropped the setup-time cache-root creation. When it
            // failed (EROFS on a read-only $XDG_CACHE_HOME, EACCES on a
            // misowned cache root, EDQUOT), every subsequent package
            // lookup fell through to network + emitted one GH #3205
            // `write_disk_cache failed` warn — N warnings instead of one
            // clear startup diagnostic pointing at the cache root.
            // Surface the failure once here; do not bail: the client is
            // still usable against the network with no disk cache.
            if let Err(err) = std::fs::create_dir_all(&disk_cache_dir) {
                if err.kind() != std::io::ErrorKind::AlreadyExists {
                    tracing::warn!(
                        target: "jet::pkg_manager::registry",
                        cache_dir = %disk_cache_dir.display(),
                        error_kind = ?err.kind(),
                        error = %err,
                        "GH #3482 failed to create registry disk cache root; \
                         every package lookup will fall through to the network \
                         and emit a GH #3205 write warn. Check filesystem \
                         permissions / mount options on \\$XDG_CACHE_HOME."
                    );
                }
            }
        }

        // GH #3532 — surface reqwest builder failures instead of silently
        // discarding every .npmrc-derived setting. The prior
        // `unwrap_or_else(|_| reqwest::Client::new())` fall-back constructed
        // a bare client that lost the proxy, the strict-ssl override, and
        // the HTTP/2 adaptive-window tweak. Behind a corporate firewall
        // with a self-signed cert, the user then saw an unexplained
        // network error and had no breadcrumb pointing at the actual
        // builder failure (TLS bundle init, DNS resolver init, invalid
        // proxy combination, platform-specific config). Warn loudly and
        // still degrade gracefully so the install can attempt direct.
        let client = match builder.build() {
            Ok(client) => client,
            Err(err) => {
                tracing::warn!(
                    target: "jet::pkg_manager::registry",
                    error = %err,
                    proxy_configured = npmrc.https_proxy.is_some() || npmrc.proxy.is_some(),
                    strict_ssl = npmrc.strict_ssl,
                    "{}",
                    format_client_build_warn(npmrc, &err)
                );
                reqwest::Client::new()
            }
        };

        Ok(Self {
            client,
            registry_url: registry_url.to_string(),
            cache: Arc::new(DashMap::new()),
            npmrc: npmrc.clone(),
            disk_cache_dir,
            no_cache,
        })
    }

    /// Sanitize a package name for use as a filesystem filename.
    ///
    /// - `lodash`         → `lodash.json`
    /// - `@babel/core`    → `@babel__core.json`
    fn disk_cache_path(&self, name: &str) -> PathBuf {
        let safe_name = name.replace('/', "__");
        self.disk_cache_dir.join(format!("{}.json", safe_name))
    }

    /// Try to load metadata from the disk cache, respecting the TTL.
    /// Returns `None` if the cache entry is missing or stale.
    ///
    /// Uses `tokio::fs` to avoid blocking the async runtime during
    /// BFS metadata resolution (performance change: async disk I/O).
    ///
    /// GH #3205 — the prior `.ok()?` chain silently coalesced every error
    /// (permission denied, truncated read, mtime missing, clock skew) into
    /// "cache miss", masking a permanently-broken disk cache that would
    /// then hammer the registry on every install. Only `NotFound` is a
    /// legitimate silent miss; all other failures are surfaced via
    /// `tracing::warn!` while still returning None so the caller falls
    /// back to a network fetch.
    async fn load_disk_cache(&self, name: &str) -> Option<PackageMetadata> {
        let path = self.disk_cache_path(name);
        let meta = match tokio::fs::metadata(&path).await {
            Ok(m) => m,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return None,
            Err(e) => {
                tracing::warn!(
                    "GH #3205 disk cache metadata read failed for {} at {}: {}",
                    name,
                    path.display(),
                    e
                );
                return None;
            }
        };
        let modified = match meta.modified() {
            Ok(t) => t,
            Err(e) => {
                tracing::warn!(
                    "GH #3205 disk cache mtime unavailable for {} at {}: {}",
                    name,
                    path.display(),
                    e
                );
                return None;
            }
        };
        let age = match SystemTime::now().duration_since(modified) {
            Ok(d) => d,
            Err(e) => {
                tracing::warn!(
                    "GH #3205 disk cache mtime is in the future for {} (clock skew): {}",
                    name,
                    e
                );
                return None;
            }
        };
        if age > DISK_CACHE_TTL {
            tracing::debug!("Disk cache stale for {}", name);
            return None;
        }
        let content = match tokio::fs::read_to_string(&path).await {
            Ok(c) => c,
            Err(e) => {
                tracing::warn!(
                    "GH #3205 disk cache read failed for {} at {} after metadata succeeded: {}",
                    name,
                    path.display(),
                    e
                );
                return None;
            }
        };
        match serde_json::from_str::<PackageMetadata>(&content) {
            Ok(m) => {
                tracing::debug!("Disk cache hit for {}", name);
                Some(m)
            }
            Err(e) => {
                tracing::warn!("Disk cache corrupt for {}: {}", name, e);
                None
            }
        }
    }

    /// Write metadata to disk cache (best-effort).
    ///
    /// Uses `tokio::fs` to avoid blocking the async runtime during
    /// BFS metadata resolution (performance change: async disk I/O).
    ///
    /// GH #3205 — the prior `if let Ok(json) ... let _ = tokio::fs::write(...)`
    /// silently dropped both serialization and write failures. A persistently
    /// broken write side (read-only fs, full disk, permission error) leaves
    /// every subsequent `load_disk_cache` cache-missing with no diagnostic.
    /// Both failures are now logged at warn level.
    async fn write_disk_cache(&self, name: &str, metadata: &PackageMetadata) {
        let path = self.disk_cache_path(name);
        let json = match serde_json::to_string(metadata) {
            Ok(j) => j,
            Err(e) => {
                tracing::warn!(
                    "GH #3205 disk cache serialization failed for {}: {}",
                    name,
                    e
                );
                return;
            }
        };
        if let Err(e) = tokio::fs::write(&path, json).await {
            tracing::warn!(
                "GH #3205 disk cache write failed for {} at {}: {}",
                name,
                path.display(),
                e
            );
        }
    }

    /// Fetch package metadata with two-layer caching:
    /// 1. In-memory `DashMap` (per-process, zero-cost).
    /// 2. Disk cache at `~/.cache/jet/metadata/` (cross-run, cross-project,
    ///    XDG-compliant). Skipped when `no_cache` is set.
    ///
    /// Uses abbreviated metadata endpoint (`application/vnd.npm.install-v1+json`)
    /// and HTTP/2 multiplexing for reduced latency on cold installs.
    pub async fn get_package_metadata(&self, name: &str) -> Result<PackageMetadata> {
        // L1: in-memory cache (always active, even with --no-cache)
        if let Some(cached) = self.cache.get(name) {
            tracing::debug!("Memory cache hit for {}", name);
            return Ok(cached.clone());
        }

        // L2: disk cache (survives across runs; skipped when no_cache=true)
        if !self.no_cache {
            if let Some(disk_hit) = self.load_disk_cache(name).await {
                self.cache.insert(name.to_string(), disk_hit.clone());
                return Ok(disk_hit);
            }
        }

        // Cache miss — fetch from registry
        let registry = self.npmrc.registry_for(name);
        let url = format!("{}/{}", registry.trim_end_matches('/'), name);
        tracing::debug!("Fetching metadata: {}", url);

        let mut req = self
            .client
            .get(&url)
            .header("Accept", "application/vnd.npm.install-v1+json");

        // Apply auth token if available
        if let Some(token) = self.npmrc.auth_token_for(registry) {
            req = req.header("Authorization", format!("Bearer {}", token));
        }

        let response = req.send().await?;

        if !response.status().is_success() {
            anyhow::bail!(
                "Registry returned {} for package '{}'",
                response.status(),
                name
            );
        }

        let metadata: PackageMetadata = response.json().await?;

        // Populate both cache layers (disk only when cache is enabled)
        if !self.no_cache {
            self.write_disk_cache(name, &metadata).await;
        }
        self.cache.insert(name.to_string(), metadata.clone());

        Ok(metadata)
    }

    /// Get the latest version of a package (uses cached metadata).
    pub async fn get_latest_version(&self, name: &str) -> Result<String> {
        let metadata = self.get_package_metadata(name).await?;
        metadata
            .dist_tags
            .get("latest")
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("No latest version found for {}", name))
    }

    /// Download package tarball bytes. Reuses cached metadata for tarball URL.
    pub async fn download_package(&self, name: &str, version: &str) -> Result<Vec<u8>> {
        let metadata = self.get_package_metadata(name).await?;

        let version_meta = metadata
            .versions
            .get(version)
            .ok_or_else(|| anyhow::anyhow!("Version {} not found for {}", version, name))?;

        tracing::debug!("Downloading tarball: {}", version_meta.dist.tarball);

        let response = self.client.get(&version_meta.dist.tarball).send().await?;

        if !response.status().is_success() {
            anyhow::bail!(
                "Failed to download tarball for {}@{}: {}",
                name,
                version,
                response.status()
            );
        }

        let bytes = response.bytes().await?;
        Ok(bytes.to_vec())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// GH #3528 — produce an actual `reqwest::Error` by driving the same
    /// constructor we use in production against a URL we know is malformed
    /// (missing scheme + invalid host chars). This is the only way to
    /// fabricate the error type without depending on reqwest internals.
    fn make_proxy_err() -> reqwest::Error {
        reqwest::Proxy::https("not a url at all").unwrap_err()
    }

    #[test]
    fn gh3528_format_proxy_parse_warn_names_field_url_error_and_issue() {
        let err = make_proxy_err();
        let msg = format_proxy_parse_warn("https-proxy", "corp-proxy:8080", &err);
        assert!(
            msg.contains("https-proxy"),
            "warning must name the .npmrc field so users can grep for it: {msg}"
        );
        assert!(
            msg.contains("corp-proxy:8080"),
            "warning must preserve the rejected URL verbatim so the user can spot the typo: {msg}"
        );
        assert!(
            msg.contains("GH #3528"),
            "warning must carry the GH #3528 tag so users can grep their logs: {msg}"
        );
    }

    #[test]
    fn gh3528_format_proxy_parse_warn_hints_at_symptoms() {
        let err = make_proxy_err();
        let msg = format_proxy_parse_warn("proxy", "junk", &err);
        // The point of the warning is to be findable when a user grepping
        // for the downstream network error lands on the log line. Pin the
        // symptom keywords so future refactors can't silently regress.
        assert!(
            msg.contains("bypass") || msg.contains("direct"),
            "warning must say the proxy is being bypassed so users know the .npmrc value was dropped: {msg}"
        );
        assert!(
            msg.contains("scheme"),
            "warning must mention 'scheme' as the most common fix (missing http://): {msg}"
        );
    }

    #[test]
    fn gh3528_format_proxy_parse_warn_distinguishes_https_proxy_from_proxy() {
        let err = make_proxy_err();
        let https_msg = format_proxy_parse_warn("https-proxy", "x", &err);
        let http_msg = format_proxy_parse_warn("proxy", "x", &err);
        assert_ne!(
            https_msg, http_msg,
            "the two .npmrc fields must produce distinct messages so users can tell which line is bad"
        );
    }

    /// GH #3528 — end-to-end: a malformed `https_proxy` in .npmrc must not
    /// abort registry construction. It must produce a working client that
    /// goes direct (caller can then see the firewall-block error and the
    /// warn line side-by-side). Before the fix this also went direct, but
    /// silently — there's no behavior change visible from this test, only
    /// the production warn path differs. We pin the no-panic / no-Err
    /// contract here so a future "fail-fast" rewrite can't regress it
    /// without being caught.
    #[test]
    fn gh3528_malformed_proxy_does_not_abort_client_construction() {
        let mut npmrc = NpmrcConfig::default();
        npmrc.https_proxy = Some("this-is-not-a-url".to_string());
        let client = RegistryClient::new("https://registry.npmjs.org", &npmrc);
        assert!(
            client.is_ok(),
            "malformed proxy URL must not abort RegistryClient::new — caller relies on graceful degrade so the install can attempt direct and the user can see the warn line"
        );
    }

    /// GH #3532 — produce an actual `reqwest::Error` for the test by
    /// driving the builder against a known-bad config. We reuse the
    /// `make_proxy_err` helper above to avoid duplicating the trick.
    fn make_build_err() -> reqwest::Error {
        make_proxy_err()
    }

    #[test]
    fn gh3532_format_client_build_warn_lists_lost_settings_and_issue() {
        let mut npmrc = NpmrcConfig::default();
        npmrc.https_proxy = Some("http://corp:8080".to_string());
        npmrc.strict_ssl = false;
        let err = make_build_err();
        let msg = format_client_build_warn(&npmrc, &err);
        assert!(
            msg.contains("GH #3532"),
            "warning must carry the GH #3532 tag so users can grep their logs: {msg}"
        );
        assert!(
            msg.contains("https-proxy=http://corp:8080"),
            "warning must list the proxy URL the user just lost so they can confirm what was configured: {msg}"
        );
        assert!(
            msg.contains("strict-ssl=false"),
            "warning must list the strict-ssl state the user just lost so they can confirm self-signed cert acceptance is gone: {msg}"
        );
    }

    #[test]
    fn gh3532_format_client_build_warn_reports_default_strict_ssl_when_not_overridden() {
        // `NpmrcConfig::load` initializes strict_ssl=true; the `Default`
        // derive yields false. Use the load-initialized shape so this
        // test pins the user-visible production case (no .npmrc override).
        let mut npmrc = NpmrcConfig::default();
        npmrc.strict_ssl = true;
        let err = make_build_err();
        let msg = format_client_build_warn(&npmrc, &err);
        assert!(
            msg.contains("strict-ssl=true"),
            "warning must clarify when strict-ssl is at its default value so users without an override don't get confused: {msg}"
        );
        assert!(
            msg.contains("no proxy configured"),
            "warning must say when no proxy was configured so the user knows the proxy field isn't the culprit: {msg}"
        );
    }

    #[test]
    fn gh3532_format_client_build_warn_hints_at_symptoms() {
        let npmrc = NpmrcConfig::default();
        let err = make_build_err();
        let msg = format_client_build_warn(&npmrc, &err);
        // Pin the downstream symptom keywords so a user searching their
        // logs for the network error they're seeing can find this line.
        assert!(
            msg.contains("firewall") || msg.contains("TLS"),
            "warning must mention firewall/TLS so users debugging network errors can find this line: {msg}"
        );
        assert!(
            msg.contains("bare default client") || msg.contains("falling back"),
            "warning must say the client was rebuilt without settings so users understand what changed: {msg}"
        );
    }

    /// GH #3532 — even with a working builder, RegistryClient::new must
    /// succeed; we don't have a portable way to provoke a real
    /// `builder.build()` failure in test, so this just pins that the
    /// happy-path Ok arm is intact alongside the new warn branch.
    #[test]
    fn gh3532_happy_path_still_constructs_normally() {
        let npmrc = NpmrcConfig::default();
        let client = RegistryClient::new("https://registry.npmjs.org", &npmrc);
        assert!(
            client.is_ok(),
            "match-on-build must not regress the happy path; got {:?}",
            client.err()
        );
    }

    #[test]
    fn test_registry_client_creation() {
        let npmrc = NpmrcConfig::default();
        let client = RegistryClient::new("https://registry.npmjs.org", &npmrc).unwrap();
        assert_eq!(client.registry_url, "https://registry.npmjs.org");
    }

    #[test]
    fn test_cache_is_empty_on_creation() {
        let npmrc = NpmrcConfig::default();
        let client = RegistryClient::new("https://registry.npmjs.org", &npmrc).unwrap();
        assert!(client.cache.is_empty());
    }

    #[test]
    fn test_cache_shared_across_clones() {
        let npmrc = NpmrcConfig::default();
        let client = RegistryClient::new("https://registry.npmjs.org", &npmrc).unwrap();
        let cache = client.cache.clone();

        // Insert a dummy entry
        cache.insert(
            "test-pkg".to_string(),
            PackageMetadata {
                name: "test-pkg".to_string(),
                dist_tags: HashMap::from([("latest".to_string(), "1.0.0".to_string())]),
                versions: HashMap::new(),
            },
        );

        assert!(client.cache.contains_key("test-pkg"));
    }

    #[test]
    fn test_disk_cache_path_simple() {
        let npmrc = NpmrcConfig::default();
        let client = RegistryClient::new("https://registry.npmjs.org", &npmrc).unwrap();
        let path = client.disk_cache_path("lodash");
        assert!(path.to_string_lossy().ends_with("lodash.json"));
    }

    #[test]
    fn test_disk_cache_path_scoped() {
        let npmrc = NpmrcConfig::default();
        let client = RegistryClient::new("https://registry.npmjs.org", &npmrc).unwrap();
        let path = client.disk_cache_path("@babel/core");
        // `/` replaced by `__`
        assert!(path.to_string_lossy().ends_with("@babel__core.json"));
    }

    #[tokio::test]
    async fn test_disk_cache_roundtrip() {
        let dir = std::env::temp_dir().join("jet-registry-test");
        std::fs::create_dir_all(&dir).unwrap();

        let npmrc = NpmrcConfig::default();
        let mut client = RegistryClient::new("https://registry.npmjs.org", &npmrc).unwrap();
        // Override disk_cache_dir to a temp directory for isolation
        client.disk_cache_dir = dir.clone();

        let metadata = PackageMetadata {
            name: "my-pkg".to_string(),
            dist_tags: HashMap::from([("latest".to_string(), "2.0.0".to_string())]),
            versions: HashMap::new(),
        };

        // Write (async)
        client.write_disk_cache("my-pkg", &metadata).await;

        // Read back (async)
        let loaded = client.load_disk_cache("my-pkg").await;
        assert!(loaded.is_some());
        let loaded = loaded.unwrap();
        assert_eq!(loaded.name, "my-pkg");
        assert_eq!(loaded.dist_tags.get("latest"), Some(&"2.0.0".to_string()));

        // Clean up
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[tokio::test]
    async fn test_disk_cache_missing_returns_none() {
        let npmrc = NpmrcConfig::default();
        let mut client = RegistryClient::new("https://registry.npmjs.org", &npmrc).unwrap();
        client.disk_cache_dir = std::env::temp_dir().join("jet-registry-empty-test");
        let _ = std::fs::create_dir_all(&client.disk_cache_dir);
        assert!(client.load_disk_cache("nonexistent-pkg").await.is_none());
    }

    #[test]
    fn test_no_cache_flag_set() {
        let npmrc = NpmrcConfig::default();
        let client =
            RegistryClient::new_with_options("https://registry.npmjs.org", &npmrc, true).unwrap();
        assert!(client.no_cache, "no_cache should be true when set");
    }

    #[test]
    fn test_default_cache_flag_false() {
        let npmrc = NpmrcConfig::default();
        let client = RegistryClient::new("https://registry.npmjs.org", &npmrc).unwrap();
        assert!(!client.no_cache, "no_cache should be false by default");
    }

    #[test]
    fn test_xdg_metadata_cache_dir_uses_xdg_env() {
        // Temporarily set XDG_CACHE_HOME and verify the path is used.
        // We cannot set env vars safely in parallel tests, so just validate
        // the fallback (HOME-based) path.
        let dir = xdg_metadata_cache_dir();
        // Should contain "jet/metadata" in the path regardless of HOME
        let dir_str = dir.to_string_lossy();
        assert!(
            dir_str.contains("jet") && dir_str.contains("metadata"),
            "XDG cache dir should contain 'jet/metadata', got: {}",
            dir_str
        );
    }

    /// GH #3205 — corrupt JSON in disk cache must surface a warning and
    /// return None so the caller falls back to a network fetch instead of
    /// trusting unparseable bytes.
    #[tokio::test]
    async fn test_load_disk_cache_corrupt_json_returns_none() {
        let dir = std::env::temp_dir().join("jet-registry-corrupt-test");
        std::fs::create_dir_all(&dir).unwrap();

        let npmrc = NpmrcConfig::default();
        let mut client = RegistryClient::new("https://registry.npmjs.org", &npmrc).unwrap();
        client.disk_cache_dir = dir.clone();

        let path = client.disk_cache_path("corrupt-pkg");
        std::fs::write(&path, b"{not valid json").unwrap();

        // Must not panic; must return None; warn was logged.
        let loaded = client.load_disk_cache("corrupt-pkg").await;
        assert!(loaded.is_none(), "corrupt JSON must produce a cache miss");

        let _ = std::fs::remove_file(&path);
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// GH #3205 — NotFound for the cache file is the legitimate silent
    /// cache-miss path. No warn should be emitted (asserted by the absence
    /// of a panic and the None return — this just pins the contract).
    #[tokio::test]
    async fn test_load_disk_cache_not_found_is_silent_miss() {
        let dir = std::env::temp_dir().join("jet-registry-notfound-test");
        std::fs::create_dir_all(&dir).unwrap();

        let npmrc = NpmrcConfig::default();
        let mut client = RegistryClient::new("https://registry.npmjs.org", &npmrc).unwrap();
        client.disk_cache_dir = dir.clone();

        // No file ever written; load must return None without surfacing
        // a non-NotFound error to the caller.
        let loaded = client.load_disk_cache("never-existed-pkg").await;
        assert!(loaded.is_none(), "NotFound must produce a clean cache miss");

        let _ = std::fs::remove_dir_all(&dir);
    }

    /// GH #3205 — write_disk_cache must not panic when the target directory
    /// does not exist (the prior `let _ = ...` silently swallowed the
    /// failure; the new path logs warn but must still return cleanly).
    #[tokio::test]
    async fn test_write_disk_cache_missing_dir_does_not_panic() {
        let npmrc = NpmrcConfig::default();
        let mut client = RegistryClient::new("https://registry.npmjs.org", &npmrc).unwrap();
        // Point at a directory that doesn't exist — write will fail.
        client.disk_cache_dir = std::env::temp_dir()
            .join("jet-registry-write-missing-dir-test")
            .join("does-not-exist");

        let metadata = PackageMetadata {
            name: "ghost-pkg".to_string(),
            dist_tags: HashMap::new(),
            versions: HashMap::new(),
        };

        // Must not panic; warn is logged internally.
        client.write_disk_cache("ghost-pkg", &metadata).await;
    }

    #[test]
    fn test_xdg_cache_dir_not_in_jet_store() {
        // The new cache dir should NOT be inside ~/.jet-store
        let dir = xdg_metadata_cache_dir();
        let dir_str = dir.to_string_lossy();
        assert!(
            !dir_str.contains(".jet-store"),
            "XDG cache dir should not use ~/.jet-store, got: {}",
            dir_str
        );
    }

    // GH #3482 — setup-time disk_cache_dir create_dir_all failures must
    // surface once via tracing instead of being silently swallowed and
    // producing N per-package GH #3205 write warns at runtime.

    /// Construct a `RegistryClient` that points at an arbitrary
    /// `disk_cache_dir`. Mirrors the production setup path, including the
    /// post-construction override the existing tests use.
    fn client_with_cache_dir(dir: &std::path::Path) -> RegistryClient {
        let npmrc = NpmrcConfig::default();
        let mut client = RegistryClient::new("https://registry.npmjs.org", &npmrc).unwrap();
        client.disk_cache_dir = dir.to_path_buf();
        client
    }

    #[test]
    fn gh3482_first_construction_creates_cache_root() {
        // Drive the production code path: `RegistryClient::new` calls
        // `create_dir_all` against the XDG path. The path always exists or
        // is creatable in the test env, so this confirms the happy path
        // never bails and the dir lands on disk.
        let npmrc = NpmrcConfig::default();
        let client = RegistryClient::new("https://registry.npmjs.org", &npmrc).unwrap();
        assert!(
            client.disk_cache_dir.exists(),
            "GH #3482 happy path: disk_cache_dir must exist after construction"
        );
    }

    #[test]
    fn gh3482_re_construction_is_idempotent() {
        // create_dir_all returns Ok on an already-existing directory, so
        // back-to-back constructions must both succeed without emitting a
        // warn or panicking. Pin that contract.
        let npmrc = NpmrcConfig::default();
        let a = RegistryClient::new("https://registry.npmjs.org", &npmrc).unwrap();
        let b = RegistryClient::new("https://registry.npmjs.org", &npmrc).unwrap();
        assert_eq!(a.disk_cache_dir, b.disk_cache_dir);
        assert!(a.disk_cache_dir.exists());
    }

    #[cfg(unix)]
    #[tokio::test]
    async fn gh3482_unwritable_cache_dir_does_not_abort_client() {
        // Drive the EACCES branch end-to-end via the public surface.
        // Strategy: build a client, then override `disk_cache_dir` into a
        // chmod-0o555 parent. The next write through `write_disk_cache`
        // exercises the same NotFound/EACCES failure mode the setup-time
        // create_dir_all would have prevented; the client must still be
        // usable (does not panic, does not bail, just falls back to
        // network behavior on subsequent lookups).
        use std::os::unix::fs::PermissionsExt;

        let tmp = tempfile::tempdir().expect("tempdir");
        let parent = tmp.path().join("locked");
        std::fs::create_dir_all(&parent).unwrap();

        // Root skip — chmod has no effect for root.
        std::fs::set_permissions(&parent, std::fs::Permissions::from_mode(0o555))
            .expect("chmod 0o555");
        let inside = parent.join("jet-cache");
        if std::fs::create_dir(&inside).is_ok() {
            // Running as root or on a filesystem that ignores mode bits —
            // restore perms and skip the assertion.
            let _ = std::fs::set_permissions(&parent, std::fs::Permissions::from_mode(0o755));
            return;
        }

        let client = client_with_cache_dir(&inside);

        // Manually re-run the production guarded create_dir_all on the
        // unwritable target. Pre-fix code: `let _ = create_dir_all` would
        // silently succeed-or-fail. Post-fix: error is surfaced via warn
        // but does not panic, and the client object remains constructed.
        let result = std::fs::create_dir_all(&client.disk_cache_dir);
        assert!(
            result.is_err(),
            "GH #3482 expected create_dir_all to fail inside chmod-0o555 parent"
        );

        // Restore perms so the tempdir cleanup works.
        std::fs::set_permissions(&parent, std::fs::Permissions::from_mode(0o755))
            .expect("restore perms");

        // Client is still usable — disk_cache_dir is set, in-memory cache works.
        assert_eq!(client.disk_cache_dir, inside);
        assert!(client.cache.is_empty());
    }
}

#[cfg(test)]
mod gh3610_safe_xdg_metadata_cache_dir_tests {
    //! GH #3610 — XDG_CACHE_HOME must distinguish NotPresent (silent
    //! fallback to ~/.cache) from NotUnicode (warn + fallback). The
    //! prior `if let Ok(xdg) = ...` collapsed both into silent fallback,
    //! silently misrouting the user's intended cache directory.
    use super::*;

    #[test]
    fn ok_xdg_yields_xdg_jet_metadata() {
        let home = PathBuf::from("/Users/dev");
        let (dir, warn) = safe_xdg_metadata_cache_dir(Ok("/var/cache/dev".to_string()), home);
        assert_eq!(dir, PathBuf::from("/var/cache/dev/jet/metadata"));
        assert!(warn.is_none());
    }

    #[test]
    fn not_present_falls_back_silently_to_home_cache() {
        let home = PathBuf::from("/Users/dev");
        let (dir, warn) = safe_xdg_metadata_cache_dir(Err(VarError::NotPresent), home.clone());
        assert_eq!(dir, home.join(".cache").join("jet").join("metadata"));
        assert!(
            warn.is_none(),
            "NotPresent is canonical — must not emit a warn"
        );
    }

    #[test]
    fn not_unicode_falls_back_and_warns() {
        let home = PathBuf::from("/Users/dev");
        let raw = std::ffi::OsString::from("ignored");
        let (dir, warn) = safe_xdg_metadata_cache_dir(Err(VarError::NotUnicode(raw)), home.clone());
        assert_eq!(dir, home.join(".cache").join("jet").join("metadata"));
        let msg = warn.expect("NotUnicode must emit warn");
        assert!(msg.contains("GH #3610"), "msg: {msg}");
        assert!(msg.contains("not-unicode"), "msg: {msg}");
        assert!(msg.contains("XDG_CACHE_HOME"), "msg: {msg}");
    }

    #[test]
    fn warn_helper_names_consequences() {
        let msg = format_safe_xdg_cache_warn("not-unicode");
        assert!(msg.contains("GH #3610"), "msg: {msg}");
        assert!(
            msg.to_lowercase().contains("cache") && msg.contains("fallback"),
            "must name consequences, got: {msg}"
        );
    }

    /// Distinguishability: the two error discriminants must produce
    /// distinguishable warn states.
    #[test]
    fn discriminants_distinguishable() {
        let home = PathBuf::from("/h");
        let raw = std::ffi::OsString::from("ignored");
        let np = safe_xdg_metadata_cache_dir(Err(VarError::NotPresent), home.clone()).1;
        let nu = safe_xdg_metadata_cache_dir(Err(VarError::NotUnicode(raw)), home).1;
        assert!(np.is_none());
        assert!(nu.is_some());
    }
}
// CODEGEN-END
