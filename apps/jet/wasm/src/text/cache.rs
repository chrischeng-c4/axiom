// SPEC-MANAGED: .aw/tech-design/projects/jet/semantic/jet-wasm-src-text.md#schema
// CODEGEN-BEGIN
//! `ShapeCache` — externally-owned cache for shaped runs.
//!
//! @spec .aw/tech-design/projects/jet/wasm-renderer/text-shaping.md#schema
//!
//! Per the spec's R6 contract: callers own the cache lifetime + choose
//! the eviction policy. This module ships a minimal `HashMap`-backed
//! reference implementation for callers that don't need eviction
//! (e.g. unit tests, single-paragraph apps). For long-lived apps the
//! caller wraps an LRU around `ShapeCacheKey → ShapedRun` themselves.
//!
//! Cache key shape: `(font_id, text, size_bits)` where `size_bits =
//! size_px.to_bits()`. Integer bit-pattern equality is used — NaN
//! bit patterns are treated like any other bit pattern, so callers
//! can avoid the `NaN != NaN` problem by ensuring `size_px` is
//! finite before insertion.

use std::collections::HashMap;

use super::font_face::FontFace;
use super::shaped::ShapedRun;

/// @spec .aw/tech-design/projects/jet/semantic/jet-wasm-src-text.md#schema
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ShapeCacheKey {
    pub font_id: u64,
    pub text: String,
    /// `f32::to_bits()` of the size_px input. Integer equality —
    /// NaN-safe (same NaN bit pattern produces the same key).
    pub size_bits: u32,
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-wasm-src-text.md#schema
impl ShapeCacheKey {
    pub fn new(font: &FontFace, text: impl Into<String>, size_px: f32) -> Self {
        Self {
            font_id: font.font_id,
            text: text.into(),
            size_bits: size_px.to_bits(),
        }
    }
}

/// Reference cache implementation — a plain `HashMap`. Use it
/// directly for tests and small apps; wrap your own `LruCache`
/// around `ShapeCacheKey → ShapedRun` for long-running renderers.
/// @spec .aw/tech-design/projects/jet/semantic/jet-wasm-src-text.md#schema
#[derive(Debug, Default, Clone)]
pub struct ShapeCache {
    inner: HashMap<ShapeCacheKey, ShapedRun>,
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-wasm-src-text.md#schema
impl ShapeCache {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get(&self, key: &ShapeCacheKey) -> Option<&ShapedRun> {
        self.inner.get(key)
    }

    pub fn insert(&mut self, key: ShapeCacheKey, run: ShapedRun) {
        self.inner.insert(key, run);
    }

    pub fn len(&self) -> usize {
        self.inner.len()
    }

    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    pub fn clear(&mut self) {
        self.inner.clear()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn synth_run(advance: f32) -> ShapedRun {
        ShapedRun {
            glyphs: Vec::new(),
            ascent: 12.0,
            descent: 4.0,
            advance_width: advance,
        }
    }

    #[test]
    fn keys_with_identical_bits_collide() {
        let k1 = ShapeCacheKey {
            font_id: 42,
            text: "Hi".to_string(),
            size_bits: 16.0_f32.to_bits(),
        };
        let k2 = ShapeCacheKey {
            font_id: 42,
            text: "Hi".to_string(),
            size_bits: 16.0_f32.to_bits(),
        };
        assert_eq!(k1, k2);
    }

    #[test]
    fn nan_bit_pattern_is_equal_to_itself() {
        let nan_bits = f32::NAN.to_bits();
        let k1 = ShapeCacheKey {
            font_id: 42,
            text: "x".to_string(),
            size_bits: nan_bits,
        };
        let k2 = ShapeCacheKey {
            font_id: 42,
            text: "x".to_string(),
            size_bits: nan_bits,
        };
        // f32::NAN != f32::NAN, but the bit patterns are equal — that's
        // why the key uses integer equality on bits.
        assert_eq!(k1, k2);
    }

    #[test]
    fn negative_zero_bit_pattern_distinct_from_positive_zero() {
        let pz = ShapeCacheKey {
            font_id: 1,
            text: "z".to_string(),
            size_bits: 0.0_f32.to_bits(),
        };
        let nz = ShapeCacheKey {
            font_id: 1,
            text: "z".to_string(),
            size_bits: (-0.0_f32).to_bits(),
        };
        // Bit pattern of -0.0 differs from +0.0 — keys are distinct,
        // even though `0.0 == -0.0` as f32.
        assert_ne!(pz, nz);
    }

    #[test]
    fn cache_get_and_insert() {
        let mut cache = ShapeCache::new();
        let key = ShapeCacheKey {
            font_id: 7,
            text: "ab".to_string(),
            size_bits: 14.0_f32.to_bits(),
        };
        assert!(cache.get(&key).is_none());
        cache.insert(key.clone(), synth_run(20.0));
        assert_eq!(cache.get(&key).unwrap().advance_width, 20.0);
        assert_eq!(cache.len(), 1);
    }
}
// CODEGEN-END
