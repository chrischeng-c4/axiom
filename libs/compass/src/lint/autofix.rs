//! Auto-fix: apply quick fixes from diagnostics
//!
//! Provides utilities to apply single or batch fixes to source code.

use crate::diagnostic::{Diagnostic, QuickFix, Range, TextEdit};

/// Result of applying a fix
#[derive(Debug, Clone)]
pub struct FixResult {
    /// The modified source code
    pub source: String,
    /// Number of edits applied
    pub edits_applied: usize,
    /// Description of the fix
    pub description: String,
}

/// Apply a single quick fix to source code.
///
/// Edits within the fix are sorted by position (reverse order) and applied
/// from bottom to top to avoid offset invalidation.
pub fn apply_fix(source: &str, fix: &QuickFix) -> FixResult {
    let mut result = source.to_string();
    let mut edits: Vec<&TextEdit> = fix.edits.iter().collect();
    // Sort reverse (bottom-to-top) so earlier offsets stay valid
    edits.sort_by(|a, b| {
        b.range
            .start
            .line
            .cmp(&a.range.start.line)
            .then(b.range.start.character.cmp(&a.range.start.character))
    });

    let mut applied = 0;
    for edit in &edits {
        let start = position_to_offset(&result, &edit.range.start);
        let end = position_to_offset(&result, &edit.range.end);
        if start <= end && end <= result.len() {
            result.replace_range(start..end, &edit.new_text);
            applied += 1;
        }
    }

    FixResult {
        source: result,
        edits_applied: applied,
        description: fix.title.clone(),
    }
}

/// Apply all fixable diagnostics to source code.
///
/// Only diagnostics with at least one quick fix are considered.
/// Uses the first (preferred) fix from each diagnostic.
/// Non-overlapping edits are applied; overlapping ones are skipped.
pub fn apply_all_fixes(source: &str, diagnostics: &[Diagnostic]) -> FixResult {
    // Collect all edits from the first fix of each diagnostic
    let mut all_edits: Vec<&TextEdit> = Vec::new();
    for diag in diagnostics {
        if let Some(fix) = diag.quick_fixes.first() {
            all_edits.extend(fix.edits.iter());
        }
    }

    // Sort by position (top-to-bottom) for overlap detection
    all_edits.sort_by(|a, b| {
        a.range
            .start
            .line
            .cmp(&b.range.start.line)
            .then(a.range.start.character.cmp(&b.range.start.character))
    });

    // Remove overlapping edits (keep earlier ones)
    let mut non_overlapping: Vec<&TextEdit> = Vec::new();
    for edit in &all_edits {
        let overlaps = non_overlapping
            .iter()
            .any(|prev| ranges_overlap(&prev.range, &edit.range));
        if !overlaps {
            non_overlapping.push(edit);
        }
    }

    // Reverse for bottom-to-top application
    non_overlapping.reverse();

    let mut result = source.to_string();
    let mut applied = 0;
    for edit in &non_overlapping {
        let start = position_to_offset(&result, &edit.range.start);
        let end = position_to_offset(&result, &edit.range.end);
        if start <= end && end <= result.len() {
            result.replace_range(start..end, &edit.new_text);
            applied += 1;
        }
    }

    let desc = format!("Applied {} fix(es)", applied);
    FixResult {
        source: result,
        edits_applied: applied,
        description: desc,
    }
}

/// Check if two ranges overlap
fn ranges_overlap(a: &Range, b: &Range) -> bool {
    // a ends before b starts → no overlap
    let a_before_b = (a.end.line < b.start.line)
        || (a.end.line == b.start.line && a.end.character <= b.start.character);
    // b ends before a starts → no overlap
    let b_before_a = (b.end.line < a.start.line)
        || (b.end.line == a.start.line && b.end.character <= a.start.character);
    !(a_before_b || b_before_a)
}

/// Convert line/character position to byte offset in source
fn position_to_offset(source: &str, pos: &crate::diagnostic::Position) -> usize {
    let mut offset = 0;
    for (i, line) in source.lines().enumerate() {
        if i == pos.line as usize {
            return offset + (pos.character as usize).min(line.len());
        }
        offset += line.len() + 1; // +1 for '\n'
    }
    // Position past last line → end of source
    source.len()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::diagnostic::{
        Diagnostic, DiagnosticCategory, DiagnosticSeverity, Position, QuickFix, Range, TextEdit,
    };

    fn make_edit(sl: u32, sc: u32, el: u32, ec: u32, text: &str) -> TextEdit {
        TextEdit {
            range: Range::new(Position::new(sl, sc), Position::new(el, ec)),
            new_text: text.to_string(),
        }
    }

    fn make_fix(title: &str, edits: Vec<TextEdit>) -> QuickFix {
        QuickFix {
            title: title.to_string(),
            edits,
        }
    }

    fn make_diag(fix: QuickFix) -> Diagnostic {
        let mut d = Diagnostic::new(
            Range::new(Position::new(0, 0), Position::new(0, 1)),
            DiagnosticSeverity::Warning,
            "test",
            DiagnosticCategory::Style,
            "test diagnostic",
        );
        d.quick_fixes.push(fix);
        d
    }

    // 1. Simple text replacement
    #[test]
    fn test_apply_fix_simple_replacement() {
        let source = "let foo = 1;\nlet bar = 2;\n";
        let fix = make_fix("Rename foo to baz", vec![make_edit(0, 4, 0, 7, "baz")]);
        let result = apply_fix(source, &fix);
        assert_eq!(result.source, "let baz = 1;\nlet bar = 2;\n");
        assert_eq!(result.edits_applied, 1);
        assert_eq!(result.description, "Rename foo to baz");
    }

    // 2. Line deletion (new_text is empty)
    #[test]
    fn test_apply_fix_line_deletion() {
        let source = "line1\nline2\nline3\n";
        // Delete "line2\n" (line 1, char 0 to line 2, char 0)
        let fix = make_fix("Delete line2", vec![make_edit(1, 0, 2, 0, "")]);
        let result = apply_fix(source, &fix);
        assert_eq!(result.source, "line1\nline3\n");
        assert_eq!(result.edits_applied, 1);
    }

    // 3. Multiple non-overlapping edits
    #[test]
    fn test_apply_all_fixes_non_overlapping() {
        let source = "aaa\nbbb\nccc\n";
        let d1 = make_diag(make_fix("fix1", vec![make_edit(0, 0, 0, 3, "AAA")]));
        let d2 = make_diag(make_fix("fix2", vec![make_edit(2, 0, 2, 3, "CCC")]));
        let result = apply_all_fixes(source, &[d1, d2]);
        assert_eq!(result.source, "AAA\nbbb\nCCC\n");
        assert_eq!(result.edits_applied, 2);
    }

    // 4. Overlapping edits are skipped
    #[test]
    fn test_apply_all_fixes_skips_overlapping() {
        let source = "hello world\n";
        // First diag fixes chars 0..5 ("hello")
        let d1 = make_diag(make_fix("fix1", vec![make_edit(0, 0, 0, 5, "HELLO")]));
        // Second diag overlaps chars 3..8 ("lo wo")
        let d2 = make_diag(make_fix("fix2", vec![make_edit(0, 3, 0, 8, "LO WO")]));
        let result = apply_all_fixes(source, &[d1, d2]);
        // Only d1 should be applied (d2 overlaps)
        assert_eq!(result.source, "HELLO world\n");
        assert_eq!(result.edits_applied, 1);
    }

    // 5. position_to_offset edge cases
    #[test]
    fn test_position_to_offset_edges() {
        let source = "ab\ncd\nef";
        // Start of file
        assert_eq!(position_to_offset(source, &Position::new(0, 0)), 0);
        // End of first line
        assert_eq!(position_to_offset(source, &Position::new(0, 2)), 2);
        // Start of second line
        assert_eq!(position_to_offset(source, &Position::new(1, 0)), 3);
        // Middle of third line
        assert_eq!(position_to_offset(source, &Position::new(2, 1)), 7);
        // Character past end of line is clamped
        assert_eq!(position_to_offset(source, &Position::new(0, 99)), 2);
        // Line past end of source returns source.len()
        assert_eq!(position_to_offset(source, &Position::new(10, 0)), 8);
    }
}
