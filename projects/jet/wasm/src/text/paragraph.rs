// SPEC-MANAGED: .aw/tech-design/projects/jet/semantic/jet-wasm-src-text.md#schema
// CODEGEN-BEGIN
//! `Paragraph` aggregate + `shape_paragraph` orchestrator.
//!
//! @spec .aw/tech-design/projects/jet/wasm-renderer/line-bidi.md#schema
//!
//! `shape_paragraph` is the Phase 6b entry point. It composes the
//! Phase 6a shaper with the new bidi + line-break + script-itemization
//! layers and returns a `Paragraph` aggregating shaped runs in
//! LOGICAL byte-offset order plus the line-break / bidi / script
//! metadata that produced them.
//!
//! Pure function: same `(text, font_id, size_px.to_bits(), base)`
//! inputs always produce byte-identical `Paragraph` output.

use std::ops::Range;

use super::bidi::{BidiResolver, BidiRun, Direction};
use super::font_face::{FontError, FontFace};
use super::line_break::{LineBreakOpportunity, LineBreaker};
use super::script_run::{script_runs, ScriptRun};
use super::shaped::ShapedRun;
use super::shaper::shape_text;

/// @spec .aw/tech-design/projects/jet/semantic/jet-wasm-src-text.md#schema
#[derive(Debug, Clone, PartialEq)]
pub struct Paragraph {
    /// Shaped runs in LOGICAL byte-offset order. Visual order is
    /// reconstructed by consumers via `bidi_runs.level`.
    pub runs: Vec<ShapedRun>,
    pub line_breaks: Vec<LineBreakOpportunity>,
    pub bidi_runs: Vec<BidiRun>,
    pub script_runs: Vec<ScriptRun>,
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-wasm-src-text.md#schema
impl Paragraph {
    pub fn empty() -> Self {
        Self {
            runs: Vec::new(),
            line_breaks: Vec::new(),
            bidi_runs: Vec::new(),
            script_runs: Vec::new(),
        }
    }
}

/// Shape `text` into a `Paragraph`. Internally:
///   1. Resolve bidi runs (UAX #9).
///   2. Compute script runs (UAX #24).
///   3. Collect line-break opportunities (UAX #14).
///   4. For each `bidi_run × script_run` intersection in logical
///      order, call `shape_text` on the byte slice and push the
///      `ShapedRun` onto `paragraph.runs`.
///
/// Returns `Err(FontError)` if any segment fails to shape.
/// @spec .aw/tech-design/projects/jet/semantic/jet-wasm-src-text.md#schema
pub fn shape_paragraph(
    text: &str,
    font: &FontFace,
    size_px: f32,
    base: Direction,
) -> Result<Paragraph, FontError> {
    if text.is_empty() {
        return Ok(Paragraph::empty());
    }

    let bidi_runs = BidiResolver::resolve(text, base);
    let script_runs_v = script_runs(text);
    let line_breaks: Vec<LineBreakOpportunity> = LineBreaker::new(text).collect();

    let mut runs: Vec<ShapedRun> = Vec::new();
    for segment in intersections(&bidi_runs, &script_runs_v) {
        if segment.start >= segment.end {
            continue;
        }
        let slice = &text[segment.start..segment.end];
        if slice.is_empty() {
            continue;
        }
        let run = shape_text(font, slice, size_px)?;
        runs.push(run);
    }

    Ok(Paragraph {
        runs,
        line_breaks,
        bidi_runs,
        script_runs: script_runs_v,
    })
}

/// Yield the intersection ranges of two sorted, non-overlapping
/// range lists in ascending logical-byte order.
fn intersections(a: &[BidiRun], b: &[ScriptRun]) -> Vec<Range<usize>> {
    let mut out: Vec<Range<usize>> = Vec::new();
    let mut i = 0;
    let mut j = 0;
    while i < a.len() && j < b.len() {
        let ar = &a[i].byte_range;
        let br = &b[j].byte_range;
        let start = ar.start.max(br.start);
        let end = ar.end.min(br.end);
        if start < end {
            out.push(start..end);
        }
        if ar.end <= br.end {
            i += 1;
        } else {
            j += 1;
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn br(start: usize, end: usize, level: u8) -> BidiRun {
        BidiRun {
            byte_range: start..end,
            level,
        }
    }

    fn sr(start: usize, end: usize, script: &str) -> ScriptRun {
        ScriptRun {
            byte_range: start..end,
            script: script.to_string(),
        }
    }

    #[test]
    fn empty_paragraph_has_empty_fields() {
        let p = Paragraph::empty();
        assert!(p.runs.is_empty());
        assert!(p.line_breaks.is_empty());
        assert!(p.bidi_runs.is_empty());
        assert!(p.script_runs.is_empty());
    }

    #[test]
    fn intersections_full_overlap() {
        let bidi = vec![br(0, 5, 0)];
        let script = vec![sr(0, 5, "Latin")];
        assert_eq!(intersections(&bidi, &script), vec![0..5]);
    }

    #[test]
    fn intersections_split_at_script_boundary() {
        // Single bidi run [0, 10), script split at 5.
        let bidi = vec![br(0, 10, 0)];
        let script = vec![sr(0, 5, "Latin"), sr(5, 10, "Han")];
        assert_eq!(intersections(&bidi, &script), vec![0..5, 5..10]);
    }

    #[test]
    fn intersections_split_at_bidi_boundary() {
        // Single script run [0, 10), bidi split at 5.
        let bidi = vec![br(0, 5, 0), br(5, 10, 1)];
        let script = vec![sr(0, 10, "Latin")];
        assert_eq!(intersections(&bidi, &script), vec![0..5, 5..10]);
    }

    #[test]
    fn intersections_skip_no_overlap() {
        // Disjoint shouldn't happen in practice (both layers cover the
        // full text), but the algorithm must not produce phantom ranges.
        let bidi = vec![br(0, 5, 0)];
        let script = vec![sr(7, 10, "Han")];
        assert!(intersections(&bidi, &script).is_empty());
    }

    #[test]
    fn intersections_in_logical_order() {
        let bidi = vec![br(0, 5, 0), br(5, 10, 1)];
        let script = vec![sr(0, 3, "Latin"), sr(3, 8, "Common"), sr(8, 10, "Han")];
        let ints = intersections(&bidi, &script);
        let mut prev = 0_usize;
        for r in &ints {
            assert!(r.start >= prev);
            prev = r.end;
        }
    }
}
// CODEGEN-END
