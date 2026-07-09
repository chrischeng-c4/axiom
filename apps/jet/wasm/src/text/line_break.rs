// SPEC-MANAGED: .aw/tech-design/projects/jet/semantic/jet-wasm-src-text.md#schema
// CODEGEN-BEGIN
//! UAX #14 line-break iteration via `icu_segmenter::LineSegmenter`.
//!
//! @spec .aw/tech-design/projects/jet/wasm-renderer/line-bidi.md#schema

use icu_segmenter::LineSegmenter;

/// @spec .aw/tech-design/projects/jet/semantic/jet-wasm-src-text.md#schema
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LineBreakKind {
    /// Required break (e.g. U+000A LINE FEED). The line MUST end here.
    Mandatory,
    /// Permitted break (e.g. after a space or hyphen). MAY end here.
    Allowed,
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-wasm-src-text.md#schema
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LineBreakOpportunity {
    /// UTF-8 byte index — points to the FIRST byte of the character
    /// AFTER the break opportunity.
    pub byte_offset: usize,
    pub kind: LineBreakKind,
}

/// UAX #14 line-break iterator backed by `icu_segmenter::LineSegmenter`
/// in auto mode (compiled-in data tables, no runtime I/O).
/// @spec .aw/tech-design/projects/jet/semantic/jet-wasm-src-text.md#schema
pub struct LineBreaker<'a> {
    text: &'a str,
    breakpoints: Vec<usize>,
    cursor: usize,
}

impl<'a> LineBreaker<'a> {
    pub fn new(text: &'a str) -> Self {
        let segmenter = LineSegmenter::new_auto();
        let mut breakpoints: Vec<usize> = segmenter.segment_str(text).collect();
        // icu_segmenter emits 0 as the start-of-text marker; drop it
        // per the spec contract ("0 is never emitted").
        if breakpoints.first() == Some(&0) {
            breakpoints.remove(0);
        }
        Self {
            text,
            breakpoints,
            cursor: 0,
        }
    }

    fn classify(&self, byte_offset: usize) -> LineBreakKind {
        // Look at the character ENDING at byte_offset to decide
        // mandatory vs allowed. Hard breaks: U+000A LF, U+000D CR,
        // U+0085 NEL, U+2028 LINE SEP, U+2029 PARA SEP, U+000C FF.
        // Anything else (or end-of-text) is Allowed by default; we
        // only escalate to Mandatory when we see one of these
        // characters immediately preceding the break point.
        if byte_offset == 0 {
            return LineBreakKind::Allowed;
        }
        let prefix = &self.text[..byte_offset];
        match prefix.chars().next_back() {
            Some('\n' | '\r' | '\u{000C}' | '\u{0085}' | '\u{2028}' | '\u{2029}') => {
                LineBreakKind::Mandatory
            }
            None => LineBreakKind::Allowed,
            _ => {
                // End-of-text (no character after this break) is
                // implicitly mandatory per UAX #14 LB3.
                if byte_offset == self.text.len() {
                    LineBreakKind::Mandatory
                } else {
                    LineBreakKind::Allowed
                }
            }
        }
    }
}

impl<'a> Iterator for LineBreaker<'a> {
    type Item = LineBreakOpportunity;

    fn next(&mut self) -> Option<Self::Item> {
        let byte_offset = *self.breakpoints.get(self.cursor)?;
        self.cursor += 1;
        let kind = self.classify(byte_offset);
        Some(LineBreakOpportunity { byte_offset, kind })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn opportunities(text: &str) -> Vec<LineBreakOpportunity> {
        LineBreaker::new(text).collect()
    }

    #[test]
    fn empty_text_no_opportunities() {
        let v = opportunities("");
        assert!(v.is_empty());
    }

    #[test]
    fn ascending_byte_offsets() {
        let v = opportunities("Hello world, hello world.");
        let mut prev = 0_usize;
        for op in &v {
            assert!(op.byte_offset > prev, "expected strict ascending offsets");
            prev = op.byte_offset;
        }
    }

    #[test]
    fn never_yields_zero() {
        for op in opportunities("abc def") {
            assert!(op.byte_offset > 0);
        }
    }

    #[test]
    fn final_offset_at_or_below_len() {
        let s = "Hello, world!";
        for op in opportunities(s) {
            assert!(op.byte_offset <= s.len());
        }
    }

    #[test]
    fn mandatory_at_newline() {
        // After a "\n" character, the break is mandatory.
        let s = "abc\ndef";
        let v = opportunities(s);
        let after_newline = v
            .iter()
            .find(|op| op.byte_offset == 4) // byte after "\n"
            .expect("expected break after newline");
        assert_eq!(after_newline.kind, LineBreakKind::Mandatory);
    }

    #[test]
    fn end_of_text_mandatory() {
        let s = "Hello";
        let v = opportunities(s);
        let last = v.last().expect("at least one opportunity");
        assert_eq!(last.byte_offset, s.len());
        assert_eq!(last.kind, LineBreakKind::Mandatory);
    }
}
// CODEGEN-END
