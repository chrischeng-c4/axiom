//! Patch parsing and application.
//!
//! Parses unified diff format and applies patches to text.

/// A parsed patch containing one or more file diffs.
#[derive(Debug, Clone)]
pub struct Patch {
    /// The individual file patches.
    pub files: Vec<FilePatch>,
}

/// A patch for a single file.
#[derive(Debug, Clone)]
pub struct FilePatch {
    /// Original file path.
    pub old_file: String,
    /// New file path.
    pub new_file: String,
    /// The hunks in this patch.
    pub hunks: Vec<PatchHunk>,
}

/// A single hunk in a patch.
#[derive(Debug, Clone)]
pub struct PatchHunk {
    /// Starting line in old file (1-indexed).
    pub old_start: usize,
    /// Number of lines from old file.
    pub old_count: usize,
    /// Starting line in new file (1-indexed).
    pub new_start: usize,
    /// Number of lines from new file.
    pub new_count: usize,
    /// The lines in this hunk.
    pub lines: Vec<PatchLine>,
}

/// A line in a patch hunk.
#[derive(Debug, Clone, PartialEq)]
pub enum PatchLine {
    /// Context line (unchanged).
    Context(String),
    /// Added line.
    Add(String),
    /// Removed line.
    Remove(String),
}

/// Parse a unified diff string into a Patch.
pub fn parse_patch(patch_text: &str) -> Result<Patch, String> {
    let mut files = Vec::new();
    let lines: Vec<&str> = patch_text.lines().collect();
    let mut i = 0;

    while i < lines.len() {
        // Look for --- / +++ header
        if i + 1 < lines.len() && lines[i].starts_with("--- ") && lines[i + 1].starts_with("+++ ") {
            let old_file = lines[i].strip_prefix("--- ").unwrap().to_string();
            let new_file = lines[i + 1].strip_prefix("+++ ").unwrap().to_string();
            i += 2;

            let mut hunks = Vec::new();

            // Parse hunks
            while i < lines.len() && lines[i].starts_with("@@ ") {
                let hunk = parse_hunk_header(lines[i])?;
                i += 1;

                let mut hunk_lines = Vec::new();
                while i < lines.len()
                    && !lines[i].starts_with("@@ ")
                    && !lines[i].starts_with("--- ")
                {
                    let line = lines[i];
                    if let Some(content) = line.strip_prefix('+') {
                        hunk_lines.push(PatchLine::Add(content.to_string()));
                    } else if let Some(content) = line.strip_prefix('-') {
                        hunk_lines.push(PatchLine::Remove(content.to_string()));
                    } else if let Some(content) = line.strip_prefix(' ') {
                        hunk_lines.push(PatchLine::Context(content.to_string()));
                    } else if line.is_empty() {
                        // Empty context line
                        hunk_lines.push(PatchLine::Context(String::new()));
                    } else {
                        // Assume context line without leading space
                        hunk_lines.push(PatchLine::Context(line.to_string()));
                    }
                    i += 1;
                }

                hunks.push(PatchHunk {
                    old_start: hunk.0,
                    old_count: hunk.1,
                    new_start: hunk.2,
                    new_count: hunk.3,
                    lines: hunk_lines,
                });
            }

            files.push(FilePatch {
                old_file,
                new_file,
                hunks,
            });
        } else {
            i += 1;
        }
    }

    Ok(Patch { files })
}

/// Parse a hunk header line: @@ -old_start,old_count +new_start,new_count @@
fn parse_hunk_header(line: &str) -> Result<(usize, usize, usize, usize), String> {
    let line = line.strip_prefix("@@ ").ok_or("Invalid hunk header")?;
    let end = line
        .find(" @@")
        .ok_or("Invalid hunk header: missing closing @@")?;
    let range_part = &line[..end];

    let parts: Vec<&str> = range_part.split_whitespace().collect();
    if parts.len() != 2 {
        return Err(format!("Invalid hunk header ranges: {}", range_part));
    }

    let old_range = parts[0].strip_prefix('-').ok_or("Invalid old range")?;
    let new_range = parts[1].strip_prefix('+').ok_or("Invalid new range")?;

    let (old_start, old_count) = parse_range(old_range)?;
    let (new_start, new_count) = parse_range(new_range)?;

    Ok((old_start, old_count, new_start, new_count))
}

fn parse_range(range: &str) -> Result<(usize, usize), String> {
    if let Some((start, count)) = range.split_once(',') {
        let s: usize = start
            .parse()
            .map_err(|e| format!("Invalid range start: {}", e))?;
        let c: usize = count
            .parse()
            .map_err(|e| format!("Invalid range count: {}", e))?;
        Ok((s, c))
    } else {
        let s: usize = range.parse().map_err(|e| format!("Invalid range: {}", e))?;
        Ok((s, 1))
    }
}

/// Apply a file patch to the given text.
///
/// Returns the patched text or an error if the patch cannot be applied.
pub fn apply_patch(text: &str, file_patch: &FilePatch) -> Result<String, String> {
    let mut lines: Vec<String> = text.lines().map(String::from).collect();

    // Apply hunks in reverse order to avoid index shifting
    let mut hunks = file_patch.hunks.clone();
    hunks.sort_by(|a, b| b.old_start.cmp(&a.old_start));

    for hunk in &hunks {
        let start_idx = hunk.old_start.saturating_sub(1);

        // Verify context lines match
        let mut old_idx = start_idx;
        for patch_line in &hunk.lines {
            match patch_line {
                PatchLine::Context(content) => {
                    if old_idx >= lines.len() || lines[old_idx] != *content {
                        // Try fuzzy matching - allow whitespace differences
                        if old_idx >= lines.len() || lines[old_idx].trim() != content.trim() {
                            return Err(format!(
                                "Context mismatch at line {}: expected '{}', got '{}'",
                                old_idx + 1,
                                content,
                                lines.get(old_idx).unwrap_or(&String::new())
                            ));
                        }
                    }
                    old_idx += 1;
                }
                PatchLine::Remove(_) => {
                    old_idx += 1;
                }
                PatchLine::Add(_) => {}
            }
        }

        // Apply the hunk
        let mut new_lines = Vec::new();
        for patch_line in &hunk.lines {
            match patch_line {
                PatchLine::Context(content) => {
                    new_lines.push(content.clone());
                }
                PatchLine::Add(content) => {
                    new_lines.push(content.clone());
                }
                PatchLine::Remove(_) => {
                    // Skip removed lines
                }
            }
        }

        // Calculate how many old lines this hunk covers
        let old_line_count = hunk
            .lines
            .iter()
            .filter(|l| matches!(l, PatchLine::Context(_) | PatchLine::Remove(_)))
            .count();

        // Replace the old lines with the new lines
        let end_idx = (start_idx + old_line_count).min(lines.len());
        lines.splice(start_idx..end_idx, new_lines);
    }

    Ok(lines.join("\n"))
}

/// Apply a full patch (potentially multiple files) to a single text.
/// Uses the first file patch found.
pub fn apply_patch_text(text: &str, patch_text: &str) -> Result<String, String> {
    let patch = parse_patch(patch_text)?;
    if patch.files.is_empty() {
        return Err("No file patches found".to_string());
    }
    apply_patch(text, &patch.files[0])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_patch() {
        let patch_text = "\
--- a/file.txt
+++ b/file.txt
@@ -1,3 +1,3 @@
 line1
-line2
+changed
 line3
";
        let patch = parse_patch(patch_text).unwrap();
        assert_eq!(patch.files.len(), 1);
        assert_eq!(patch.files[0].hunks.len(), 1);
        assert_eq!(patch.files[0].hunks[0].old_start, 1);
        assert_eq!(patch.files[0].hunks[0].old_count, 3);
    }

    #[test]
    fn test_apply_patch() {
        let original = "line1\nline2\nline3";
        let patch_text = "\
--- a/file.txt
+++ b/file.txt
@@ -1,3 +1,3 @@
 line1
-line2
+changed
 line3
";
        let result = apply_patch_text(original, patch_text).unwrap();
        assert_eq!(result, "line1\nchanged\nline3");
    }

    #[test]
    fn test_apply_patch_insert() {
        let original = "line1\nline3";
        let patch_text = "\
--- a/file.txt
+++ b/file.txt
@@ -1,2 +1,3 @@
 line1
+line2
 line3
";
        let result = apply_patch_text(original, patch_text).unwrap();
        assert_eq!(result, "line1\nline2\nline3");
    }

    #[test]
    fn test_apply_patch_delete() {
        let original = "line1\nline2\nline3";
        let patch_text = "\
--- a/file.txt
+++ b/file.txt
@@ -1,3 +1,2 @@
 line1
-line2
 line3
";
        let result = apply_patch_text(original, patch_text).unwrap();
        assert_eq!(result, "line1\nline3");
    }

    #[test]
    fn test_roundtrip_diff_patch() {
        let old = "line1\nline2\nline3\nline4\nline5";
        let new = "line1\nchanged\nline3\nline4\nadded\nline5";

        let diff = super::super::line_diff::unified_diff(old, new, "a/f.txt", "b/f.txt", 3);
        let result = apply_patch_text(old, &diff).unwrap();
        assert_eq!(result, new);
    }
}
