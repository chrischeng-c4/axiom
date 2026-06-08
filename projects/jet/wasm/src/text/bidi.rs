// SPEC-MANAGED: .aw/tech-design/projects/jet/semantic/jet-wasm-src-text.md#schema
// CODEGEN-BEGIN
//! UAX #9 bidi resolution via `unicode-bidi`.
//!
//! @spec .aw/tech-design/projects/jet/wasm-renderer/line-bidi.md#schema

use std::ops::Range;

use unicode_bidi::{BidiInfo, Level};

/// @spec .aw/tech-design/projects/jet/semantic/jet-wasm-src-text.md#schema
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    /// Left-to-right base direction (embedding level 0).
    Ltr,
    /// Right-to-left base direction (embedding level 1).
    Rtl,
    /// First-strong-character detection per UAX #9 P2 — defaults to
    /// Ltr if no strong character is present.
    Auto,
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-wasm-src-text.md#schema
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BidiRun {
    pub byte_range: Range<usize>,
    /// UAX #9 embedding level. Even = LTR, odd = RTL. Max 125.
    pub level: u8,
}

/// Stateless resolver that maps `(text, base_direction)` to a list of
/// `BidiRun`s in logical byte-offset order.
/// @spec .aw/tech-design/projects/jet/semantic/jet-wasm-src-text.md#schema
pub struct BidiResolver;

/// @spec .aw/tech-design/projects/jet/semantic/jet-wasm-src-text.md#schema
impl BidiResolver {
    pub fn resolve(text: &str, base: Direction) -> Vec<BidiRun> {
        if text.is_empty() {
            return Vec::new();
        }
        let level = match base {
            Direction::Ltr => Some(Level::ltr()),
            Direction::Rtl => Some(Level::rtl()),
            Direction::Auto => None,
        };
        let info = BidiInfo::new(text, level);
        let mut out: Vec<BidiRun> = Vec::new();
        for paragraph in &info.paragraphs {
            // Walk codepoints and group by level into contiguous runs.
            let levels = info.reordered_levels(paragraph, paragraph.range.clone());
            // levels is indexed per-byte for the paragraph slice — build
            // contiguous-level runs.
            let p_start = paragraph.range.start;
            let mut cursor = 0_usize;
            while cursor < levels.len() {
                let run_level = levels[cursor];
                let mut end = cursor + 1;
                while end < levels.len() && levels[end] == run_level {
                    end += 1;
                }
                out.push(BidiRun {
                    byte_range: (p_start + cursor)..(p_start + end),
                    level: run_level.number(),
                });
                cursor = end;
            }
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_text_no_runs() {
        let v = BidiResolver::resolve("", Direction::Ltr);
        assert!(v.is_empty());
    }

    #[test]
    fn ltr_only_one_run_level_zero() {
        let runs = BidiResolver::resolve("Hello world", Direction::Ltr);
        assert!(!runs.is_empty());
        assert!(
            runs.iter().all(|r| r.level % 2 == 0),
            "all even levels (LTR)"
        );
        assert_eq!(runs[0].byte_range.start, 0);
    }

    #[test]
    fn rtl_arabic_odd_level() {
        let s = "مرحبا";
        let runs = BidiResolver::resolve(s, Direction::Rtl);
        assert!(!runs.is_empty());
        // Some runs MUST be odd (RTL). The whole string has only Arabic
        // strong chars, so the entire range is at level 1.
        assert!(runs.iter().any(|r| r.level % 2 == 1));
    }

    #[test]
    fn ltr_with_embedded_rtl_produces_two_levels() {
        // "Hello مرحبا" — Latin first, then Arabic.
        let s = "Hello \u{0645}\u{0631}\u{062D}\u{0628}\u{0627}";
        let runs = BidiResolver::resolve(s, Direction::Ltr);
        let has_even = runs.iter().any(|r| r.level % 2 == 0);
        let has_odd = runs.iter().any(|r| r.level % 2 == 1);
        assert!(has_even, "expected at least one LTR run");
        assert!(has_odd, "expected at least one RTL run");
    }

    #[test]
    fn auto_with_strong_rtl_first_yields_rtl_paragraph() {
        // "مرحبا hello" — first strong char is Arabic; with Auto, base
        // direction should resolve to RTL (paragraph level 1).
        let s = "\u{0645}\u{0631}\u{062D}\u{0628}\u{0627} hello";
        let runs = BidiResolver::resolve(s, Direction::Auto);
        // The first run should be the Arabic part at an odd level.
        let first_arabic = runs
            .iter()
            .find(|r| r.byte_range.start == 0)
            .expect("first run starts at 0");
        assert_eq!(
            first_arabic.level % 2,
            1,
            "first strong is Arabic → RTL level"
        );
    }

    #[test]
    fn runs_in_logical_byte_order() {
        let s = "abc def";
        let runs = BidiResolver::resolve(s, Direction::Ltr);
        let mut prev = 0_usize;
        for r in &runs {
            assert!(r.byte_range.start >= prev);
            prev = r.byte_range.end;
        }
    }
}
// CODEGEN-END
