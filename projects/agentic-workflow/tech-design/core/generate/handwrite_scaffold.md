---
id: projects-sdd-src-generate-handwrite-scaffold-rs
fill_sections: [overview, source, changes]
capability_refs:
  - id: td-cb-lifecycle-automation
    role: primary
    gap: cb-lifecycle-dispatch
    claim: cb-lifecycle-dispatch
    coverage: full
    rationale: "Generator primitives are part of TD/CB lifecycle automation because they produce reviewable code artifacts from TD sections."
---

# Standardized projects/agentic-workflow/src/generate/handwrite_scaffold.rs

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/agentic-workflow/src/generate/handwrite_scaffold.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `PENDING_TRACKER` | projects/agentic-workflow/src/generate/handwrite_scaffold.rs | constant | pub | 131 |  |
| `ScaffoldOutcome` | projects/agentic-workflow/src/generate/handwrite_scaffold.rs | enum | pub | 24 |  |
| `scaffold_handwrite` | projects/agentic-workflow/src/generate/handwrite_scaffold.rs | function | pub | 45 | scaffold_handwrite(     entry: &HandwriteEntry,     target_path: &Path,     anchor_symbol: &str,     section_id: Option<&str>, ) -> std::io::Result<ScaffoldOutcome> |
## Source
<!-- type: source lang: rust -->
<!-- source-from-target: strip-handwrite -->

<!-- source-snapshot: path=projects/agentic-workflow/src/generate/handwrite_scaffold.rs -->
```rust
//! HANDWRITE marker scaffold inserter.
//!
//! Implements the `scaffold-handwrite` flowchart from
//! `projects/agentic-workflow/tech-design/core/generate/handwrite-marker.md#logic`.
//!
//! `aw td gen-code` calls [`scaffold_handwrite`] for every spec
//! `changes` entry whose `impl_mode` is `hand-written`. The function
//! locates the entry's anchor symbol inside the target file, derives
//! `gap` / `tracker` / `reason` per the spec's `HandwriteEntry` defaults,
//! builds an XML-form `<HANDWRITE ...>` open/close pair, and inserts it
//! at the anchor. Idempotent: a file already carrying a HANDWRITE pair
//! for the same anchor is left untouched.

use std::path::Path;

use crate::generate::handwrite::HandwriteEntry;

/// Result of a single [`scaffold_handwrite`] call.
///
/// @spec projects/agentic-workflow/tech-design/core/generate/handwrite-marker.md#logic
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ScaffoldOutcome {
    /// Marker inserted; file written.
    Inserted,
    /// File already carried a marker for this anchor — no change made.
    Skipped,
    /// Anchor symbol was not found in the file.
    AnchorMissing,
}

/// Locate `anchor_symbol` in `target_path` and insert a HANDWRITE marker
/// pair (XML-form `<HANDWRITE gap="..." tracker="..." reason="...">` /
/// `</HANDWRITE>`) immediately surrounding the anchor's `pub fn` /
/// `pub struct` / `pub enum` line. The `entry` carries optional overrides
/// for `gap`, `tracker`, and `reason`; absent fields are derived from
/// `section_id` and the target filename per the spec.
///
/// Idempotent (R6): if the file already has any `<HANDWRITE ...>` line
/// referencing `anchor_symbol`'s line, the function returns
/// [`ScaffoldOutcome::Skipped`] without modifying the file.
///
/// @spec projects/agentic-workflow/tech-design/core/generate/handwrite-marker.md#logic
pub fn scaffold_handwrite(
    entry: &HandwriteEntry,
    target_path: &Path,
    anchor_symbol: &str,
    section_id: Option<&str>,
) -> std::io::Result<ScaffoldOutcome> {
    // s_read — read target file lines.
    let original = if target_path.exists() {
        std::fs::read_to_string(target_path)?
    } else {
        // Brand-new file path: cannot anchor; fall through and let caller
        // decide. We return AnchorMissing so the caller can choose to
        // create a stub or report the gap.
        String::new()
    };

    let mut lines: Vec<String> = if original.is_empty() {
        Vec::new()
    } else {
        original.lines().map(|l| l.to_string()).collect()
    };

    // s_anchor — locate anchor symbol.
    let anchor_idx = lines
        .iter()
        .position(|l| line_matches_anchor(l, anchor_symbol));
    let Some(anchor_idx) = anchor_idx else {
        return Ok(ScaffoldOutcome::AnchorMissing);
    };

    // s_existing — already wrapped?
    if has_surrounding_marker(&lines, anchor_idx) {
        return Ok(ScaffoldOutcome::Skipped);
    }

    // s_gap, s_tracker, s_reason — derive defaults.
    let gap = derive_gap(entry, section_id);
    let tracker = derive_tracker(entry);
    let reason = derive_reason(entry, section_id, target_path);

    // s_build — build BEGIN / END marker lines.
    let begin = format!(
        "// <HANDWRITE gap=\"{}\" tracker=\"{}\" reason=\"{}\">",
        escape_attr(&gap),
        escape_attr(&tracker),
        escape_attr(&reason),
    );
    let end = "// </HANDWRITE>".to_string();

    // s_insert — splice marker pair around the anchor.
    //
    // Preserve any `///` doc-comments and `#[...]` attributes that sit
    // immediately above the anchor line so the BEGIN marker lands above
    // them (matching the convention in existing markers).
    let mut block_start = anchor_idx;
    while block_start > 0 {
        let prev = lines[block_start - 1].trim_start();
        if prev.starts_with("///") || prev.starts_with("//!") || prev.starts_with("#[") {
            block_start -= 1;
        } else {
            break;
        }
    }

    // Find the matching close brace for fn / struct / enum / impl block.
    let block_end = find_block_end(&lines, anchor_idx);

    // Insert END after block_end, then BEGIN before block_start (insert
    // higher index first so block_start stays valid).
    lines.insert(block_end + 1, end);
    lines.insert(block_start, begin);

    // s_write — write back, preserving trailing newline if the original
    // file had one.
    let mut out = lines.join("\n");
    if original.ends_with('\n') || original.is_empty() {
        out.push('\n');
    }
    std::fs::write(target_path, out)?;
    Ok(ScaffoldOutcome::Inserted)
}

/// Sentinel value used when `entry.tracker` is absent. Matches R3 / R10
/// in `sdd-handwrite-marker#schema`.
///
/// @spec projects/agentic-workflow/tech-design/core/generate/handwrite-marker.md#schema
pub const PENDING_TRACKER: &str = "pending-tracker";

/// Derive the `gap` attribute for the inserted marker.
///
/// @spec projects/agentic-workflow/tech-design/core/generate/handwrite-marker.md#logic
fn derive_gap(entry: &HandwriteEntry, section_id: Option<&str>) -> String {
    if let Some(g) = entry.gap.as_ref().filter(|s| !s.is_empty()) {
        return g.clone();
    }
    match section_id.unwrap_or("logic") {
        "logic" => "missing-generator:logic".to_string(),
        "schema" => "missing-generator:schema".to_string(),
        "cli" => "missing-generator:cli".to_string(),
        "state-machine" => "missing-generator:state-machine".to_string(),
        "interaction" => "missing-generator:interaction".to_string(),
        "test-plan" => "missing-generator:test-plan".to_string(),
        other => format!("missing-generator:{}", other),
    }
}

/// Derive the `tracker` attribute. Returns the spec-provided value when
/// present and non-empty, otherwise the [`PENDING_TRACKER`] sentinel.
///
/// @spec projects/agentic-workflow/tech-design/core/generate/handwrite-marker.md#logic
fn derive_tracker(entry: &HandwriteEntry) -> String {
    entry
        .tracker
        .as_ref()
        .filter(|s| !s.is_empty())
        .cloned()
        .unwrap_or_else(|| PENDING_TRACKER.to_string())
}

/// Derive the `reason` attribute. Synthesises a placeholder from
/// `section_id` and the target file when no explicit reason is given,
/// guaranteeing a non-empty value (R9).
///
/// @spec projects/agentic-workflow/tech-design/core/generate/handwrite-marker.md#logic
fn derive_reason(entry: &HandwriteEntry, section_id: Option<&str>, target_path: &Path) -> String {
    if let Some(r) = entry.reason.as_ref().filter(|s| !s.is_empty()) {
        return r.clone();
    }
    let section = section_id.unwrap_or("logic");
    let file = target_path
        .file_name()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_else(|| "<target>".to_string());
    format!(
        "{} section in {} is hand-written pending codegen support",
        section, file
    )
}

/// Match a candidate `pub fn` / `pub struct` / `pub enum` / `pub trait` /
/// `impl` line against `anchor_symbol`. The anchor symbol may be a bare
/// identifier (matches first `pub fn|struct|enum|trait <ident>`) or
/// `impl Trait for Type` style prefix.
fn line_matches_anchor(line: &str, anchor: &str) -> bool {
    let t = line.trim_start();
    if anchor.starts_with("impl ") {
        return t.starts_with(anchor);
    }
    let needles = [
        format!("pub fn {}", anchor),
        format!("pub fn {}<", anchor),
        format!("pub fn {}(", anchor),
        format!("pub struct {}", anchor),
        format!("pub enum {}", anchor),
        format!("pub trait {}", anchor),
        format!("pub(crate) fn {}", anchor),
        format!("pub(crate) struct {}", anchor),
        format!("pub(crate) enum {}", anchor),
        format!("fn {}(", anchor),
        format!("fn {}<", anchor),
    ];
    needles.iter().any(|n| t.starts_with(n))
}

/// True when an existing HANDWRITE BEGIN marker sits within 8 lines above
/// the anchor (covers the doc-comment + attr preamble we emit ourselves).
fn has_surrounding_marker(lines: &[String], anchor_idx: usize) -> bool {
    let lookback = anchor_idx.saturating_sub(16);
    for line in lines[lookback..anchor_idx].iter() {
        let t = line.trim_start();
        if t.starts_with("// <HANDWRITE") {
            return true;
        }
    }
    false
}

/// Find the matching close-brace line for a Rust block started at
/// `anchor_idx`. Counts `{` / `}` ignoring string literals only crudely
/// (sufficient for idiomatic Rust). Returns `anchor_idx` itself when the
/// anchor is a one-line item (e.g. `pub use`, `pub struct Foo;`).
fn find_block_end(lines: &[String], anchor_idx: usize) -> usize {
    // Find first line that opens a brace from anchor_idx onward.
    let mut depth: i64 = 0;
    let mut started = false;
    for (i, line) in lines.iter().enumerate().skip(anchor_idx) {
        for c in line.chars() {
            if c == '{' {
                depth += 1;
                started = true;
            } else if c == '}' {
                depth -= 1;
                if started && depth == 0 {
                    return i;
                }
            }
        }
        // Single-line item ending in `;` before any brace.
        if !started && line.trim_end().ends_with(';') {
            return i;
        }
    }
    // Unbalanced — best-effort: end of file.
    lines.len().saturating_sub(1).max(anchor_idx)
}

/// Escape a value for inclusion inside a `attr="..."` HTML/XML attribute.
/// We simply replace `"` and embedded newlines; gap/tracker never contain
/// quotes by convention, but reasons sometimes do.
fn escape_attr(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', " ")
        .replace('\r', " ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn derive_gap_uses_entry_value_when_present() {
        let entry = HandwriteEntry {
            gap: Some("missing-primitive:foo".to_string()),
            tracker: None,
            reason: None,
        };
        assert_eq!(derive_gap(&entry, Some("logic")), "missing-primitive:foo");
    }

    #[test]
    fn derive_gap_falls_back_to_section_id() {
        let entry = HandwriteEntry::default();
        assert_eq!(derive_gap(&entry, Some("logic")), "missing-generator:logic");
        assert_eq!(
            derive_gap(&entry, Some("schema")),
            "missing-generator:schema"
        );
        assert_eq!(derive_gap(&entry, Some("cli")), "missing-generator:cli");
        assert_eq!(
            derive_gap(&entry, Some("custom")),
            "missing-generator:custom"
        );
    }

    #[test]
    fn derive_tracker_defaults_to_pending() {
        let entry = HandwriteEntry::default();
        assert_eq!(derive_tracker(&entry), PENDING_TRACKER);
        let entry = HandwriteEntry {
            gap: None,
            tracker: Some("issue-foo-bar".to_string()),
            reason: None,
        };
        assert_eq!(derive_tracker(&entry), "issue-foo-bar");
    }

    #[test]
    fn derive_reason_synthesises_when_absent() {
        let entry = HandwriteEntry::default();
        let p = Path::new("projects/agentic-workflow/src/foo.rs");
        let r = derive_reason(&entry, Some("logic"), p);
        assert!(!r.is_empty());
        assert!(r.contains("foo.rs"));
    }
}
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/generate/handwrite_scaffold.rs
    action: modify
    section: source
    impl_mode: codegen
    description: |
      Source template owns the complete HANDWRITE scaffold insertion module.
```
