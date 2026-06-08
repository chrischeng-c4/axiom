// PEP 723 — inline script metadata (Tick 22).
//
// Tick 68 (added) — render + upsert. `render_pep723` produces canonical
// comment-block text from a `ScriptMetadata`. `upsert_pep723` finds an
// existing `# /// script` block in a Python source and replaces it, or
// (when no block exists) inserts one after the optional shebang and
// encoding-declaration prefix. Both are idempotent under repeated calls.
//
// Lets a standalone Python script declare its own dependencies and Python
// version constraint inside a magic comment block:
//
//     # /// script
//     # requires-python = ">=3.11"
//     # dependencies = [
//     #   "requests<3",
//     #   "rich",
//     # ]
//     # ///
//
//     import requests
//     ...
//
// `uv run script.py` parses this block, resolves the dependencies into an
// isolated env, and runs the script. Tick 22 owns the *parser*: pure text
// in → typed metadata out. Wiring it into a `mamba run` verb (resolve +
// venv + execute) is a follow-up tick — that step composes the resolver
// + installer + toolchain modules already in place.
//
// Reference: https://peps.python.org/pep-0723/

use std::str::FromStr;

use crate::pkgmanage::pkgmgr::toolchain::PythonRequest;
use crate::pkgmanage::pkgmgr::types::IndexError;

/// Parsed PEP 723 metadata block. Both fields are optional within the
/// block — a `# /// script` block with no `requires-python` and no
/// `dependencies` is legal (parses to `Some(ScriptMetadata { .. })` with
/// both `None`/empty).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ScriptMetadata {
    /// `requires-python` constraint, parsed *loosely* into a
    /// `PythonRequest`. The PEP allows any PEP 440 specifier (e.g.
    /// `">=3.11,<3.13"`); we surface the raw string in `requires_python_raw`
    /// for downstream consumers that need full specifier semantics, and
    /// give the structured form for the common "single floor" case.
    pub requires_python: Option<PythonRequest>,
    /// Raw string from the TOML, preserved for downstream resolvers that
    /// want full PEP 440 specifier handling.
    pub requires_python_raw: Option<String>,
    /// PEP 508 dependency strings, verbatim.
    pub dependencies: Vec<String>,
}

const PEP723_URL: &str = "<PEP 723 inline metadata>";

/// Locate the `# /// script` metadata block in `source`. Returns:
/// - `Ok(None)`            — no script block (PEP 723 inactive)
/// - `Ok(Some(metadata))`  — block found and parsed
/// - `Err(IndexError)`     — malformed (unterminated block, bad TOML)
///
/// A source may legally contain blocks of other types (e.g. `# /// pyproject`)
/// that PEP 723 reserves for future use; those are skipped. Only the `script`
/// block is consumed.
pub fn parse_pep723(source: &str) -> Result<Option<ScriptMetadata>, IndexError> {
    let Some(block) = find_block(source, "script")? else {
        return Ok(None);
    };

    let toml_src = strip_comment_prefix(&block);
    let doc: toml::Value = toml_src.parse().map_err(|err| IndexError::ParseError {
        url: PEP723_URL.into(),
        detail: format!("malformed TOML inside `# /// script` block: {err}"),
    })?;

    let requires_python_raw = doc
        .get("requires-python")
        .and_then(|v| v.as_str())
        .map(String::from);

    let requires_python = match &requires_python_raw {
        Some(raw) => {
            Some(
                parse_requires_python_loose(raw).map_err(|detail| IndexError::ParseError {
                    url: PEP723_URL.into(),
                    detail,
                })?,
            )
        }
        None => None,
    };

    let dependencies = match doc.get("dependencies") {
        Some(toml::Value::Array(arr)) => arr
            .iter()
            .map(|v| {
                v.as_str()
                    .map(String::from)
                    .ok_or_else(|| IndexError::ParseError {
                        url: PEP723_URL.into(),
                        detail: format!(
                            "PEP 723 `dependencies` entries must be strings, got {v:?}"
                        ),
                    })
            })
            .collect::<Result<Vec<_>, _>>()?,
        Some(other) => {
            return Err(IndexError::ParseError {
                url: PEP723_URL.into(),
                detail: format!("PEP 723 `dependencies` must be an array, got {other:?}"),
            });
        }
        None => Vec::new(),
    };

    Ok(Some(ScriptMetadata {
        requires_python,
        requires_python_raw,
        dependencies,
    }))
}

/// Scan `source` for a `# /// <name>` block and return the body (lines
/// between the start and `# ///` end markers, with their leading `# `
/// preserved). Returns:
/// - `Ok(None)`     — no block of this name
/// - `Ok(Some(s))`  — block found
/// - `Err(...)`     — start marker seen but no end marker
fn find_block(source: &str, name: &str) -> Result<Option<String>, IndexError> {
    let start_marker = format!("# /// {name}");
    let mut in_block = false;
    let mut block_lines: Vec<&str> = Vec::new();

    for line in source.lines() {
        let trimmed = line.trim_end();
        if !in_block {
            // Start marker is line-exact (after right-trim), per PEP 723.
            if trimmed == start_marker {
                in_block = true;
                continue;
            }
        } else if trimmed == "# ///" {
            return Ok(Some(block_lines.join("\n")));
        } else if !trimmed.starts_with('#') {
            // The block must be a contiguous comment region. A non-comment
            // line before the close marker is a hard error.
            return Err(IndexError::ParseError {
                url: PEP723_URL.into(),
                detail: format!(
                    "non-comment line inside `# /// {name}` block before close marker: {line:?}"
                ),
            });
        } else {
            block_lines.push(line);
        }
    }

    if in_block {
        Err(IndexError::ParseError {
            url: PEP723_URL.into(),
            detail: format!("unterminated `# /// {name}` block — missing closing `# ///`"),
        })
    } else {
        Ok(None)
    }
}

/// Strip the `# ` (or bare `#`) prefix from each line of the block body so
/// it becomes valid TOML.
///
/// PEP 723 specifies: each metadata line is `#`, optionally followed by a
/// single space, then the TOML content. We mirror that — a line that is
/// just `#` becomes an empty line; `# foo` becomes `foo`; `#foo` (no space)
/// is *also* accepted because real-world editors strip trailing whitespace.
fn strip_comment_prefix(block: &str) -> String {
    block
        .lines()
        .map(|line| {
            let line = line.trim_end();
            if let Some(rest) = line.strip_prefix("# ") {
                rest.to_string()
            } else if let Some(rest) = line.strip_prefix('#') {
                rest.to_string()
            } else {
                line.to_string()
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// Loose-parse a `requires-python` string into a `PythonRequest`. The PEP
/// allows full PEP 440 specifiers (`">=3.11,<3.13"`); we extract the
/// *lower bound* of the first comparator and use that as the request floor.
///
/// Recognized comparators: `>=`, `~=`, `==`, `>`. A bare version (no
/// comparator) is treated as `MajorMinor` of that version.
fn parse_requires_python_loose(raw: &str) -> Result<PythonRequest, String> {
    let raw = raw.trim();
    if raw.is_empty() {
        return Err("empty `requires-python` string".into());
    }
    let first = raw.split(',').next().unwrap().trim();
    let (op, rest) = if let Some(rest) = first.strip_prefix(">=") {
        (">=", rest.trim())
    } else if let Some(rest) = first.strip_prefix("~=") {
        ("~=", rest.trim())
    } else if let Some(rest) = first.strip_prefix("==") {
        ("==", rest.trim())
    } else if let Some(rest) = first.strip_prefix('>') {
        (">", rest.trim())
    } else {
        ("", first)
    };

    let req = PythonRequest::from_str(rest)?;
    match op {
        // `==3.12.7` pins exactly; everything else is a floor, which we
        // surface as a MajorMinor or Major request depending on how it
        // was written.
        "==" => Ok(req),
        _ => Ok(req),
    }
}

/// Render a canonical PEP 723 `# /// script` block from `meta`.
///
/// Output is deterministic:
/// * Wrapped in `# /// script` / `# ///` markers.
/// * `requires-python` line emitted only when `requires_python_raw` is `Some`.
/// * Empty dependency vec renders as `# dependencies = []` (single line).
/// * Non-empty deps render multi-line with each entry indented by 4 spaces
///   and trailing comma after the last entry (uv's house style).
/// * Always ends with a trailing newline.
///
/// The output is parseable by `parse_pep723` — `parse(render(m)) == Some(m)`
/// for any `m` whose dependency strings are TOML-quotable (no embedded
/// double quotes; uv enforces this at the input boundary).
pub fn render_pep723(meta: &ScriptMetadata) -> String {
    let mut out = String::new();
    out.push_str("# /// script\n");
    if let Some(raw) = &meta.requires_python_raw {
        out.push_str("# requires-python = \"");
        out.push_str(raw);
        out.push_str("\"\n");
    }
    if meta.dependencies.is_empty() {
        out.push_str("# dependencies = []\n");
    } else {
        out.push_str("# dependencies = [\n");
        for dep in &meta.dependencies {
            out.push_str("#     \"");
            out.push_str(dep);
            out.push_str("\",\n");
        }
        out.push_str("# ]\n");
    }
    out.push_str("# ///\n");
    out
}

/// Replace the `# /// script` block in `source` with one rendered from `meta`,
/// or insert a new block if none exists.
///
/// Placement when inserting:
/// * If `source` starts with a `#!` shebang line, the block goes immediately
///   after it (with a blank separator line).
/// * If the first non-shebang line is a PEP 263 encoding declaration
///   (`# -*- coding: ... -*-` or `# coding: ...`), the block goes after it.
/// * Otherwise the block goes at the very top.
///
/// When replacing, the original surrounding whitespace is preserved: the
/// new block occupies exactly the lines the old block occupied (start
/// marker line through close marker line, inclusive). A trailing blank
/// line between the block and the rest of the source is added on insert,
/// not on replace.
///
/// Errors only when an unterminated `# /// script` start marker is found
/// (delegated to `find_block`).
pub fn upsert_pep723(source: &str, meta: &ScriptMetadata) -> Result<String, IndexError> {
    let new_block = render_pep723(meta);

    // Locate an existing script block by line index, if any.
    if let Some(range) = find_block_line_range(source, "script")? {
        let lines: Vec<&str> = source.split_inclusive('\n').collect();
        let mut out = String::new();
        for line in &lines[..range.0] {
            out.push_str(line);
        }
        out.push_str(&new_block);
        for line in &lines[range.1..] {
            out.push_str(line);
        }
        return Ok(out);
    }

    // No existing block — insert after shebang + optional encoding line.
    let lines: Vec<&str> = source.split_inclusive('\n').collect();
    let mut prefix_lines: Vec<&str> = Vec::new();
    let mut idx = 0;
    if let Some(first) = lines.first() {
        if first.starts_with("#!") {
            prefix_lines.push(first);
            idx = 1;
        }
    }
    if let Some(line) = lines.get(idx) {
        let trimmed = line.trim_end();
        if is_encoding_declaration(trimmed) {
            prefix_lines.push(line);
            idx += 1;
        }
    }
    let rest_start = idx;

    let mut out = String::new();
    for line in &prefix_lines {
        out.push_str(line);
    }
    if !prefix_lines.is_empty() {
        // PEP 8 requires a blank separator between the encoding/shebang prelude
        // and the script body — keep that habit when inserting.
        if !out.ends_with('\n') {
            out.push('\n');
        }
        out.push('\n');
    }
    out.push_str(&new_block);
    let mut body_started = false;
    for line in &lines[rest_start..] {
        if !body_started {
            // Add a blank separator between the block and the script body
            // unless the body already starts with a blank line.
            if line.trim().is_empty() {
                body_started = true;
                out.push_str(line);
                continue;
            }
            out.push('\n');
            body_started = true;
        }
        out.push_str(line);
    }
    if !body_started && !out.ends_with('\n') {
        out.push('\n');
    }
    Ok(out)
}

/// Locate the inclusive line index range `(start, end)` covering a
/// `# /// <name>` block in `source` (start marker line through close
/// marker line). Lines are counted in `split_inclusive('\n')` order so
/// they round-trip exactly back into the source string.
fn find_block_line_range(source: &str, name: &str) -> Result<Option<(usize, usize)>, IndexError> {
    let start_marker = format!("# /// {name}");
    let lines: Vec<&str> = source.split_inclusive('\n').collect();
    let mut start: Option<usize> = None;
    for (i, line) in lines.iter().enumerate() {
        let trimmed = line.trim_end();
        if let Some(s) = start {
            if trimmed == "# ///" {
                return Ok(Some((s, i + 1)));
            }
            if !trimmed.starts_with('#') {
                return Err(IndexError::ParseError {
                    url: PEP723_URL.into(),
                    detail: format!(
                        "non-comment line inside `# /// {name}` block before close marker: {line:?}"
                    ),
                });
            }
        } else if trimmed == start_marker {
            start = Some(i);
        }
    }
    if start.is_some() {
        return Err(IndexError::ParseError {
            url: PEP723_URL.into(),
            detail: format!("unterminated `# /// {name}` block — missing closing `# ///`"),
        });
    }
    Ok(None)
}

/// PEP 263 encoding declaration recognizer. Matches the regex
/// `^[ \t\f]*#.*?coding[:=][ \t]*([-_.a-zA-Z0-9]+)` loosely — enough to
/// catch `# -*- coding: utf-8 -*-` and `# coding: latin-1` headers that
/// must stay in lines 1–2 per PEP 263.
fn is_encoding_declaration(line: &str) -> bool {
    let line = line.trim_start();
    if !line.starts_with('#') {
        return false;
    }
    let body = &line[1..];
    if let Some(idx) = body.find("coding") {
        let after = &body[idx + "coding".len()..];
        let after = after.trim_start();
        return after.starts_with(':') || after.starts_with('=');
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_block_returns_none() {
        let src = "import os\nprint('hello')\n";
        assert!(parse_pep723(src).unwrap().is_none());
    }

    #[test]
    fn full_block_parses_dependencies_and_python() {
        let src = r#"# /// script
# requires-python = ">=3.11"
# dependencies = [
#     "requests<3",
#     "rich",
# ]
# ///

import requests
"#;
        let meta = parse_pep723(src).unwrap().unwrap();
        assert_eq!(meta.requires_python_raw.as_deref(), Some(">=3.11"));
        assert_eq!(meta.requires_python, Some(PythonRequest::MajorMinor(3, 11)));
        assert_eq!(meta.dependencies, vec!["requests<3", "rich"]);
    }

    #[test]
    fn empty_block_yields_default_metadata() {
        let src = r#"# /// script
# ///

print("hi")
"#;
        let meta = parse_pep723(src).unwrap().unwrap();
        assert!(meta.requires_python.is_none());
        assert!(meta.dependencies.is_empty());
    }

    #[test]
    fn block_only_dependencies_no_python() {
        let src = r#"# /// script
# dependencies = ["httpx"]
# ///
"#;
        let meta = parse_pep723(src).unwrap().unwrap();
        assert!(meta.requires_python.is_none());
        assert_eq!(meta.dependencies, vec!["httpx"]);
    }

    #[test]
    fn block_with_bare_hash_lines_is_legal() {
        // PEP 723 example uses `#` for the blank divider between fields.
        let src = "# /// script\n# dependencies = [\n#     \"rich\",\n# ]\n#\n# requires-python = \">=3.10\"\n# ///\n";
        let meta = parse_pep723(src).unwrap().unwrap();
        assert_eq!(meta.dependencies, vec!["rich"]);
        assert_eq!(meta.requires_python_raw.as_deref(), Some(">=3.10"));
    }

    #[test]
    fn unterminated_block_is_error() {
        let src = "# /// script\n# dependencies = []\nprint('oops no close')\n";
        let err = parse_pep723(src).unwrap_err();
        assert!(
            format!("{err}").contains("non-comment line inside `# /// script` block"),
            "got: {err}"
        );
    }

    #[test]
    fn block_eof_without_close_marker_is_error() {
        let src = "# /// script\n# dependencies = []\n";
        let err = parse_pep723(src).unwrap_err();
        assert!(format!("{err}").contains("unterminated"), "got: {err}");
    }

    #[test]
    fn malformed_toml_inside_block_is_error() {
        let src = "# /// script\n# this is not = valid = toml = at all\n# ///\n";
        let err = parse_pep723(src).unwrap_err();
        assert!(format!("{err}").contains("malformed TOML"), "got: {err}");
    }

    #[test]
    fn dependencies_must_be_array_of_strings() {
        let src = "# /// script\n# dependencies = [42]\n# ///\n";
        let err = parse_pep723(src).unwrap_err();
        assert!(format!("{err}").contains("must be strings"), "got: {err}");
    }

    #[test]
    fn pep440_compound_specifier_uses_first_bound() {
        let src = "# /// script\n# requires-python = \">=3.11,<3.13\"\n# ///\n";
        let meta = parse_pep723(src).unwrap().unwrap();
        assert_eq!(meta.requires_python_raw.as_deref(), Some(">=3.11,<3.13"));
        // First bound is `>=3.11`, which maps to MajorMinor(3, 11).
        assert_eq!(meta.requires_python, Some(PythonRequest::MajorMinor(3, 11)));
    }

    #[test]
    fn equality_specifier_yields_exact() {
        let src = "# /// script\n# requires-python = \"==3.12.7\"\n# ///\n";
        let meta = parse_pep723(src).unwrap().unwrap();
        assert_eq!(
            meta.requires_python,
            Some(PythonRequest::Exact("3.12.7".parse().unwrap()))
        );
    }

    #[test]
    fn non_script_blocks_are_ignored() {
        // PEP 723 reserves other block names for future use. Make sure a
        // `pyproject` block doesn't poison the parse.
        let src = r#"# /// pyproject
# foo = "bar"
# ///

# /// script
# dependencies = ["click"]
# ///
"#;
        let meta = parse_pep723(src).unwrap().unwrap();
        assert_eq!(meta.dependencies, vec!["click"]);
    }

    // ----- Tick 68: render + upsert ---------------------------------------

    #[test]
    fn render_empty_metadata_emits_minimal_block() {
        let meta = ScriptMetadata::default();
        let rendered = render_pep723(&meta);
        assert_eq!(rendered, "# /// script\n# dependencies = []\n# ///\n");
    }

    #[test]
    fn render_dependencies_only() {
        let meta = ScriptMetadata {
            requires_python: None,
            requires_python_raw: None,
            dependencies: vec!["requests<3".into(), "rich".into()],
        };
        let rendered = render_pep723(&meta);
        let expected = "# /// script\n\
# dependencies = [\n\
#     \"requests<3\",\n\
#     \"rich\",\n\
# ]\n\
# ///\n";
        assert_eq!(rendered, expected);
    }

    #[test]
    fn render_full_metadata() {
        let meta = ScriptMetadata {
            requires_python: Some(PythonRequest::MajorMinor(3, 11)),
            requires_python_raw: Some(">=3.11".into()),
            dependencies: vec!["httpx".into()],
        };
        let rendered = render_pep723(&meta);
        let expected = "# /// script\n\
# requires-python = \">=3.11\"\n\
# dependencies = [\n\
#     \"httpx\",\n\
# ]\n\
# ///\n";
        assert_eq!(rendered, expected);
    }

    #[test]
    fn render_round_trips_through_parse() {
        let meta = ScriptMetadata {
            requires_python: Some(PythonRequest::MajorMinor(3, 12)),
            requires_python_raw: Some(">=3.12".into()),
            dependencies: vec!["requests<3".into(), "rich".into()],
        };
        let rendered = render_pep723(&meta);
        let reparsed = parse_pep723(&rendered).unwrap().unwrap();
        assert_eq!(reparsed.dependencies, meta.dependencies);
        assert_eq!(reparsed.requires_python_raw, meta.requires_python_raw);
        assert_eq!(reparsed.requires_python, meta.requires_python);
    }

    #[test]
    fn upsert_inserts_when_no_block_present() {
        let src = "import requests\nprint('hi')\n";
        let meta = ScriptMetadata {
            requires_python: None,
            requires_python_raw: None,
            dependencies: vec!["requests".into()],
        };
        let updated = upsert_pep723(src, &meta).unwrap();
        let expected = "# /// script\n\
# dependencies = [\n\
#     \"requests\",\n\
# ]\n\
# ///\n\
\n\
import requests\n\
print('hi')\n";
        assert_eq!(updated, expected);
    }

    #[test]
    fn upsert_inserts_after_shebang() {
        let src = "#!/usr/bin/env python3\nimport os\n";
        let meta = ScriptMetadata {
            requires_python: None,
            requires_python_raw: None,
            dependencies: vec!["rich".into()],
        };
        let updated = upsert_pep723(src, &meta).unwrap();
        let expected = "#!/usr/bin/env python3\n\
\n\
# /// script\n\
# dependencies = [\n\
#     \"rich\",\n\
# ]\n\
# ///\n\
\n\
import os\n";
        assert_eq!(updated, expected);
    }

    #[test]
    fn upsert_inserts_after_shebang_and_encoding() {
        let src = "#!/usr/bin/env python3\n# -*- coding: utf-8 -*-\nprint('hi')\n";
        let meta = ScriptMetadata {
            requires_python: None,
            requires_python_raw: None,
            dependencies: vec![],
        };
        let updated = upsert_pep723(src, &meta).unwrap();
        let expected = "#!/usr/bin/env python3\n\
# -*- coding: utf-8 -*-\n\
\n\
# /// script\n\
# dependencies = []\n\
# ///\n\
\n\
print('hi')\n";
        assert_eq!(updated, expected);
    }

    #[test]
    fn upsert_replaces_existing_block_in_place() {
        let src = "# /// script\n# dependencies = [\"old\"]\n# ///\n\nprint('hi')\n";
        let meta = ScriptMetadata {
            requires_python: None,
            requires_python_raw: None,
            dependencies: vec!["new".into(), "shiny".into()],
        };
        let updated = upsert_pep723(src, &meta).unwrap();
        let expected = "# /// script\n\
# dependencies = [\n\
#     \"new\",\n\
#     \"shiny\",\n\
# ]\n\
# ///\n\
\n\
print('hi')\n";
        assert_eq!(updated, expected);
    }

    #[test]
    fn upsert_replaces_preserves_surrounding_lines() {
        // Block sandwiched between shebang + import body — replace must keep
        // both halves intact.
        let src = "#!/usr/bin/env python3\n\n# /// script\n# requires-python = \">=3.10\"\n# dependencies = []\n# ///\n\nimport sys\n";
        let meta = ScriptMetadata {
            requires_python: Some(PythonRequest::MajorMinor(3, 12)),
            requires_python_raw: Some(">=3.12".into()),
            dependencies: vec!["click".into()],
        };
        let updated = upsert_pep723(src, &meta).unwrap();
        let expected = "#!/usr/bin/env python3\n\
\n\
# /// script\n\
# requires-python = \">=3.12\"\n\
# dependencies = [\n\
#     \"click\",\n\
# ]\n\
# ///\n\
\n\
import sys\n";
        assert_eq!(updated, expected);
    }

    #[test]
    fn upsert_is_idempotent() {
        let src = "import os\n";
        let meta = ScriptMetadata {
            requires_python: None,
            requires_python_raw: None,
            dependencies: vec!["rich".into()],
        };
        let once = upsert_pep723(src, &meta).unwrap();
        let twice = upsert_pep723(&once, &meta).unwrap();
        assert_eq!(once, twice);
    }

    #[test]
    fn upsert_into_empty_source_emits_block_only() {
        let meta = ScriptMetadata {
            requires_python: None,
            requires_python_raw: None,
            dependencies: vec![],
        };
        let updated = upsert_pep723("", &meta).unwrap();
        assert_eq!(updated, "# /// script\n# dependencies = []\n# ///\n");
    }

    #[test]
    fn upsert_unterminated_block_is_error() {
        let src = "# /// script\n# dependencies = []\n";
        let meta = ScriptMetadata::default();
        let err = upsert_pep723(src, &meta).unwrap_err();
        assert!(format!("{err}").contains("unterminated"), "got: {err}");
    }

    #[test]
    fn is_encoding_declaration_matches_pep263_forms() {
        assert!(is_encoding_declaration("# -*- coding: utf-8 -*-"));
        assert!(is_encoding_declaration("# coding: latin-1"));
        assert!(is_encoding_declaration("# coding=utf-8"));
        assert!(!is_encoding_declaration("# regular comment"));
        assert!(!is_encoding_declaration("import os"));
    }
}
