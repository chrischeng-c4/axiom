//! Line-level diff algorithm using Myers' diff.
//!
//! Computes the minimal edit script to transform one text into another.

/// Type of change in a diff.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DiffOp {
    /// Line is the same in both texts.
    Equal(String),
    /// Line was inserted (present in new, absent in old).
    Insert(String),
    /// Line was deleted (present in old, absent in new).
    Delete(String),
    /// Line was replaced (present in both but different).
    Replace { old: String, new: String },
}

/// A hunk in a unified diff.
#[derive(Debug, Clone)]
pub struct DiffHunk {
    /// Starting line in the old file (1-indexed).
    pub old_start: usize,
    /// Number of lines from old file in this hunk.
    pub old_count: usize,
    /// Starting line in the new file (1-indexed).
    pub new_start: usize,
    /// Number of lines from new file in this hunk.
    pub new_count: usize,
    /// The diff operations in this hunk.
    pub ops: Vec<DiffOp>,
}

/// Compute a line-level diff between two strings.
///
/// Uses the longest common subsequence (LCS) approach to find minimal edits.
pub fn diff_lines(old: &str, new: &str) -> Vec<DiffOp> {
    let old_lines: Vec<&str> = old.lines().collect();
    let new_lines: Vec<&str> = new.lines().collect();

    let lcs = compute_lcs(&old_lines, &new_lines);
    build_diff_ops(&old_lines, &new_lines, &lcs)
}

/// Compute the LCS table for two sequences of lines.
fn compute_lcs(old: &[&str], new: &[&str]) -> Vec<Vec<usize>> {
    let m = old.len();
    let n = new.len();
    let mut dp = vec![vec![0usize; n + 1]; m + 1];

    for i in 1..=m {
        for j in 1..=n {
            if old[i - 1] == new[j - 1] {
                dp[i][j] = dp[i - 1][j - 1] + 1;
            } else {
                dp[i][j] = dp[i - 1][j].max(dp[i][j - 1]);
            }
        }
    }

    dp
}

/// Build diff operations from the LCS table.
fn build_diff_ops(old: &[&str], new: &[&str], lcs: &[Vec<usize>]) -> Vec<DiffOp> {
    let mut ops = Vec::new();
    let mut i = old.len();
    let mut j = new.len();

    // Trace back through the LCS table
    let mut trace = Vec::new();

    while i > 0 && j > 0 {
        if old[i - 1] == new[j - 1] {
            trace.push(DiffOp::Equal(old[i - 1].to_string()));
            i -= 1;
            j -= 1;
        } else if lcs[i - 1][j] >= lcs[i][j - 1] {
            trace.push(DiffOp::Delete(old[i - 1].to_string()));
            i -= 1;
        } else {
            trace.push(DiffOp::Insert(new[j - 1].to_string()));
            j -= 1;
        }
    }

    while i > 0 {
        trace.push(DiffOp::Delete(old[i - 1].to_string()));
        i -= 1;
    }

    while j > 0 {
        trace.push(DiffOp::Insert(new[j - 1].to_string()));
        j -= 1;
    }

    trace.reverse();
    ops.extend(trace);
    ops
}

/// Generate unified diff format output.
///
/// # Arguments
/// * `old` - The original text
/// * `new` - The modified text
/// * `old_label` - Label for the old file (e.g., "a/file.txt")
/// * `new_label` - Label for the new file (e.g., "b/file.txt")
/// * `context_lines` - Number of context lines around changes (default: 3)
pub fn unified_diff(
    old: &str,
    new: &str,
    old_label: &str,
    new_label: &str,
    context_lines: usize,
) -> String {
    let ops = diff_lines(old, new);

    // Check if there are any changes
    let has_changes = ops.iter().any(|op| !matches!(op, DiffOp::Equal(_)));
    if !has_changes {
        return String::new();
    }

    let hunks = build_hunks(&ops, context_lines);

    let mut output = String::new();
    output.push_str(&format!("--- {}\n", old_label));
    output.push_str(&format!("+++ {}\n", new_label));

    for hunk in hunks {
        output.push_str(&format!(
            "@@ -{},{} +{},{} @@\n",
            hunk.old_start, hunk.old_count, hunk.new_start, hunk.new_count
        ));

        for op in &hunk.ops {
            match op {
                DiffOp::Equal(line) => {
                    output.push_str(&format!(" {}\n", line));
                }
                DiffOp::Insert(line) => {
                    output.push_str(&format!("+{}\n", line));
                }
                DiffOp::Delete(line) => {
                    output.push_str(&format!("-{}\n", line));
                }
                DiffOp::Replace { old, new } => {
                    output.push_str(&format!("-{}\n", old));
                    output.push_str(&format!("+{}\n", new));
                }
            }
        }
    }

    output
}

/// Build hunks from diff operations with context.
fn build_hunks(ops: &[DiffOp], context: usize) -> Vec<DiffHunk> {
    if ops.is_empty() {
        return Vec::new();
    }

    // Find indices of changed ops
    let change_indices: Vec<usize> = ops
        .iter()
        .enumerate()
        .filter(|(_, op)| !matches!(op, DiffOp::Equal(_)))
        .map(|(i, _)| i)
        .collect();

    if change_indices.is_empty() {
        return Vec::new();
    }

    // Group changes into hunks
    let mut hunks = Vec::new();
    let mut hunk_start = change_indices[0].saturating_sub(context);
    let mut hunk_end = (change_indices[0] + context + 1).min(ops.len());

    for &idx in &change_indices[1..] {
        let new_start = idx.saturating_sub(context);
        let new_end = (idx + context + 1).min(ops.len());

        if new_start <= hunk_end {
            // Merge with current hunk
            hunk_end = new_end;
        } else {
            // Finalize current hunk
            hunks.push(build_single_hunk(ops, hunk_start, hunk_end));
            hunk_start = new_start;
            hunk_end = new_end;
        }
    }

    hunks.push(build_single_hunk(ops, hunk_start, hunk_end));
    hunks
}

fn build_single_hunk(ops: &[DiffOp], start: usize, end: usize) -> DiffHunk {
    let hunk_ops: Vec<DiffOp> = ops[start..end].to_vec();

    // Calculate old/new line numbers
    let mut old_line = 1;
    let mut new_line = 1;

    // Count lines before hunk start
    for op in &ops[..start] {
        match op {
            DiffOp::Equal(_) => {
                old_line += 1;
                new_line += 1;
            }
            DiffOp::Delete(_) => old_line += 1,
            DiffOp::Insert(_) => new_line += 1,
            DiffOp::Replace { .. } => {
                old_line += 1;
                new_line += 1;
            }
        }
    }

    let old_start = old_line;
    let new_start = new_line;

    let mut old_count = 0;
    let mut new_count = 0;

    for op in &hunk_ops {
        match op {
            DiffOp::Equal(_) => {
                old_count += 1;
                new_count += 1;
            }
            DiffOp::Delete(_) => old_count += 1,
            DiffOp::Insert(_) => new_count += 1,
            DiffOp::Replace { .. } => {
                old_count += 1;
                new_count += 1;
            }
        }
    }

    DiffHunk {
        old_start,
        old_count,
        new_start,
        new_count,
        ops: hunk_ops,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_diff_identical() {
        let ops = diff_lines("hello\nworld", "hello\nworld");
        assert!(ops.iter().all(|op| matches!(op, DiffOp::Equal(_))));
    }

    #[test]
    fn test_diff_insert() {
        let ops = diff_lines("a\nc", "a\nb\nc");
        assert!(ops
            .iter()
            .any(|op| matches!(op, DiffOp::Insert(s) if s == "b")));
    }

    #[test]
    fn test_diff_delete() {
        let ops = diff_lines("a\nb\nc", "a\nc");
        assert!(ops
            .iter()
            .any(|op| matches!(op, DiffOp::Delete(s) if s == "b")));
    }

    #[test]
    fn test_unified_diff() {
        let old = "line1\nline2\nline3\nline4\nline5";
        let new = "line1\nline2\nchanged\nline4\nline5";
        let diff = unified_diff(old, new, "a/file.txt", "b/file.txt", 1);
        assert!(diff.contains("--- a/file.txt"));
        assert!(diff.contains("+++ b/file.txt"));
        assert!(diff.contains("@@"));
    }

    #[test]
    fn test_unified_diff_no_changes() {
        let text = "same\ntext";
        let diff = unified_diff(text, text, "a", "b", 3);
        assert!(diff.is_empty());
    }
}
