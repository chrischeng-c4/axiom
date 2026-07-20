//! Durable persistence framing — the header + codec every saved artifact shares.
//!
//! Beam is a real database, so its CPU-side source-of-truth survives a process
//! restart. Two things are persisted (each with its own magic so a file can't be
//! loaded as the wrong type):
//!
//! - the [`Collection`](crate::collection::Collection) **segment** — row-major
//!   vectors, payloads, external ids, and the `live` tombstone bits;
//! - the trained [`IvfPqIndex`](crate::index::ivf_pq::IvfPqIndex) **model** —
//!   coarse centroids, PQ codebooks, and the inverted lists / codes / residuals.
//!
//! The **GPU buffers are never persisted**: they are ephemeral and rebuilt from
//! the CPU-side state on load, exactly as they are for a freshly-built index (see
//! [`GpuFlatIndex::new`](crate::gpu::GpuFlatIndex::new) /
//! [`GpuIvfScanner`](crate::gpu::ivfpq::GpuIvfScanner)).
//!
//! ## On-disk format
//!
//! ```text
//! [ 8-byte magic ][ 4-byte little-endian version ][ bincode payload ]
//! ```
//!
//! The magic + version header makes a stale or foreign file detectable up front
//! (a wrong magic or an unknown version is a clean error, not a garbage decode),
//! and lets the format evolve — bump [`FORMAT_VERSION`] and branch on it here.
//! The payload is `bincode` (compact, non-self-describing, deterministic), keyed
//! to the `Serialize`/`Deserialize` derives on the persisted structs.

use std::fs::File;
use std::io::{BufReader, BufWriter, Read, Write};
use std::path::Path;

use anyhow::{bail, Context};
use serde::de::DeserializeOwned;
use serde::Serialize;

/// Magic for a persisted [`Collection`](crate::collection::Collection) segment.
pub const COLLECTION_MAGIC: &[u8; 8] = b"BEAMCOL\0";

/// Magic for a persisted [`IvfPqIndex`](crate::index::ivf_pq::IvfPqIndex) model.
pub const INDEX_MAGIC: &[u8; 8] = b"BEAMIVP\0";

/// The current on-disk format version. Bump this when the payload layout of a
/// persisted struct changes; [`load_framed`] rejects any other version.
pub const FORMAT_VERSION: u32 = 1;

// <HANDWRITE gap="missing-generator:logic" tracker="pending-tracker" reason="logic section in persist.rs is hand-written pending codegen support">
/// Write `value` to `path` behind the standard `magic` + [`FORMAT_VERSION`]
/// header, bincode-encoding the payload. Buffered + flushed so the whole artifact
/// lands in one pass.
pub fn save_framed<T: Serialize>(path: &Path, magic: &[u8; 8], value: &T) -> anyhow::Result<()> {
    let file = File::create(path).with_context(|| format!("create {}", path.display()))?;
    let mut writer = BufWriter::new(file);
    writer.write_all(magic).context("write magic")?;
    writer
        .write_all(&FORMAT_VERSION.to_le_bytes())
        .context("write version")?;
    bincode::serialize_into(&mut writer, value).context("bincode serialize payload")?;
    writer.flush().context("flush")?;
    Ok(())
}
// </HANDWRITE>

/// Read a `T` from `path`, validating the `magic` and [`FORMAT_VERSION`] header
/// before decoding the bincode payload. A wrong magic (foreign / mismatched file)
/// or an unknown version is a clean error rather than a garbage decode.
pub fn load_framed<T: DeserializeOwned>(path: &Path, magic: &[u8; 8]) -> anyhow::Result<T> {
    let file = File::open(path).with_context(|| format!("open {}", path.display()))?;
    let mut reader = BufReader::new(file);

    let mut got_magic = [0u8; 8];
    reader.read_exact(&mut got_magic).context("read magic")?;
    if &got_magic != magic {
        bail!(
            "bad magic in {}: expected {magic:?}, got {got_magic:?}",
            path.display()
        );
    }

    let mut version_bytes = [0u8; 4];
    reader
        .read_exact(&mut version_bytes)
        .context("read version")?;
    let version = u32::from_le_bytes(version_bytes);
    if version != FORMAT_VERSION {
        bail!(
            "unsupported beam format version {version} in {} (this build reads {FORMAT_VERSION})",
            path.display()
        );
    }

    bincode::deserialize_from(&mut reader).context("bincode deserialize payload")
}

/// A snapshot of the entire vector database: maps collection name to its deserialized [`Collection`](crate::collection::Collection).
#[derive(serde::Serialize, serde::Deserialize)]
pub struct BeamSnapshot {
    pub collections: std::collections::HashMap<String, crate::collection::Collection>,
}

impl BeamSnapshot {
    /// Encode as CBOR + lz4 (compact binary format matching lumen).
    pub fn encode(&self) -> anyhow::Result<Vec<u8>> {
        let mut raw = Vec::new();
        ciborium::into_writer(self, &mut raw)
            .map_err(|e| anyhow::anyhow!("cbor encode BeamSnapshot: {e}"))?;
        Ok(lz4_flex::compress_prepend_size(&raw))
    }

    /// Decode from CBOR + lz4.
    pub fn decode(bytes: &[u8]) -> anyhow::Result<Self> {
        let raw = lz4_flex::decompress_size_prepended(bytes)
            .context("lz4 decompress BeamSnapshot")?;
        let mut snap: Self = ciborium::from_reader(&raw[..])
            .map_err(|e| anyhow::anyhow!("cbor decode BeamSnapshot: {e}"))?;
        for col in snap.collections.values_mut() {
            col.rebuild_id_map();
        }
        Ok(snap)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, PartialEq, serde::Serialize, serde::Deserialize)]
    struct Sample {
        a: u32,
        b: Vec<f32>,
        c: String,
    }

    fn tmp(name: &str) -> std::path::PathBuf {
        let mut p = std::env::temp_dir();
        p.push(format!("beam_persist_unit_{}_{name}", std::process::id()));
        p
    }

    #[test]
    fn round_trips_a_value() {
        let path = tmp("roundtrip");
        let v = Sample {
            a: 7,
            b: vec![1.0, 2.5, -3.0],
            c: "hello".into(),
        };
        save_framed(&path, COLLECTION_MAGIC, &v).unwrap();
        let back: Sample = load_framed(&path, COLLECTION_MAGIC).unwrap();
        assert_eq!(v, back);
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn rejects_wrong_magic() {
        let path = tmp("wrongmagic");
        let v = Sample {
            a: 1,
            b: vec![],
            c: String::new(),
        };
        save_framed(&path, COLLECTION_MAGIC, &v).unwrap();
        // Loading a collection file as an index file must fail on the magic check.
        let err = load_framed::<Sample>(&path, INDEX_MAGIC).unwrap_err();
        assert!(err.to_string().contains("bad magic"), "{err}");
        let _ = std::fs::remove_file(&path);
    }
}
