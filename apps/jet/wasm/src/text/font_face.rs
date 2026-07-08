// SPEC-MANAGED: .aw/tech-design/projects/jet/semantic/jet-wasm-src-text.md#schema
// CODEGEN-BEGIN
//! `FontFace` — parsed-font handle wrapping rustybuzz's `Face`.
//!
//! @spec .aw/tech-design/projects/jet/wasm-renderer/text-shaping.md#schema
//!
//! Owns the font bytes via `Arc<[u8]>` for cheap clones and a stable
//! `font_id` derived from `xxhash_rust::xxh3::xxh3_64` over those bytes
//! (R6 cache-stability invariant). Lifetimes are static-ish: the bytes
//! live as long as any clone of `FontFace` lives.

use std::sync::Arc;

use rustybuzz::Face;
use xxhash_rust::xxh3::xxh3_64;

/// @spec .aw/tech-design/projects/jet/semantic/jet-wasm-src-text.md#schema
#[derive(Debug, Clone, PartialEq)]
pub enum FontError {
    /// The byte buffer is not a valid OpenType / TrueType font.
    InvalidData(String),
    /// The font is syntactically valid but uses a format rustybuzz cannot shape.
    UnsupportedFormat(String),
    /// No glyph found for the requested codepoint in the font's cmap.
    GlyphMissing(u32),
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-wasm-src-text.md#schema
impl std::fmt::Display for FontError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidData(msg) => write!(f, "InvalidData: {msg}"),
            Self::UnsupportedFormat(msg) => write!(f, "UnsupportedFormat: {msg}"),
            Self::GlyphMissing(cp) => write!(f, "GlyphMissing: U+{cp:04X}"),
        }
    }
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-wasm-src-text.md#schema
impl std::error::Error for FontError {}

/// Opaque handle to a parsed font.
///
/// `Send + Sync` because the underlying `Arc<[u8]>` is, and rustybuzz
/// `Face` is constructed on demand in [`FontFace::face`] without
/// shared mutable state.
/// @spec .aw/tech-design/projects/jet/semantic/jet-wasm-src-text.md#schema
#[derive(Debug, Clone)]
pub struct FontFace {
    bytes: Arc<[u8]>,
    pub font_id: u64,
    /// Index into a TrueType collection (TTC). 0 for plain TTF/OTF.
    pub face_index: u32,
    units_per_em: u16,
    /// Font-design-units ascent (pre-scaling). Scaled to size_px at shape time.
    ascent_units: i16,
    /// Font-design-units descent (positive value; pre-scaling).
    descent_units: i16,
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-wasm-src-text.md#schema
impl FontFace {
    /// Parse a TrueType / OpenType font from raw bytes.
    ///
    /// `font_id` is derived from `xxh3_64(bytes)` — pinned algorithm
    /// per the spec's cache-stability rationale.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, FontError> {
        Self::from_bytes_with_index(bytes, 0)
    }

    pub fn from_bytes_with_index(bytes: &[u8], face_index: u32) -> Result<Self, FontError> {
        let owned: Arc<[u8]> = Arc::from(bytes);
        let face = Face::from_slice(&owned, face_index).ok_or_else(|| {
            FontError::InvalidData(format!(
                "rustybuzz::Face::from_slice rejected the bytes (face_index={face_index}, len={})",
                owned.len()
            ))
        })?;
        let units_per_em: u16 = u16::try_from(face.units_per_em()).map_err(|_| {
            FontError::UnsupportedFormat(format!(
                "units_per_em out of u16 range: {}",
                face.units_per_em()
            ))
        })?;
        let ascent_units = face.ascender();
        let descent_units = face.descender().abs();
        let font_id = xxh3_64(bytes);
        Ok(Self {
            bytes: owned,
            font_id,
            face_index,
            units_per_em,
            ascent_units,
            descent_units,
        })
    }

    /// Build a fresh rustybuzz `Face` for shaping. Cheap — `Face` is
    /// thin metadata over the underlying byte slice.
    pub fn face(&self) -> Face<'_> {
        // SAFETY of unwrap: we already constructed a Face from the same
        // bytes successfully in `from_bytes`; bytes are never mutated
        // (Arc<[u8]>), so re-parsing is guaranteed to succeed.
        Face::from_slice(&self.bytes, self.face_index)
            .expect("bytes were parsed successfully on construction")
    }

    /// Scale the font's ascent metric to pixels at `size_px`.
    pub fn ascent_at(&self, size_px: f32) -> f32 {
        self.scale(self.ascent_units, size_px)
    }

    /// Scale the font's descent metric to pixels at `size_px`.
    pub fn descent_at(&self, size_px: f32) -> f32 {
        self.scale(self.descent_units, size_px)
    }

    fn scale(&self, units: i16, size_px: f32) -> f32 {
        if self.units_per_em == 0 {
            return 0.0;
        }
        (units as f32) * size_px / (self.units_per_em as f32)
    }

    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-wasm-src-text.md#schema
impl PartialEq for FontFace {
    fn eq(&self, other: &Self) -> bool {
        self.font_id == other.font_id && self.face_index == other.face_index
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn invalid_bytes_rejected() {
        let err = FontFace::from_bytes(b"not a font").unwrap_err();
        assert!(matches!(err, FontError::InvalidData(_)));
    }

    #[test]
    fn font_error_display_glyph_missing() {
        let e = FontError::GlyphMissing(0x1F600);
        assert_eq!(e.to_string(), "GlyphMissing: U+1F600");
    }

    #[test]
    fn font_error_display_invalid_data() {
        let e = FontError::InvalidData("bad header".to_string());
        assert_eq!(e.to_string(), "InvalidData: bad header");
    }
}
// CODEGEN-END
