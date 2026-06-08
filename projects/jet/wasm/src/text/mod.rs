// SPEC-MANAGED: .aw/tech-design/projects/jet/semantic/jet-wasm-src-text.md#schema
// CODEGEN-BEGIN
//! Text shaping engine — rustybuzz integration.
//!
//! @spec .aw/tech-design/projects/jet/wasm-renderer/text-shaping.md
//!
//! Public surface (Phase 6a):
//!   - [`FontFace`] — opaque handle to a parsed font, loadable via
//!     [`FontFace::from_bytes`].
//!   - [`ShapedGlyph`], [`ShapedRun`] — atomic + per-run shaped output.
//!   - [`shape_text`] — pure shaper given `(&FontFace, &str, f32)`.
//!   - [`measure_text`] — convenience wrapper returning
//!     [`taffy::Size<f32>`]; degrades gracefully on font errors so
//!     layout never crashes.
//!   - [`FontError`] — parse / shape error variants.
//!   - [`ShapeCache`], [`ShapeCacheKey`] — externally-owned shape cache.
//!   - [`emit_draw_glyphs`] — paint-runtime integration boundary.
//!
//! Out of scope (Phase 6b–6f follow-ups): line breaking, bidi, text
//! selection geometry, IME overlay, clipboard, font fallback chains.

pub mod bidi;
pub mod cache;
pub mod font_face;
pub mod line_break;
pub mod paint_bridge;
pub mod paragraph;
pub mod script_run;
pub mod shaped;
pub mod shaper;

pub use bidi::{BidiResolver, BidiRun, Direction};
pub use cache::{ShapeCache, ShapeCacheKey};
pub use font_face::{FontError, FontFace};
pub use line_break::{LineBreakKind, LineBreakOpportunity, LineBreaker};
pub use paint_bridge::{emit_draw_glyphs, DrawGlyph, GlyphPaintOp};
pub use paragraph::{shape_paragraph, Paragraph};
pub use script_run::{script_runs, ScriptRun};
pub use shaped::{ShapedGlyph, ShapedRun};
pub use shaper::{measure_text, shape_text};
// CODEGEN-END
