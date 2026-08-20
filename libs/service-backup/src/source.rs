// SPEC-MANAGED: libs/service-backup/tech-design/semantic/source/libs-service-backup-src-source-rs.md#rust-source-unit
// CODEGEN-BEGIN
//! Fetch one exact backup object, for bootstrap and restore.
//!
//! This is narrower than [`BackupDestination`](crate::BackupDestination) on
//! purpose, and the distinction is the whole reason the module exists: a
//! destination URI names a **prefix** that a sink writes many objects under, while
//! a source URI names **one concrete object** —
//! `file:///path/to/snapshot.json` or `s3://bucket/path/to/snapshot.json`. So
//! `s3://bucket/prefix` is a valid destination and a wrong source.
//!
//! The `s3://` arm is feature-gated, and the failure is deliberately at **run
//! time**, not compile time: a binary built without the `s3` feature still
//! accepts an `s3://` source URI in its configuration and only fails when the
//! restore actually runs, with a message naming the rebuild flag. `file://` and
//! `gs://` are always available.
//!
//! The rejection message enumerates
//! [`SUPPORTED_SCHEMES`], which lives in
//! `destination.rs` and is that file's inventory of what *destinations* accept.
//! Adding a scheme to that table without adding an arm here therefore produces an
//! error message advertising a scheme this function rejects -- the two must move
//! together.
use anyhow::{bail, ensure, Context, Result};

use crate::destination::SUPPORTED_SCHEMES;
use crate::gcs;
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
        return gcs::get_exact_object(uri);
    }
    bail!(
        "unsupported backup object URI `{uri}`; use {}",
        SUPPORTED_SCHEMES
            .iter()
            .map(|s| s.scheme)
            .collect::<Vec<_>>()
            .join(", ")
    )
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

    #[test]
    fn unsupported_scheme_error_lists_every_supported_scheme() {
        let err = fetch_backup_object("ftp://nope").unwrap_err().to_string();
        for info in SUPPORTED_SCHEMES {
            assert!(
                err.contains(info.scheme),
                "error message {err:?} missing scheme {}",
                info.scheme
            );
        }
    }
}
// CODEGEN-END
