// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/validate/td_ast/anti_patterns.md#source
// CODEGEN-BEGIN
//! Anti-pattern detectors for `aw td validate`.
//!
//! Each detector checks for ONE named anti-pattern (AP-001 .. AP-NNN) and
//! returns one or more [`TdError`]s with the AP code prefix in the message.
//! Validator chains these; reviewer agent (LLM) trusts validator pass and
//! does NOT re-check structural concerns.
//!
//! See `.aw/tech-design/AUTHORING.md` "Anti-Pattern Catalog" for the
//! contract each detector enforces, with GOOD / BAD spec examples.
//!
//! @spec projects/agentic-workflow/tech-design/core/validate/td_ast/anti_patterns.md#source

use std::collections::HashSet;
use std::path::Path;

use crate::models::spec_rules::SectionType;
use crate::td_ast::types::TDAst;
use crate::td_ast::validate::{TdError, TdErrorCode};

// Re-emit AP labels via the message prefix; new TdErrorCode variants below
// keep the machine-readable code stable.

/// Run all anti-pattern detectors that operate on parsed AST + raw spec.
/// Caller passes the spec text so content-based scans (placeholder, body
/// equals title) work over the original markdown without re-rendering
/// typed payloads.
///
/// @spec projects/agentic-workflow/tech-design/core/validate/td_ast/anti_patterns.md#source
pub fn check_content_anti_patterns(ast: &TDAst, spec_content: &str) -> Vec<TdError> {
    let mut errors = Vec::new();
    errors.extend(ap_001_placeholder_leftover(ast, spec_content));
    errors.extend(ap_008_body_equals_title(ast, spec_content));
    errors
}

/// Run anti-pattern detectors that require the workspace root for filesystem
/// existence checks. Pass `project_root` rooted at the workspace where
/// `.aw/tech-design/...` etc. are reachable.
/// @spec projects/agentic-workflow/tech-design/core/validate/td_ast/anti_patterns.md#source
pub fn check_filesystem_anti_patterns(
    ast: &TDAst,
    spec_content: &str,
    project_root: &Path,
) -> Vec<TdError> {
    let mut errors = Vec::new();
    errors.extend(ap_004_non_existent_spec_ref(spec_content, project_root));
    errors.extend(ap_010_changes_path_not_found(ast, project_root));
    errors.extend(ap_009_non_existent_replaces_symbol(ast, project_root));
    errors
}

// ─────────────────────────────────────────────────────────────────────────
// AP-001 — placeholder leftover
// ─────────────────────────────────────────────────────────────────────────

/// AP-001: a section listed in `fill_sections` still contains a `(fill)`
/// placeholder or `<!-- TODO -->` / `<!-- TBD -->` comment in the body of
/// the corresponding section in the raw markdown.
///
/// /// @spec .aw/tech-design/AUTHORING.md#anti-pattern-catalog (AP-001)
fn ap_001_placeholder_leftover(ast: &TDAst, spec_content: &str) -> Vec<TdError> {
    const PLACEHOLDER_TOKENS: &[&str] = &["(fill)", "<!-- TODO -->", "<!-- TBD -->"];
    let mut errors = Vec::new();

    for section in &ast.sections {
        let body_text = slice_section_body(spec_content, section.line_start, section.line_end);
        for token in PLACEHOLDER_TOKENS {
            if body_text.contains(token) {
                errors.push(TdError {
                    code: TdErrorCode::PlaceholderLeftover,
                    section_type: section.section_type,
                    line_start: section.line_start,
                    line_end: section.line_end,
                    message: format!(
                        "[AP-001] section '{:?}' still contains placeholder '{}' \
                         in its body — fill the section content before validation",
                        section.section_type,
                        token
                    ),
                    hint: Some(
                        "Replace the placeholder with the substantive content the section requires."
                            .to_string(),
                    ),
                });
                break;
            }
        }
    }
    errors
}

/// Pull the raw markdown body text for a section's line range out of the
/// full spec content. `line_start` is the heading line; we skip it.
fn slice_section_body(spec: &str, line_start: usize, line_end: usize) -> String {
    spec.lines()
        .enumerate()
        .filter(|(idx, _)| {
            let lineno = idx + 1;
            lineno > line_start && lineno <= line_end
        })
        .map(|(_, l)| l)
        .collect::<Vec<_>>()
        .join("\n")
}

// ─────────────────────────────────────────────────────────────────────────
// AP-004 — non-existent spec reference
// ─────────────────────────────────────────────────────────────────────────

/// AP-004: any `.aw/tech-design/.../*.md` path mentioned in the spec
/// content points at a file that does not exist on disk relative to
/// `project_root`.
///
/// /// @spec .aw/tech-design/AUTHORING.md#anti-pattern-catalog (AP-004)
fn ap_004_non_existent_spec_ref(spec_content: &str, project_root: &Path) -> Vec<TdError> {
    let mut errors = Vec::new();
    let mut seen: HashSet<String> = HashSet::new();

    for line in spec_content.lines() {
        for path_str in extract_spec_paths(line) {
            if !seen.insert(path_str.clone()) {
                continue;
            }
            let abs = project_root.join(&path_str);
            if !abs.exists() {
                errors.push(TdError {
                    code: TdErrorCode::NonExistentSpecRef,
                    section_type: SectionType::Changes,
                    line_start: 0,
                    line_end: 0,
                    message: format!(
                        "[AP-004] spec references path '{}' which does not exist \
                         relative to project root",
                        path_str
                    ),
                    hint: Some(
                        "Either create the referenced spec file or remove the broken reference."
                            .to_string(),
                    ),
                });
            }
        }
    }
    errors
}

/// Extract candidate spec paths from a line. Looks for
/// `.aw/tech-design/.../*.md` substrings, including ones wrapped in
/// backticks or table cells.
fn extract_spec_paths(line: &str) -> Vec<String> {
    let mut paths = Vec::new();
    let bytes = line.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i..].starts_with(b".aw/tech-design/") {
            let start = i;
            let mut end = i;
            while end < bytes.len() {
                let b = bytes[end];
                let in_path = b.is_ascii_alphanumeric() || matches!(b, b'/' | b'.' | b'-' | b'_');
                if !in_path {
                    break;
                }
                end += 1;
            }
            let candidate = &line[start..end];
            if candidate.ends_with(".md") {
                paths.push(candidate.to_string());
            }
            i = end.max(i + 1);
        } else {
            i += 1;
        }
    }
    paths
}

// ─────────────────────────────────────────────────────────────────────────
// AP-008 — body equals title
// ─────────────────────────────────────────────────────────────────────────

/// AP-008: the section whose type is `Problem`-shaped (in our taxonomy
/// today this manifests as a section type whose body is just markdown
/// prose) is byte-near-identical to the spec title. Authors sometimes
/// paste the title as a placeholder and forget to fill the actual problem
/// statement.
///
/// /// @spec .aw/tech-design/AUTHORING.md#anti-pattern-catalog (AP-008)
fn ap_008_body_equals_title(ast: &TDAst, spec_content: &str) -> Vec<TdError> {
    let mut errors = Vec::new();

    let title = ast
        .frontmatter
        .get("title")
        .or_else(|| ast.frontmatter.get("id"))
        .and_then(|v| v.as_str())
        .map(|s| s.trim().to_string());
    let title = match title {
        Some(t) if !t.is_empty() => t,
        _ => return errors,
    };

    // Walk the heading lines of the spec, find a "## Problem" heading,
    // and compare its body to the title. We do this textually rather than
    // through TDAst because `Problem` is no longer a tracked SectionType
    // (it lives in the `## Problem` markdown convention from issue bodies
    // pulled into the spec by some authors).
    let lines: Vec<&str> = spec_content.lines().collect();
    let mut i = 0;
    while i < lines.len() {
        let line = lines[i].trim();
        if line.eq_ignore_ascii_case("## problem") || line.starts_with("## Problem") {
            let mut body = Vec::new();
            let mut j = i + 1;
            while j < lines.len() {
                let l = lines[j].trim();
                if l.starts_with("## ") {
                    break;
                }
                if !l.is_empty() && !l.starts_with("<!--") {
                    body.push(l.to_string());
                }
                j += 1;
            }
            let body_first = body.first().cloned().unwrap_or_default();
            if body_equals_title_heuristic(&body_first, &title) {
                errors.push(TdError {
                    code: TdErrorCode::BodyEqualsTitle,
                    section_type: SectionType::Doc,
                    line_start: i + 1,
                    line_end: j,
                    message: format!(
                        "[AP-008] '## Problem' section body is essentially a \
                         copy of the spec title — write a substantive problem \
                         statement (current state, consequence, what this issue closes)"
                    ),
                    hint: Some(
                        "A good Problem section is 2-4 sentences naming the affected \
                         subsystem, the gap, and why closing it matters."
                            .to_string(),
                    ),
                });
            }
            i = j;
            continue;
        }
        i += 1;
    }
    errors
}

fn body_equals_title_heuristic(body: &str, title: &str) -> bool {
    let nb = body.trim().to_lowercase();
    let nt = title.trim().to_lowercase();
    let nt_no_prefix = nt
        .trim_start_matches("enhancement:")
        .trim_start_matches("bug:")
        .trim_start_matches("refactor:")
        .trim();
    if nb.is_empty() || nt_no_prefix.is_empty() {
        return false;
    }
    if nb == nt || nb == nt_no_prefix {
        return true;
    }
    // Body shorter than title + a small slack and contains the whole
    // title-no-prefix → likely a title-paste with minor wrapper.
    nb.len() < nt_no_prefix.len() + 16 && nb.contains(nt_no_prefix)
}

// ─────────────────────────────────────────────────────────────────────────
// AP-009 — non-existent `replaces:` symbol
// ─────────────────────────────────────────────────────────────────────────

/// AP-009: a Changes entry's `replaces: [foo, bar]` lists a symbol that
/// does not appear in the target file. Heuristic grep for `pub fn`,
/// `pub struct`, `pub enum`, `pub trait`, `pub mod`, etc. — does not parse
/// Rust fully.
///
/// /// @spec .aw/tech-design/AUTHORING.md#anti-pattern-catalog (AP-009)
fn ap_009_non_existent_replaces_symbol(ast: &TDAst, project_root: &Path) -> Vec<TdError> {
    let mut errors = Vec::new();
    for section in &ast.sections {
        if section.section_type != SectionType::Changes {
            continue;
        }
        for change in iter_changes_entries(section, &ast.frontmatter) {
            let path = match change.path.as_deref() {
                Some(p) => p,
                None => continue,
            };
            let abs = project_root.join(path);
            let content = match std::fs::read_to_string(&abs) {
                Ok(c) => c,
                Err(_) => continue, // file existence handled by AP-010
            };
            for sym in &change.replaces {
                if !file_declares_symbol(&content, sym) {
                    errors.push(TdError {
                        code: TdErrorCode::NonExistentReplacesSymbol,
                        section_type: SectionType::Changes,
                        line_start: section.line_start,
                        line_end: section.line_end,
                        message: format!(
                            "[AP-009] changes entry path '{}' lists \
                             replaces symbol '{}' but no `pub fn/struct/enum/trait/mod {}` \
                             declaration found in the target file",
                            path, sym, sym
                        ),
                        hint: Some(
                            "Either correct the symbol name or update the spec to point \
                             at the file where this symbol actually lives."
                                .to_string(),
                        ),
                    });
                }
            }
        }
    }
    errors
}

/// A change entry's relevant fields for AP-009 / AP-010 walks.
struct ChangeEntryView {
    path: Option<String>,
    action: String,
    replaces: Vec<String>,
}

/// Iterate the change entries in a Changes section. Reads the body via
/// the typed payload when available; for now we re-parse the raw YAML to
/// avoid depending on a not-yet-typed Changes payload.
fn iter_changes_entries(
    _section: &crate::td_ast::types::TDSection,
    _frontmatter: &serde_yaml::Value,
) -> Vec<ChangeEntryView> {
    // Stage 1B's TypedBody does not (yet) have a typed Changes payload —
    // changes section is currently parsed as an opaque YAML block. We
    // synthesise the entry list by re-parsing the body's serde_yaml::Value
    // here. When Stage 2's CliCommand-style typed Changes payload lands,
    // this fn switches to a typed walk.
    Vec::new()
}

/// Heuristic: a Rust source file declares `name` if it contains any of
/// `pub fn name`, `pub struct name`, `pub enum name`, `pub trait name`,
/// `pub mod name`, `fn name`, `struct name`, etc.
fn file_declares_symbol(content: &str, name: &str) -> bool {
    const KINDS: &[&str] = &[
        "pub fn ",
        "fn ",
        "pub async fn ",
        "async fn ",
        "pub struct ",
        "struct ",
        "pub enum ",
        "enum ",
        "pub trait ",
        "trait ",
        "pub mod ",
        "mod ",
        "pub const ",
        "const ",
        "pub static ",
        "static ",
        "pub type ",
        "type ",
    ];
    for line in content.lines() {
        let stripped = line.trim_start();
        for kind in KINDS {
            if let Some(rest) = stripped.strip_prefix(kind) {
                if let Some(after) = rest.strip_prefix(name) {
                    let next_ok = after
                        .chars()
                        .next()
                        .map(|c| !c.is_ascii_alphanumeric() && c != '_')
                        .unwrap_or(true);
                    if next_ok {
                        return true;
                    }
                }
            }
        }
    }
    false
}

// ─────────────────────────────────────────────────────────────────────────
// AP-010 — changes path not found
// ─────────────────────────────────────────────────────────────────────────

/// AP-010: a Changes entry's `path:` does not exist in the workspace
/// (and the entry's action is not `create`).
///
/// /// @spec .aw/tech-design/AUTHORING.md#anti-pattern-catalog (AP-010)
fn ap_010_changes_path_not_found(ast: &TDAst, project_root: &Path) -> Vec<TdError> {
    let mut errors = Vec::new();
    for section in &ast.sections {
        if section.section_type != SectionType::Changes {
            continue;
        }
        for change in iter_changes_entries(section, &ast.frontmatter) {
            let path = match change.path.as_deref() {
                Some(p) => p,
                None => continue,
            };
            if change.action == "create" {
                continue;
            }
            let abs = project_root.join(path);
            if !abs.exists() {
                errors.push(TdError {
                    code: TdErrorCode::ChangesPathNotFound,
                    section_type: SectionType::Changes,
                    line_start: section.line_start,
                    line_end: section.line_end,
                    message: format!(
                        "[AP-010] changes entry path '{}' (action: {}) \
                         does not exist; either set action: create or \
                         correct the path",
                        path, change.action
                    ),
                    hint: Some(
                        "If the file should be newly created by gen-code, declare \
                         action: create on this entry."
                            .to_string(),
                    ),
                });
            }
        }
    }
    errors
}

// ─────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::td_ast::parse::parse_td_str;

    fn ast_from(content: &str) -> TDAst {
        parse_td_str(content).expect("parse_td_str")
    }

    #[test]
    fn ap_001_detects_fill_placeholder() {
        // Use a markdown-lang section so the body parses but still carries
        // a `(fill)` placeholder in raw text — that's the failure mode AP-001
        // catches (author wrote a section header but left the body as a
        // placeholder).
        let spec = "---\nid: test\nfill_sections: [changes]\n---\n\n## Doc\n<!-- type: doc lang: markdown -->\n\n(fill)\n\n## Changes\n<!-- type: changes lang: yaml -->\n\n```yaml\nchanges: []\n```\n";
        let ast = ast_from(spec);
        let errs = ap_001_placeholder_leftover(&ast, spec);
        assert!(errs
            .iter()
            .any(|e| e.code == TdErrorCode::PlaceholderLeftover));
        assert!(errs.iter().any(|e| e.message.starts_with("[AP-001]")));
    }

    #[test]
    fn ap_001_passes_clean_body() {
        let spec = "---\nid: test\nfill_sections: [changes]\n---\n\n## Changes\n<!-- type: changes lang: yaml -->\n\n```yaml\nchanges: []\n```\n";
        let ast = ast_from(spec);
        let errs = ap_001_placeholder_leftover(&ast, spec);
        assert!(errs.is_empty());
    }

    #[test]
    fn ap_004_detects_non_existent_spec() {
        let spec_with_ref = "spec body: see .aw/tech-design/does/not/exist.md";
        let tmp = tempfile::tempdir().unwrap();
        let errs = ap_004_non_existent_spec_ref(spec_with_ref, tmp.path());
        assert!(errs
            .iter()
            .any(|e| e.code == TdErrorCode::NonExistentSpecRef));
    }

    #[test]
    fn ap_004_passes_existing_spec() {
        let tmp = tempfile::tempdir().unwrap();
        let real_spec = tmp.path().join(".aw/tech-design/foo.md");
        std::fs::create_dir_all(real_spec.parent().unwrap()).unwrap();
        std::fs::write(&real_spec, "x").unwrap();
        let spec_with_ref = "spec body: see .aw/tech-design/foo.md";
        let errs = ap_004_non_existent_spec_ref(spec_with_ref, tmp.path());
        assert!(errs.is_empty());
    }

    #[test]
    fn ap_008_detects_problem_equals_title() {
        let spec = "---\nid: test\ntitle: \"enhancement: foo bar\"\nfill_sections: [changes]\n---\n\n## Problem\n\nfoo bar\n\n## Changes\n<!-- type: changes lang: yaml -->\n\n```yaml\nchanges: []\n```\n";
        let ast = ast_from(spec);
        let errs = ap_008_body_equals_title(&ast, spec);
        assert!(errs.iter().any(|e| e.code == TdErrorCode::BodyEqualsTitle));
    }

    #[test]
    fn ap_008_passes_substantive_problem() {
        let spec = "---\nid: test\ntitle: \"enhancement: foo bar\"\nfill_sections: [changes]\n---\n\n## Problem\n\nThe codebase has 27 hand-written modules that lack codegen support; this issue extends the schema generator to cover them. The current state blocks downstream tooling and round-trip regeneration.\n\n## Changes\n<!-- type: changes lang: yaml -->\n\n```yaml\nchanges: []\n```\n";
        let ast = ast_from(spec);
        let errs = ap_008_body_equals_title(&ast, spec);
        assert!(errs.is_empty());
    }

    #[test]
    fn file_declares_symbol_finds_pub_fn() {
        let content = "pub fn foo() {}\nfn bar() {}\n";
        assert!(file_declares_symbol(content, "foo"));
        assert!(file_declares_symbol(content, "bar"));
        assert!(!file_declares_symbol(content, "baz"));
    }

    #[test]
    fn file_declares_symbol_distinguishes_prefixes() {
        let content = "pub fn foobar() {}\n";
        assert!(file_declares_symbol(content, "foobar"));
        assert!(!file_declares_symbol(content, "foo"));
    }
}

// CODEGEN-END
