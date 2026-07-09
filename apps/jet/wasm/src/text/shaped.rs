// SPEC-MANAGED: .aw/tech-design/projects/jet/semantic/jet-wasm-src-text.md#schema
// CODEGEN-BEGIN
//! Shaped output types — `ShapedGlyph` (atomic) and `ShapedRun` (per-run).
//!
//! @spec .aw/tech-design/projects/jet/wasm-renderer/text-shaping.md#schema

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ShapedGlyph {
    /// Glyph identifier in the font's glyph set (NOT a Unicode codepoint).
    pub glyph_id: u32,
    /// Horizontal offset from pen position to glyph origin, in pixels at size_px.
    pub x_offset: f32,
    /// Vertical offset (positive = down, CSS/canvas coordinate system).
    pub y_offset: f32,
    /// Horizontal advance width for this glyph in pixels at size_px.
    pub x_advance: f32,
    /// Byte offset into the source UTF-8 text at which this glyph's cluster begins.
    pub cluster: u32,
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-wasm-src-text.md#schema
#[derive(Debug, Clone, PartialEq)]
pub struct ShapedRun {
    /// Ordered list of shaped glyphs (visual left-to-right for LTR).
    pub glyphs: Vec<ShapedGlyph>,
    /// Distance from baseline to topmost point of the line box, in pixels at size_px.
    pub ascent: f32,
    /// Distance from baseline to bottommost point of the line box, in pixels at size_px.
    pub descent: f32,
    /// Sum of all glyphs[*].x_advance — pre-computed for measure_text.
    pub advance_width: f32,
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-wasm-src-text.md#schema
impl ShapedRun {
    pub fn empty(ascent: f32, descent: f32) -> Self {
        Self {
            glyphs: Vec::new(),
            ascent,
            descent,
            advance_width: 0.0,
        }
    }

    pub fn line_height(&self) -> f32 {
        self.ascent + self.descent
    }
}
// CODEGEN-END
