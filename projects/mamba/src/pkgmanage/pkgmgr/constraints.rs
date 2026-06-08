// Constraints + requirements file parser (Tick 33).
//
// pip and uv both accept `-c constraints.txt`: a list of version pins
// that constrain the resolver *without* adding the packages to the
// install set. The file format is mostly the same as requirements.txt
// (PEP 440 specifiers, PEP 508 markers, comments, continuations,
// recursive `-c file` / `-r file` directives), so we share a parser.
//
// What's covered:
//   * One requirement per logical line, with `\`-continuations folded.
//   * `# comment` from any '#' to end-of-line, except inside an
//     environment-marker string literal (uv preserves '#' in markers).
//   * `-c FILE` and `-r FILE` recursive directives. We surface them as
//     `ParsedLine::Include { kind, path }` and leave actual file I/O
//     to the caller (so the resolver can decide cycle detection,
//     relative-path resolution, etc).
//   * Empty lines.
//
// What's NOT covered (intentionally):
//   * Full PEP 508 parsing of the requirement body. We keep the raw
//     requirement string and let downstream `pep508` / `markers` do the
//     hard work — keeps this module tightly scoped.
//   * Network-driven options (`--index-url`, `--extra-index-url`, etc).
//     They round-trip as `ParsedLine::Option { name, value }` so the
//     caller can act on them; we don't validate.

use crate::pkgmanage::pkgmgr::types::IndexError;

/// One logical (after continuations + comments) entry from the file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParsedLine {
    /// A requirement / constraint string like `flask>=2.0; python_version >= "3.11"`.
    Requirement {
        /// Original requirement text with the comment stripped but
        /// otherwise verbatim. Downstream parses this into name +
        /// specifier + marker.
        text: String,
    },
    /// A `-c FILE` or `-r FILE` directive.
    Include {
        kind: IncludeKind,
        path: String,
    },
    /// A long-form pip option like `--index-url URL`. Verbatim.
    Option {
        name: String,
        value: Option<String>,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IncludeKind {
    /// `-c FILE` — pull in constraints.
    Constraint,
    /// `-r FILE` — pull in requirements.
    Requirement,
}

/// Parse a constraints or requirements file body. Caller has already
/// read the bytes; we work on the `&str` so we stay testable without
/// touching the filesystem.
///
/// Order is preserved. Empty logical lines are skipped.
pub fn parse_file(src: &str) -> Result<Vec<ParsedLine>, IndexError> {
    let mut out = Vec::new();
    // Fold continuations: a line ending in `\` joins to the next.
    let logical_lines = fold_continuations(src);
    for raw in logical_lines {
        let trimmed = strip_comment(&raw);
        let trimmed = trimmed.trim();
        if trimmed.is_empty() {
            continue;
        }
        if let Some(rest) = trimmed.strip_prefix("-c ") {
            out.push(ParsedLine::Include {
                kind: IncludeKind::Constraint,
                path: rest.trim().to_string(),
            });
            continue;
        }
        if let Some(rest) = trimmed.strip_prefix("--constraint ") {
            out.push(ParsedLine::Include {
                kind: IncludeKind::Constraint,
                path: rest.trim().to_string(),
            });
            continue;
        }
        if let Some(rest) = trimmed.strip_prefix("-r ") {
            out.push(ParsedLine::Include {
                kind: IncludeKind::Requirement,
                path: rest.trim().to_string(),
            });
            continue;
        }
        if let Some(rest) = trimmed.strip_prefix("--requirement ") {
            out.push(ParsedLine::Include {
                kind: IncludeKind::Requirement,
                path: rest.trim().to_string(),
            });
            continue;
        }
        if trimmed.starts_with("--") {
            // Long-form option. Split on first '=' or first space.
            if let Some(option) = parse_long_option(trimmed)? {
                out.push(ParsedLine::Option {
                    name: option.0,
                    value: option.1,
                });
                continue;
            }
        }
        out.push(ParsedLine::Requirement {
            text: trimmed.to_string(),
        });
    }
    Ok(out)
}

fn parse_long_option(line: &str) -> Result<Option<(String, Option<String>)>, IndexError> {
    let body = line.trim_start_matches('-');
    // `--name=value` form.
    if let Some(eq) = body.find('=') {
        let name = body[..eq].to_string();
        let value = body[eq + 1..].trim().to_string();
        if name.is_empty() {
            return Err(IndexError::ParseError {
                url: "<requirements file>".into(),
                detail: format!("malformed option {line:?}"),
            });
        }
        return Ok(Some((name, Some(value))));
    }
    // `--name value` form (space-separated). pip-compatible.
    let mut parts = body.splitn(2, char::is_whitespace);
    let name = parts.next().unwrap_or("").to_string();
    let value = parts.next().map(|s| s.trim().to_string());
    if name.is_empty() {
        return Err(IndexError::ParseError {
            url: "<requirements file>".into(),
            detail: format!("malformed option {line:?}"),
        });
    }
    Ok(Some((name, value)))
}

/// Fold `\`-continued lines into single logical lines. A backslash at
/// end-of-line continues the next line; the backslash and newline are
/// dropped. Trailing whitespace before the backslash is preserved.
fn fold_continuations(src: &str) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    let mut acc = String::new();
    let mut continued = false;
    for line in src.lines() {
        if continued {
            acc.push_str(line);
        } else {
            acc = line.to_string();
        }
        // Strip trailing CR (CRLF on Windows is handled by `lines()`,
        // but bare CR can sneak in via weird editors).
        while acc.ends_with('\r') {
            acc.pop();
        }
        if acc.ends_with('\\') {
            acc.pop();
            continued = true;
        } else {
            out.push(std::mem::take(&mut acc));
            continued = false;
        }
    }
    if continued || !acc.is_empty() {
        out.push(acc);
    }
    out
}

/// Strip a `# comment` from end of line. We preserve '#' inside double-
/// quoted strings so PEP 508 markers like `; sys_platform == "win32" #
/// note` correctly split — anything before the LAST unquoted '#' wins.
fn strip_comment(line: &str) -> String {
    let mut out = String::new();
    let mut in_quote: Option<char> = None;
    for c in line.chars() {
        match in_quote {
            Some(q) => {
                out.push(c);
                if c == q {
                    in_quote = None;
                }
            }
            None => {
                if c == '"' || c == '\'' {
                    in_quote = Some(c);
                    out.push(c);
                } else if c == '#' {
                    break;
                } else {
                    out.push(c);
                }
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn req(text: &str) -> ParsedLine {
        ParsedLine::Requirement {
            text: text.to_string(),
        }
    }
    fn inc_c(p: &str) -> ParsedLine {
        ParsedLine::Include {
            kind: IncludeKind::Constraint,
            path: p.to_string(),
        }
    }
    fn inc_r(p: &str) -> ParsedLine {
        ParsedLine::Include {
            kind: IncludeKind::Requirement,
            path: p.to_string(),
        }
    }
    fn opt(name: &str, value: Option<&str>) -> ParsedLine {
        ParsedLine::Option {
            name: name.to_string(),
            value: value.map(|s| s.to_string()),
        }
    }

    #[test]
    fn parse_empty_file_yields_no_lines() {
        let out = parse_file("").unwrap();
        assert!(out.is_empty());
    }

    #[test]
    fn parse_blank_and_comment_only_lines_are_skipped() {
        let out = parse_file("\n\n   \n# only a comment\n  # indented\n").unwrap();
        assert!(out.is_empty());
    }

    #[test]
    fn parse_requirement_lines() {
        let out = parse_file("flask>=2.0\nrequests==2.31.0\n").unwrap();
        assert_eq!(out, vec![req("flask>=2.0"), req("requests==2.31.0")]);
    }

    #[test]
    fn parse_strips_trailing_comments_outside_strings() {
        let out = parse_file("flask>=2.0  # the web framework\n").unwrap();
        assert_eq!(out, vec![req("flask>=2.0")]);
    }

    #[test]
    fn parse_preserves_hash_inside_quoted_marker() {
        let out = parse_file(
            "flask>=2.0; sys_platform == \"linux # not a comment\"\n",
        )
        .unwrap();
        assert_eq!(
            out,
            vec![req(
                "flask>=2.0; sys_platform == \"linux # not a comment\""
            )]
        );
    }

    #[test]
    fn parse_folds_backslash_continuations() {
        let out = parse_file("flask>=2.0; \\\n  python_version >= \"3.11\"\n").unwrap();
        assert_eq!(
            out,
            vec![req("flask>=2.0;   python_version >= \"3.11\"")]
        );
    }

    #[test]
    fn parse_constraint_include_dash_c() {
        let out = parse_file("-c constraints/prod.txt\n").unwrap();
        assert_eq!(out, vec![inc_c("constraints/prod.txt")]);
    }

    #[test]
    fn parse_constraint_include_long_form() {
        let out = parse_file("--constraint constraints/prod.txt\n").unwrap();
        assert_eq!(out, vec![inc_c("constraints/prod.txt")]);
    }

    #[test]
    fn parse_requirement_include() {
        let out = parse_file("-r dev-reqs.txt\n--requirement other.txt\n").unwrap();
        assert_eq!(out, vec![inc_r("dev-reqs.txt"), inc_r("other.txt")]);
    }

    #[test]
    fn parse_index_url_option_equals_form() {
        let out = parse_file("--index-url=https://pypi.org/simple\n").unwrap();
        assert_eq!(
            out,
            vec![opt("index-url", Some("https://pypi.org/simple"))]
        );
    }

    #[test]
    fn parse_index_url_option_space_form() {
        let out = parse_file("--index-url https://pypi.org/simple\n").unwrap();
        assert_eq!(
            out,
            vec![opt("index-url", Some("https://pypi.org/simple"))]
        );
    }

    #[test]
    fn parse_value_less_option() {
        let out = parse_file("--pre\n").unwrap();
        assert_eq!(out, vec![opt("pre", None)]);
    }

    #[test]
    fn parse_mixed_block() {
        let src = "\
# top-level header
-c constraints/prod.txt
flask>=2.0  # pinned for SSE
requests==2.31.0; python_version >= \"3.11\"
--index-url https://internal.example/simple
-r dev-reqs.txt
";
        let out = parse_file(src).unwrap();
        assert_eq!(
            out,
            vec![
                inc_c("constraints/prod.txt"),
                req("flask>=2.0"),
                req("requests==2.31.0; python_version >= \"3.11\""),
                opt("index-url", Some("https://internal.example/simple")),
                inc_r("dev-reqs.txt"),
            ]
        );
    }

    #[test]
    fn parse_handles_crlf_line_endings() {
        let out = parse_file("flask>=2.0\r\nrequests==2.31\r\n").unwrap();
        assert_eq!(out, vec![req("flask>=2.0"), req("requests==2.31")]);
    }

    #[test]
    fn parse_continuation_at_end_of_file_does_not_panic() {
        // Trailing backslash with no following line — should still
        // emit whatever accumulated, not panic.
        let out = parse_file("flask>=2.0 \\\n").unwrap();
        assert_eq!(out.len(), 1);
    }
}
