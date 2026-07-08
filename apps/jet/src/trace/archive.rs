// SPEC-MANAGED: .aw/tech-design/projects/jet/semantic/jet-trace.md#schema
// CODEGEN-BEGIN
//! Zip archive writer for trace files.
//!
//! Creates a `.jet-trace` zip archive containing:
//! - `manifest.ndjson` — manifest header + NDJSON events
//! - `assets/<asset_id>` — binary assets (DOM snapshots, PNG screenshots)
//!
// @spec enhancement-native-trace-viewer-trace-capture-standalone-html-spec#R3

use crate::trace::manifest::{encode_ndjson, TraceManifest};
use anyhow::{Context, Result};
use std::io::Write;
use std::path::Path;

/// A binary asset to be bundled inside the trace zip.
/// @spec .aw/tech-design/projects/jet/semantic/jet-trace.md#schema
#[derive(Debug, Clone)]
pub struct TraceAsset {
    /// The asset id used as the filename under `assets/` in the zip.
    pub id: String,
    /// Raw bytes (PNG, HTML, etc.).
    pub bytes: Vec<u8>,
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-trace.md#schema
impl TraceAsset {
    pub fn new(id: impl Into<String>, bytes: Vec<u8>) -> Self {
        Self {
            id: id.into(),
            bytes,
        }
    }
}

/// Write a complete trace archive to `out_path`.
///
/// The archive contains:
/// - `manifest.ndjson` — manifest header + events
/// - `assets/<id>` for each `TraceAsset`
///
/// The manifest's `assets` map is updated to point each asset id to its
/// zip entry path before writing.
// @spec enhancement-native-trace-viewer-trace-capture-standalone-html-spec#R3
pub fn write_trace_zip(
    manifest: &mut TraceManifest,
    assets: &[TraceAsset],
    out_path: &Path,
) -> Result<()> {
    // Update the assets map in the manifest before serialising.
    for asset in assets {
        let zip_entry = format!("assets/{}", asset.id);
        manifest.assets.insert(asset.id.clone(), zip_entry);
    }

    // Create parent directory if needed.
    if let Some(parent) = out_path.parent() {
        std::fs::create_dir_all(parent).with_context(|| {
            format!(
                "Failed to create trace output directory: {}",
                parent.display()
            )
        })?;
    }

    let file = std::fs::File::create(out_path)
        .with_context(|| format!("Failed to create trace archive: {}", out_path.display()))?;

    let mut zip = zip::ZipWriter::new(file);
    let options = zip::write::SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated)
        .unix_permissions(0o644);

    // Write manifest.ndjson
    let ndjson_bytes =
        encode_ndjson(manifest).context("Failed to encode trace manifest as NDJSON")?;
    zip.start_file("manifest.ndjson", options)
        .context("Failed to start manifest.ndjson zip entry")?;
    zip.write_all(&ndjson_bytes)
        .context("Failed to write manifest.ndjson")?;

    // Write assets
    for asset in assets {
        let entry_name = format!("assets/{}", asset.id);
        zip.start_file(&entry_name, options)
            .with_context(|| format!("Failed to start zip entry: {entry_name}"))?;
        zip.write_all(&asset.bytes)
            .with_context(|| format!("Failed to write zip asset: {entry_name}"))?;
    }

    zip.finish()
        .context("Failed to finalise trace zip archive")?;
    Ok(())
}

/// Read the `manifest.ndjson` entry from a trace zip archive and return the
/// parsed `TraceManifest`.
// @spec enhancement-native-trace-viewer-trace-capture-standalone-html-spec#R5
pub fn read_manifest_from_zip(zip_path: &Path) -> Result<TraceManifest> {
    let file = std::fs::File::open(zip_path)
        .with_context(|| format!("Failed to open trace archive: {}", zip_path.display()))?;
    let mut archive = zip::ZipArchive::new(file)
        .with_context(|| format!("Failed to parse zip archive: {}", zip_path.display()))?;

    let mut entry = archive
        .by_name("manifest.ndjson")
        .context("Trace archive missing manifest.ndjson entry")?;

    let mut bytes = Vec::new();
    std::io::Read::read_to_end(&mut entry, &mut bytes)
        .context("Failed to read manifest.ndjson from zip")?;

    crate::trace::manifest::decode_ndjson(&bytes)
}

/// Read a named asset from a trace zip archive.
// @spec enhancement-native-trace-viewer-trace-capture-standalone-html-spec#R5
pub fn read_asset_from_zip(zip_path: &Path, asset_zip_entry: &str) -> Result<Vec<u8>> {
    let file = std::fs::File::open(zip_path)
        .with_context(|| format!("Failed to open trace archive: {}", zip_path.display()))?;
    let mut archive = zip::ZipArchive::new(file)
        .with_context(|| format!("Failed to parse zip archive: {}", zip_path.display()))?;

    let mut entry = archive
        .by_name(asset_zip_entry)
        .with_context(|| format!("Asset not found in trace archive: {asset_zip_entry}"))?;

    let mut bytes = Vec::new();
    std::io::Read::read_to_end(&mut entry, &mut bytes)
        .with_context(|| format!("Failed to read asset: {asset_zip_entry}"))?;

    Ok(bytes)
}
// CODEGEN-END
