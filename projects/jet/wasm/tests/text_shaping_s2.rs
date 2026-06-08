// SPEC-MANAGED: .aw/tech-design/projects/jet/semantic/jet-wasm-tests.md#tests
// CODEGEN-BEGIN
//! Text-shaping S2: Cache hit.
//!
//! @spec .aw/tech-design/projects/jet/wasm-renderer/text-shaping.md#scenarios
//!
//! L0 pure-Rust tier. The cache-key contract is exercised both with a
//! synthesized ShapedRun and with the vendored Tuffy face used by
//! the WebGPU glyph atlas bridge.

use jet_wasm::text::{shape_text, ShapeCache, ShapeCacheKey, ShapedRun};

mod common;

fn synth_run(advance: f32) -> ShapedRun {
    ShapedRun {
        glyphs: Vec::new(),
        ascent: 12.0,
        descent: 4.0,
        advance_width: advance,
    }
}

#[test]
fn s2_cache_lookup_returns_inserted_run() {
    let mut cache = ShapeCache::new();
    let key = ShapeCacheKey {
        font_id: 0xDEAD_BEEF,
        text: "Hi".to_string(),
        size_bits: 16.0_f32.to_bits(),
    };
    assert!(cache.get(&key).is_none());
    cache.insert(key.clone(), synth_run(20.0));
    let hit = cache.get(&key).expect("cache hit after insert");
    assert_eq!(hit.advance_width, 20.0);
}

#[test]
fn s2_distinct_size_distinct_key() {
    let k14 = ShapeCacheKey {
        font_id: 1,
        text: "x".to_string(),
        size_bits: 14.0_f32.to_bits(),
    };
    let k16 = ShapeCacheKey {
        font_id: 1,
        text: "x".to_string(),
        size_bits: 16.0_f32.to_bits(),
    };
    assert_ne!(k14, k16);
}

#[test]
fn s2_end_to_end_cache_hit_skips_re_shape() {
    let font = common::tuffy_regular();
    let key = ShapeCacheKey::new(&font, "Hello", 16.0);
    let shaped = shape_text(&font, "Hello", 16.0).expect("Hello shapes through Tuffy");
    assert_eq!(shaped.glyphs.len(), 5);

    let mut cache = ShapeCache::new();
    assert!(cache.get(&key).is_none());
    cache.insert(key.clone(), shaped.clone());

    let hit = cache
        .get(&key)
        .expect("shape cache should return the inserted real-font run");
    assert_eq!(hit.glyphs, shaped.glyphs);
    assert_eq!(hit.advance_width, shaped.advance_width);
}
// CODEGEN-END
