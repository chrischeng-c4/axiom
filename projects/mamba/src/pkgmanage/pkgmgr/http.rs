// REQ: fetch_metadata_json — HTTP GET /pypi/{name}/json with retry + backoff
// REQ: fetch_metadata_simple — GET simple/{name}/ with PEP 691 content-negotiation
// REQ: fetch_metadata — try fetch_metadata_json; on NotFound fall back to fetch_metadata_simple
// REQ: PEP 503 name normalization before URL interpolation
// REQ: 404 → IndexError::NotFound, 429/5xx → retry with exponential backoff + jitter
// REQ: other non-2xx → IndexError::NetworkError
// REQ: transport error after retries → IndexError::NetworkError
// REQ: download_artifact — streaming sha256-verified artifact download with cache layer

use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use futures_util::StreamExt;
use rand::Rng;
use reqwest::StatusCode;
use sha2::{Digest, Sha256};
use tokio::sync::Semaphore;

use crate::pkgmanage::pkgmgr::{
    cache::{
        artifact_path, default_cache_dir, promote_to_content_addressed, read_cached_artifact,
        read_cached_etag, read_cached_metadata, read_content_addressed_artifact, write_cached_etag,
        write_cached_metadata, METADATA_TTL_SECS,
    },
    json_api::parse_json_metadata,
    simple_api::{parse_simple_html, parse_simple_json},
    types::{IndexClient, IndexError, PackageMetadata, ReleaseFile},
};

/// Normalize a package name per PEP 503: lowercase and replace runs of
/// `[-_.]` with a single `-`.
fn normalize_name(name: &str) -> String {
    let lower = name.to_lowercase();
    // Replace one or more occurrences of [-_.] with a single dash.
    let re = regex::Regex::new(r"[-_.]+").expect("static regex");
    re.replace_all(&lower, "-").into_owned()
}

/// Status codes that should trigger a retry with exponential backoff.
const RETRYABLE_CODES: &[u16] = &[429, 500, 502, 503, 504];

impl IndexClient {
    /// Resolve the effective cache directory for this client.
    ///
    /// Uses `self.cache_dir` when non-empty, otherwise falls back to
    /// [`default_cache_dir()`]. Does not modify `IndexClient`.
    fn effective_cache_dir(&self) -> std::path::PathBuf {
        if self.cache_dir.is_empty() {
            default_cache_dir()
        } else {
            std::path::PathBuf::from(&self.cache_dir)
        }
    }

    /// Fetch package metadata from the PyPI JSON API endpoint.
    ///
    /// Endpoint: `{index_url}/pypi/{normalized_name}/json`
    ///
    /// Checks the on-disk metadata cache before performing HTTP. On hit, returns
    /// the cached value immediately. On miss, performs the HTTP request, and on
    /// success writes the result to cache (best-effort; cache errors are ignored).
    ///
    /// Retries on status codes 429, 500, 502, 503, 504 with exponential backoff:
    /// base 200 ms, 2× multiplier, ±20% uniform jitter, max `self.retry_max` retries.
    ///
    /// # Errors
    ///
    /// - [`IndexError::NotFound`] — HTTP 404
    /// - [`IndexError::NetworkError`] — transport error or non-retryable non-2xx after retries
    /// - [`IndexError::ParseError`] — response body is not valid PyPI JSON
    pub async fn fetch_metadata_json(&self, name: &str) -> Result<PackageMetadata, IndexError> {
        let cache_dir = self.effective_cache_dir();

        // Cache hit — return immediately without HTTP.
        if let Some(cached) =
            read_cached_metadata(&cache_dir, name, "json", METADATA_TTL_SECS).await
        {
            return Ok(cached);
        }

        // REQ: AC4 — read ETag sidecar for conditional-request support.
        let cached_etag = read_cached_etag(&cache_dir, name, "json").await;

        let normalized = normalize_name(name);
        let url = format!(
            "{}/pypi/{}/json",
            self.index_url.trim_end_matches('/'),
            normalized
        );

        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(self.timeout_secs))
            .build()
            .map_err(|e| IndexError::NetworkError {
                url: url.clone(),
                detail: format!("failed to build HTTP client: {e}"),
            })?;

        let mut attempt = 0u32;
        loop {
            // REQ: AC4 — attach If-None-Match when a cached ETag is available.
            let req = client.get(&url);
            let req = if let Some(ref etag) = cached_etag {
                req.header("If-None-Match", etag)
            } else {
                req
            };
            let result = req.send().await;

            match result {
                Err(e) => {
                    // Transport-level error (DNS, connect, timeout, etc.)
                    if attempt >= self.retry_max {
                        return Err(IndexError::NetworkError {
                            url: url.clone(),
                            detail: e.to_string(),
                        });
                    }
                    // Retry transport errors just like retryable status codes.
                    backoff_sleep(attempt).await;
                    attempt += 1;
                    continue;
                }
                Ok(response) => {
                    let status = response.status();

                    // REQ: AC4 — 304 Not Modified: serve stale cached entry (ETag validated).
                    if status == StatusCode::NOT_MODIFIED {
                        match read_cached_metadata(&cache_dir, name, "json", u64::MAX).await {
                            Some(stale) => {
                                // Refresh mtime so TTL resets from now.
                                let _ =
                                    write_cached_metadata(&cache_dir, name, "json", &stale).await;
                                return Ok(stale);
                            }
                            None => {
                                return Err(IndexError::NetworkError {
                                    url: url.clone(),
                                    detail: "304 received but cache entry missing".to_string(),
                                });
                            }
                        }
                    }

                    if status == StatusCode::NOT_FOUND {
                        return Err(IndexError::NotFound {
                            name: name.to_string(),
                        });
                    }

                    if RETRYABLE_CODES.contains(&status.as_u16()) {
                        if attempt >= self.retry_max {
                            return Err(IndexError::NetworkError {
                                url: url.clone(),
                                detail: format!(
                                    "retryable status {} after {} attempts",
                                    status,
                                    attempt + 1
                                ),
                            });
                        }
                        backoff_sleep(attempt).await;
                        attempt += 1;
                        continue;
                    }

                    if !status.is_success() {
                        return Err(IndexError::NetworkError {
                            url: url.clone(),
                            detail: format!("unexpected HTTP status {}", status),
                        });
                    }

                    // REQ: AC4 — extract ETag before consuming body.
                    let etag_value = response
                        .headers()
                        .get(reqwest::header::ETAG)
                        .and_then(|v| v.to_str().ok())
                        .map(|s| s.to_string());

                    // 2xx — read the body and parse it.
                    let body = response
                        .text()
                        .await
                        .map_err(|e| IndexError::NetworkError {
                            url: url.clone(),
                            detail: format!("failed to read response body: {e}"),
                        })?;

                    let meta = parse_json_metadata(&body).map_err(|e| match e {
                        IndexError::ParseError { detail, .. } => IndexError::ParseError {
                            url: url.clone(),
                            detail,
                        },
                        other => other,
                    })?;

                    // Best-effort cache write — ignore errors so the hot path never fails.
                    let _ = write_cached_metadata(&cache_dir, name, "json", &meta).await;

                    // REQ: AC4 — persist ETag sidecar for future conditional requests.
                    if let Some(etag_str) = etag_value {
                        let _ = write_cached_etag(&cache_dir, name, "json", &etag_str).await;
                    }

                    return Ok(meta);
                }
            }
        }
    }

    /// Fetch package metadata from the PEP 503 Simple API (HTML) or PEP 691 Simple API (JSON).
    ///
    /// Endpoint: `{index_url}/simple/{normalized_name}/`
    ///
    /// Checks the on-disk metadata cache before performing HTTP. On hit, returns
    /// the cached value immediately. On miss, performs the HTTP request, and on
    /// success writes the result to cache (best-effort; cache errors are ignored).
    ///
    /// Sends `Accept: application/vnd.pypi.simple.v1+json, text/html;q=0.5`.
    /// Dispatches parsing based on the `content-type` of the response:
    /// - `application/vnd.pypi.simple.v1+json` → [`parse_simple_json`]
    /// - anything else → [`parse_simple_html`]
    ///
    /// Same retry policy as [`Self::fetch_metadata_json`].
    ///
    /// # Errors
    ///
    /// - [`IndexError::NotFound`] — HTTP 404
    /// - [`IndexError::NetworkError`] — transport error or non-retryable non-2xx after retries
    /// - [`IndexError::ParseError`] — response body does not parse as the expected format
    pub async fn fetch_metadata_simple(&self, name: &str) -> Result<PackageMetadata, IndexError> {
        let cache_dir = self.effective_cache_dir();

        // Cache hit — return immediately without HTTP.
        if let Some(cached) =
            read_cached_metadata(&cache_dir, name, "simple", METADATA_TTL_SECS).await
        {
            return Ok(cached);
        }

        // REQ: AC4 — read ETag sidecar for conditional-request support.
        let cached_etag = read_cached_etag(&cache_dir, name, "simple").await;

        let normalized = normalize_name(name);
        let url = format!(
            "{}/simple/{}/",
            self.index_url.trim_end_matches('/'),
            normalized
        );

        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(self.timeout_secs))
            .build()
            .map_err(|e| IndexError::NetworkError {
                url: url.clone(),
                detail: format!("failed to build HTTP client: {e}"),
            })?;

        let mut attempt = 0u32;
        loop {
            // REQ: AC4 — attach If-None-Match when a cached ETag is available.
            let req = client.get(&url).header(
                "Accept",
                "application/vnd.pypi.simple.v1+json, text/html;q=0.5",
            );
            let req = if let Some(ref etag) = cached_etag {
                req.header("If-None-Match", etag)
            } else {
                req
            };
            let result = req.send().await;

            match result {
                Err(e) => {
                    if attempt >= self.retry_max {
                        return Err(IndexError::NetworkError {
                            url: url.clone(),
                            detail: e.to_string(),
                        });
                    }
                    backoff_sleep(attempt).await;
                    attempt += 1;
                    continue;
                }
                Ok(response) => {
                    let status = response.status();

                    // REQ: AC4 — 304 Not Modified: serve stale cached entry (ETag validated).
                    if status == StatusCode::NOT_MODIFIED {
                        match read_cached_metadata(&cache_dir, name, "simple", u64::MAX).await {
                            Some(stale) => {
                                // Refresh mtime so TTL resets from now.
                                let _ =
                                    write_cached_metadata(&cache_dir, name, "simple", &stale).await;
                                return Ok(stale);
                            }
                            None => {
                                return Err(IndexError::NetworkError {
                                    url: url.clone(),
                                    detail: "304 received but cache entry missing".to_string(),
                                });
                            }
                        }
                    }

                    if status == StatusCode::NOT_FOUND {
                        return Err(IndexError::NotFound {
                            name: name.to_string(),
                        });
                    }

                    if RETRYABLE_CODES.contains(&status.as_u16()) {
                        if attempt >= self.retry_max {
                            return Err(IndexError::NetworkError {
                                url: url.clone(),
                                detail: format!(
                                    "retryable status {} after {} attempts",
                                    status,
                                    attempt + 1
                                ),
                            });
                        }
                        backoff_sleep(attempt).await;
                        attempt += 1;
                        continue;
                    }

                    if !status.is_success() {
                        return Err(IndexError::NetworkError {
                            url: url.clone(),
                            detail: format!("unexpected HTTP status {}", status),
                        });
                    }

                    // Inspect content-type to choose parser. Extract ETag before consuming body.
                    let content_type = response
                        .headers()
                        .get("content-type")
                        .and_then(|v| v.to_str().ok())
                        .unwrap_or("")
                        .to_string();

                    // REQ: AC4 — extract ETag before consuming body.
                    let etag_value = response
                        .headers()
                        .get(reqwest::header::ETAG)
                        .and_then(|v| v.to_str().ok())
                        .map(|s| s.to_string());

                    let body = response
                        .text()
                        .await
                        .map_err(|e| IndexError::NetworkError {
                            url: url.clone(),
                            detail: format!("failed to read response body: {e}"),
                        })?;

                    let is_json = content_type
                        .split(';')
                        .next()
                        .map(|ct| ct.trim())
                        .unwrap_or("")
                        == "application/vnd.pypi.simple.v1+json";

                    let meta = if is_json {
                        parse_simple_json(name, &body)
                    } else {
                        parse_simple_html(name, &body)
                    }
                    .map_err(|e| match e {
                        IndexError::ParseError { detail, .. } => IndexError::ParseError {
                            url: url.clone(),
                            detail,
                        },
                        other => other,
                    })?;

                    // Best-effort cache write — ignore errors so the hot path never fails.
                    let _ = write_cached_metadata(&cache_dir, name, "simple", &meta).await;

                    // REQ: AC4 — persist ETag sidecar for future conditional requests.
                    if let Some(etag_str) = etag_value {
                        let _ = write_cached_etag(&cache_dir, name, "simple", &etag_str).await;
                    }

                    return Ok(meta);
                }
            }
        }
    }

    /// Download a distribution artifact from the index, verifying its sha256 digest.
    ///
    /// Pipeline (per spec §Cache Layout diagram):
    /// 1. Check cache: if `{cache_dir}/artifacts/{name}/{filename}` exists with a valid
    ///    sidecar and the bytes match, return the cached path immediately.
    /// 2. GET `file.url` with reqwest streaming (`.bytes_stream()`). Write chunks to
    ///    `{artifact_path}.tmp` while feeding a `Sha256` hasher.
    /// 3. On EOF: compare digest to `file.hash.digest` (case-insensitive).
    ///    - Match → write sidecar, atomic rename `.tmp → final`, return path.
    ///    - Mismatch → delete `.tmp` (and sidecar if partially written), return
    ///      [`IndexError::HashMismatch`].
    /// 4. Same retry policy as other methods (429/5xx exponential backoff).
    ///
    /// # Errors
    ///
    /// - [`IndexError::HashMismatch`] — downloaded digest doesn't match `file.hash.digest`
    /// - [`IndexError::NotFound`] — HTTP 404 for the artifact URL
    /// - [`IndexError::NetworkError`] — transport error or non-retryable status after retries
    /// - [`IndexError::CacheIo`] — failed to write artifact or sidecar to disk
    pub async fn download_artifact(
        &self,
        name: &str,
        file: &ReleaseFile,
    ) -> Result<PathBuf, IndexError> {
        let cache_dir = self.effective_cache_dir();

        // Cache hit path — verify sidecar + bytes match.
        if let Some(cached) =
            read_cached_artifact(&cache_dir, name, &file.filename, &file.hash).await
        {
            return Ok(cached);
        }

        // Ensure the artifacts directory exists.
        let dest = artifact_path(&cache_dir, name, &file.filename);
        if let Some(parent) = dest.parent() {
            tokio::fs::create_dir_all(parent)
                .await
                .map_err(|e| IndexError::CacheIo {
                    path: parent.display().to_string(),
                    detail: format!("create_dir_all failed: {e}"),
                })?;
        }
        let tmp_path = PathBuf::from(format!("{}.tmp", dest.display()));
        let sidecar_path = PathBuf::from(format!("{}.sha256", dest.display()));

        // Content-addressed cache hit — same wheel may already be staked under
        // a different package name (e.g. extras, sibling releases sharing
        // identical bytes). On a verified CAS hit, hard-link the shared blob
        // back to the name-addressed slot and write the sha256 sidecar so the
        // sidecar invariants used elsewhere still hold.
        if !file.hash.digest.is_empty() {
            if let Some(cas_path) =
                read_content_addressed_artifact(&cache_dir, &file.hash.digest).await
            {
                let linked = {
                    let src = cas_path.clone();
                    let dst = dest.clone();
                    tokio::task::spawn_blocking(move || std::fs::hard_link(&src, &dst))
                        .await
                        .ok()
                        .and_then(|r| r.ok())
                        .is_some()
                };
                let materialized = if linked {
                    true
                } else {
                    // Hard-link failed (cross-fs or unsupported). Fall back to copy.
                    tokio::fs::copy(&cas_path, &dest).await.is_ok()
                };
                if materialized {
                    let digest_lower = file.hash.digest.to_lowercase();
                    tokio::fs::write(&sidecar_path, &digest_lower)
                        .await
                        .map_err(|e| IndexError::CacheIo {
                            path: sidecar_path.display().to_string(),
                            detail: format!("write sidecar (CAS link) failed: {e}"),
                        })?;
                    return Ok(dest);
                }
                // Fall through to network download if both link and copy failed.
            }
        }

        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(self.timeout_secs))
            .build()
            .map_err(|e| IndexError::NetworkError {
                url: file.url.clone(),
                detail: format!("failed to build HTTP client: {e}"),
            })?;

        let mut attempt = 0u32;
        loop {
            let response = match client.get(&file.url).send().await {
                Err(e) => {
                    if attempt >= self.retry_max {
                        return Err(IndexError::NetworkError {
                            url: file.url.clone(),
                            detail: e.to_string(),
                        });
                    }
                    backoff_sleep(attempt).await;
                    attempt += 1;
                    continue;
                }
                Ok(r) => r,
            };

            let status = response.status();

            if status == StatusCode::NOT_FOUND {
                return Err(IndexError::NotFound {
                    name: file.filename.clone(),
                });
            }

            if RETRYABLE_CODES.contains(&status.as_u16()) {
                if attempt >= self.retry_max {
                    return Err(IndexError::NetworkError {
                        url: file.url.clone(),
                        detail: format!(
                            "retryable status {} after {} attempts",
                            status,
                            attempt + 1
                        ),
                    });
                }
                backoff_sleep(attempt).await;
                attempt += 1;
                continue;
            }

            if !status.is_success() {
                return Err(IndexError::NetworkError {
                    url: file.url.clone(),
                    detail: format!("unexpected HTTP status {}", status),
                });
            }

            // Stream bytes to .tmp while hashing.
            let mut hasher = Sha256::new();
            let mut stream = response.bytes_stream();

            // Open .tmp for writing.
            let mut tmp_file =
                tokio::fs::File::create(&tmp_path)
                    .await
                    .map_err(|e| IndexError::CacheIo {
                        path: tmp_path.display().to_string(),
                        detail: format!("create .tmp failed: {e}"),
                    })?;

            use tokio::io::AsyncWriteExt;
            while let Some(chunk_result) = stream.next().await {
                let chunk = chunk_result.map_err(|e| {
                    // Clean up .tmp on stream error.
                    let _ = std::fs::remove_file(&tmp_path);
                    IndexError::NetworkError {
                        url: file.url.clone(),
                        detail: format!("stream read error: {e}"),
                    }
                })?;
                hasher.update(&chunk);
                tmp_file.write_all(&chunk).await.map_err(|e| {
                    let _ = std::fs::remove_file(&tmp_path);
                    IndexError::CacheIo {
                        path: tmp_path.display().to_string(),
                        detail: format!("write to .tmp failed: {e}"),
                    }
                })?;
            }

            // Flush and close the tmp file before rename.
            tmp_file.flush().await.map_err(|e| IndexError::CacheIo {
                path: tmp_path.display().to_string(),
                detail: format!("flush .tmp failed: {e}"),
            })?;
            drop(tmp_file);

            // Finalize hash and compare.
            let actual_digest = format!("{:x}", hasher.finalize());
            let expected_digest = file.hash.digest.to_lowercase();

            if actual_digest != expected_digest {
                // Mismatch — clean up and fail.
                let _ = tokio::fs::remove_file(&tmp_path).await;
                let _ = tokio::fs::remove_file(&sidecar_path).await;
                return Err(IndexError::HashMismatch {
                    filename: file.filename.clone(),
                    expected: expected_digest,
                    actual: actual_digest,
                });
            }

            // Write sidecar (no trailing newline per spec).
            tokio::fs::write(&sidecar_path, &actual_digest)
                .await
                .map_err(|e| IndexError::CacheIo {
                    path: sidecar_path.display().to_string(),
                    detail: format!("write sidecar failed: {e}"),
                })?;

            // Atomic rename .tmp → final path.
            tokio::fs::rename(&tmp_path, &dest)
                .await
                .map_err(|e| IndexError::CacheIo {
                    path: dest.display().to_string(),
                    detail: format!("rename .tmp → final failed: {e}"),
                })?;

            // Best-effort: promote the verified bytes into the content-addressed
            // store so future downloads of the same digest (across different
            // package names or versions) can hard-link instead of re-fetching.
            // CAS promotion failures are non-fatal: the name-addressed copy is
            // already good and the next caller will simply re-download.
            let _ = promote_to_content_addressed(&cache_dir, &dest, &actual_digest).await;

            return Ok(dest);
        }
    }

    /// Fetch package metadata, trying the JSON API first and falling back to
    /// the Simple API on `IndexError::NotFound`.
    ///
    /// Strategy:
    /// 1. Call [`Self::fetch_metadata_json`] (`GET /pypi/{name}/json`).
    /// 2. If that returns [`IndexError::NotFound`] → call [`Self::fetch_metadata_simple`].
    /// 3. Any other error from step 1 → propagate immediately (do not fall through).
    ///
    /// # Errors
    ///
    /// Returns the first non-`NotFound` error encountered, or `NotFound` if
    /// both strategies fail to find the package.
    pub async fn fetch_metadata(&self, name: &str) -> Result<PackageMetadata, IndexError> {
        match self.fetch_metadata_json(name).await {
            Ok(meta) => Ok(meta),
            Err(IndexError::NotFound { .. }) => self.fetch_metadata_simple(name).await,
            Err(other) => Err(other),
        }
    }

    /// Fetch metadata for multiple packages concurrently with a bounded in-flight limit.
    ///
    /// Uses a [`tokio::sync::Semaphore`] to cap the number of simultaneous in-flight
    /// HTTP requests at `max_in_flight`. Each package is fetched via [`Self::fetch_metadata`]
    /// (JSON primary + Simple fallback). Per-package failures do NOT abort the batch —
    /// the error is captured in the corresponding result slot instead.
    ///
    /// The returned `Vec` preserves the input order: element `i` corresponds to `names[i]`.
    ///
    /// # Arguments
    ///
    /// * `names` — package names to fetch (PEP 503 normalization applied internally)
    /// * `max_in_flight` — maximum number of concurrent HTTP requests; suggested default: 8
    ///
    /// # Returns
    ///
    /// A `Vec<(String, Result<PackageMetadata, IndexError>)>` in the same order as `names`.
    pub async fn fetch_metadata_batch(
        &self,
        names: Vec<String>,
        max_in_flight: usize,
    ) -> Vec<(String, Result<PackageMetadata, IndexError>)> {
        // REQ: R5 — bounded concurrency via semaphore
        let semaphore = Arc::new(Semaphore::new(max_in_flight));

        // Build one future per package. We use local async blocks (not tokio::spawn)
        // so that `&self` borrows are valid without requiring 'static. The futures are
        // driven concurrently by `futures_util::future::join_all`.
        let futures: Vec<_> = names
            .into_iter()
            .map(|name| {
                let sem = Arc::clone(&semaphore);
                async move {
                    // Acquire permit before starting the HTTP work.
                    let _permit = sem.acquire().await.expect("semaphore never closed");
                    let result = self.fetch_metadata(&name).await;
                    (name, result)
                }
            })
            .collect();

        // Drive all futures concurrently; results are in input order.
        futures_util::future::join_all(futures).await
    }

    /// Fetch per-version metadata and return the `requires_dist` PEP 508
    /// requirement strings (raw — caller parses them). Endpoint:
    /// `{index_url}/pypi/{pep503-name}/{version}/json`.
    ///
    /// Simpler than [`Self::fetch_metadata_json`] — no cache, no ETag,
    /// no conditional. Tick 13.5 wires this for transitive resolution;
    /// later ticks can layer caching on top once the call shape is stable.
    pub async fn fetch_version_requires(
        &self,
        name: &str,
        version: &str,
    ) -> Result<Vec<String>, IndexError> {
        use crate::pkgmanage::pkgmgr::json_api::parse_version_requires;
        let normalized = normalize_name(name);
        let url = format!(
            "{}/pypi/{}/{}/json",
            self.index_url.trim_end_matches('/'),
            normalized,
            version
        );

        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(self.timeout_secs))
            .build()
            .map_err(|e| IndexError::NetworkError {
                url: url.clone(),
                detail: format!("failed to build HTTP client: {e}"),
            })?;

        let mut attempt = 0u32;
        loop {
            let result = client.get(&url).send().await;
            match result {
                Err(e) => {
                    if attempt >= self.retry_max {
                        return Err(IndexError::NetworkError {
                            url: url.clone(),
                            detail: e.to_string(),
                        });
                    }
                    backoff_sleep(attempt).await;
                    attempt += 1;
                    continue;
                }
                Ok(response) => {
                    let status = response.status();
                    if status == StatusCode::NOT_FOUND {
                        // Treat 404 on a per-version endpoint as "no
                        // dependencies declared" — older / private indexes
                        // don't expose this route at all.
                        return Ok(Vec::new());
                    }
                    if RETRYABLE_CODES.contains(&status.as_u16()) {
                        if attempt >= self.retry_max {
                            return Err(IndexError::NetworkError {
                                url: url.clone(),
                                detail: format!(
                                    "retryable status {} after {} attempts",
                                    status,
                                    attempt + 1
                                ),
                            });
                        }
                        backoff_sleep(attempt).await;
                        attempt += 1;
                        continue;
                    }
                    if !status.is_success() {
                        return Err(IndexError::NetworkError {
                            url: url.clone(),
                            detail: format!("unexpected HTTP status {}", status),
                        });
                    }
                    let body = response
                        .text()
                        .await
                        .map_err(|e| IndexError::NetworkError {
                            url: url.clone(),
                            detail: format!("failed to read response body: {e}"),
                        })?;
                    return parse_version_requires(&body).map_err(|e| match e {
                        IndexError::ParseError { detail, .. } => IndexError::ParseError {
                            url: url.clone(),
                            detail,
                        },
                        other => other,
                    });
                }
            }
        }
    }
}

/// Sleep with exponential backoff and ±20% uniform jitter.
///
/// Base: 200 ms, multiplier: 2×. Attempt 0 → ~200 ms, attempt 1 → ~400 ms, etc.
async fn backoff_sleep(attempt: u32) {
    let base_ms = 200u64;
    let delay_ms = base_ms * (1u64 << attempt.min(10));
    // jitter: uniform ±20% of delay
    let jitter_range = (delay_ms as f64 * 0.2) as i64;
    let jitter_ms = rand::thread_rng().gen_range(-jitter_range..=jitter_range);
    let final_ms = (delay_ms as i64 + jitter_ms).max(1) as u64;
    tokio::time::sleep(Duration::from_millis(final_ms)).await;
}

#[cfg(test)]
mod tests {
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    use super::*;
    use crate::pkgmanage::pkgmgr::types::{FileHash, IndexClient, ReleaseFile};

    /// Create a test client with an isolated temp cache dir.
    fn make_client_with_cache(base_url: &str, cache_dir: &str) -> IndexClient {
        IndexClient {
            index_url: base_url.to_string(),
            cache_dir: cache_dir.to_string(),
            max_concurrent: 4,
            timeout_secs: 10,
            retry_max: 3,
        }
    }

    fn make_client(base_url: &str) -> IndexClient {
        use std::sync::atomic::{AtomicU64, Ordering};
        static COUNTER: AtomicU64 = AtomicU64::new(0);
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        let n = COUNTER.fetch_add(1, Ordering::Relaxed);
        IndexClient {
            index_url: base_url.to_string(),
            cache_dir: format!("/tmp/mamba-test-cache-{nanos}-{n}"),
            max_concurrent: 4,
            timeout_secs: 10,
            retry_max: 3,
        }
    }

    /// Minimal canned PyPI JSON body for package "requests" version "2.31.0".
    fn requests_json() -> String {
        r#"{
            "info": {
                "name": "requests",
                "version": "2.31.0",
                "requires_python": ">=3.7"
            },
            "releases": {
                "2.31.0": [
                    {
                        "filename": "requests-2.31.0-py3-none-any.whl",
                        "url": "https://files.pythonhosted.org/packages/requests-2.31.0-py3-none-any.whl",
                        "digests": {
                            "sha256": "58cd2187423839aa6e34d77a8f45b4a28a5f3e0e8c7e6b7b0e7e8e8e8e8e8e8e"
                        },
                        "requires_python": ">=3.7",
                        "yanked": false
                    }
                ]
            }
        }"#.to_string()
    }

    /// Minimal PEP 691 JSON body for content-negotiation tests.
    fn simple_json_body() -> String {
        r#"{
            "meta": { "api-version": "1.0" },
            "name": "requests",
            "files": [
                {
                    "filename": "requests-2.31.0-py3-none-any.whl",
                    "url": "https://files.example.com/requests-2.31.0-py3-none-any.whl",
                    "hashes": { "sha256": "abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890" },
                    "requires-python": ">=3.7"
                }
            ]
        }"#.to_string()
    }

    /// Minimal PEP 503 HTML body for fallback tests.
    fn simple_html_body() -> String {
        r#"<!DOCTYPE html>
<html>
<body>
<a href="https://files.example.com/requests-2.31.0-py3-none-any.whl#sha256=abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890"
   data-requires-python=">=3.7">requests-2.31.0-py3-none-any.whl</a>
</body>
</html>"#.to_string()
    }

    // REQ: fetch_metadata_json_success — 200 with valid JSON returns PackageMetadata
    #[tokio::test]
    async fn test_fetch_metadata_json_success() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/pypi/requests/json"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_string(requests_json())
                    .insert_header("content-type", "application/json"),
            )
            .mount(&server)
            .await;

        let client = make_client(&server.uri());
        let meta = client
            .fetch_metadata_json("requests")
            .await
            .expect("should succeed");

        assert_eq!(meta.name, "requests");
        assert!(
            !meta.versions.is_empty(),
            "versions should have at least 1 entry"
        );
        assert!(meta.releases.contains_key("2.31.0"));
    }

    // REQ: fetch_metadata_json_404_returns_not_found — 404 → IndexError::NotFound with matching name
    #[tokio::test]
    async fn test_fetch_metadata_json_404_returns_not_found() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/pypi/nonexistent-pkg/json"))
            .respond_with(ResponseTemplate::new(404))
            .mount(&server)
            .await;

        let client = make_client(&server.uri());
        let err = client
            .fetch_metadata_json("nonexistent-pkg")
            .await
            .expect_err("should fail with NotFound");

        match err {
            IndexError::NotFound { name } => {
                assert_eq!(name, "nonexistent-pkg");
            }
            other => panic!("expected NotFound, got: {:?}", other),
        }
    }

    // REQ: fetch_metadata_json_retry_on_503_then_success — 503 twice then 200 → success
    // Verifies retry wiring: client must retry on retryable status codes.
    #[tokio::test]
    async fn test_fetch_metadata_json_retry_on_503_then_success() {
        let server = MockServer::start().await;

        // First two responses: 503; third: 200 with valid JSON.
        // wiremock 0.6 supports `up_to_n_times` for ordered response cycling.
        Mock::given(method("GET"))
            .and(path("/pypi/requests/json"))
            .respond_with(ResponseTemplate::new(503))
            .up_to_n_times(2)
            .with_priority(1)
            .mount(&server)
            .await;

        Mock::given(method("GET"))
            .and(path("/pypi/requests/json"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_string(requests_json())
                    .insert_header("content-type", "application/json"),
            )
            .mount(&server)
            .await;

        // Use a very short backoff by overriding retry_max; the test still uses
        // real backoff_sleep so we need it to be fast — patch: create the client
        // with retry_max=3 (default) but the mock resolves in-process so
        // backoff is the only wall-clock wait. We cannot easily shorten it in
        // the current design without adding cfg(test) hooks. Accept the ~600ms
        // wait (200ms + 400ms) as acceptable for an offline unit test.
        let client = make_client(&server.uri());
        let meta = client
            .fetch_metadata_json("requests")
            .await
            .expect("should succeed after retries");

        assert_eq!(meta.name, "requests");
        assert!(meta.releases.contains_key("2.31.0"));
    }

    // REQ: test_fetch_metadata_simple_content_neg_json — content-type json → json parser used
    #[tokio::test]
    async fn test_fetch_metadata_simple_content_neg_json() {
        let server = MockServer::start().await;
        // Use set_body_raw so the mime type is not overridden by set_body_string.
        Mock::given(method("GET"))
            .and(path("/simple/requests/"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_raw(simple_json_body(), "application/vnd.pypi.simple.v1+json"),
            )
            .mount(&server)
            .await;

        let client = make_client(&server.uri());
        let meta = client
            .fetch_metadata_simple("requests")
            .await
            .expect("should succeed with JSON response");

        assert_eq!(meta.name, "requests");
        assert_eq!(meta.source, "simple-api");
        assert!(
            meta.releases.contains_key("2.31.0"),
            "version 2.31.0 should be present"
        );
    }

    // REQ: test_fetch_metadata_simple_html_fallback — content-type text/html → html parser used
    #[tokio::test]
    async fn test_fetch_metadata_simple_html_fallback() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/simple/requests/"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_raw(simple_html_body(), "text/html; charset=utf-8"),
            )
            .mount(&server)
            .await;

        let client = make_client(&server.uri());
        let meta = client
            .fetch_metadata_simple("requests")
            .await
            .expect("should succeed with HTML response");

        assert_eq!(meta.name, "requests");
        assert_eq!(meta.source, "simple-api");
        assert!(
            meta.releases.contains_key("2.31.0"),
            "version 2.31.0 should be present"
        );
    }

    // REQ: test_fetch_metadata_falls_back_on_404 — /pypi/.../json 404 → falls back to simple/ → success
    #[tokio::test]
    async fn test_fetch_metadata_falls_back_on_404() {
        let server = MockServer::start().await;

        // JSON API returns 404
        Mock::given(method("GET"))
            .and(path("/pypi/requests/json"))
            .respond_with(ResponseTemplate::new(404))
            .mount(&server)
            .await;

        // Simple API returns valid HTML
        Mock::given(method("GET"))
            .and(path("/simple/requests/"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_raw(simple_html_body(), "text/html; charset=utf-8"),
            )
            .mount(&server)
            .await;

        let client = make_client(&server.uri());
        let meta = client
            .fetch_metadata("requests")
            .await
            .expect("should succeed via simple-api fallback");

        assert_eq!(
            meta.source, "simple-api",
            "should be sourced from simple-api after fallback"
        );
        assert!(meta.releases.contains_key("2.31.0"));
    }

    // REQ: test_download_artifact_mock_success_verifies_sha256
    // wiremock serves known bytes; download succeeds, sidecar written, returned path exists.
    #[tokio::test]
    async fn test_download_artifact_mock_success_verifies_sha256() {
        use sha2::{Digest, Sha256};

        let tmp = tempfile::TempDir::new().unwrap();
        let server = MockServer::start().await;

        let content = b"fake-wheel-bytes-for-sha256-test";
        let mut hasher = Sha256::new();
        hasher.update(content);
        let digest = format!("{:x}", hasher.finalize());

        Mock::given(method("GET"))
            .and(path("/files/mypkg-1.0.0-py3-none-any.whl"))
            .respond_with(ResponseTemplate::new(200).set_body_bytes(content.to_vec()))
            .mount(&server)
            .await;

        let client = make_client_with_cache(&server.uri(), tmp.path().to_str().unwrap());
        let file = ReleaseFile {
            filename: "mypkg-1.0.0-py3-none-any.whl".to_string(),
            url: format!("{}/files/mypkg-1.0.0-py3-none-any.whl", server.uri()),
            hash: FileHash {
                algorithm: "sha256".to_string(),
                digest: digest.clone(),
            },
            requires_python: None,
            size: None,
            upload_time: None,
            yanked: false,
            yanked_reason: None,
            dist_info_metadata: serde_json::Value::Null,
            source: Some("json-api".to_string()),
        };

        let result = client.download_artifact("mypkg", &file).await;
        assert!(
            result.is_ok(),
            "download should succeed: {:?}",
            result.err()
        );
        let path = result.unwrap();
        assert!(path.exists(), "downloaded file should exist on disk");

        // Verify sidecar.
        let sidecar_path = std::path::PathBuf::from(format!("{}.sha256", path.display()));
        assert!(sidecar_path.exists(), "sidecar should exist");
        let sidecar_content = std::fs::read_to_string(&sidecar_path).unwrap();
        assert_eq!(
            sidecar_content.trim(),
            digest,
            "sidecar should contain correct digest"
        );
    }

    // REQ: test_download_artifact_hash_mismatch_raises
    // wiremock serves bytes that don't match file.hash.digest → IndexError::HashMismatch,
    // .tmp cleaned.
    #[tokio::test]
    async fn test_download_artifact_hash_mismatch_raises() {
        let tmp = tempfile::TempDir::new().unwrap();
        let server = MockServer::start().await;

        let content = b"real-content-bytes";
        let wrong_digest = "a".repeat(64); // wrong hash

        Mock::given(method("GET"))
            .and(path("/files/mypkg-2.0.0-py3-none-any.whl"))
            .respond_with(ResponseTemplate::new(200).set_body_bytes(content.to_vec()))
            .mount(&server)
            .await;

        let client = make_client_with_cache(&server.uri(), tmp.path().to_str().unwrap());
        let file = ReleaseFile {
            filename: "mypkg-2.0.0-py3-none-any.whl".to_string(),
            url: format!("{}/files/mypkg-2.0.0-py3-none-any.whl", server.uri()),
            hash: FileHash {
                algorithm: "sha256".to_string(),
                digest: wrong_digest.clone(),
            },
            requires_python: None,
            size: None,
            upload_time: None,
            yanked: false,
            yanked_reason: None,
            dist_info_metadata: serde_json::Value::Null,
            source: Some("json-api".to_string()),
        };

        let err = client
            .download_artifact("mypkg", &file)
            .await
            .expect_err("should fail with HashMismatch");

        assert!(
            matches!(err, IndexError::HashMismatch { .. }),
            "expected HashMismatch, got: {:?}",
            err
        );

        // .tmp file should be cleaned up.
        use crate::pkgmanage::pkgmgr::cache::artifact_path;
        let dest = artifact_path(tmp.path(), "mypkg", "mypkg-2.0.0-py3-none-any.whl");
        let tmp_file = std::path::PathBuf::from(format!("{}.tmp", dest.display()));
        assert!(
            !tmp_file.exists(),
            ".tmp file should be deleted after mismatch"
        );
    }

    // REQ: test_fetch_metadata_json_cache_hit_skips_network
    // First call hits wiremock (1 HTTP request). Second call returns cached data
    // (wiremock verifies exactly 1 request served via received_requests).
    #[tokio::test]
    async fn test_fetch_metadata_json_cache_hit_skips_network() {
        let tmp = tempfile::TempDir::new().unwrap();
        let server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/pypi/requests/json"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_string(requests_json())
                    .insert_header("content-type", "application/json"),
            )
            .expect(1) // exactly 1 HTTP request should be served
            .mount(&server)
            .await;

        let client = make_client_with_cache(&server.uri(), tmp.path().to_str().unwrap());

        // First call — cache miss → HTTP request.
        let meta1 = client
            .fetch_metadata_json("requests")
            .await
            .expect("first call should succeed");
        assert_eq!(meta1.name, "requests");

        // Second call — cache hit → no HTTP request (wiremock .expect(1) enforces this).
        let meta2 = client
            .fetch_metadata_json("requests")
            .await
            .expect("second call should succeed from cache");
        assert_eq!(meta2.name, "requests");
        assert_eq!(meta1.versions, meta2.versions);

        // wiremock verifies the mock expectation (.expect(1)) was satisfied on server drop.
        server.verify().await;
    }

    // REQ: AC3 — download_artifact returns HashMismatch with correct expected/actual fields
    // when the simple API advertises a wrong sha256 (64-zero hex) but the server serves
    // real bytes whose actual digest differs.
    #[tokio::test]
    async fn test_download_artifact_hash_mismatch_fields_populated() {
        use sha2::{Digest, Sha256};

        let tmp = tempfile::TempDir::new().unwrap();
        let server = MockServer::start().await;

        let content = b"wheel-bytes-42";
        // Compute the real digest so we can verify it appears in the error's `actual` field.
        let mut h = Sha256::new();
        h.update(content);
        let real_digest = format!("{:x}", h.finalize());

        // Advertise 64 zeros — intentionally wrong.
        let wrong_digest = "0".repeat(64);

        Mock::given(method("GET"))
            .and(path("/pkg/foo-1.0-py3-none-any.whl"))
            .respond_with(ResponseTemplate::new(200).set_body_bytes(content.to_vec()))
            .mount(&server)
            .await;

        let client = make_client_with_cache(&server.uri(), tmp.path().to_str().unwrap());
        let file = ReleaseFile {
            filename: "foo-1.0-py3-none-any.whl".to_string(),
            url: format!("{}/pkg/foo-1.0-py3-none-any.whl", server.uri()),
            hash: FileHash {
                algorithm: "sha256".to_string(),
                digest: wrong_digest.clone(),
            },
            requires_python: None,
            size: None,
            upload_time: None,
            yanked: false,
            yanked_reason: None,
            dist_info_metadata: serde_json::Value::Null,
            source: Some("simple-api".to_string()),
        };

        let err = client
            .download_artifact("foo", &file)
            .await
            .expect_err("should fail with HashMismatch");

        match err {
            IndexError::HashMismatch {
                expected, actual, ..
            } => {
                assert_eq!(
                    expected, wrong_digest,
                    "expected field must echo the advertised hash"
                );
                assert_eq!(
                    actual, real_digest,
                    "actual field must contain the true digest"
                );
            }
            other => panic!("expected HashMismatch, got: {:?}", other),
        }
    }

    // REQ: D1 — batch happy path: 3 names all succeed, order preserved
    #[tokio::test]
    async fn test_fetch_metadata_batch_all_success_order_preserved() {
        let server = MockServer::start().await;

        // Mount JSON API mocks for three packages.
        for pkg in ["alpha", "beta", "gamma"] {
            let body = format!(
                r#"{{
                    "info": {{"name": "{pkg}", "version": "1.0.0", "requires_python": ">=3.8"}},
                    "releases": {{
                        "1.0.0": [{{
                            "filename": "{pkg}-1.0.0-py3-none-any.whl",
                            "url": "https://example.com/{pkg}-1.0.0-py3-none-any.whl",
                            "digests": {{"sha256": "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"}},
                            "yanked": false
                        }}]
                    }}
                }}"#
            );
            Mock::given(method("GET"))
                .and(path(format!("/pypi/{pkg}/json")))
                .respond_with(
                    ResponseTemplate::new(200)
                        .set_body_string(body)
                        .insert_header("content-type", "application/json"),
                )
                .mount(&server)
                .await;
        }

        let client = make_client(&server.uri());
        let names = vec!["alpha".to_string(), "beta".to_string(), "gamma".to_string()];
        // max_in_flight = 2 exercises the semaphore code path with 3 requests.
        let results = client.fetch_metadata_batch(names.clone(), 2).await;

        assert_eq!(results.len(), 3, "should have 3 results");

        // Verify order and success.
        for (i, (name, result)) in results.iter().enumerate() {
            assert_eq!(name, &names[i], "result order must match input order");
            let meta = result.as_ref().expect("should succeed");
            assert_eq!(meta.name, names[i]);
        }
    }

    // REQ: AC4 — 200 with ETag header writes ETag sidecar to cache.
    #[tokio::test]
    async fn test_fetch_metadata_json_200_writes_etag_sidecar() {
        use crate::pkgmanage::pkgmgr::cache::read_cached_etag;

        let tmp = tempfile::TempDir::new().unwrap();
        let server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/pypi/requests/json"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_string(requests_json())
                    .insert_header("content-type", "application/json")
                    .insert_header("etag", "W/\"etag-v1\""),
            )
            .mount(&server)
            .await;

        let client = make_client_with_cache(&server.uri(), tmp.path().to_str().unwrap());
        let meta = client
            .fetch_metadata_json("requests")
            .await
            .expect("should succeed");
        assert_eq!(meta.name, "requests");

        let etag = read_cached_etag(tmp.path(), "requests", "json").await;
        assert_eq!(
            etag,
            Some("W/\"etag-v1\"".to_string()),
            "ETag sidecar should be written"
        );
    }

    // REQ: AC4 — 304 Not Modified returns stale cached metadata without a new network body.
    #[tokio::test]
    async fn test_fetch_metadata_json_304_returns_stale_cached_metadata() {
        use crate::pkgmanage::pkgmgr::cache::{
            metadata_path, write_cached_etag, write_cached_metadata,
        };
        use wiremock::matchers::header;

        let tmp = tempfile::TempDir::new().unwrap();
        let server = MockServer::start().await;

        // Pre-populate cache with stale metadata + ETag sidecar.
        let pre_meta = {
            let body = requests_json();
            crate::pkgmanage::pkgmgr::json_api::parse_json_metadata(&body).expect("parse ok")
        };
        write_cached_metadata(tmp.path(), "requests", "json", &pre_meta)
            .await
            .expect("write metadata ok");
        write_cached_etag(tmp.path(), "requests", "json", "W/\"abc\"")
            .await
            .expect("write etag ok");

        // Expire the TTL by backdating the metadata file mtime using libc::utimes.
        let meta_file = metadata_path(tmp.path(), "requests", "json");
        let c_path = std::ffi::CString::new(meta_file.to_str().unwrap()).unwrap();
        // Set both atime and mtime to 1 second past epoch.
        let ancient = libc::timeval {
            tv_sec: 1,
            tv_usec: 0,
        };
        unsafe { libc::utimes(c_path.as_ptr(), [ancient, ancient].as_ptr()) };

        // Mock returns 304 when If-None-Match matches.
        Mock::given(method("GET"))
            .and(path("/pypi/requests/json"))
            .and(header("if-none-match", "W/\"abc\""))
            .respond_with(ResponseTemplate::new(304))
            .mount(&server)
            .await;

        let client = make_client_with_cache(&server.uri(), tmp.path().to_str().unwrap());
        let result = client.fetch_metadata_json("requests").await;
        let meta = result.expect("304 should return Ok with stale metadata");
        assert_eq!(meta.name, "requests");
        assert!(meta.releases.contains_key("2.31.0"));
    }

    // REQ: AC4 — end-to-end 200→304 round-trip: first fetch populates cache + ETag sidecar
    // from a 200 response; second fetch sends If-None-Match from the cached ETag, receives 304,
    // returns the same metadata from cache without a new response body.
    #[tokio::test]
    async fn test_fetch_metadata_json_200_then_304_end_to_end_round_trip() {
        use crate::pkgmanage::pkgmgr::cache::{metadata_path, read_cached_etag};
        use wiremock::matchers::header;

        let tmp = tempfile::TempDir::new().unwrap();
        let server = MockServer::start().await;

        // FIRST REQUEST: ordered 200 response with ETag — served ONCE via up_to_n_times(1).
        // Mock ordering in wiremock 0.6: mocks registered first match first; combining up_to_n_times(1)
        // on the 200 mock + a catch-all 304 mock gives exactly-one-then-304 sequencing.
        Mock::given(method("GET"))
            .and(path("/pypi/requests/json"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_string(requests_json())
                    .insert_header("content-type", "application/json")
                    .insert_header("etag", "W/\"round-trip-v1\""),
            )
            .up_to_n_times(1)
            .mount(&server)
            .await;

        // SECOND REQUEST: 304 response — matched ONLY when If-None-Match header is present and
        // equals the ETag from the first response. This asserts the client correctly reads its
        // own cached ETag sidecar and attaches it on the follow-up request.
        Mock::given(method("GET"))
            .and(path("/pypi/requests/json"))
            .and(header("if-none-match", "W/\"round-trip-v1\""))
            .respond_with(ResponseTemplate::new(304))
            .mount(&server)
            .await;

        let client = make_client_with_cache(&server.uri(), tmp.path().to_str().unwrap());

        // First call — serves 200, populates both metadata cache + ETag sidecar.
        let meta1 = client
            .fetch_metadata_json("requests")
            .await
            .expect("first fetch (200) should succeed");
        assert_eq!(meta1.name, "requests");
        assert!(meta1.releases.contains_key("2.31.0"));

        // Verify ETag sidecar was written from the 200 response.
        let etag = read_cached_etag(tmp.path(), "requests", "json").await;
        assert_eq!(
            etag,
            Some("W/\"round-trip-v1\"".to_string()),
            "first 200 response must write ETag sidecar"
        );

        // Expire the TTL so the second call re-hits the network (and triggers the 304 path).
        // Backdates the cached metadata mtime to epoch+1s using libc::utimes — same pattern as
        // test_fetch_metadata_json_304_returns_stale_cached_metadata (L1184-1189).
        let meta_file = metadata_path(tmp.path(), "requests", "json");
        let c_path = std::ffi::CString::new(meta_file.to_str().unwrap()).unwrap();
        let ancient = libc::timeval {
            tv_sec: 1,
            tv_usec: 0,
        };
        unsafe { libc::utimes(c_path.as_ptr(), [ancient, ancient].as_ptr()) };

        // Second call — sends If-None-Match from cached ETag, receives 304, returns cached metadata.
        let meta2 = client
            .fetch_metadata_json("requests")
            .await
            .expect("second fetch (304) should return Ok with cached metadata");
        assert_eq!(meta2.name, "requests");
        assert_eq!(
            meta2
                .releases
                .keys()
                .collect::<std::collections::BTreeSet<_>>(),
            meta1
                .releases
                .keys()
                .collect::<std::collections::BTreeSet<_>>(),
            "304 round-trip must return the same metadata set as the 200 response"
        );

        // Verify exactly 2 requests were served (1 x 200, 1 x 304) — not 1 (TTL didn't expire)
        // and not 3+ (retry loops). Counts received_requests, not mock.expect().
        let received = server.received_requests().await.unwrap_or_default();
        assert_eq!(
            received.len(),
            2,
            "round-trip should serve exactly 2 HTTP requests (200 + 304), got {}",
            received.len()
        );
    }

    // REQ: D1 — batch mixed: 1 success, 1 404 NotFound, 1 success; Err in middle slot
    #[tokio::test]
    async fn test_fetch_metadata_batch_mixed_success_and_not_found() {
        let server = MockServer::start().await;

        // "pkgone" succeeds.
        Mock::given(method("GET"))
            .and(path("/pypi/pkgone/json"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_string(
                        r#"{"info":{"name":"pkgone","version":"1.0.0"},"releases":{"1.0.0":[{"filename":"pkgone-1.0.0-py3-none-any.whl","url":"https://example.com/pkgone-1.0.0.whl","digests":{"sha256":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"},"yanked":false}]}}"#,
                    )
                    .insert_header("content-type", "application/json"),
            )
            .mount(&server)
            .await;

        // "nosuchpkg" returns 404 on both JSON and Simple endpoints.
        Mock::given(method("GET"))
            .and(path("/pypi/nosuchpkg/json"))
            .respond_with(ResponseTemplate::new(404))
            .mount(&server)
            .await;
        Mock::given(method("GET"))
            .and(path("/simple/nosuchpkg/"))
            .respond_with(ResponseTemplate::new(404))
            .mount(&server)
            .await;

        // "pkgtwo" succeeds.
        Mock::given(method("GET"))
            .and(path("/pypi/pkgtwo/json"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_string(
                        r#"{"info":{"name":"pkgtwo","version":"2.0.0"},"releases":{"2.0.0":[{"filename":"pkgtwo-2.0.0-py3-none-any.whl","url":"https://example.com/pkgtwo-2.0.0.whl","digests":{"sha256":"bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"},"yanked":false}]}}"#,
                    )
                    .insert_header("content-type", "application/json"),
            )
            .mount(&server)
            .await;

        let client = make_client(&server.uri());
        let names = vec![
            "pkgone".to_string(),
            "nosuchpkg".to_string(),
            "pkgtwo".to_string(),
        ];
        // max_in_flight = 2 exercises semaphore.
        let results = client.fetch_metadata_batch(names.clone(), 2).await;

        assert_eq!(results.len(), 3);

        // Slot 0: pkgone → Ok
        assert_eq!(results[0].0, "pkgone");
        assert!(results[0].1.is_ok(), "pkgone should succeed");

        // Slot 1: nosuchpkg → Err(NotFound)
        assert_eq!(results[1].0, "nosuchpkg");
        match &results[1].1 {
            Err(IndexError::NotFound { name }) => assert_eq!(name, "nosuchpkg"),
            other => panic!("expected NotFound for nosuchpkg, got: {:?}", other),
        }

        // Slot 2: pkgtwo → Ok
        assert_eq!(results[2].0, "pkgtwo");
        assert!(results[2].1.is_ok(), "pkgtwo should succeed");
    }
}
