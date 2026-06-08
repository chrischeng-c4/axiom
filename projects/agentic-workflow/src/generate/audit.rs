// audit: skip-file — this module embeds CODEGEN-BEGIN/END + SPEC-MANAGED
// markers inside raw-string test fixtures; the scanner has no parser and
// would otherwise flag them as real blocks. The directive is checked by
// [`has_skip_file_directive`].

//! Codegen audit — regenerate-per-block + diff.
//!
//! The policy locked in `AUTHORING.md` §"Codegen Policy (1 / 2-1 / 2-2)"
//! says CODEGEN-BEGIN/END blocks are 100% generator-owned. Anything a
//! developer hand-edits inside a block is a contract violation. This
//! module provides the detection mechanism: for each block in a target
//! file, re-run its `SPEC-MANAGED` generator and diff the output against
//! the block's current content.
//!
//! The check is surgical — each section type has an independent callable
//! generator, so we re-render just the block's section rather than the
//! whole spec or the whole file. No hash sidecar needed; the generator
//! output is the authoritative "expected content" every time.
//!
//! # Scope (this iteration)
//!
//! - Regular CODEGEN blocks whose `SPEC-MANAGED` points at an existing
//!   spec file (e.g. `.aw/tech-design/projects/httpkit/http-exception.md#schema`).
//! - Recognises the four section types the apply pipeline dispatches
//!   today: manifest, unit-test/e2e-test, schema (and its mamba-binding sub-anchors),
//!   and the mermaid/cli/rpc-api fallback.
//! - Aggregator blocks whose `SPEC-MANAGED` starts with `generated/` —
//!   mod-decls, register-body, readme symbols — are tagged as
//!   `ReportKind::Aggregate` and skipped from content comparison (they
//!   depend on cross-file state that a standalone audit doesn't have).
//!   Handling them cleanly is R2-follow-up work.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::generate::apply::{
    extract_change_entries, generate_code_for_entry, generate_source_section_code, ChangeEntry,
    ImplMode,
};
use crate::generate::frontmatter::extract_mermaid_plus_blocks;
use crate::generate::marker::parse_codegen_blocks;

const AW_EC_BEGIN_MARKER: &str = "AW-EC-BEGIN";

/// Kind of audit finding per block.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReportKind {
    /// Block content matches what the generator would produce today.
    Clean,
    /// Block content differs from generator output → suspected hand-edit.
    /// `diff` carries the human-readable summary.
    Drift { diff: String },
    /// Block is produced by a cross-file aggregator (`generated/*` spec_ref).
    /// Content comparison requires whole-project context; this audit defers
    /// it to a follow-up check.
    Aggregate,
    /// Block's `SPEC-MANAGED` points at a spec file we could not load.
    Unresolvable { reason: String },
}

#[derive(Debug, Clone)]
pub struct BlockReport {
    pub file: PathBuf,
    pub spec_ref: String,
    pub kind: ReportKind,
}

/// Top-level item seen inside a CODEGEN block that lacks a preceding
/// `@spec` marker. Emitted by [`audit_markers`]; complements [`audit_file`]
/// (which catches content-level drift). Together they cover the safety
/// epic's R1 + R2-structural requirements.
#[derive(Debug, Clone)]
pub struct MarkerGap {
    pub file: PathBuf,
    /// The item signature line (e.g. `pub struct Foo`) that lacks a marker.
    pub item_line: String,
    /// 1-indexed line number in the target file.
    pub line_no: usize,
    /// The `SPEC-MANAGED` spec_ref of the enclosing CODEGEN block.
    pub enclosing_spec_ref: String,
}

/// Top-level `pub` item in a source file that lives OUTSIDE every CODEGEN
/// block AND resides in a file whose repo-relative path is claimed by at
/// least one spec's `changes:` entry. Hand-written code next to codegen'd
/// code is either (a) deliberate, (b) a spec-gap (the spec claims the file
/// but doesn't cover this item). This audit surfaces candidates for
/// governance review — it cannot tell (a) from (b).
#[derive(Debug, Clone)]
pub struct UncoveredItem {
    pub file: PathBuf,
    /// The item signature line (e.g. `pub struct Foo`).
    pub item_line: String,
    /// 1-indexed line number in the target file.
    pub line_no: usize,
    /// Relative paths of the TD specs whose `changes:` list includes
    /// `file`. Non-empty by construction — only items in spec-listed files
    /// become `UncoveredItem`s.
    pub claiming_specs: Vec<PathBuf>,
}

/// Audit every CODEGEN block in `file_path`. Returns one report per block.
pub fn audit_file(file_path: &Path, project_root: &Path) -> std::io::Result<Vec<BlockReport>> {
    let content = std::fs::read_to_string(file_path)?;
    if has_skip_file_directive(&content) {
        return Ok(Vec::new());
    }
    let blocks = parse_codegen_blocks(&content);

    let rel_path = file_path
        .strip_prefix(project_root)
        .unwrap_or(file_path)
        .to_string_lossy()
        .into_owned();

    let mut out = Vec::new();
    for block in blocks {
        let kind = audit_block(&block.spec_ref, &block.content, &rel_path, project_root);
        out.push(BlockReport {
            file: file_path.to_path_buf(),
            spec_ref: block.spec_ref.clone(),
            kind,
        });
    }
    Ok(out)
}

/// Audit a single block by regenerating its content from the spec it points at.
fn audit_block(
    spec_ref: &str,
    current_content: &str,
    target_rel_path: &str,
    project_root: &Path,
) -> ReportKind {
    let Some((spec_rel, section_id)) = split_spec_ref(spec_ref) else {
        return ReportKind::Unresolvable {
            reason: format!("malformed spec_ref: {}", spec_ref),
        };
    };

    // Cross-file aggregator blocks (mod decls, register body, readme symbols)
    // need project-level context. Defer them.
    if spec_rel.starts_with("generated/") {
        return ReportKind::Aggregate;
    }

    let spec_full = project_root.join(&spec_rel);
    let spec_content = match std::fs::read_to_string(&spec_full) {
        Ok(c) => c,
        Err(e) => {
            return ReportKind::Unresolvable {
                reason: format!("failed to read spec {}: {}", spec_rel, e),
            };
        }
    };

    if current_content.contains(AW_EC_BEGIN_MARKER) {
        return ReportKind::Clean;
    }

    if is_rust_source_path(target_rel_path)
        && is_handwritten_change_entry(&spec_content, target_rel_path, &section_id)
    {
        return ReportKind::Clean;
    }

    if crate::generate::apply::supports_source_backed_replay_for_spec(
        target_rel_path,
        Some(&section_id),
        &spec_rel,
    ) {
        return ReportKind::Clean;
    }
    let td_ast = crate::td_ast::parse::parse_td_str(&spec_content).ok();
    let expected_generated = regenerate_for_block(
        &spec_rel,
        &spec_content,
        td_ast.as_ref(),
        target_rel_path,
        &section_id,
        project_root,
    );
    let expected_raw = generated_block_content(&expected_generated, spec_ref);
    // Apply the same `use`-dedup the write path runs so audit's expected
    // content matches what actually lands on disk. The generator emits one
    // `use serde::...` per type inside a multi-type block; the write-side
    // post-pass collapses them. Without this, audit would flag every
    // multi-type block as drift.
    let expected = crate::generate::apply::dedupe_use_statements(&expected_raw);

    if normalize(&expected) == normalize(current_content)
        || (is_rust_source_path(target_rel_path)
            && rustfmt_normalized_eq(&expected, current_content).unwrap_or(false))
    {
        ReportKind::Clean
    } else {
        ReportKind::Drift {
            diff: summarize_diff(&expected, current_content),
        }
    }
}

fn generated_block_content(generated: &str, spec_ref: &str) -> String {
    let blocks = parse_codegen_blocks(generated);
    if let Some(block) = blocks.iter().find(|block| block.spec_ref == spec_ref) {
        return block.content.clone();
    }
    if blocks.len() == 1 {
        return blocks[0].content.clone();
    }
    generated.to_string()
}

fn is_rust_source_path(target_rel_path: &str) -> bool {
    Path::new(target_rel_path)
        .extension()
        .and_then(|ext| ext.to_str())
        == Some("rs")
}

fn is_handwritten_change_entry(
    spec_content: &str,
    target_rel_path: &str,
    section_id: &str,
) -> bool {
    extract_change_entries(spec_content)
        .into_iter()
        .any(|entry| {
            entry.path == target_rel_path
                && entry.section_id.as_deref() == Some(section_id)
                && entry.impl_mode == ImplMode::HandWritten
        })
}

/// Parse `"<spec_path>#<section_id>"` into its two parts.
fn split_spec_ref(spec_ref: &str) -> Option<(String, String)> {
    let (spec, section) = spec_ref.split_once('#')?;
    Some((spec.to_string(), section.to_string()))
}

/// Call the right generator for a block, mirroring `apply.rs::run_apply_inner`
/// dispatch but scoped to a single entry.
fn regenerate_for_block(
    spec_path: &str,
    spec_content: &str,
    td_ast: Option<&crate::td_ast::types::TDAst>,
    target_rel_path: &str,
    section_id: &str,
    project_root: &Path,
) -> String {
    // Re-use the dispatch from the apply pipeline so audit output is by
    // construction byte-identical to what a fresh `score td gen-code` run
    // would emit.
    //
    // The change-entries table in `spec_content` already lists this target;
    // we look it up to inherit the correct `action` and description, but
    // fall back to a synthesised entry if the spec's changes section no
    // longer mentions the target (spec edited after code was generated —
    // worth flagging, but not this function's job).
    let entries: HashMap<(String, Option<String>), ChangeEntry> =
        extract_change_entries(spec_content)
            .into_iter()
            .map(|e| ((e.path.clone(), e.section_id.clone()), e))
            .collect();
    let entry = entries
        .get(&(target_rel_path.to_string(), Some(section_id.to_string())))
        .map(|e| ChangeEntry {
            path: e.path.clone(),
            action: e.action.clone(),
            description: e.description.clone(),
            section_id: e.section_id.clone(),
            impl_mode: e.impl_mode,
            replaces: e.replaces.clone(),
            exports: e.exports.clone(),
            preamble: e.preamble.clone(),
            pub_uses: e.pub_uses.clone(),
            rust_source: e.rust_source.clone(),
            trait_impl: e.trait_impl.clone(),
            handwrite_gap: e.handwrite_gap.clone(),
            handwrite_tracker: e.handwrite_tracker.clone(),
            handwrite_reason: e.handwrite_reason.clone(),
            handwrite_anchor: e.handwrite_anchor.clone(),
        })
        .unwrap_or_else(|| ChangeEntry {
            path: target_rel_path.to_string(),
            action: "modify".to_string(),
            description: None,
            section_id: Some(section_id.to_string()),
            impl_mode: ImplMode::Codegen,
            replaces: Vec::new(),
            exports: Vec::new(),
            preamble: None,
            pub_uses: Vec::new(),
            rust_source: None,
            trait_impl: None,
            handwrite_gap: None,
            handwrite_tracker: None,
            handwrite_reason: None,
            handwrite_anchor: None,
        });

    match section_id {
        "manifest" => crate::generate::gen::rust::manifest::generate_manifest(spec_content).code,
        "unit-test" | "tests" => {
            crate::generate::gen::rust::tests_gen::generate_tests(spec_content).code
        }
        "e2e-test" => crate::generate::gen::rust::tests_gen::generate_e2e_tests(spec_content).code,
        "source" => generate_source_section_code(
            spec_content,
            spec_path,
            Some(target_rel_path),
            project_root,
        ),
        _ => {
            let mermaid = td_ast
                .map(crate::generate::apply::mermaid_blocks_from_td_ast)
                .unwrap_or_else(|| extract_mermaid_plus_blocks(spec_content));
            generate_code_for_entry(&entry, spec_path, &mermaid, spec_content, td_ast)
        }
    }
}

/// Trim trailing whitespace per line + collapse leading/trailing blank lines.
fn normalize(s: &str) -> String {
    s.lines()
        .map(|l| l.trim_end().to_string())
        .collect::<Vec<_>>()
        .join("\n")
        .trim_matches('\n')
        .to_string()
}

fn rustfmt_normalized_eq(expected: &str, actual: &str) -> std::io::Result<bool> {
    let Some(expected_fmt) = rustfmt_snippet(expected)? else {
        return Ok(false);
    };
    let Some(actual_fmt) = rustfmt_snippet(actual)? else {
        return Ok(false);
    };
    Ok(normalize(&expected_fmt) == normalize(&actual_fmt))
}

fn rustfmt_snippet(source: &str) -> std::io::Result<Option<String>> {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or_default();
    let path = std::env::temp_dir().join(format!(
        "agentic-workflow-audit-rustfmt-{}-{nanos}.rs",
        std::process::id()
    ));
    std::fs::write(&path, source)?;
    let output = Command::new("rustfmt")
        .args(["--edition", "2021", "--"])
        .arg(&path)
        .output()?;
    if !output.status.success() {
        std::fs::remove_file(&path).ok();
        return Ok(None);
    }
    let formatted = std::fs::read_to_string(&path)?;
    std::fs::remove_file(&path).ok();
    Ok(Some(formatted))
}

/// Human-readable summary of an expected-vs-actual mismatch. Deliberately
/// simple — line count + first differing-line excerpt. A richer diff
/// renderer lives at the CLI level; core audit just needs to know IF + roughly
/// WHERE.
fn summarize_diff(expected: &str, actual: &str) -> String {
    let expected_lines: Vec<&str> = expected.lines().collect();
    let actual_lines: Vec<&str> = actual.lines().collect();
    for (i, (e, a)) in expected_lines.iter().zip(actual_lines.iter()).enumerate() {
        if e.trim_end() != a.trim_end() {
            return format!(
                "first diff at line {}: expected `{}`, actual `{}`",
                i + 1,
                e.trim_end(),
                a.trim_end(),
            );
        }
    }
    format!(
        "length differs: expected {} lines, actual {} lines",
        expected_lines.len(),
        actual_lines.len(),
    )
}

// ── marker coverage audit ──────────────────────────────────────────────────

/// Scan `file_path` for top-level items that live inside a CODEGEN block but
/// are missing a preceding `@spec` marker. This is the structural half of
/// the four-quadrant policy check (the content half lives in [`audit_file`]).
///
/// Rule 1 of the codegen policy says a CODEGEN block is 100% generator-
/// owned; anything hand-added inside a block — even syntactically valid
/// code — is a violation. Generators must emit an `@spec` marker on every
/// item they produce, so an item WITHOUT a marker inside a block is the
/// smoking gun of a hand-edit.
///
/// Pattern recognition is regex-style on raw text — no syn/AST — because
/// the generator output is tightly constrained. Items recognised at column
/// 0 inside a CODEGEN block:
///
/// - `pub struct <Name>`
/// - `pub enum <Name>`
/// - `impl <...>` (including `impl <Trait> for <Type>`)
/// - `#[no_mangle]` (attribute preceding the FFI shim)
/// - `pub unsafe extern "C" fn <name>`
/// - `pub fn <name>`
///
/// `use <...>;` lines are imports, not items — they get a pass.
pub fn audit_markers(file_path: &Path) -> std::io::Result<Vec<MarkerGap>> {
    let content = std::fs::read_to_string(file_path)?;
    if has_skip_file_directive(&content) {
        return Ok(Vec::new());
    }
    let lines: Vec<&str> = content.lines().collect();

    // Walk the file, tracking CODEGEN-BEGIN / END state + the enclosing
    // spec_ref from the preceding SPEC-MANAGED line. Within a block, check
    // each item-signature line against a 5-line backward window for an
    // `@spec` marker.
    let mut in_block = false;
    let mut enclosing_ref = String::new();
    let mut gaps = Vec::new();
    let mut in_raw_string: Option<usize> = None;
    let mut in_regular_string = false;

    for (i, line) in lines.iter().enumerate() {
        if skip_literal_line(line, &mut in_raw_string, &mut in_regular_string) {
            continue;
        }

        let trimmed = line.trim_start();

        // Pick up the spec_ref when we see the SPEC-MANAGED comment.
        if let Some(r) = parse_spec_managed(trimmed) {
            enclosing_ref = r;
            continue;
        }
        if trimmed == "// CODEGEN-BEGIN"
            || trimmed == "# CODEGEN-BEGIN"
            || trimmed == "<!-- CODEGEN-BEGIN -->"
        {
            in_block = true;
            continue;
        }
        if trimmed == "// CODEGEN-END"
            || trimmed == "# CODEGEN-END"
            || trimmed == "<!-- CODEGEN-END -->"
        {
            in_block = false;
            enclosing_ref.clear();
            continue;
        }

        if !in_block {
            continue;
        }

        if !looks_like_top_level_item(line) {
            continue;
        }
        if has_spec_marker_above(&lines, i) {
            continue;
        }
        gaps.push(MarkerGap {
            file: file_path.to_path_buf(),
            item_line: line.trim_end().to_string(),
            line_no: i + 1,
            enclosing_spec_ref: enclosing_ref.clone(),
        });
    }
    Ok(gaps)
}

/// True when the file carries `// audit: skip-file` anywhere in its first
/// 20 lines. Used by modules that embed CODEGEN markers inside string
/// literals for test fixtures (e.g. this audit module's own unit tests) —
/// without the directive the scanner would read the literals as real blocks.
/// Scope is intentionally narrow: not a policy escape hatch, only a
/// false-positive filter for scanner-unfriendly source.
fn has_skip_file_directive(content: &str) -> bool {
    content
        .lines()
        .take(20)
        .any(|l| l.trim_start().starts_with("// audit: skip-file"))
}

fn parse_spec_managed(trimmed: &str) -> Option<String> {
    trimmed
        .strip_prefix("// SPEC-MANAGED: ")
        .or_else(|| trimmed.strip_prefix("# SPEC-MANAGED: "))
        .or_else(|| {
            trimmed
                .strip_prefix("<!-- SPEC-MANAGED: ")
                .and_then(|s| s.strip_suffix(" -->"))
        })
        .map(|s| s.trim().to_string())
}

/// Recognise generator-emitted top-level items. Keeps the match conservative
/// — only shapes we actually generate today.
fn looks_like_top_level_item(line: &str) -> bool {
    // Items live at column 0 in generator output (mamba_binding + schema
    // emit at module scope). Lines indented further are body content, not
    // item declarations.
    if line.starts_with(' ') || line.starts_with('\t') {
        return false;
    }
    let trimmed = line.trim();
    [
        "pub struct ",
        "pub enum ",
        "impl ",
        "pub fn ",
        "pub unsafe extern ",
        "pub unsafe fn ",
        "#[no_mangle]",
    ]
    .iter()
    .any(|prefix| trimmed.starts_with(prefix))
}

/// True when the immediate predecessor window before `idx` contains an
/// `@spec` marker. Generators emit `/// @spec <path>#<anchor>` directly on
/// the item, sometimes separated from the item by Rust outer attributes. Scan
/// upward through doc comments, ordinary comments, blank lines, and attributes
/// until a non-attribute source line establishes a boundary.
fn has_spec_marker_above(lines: &[&str], idx: usize) -> bool {
    let start = idx.saturating_sub(32);
    let mut in_multiline_attr = false;
    for i in (start..idx).rev() {
        let t = lines[i].trim_start();
        if t.contains("@spec ") {
            return true;
        }
        if t == "]" || t == ")]" || t.starts_with(")]") {
            in_multiline_attr = true;
            continue;
        }
        if in_multiline_attr {
            if t.starts_with("#[") {
                in_multiline_attr = false;
            }
            continue;
        }
        if t.is_empty() || t.starts_with("///") || t.starts_with("//") || t.starts_with("#[") {
            // doc / attribute / blank -- keep looking further up
            continue;
        }
        // anything else breaks the immediate-predecessor window
        break;
    }
    false
}

// ── Uncovered classification (R7) ──────────────────────────────────────────

/// Map `target file path (repo-relative) → list of spec paths (repo-relative)
/// whose `changes:` section claims that file`. Used by [`audit_uncovered`] to
/// decide whether a hand-written pub item deserves surfacing.
///
/// Both keys and values are repo-relative. Callers join against
/// `project_root` when they need absolute paths.
pub type SpecFileIndex = HashMap<PathBuf, Vec<PathBuf>>;

/// Walk `.aw/tech-design/` under `project_root` and build a file-index:
/// for each spec `.md`, read its Changes section and record every
/// `changes[].path` as being claimed by that spec.
pub fn build_spec_file_index(project_root: &Path) -> std::io::Result<SpecFileIndex> {
    let td_root = crate::shared::workspace::tech_design_path(project_root);
    let mut index: SpecFileIndex = HashMap::new();
    if !td_root.is_dir() {
        return Ok(index);
    }
    let mut spec_files = Vec::new();
    walk_md_recursive(&td_root, &mut spec_files)?;
    for spec in spec_files {
        let content = match std::fs::read_to_string(&spec) {
            Ok(c) => c,
            Err(_) => continue, // skip unreadable specs, not a rule violation
        };
        let entries = extract_change_entries(&content);
        if entries.is_empty() {
            continue;
        }
        let spec_rel = spec
            .strip_prefix(project_root)
            .unwrap_or(&spec)
            .to_path_buf();
        for e in entries {
            index
                .entry(PathBuf::from(&e.path))
                .or_default()
                .push(spec_rel.clone());
        }
    }
    Ok(index)
}

fn walk_md_recursive(dir: &Path, out: &mut Vec<PathBuf>) -> std::io::Result<()> {
    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        let Some(name) = path.file_name().and_then(|n| n.to_str()) else {
            continue;
        };
        if name.starts_with('.') {
            continue;
        }
        let ft = entry.file_type()?;
        if ft.is_symlink() {
            continue;
        }
        if ft.is_dir() {
            walk_md_recursive(&path, out)?;
        } else if ft.is_file()
            && path
                .extension()
                .and_then(|e| e.to_str())
                .is_some_and(|e| e == "md")
        {
            out.push(path);
        }
    }
    Ok(())
}

/// Scan `file_path` for top-level `pub` items outside every CODEGEN block
/// and return an `UncoveredItem` per item IFF `file_path`'s repo-relative
/// form is claimed by at least one spec in `index`. Files not in the index
/// are treated as "not under spec governance" — their hand-written items
/// produce zero findings.
pub fn audit_uncovered(
    file_path: &Path,
    project_root: &Path,
    index: &SpecFileIndex,
) -> std::io::Result<Vec<UncoveredItem>> {
    let content = std::fs::read_to_string(file_path)?;
    if has_skip_file_directive(&content) {
        return Ok(Vec::new());
    }

    // Only act on files that at least one spec claims.
    let rel = file_path
        .strip_prefix(project_root)
        .unwrap_or(file_path)
        .to_path_buf();
    let Some(claiming_specs) = index.get(&rel) else {
        return Ok(Vec::new());
    };

    let handwrite_ranges = handwrite_ranges_for_uncovered(&content, &file_path.to_string_lossy());
    let lines: Vec<&str> = content.lines().collect();
    let mut in_block = false;
    let mut in_raw_string: Option<usize> = None;
    let mut in_regular_string = false;
    let mut out = Vec::new();
    for (i, line) in lines.iter().enumerate() {
        let line_no = i + 1;
        if skip_literal_line(line, &mut in_raw_string, &mut in_regular_string) {
            continue;
        }
        if handwrite_ranges
            .iter()
            .any(|(start, end)| (*start..=*end).contains(&line_no))
        {
            continue;
        }
        let trimmed = line.trim_start();
        if trimmed == "// CODEGEN-BEGIN"
            || trimmed == "# CODEGEN-BEGIN"
            || trimmed == "<!-- CODEGEN-BEGIN -->"
        {
            in_block = true;
            continue;
        }
        if trimmed == "// CODEGEN-END"
            || trimmed == "# CODEGEN-END"
            || trimmed == "<!-- CODEGEN-END -->"
        {
            in_block = false;
            continue;
        }
        if in_block {
            continue;
        }
        if !looks_like_top_level_item(line) {
            continue;
        }
        out.push(UncoveredItem {
            file: file_path.to_path_buf(),
            item_line: line.trim_end().to_string(),
            line_no,
            claiming_specs: claiming_specs.clone(),
        });
    }
    Ok(out)
}

fn handwrite_ranges_for_uncovered(content: &str, file_path: &str) -> Vec<(usize, usize)> {
    if let Ok(markers) = parse_handwrite_markers(content, file_path) {
        return markers
            .into_iter()
            .map(|marker| (marker.line_start, marker.line_end))
            .collect();
    }
    lenient_handwrite_ranges(content)
}

fn lenient_handwrite_ranges(content: &str) -> Vec<(usize, usize)> {
    let mut ranges = Vec::new();
    let mut open_stack: Vec<usize> = Vec::new();
    let mut in_raw_string: Option<usize> = None;
    let mut in_regular_string = false;

    for (idx, raw_line) in content.lines().enumerate() {
        let line_no = idx + 1;
        if skip_literal_line(raw_line, &mut in_raw_string, &mut in_regular_string) {
            continue;
        }

        let body = strip_comment_lead(raw_line.trim_start());
        if body.starts_with("<HANDWRITE") && !body.starts_with("</HANDWRITE") {
            open_stack.push(line_no);
            continue;
        }
        if body.starts_with("</HANDWRITE") {
            if let Some(open) = open_stack.pop() {
                ranges.push((open, line_no));
            }
        }
    }

    ranges.sort_by_key(|(start, _)| *start);
    ranges
}

fn skip_literal_line(
    line: &str,
    in_raw_string: &mut Option<usize>,
    in_regular_string: &mut bool,
) -> bool {
    if *in_regular_string {
        if regular_string_continuation_closes(line) {
            *in_regular_string = false;
        }
        return true;
    }

    if let Some(pounds) = *in_raw_string {
        let closer = format!("\"{}", "#".repeat(pounds));
        if line.contains(&closer) {
            *in_raw_string = None;
        }
        return true;
    }

    if let Some(pounds) = detect_unclosed_raw_string(line) {
        *in_raw_string = Some(pounds);
        return true;
    }

    if opens_regular_string_continuation(line) {
        *in_regular_string = true;
        return true;
    }

    false
}

fn opens_regular_string_continuation(line: &str) -> bool {
    let mut in_string = false;
    let mut escaped = false;

    for ch in line.chars() {
        if escaped {
            escaped = false;
            continue;
        }
        if in_string {
            match ch {
                '\\' => escaped = true,
                '"' => in_string = false,
                _ => {}
            }
        } else if ch == '"' {
            in_string = true;
        }
    }

    in_string
}

fn regular_string_continuation_closes(line: &str) -> bool {
    let mut escaped = false;

    for ch in line.chars() {
        if escaped {
            escaped = false;
            continue;
        }
        match ch {
            '\\' => escaped = true,
            '"' => return true,
            _ => {}
        }
    }

    false
}

/// Four-status unified view used by the CLI + tests.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UnifiedReport {
    /// BlockReport status in `ReportKind::Clean`.
    Clean { file: PathBuf, spec_ref: String },
    /// BlockReport status in `ReportKind::Drift`.
    Drift {
        file: PathBuf,
        spec_ref: String,
        diff: String,
    },
    /// From `audit_markers` — pub item inside CODEGEN block without `@spec`.
    MarkerGap {
        file: PathBuf,
        item_line: String,
        line_no: usize,
        enclosing_spec_ref: String,
    },
    /// From `audit_uncovered` — pub item outside any block in a spec-listed file.
    Uncovered {
        file: PathBuf,
        item_line: String,
        line_no: usize,
        claiming_specs: Vec<PathBuf>,
    },
    /// Aggregator blocks deferred.
    Aggregate { file: PathBuf, spec_ref: String },
    /// SPEC-MANAGED points at a missing spec.
    Unresolvable {
        file: PathBuf,
        spec_ref: String,
        reason: String,
    },
    /// HANDWRITE marker region — hand-written code annotated with a codegen
    /// gap. Produced by `parse_handwrite_markers`. Subsumes the old
    /// `score sdd coverage` view: each HANDWRITE block is a remaining
    /// generator-gap candidate.
    Handwrite {
        file: PathBuf,
        gap: String,
        tracker: String,
        reason: String,
        line_start: usize,
        line_end: usize,
    },
}

impl UnifiedReport {
    pub fn is_clean(&self) -> bool {
        matches!(
            self,
            UnifiedReport::Clean { .. }
                | UnifiedReport::Aggregate { .. }
                | UnifiedReport::Handwrite { .. }
        )
    }

    /// File path the report points at (used for `--group-by file`).
    pub fn file(&self) -> &Path {
        match self {
            UnifiedReport::Clean { file, .. }
            | UnifiedReport::Drift { file, .. }
            | UnifiedReport::MarkerGap { file, .. }
            | UnifiedReport::Uncovered { file, .. }
            | UnifiedReport::Aggregate { file, .. }
            | UnifiedReport::Unresolvable { file, .. }
            | UnifiedReport::Handwrite { file, .. } => file,
        }
    }

    /// Status string used for `--group-by status` and JSON output.
    pub fn status(&self) -> &'static str {
        match self {
            UnifiedReport::Clean { .. } => "clean",
            UnifiedReport::Drift { .. } => "drift",
            UnifiedReport::MarkerGap { .. } => "marker_gap",
            UnifiedReport::Uncovered { .. } => "uncovered",
            UnifiedReport::Aggregate { .. } => "aggregate",
            UnifiedReport::Unresolvable { .. } => "unresolvable",
            UnifiedReport::Handwrite { .. } => "handwrite",
        }
    }

    /// Optional codegen-gap label (only Handwrite reports carry one).
    /// `MarkerGap` reports surface a synthetic `"missing-spec-marker"` gap so
    /// `--group-by gap` aggregates them alongside true gaps.
    pub fn gap(&self) -> Option<&str> {
        match self {
            UnifiedReport::Handwrite { gap, .. } => Some(gap.as_str()),
            UnifiedReport::MarkerGap { .. } => Some("missing-spec-marker"),
            _ => None,
        }
    }
}

/// Single-pass audit of one code file: runs `audit_file`, `audit_markers`,
/// and `audit_uncovered`, concatenates into a flat list of [`UnifiedReport`].
pub fn audit_file_unified(
    file_path: &Path,
    project_root: &Path,
    index: &SpecFileIndex,
) -> std::io::Result<Vec<UnifiedReport>> {
    let mut out = Vec::new();

    for b in audit_file(file_path, project_root)? {
        out.push(match b.kind {
            ReportKind::Clean => UnifiedReport::Clean {
                file: b.file,
                spec_ref: b.spec_ref,
            },
            ReportKind::Drift { diff } => UnifiedReport::Drift {
                file: b.file,
                spec_ref: b.spec_ref,
                diff,
            },
            ReportKind::Aggregate => UnifiedReport::Aggregate {
                file: b.file,
                spec_ref: b.spec_ref,
            },
            ReportKind::Unresolvable { reason } => UnifiedReport::Unresolvable {
                file: b.file,
                spec_ref: b.spec_ref,
                reason,
            },
        });
    }
    for g in audit_markers(file_path)? {
        out.push(UnifiedReport::MarkerGap {
            file: g.file,
            item_line: g.item_line,
            line_no: g.line_no,
            enclosing_spec_ref: g.enclosing_spec_ref,
        });
    }
    for u in audit_uncovered(file_path, project_root, index)? {
        out.push(UnifiedReport::Uncovered {
            file: u.file,
            item_line: u.item_line,
            line_no: u.line_no,
            claiming_specs: u.claiming_specs,
        });
    }
    // HANDWRITE markers — surface each as a Handwrite finding so audit
    // becomes the single source of truth for "where is the codegen gap?"
    // (subsumes the deprecated `score sdd coverage` view).
    if let Ok(content) = std::fs::read_to_string(file_path) {
        let path_str = file_path.to_string_lossy().to_string();
        if let Ok(markers) = parse_handwrite_markers(&content, &path_str) {
            for m in markers {
                out.push(UnifiedReport::Handwrite {
                    file: file_path.to_path_buf(),
                    gap: m.gap,
                    tracker: m.tracker,
                    reason: m.reason,
                    line_start: m.line_start,
                    line_end: m.line_end,
                });
            }
        }
    }
    Ok(out)
}

// ── HANDWRITE marker parser ─────────────────────────────────────────────────

// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/generate/audit.md#source
// CODEGEN-BEGIN

use crate::generate::handwrite::{HandwriteMarker, HandwriteParseError};

/// Failure variants surfaced by [`parse_handwrite_markers`].
///
/// @spec projects/agentic-workflow/tech-design/core/generate/handwrite-marker.md#schema
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HandwriteParseFailure {
    /// `<HANDWRITE ...>` line had no closing `</HANDWRITE>` before EOF.
    UnmatchedOpen { file: String, line: usize },
    /// `</HANDWRITE>` line appeared without a preceding open marker.
    UnmatchedClose { file: String, line: usize },
    /// `reason="..."` attribute was missing or empty.
    EmptyReason { file: String, line: usize },
    /// Required attribute `gap` / `tracker` / `reason` could not be parsed.
    MalformedAttributes {
        file: String,
        line: usize,
        message: String,
    },
}

/// @spec projects/agentic-workflow/tech-design/core/generate/audit.md#source
impl HandwriteParseFailure {
    fn into_struct_error(self) -> HandwriteParseError {
        match self {
            Self::UnmatchedOpen { file, line } => HandwriteParseError {
                file_path: file,
                line,
                message: "unmatched <HANDWRITE> open marker".to_string(),
            },
            Self::UnmatchedClose { file, line } => HandwriteParseError {
                file_path: file,
                line,
                message: "unmatched </HANDWRITE> close marker".to_string(),
            },
            Self::EmptyReason { file, line } => HandwriteParseError {
                file_path: file,
                line,
                message: "<HANDWRITE> marker missing or empty reason attribute".to_string(),
            },
            Self::MalformedAttributes {
                file,
                line,
                message,
            } => HandwriteParseError {
                file_path: file,
                line,
                message,
            },
        }
    }
}

/// Parse all HANDWRITE marker pairs in `content`, treating it as the body
/// of `file_path` (used only to populate error / marker records).
///
/// Implements the `parse-handwrite-markers` flowchart in
/// `projects/agentic-workflow/tech-design/core/generate/handwrite-marker.md#logic`:
///
/// 1. Scan each line.
/// 2. On `<HANDWRITE gap="..." tracker="..." reason="...">` push the
///    captured attributes onto an open-stack with the 1-based line number.
///    Empty / missing `reason` → push `EmptyReason`. Unparseable attrs →
///    push `MalformedAttributes`.
/// 3. On `</HANDWRITE>` pop the open-stack and emit a [`HandwriteMarker`].
///    Pop on empty stack → push `UnmatchedClose`.
/// 4. After scan, every record left on the open-stack → push
///    `UnmatchedOpen`.
/// 5. Return `Ok(Vec<HandwriteMarker>)` if no errors collected, else
///    `Err(Vec<HandwriteParseError>)`.
///
/// Markers may be nested (rare); the parser uses a stack so inner pairs
/// are emitted before outer pairs.
///
/// @spec projects/agentic-workflow/tech-design/core/generate/handwrite-marker.md#logic
pub fn parse_handwrite_markers(
    content: &str,
    file_path: &str,
) -> std::result::Result<Vec<HandwriteMarker>, Vec<HandwriteParseError>> {
    // Open-marker record (waiting for its closer).
    struct OpenMarker {
        gap: String,
        tracker: String,
        reason: String,
        line: usize,
    }

    let mut open_stack: Vec<OpenMarker> = Vec::new();
    let mut markers: Vec<HandwriteMarker> = Vec::new();
    let mut failures: Vec<HandwriteParseFailure> = Vec::new();

    // Raw-string state — when Some(n), we're inside an `r{n*'#'}"..."{n*'#'}`
    // raw string literal and lines must be skipped (e.g., test fixture lines
    // that *look* like HANDWRITE markers). Reset when the matching closer
    // appears.
    let mut in_raw_string: Option<usize> = None;

    // p_scan loop.
    for (idx, raw_line) in content.lines().enumerate() {
        let line_no = idx + 1;

        // Inside a raw string: only look for the closer; skip everything else.
        if let Some(pounds) = in_raw_string {
            let closer = format!("\"{}", "#".repeat(pounds));
            if raw_line.contains(&closer) {
                in_raw_string = None;
            }
            continue;
        }

        // Detect a raw-string opener on this line that does NOT close on the
        // same line. Supports `r"..."`, `r#"..."#`, `r##"..."##`, etc. We only
        // care about openers that span multiple lines — single-line raw
        // strings can't contain multi-line HANDWRITE blocks anyway.
        if let Some(pounds) = detect_unclosed_raw_string(raw_line) {
            in_raw_string = Some(pounds);
            continue;
        }

        let line = raw_line.trim_start();

        // Strip the leading `// ` comment marker if present so we treat
        // both `// <HANDWRITE ...>` and inline `<HANDWRITE ...>` (spec
        // examples) the same way.
        let body = strip_comment_lead(line);

        // p_match_begin
        if body.starts_with("<HANDWRITE") && body.contains("HANDWRITE ") {
            // Confirm not the close form.
            if body.starts_with("</HANDWRITE") {
                // fall through to close path below
            } else {
                match parse_attributes(body) {
                    Ok((gap, tracker, reason)) => {
                        // p_validate_reason
                        if reason.is_empty() {
                            failures.push(HandwriteParseFailure::EmptyReason {
                                file: file_path.to_string(),
                                line: line_no,
                            });
                        } else {
                            // p_push
                            open_stack.push(OpenMarker {
                                gap,
                                tracker,
                                reason,
                                line: line_no,
                            });
                        }
                        continue;
                    }
                    Err(msg) => {
                        // p_err_malformed
                        failures.push(HandwriteParseFailure::MalformedAttributes {
                            file: file_path.to_string(),
                            line: line_no,
                            message: msg,
                        });
                        continue;
                    }
                }
            }
        }

        // p_match_end — accept both `</HANDWRITE>` and any line whose
        // trimmed body starts with `</HANDWRITE` (covers `// </HANDWRITE>`
        // after the comment-strip above).
        if body.starts_with("</HANDWRITE") {
            // p_check_stack
            if let Some(open) = open_stack.pop() {
                // p_pop — emit marker.
                markers.push(HandwriteMarker {
                    file_path: file_path.to_string(),
                    line_start: open.line,
                    line_end: line_no,
                    gap: open.gap,
                    tracker: open.tracker,
                    reason: open.reason,
                });
            } else {
                // p_err_close
                failures.push(HandwriteParseFailure::UnmatchedClose {
                    file: file_path.to_string(),
                    line: line_no,
                });
            }
            continue;
        }
        // p_skip — non-marker line.
    }

    // p_check_leftovers — every BEGIN still on the stack is unmatched.
    while let Some(open) = open_stack.pop() {
        failures.push(HandwriteParseFailure::UnmatchedOpen {
            file: file_path.to_string(),
            line: open.line,
        });
    }

    // p_check_errors
    if failures.is_empty() {
        // Stack-pop ordering reverses inner-most last; sort by line_start
        // for deterministic output.
        markers.sort_by_key(|m| m.line_start);
        Ok(markers)
    } else {
        Err(failures
            .into_iter()
            .map(|f| f.into_struct_error())
            .collect())
    }
}

/// Detect an `r{n*'#'}"...` raw-string opener on `line` that does NOT close
/// on the same line. Returns the number of `#` signs (so the caller can
/// match the corresponding `"...{n*'#'}` closer on a later line).
///
/// Supports `r"..."` (n=0), `r#"..."#` (n=1), `r##"..."##` (n=2), etc.
/// Single-line raw strings (closer on same line) return `None` — they can't
/// contain multi-line HANDWRITE blocks.
fn detect_unclosed_raw_string(line: &str) -> Option<usize> {
    let bytes = line.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        // Look for `r` followed by zero-or-more `#` followed by `"`.
        // Only treat `r` as a raw-string opener when it's at a token boundary —
        // i.e. NOT preceded by an identifier char. Otherwise patterns like
        // `tracker="..."` (where `r"` appears as a substring inside an
        // attribute value) would falsely trigger raw-string mode.
        if bytes[i] == b'r' {
            // Skip when `r` is clearly NOT at a Rust token boundary:
            //   - preceded by an identifier char (e.g. `tracker="..."` — the
            //     `r"` is a substring inside an attribute name);
            //   - preceded by `"` or `'` (we're inside a regular string or
            //     char literal — e.g. `reason="r"` carries a literal `r`).
            // These heuristics avoid false positives without needing a full
            // Rust lexer.
            if i > 0 {
                let prev = bytes[i - 1];
                if prev.is_ascii_alphanumeric() || prev == b'_' || prev == b'"' || prev == b'\'' {
                    i += 1;
                    continue;
                }
            }
            let mut j = i + 1;
            while j < bytes.len() && bytes[j] == b'#' {
                j += 1;
            }
            let pounds = j - i - 1;
            if j < bytes.len() && bytes[j] == b'"' {
                // Found `r{pounds*'#'}"`. Look for matching closer on same line.
                let closer = format!("\"{}", "#".repeat(pounds));
                let after = &line[j + 1..];
                if let Some(rel) = after.find(&closer) {
                    // Closes on same line — skip past it and keep scanning
                    // for additional openers.
                    i = j + 1 + rel + closer.len();
                    continue;
                } else {
                    // No closer this line → unclosed opener.
                    return Some(pounds);
                }
            }
        }
        i += 1;
    }
    None
}

/// Strip the leading `// ` (or `//!`, `///`) line-comment marker so the
/// HANDWRITE body parser sees the same XML on every line variant.
fn strip_comment_lead(line: &str) -> &str {
    let s = line.trim_start();
    if let Some(rest) = s.strip_prefix("///") {
        return rest.trim_start();
    }
    if let Some(rest) = s.strip_prefix("//!") {
        return rest.trim_start();
    }
    if let Some(rest) = s.strip_prefix("//") {
        return rest.trim_start();
    }
    s
}

/// Parse the three required attributes from a `<HANDWRITE gap="..."
/// tracker="..." reason="...">` line. Attribute order is not fixed.
///
/// @spec projects/agentic-workflow/tech-design/core/generate/handwrite-marker.md#logic
fn parse_attributes(body: &str) -> std::result::Result<(String, String, String), String> {
    let gap =
        extract_attr(body, "gap").ok_or_else(|| "missing required attribute: gap".to_string())?;
    let tracker = extract_attr(body, "tracker")
        .ok_or_else(|| "missing required attribute: tracker".to_string())?;
    let reason = extract_attr(body, "reason")
        .ok_or_else(|| "missing required attribute: reason".to_string())?;
    Ok((gap, tracker, reason))
}

/// Extract `name="value"` from an XML-ish attribute soup. Supports
/// embedded escaped quotes (`\"`). Returns `None` when the attribute key
/// is absent.
fn extract_attr(body: &str, name: &str) -> Option<String> {
    let needle = format!("{}=\"", name);
    let start = body.find(&needle)? + needle.len();
    let bytes = body.as_bytes();
    let mut i = start;
    let mut value = String::new();
    while i < bytes.len() {
        let b = bytes[i];
        if b == b'\\' && i + 1 < bytes.len() {
            // escape sequence — keep literal next byte.
            value.push(bytes[i + 1] as char);
            i += 2;
            continue;
        }
        if b == b'"' {
            return Some(value);
        }
        value.push(b as char);
        i += 1;
    }
    // No closing quote → malformed.
    None
}
// CODEGEN-END
// ── tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod handwrite_tests {
    use super::*;

    const HANDWRITE_TOKEN: &str = "HANDWRITE";

    fn marker_begin(gap: &str, tracker: &str, reason: &str) -> String {
        format!(
            "// <{} gap=\"{}\" tracker=\"{}\" reason=\"{}\">",
            HANDWRITE_TOKEN, gap, tracker, reason
        )
    }

    fn marker_begin_without_reason(gap: &str, tracker: &str) -> String {
        format!(
            "// <{} gap=\"{}\" tracker=\"{}\">",
            HANDWRITE_TOKEN, gap, tracker
        )
    }

    fn marker_end() -> String {
        format!("// </{}>", HANDWRITE_TOKEN)
    }

    #[test]
    fn parses_well_formed_marker() {
        let src = format!(
            r#"
fn before() {{}}
{}
pub fn target() {{}}
{}
fn after() {{}}
"#,
            marker_begin(
                "missing-generator:logic",
                "pending-tracker",
                "example reason"
            ),
            marker_end(),
        );
        let out = parse_handwrite_markers(&src, "test.rs").unwrap();
        assert_eq!(out.len(), 1);
        assert_eq!(out[0].gap, "missing-generator:logic");
        assert_eq!(out[0].tracker, "pending-tracker");
        assert_eq!(out[0].reason, "example reason");
        assert!(out[0].line_start < out[0].line_end);
    }

    #[test]
    fn parses_multiple_markers() {
        let src = format!(
            r#"
{}
fn a() {{}}
{}
{}
fn b() {{}}
{}
"#,
            marker_begin("g1", "t1", "r1"),
            marker_end(),
            marker_begin("g2", "pending-tracker", "r2"),
            marker_end(),
        );
        let out = parse_handwrite_markers(&src, "x.rs").unwrap();
        assert_eq!(out.len(), 2);
        assert_eq!(out[0].gap, "g1");
        assert_eq!(out[1].gap, "g2");
    }

    #[test]
    fn rejects_empty_reason() {
        let src = format!("{}\n{}\n", marker_begin("g", "t", ""), marker_end());
        let err = parse_handwrite_markers(&src, "x.rs").unwrap_err();
        assert!(err.iter().any(|e| e.message.contains("empty reason")));
    }

    #[test]
    fn detects_unmatched_open() {
        let src = format!("{}\nfn x() {{}}\n", marker_begin("g", "t", "r"));
        let err = parse_handwrite_markers(&src, "x.rs").unwrap_err();
        assert!(err
            .iter()
            .any(|e| e.message.contains("unmatched <HANDWRITE>")));
    }

    #[test]
    fn detects_unmatched_close() {
        let src = format!("fn x() {{}}\n{}\n", marker_end());
        let err = parse_handwrite_markers(&src, "x.rs").unwrap_err();
        assert!(err
            .iter()
            .any(|e| e.message.contains("unmatched </HANDWRITE>")));
    }

    #[test]
    fn rejects_missing_attribute() {
        let src = format!(
            "{}\n{}\n",
            marker_begin_without_reason("g", "t"),
            marker_end()
        );
        let err = parse_handwrite_markers(&src, "x.rs").unwrap_err();
        assert!(err
            .iter()
            .any(|e| e.message.contains("missing required attribute")));
    }

    #[test]
    fn extract_attr_handles_escaped_quote() {
        let v = extract_attr("gap=\"a\\\"b\" tracker=\"t\" reason=\"r\"", "gap").unwrap();
        assert_eq!(v, "a\"b");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::generate::apply::run_apply;

    #[test]
    fn audit_accepts_aw_ec_generated_wrapper_with_canonical_ref() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        let td_dir = root.join(".aw/tech-design/projects/demo");
        std::fs::create_dir_all(&td_dir).unwrap();
        std::fs::write(
            td_dir.join("external-contracts.md"),
            r#"---
id: demo-external-contracts
fill_sections: [e2e-test]
---

## Demo Contract
<!-- type: e2e-test lang: yaml -->

```yaml
e2e_tests:
  - id: demo-contract
    command: cargo test -p demo demo_contract
```
"#,
        )
        .unwrap();

        let wrapper = r#"// AW-EC-BEGIN
// @ec demo-contract
// AW-EC-END

#[test]
fn demo_contract() {}
"#;

        assert_eq!(
            audit_block(
                ".aw/tech-design/projects/demo/external-contracts.md#demo-contract",
                wrapper,
                "projects/demo/tests/behavior_demo_contract.rs",
                root,
            ),
            ReportKind::Clean
        );
        assert!(matches!(
            audit_block(
                ".aw/tech-design/projects/demo/external-contracts.md:L8",
                wrapper,
                "projects/demo/tests/behavior_demo_contract.rs",
                root,
            ),
            ReportKind::Unresolvable { .. }
        ));
    }

    /// End-to-end: after running `run_apply` on a tiny spec, audit the
    /// generated file and verify every non-aggregate block reports `Clean`.
    /// Then corrupt one line and verify it flips to `Drift`.
    #[test]
    fn audit_detects_drift_after_hand_edit() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();

        let crate_dir = root.join("projects/mamba/mambalibs/httpkit");
        std::fs::create_dir_all(crate_dir.join("src")).unwrap();
        std::fs::write(
            crate_dir.join("Cargo.toml"),
            "[package]\nname = \"api\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
        )
        .unwrap();
        std::fs::write(
            crate_dir.join("src/lib.rs"),
            r#"pub struct X;
"#,
        )
        .unwrap();

        let td_dir = root.join(".aw/tech-design/projects/httpkit");
        std::fs::create_dir_all(&td_dir).unwrap();
        let spec_path = td_dir.join("http-exception.md");
        std::fs::write(
            &spec_path,
            r#"---
id: http-exception
fill_sections: [overview, schema, changes]
---

## Overview
<!-- type: overview lang: markdown -->

Audit fixture.

## Schema
<!-- type: schema lang: yaml -->

```yaml
title: HTTPException
type: object
required: [status_code]
properties:
  status_code:
    type: integer
    x-rust-type: u16
x-constructor:
  args:
    - { name: status_code, mb_type: int, rust_type: u16, default: "500" }
  validations:
    - { field: status_code, rule: range, min: 100, max: 599 }
x-mamba-binding:
  symbol: HTTPException
  extern_fn: http_exception_new
  signature: "HTTPException(status_code: int)"
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/mamba/mambalibs/httpkit/src/http_exception.rs
    action: create
    section: schema
```
"#,
        )
        .unwrap();

        run_apply(&spec_path, root, false).unwrap();

        // Happy path — audit reports clean for the schema block, aggregate
        // for any cross-file block.
        let target = crate_dir.join("src/http_exception.rs");
        let reports = audit_file(&target, root).unwrap();
        let non_aggregate: Vec<_> = reports
            .iter()
            .filter(|r| !matches!(r.kind, ReportKind::Aggregate))
            .collect();
        assert!(
            !non_aggregate.is_empty(),
            "at least one schema-backed block should be audited\n---\nreports: {:#?}",
            reports
        );
        for r in &non_aggregate {
            assert!(
                matches!(r.kind, ReportKind::Clean),
                "fresh output should audit clean, got: {:?}\nblock: {}",
                r.kind,
                r.spec_ref,
            );
        }

        // Drift path — corrupt a generated line, re-audit, expect `Drift`.
        let written = std::fs::read_to_string(&target).unwrap();
        let corrupted = written.replace("(100..=599).contains", "(100..=200).contains");
        assert_ne!(written, corrupted, "sanity: replace must have hit");
        std::fs::write(&target, corrupted).unwrap();

        let reports = audit_file(&target, root).unwrap();
        let drifted: Vec<_> = reports
            .iter()
            .filter(|r| matches!(r.kind, ReportKind::Drift { .. }))
            .collect();
        assert_eq!(
            drifted.len(),
            1,
            "exactly one block should report drift after a targeted edit\n---\nall reports: {:#?}",
            reports,
        );
        if let ReportKind::Drift { diff } = &drifted[0].kind {
            assert!(
                diff.contains("(100..=599)") || diff.contains("(100..=200)"),
                "diff message should name the offending range: {}",
                diff,
            );
        }
    }

    #[test]
    fn audit_replay_uses_typed_td_ast_schema_order() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();

        let crate_dir = root.join("projects/mamba/mambalibs/httpkit");
        std::fs::create_dir_all(crate_dir.join("src")).unwrap();

        let td_dir = root.join(".aw/tech-design/projects/httpkit");
        std::fs::create_dir_all(&td_dir).unwrap();
        let spec_path = td_dir.join("schema-order.md");
        std::fs::write(
            &spec_path,
            r#"---
id: schema-order
fill_sections: [schema, changes]
---

# Schema Order

## Schema
<!-- type: schema lang: yaml -->

```yaml
definitions:
  BetaType:
    type: object
    required: [value]
    properties:
      value: { type: string }
  AlphaType:
    type: object
    required: [value]
    properties:
      value: { type: string }
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/mamba/mambalibs/httpkit/src/schema_order.rs
    action: create
    section: schema
```
"#,
        )
        .unwrap();

        run_apply(&spec_path, root, false).unwrap();

        let target = crate_dir.join("src/schema_order.rs");
        let written = std::fs::read_to_string(&target).unwrap();
        assert!(
            written.find("pub struct AlphaType").unwrap()
                < written.find("pub struct BetaType").unwrap(),
            "fixture should exercise typed TD AST schema ordering\n{}",
            written
        );

        let reports = audit_file(&target, root).unwrap();
        assert!(
            reports
                .iter()
                .all(|report| !matches!(report.kind, ReportKind::Drift { .. })),
            "fresh typed-AST generated schema must audit clean\n{:#?}",
            reports
        );
    }

    /// Aggregator blocks (`generated/...` spec_ref) are deferred — audit
    /// reports `Aggregate` without attempting content comparison.
    #[test]
    fn audit_defers_aggregator_blocks() {
        // Minimal file with a fake aggregator block.
        let tmp = tempfile::tempdir().unwrap();
        let file = tmp.path().join("lib.rs");
        std::fs::write(
            &file,
            r#"// SPEC-MANAGED: generated/mamba-registry#mamba-mod-decls
// CODEGEN-BEGIN
pub mod foo;
// CODEGEN-END
"#,
        )
        .unwrap();

        let reports = audit_file(&file, tmp.path()).unwrap();
        assert_eq!(reports.len(), 1);
        assert!(matches!(reports[0].kind, ReportKind::Aggregate));
    }

    /// Marker audit — generator-shaped block with every item carrying
    /// `@spec` reports zero gaps.
    #[test]
    fn marker_audit_clean_when_every_item_has_spec() {
        let tmp = tempfile::tempdir().unwrap();
        let file = tmp.path().join("foo.rs");
        std::fs::write(
            &file,
            r#"// SPEC-MANAGED: .aw/tech-design/foo.md#schema
// CODEGEN-BEGIN
use serde::{Deserialize, Serialize};

/// @spec .aw/tech-design/foo.md#schema
#[derive(Debug, Clone)]
pub struct Foo {
    pub bar: String,
}

/// @spec .aw/tech-design/foo.md#x-constructor
impl Foo {
    pub fn new(bar: String) -> Self { Self { bar } }
}
// CODEGEN-END
"#,
        )
        .unwrap();

        let gaps = audit_markers(&file).unwrap();
        assert!(
            gaps.is_empty(),
            "clean file should produce no gaps, got {:#?}",
            gaps
        );
    }

    /// Marker audit treats multiline Rust outer attributes as part of the
    /// annotated item instead of as a boundary between `@spec` and the item.
    #[test]
    fn marker_audit_accepts_spec_before_multiline_attributes() {
        let tmp = tempfile::tempdir().unwrap();
        let file = tmp.path().join("cli.rs");
        std::fs::write(
            &file,
            r#"// SPEC-MANAGED: .aw/tech-design/foo.md#schema
// CODEGEN-BEGIN
/// @spec .aw/tech-design/foo.md#schema
#[derive(Debug, Parser)]
#[command(
    name = "jet-parity-corpus",
    about = "List, inspect, and verify the jet parity fixture corpus."
)]
pub struct FixturesCli {
    pub command: String,
}
// CODEGEN-END
"#,
        )
        .unwrap();

        let gaps = audit_markers(&file).unwrap();
        assert!(
            gaps.is_empty(),
            "multiline attributes should not hide the @spec marker, got {:#?}",
            gaps
        );
    }

    /// Marker audit — hand-added item inside a CODEGEN block without a
    /// preceding `@spec` is flagged.
    #[test]
    fn marker_audit_flags_item_without_spec_inside_block() {
        let tmp = tempfile::tempdir().unwrap();
        let file = tmp.path().join("foo.rs");
        std::fs::write(
            &file,
            r#"// SPEC-MANAGED: .aw/tech-design/foo.md#schema
// CODEGEN-BEGIN
/// @spec .aw/tech-design/foo.md#schema
pub struct Foo;

pub fn hand_added_helper() { }
// CODEGEN-END
"#,
        )
        .unwrap();

        let gaps = audit_markers(&file).unwrap();
        assert_eq!(gaps.len(), 1, "exactly one gap expected, got {:#?}", gaps);
        assert!(
            gaps[0].item_line.contains("hand_added_helper"),
            "gap should name the offending fn: {:?}",
            gaps[0]
        );
        assert_eq!(gaps[0].enclosing_spec_ref, ".aw/tech-design/foo.md#schema");
    }

    /// Marker audit — items outside any CODEGEN block are hand-written by
    /// definition; their absence of `@spec` is correct, not a gap.
    #[test]
    fn marker_audit_ignores_items_outside_codegen_blocks() {
        let tmp = tempfile::tempdir().unwrap();
        let file = tmp.path().join("lib.rs");
        std::fs::write(
            &file,
            r#"pub struct HandWritten;

impl HandWritten {
    pub fn new() -> Self { Self }
}

// SPEC-MANAGED: .aw/tech-design/foo.md#schema
// CODEGEN-BEGIN
/// @spec .aw/tech-design/foo.md#schema
pub struct Gen;
// CODEGEN-END
"#,
        )
        .unwrap();

        let gaps = audit_markers(&file).unwrap();
        assert!(
            gaps.is_empty(),
            "hand-written items outside blocks are fine, got {:#?}",
            gaps
        );
    }

    /// Marker audit — `use` lines inside blocks don't need `@spec`.
    #[test]
    fn marker_audit_use_statements_need_no_marker() {
        let tmp = tempfile::tempdir().unwrap();
        let file = tmp.path().join("foo.rs");
        std::fs::write(
            &file,
            r#"// SPEC-MANAGED: .aw/tech-design/foo.md#schema
// CODEGEN-BEGIN
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// @spec .aw/tech-design/foo.md#schema
pub struct Foo;
// CODEGEN-END
"#,
        )
        .unwrap();

        let gaps = audit_markers(&file).unwrap();
        assert!(
            gaps.is_empty(),
            "use lines should not trigger gaps, got {:#?}",
            gaps
        );
    }

    /// Unresolvable blocks (spec file missing) report a clear reason.
    #[test]
    fn audit_reports_unresolvable_when_spec_missing() {
        let tmp = tempfile::tempdir().unwrap();
        let file = tmp.path().join("src.rs");
        std::fs::write(
            &file,
            r#"// SPEC-MANAGED: .aw/tech-design/does-not-exist.md#schema
// CODEGEN-BEGIN
pub struct Nope;
// CODEGEN-END
"#,
        )
        .unwrap();

        let reports = audit_file(&file, tmp.path()).unwrap();
        assert_eq!(reports.len(), 1);
        match &reports[0].kind {
            ReportKind::Unresolvable { reason } => {
                assert!(
                    reason.contains("failed to read spec"),
                    "reason should mention read failure: {}",
                    reason
                );
            }
            other => panic!("expected Unresolvable, got {:?}", other),
        }
    }

    /// `// audit: skip-file` short-circuits both audits. Used for source
    /// files that embed CODEGEN markers inside string literals for test
    /// fixtures; the scanner has no parser and would otherwise flag them.
    #[test]
    fn skip_file_directive_suppresses_both_audits() {
        let tmp = tempfile::tempdir().unwrap();
        let file = tmp.path().join("meta.rs");
        std::fs::write(
            &file,
            r#"// audit: skip-file — embeds markers in test fixtures
//! Some module.

fn embed_for_tests() {
    let _ = r"// SPEC-MANAGED: .aw/tech-design/nope.md#schema
// CODEGEN-BEGIN
pub struct LooksReal;
// CODEGEN-END
";
}
"#,
        )
        .unwrap();

        let reports = audit_file(&file, tmp.path()).unwrap();
        assert!(
            reports.is_empty(),
            "skip-file should suppress block reports, got {:?}",
            reports
        );
        let gaps = audit_markers(&file).unwrap();
        assert!(
            gaps.is_empty(),
            "skip-file should suppress marker gaps, got {:?}",
            gaps
        );
    }

    // ── Uncovered (R7) tests ──────────────────────────────────────────────

    /// Write a TD spec with a `changes:` list naming `files`.
    fn write_td_spec(project_root: &Path, spec_rel: &str, claimed_files: &[&str]) {
        let spec_path = project_root.join(spec_rel);
        std::fs::create_dir_all(spec_path.parent().unwrap()).unwrap();
        let mut body = String::from("---\nid: test\n---\n\n## Changes\n<!-- type: changes lang: yaml -->\n\n```yaml\nchanges:\n");
        for f in claimed_files {
            body.push_str(&format!("  - path: {}\n    action: create\n", f));
        }
        body.push_str("```\n");
        std::fs::write(&spec_path, body).unwrap();
    }

    #[test]
    fn spec_index_maps_files_to_specs() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        write_td_spec(root, ".aw/tech-design/a.md", &["src/foo.rs", "src/bar.rs"]);
        write_td_spec(root, ".aw/tech-design/b.md", &["src/bar.rs"]);

        let idx = build_spec_file_index(root).unwrap();
        assert_eq!(idx.len(), 2);
        assert_eq!(idx[&PathBuf::from("src/foo.rs")].len(), 1);
        assert_eq!(idx[&PathBuf::from("src/bar.rs")].len(), 2);
    }

    #[test]
    fn spec_index_absent_tech_design_returns_empty() {
        let tmp = tempfile::tempdir().unwrap();
        let idx = build_spec_file_index(tmp.path()).unwrap();
        assert!(idx.is_empty());
    }

    #[test]
    fn uncovered_flags_pub_item_in_claimed_file() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        write_td_spec(root, ".aw/tech-design/a.md", &["src/foo.rs"]);

        std::fs::create_dir_all(root.join("src")).unwrap();
        std::fs::write(
            root.join("src/foo.rs"),
            r#"// SPEC-MANAGED: .aw/tech-design/a.md#schema
// CODEGEN-BEGIN
/// @spec .aw/tech-design/a.md#schema
pub struct Foo;
// CODEGEN-END

pub fn hand_written_helper() { }
"#,
        )
        .unwrap();

        let idx = build_spec_file_index(root).unwrap();
        let items = audit_uncovered(&root.join("src/foo.rs"), root, &idx).unwrap();
        assert_eq!(items.len(), 1);
        assert!(items[0].item_line.contains("hand_written_helper"));
        assert_eq!(items[0].claiming_specs.len(), 1);
    }

    #[test]
    fn uncovered_ignores_file_not_claimed_by_any_spec() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        // Spec claims a different file.
        write_td_spec(root, ".aw/tech-design/a.md", &["src/other.rs"]);

        std::fs::create_dir_all(root.join("src")).unwrap();
        std::fs::write(root.join("src/unclaimed.rs"), "pub fn whatever() { }\n").unwrap();

        let idx = build_spec_file_index(root).unwrap();
        let items = audit_uncovered(&root.join("src/unclaimed.rs"), root, &idx).unwrap();
        assert!(items.is_empty());
    }

    #[test]
    fn uncovered_ignores_items_inside_codegen_block() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        write_td_spec(root, ".aw/tech-design/a.md", &["src/foo.rs"]);

        std::fs::create_dir_all(root.join("src")).unwrap();
        std::fs::write(
            root.join("src/foo.rs"),
            r#"// SPEC-MANAGED: .aw/tech-design/a.md#schema
// CODEGEN-BEGIN
/// @spec .aw/tech-design/a.md#schema
pub struct Foo;

pub fn inside_block_fn() { }
// CODEGEN-END
"#,
        )
        .unwrap();

        let idx = build_spec_file_index(root).unwrap();
        let items = audit_uncovered(&root.join("src/foo.rs"), root, &idx).unwrap();
        assert!(
            items.is_empty(),
            "items inside block don't get Uncovered — that's a MarkerGap instead"
        );
    }

    #[test]
    fn uncovered_ignores_items_inside_handwrite_block() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        write_td_spec(root, ".aw/tech-design/a.md", &["src/foo.rs"]);

        std::fs::create_dir_all(root.join("src")).unwrap();
        std::fs::write(
            root.join("src/foo.rs"),
            r#"// <HANDWRITE gap="standardize:fold-shadow" tracker="t" reason="manual region">
pub fn hand_written_helper() { }
// </HANDWRITE>
"#,
        )
        .unwrap();

        let idx = build_spec_file_index(root).unwrap();
        let items = audit_uncovered(&root.join("src/foo.rs"), root, &idx).unwrap();
        assert!(
            items.is_empty(),
            "HANDWRITE regions should not be reported as uncovered"
        );
    }

    #[test]
    fn uncovered_ignores_handwrite_when_fixture_marker_confuses_strict_parser() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        write_td_spec(root, ".aw/tech-design/a.md", &["src/foo.rs"]);

        std::fs::create_dir_all(root.join("src")).unwrap();
        std::fs::write(
            root.join("src/foo.rs"),
            r#"// <HANDWRITE gap="standardize:fold-shadow" tracker="t" reason="manual region">
pub fn hand_written_helper() { }
// </HANDWRITE>

const FIXTURE: &str = "// <HANDWRITE gap=\"fixture\" tracker=\"t\" reason=\"fixture\">\n\
pub fn fixture_only() {}\n\
// </HANDWRITE>\n";
"#,
        )
        .unwrap();

        assert!(
            parse_handwrite_markers(
                &std::fs::read_to_string(root.join("src/foo.rs")).unwrap(),
                "src/foo.rs"
            )
            .is_err(),
            "fixture close marker should keep this regression on the fallback path"
        );

        let idx = build_spec_file_index(root).unwrap();
        let items = audit_uncovered(&root.join("src/foo.rs"), root, &idx).unwrap();
        assert!(
            items.is_empty(),
            "fallback HANDWRITE ranges and literal skipping should suppress fixture-only items"
        );
    }

    #[test]
    fn uncovered_ignores_pub_items_inside_raw_string_fixtures() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        write_td_spec(root, ".aw/tech-design/a.md", &["src/foo.rs"]);

        std::fs::create_dir_all(root.join("src")).unwrap();
        std::fs::write(
            root.join("src/foo.rs"),
            r##"const FIXTURE: &str = r#"
pub fn fixture_only() {}
"#;

pub fn real_shadow() {}
"##,
        )
        .unwrap();

        let idx = build_spec_file_index(root).unwrap();
        let items = audit_uncovered(&root.join("src/foo.rs"), root, &idx).unwrap();
        assert_eq!(items.len(), 1);
        assert!(items[0].item_line.contains("real_shadow"));
    }

    #[test]
    fn uncovered_respects_skip_file_directive() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        write_td_spec(root, ".aw/tech-design/a.md", &["src/foo.rs"]);

        std::fs::create_dir_all(root.join("src")).unwrap();
        std::fs::write(
            root.join("src/foo.rs"),
            "// audit: skip-file\npub fn x() { }\n",
        )
        .unwrap();

        let idx = build_spec_file_index(root).unwrap();
        let items = audit_uncovered(&root.join("src/foo.rs"), root, &idx).unwrap();
        assert!(items.is_empty());
    }

    #[test]
    fn unified_walks_emit_all_four_categories() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        write_td_spec(root, ".aw/tech-design/a.md", &["src/foo.rs"]);

        std::fs::create_dir_all(root.join("src")).unwrap();
        std::fs::write(
            root.join("src/foo.rs"),
            r#"// SPEC-MANAGED: .aw/tech-design/a.md#schema
// CODEGEN-BEGIN
/// @spec .aw/tech-design/a.md#schema
pub struct Foo;

pub fn smuggled_into_block() { }
// CODEGEN-END

pub fn hand_written_helper() { }
"#,
        )
        .unwrap();

        let idx = build_spec_file_index(root).unwrap();
        let reports = audit_file_unified(&root.join("src/foo.rs"), root, &idx).unwrap();

        let has_marker_gap = reports
            .iter()
            .any(|r| matches!(r, UnifiedReport::MarkerGap { .. }));
        let has_uncovered = reports
            .iter()
            .any(|r| matches!(r, UnifiedReport::Uncovered { .. }));
        assert!(
            has_marker_gap,
            "expected MarkerGap for smuggled_into_block, got: {:#?}",
            reports
        );
        assert!(
            has_uncovered,
            "expected Uncovered for hand_written_helper, got: {:#?}",
            reports
        );
    }
}
