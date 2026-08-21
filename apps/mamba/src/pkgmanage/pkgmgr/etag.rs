// etag.rs — HTTP ETag parser and comparator (RFC 9110 §8.8.3).
//
// The simple-API and JSON-API responses from a Python package index ship
// an `ETag` header; uv pairs that ETag with `If-None-Match` on its next
// conditional GET to get 304 Not Modified back, saving the body bytes.
// mamba's cache layer needs the same primitive.
//
// Grammar (RFC 9110 §8.8.3):
//
//     ETag       = entity-tag
//     entity-tag = [ weak ] opaque-tag
//     weak       = "W/"
//     opaque-tag = DQUOTE *etagc DQUOTE
//     etagc      = %x21 / %x23-7E / obs-text          ; VCHAR except `"`
//
// `If-None-Match` may carry a list of ETags or the special wildcard `*`.
//
// Strong vs weak comparison (§8.8.3.2):
//   * strong_compare — both ETags must be strong AND have identical
//     opaque values.
//   * weak_compare   — opaque values must be identical regardless of
//     strong/weak flag.

use crate::pkgmanage::pkgmgr::types::IndexError;

/// One parsed ETag.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ETag {
    /// The opaque tag body, with surrounding `"` stripped and `\` escapes
    /// processed. This is the value to compare across requests.
    pub value: String,
    /// True iff the ETag was emitted with the `W/` prefix.
    pub weak: bool,
}

/// One entry in an `If-None-Match` (or `If-Match`) list.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ETagListEntry {
    Star,
    Tag(ETag),
}

/// Parse a single `ETag:` header value.
pub fn parse_etag(header: &str) -> Result<ETag, IndexError> {
    let s = header.trim();
    if s.is_empty() {
        return Err(pe("empty ETag header"));
    }
    let (weak, rest) = if let Some(rest) = s.strip_prefix("W/") {
        (true, rest)
    } else {
        (false, s)
    };
    let value = read_quoted(rest)?;
    Ok(ETag { weak, value })
}

/// Parse a comma-separated `If-None-Match` (or `If-Match`) header value.
/// The special wildcard `*` is recognised as the sole legal non-tag
/// entry — it must appear alone, per RFC 9110 §13.1.
pub fn parse_etag_list(header: &str) -> Result<Vec<ETagListEntry>, IndexError> {
    let s = header.trim();
    if s.is_empty() {
        return Err(pe("empty If-None-Match header"));
    }
    if s == "*" {
        return Ok(vec![ETagListEntry::Star]);
    }
    let mut out = Vec::new();
    let mut chars = s.chars().peekable();
    loop {
        while matches!(chars.peek(), Some(c) if c.is_whitespace() || *c == ',') {
            chars.next();
        }
        if chars.peek().is_none() {
            break;
        }
        // Peek the next chunk: either `W/"…"` or `"…"`.
        let weak = matches!(chars.clone().take(2).collect::<String>().as_str(), "W/");
        if weak {
            chars.next();
            chars.next();
        }
        if chars.peek() != Some(&'"') {
            return Err(pe(&format!(
                "expected '\"' at start of opaque-tag, found {:?}",
                chars.peek()
            )));
        }
        chars.next();
        let mut value = String::new();
        loop {
            match chars.next() {
                Some('\\') => match chars.next() {
                    Some(c) => value.push(c),
                    None => return Err(pe("dangling '\\' in opaque-tag")),
                },
                Some('"') => break,
                Some(c) => value.push(c),
                None => return Err(pe("unterminated opaque-tag")),
            }
        }
        out.push(ETagListEntry::Tag(ETag { weak, value }));
    }
    // RFC 9110 §13.1: `*` must appear alone, never alongside other ETags.
    let star_count = out
        .iter()
        .filter(|e| matches!(e, ETagListEntry::Star))
        .count();
    if star_count > 0 && out.len() > 1 {
        return Err(pe("'*' must be the only entry in an ETag list"));
    }
    Ok(out)
}

/// Strong comparison: both ETags must be strong and have identical
/// opaque values (RFC 9110 §8.8.3.2). Strong comparison is required for
/// PATCH / range requests; conditional GET uses weak.
pub fn strong_compare(a: &ETag, b: &ETag) -> bool {
    !a.weak && !b.weak && a.value == b.value
}

/// Weak comparison: opaque values must match regardless of weak flag.
pub fn weak_compare(a: &ETag, b: &ETag) -> bool {
    a.value == b.value
}

fn read_quoted(s: &str) -> Result<String, IndexError> {
    let mut chars = s.chars();
    if chars.next() != Some('"') {
        return Err(pe(&format!(
            "expected '\"' at start of opaque-tag, found {s:?}"
        )));
    }
    let mut out = String::new();
    loop {
        match chars.next() {
            Some('\\') => match chars.next() {
                Some(c) => out.push(c),
                None => return Err(pe("dangling '\\' in opaque-tag")),
            },
            Some('"') => {
                let rest: String = chars.collect();
                if !rest.trim().is_empty() {
                    return Err(pe(&format!("trailing data after ETag: {rest:?}")));
                }
                return Ok(out);
            }
            Some(c) => out.push(c),
            None => return Err(pe("unterminated opaque-tag")),
        }
    }
}

fn pe(msg: &str) -> IndexError {
    IndexError::ParseError {
        url: "ETag".into(),
        detail: msg.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn strong(v: &str) -> ETag {
        ETag {
            value: v.into(),
            weak: false,
        }
    }

    fn weak(v: &str) -> ETag {
        ETag {
            value: v.into(),
            weak: true,
        }
    }

    #[test]
    fn parses_strong_etag() {
        assert_eq!(parse_etag("\"abc123\"").unwrap(), strong("abc123"));
    }

    #[test]
    fn parses_weak_etag() {
        assert_eq!(parse_etag("W/\"abc123\"").unwrap(), weak("abc123"));
    }

    #[test]
    fn parses_etag_with_whitespace() {
        assert_eq!(parse_etag("   \"v1\"   ").unwrap(), strong("v1"));
    }

    #[test]
    fn parses_etag_with_escape() {
        // Backslash-escaped `"` should make it into the value.
        let e = parse_etag(r#""quote: \"x\"""#).unwrap();
        assert_eq!(e.value, r#"quote: "x""#);
    }

    #[test]
    fn rejects_unterminated_etag() {
        let err = parse_etag("\"oops").unwrap_err();
        assert!(err.to_string().contains("unterminated"));
    }

    #[test]
    fn rejects_missing_quote() {
        let err = parse_etag("abc123").unwrap_err();
        assert!(err.to_string().contains("opaque-tag"));
    }

    #[test]
    fn rejects_empty() {
        let err = parse_etag("   ").unwrap_err();
        assert!(err.to_string().contains("empty"));
    }

    #[test]
    fn rejects_trailing_data() {
        let err = parse_etag("\"abc\" garbage").unwrap_err();
        assert!(err.to_string().contains("trailing data"));
    }

    #[test]
    fn parses_etag_list_single_tag() {
        let l = parse_etag_list("\"abc\"").unwrap();
        assert_eq!(l, vec![ETagListEntry::Tag(strong("abc"))]);
    }

    #[test]
    fn parses_etag_list_multi() {
        let l = parse_etag_list("\"a\", \"b\", W/\"c\"").unwrap();
        assert_eq!(
            l,
            vec![
                ETagListEntry::Tag(strong("a")),
                ETagListEntry::Tag(strong("b")),
                ETagListEntry::Tag(weak("c")),
            ]
        );
    }

    #[test]
    fn parses_etag_list_star() {
        let l = parse_etag_list("*").unwrap();
        assert_eq!(l, vec![ETagListEntry::Star]);
    }

    #[test]
    fn rejects_mixed_star_and_tags() {
        // Lists `"a", *` and `*, "b"` would parse "*" as the start of a
        // bad opaque-tag (no '"'). We surface the wildcard-vs-tag rule
        // via the parser shape; spell out the expected behavior here.
        let err = parse_etag_list("\"a\", *").unwrap_err();
        assert!(err.to_string().contains("opaque-tag"));
    }

    #[test]
    fn rejects_empty_list() {
        let err = parse_etag_list("   ").unwrap_err();
        assert!(err.to_string().contains("empty"));
    }

    #[test]
    fn strong_compare_both_strong_match() {
        assert!(strong_compare(&strong("x"), &strong("x")));
    }

    #[test]
    fn strong_compare_mismatch_value() {
        assert!(!strong_compare(&strong("x"), &strong("y")));
    }

    #[test]
    fn strong_compare_one_weak_fails() {
        assert!(!strong_compare(&strong("x"), &weak("x")));
        assert!(!strong_compare(&weak("x"), &strong("x")));
    }

    #[test]
    fn weak_compare_ignores_weak_flag() {
        assert!(weak_compare(&strong("x"), &weak("x")));
        assert!(weak_compare(&weak("x"), &weak("x")));
        assert!(weak_compare(&strong("x"), &strong("x")));
    }

    #[test]
    fn weak_compare_mismatch_value_still_fails() {
        assert!(!weak_compare(&strong("x"), &weak("y")));
    }
}
