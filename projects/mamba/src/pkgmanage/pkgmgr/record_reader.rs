// PEP 376 RECORD reader (Tick 57).
//
// `record_writer` already emits PEP 376-shaped CSV bodies; the
// installer's verify path has a forgiving parser, but it doesn't
// handle RFC 4180 quoting (paths with commas, embedded quotes, or
// CRLF inside fields). This module is the round-trip inverse of
// `record_writer::render_record`:
//
//   * Accepts CSV bodies emitted by `render_record` directly,
//   * Tolerates `installer/record.rs`'s plain comma-split output,
//   * Decodes quoted fields per RFC 4180 (paired `""` → `"`).
//
// Returned shape is `RecordEntryDraft` — the same type the writer
// consumes — so a body can be parsed, mutated, and re-rendered without
// type conversion churn. Pure-data: no I/O.

use crate::pkgmanage::pkgmgr::record_writer::RecordEntryDraft;
use crate::pkgmanage::pkgmgr::types::IndexError;

/// Parse a RECORD CSV body into typed entries. Blank lines and lines
/// that consist of nothing but whitespace are skipped (matching pip).
pub fn parse_record(text: &str) -> Result<Vec<RecordEntryDraft>, IndexError> {
    let mut out = Vec::new();
    for (lineno, raw) in physical_lines(text).enumerate() {
        let line = raw.trim_end_matches('\r');
        if line.trim().is_empty() {
            continue;
        }
        let fields = split_csv_row(line, lineno + 1)?;
        if fields.is_empty() {
            continue;
        }
        if fields.len() > 3 {
            return Err(IndexError::ParseError {
                url: "<RECORD>".into(),
                detail: format!(
                    "RECORD line {}: expected at most 3 fields, got {}",
                    lineno + 1,
                    fields.len()
                ),
            });
        }
        let path = fields[0].clone();
        if path.is_empty() {
            return Err(IndexError::ParseError {
                url: "<RECORD>".into(),
                detail: format!("RECORD line {} missing path field", lineno + 1),
            });
        }
        let hash_field = fields.get(1).map(|s| s.as_str()).unwrap_or("");
        let size_field = fields.get(2).map(|s| s.as_str()).unwrap_or("");

        let sha256_b64url = if hash_field.is_empty() {
            None
        } else if let Some(rest) = hash_field.strip_prefix("sha256=") {
            Some(rest.to_string())
        } else {
            return Err(IndexError::ParseError {
                url: "<RECORD>".into(),
                detail: format!(
                    "RECORD line {}: unsupported hash algorithm in {:?}, only sha256= accepted",
                    lineno + 1,
                    hash_field
                ),
            });
        };

        let size = if size_field.is_empty() {
            None
        } else {
            Some(size_field.parse::<u64>().map_err(|_| IndexError::ParseError {
                url: "<RECORD>".into(),
                detail: format!(
                    "RECORD line {}: invalid size {:?}",
                    lineno + 1,
                    size_field
                ),
            })?)
        };

        out.push(RecordEntryDraft {
            path,
            sha256_b64url,
            size,
        });
    }
    Ok(out)
}

/// Iterate the body as a sequence of physical RECORD rows. A row ends
/// at a newline that is *not* inside a double-quoted field. Embedded
/// quotes are doubled (`""`), per RFC 4180.
fn physical_lines(text: &str) -> impl Iterator<Item = String> + '_ {
    let mut chars = text.chars().peekable();
    std::iter::from_fn(move || {
        let mut buf = String::new();
        let mut in_quotes = false;
        let mut seen_any = false;
        while let Some(&c) = chars.peek() {
            seen_any = true;
            chars.next();
            if c == '"' {
                in_quotes = !in_quotes;
                buf.push(c);
                continue;
            }
            if c == '\n' && !in_quotes {
                return Some(buf);
            }
            buf.push(c);
        }
        if seen_any {
            Some(buf)
        } else {
            None
        }
    })
}

/// RFC 4180 CSV row split. Fields are comma-separated; a field may be
/// wrapped in double quotes to allow embedded commas, quotes (doubled
/// as `""`), CR, or LF.
fn split_csv_row(line: &str, lineno: usize) -> Result<Vec<String>, IndexError> {
    let mut fields = Vec::new();
    let mut cur = String::new();
    let mut in_quotes = false;
    let mut chars = line.chars().peekable();
    while let Some(c) = chars.next() {
        if in_quotes {
            if c == '"' {
                if chars.peek() == Some(&'"') {
                    chars.next();
                    cur.push('"');
                } else {
                    in_quotes = false;
                }
            } else {
                cur.push(c);
            }
        } else if c == ',' {
            fields.push(std::mem::take(&mut cur));
        } else if c == '"' {
            if !cur.is_empty() {
                return Err(IndexError::ParseError {
                    url: "<RECORD>".into(),
                    detail: format!(
                        "RECORD line {}: stray text before quoted field",
                        lineno
                    ),
                });
            }
            in_quotes = true;
        } else {
            cur.push(c);
        }
    }
    if in_quotes {
        return Err(IndexError::ParseError {
            url: "<RECORD>".into(),
            detail: format!("RECORD line {}: unterminated quoted field", lineno),
        });
    }
    fields.push(cur);
    Ok(fields)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pkgmanage::pkgmgr::record_writer::render_record;

    #[test]
    fn parses_plain_rows() {
        let body = "pkg/a.py,sha256=AAAA,123\npkg/b.py,sha256=BBBB,456\n";
        let got = parse_record(body).unwrap();
        assert_eq!(got.len(), 2);
        assert_eq!(got[0].path, "pkg/a.py");
        assert_eq!(got[0].sha256_b64url.as_deref(), Some("AAAA"));
        assert_eq!(got[0].size, Some(123));
        assert_eq!(got[1].size, Some(456));
    }

    #[test]
    fn empty_hash_and_size_are_none() {
        let body = "pkg-1.0.dist-info/RECORD,,\n";
        let got = parse_record(body).unwrap();
        assert_eq!(got.len(), 1);
        assert!(got[0].sha256_b64url.is_none());
        assert!(got[0].size.is_none());
    }

    #[test]
    fn skips_blank_lines() {
        let body = "\n  \npkg/a.py,sha256=AA,1\n\n";
        let got = parse_record(body).unwrap();
        assert_eq!(got.len(), 1);
    }

    #[test]
    fn handles_crlf_line_endings() {
        let body = "pkg/a.py,sha256=AA,1\r\npkg/b.py,sha256=BB,2\r\n";
        let got = parse_record(body).unwrap();
        assert_eq!(got.len(), 2);
        assert_eq!(got[1].path, "pkg/b.py");
    }

    #[test]
    fn parses_quoted_path_with_embedded_comma() {
        let body = "\"pkg/odd,name.py\",sha256=CC,9\n";
        let got = parse_record(body).unwrap();
        assert_eq!(got.len(), 1);
        assert_eq!(got[0].path, "pkg/odd,name.py");
        assert_eq!(got[0].size, Some(9));
    }

    #[test]
    fn parses_quoted_path_with_embedded_quote() {
        let body = "\"pkg/with\"\"quote.py\",sha256=DD,3\n";
        let got = parse_record(body).unwrap();
        assert_eq!(got.len(), 1);
        assert_eq!(got[0].path, "pkg/with\"quote.py");
    }

    #[test]
    fn parses_quoted_path_with_embedded_newline() {
        let body = "\"pkg/multi\nline.py\",sha256=EE,5\n";
        let got = parse_record(body).unwrap();
        assert_eq!(got.len(), 1);
        assert_eq!(got[0].path, "pkg/multi\nline.py");
    }

    #[test]
    fn rejects_unsupported_hash_algorithm() {
        let body = "pkg/a.py,md5=AAA,1\n";
        let err = parse_record(body).unwrap_err();
        assert!(matches!(err, IndexError::ParseError { .. }));
    }

    #[test]
    fn rejects_invalid_size() {
        let body = "pkg/a.py,sha256=AA,not_a_number\n";
        let err = parse_record(body).unwrap_err();
        assert!(matches!(err, IndexError::ParseError { .. }));
    }

    #[test]
    fn rejects_too_many_fields() {
        let body = "pkg/a.py,sha256=AA,1,extra\n";
        let err = parse_record(body).unwrap_err();
        assert!(matches!(err, IndexError::ParseError { .. }));
    }

    #[test]
    fn rejects_empty_path() {
        let body = ",sha256=AA,1\n";
        let err = parse_record(body).unwrap_err();
        assert!(matches!(err, IndexError::ParseError { .. }));
    }

    #[test]
    fn rejects_unterminated_quote() {
        let body = "\"open/quote.py,sha256=AA,1\n";
        let err = parse_record(body).unwrap_err();
        assert!(matches!(err, IndexError::ParseError { .. }));
    }

    #[test]
    fn rejects_text_before_quoted_field() {
        let body = "abc\"def\",sha256=AA,1\n";
        let err = parse_record(body).unwrap_err();
        assert!(matches!(err, IndexError::ParseError { .. }));
    }

    #[test]
    fn round_trips_writer_output() {
        let drafts = vec![
            RecordEntryDraft {
                path: "pkg/a.py".into(),
                sha256_b64url: Some("aaaa".into()),
                size: Some(10),
            },
            RecordEntryDraft {
                path: "pkg/b.py".into(),
                sha256_b64url: Some("bbbb".into()),
                size: Some(20),
            },
        ];
        let body = render_record(&drafts, "pkg-1.0.dist-info/RECORD").unwrap();
        let parsed = parse_record(&body).unwrap();
        assert_eq!(parsed.len(), 3); // 2 source + RECORD self-row
        assert!(parsed.iter().any(|e| e.path == "pkg-1.0.dist-info/RECORD"));
        let self_row = parsed
            .iter()
            .find(|e| e.path == "pkg-1.0.dist-info/RECORD")
            .unwrap();
        assert!(self_row.sha256_b64url.is_none());
        assert!(self_row.size.is_none());
    }

    #[test]
    fn round_trips_quoted_paths_through_writer() {
        let drafts = vec![RecordEntryDraft {
            path: "weird,name.py".into(),
            sha256_b64url: Some("ZZ".into()),
            size: Some(7),
        }];
        let body = render_record(&drafts, "pkg-1.0.dist-info/RECORD").unwrap();
        let parsed = parse_record(&body).unwrap();
        let entry = parsed.iter().find(|e| e.path == "weird,name.py").unwrap();
        assert_eq!(entry.size, Some(7));
    }

    #[test]
    fn empty_body_returns_empty_vec() {
        let got = parse_record("").unwrap();
        assert!(got.is_empty());
    }

    #[test]
    fn pyc_with_empty_hash_and_size_round_trips() {
        let body = "pkg/__pycache__/a.cpython-312.pyc,,\n";
        let got = parse_record(body).unwrap();
        assert_eq!(got.len(), 1);
        assert!(got[0].sha256_b64url.is_none());
        assert!(got[0].size.is_none());
    }
}
