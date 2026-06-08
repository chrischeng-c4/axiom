// SPEC-MANAGED: .aw/tech-design/projects/jet/semantic/jet-e2e.md#schema
// CODEGEN-BEGIN
//! Failure-step assertion diff rendering (#2728).
//!
//! The open-mode failure inspector and the static PM report both
//! need to show readable expected/actual diffs for failed
//! assertions. This module owns the diff projection: it accepts the
//! raw expected/actual strings recorded by the assertion engine and
//! renders a unified-style diff that highlights the line-level
//! changes.
//!
//! No assertion language design is in scope here — this is purely a
//! presentation layer on top of strings the runner already carries
//! through `ResultFailure::diff` / `E2eAssertionDetail::diff`.

use serde::{Deserialize, Serialize};

/// Stable schema tag for [`AssertionDiff`].
pub const ASSERTION_DIFF_SCHEMA_VERSION: &str = "jet.e2e.assertion-diff.v1";

/// One line of a rendered diff. Mirrors `git diff` conventions:
/// context lines start with a space, removals with `-`, additions
/// with `+`. The `kind` field is the structured form so renderers
/// don't have to re-parse the leading character.
/// @spec .aw/tech-design/projects/jet/semantic/jet-e2e.md#schema
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum DiffLine {
    Context { text: String },
    Removed { text: String },
    Added { text: String },
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-e2e.md#schema
impl DiffLine {
    /// Single-char marker for textual rendering (`-`, `+`, ` `).
    pub fn marker(&self) -> char {
        match self {
            Self::Context { .. } => ' ',
            Self::Removed { .. } => '-',
            Self::Added { .. } => '+',
        }
    }

    pub fn text(&self) -> &str {
        match self {
            Self::Context { text } | Self::Removed { text } | Self::Added { text } => text,
        }
    }
}

/// Rendered diff for a single failed assertion. `expected` /
/// `actual` are preserved verbatim so the inspector can offer
/// "copy original" without rebuilding from the line list.
/// @spec .aw/tech-design/projects/jet/semantic/jet-e2e.md#schema
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AssertionDiff {
    pub schema_version: String,
    pub expected: String,
    pub actual: String,
    pub lines: Vec<DiffLine>,
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-e2e.md#schema
impl AssertionDiff {
    /// Build the diff from a single (expected, actual) pair.
    /// Uses a simple LCS-style line diff so equal prefix/suffix
    /// lines render as context.
    pub fn from_pair(expected: &str, actual: &str) -> Self {
        let lines = compute_line_diff(expected, actual);
        Self {
            schema_version: ASSERTION_DIFF_SCHEMA_VERSION.to_string(),
            expected: expected.to_string(),
            actual: actual.to_string(),
            lines,
        }
    }

    /// Render the diff as a plain-text unified block. Each line is
    /// prefixed with its marker and a single space, so the output
    /// is paste-friendly in PR comments and terminal logs.
    pub fn to_text(&self) -> String {
        let mut out = String::new();
        for line in &self.lines {
            out.push(line.marker());
            out.push(' ');
            out.push_str(line.text());
            out.push('\n');
        }
        out
    }

    /// True when expected and actual matched (no removals or
    /// additions). The diff still serialises as context-only.
    pub fn is_unchanged(&self) -> bool {
        self.lines
            .iter()
            .all(|l| matches!(l, DiffLine::Context { .. }))
    }
}

fn compute_line_diff(expected: &str, actual: &str) -> Vec<DiffLine> {
    let exp: Vec<&str> = expected.lines().collect();
    let act: Vec<&str> = actual.lines().collect();
    let lcs = longest_common_subseq(&exp, &act);

    let mut out = Vec::new();
    let (mut i, mut j, mut k) = (0usize, 0usize, 0usize);
    while i < exp.len() || j < act.len() {
        let in_lcs_e = k < lcs.len() && i < exp.len() && exp[i] == lcs[k];
        let in_lcs_a = k < lcs.len() && j < act.len() && act[j] == lcs[k];
        if in_lcs_e && in_lcs_a {
            out.push(DiffLine::Context {
                text: exp[i].to_string(),
            });
            i += 1;
            j += 1;
            k += 1;
        } else if i < exp.len() && !in_lcs_e {
            out.push(DiffLine::Removed {
                text: exp[i].to_string(),
            });
            i += 1;
        } else if j < act.len() && !in_lcs_a {
            out.push(DiffLine::Added {
                text: act[j].to_string(),
            });
            j += 1;
        } else if i < exp.len() {
            out.push(DiffLine::Removed {
                text: exp[i].to_string(),
            });
            i += 1;
        } else if j < act.len() {
            out.push(DiffLine::Added {
                text: act[j].to_string(),
            });
            j += 1;
        }
    }
    out
}

fn longest_common_subseq<'a>(a: &[&'a str], b: &[&'a str]) -> Vec<&'a str> {
    let n = a.len();
    let m = b.len();
    if n == 0 || m == 0 {
        return Vec::new();
    }
    let mut table = vec![vec![0u32; m + 1]; n + 1];
    for i in 0..n {
        for j in 0..m {
            table[i + 1][j + 1] = if a[i] == b[j] {
                table[i][j] + 1
            } else {
                table[i + 1][j].max(table[i][j + 1])
            };
        }
    }
    let mut out = Vec::with_capacity(table[n][m] as usize);
    let (mut i, mut j) = (n, m);
    while i > 0 && j > 0 {
        if a[i - 1] == b[j - 1] {
            out.push(a[i - 1]);
            i -= 1;
            j -= 1;
        } else if table[i - 1][j] >= table[i][j - 1] {
            i -= 1;
        } else {
            j -= 1;
        }
    }
    out.reverse();
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identical_strings_render_as_context_only() {
        let diff = AssertionDiff::from_pair("hello\nworld\n", "hello\nworld\n");
        assert!(diff.is_unchanged());
        let text = diff.to_text();
        assert!(text.contains("  hello"));
        assert!(text.contains("  world"));
        assert!(!text.contains("- "));
        assert!(!text.contains("+ "));
    }

    #[test]
    fn single_changed_line_renders_removed_and_added() {
        // Stop condition (#2728): one failed assertion fixture shows
        // readable diff. Common prefix/suffix stay as context.
        let diff = AssertionDiff::from_pair(
            "order id: 42\nstatus: paid\ntotal: 100\n",
            "order id: 42\nstatus: pending\ntotal: 100\n",
        );
        let text = diff.to_text();
        assert!(text.contains("  order id: 42"), "{text}");
        assert!(text.contains("- status: paid"), "{text}");
        assert!(text.contains("+ status: pending"), "{text}");
        assert!(text.contains("  total: 100"), "{text}");
    }

    #[test]
    fn pure_removal_renders_only_minus_lines() {
        let diff = AssertionDiff::from_pair("a\nb\nc\n", "a\nc\n");
        let removed: Vec<&str> = diff
            .lines
            .iter()
            .filter_map(|l| match l {
                DiffLine::Removed { text } => Some(text.as_str()),
                _ => None,
            })
            .collect();
        assert_eq!(removed, vec!["b"]);
    }

    #[test]
    fn pure_addition_renders_only_plus_lines() {
        let diff = AssertionDiff::from_pair("a\nc\n", "a\nb\nc\n");
        let added: Vec<&str> = diff
            .lines
            .iter()
            .filter_map(|l| match l {
                DiffLine::Added { text } => Some(text.as_str()),
                _ => None,
            })
            .collect();
        assert_eq!(added, vec!["b"]);
    }

    #[test]
    fn empty_expected_renders_all_actual_as_added() {
        let diff = AssertionDiff::from_pair("", "x\ny\n");
        let added: Vec<&str> = diff
            .lines
            .iter()
            .filter_map(|l| match l {
                DiffLine::Added { text } => Some(text.as_str()),
                _ => None,
            })
            .collect();
        assert_eq!(added, vec!["x", "y"]);
    }

    #[test]
    fn diff_round_trips_through_json() {
        let diff = AssertionDiff::from_pair("a\nb\n", "a\nc\n");
        let json = serde_json::to_string(&diff).unwrap();
        let back: AssertionDiff = serde_json::from_str(&json).unwrap();
        assert_eq!(back, diff);
        assert!(json.contains("\"kind\":\"removed\""), "{json}");
        assert!(json.contains("\"kind\":\"added\""), "{json}");
    }

    #[test]
    fn diff_marker_helpers_match_kinds() {
        let c = DiffLine::Context { text: "x".into() };
        let r = DiffLine::Removed { text: "x".into() };
        let a = DiffLine::Added { text: "x".into() };
        assert_eq!(c.marker(), ' ');
        assert_eq!(r.marker(), '-');
        assert_eq!(a.marker(), '+');
        assert_eq!(c.text(), "x");
    }
}
// CODEGEN-END
