// SPEC-MANAGED: libs/service-backup/tech-design/semantic/source/libs-service-backup-src-s3-rs.md#rust-source-unit
// CODEGEN-BEGIN
use std::future::Future;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use anyhow::{anyhow, bail, Context, Result};
use aws_config::{BehaviorVersion, Region};
use aws_sdk_s3::{primitives::ByteStream, Client};

use crate::{BackupDestination, BackupSink};

const DEFAULT_S3_REGION: &str = "us-east-1";
const OBJECT_NAME_PREFIX: &str = "backup-";
const OBJECT_NAME_SUFFIX: &str = ".json";

#[derive(Clone)]
/// @spec libs/service-backup/tech-design/semantic/source/libs-service-backup-src-s3-rs.md#source
pub(crate) struct S3Sink {
    bucket: String,
    prefix: String,
    region: Option<String>,
    endpoint: Option<String>,
}

/// @spec libs/service-backup/tech-design/semantic/source/libs-service-backup-src-s3-rs.md#source
impl S3Sink {
    pub(crate) fn from_destination(destination: &BackupDestination) -> Result<Self> {
        let BackupDestination::S3 {
            bucket,
            prefix,
            region,
            endpoint,
            credentials_secret,
        } = destination
        else {
            bail!("{} is not an S3 backup destination", destination.identity());
        };

        if let Some(secret) = credentials_secret {
            bail!(
                "backup destination {} sets credentials_secret `{secret}`, but service-backup/s3 does not yet load secret-mounted credentials; use ambient AWS credentials or omit credentials_secret",
                destination.identity()
            );
        }

        let bucket = bucket.clone();
        let prefix = normalize_prefix(prefix);
        let region = region
            .clone()
            .or_else(|| endpoint.as_ref().map(|_| DEFAULT_S3_REGION.to_string()));
        let endpoint = endpoint.clone();

        Ok(Self {
            bucket,
            prefix,
            region,
            endpoint,
        })
    }

    fn key_for_timestamp(&self, timestamp: SystemTime) -> String {
        build_key(&self.prefix, unix_seconds(timestamp))
    }
}

/// @spec libs/service-backup/tech-design/semantic/source/libs-service-backup-src-s3-rs.md#source
impl BackupSink for S3Sink {
    fn put(&self, timestamp: SystemTime, payload: &[u8]) -> Result<String> {
        let bucket = self.bucket.clone();
        let key = self.key_for_timestamp(timestamp);
        let body = payload.to_vec();
        let region = self.region.clone();
        let endpoint = self.endpoint.clone();
        let key_for_result = key.clone();
        block_on_in_thread(async move {
            let client = build_client(region, endpoint).await?;
            client
                .put_object()
                .bucket(&bucket)
                .key(&key)
                .body(ByteStream::from(body))
                .send()
                .await
                .with_context(|| format!("put s3://{bucket}/{key}"))?;
            Ok(key_for_result)
        })
    }

    fn prune(&self, max_age_seconds: u64) -> Result<usize> {
        let cutoff = SystemTime::now()
            .checked_sub(Duration::from_secs(max_age_seconds))
            .unwrap_or(UNIX_EPOCH);
        let bucket = self.bucket.clone();
        let prefix = self.prefix.clone();
        let region = self.region.clone();
        let endpoint = self.endpoint.clone();
        block_on_in_thread(async move {
            let client = build_client(region, endpoint).await?;
            prune_matching_objects(client, bucket, prefix, unix_seconds(cutoff)).await
        })
    }

    fn identity(&self) -> String {
        if self.prefix.is_empty() {
            format!("s3://{}", self.bucket)
        } else {
            format!("s3://{}/{}", self.bucket, self.prefix)
        }
    }
}

/// @spec libs/service-backup/tech-design/semantic/source/libs-service-backup-src-s3-rs.md#source
pub(crate) fn get_object(bucket: String, key: String) -> Result<Vec<u8>> {
    block_on_in_thread(async move {
        let client = build_client(None, None).await?;
        let object = client
            .get_object()
            .bucket(&bucket)
            .key(&key)
            .send()
            .await
            .with_context(|| format!("get s3://{bucket}/{key}"))?;
        let bytes = object
            .body
            .collect()
            .await
            .with_context(|| format!("read s3://{bucket}/{key} body"))?
            .into_bytes()
            .to_vec();
        Ok(bytes)
    })
}

async fn build_client(region: Option<String>, endpoint: Option<String>) -> Result<Client> {
    let mut loader = aws_config::defaults(BehaviorVersion::latest());
    if let Some(region) = region {
        loader = loader.region(Region::new(region));
    }
    let shared = loader.load().await;
    let mut builder = aws_sdk_s3::config::Builder::from(&shared);
    if let Some(endpoint) = endpoint {
        builder = builder.endpoint_url(endpoint).force_path_style(true);
    }
    Ok(Client::from_conf(builder.build()))
}

async fn prune_matching_objects(
    client: Client,
    bucket: String,
    prefix: String,
    cutoff_unix_seconds: u64,
) -> Result<usize> {
    let list_prefix = (!prefix.is_empty()).then(|| format!("{prefix}/"));
    let mut removed = 0usize;
    let mut continuation = None;

    loop {
        let mut request = client.list_objects_v2().bucket(&bucket);
        if let Some(prefix) = list_prefix.as_deref() {
            request = request.prefix(prefix);
        }
        if let Some(token) = continuation.as_deref() {
            request = request.continuation_token(token);
        }
        let response = request
            .send()
            .await
            .with_context(|| match list_prefix.as_deref() {
                Some(prefix) => format!("list s3://{bucket}/{}", prefix.trim_end_matches('/')),
                None => format!("list s3://{bucket}"),
            })?;

        for object in response.contents() {
            let Some(key) = object.key() else {
                continue;
            };
            let Some(unix_seconds) = parse_backup_key(&prefix, key) else {
                continue;
            };
            if unix_seconds < cutoff_unix_seconds {
                client
                    .delete_object()
                    .bucket(&bucket)
                    .key(key)
                    .send()
                    .await
                    .with_context(|| format!("delete s3://{bucket}/{key}"))?;
                removed += 1;
            }
        }

        continuation = response.next_continuation_token().map(ToOwned::to_owned);
        if !response.is_truncated().unwrap_or(false) {
            break;
        }
    }

    Ok(removed)
}

fn block_on_in_thread<F, T>(future: F) -> Result<T>
where
    F: Future<Output = Result<T>> + Send + 'static,
    T: Send + 'static,
{
    std::thread::spawn(move || -> Result<T> {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .context("build tokio runtime for service-backup/s3")?;
        runtime.block_on(future)
    })
    .join()
    .map_err(|_| anyhow!("service-backup/s3 worker thread panicked"))?
}

fn unix_seconds(timestamp: SystemTime) -> u64 {
    timestamp
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

fn normalize_prefix(prefix: &str) -> String {
    prefix.trim_matches('/').to_string()
}

fn build_key(prefix: &str, unix_seconds: u64) -> String {
    let name = format!("{OBJECT_NAME_PREFIX}{unix_seconds}{OBJECT_NAME_SUFFIX}");
    if prefix.is_empty() {
        name
    } else {
        format!("{prefix}/{name}")
    }
}

fn parse_backup_key(prefix: &str, key: &str) -> Option<u64> {
    let name = if prefix.is_empty() {
        key
    } else {
        key.strip_prefix(prefix)?.strip_prefix('/')?
    };
    name.strip_prefix(OBJECT_NAME_PREFIX)?
        .strip_suffix(OBJECT_NAME_SUFFIX)?
        .parse()
        .ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_destination(prefix: &str) -> BackupDestination {
        BackupDestination::S3 {
            bucket: "bucket".into(),
            prefix: prefix.into(),
            region: Some(DEFAULT_S3_REGION.into()),
            endpoint: Some("http://127.0.0.1:9".into()),
            credentials_secret: None,
        }
    }

    #[test]
    fn key_helpers_normalize_prefixes() {
        let sink = S3Sink::from_destination(&test_destination("/nested/prefix/")).unwrap();
        assert_eq!(sink.identity(), "s3://bucket/nested/prefix");
        assert_eq!(
            sink.key_for_timestamp(UNIX_EPOCH + Duration::from_secs(42)),
            "nested/prefix/backup-42.json"
        );
        assert_eq!(
            parse_backup_key("nested/prefix", "nested/prefix/backup-42.json"),
            Some(42)
        );
        assert_eq!(
            parse_backup_key("nested/prefix", "nested/prefix/not-a-backup.json"),
            None
        );
    }

    #[test]
    fn credentials_secret_is_explicitly_unsupported() {
        let err = match S3Sink::from_destination(&BackupDestination::S3 {
            bucket: "bucket".into(),
            prefix: "prefix".into(),
            region: Some(DEFAULT_S3_REGION.into()),
            endpoint: Some("http://127.0.0.1:9".into()),
            credentials_secret: Some("aws-creds".into()),
        }) {
            Ok(_) => panic!("credentials_secret should be rejected until secret loading exists"),
            Err(err) => err.to_string(),
        };
        assert!(err.contains("credentials_secret"));
        assert!(err.contains("aws-creds"));
    }

    #[test]
    fn integration_uploads_and_prunes_when_env_is_available() {
        let Some(destination) = integration_destination_from_env() else {
            eprintln!("skipping S3 integration test; SERVICE_BACKUP_S3_TEST_BUCKET not set");
            return;
        };

        let sink = S3Sink::from_destination(&destination).unwrap();
        let old_key = sink
            .put(UNIX_EPOCH + Duration::from_secs(10), b"old")
            .unwrap();
        let new_key = sink.put(SystemTime::now(), b"new").unwrap();
        assert!(old_key.contains("backup-10.json"));
        assert!(new_key.contains("backup-"));
        assert_eq!(sink.prune(60).unwrap(), 1);
        std::thread::sleep(Duration::from_secs(1));
        assert_eq!(sink.prune(0).unwrap(), 1);
    }

    fn integration_destination_from_env() -> Option<BackupDestination> {
        let bucket = std::env::var("SERVICE_BACKUP_S3_TEST_BUCKET").ok()?;
        let region = std::env::var("SERVICE_BACKUP_S3_TEST_REGION")
            .ok()
            .or_else(|| Some(DEFAULT_S3_REGION.into()));
        let endpoint = std::env::var("SERVICE_BACKUP_S3_TEST_ENDPOINT").ok();
        let base_prefix = normalize_prefix(
            &std::env::var("SERVICE_BACKUP_S3_TEST_PREFIX")
                .unwrap_or_else(|_| "service-backup-tests".into()),
        );
        let unique_prefix = format!(
            "{base_prefix}/pid-{}-{}",
            std::process::id(),
            unix_seconds(SystemTime::now())
        );
        Some(BackupDestination::S3 {
            bucket,
            prefix: unique_prefix,
            region,
            endpoint,
            credentials_secret: None,
        })
    }
}
// CODEGEN-END
