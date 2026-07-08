// SPEC-MANAGED: libs/service-backup/tech-design/semantic/source/libs-service-backup-src-source-rs.md#rust-source-unit
// CODEGEN-BEGIN
use anyhow::{bail, ensure, Context, Result};

#[cfg(feature = "s3")]
use crate::s3;

/// Fetch an exact backup object URI.
///
/// This is intentionally narrower than [`crate::BackupDestination`], whose URI
/// form names a sink prefix. Bootstrap and restore paths need one concrete
/// snapshot object instead: `file:///path/to/snapshot.json` or
/// `s3://bucket/path/to/snapshot.json`.
/// @spec libs/service-backup/tech-design/semantic/source/libs-service-backup-src-source-rs.md#source
pub fn fetch_backup_object(raw_uri: &str) -> Result<Vec<u8>> {
    let uri = raw_uri.trim();
    ensure!(!uri.is_empty(), "backup object URI is empty");
    if let Some(path) = uri.strip_prefix("file://") {
        ensure!(!path.is_empty(), "file backup object URI has no path");
        return std::fs::read(path).with_context(|| format!("read backup object {uri}"));
    }
    if let Some(rest) = uri.strip_prefix("s3://") {
        let (bucket, key) = split_bucket_key(rest, "s3")?;
        #[cfg(feature = "s3")]
        {
            return s3::get_object(bucket, key);
        }
        #[cfg(not(feature = "s3"))]
        {
            let _ = (bucket, key);
            bail!(
                "backup object {uri} requires the service-backup `s3` feature; rebuild with --features s3 or use file://"
            );
        }
    }
    if uri.starts_with("gs://") {
        bail!(
            "backup object {uri} parses as GCS, but service-backup does not yet ship a GCS source; use file:// or s3://"
        );
    }
    bail!("unsupported backup object URI `{uri}`; use file://, s3://, or gs://")
}

fn split_bucket_key(rest: &str, scheme: &str) -> Result<(String, String)> {
    let Some((bucket, key)) = rest.split_once('/') else {
        bail!("{scheme} backup object URI must include bucket and key");
    };
    ensure!(
        !bucket.is_empty(),
        "{scheme} backup object URI has no bucket"
    );
    let key = key.trim_start_matches('/');
    ensure!(!key.is_empty(), "{scheme} backup object URI has no key");
    Ok((bucket.to_string(), key.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fetches_exact_file_object() {
        let dir =
            std::env::temp_dir().join(format!("service-backup-source-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("snapshot.json");
        std::fs::write(&path, b"snapshot").unwrap();

        let uri = format!("file://{}", path.display());
        assert_eq!(fetch_backup_object(&uri).unwrap(), b"snapshot");

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn s3_object_uri_requires_bucket_and_key() {
        assert!(fetch_backup_object("s3://bucket").is_err());
        assert!(fetch_backup_object("s3:///key").is_err());
        assert!(fetch_backup_object("s3://bucket/").is_err());
    }
}
// CODEGEN-END
