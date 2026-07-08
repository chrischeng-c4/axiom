// SPEC-MANAGED: .aw/tech-design/projects/jet/semantic/jet-wasm-src-text.md#schema
// CODEGEN-BEGIN
//! Paint-runtime integration boundary — `emit_draw_glyphs`.
//!
//! @spec .aw/tech-design/projects/jet/wasm-renderer/text-shaping.md#schema
//!
//! Free-function shape per the spec's R7 contract: paint reads a
//! `&ShapedRun` and gets back one `DrawGlyph` op per `ShapedGlyph`.
//! No cross-crate trait inversion — paint already returns
//! `Vec<PaintOp>` from all paint functions.
//!
//! `DrawGlyph` is a typed sub-record consumed by `paint-runtime.md`'s
//! op set when the WebGPU text path grows a first-class `PaintOp::DrawGlyph`
//! variant. For now it stays in this module so the text engine can be
//! integrated without forcing a paint-runtime spec edit; once paint-runtime
//! formalizes the variant a follow-up issue will re-export it from there.

use super::shaped::ShapedRun;

/// One glyph-rasterization op produced from a [`ShapedGlyph`].
/// @spec .aw/tech-design/projects/jet/semantic/jet-wasm-src-text.md#schema
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DrawGlyph {
    pub glyph_id: u32,
    /// Absolute pen-space x in pixels.
    pub x: f32,
    /// Absolute pen-space y in pixels.
    pub y: f32,
}

/// Marker enum: when paint-runtime grows a `PaintOp::DrawGlyph`
/// variant in a follow-up, this alias becomes a re-export.
/// @spec .aw/tech-design/projects/jet/semantic/jet-wasm-src-text.md#schema
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum GlyphPaintOp {
    Draw(DrawGlyph),
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-wasm-src-text.md#schema
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Origin {
    pub x: f32,
    pub y: f32,
}

/// Walk a `ShapedRun` and emit one `DrawGlyph` per `ShapedGlyph`,
/// accumulating pen position from `x_advance`.
/// @spec .aw/tech-design/projects/jet/semantic/jet-wasm-src-text.md#schema
pub fn emit_draw_glyphs(run: &ShapedRun, origin: Origin) -> Vec<DrawGlyph> {
    let mut out = Vec::with_capacity(run.glyphs.len());
    let mut pen_x = origin.x;
    let pen_y = origin.y;
    for g in &run.glyphs {
        out.push(DrawGlyph {
            glyph_id: g.glyph_id,
            x: pen_x + g.x_offset,
            y: pen_y + g.y_offset,
        });
        pen_x += g.x_advance;
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    use super::super::shaped::ShapedGlyph;

    fn glyph(id: u32, x_off: f32, y_off: f32, x_adv: f32, cluster: u32) -> ShapedGlyph {
        ShapedGlyph {
            glyph_id: id,
            x_offset: x_off,
            y_offset: y_off,
            x_advance: x_adv,
            cluster,
        }
    }

    #[test]
    fn empty_run_emits_zero_ops() {
        let run = ShapedRun::empty(12.0, 4.0);
        let ops = emit_draw_glyphs(&run, Origin { x: 0.0, y: 0.0 });
        assert!(ops.is_empty());
    }

    #[test]
    fn pen_advances_per_glyph() {
        let run = ShapedRun {
            glyphs: vec![
                glyph(10, 0.0, 0.0, 8.0, 0),
                glyph(11, 0.0, 0.0, 8.0, 1),
                glyph(12, 0.0, 0.0, 8.0, 2),
            ],
            ascent: 12.0,
            descent: 4.0,
            advance_width: 24.0,
        };
        let ops = emit_draw_glyphs(&run, Origin { x: 100.0, y: 50.0 });
        assert_eq!(ops.len(), 3);
        assert_eq!(ops[0].x, 100.0);
        assert_eq!(ops[1].x, 108.0);
        assert_eq!(ops[2].x, 116.0);
        assert_eq!(ops[0].y, 50.0);
    }

    #[test]
    fn glyph_offset_applied_to_pen_position() {
        let run = ShapedRun {
            glyphs: vec![glyph(20, 2.0, -1.0, 8.0, 0), glyph(21, 0.0, 0.0, 8.0, 1)],
            ascent: 12.0,
            descent: 4.0,
            advance_width: 16.0,
        };
        let ops = emit_draw_glyphs(&run, Origin { x: 0.0, y: 0.0 });
        assert_eq!(ops[0].x, 2.0);
        assert_eq!(ops[0].y, -1.0);
        // Second glyph: pen advanced by first's x_advance (8.0), no
        // offset of its own.
        assert_eq!(ops[1].x, 8.0);
        assert_eq!(ops[1].y, 0.0);
    }

    #[test]
    fn glyph_paint_op_marker() {
        let run = ShapedRun {
            glyphs: vec![glyph(99, 0.0, 0.0, 5.0, 0)],
            ascent: 1.0,
            descent: 0.0,
            advance_width: 5.0,
        };
        let ops: Vec<GlyphPaintOp> = emit_draw_glyphs(&run, Origin { x: 0.0, y: 0.0 })
            .into_iter()
            .map(GlyphPaintOp::Draw)
            .collect();
        assert_eq!(ops.len(), 1);
        let GlyphPaintOp::Draw(d) = ops[0];
        assert_eq!(d.glyph_id, 99);
    }
}
// CODEGEN-END
