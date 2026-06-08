// SPEC-MANAGED: .aw/tech-design/projects/jet/semantic/jet-wasm-tests.md#tests
// CODEGEN-BEGIN
//! Text-shaping S2: Cache hit.
//!
//! @spec .aw/tech-design/projects/jet/wasm-renderer/text-shaping.md#scenarios
//!
//! L0 pure-Rust tier. The cache-key contract (font_id + text +
//! size_bits) is exercisable without a real font using a synthesized
//! ShapedRun; the end-to-end test that re-shapes through a font and
//! verifies cache hit is `#[ignore]`'d pending an embedded font.

use jet_wasm::text::{ShapeCache, ShapeCacheKey, ShapedRun};

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
#[ignore = "S2 end-to-end cache hit — needs an embedded TEST_FONT_BYTES asset"]
fn s2_end_to_end_cache_hit_skips_re_shape() {
    // GIVEN a FontFace + ShapeCache + key for ("Hello", 16.0)
    // WHEN  shape once → insert → lookup → second call short-circuits
    // THEN  cache.get(key).is_some() AND cached.advance_width matches.
}
// CODEGEN-END
