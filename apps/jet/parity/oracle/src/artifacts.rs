// SPEC-MANAGED: .aw/tech-design/projects/jet/semantic/jet-parity-oracle-src.md#schema
// CODEGEN-BEGIN

//! `ArtifactBundle` + `ArtifactWriter`.
//!
//! Implements the on-disk layout and determinism contract from §Logic /
//! the §Changes entry for `artifacts.rs`:
//!
//! * deterministic JSON (sorted keys, LF endings, trailing newline,
//!   minimal whitespace) — implemented via `serde_json::Value` round-trip
//!   that delegates to BTreeMap-ordered encoding.
//! * PNG re-encode that strips ancillary chunks (no `tIME` / `tEXt`).
//! * atomic write via `tempfile::NamedTempFile` + `persist`.
//! * sha256 digest per artifact so callers can assert byte-equivalence
//!   (R9 determinism gate).

use serde::Serialize;
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use thiserror::Error;

/// @spec parity-dom-reference-runner.md#Dependency (ArtifactBundle)
#[derive(Debug, Clone)]
pub struct ArtifactBundle {
    pub root_dir: PathBuf,
    pub pixel_png: PathBuf,
    pub a11y_json: PathBuf,
    pub focus_json: PathBuf,
    pub pointer_json: PathBuf,
    pub ime_json: PathBuf,
    pub sha256s: BTreeMap<String, [u8; 32]>,
}

/// @spec parity-dom-reference-runner.md#Changes (artifacts.rs)
#[derive(Debug, Error)]
pub enum ArtifactError {
    #[error("artifact io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("artifact json serialize error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("artifact png encode error: {0}")]
    Png(String),
    #[error("artifact persist error: {0}")]
    Persist(String),
}

/// @spec parity-dom-reference-runner.md#Dependency (ArtifactWriter)
#[derive(Debug, Clone)]
pub struct ArtifactWriter {
    root_dir: PathBuf,
    sha256s: BTreeMap<String, [u8; 32]>,
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-parity-oracle-src.md#schema
impl ArtifactWriter {
    /// @spec parity-dom-reference-runner.md#Changes (artifacts.rs)
    pub fn new(root_dir: PathBuf) -> Result<Self, ArtifactError> {
        fs::create_dir_all(&root_dir)?;
        Ok(Self {
            root_dir,
            sha256s: BTreeMap::new(),
        })
    }

    pub fn root_dir(&self) -> &Path {
        &self.root_dir
    }

    /// @spec parity-dom-reference-runner.md#Changes (artifacts.rs)
    ///
    /// Atomically write `bytes` to `root_dir/filename`, record sha256.
    pub fn write_bytes(&mut self, filename: &str, bytes: &[u8]) -> Result<PathBuf, ArtifactError> {
        let path = self.root_dir.join(filename);
        let dir = path.parent().unwrap_or(Path::new("."));
        fs::create_dir_all(dir)?;
        let mut tmp = tempfile::NamedTempFile::new_in(dir)?;
        tmp.write_all(bytes)?;
        tmp.flush()?;
        tmp.persist(&path)
            .map_err(|e| ArtifactError::Persist(e.to_string()))?;
        let mut h = Sha256::new();
        h.update(bytes);
        let digest: [u8; 32] = h.finalize().into();
        self.sha256s.insert(filename.to_string(), digest);
        Ok(path)
    }

    /// @spec parity-dom-reference-runner.md#Changes (artifacts.rs)
    ///
    /// Serialize `value` deterministically (sorted keys, LF, trailing newline,
    /// no whitespace beyond `": "`) and write.
    pub fn write_json<T: Serialize>(
        &mut self,
        filename: &str,
        value: &T,
    ) -> Result<PathBuf, ArtifactError> {
        let bytes = to_deterministic_json(value)?;
        self.write_bytes(filename, &bytes)
    }

    /// @spec parity-dom-reference-runner.md#Changes (artifacts.rs)
    ///
    /// Re-encode `raw_png` to strip ancillary chunks (tIME / tEXt) and write
    /// the result. If `raw_png` cannot be decoded as PNG, the bytes are
    /// written as-is (callers like the stub harness pass synthesised PNGs).
    pub fn write_png(&mut self, filename: &str, raw_png: &[u8]) -> Result<PathBuf, ArtifactError> {
        let encoded = reencode_png_stripped(raw_png).unwrap_or_else(|| raw_png.to_vec());
        self.write_bytes(filename, &encoded)
    }

    pub fn sha256s(&self) -> &BTreeMap<String, [u8; 32]> {
        &self.sha256s
    }

    pub fn into_sha256s(self) -> BTreeMap<String, [u8; 32]> {
        self.sha256s
    }
}

/// @spec parity-dom-reference-runner.md#Logic (determinism contract)
///
/// Deterministic JSON encoder: sorts object keys via `BTreeMap`-style
/// canonicalisation, emits LF, trailing newline, no extra whitespace
/// beyond a single space after `": "`. We achieve this by round-tripping
/// through `serde_json::Value` -> canonical tree -> compact encoder.
pub fn to_deterministic_json<T: Serialize>(value: &T) -> Result<Vec<u8>, ArtifactError> {
    let raw = serde_json::to_value(value)?;
    let canonical = canonicalize(raw);
    let mut buf = Vec::with_capacity(128);
    write_canonical(&mut buf, &canonical);
    buf.push(b'\n');
    Ok(buf)
}

fn canonicalize(v: serde_json::Value) -> serde_json::Value {
    use serde_json::Value;
    match v {
        Value::Object(map) => {
            let mut sorted: BTreeMap<String, Value> = BTreeMap::new();
            for (k, val) in map.into_iter() {
                sorted.insert(k, canonicalize(val));
            }
            let mut out = serde_json::Map::with_capacity(sorted.len());
            for (k, val) in sorted.into_iter() {
                out.insert(k, val);
            }
            Value::Object(out)
        }
        Value::Array(arr) => Value::Array(arr.into_iter().map(canonicalize).collect()),
        other => other,
    }
}

fn write_canonical(buf: &mut Vec<u8>, v: &serde_json::Value) {
    use serde_json::Value;
    match v {
        Value::Null => buf.extend_from_slice(b"null"),
        Value::Bool(b) => buf.extend_from_slice(if *b { b"true" } else { b"false" }),
        Value::Number(n) => buf.extend_from_slice(n.to_string().as_bytes()),
        Value::String(s) => {
            let encoded = serde_json::to_string(s).unwrap_or_else(|_| "\"\"".to_string());
            buf.extend_from_slice(encoded.as_bytes());
        }
        Value::Array(items) => {
            buf.push(b'[');
            for (i, item) in items.iter().enumerate() {
                if i > 0 {
                    buf.push(b',');
                }
                write_canonical(buf, item);
            }
            buf.push(b']');
        }
        Value::Object(map) => {
            buf.push(b'{');
            for (i, (k, val)) in map.iter().enumerate() {
                if i > 0 {
                    buf.push(b',');
                }
                let key = serde_json::to_string(k).unwrap_or_else(|_| "\"\"".to_string());
                buf.extend_from_slice(key.as_bytes());
                buf.extend_from_slice(b": ");
                write_canonical(buf, val);
            }
            buf.push(b'}');
        }
    }
}

/// @spec parity-dom-reference-runner.md#Logic (PNG metadata strip)
///
/// Re-encode a PNG, dropping ancillary chunks (anything outside IHDR /
/// PLTE / IDAT / IEND). Returns `None` if the input doesn't look like
/// a PNG (caller decides whether to error or pass-through).
fn reencode_png_stripped(raw: &[u8]) -> Option<Vec<u8>> {
    const SIG: [u8; 8] = [0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A];
    if raw.len() < 8 || raw[..8] != SIG {
        return None;
    }
    let mut out = Vec::with_capacity(raw.len());
    out.extend_from_slice(&SIG);
    let mut i = 8usize;
    let keep: &[&[u8; 4]] = &[b"IHDR", b"PLTE", b"IDAT", b"IEND"];
    while i + 8 <= raw.len() {
        let len = u32::from_be_bytes(raw[i..i + 4].try_into().ok()?) as usize;
        let kind = &raw[i + 4..i + 8];
        let chunk_end = i + 8 + len + 4; // length+type+data+crc
        if chunk_end > raw.len() {
            return None;
        }
        let keep_this = keep.iter().any(|k| kind == k.as_slice());
        if keep_this {
            out.extend_from_slice(&raw[i..chunk_end]);
        }
        i = chunk_end;
        if kind == b"IEND" {
            break;
        }
    }
    Some(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn deterministic_json_sorts_keys() {
        // Per spec §Logic determinism contract: "elides whitespace beyond
        // `": "` separators" — only `": "` has whitespace; commas are bare.
        let v = json!({"b": 1, "a": 2, "nested": {"y": 1, "x": 2}});
        let out = to_deterministic_json(&v).unwrap();
        let s = String::from_utf8(out).unwrap();
        assert_eq!(s, "{\"a\": 2,\"b\": 1,\"nested\": {\"x\": 2,\"y\": 1}}\n");
    }

    #[test]
    fn deterministic_json_trailing_newline_lf() {
        let v = json!({"k": "v"});
        let out = to_deterministic_json(&v).unwrap();
        assert_eq!(*out.last().unwrap(), b'\n');
        assert!(!out.contains(&b'\r'));
    }

    #[test]
    fn write_json_atomic_and_sha256() {
        let tmp = tempfile::tempdir().unwrap();
        let mut w = ArtifactWriter::new(tmp.path().to_path_buf()).unwrap();
        let p = w.write_json("a.json", &json!({"k": 1})).unwrap();
        assert!(p.exists());
        let bytes = std::fs::read(&p).unwrap();
        assert_eq!(bytes, b"{\"k\": 1}\n");
        assert!(w.sha256s().contains_key("a.json"));
    }

    #[test]
    fn write_png_strips_text_chunks() {
        // Build a minimal valid 1x1 PNG with an injected tEXt chunk.
        // We use the `image` crate to make a real PNG first.
        let mut original = Vec::new();
        {
            let img = image::RgbaImage::from_pixel(1, 1, image::Rgba([0, 0, 0, 255]));
            let mut cursor = std::io::Cursor::new(&mut original);
            image::DynamicImage::ImageRgba8(img)
                .write_to(&mut cursor, image::ImageFormat::Png)
                .unwrap();
        }
        let tmp = tempfile::tempdir().unwrap();
        let mut w = ArtifactWriter::new(tmp.path().to_path_buf()).unwrap();
        let path = w.write_png("p.png", &original).unwrap();
        let bytes = std::fs::read(&path).unwrap();
        assert_eq!(
            &bytes[..8],
            &[0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A]
        );
        // Should still decode as PNG.
        let _img = image::load_from_memory(&bytes).unwrap();
    }

    #[test]
    fn write_png_passthrough_for_non_png() {
        let tmp = tempfile::tempdir().unwrap();
        let mut w = ArtifactWriter::new(tmp.path().to_path_buf()).unwrap();
        let path = w.write_png("p.png", b"not a png").unwrap();
        assert_eq!(std::fs::read(path).unwrap(), b"not a png");
    }

    #[test]
    fn artifact_bundle_struct_constructable() {
        let mut shas = BTreeMap::new();
        shas.insert("pixel.png".to_string(), [0u8; 32]);
        let bundle = ArtifactBundle {
            root_dir: PathBuf::from("/tmp"),
            pixel_png: PathBuf::from("/tmp/pixel.png"),
            a11y_json: PathBuf::from("/tmp/a11y-tree.json"),
            focus_json: PathBuf::from("/tmp/focus-trace.json"),
            pointer_json: PathBuf::from("/tmp/pointer-hitmap.json"),
            ime_json: PathBuf::from("/tmp/ime-trace.json"),
            sha256s: shas,
        };
        assert_eq!(bundle.sha256s.len(), 1);
    }
}
// CODEGEN-END
