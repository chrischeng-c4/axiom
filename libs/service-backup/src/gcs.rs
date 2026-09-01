//! Compatibility wrapper over the shared `storage-object` GCS adapter.

use std::fmt;
use std::sync::Arc;
use std::time::SystemTime;

use anyhow::{bail, Context, Result};
use chrono::{DateTime, Utc};
use storage_object::{GcsObjectStore, ObjectStore, PutCondition};

use crate::{BackupDestination, BackupSink};

#[derive(Clone)]
pub struct GcsSink {
    bucket: String,
    prefix: String,
    store: Arc<dyn ObjectStore>,
}

impl fmt::Debug for GcsSink {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("GcsSink")
            .field("bucket", &self.bucket)
            .field("prefix", &self.prefix)
            .finish_non_exhaustive()
    }
}

impl GcsSink {
    pub fn from_destination(destination: &BackupDestination) -> Result<Self> {
        let BackupDestination::Gcs {
            bucket,
            prefix,
            credentials_secret,
        } = destination
        else {
            bail!("{} is not a GCS backup destination", destination.identity());
        };
        if let Some(secret) = credentials_secret {
            bail!(
                "GCS credentials_secret `{secret}` is not supported; use ADC and GKE Workload Identity"
            );
        }
        Ok(Self {
            bucket: bucket.clone(),
            prefix: if prefix.is_empty() {
                "backup".to_string()
            } else {
                prefix.trim_matches('/').to_string()
            },
            store: Arc::new(GcsObjectStore::new(bucket, "")?),
        })
    }

    pub fn from_exact_uri(uri: &str) -> Result<(Self, String)> {
        let (bucket, key) = split_gs_uri(uri)?;
        let store = Arc::new(GcsObjectStore::new(&bucket, "")?);
        Ok((
            Self {
                bucket,
                prefix: String::new(),
                store,
            },
            key,
        ))
    }

    pub fn put_object(&self, key: &str, payload: &[u8], content_type: &str) -> Result<String> {
        let key = key.trim_start_matches('/');
        self.store
            .put(key, payload, content_type, PutCondition::Any)?;
        Ok(format!("gs://{}/{key}", self.bucket))
    }

    pub fn get_object(&self, key: &str) -> Result<Vec<u8>> {
        Ok(self.store.get(key.trim_start_matches('/'))?.bytes)
    }

    pub fn delete_object(&self, key: &str) -> Result<()> {
        self.store.delete(key.trim_start_matches('/'))?;
        Ok(())
    }

    fn list_objects(&self) -> Result<Vec<(String, DateTime<Utc>)>> {
        self.store
            .list(&self.prefix)?
            .into_iter()
            .map(|meta| {
                let updated = meta
                    .updated
                    .as_deref()
                    .context("GCS object metadata lacks updated time")?;
                Ok((
                    meta.key,
                    DateTime::parse_from_rfc3339(updated)?.with_timezone(&Utc),
                ))
            })
            .collect()
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
