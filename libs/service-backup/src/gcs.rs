// HANDWRITE-BEGIN gap="service-backup-gcs-adapter" tracker="1659" reason="Implement GCS JSON API put/list/prune/get with Vat emulator endpoint support and ADC workload-identity bearer tokens."
//! Google Cloud Storage JSON API adapter for the synchronous backup contract.
//!
//! `STORAGE_EMULATOR_HOST` selects Vat's real HTTP emulator and disables auth.
//! Production uses an explicit access token or the GCE/GKE metadata server,
//! which is the workload-identity path. Object writes return only after GCS has
//! acknowledged the media upload.

use std::time::{Duration, SystemTime};

use anyhow::{bail, Context, Result};
use chrono::{DateTime, Utc};
use reqwest::blocking::{Client, RequestBuilder};
use serde_json::Value;

use crate::{BackupDestination, BackupSink};

#[derive(Debug, Clone)]
pub struct GcsSink {
    bucket: String,
    prefix: String,
    endpoint: String,
    emulator: bool,
    client: Client,
}

impl GcsSink {
    pub fn from_destination(destination: &BackupDestination) -> Result<Self> {
        let BackupDestination::Gcs { bucket, prefix, .. } = destination else {
            bail!("{} is not a GCS backup destination", destination.identity());
        };
        let (endpoint, emulator) = match std::env::var("STORAGE_EMULATOR_HOST") {
            Ok(value) => {
                let value = if value.starts_with("http://") || value.starts_with("https://") {
                    value
                } else {
                    format!("http://{value}")
                };
                (value.trim_end_matches('/').to_string(), true)
            }
            Err(std::env::VarError::NotPresent) => {
                ("https://storage.googleapis.com".to_string(), false)
            }
            Err(error) => return Err(error.into()),
        };
        Ok(Self {
            bucket: bucket.clone(),
            prefix: if prefix.is_empty() {
                "backup".to_string()
            } else {
                prefix.trim_matches('/').to_string()
            },
            endpoint,
            emulator,
            client: Client::builder().timeout(Duration::from_secs(60)).build()?,
        })
    }

    pub fn from_exact_uri(uri: &str) -> Result<(Self, String)> {
        let (bucket, key) = split_gs_uri(uri)?;
        let destination = BackupDestination::Gcs {
            bucket,
            prefix: String::new(),
            credentials_secret: None,
        };
        Ok((Self::from_destination(&destination)?, key))
    }

    pub fn put_object(&self, key: &str, payload: &[u8], content_type: &str) -> Result<String> {
        let key = key.trim_start_matches('/');
        let url = format!(
            "{}/upload/storage/v1/b/{}/o?uploadType=media&name={}",
            self.endpoint,
            urlencoding::encode(&self.bucket),
            urlencoding::encode(key)
        );
        let response = self
            .authorized(self.client.post(url))?
            .header(reqwest::header::CONTENT_TYPE, content_type)
            .body(payload.to_vec())
            .send()
            .context("upload GCS backup object")?;
        ensure_success(response, "upload GCS backup object")?;
        Ok(format!("gs://{}/{key}", self.bucket))
    }

    pub fn get_object(&self, key: &str) -> Result<Vec<u8>> {
        let url = format!(
            "{}/download/storage/v1/b/{}/o/{}?alt=media",
            self.endpoint,
            urlencoding::encode(&self.bucket),
            urlencoding::encode(key.trim_start_matches('/'))
        );
        let response = self
            .authorized(self.client.get(url))?
            .send()
            .context("download GCS backup object")?;
        Ok(ensure_success(response, "download GCS backup object")?
            .bytes()?
            .to_vec())
    }

    pub fn delete_object(&self, key: &str) -> Result<()> {
        let url = format!(
            "{}/storage/v1/b/{}/o/{}",
            self.endpoint,
            urlencoding::encode(&self.bucket),
            urlencoding::encode(key.trim_start_matches('/'))
        );
        let response = self.authorized(self.client.delete(url))?.send()?;
        ensure_success(response, "delete GCS backup object")?;
        Ok(())
    }

    fn list_objects(&self) -> Result<Vec<(String, DateTime<Utc>)>> {
        let mut objects = Vec::new();
        let mut page_token = None::<String>;
        loop {
            let mut url = format!(
                "{}/storage/v1/b/{}/o?prefix={}",
                self.endpoint,
                urlencoding::encode(&self.bucket),
                urlencoding::encode(&self.prefix)
            );
            if let Some(token) = &page_token {
                url.push_str("&pageToken=");
                url.push_str(&urlencoding::encode(token));
            }
            let response = self.authorized(self.client.get(url))?.send()?;
            let value: Value = ensure_success(response, "list GCS backup objects")?.json()?;
            objects.extend(
                value
                    .get("items")
                    .and_then(Value::as_array)
                    .into_iter()
                    .flatten()
                    .filter_map(|item| {
                        Some((
                            item.get("name")?.as_str()?.to_string(),
                            DateTime::parse_from_rfc3339(item.get("updated")?.as_str()?)
                                .ok()?
                                .with_timezone(&Utc),
                        ))
                    }),
            );
            page_token = value
                .get("nextPageToken")
                .and_then(Value::as_str)
                .filter(|token| !token.is_empty())
                .map(str::to_string);
            if page_token.is_none() {
                break;
            }
        }
        Ok(objects)
    }

    fn authorized(&self, request: RequestBuilder) -> Result<RequestBuilder> {
        if self.emulator {
            return Ok(request);
        }
        Ok(request.bearer_auth(access_token(&self.client)?))
    }
}

impl BackupSink for GcsSink {
    fn put(&self, timestamp: SystemTime, payload: &[u8]) -> Result<String> {
        let seconds = timestamp
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        let key = format!("{}-{seconds}.json", self.prefix);
        self.put_object(&key, payload, "application/json")?;
        Ok(key)
    }

    fn prune(&self, max_age_seconds: u64) -> Result<usize> {
        let max_age_seconds = i64::try_from(max_age_seconds).unwrap_or(i64::MAX);
        let cutoff = Utc::now() - chrono::Duration::seconds(max_age_seconds);
        let mut removed = 0;
        for (key, updated) in self.list_objects()? {
            if updated < cutoff {
                self.delete_object(&key)?;
                removed += 1;
            }
        }
        Ok(removed)
    }

    fn identity(&self) -> String {
        format!("gs://{}/{}", self.bucket, self.prefix)
    }
}

pub fn get_exact_object(uri: &str) -> Result<Vec<u8>> {
    let (sink, key) = GcsSink::from_exact_uri(uri)?;
    sink.get_object(&key)
}

fn split_gs_uri(uri: &str) -> Result<(String, String)> {
    let rest = uri
        .trim()
        .strip_prefix("gs://")
        .context("GCS object URI must start with gs://")?;
    let (bucket, key) = rest
        .split_once('/')
        .context("GCS object URI must include bucket and key")?;
    if bucket.is_empty() || key.trim_matches('/').is_empty() {
        bail!("GCS object URI must include non-empty bucket and key");
    }
    Ok((bucket.to_string(), key.trim_start_matches('/').to_string()))
}

fn access_token(client: &Client) -> Result<String> {
    for name in ["GOOGLE_OAUTH_ACCESS_TOKEN", "GCS_ACCESS_TOKEN"] {
        if let Ok(value) = std::env::var(name) {
            if !value.trim().is_empty() {
                return Ok(value);
            }
        }
    }
    let metadata_host = std::env::var("GCE_METADATA_HOST")
        .unwrap_or_else(|_| "metadata.google.internal".to_string());
    let url = format!(
        "http://{metadata_host}/computeMetadata/v1/instance/service-accounts/default/token"
    );
    let response = client
        .get(url)
        .header("Metadata-Flavor", "Google")
        .send()
        .context("obtain GCS workload-identity token from metadata server")?;
    let value: Value = ensure_success(response, "obtain GCS workload-identity token")?.json()?;
    value
        .get("access_token")
        .and_then(Value::as_str)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
        .context("metadata token response lacks access_token")
}

fn ensure_success(
    response: reqwest::blocking::Response,
    action: &str,
) -> Result<reqwest::blocking::Response> {
    if response.status().is_success() {
        return Ok(response);
    }
    let status = response.status();
    let body = response.text().unwrap_or_default();
    bail!("{action} failed with HTTP {status}: {body}")
}
// HANDWRITE-END
