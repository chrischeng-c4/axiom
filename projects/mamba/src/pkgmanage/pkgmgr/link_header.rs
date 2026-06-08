// link_header.rs — RFC 8288 `Link:` header parser.
//
// Some PyPI-style index mirrors paginate their PEP 691 JSON simple-API
// responses by attaching an HTTP Link header:
//
//     Link: <https://idx/.../?page=2>; rel="next",
//           <https://idx/.../?page=0>; rel="prev"
//
// uv walks `rel=next` to drain the full project listing; mamba's
// simple_api / json_api layers need the same primitive. RFC 8288 also
// shows up in GitHub-API style pagination and PEP 658 metadata
// pointers (`rel=metadata`), so this stays a generic parser, not a
// pagination-specific one.
//
// Grammar (RFC 8288 §3, with RFC 9110 §5.6 tokenization):
//   Link        = #link-value
//   link-value  = "<" URI-Reference ">" *( OWS ";" OWS link-param )
//   link-param  = token BWS "=" BWS ( token / quoted-string )
//   token       = 1*tchar  ; same as RFC 9110 token
//   tchar       = "!" / "#" / "$" / "%" / "&" / "'" / "*" / "+" / "-"
//               / "." / "^" / "_" / "`" / "|" / "~" / DIGIT / ALPHA

use std::collections::BTreeMap;

use crate::pkgmanage::pkgmgr::types::IndexError;

/// One `link-value`: the URI plus its parameters (rel, type, title,
/// hreflang, anchor, etc.).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LinkValue {
    /// The URI-Reference as written between the `<` and `>` (with
    /// surrounding whitespace trimmed).
    pub uri: String,
    /// Link parameters, lowercased on the key. Quoted-string values
    /// have their surrounding `"` stripped and `\` escapes processed.
    pub params: BTreeMap<String, String>,
}

impl LinkValue {
    /// Convenience accessor for the most-used parameter.
    pub fn rel(&self) -> Option<&str> {
        self.params.get("rel").map(String::as_str)
    }
}

/// Parse one `Link:` header value into its constituent link-values.
pub fn parse_link_header(header: &str) -> Result<Vec<LinkValue>, IndexError> {
    let s = header.trim();
    if s.is_empty() {
        return Err(pe("empty Link header"));
    }
    let mut out = Vec::new();
    let bytes = s.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        // Skip leading OWS and comma separators.
        while i < bytes.len() && (bytes[i] == b' ' || bytes[i] == b'\t' || bytes[i] == b',') {
            i += 1;
        }
        if i == bytes.len() {
            break;
        }
        if bytes[i] != b'<' {
            return Err(pe(&format!(
                "expected '<' at start of link-value at byte {i}, found {:?}",
                bytes[i] as char
            )));
        }
        i += 1;
        // Read URI-Reference up to `>`.
        let uri_start = i;
        while i < bytes.len() && bytes[i] != b'>' {
            i += 1;
        }
        if i == bytes.len() {
            return Err(pe("unterminated URI-Reference (missing '>')"));
        }
        let uri = s[uri_start..i].trim().to_string();
        if uri.is_empty() {
            return Err(pe("empty URI-Reference"));
        }
        i += 1; // consume '>'.

        let mut params: BTreeMap<String, String> = BTreeMap::new();

        // Read parameter list until next `,` or end-of-input.
        loop {
            // Skip OWS.
            while i < bytes.len() && (bytes[i] == b' ' || bytes[i] == b'\t') {
                i += 1;
            }
            if i == bytes.len() || bytes[i] == b',' {
                break;
            }
            if bytes[i] != b';' {
                return Err(pe(&format!(
                    "expected ';' or ',' after link-value at byte {i}, found {:?}",
                    bytes[i] as char
                )));
            }
            i += 1; // consume ';'.
            while i < bytes.len() && (bytes[i] == b' ' || bytes[i] == b'\t') {
                i += 1;
            }
            // Parse token (param name).
            let name_start = i;
            while i < bytes.len() && is_tchar(bytes[i]) {
                i += 1;
            }
            if i == name_start {
                return Err(pe(&format!("expected param-name token at byte {i}")));
            }
            let name = s[name_start..i].to_ascii_lowercase();
            while i < bytes.len() && (bytes[i] == b' ' || bytes[i] == b'\t') {
                i += 1;
            }
            // PEP 8288 allows bare-key params (no `=value`); RFC 9110
            // §5.6.6 also permits this in transfer-coding params. We
            // surface bare keys as `name -> ""` so callers can check
            // presence.
            if i == bytes.len() || bytes[i] == b';' || bytes[i] == b',' {
                params.entry(name).or_default();
                continue;
            }
            if bytes[i] != b'=' {
                return Err(pe(&format!(
                    "expected '=' after param-name at byte {i}, found {:?}",
                    bytes[i] as char
                )));
            }
            i += 1; // consume '='.
            while i < bytes.len() && (bytes[i] == b' ' || bytes[i] == b'\t') {
                i += 1;
            }
            // Read value: quoted-string or token.
            let value = if i < bytes.len() && bytes[i] == b'"' {
                read_quoted(s, &mut i)?
            } else {
                let v_start = i;
                while i < bytes.len() && is_tchar(bytes[i]) {
                    i += 1;
                }
                if i == v_start {
                    return Err(pe(&format!("expected param value at byte {i}")));
                }
                s[v_start..i].to_string()
            };
            // First occurrence wins, matching most HTTP-header
            // libraries (and what GitHub's API documentation models).
            params.entry(name).or_insert(value);
        }
        out.push(LinkValue { uri, params });
    }
    if out.is_empty() {
        return Err(pe("Link header contained no link-values"));
    }
    Ok(out)
}

/// Pick the first link with `rel="<wanted>"` from a parsed list. `rel`
/// is multi-valued in the RFC, space-separated; we match if any token
/// equals `wanted` case-insensitively.
pub fn find_rel<'a>(links: &'a [LinkValue], wanted: &str) -> Option<&'a LinkValue> {
    links.iter().find(|lv| {
        lv.rel()
            .is_some_and(|r| r.split_whitespace().any(|tok| tok.eq_ignore_ascii_case(wanted)))
    })
}

fn read_quoted(s: &str, i: &mut usize) -> Result<String, IndexError> {
    let bytes = s.as_bytes();
    if bytes[*i] != b'"' {
        return Err(pe("expected '\"' at start of quoted-string"));
    }
    *i += 1;
    let mut out = String::new();
    while *i < bytes.len() {
        match bytes[*i] {
            b'\\' => {
                *i += 1;
                if *i >= bytes.len() {
                    return Err(pe("dangling '\\' in quoted-string"));
                }
                out.push(bytes[*i] as char);
                *i += 1;
            }
            b'"' => {
                *i += 1;
                return Ok(out);
            }
            c => {
                out.push(c as char);
                *i += 1;
            }
        }
    }
    Err(pe("unterminated quoted-string"))
}

fn is_tchar(b: u8) -> bool {
    matches!(b,
        b'!' | b'#' | b'$' | b'%' | b'&' | b'\'' | b'*' | b'+' | b'-' | b'.'
        | b'^' | b'_' | b'`' | b'|' | b'~'
    ) || b.is_ascii_alphanumeric()
}

fn pe(msg: &str) -> IndexError {
    IndexError::ParseError {
        url: "Link".into(),
        detail: msg.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn lv(uri: &str, params: &[(&str, &str)]) -> LinkValue {
        LinkValue {
            uri: uri.to_string(),
            params: params
                .iter()
                .map(|(k, v)| ((*k).to_string(), (*v).to_string()))
                .collect(),
        }
    }

    #[test]
    fn parses_single_link_with_quoted_rel() {
        let h = "<https://idx/p?page=2>; rel=\"next\"";
        assert_eq!(
            parse_link_header(h).unwrap(),
            vec![lv("https://idx/p?page=2", &[("rel", "next")])]
        );
    }

    #[test]
    fn parses_single_link_with_token_rel() {
        // RFC 8288 says rel value is `relation-types`, which is a
        // token-style production — quotes are optional for safe
        // values.
        let h = "<https://idx/p?page=2>; rel=next";
        assert_eq!(
            parse_link_header(h).unwrap(),
            vec![lv("https://idx/p?page=2", &[("rel", "next")])]
        );
    }

    #[test]
    fn parses_multiple_links() {
        let h = "<https://idx/p?page=2>; rel=\"next\", <https://idx/p?page=0>; rel=\"prev\"";
        assert_eq!(
            parse_link_header(h).unwrap(),
            vec![
                lv("https://idx/p?page=2", &[("rel", "next")]),
                lv("https://idx/p?page=0", &[("rel", "prev")]),
            ]
        );
    }

    #[test]
    fn parses_multiple_params() {
        let h = "<https://idx/p>; rel=\"next\"; title=\"Page 2\"; type=\"application/json\"";
        let l = parse_link_header(h).unwrap();
        assert_eq!(l.len(), 1);
        assert_eq!(l[0].params.get("rel").unwrap(), "next");
        assert_eq!(l[0].params.get("title").unwrap(), "Page 2");
        assert_eq!(l[0].params.get("type").unwrap(), "application/json");
    }

    #[test]
    fn lowercases_param_keys() {
        let h = "<u>; REL=\"next\"; Title=\"x\"";
        let l = parse_link_header(h).unwrap();
        assert!(l[0].params.contains_key("rel"));
        assert!(l[0].params.contains_key("title"));
    }

    #[test]
    fn rel_helper_returns_first_rel() {
        let h = "<u>; rel=next";
        assert_eq!(parse_link_header(h).unwrap()[0].rel(), Some("next"));
    }

    #[test]
    fn rel_helper_returns_none_when_missing() {
        let h = "<u>; title=\"x\"";
        assert_eq!(parse_link_header(h).unwrap()[0].rel(), None);
    }

    #[test]
    fn find_rel_matches_case_insensitive() {
        let h = "<u1>; rel=PREV, <u2>; rel=NEXT";
        let l = parse_link_header(h).unwrap();
        assert_eq!(find_rel(&l, "next").unwrap().uri, "u2");
        assert_eq!(find_rel(&l, "prev").unwrap().uri, "u1");
    }

    #[test]
    fn find_rel_handles_multi_value_rel() {
        // RFC 8288 §3 — rel may be space-separated tokens.
        let h = "<u>; rel=\"first next\"";
        let l = parse_link_header(h).unwrap();
        assert_eq!(find_rel(&l, "next").unwrap().uri, "u");
        assert_eq!(find_rel(&l, "first").unwrap().uri, "u");
    }

    #[test]
    fn find_rel_returns_none_for_missing_rel() {
        let h = "<u>; rel=next";
        let l = parse_link_header(h).unwrap();
        assert!(find_rel(&l, "metadata").is_none());
    }

    #[test]
    fn tolerates_whitespace_around_punctuation() {
        let h = "  < https://idx/p >  ;  rel = \"next\"  ";
        let l = parse_link_header(h).unwrap();
        assert_eq!(l[0].uri, "https://idx/p");
        assert_eq!(l[0].rel(), Some("next"));
    }

    #[test]
    fn bare_param_recorded_as_empty_value() {
        // RFC 9110 §5.6.6 allows bare-key params.
        let h = "<u>; immutable";
        let l = parse_link_header(h).unwrap();
        assert_eq!(l[0].params.get("immutable").map(String::as_str), Some(""));
    }

    #[test]
    fn handles_escape_in_quoted_value() {
        let h = "<u>; title=\"with \\\"escape\\\"\"";
        let l = parse_link_header(h).unwrap();
        assert_eq!(l[0].params.get("title").unwrap(), r#"with "escape""#);
    }

    #[test]
    fn rejects_empty_header() {
        let err = parse_link_header("   ").unwrap_err();
        assert!(err.to_string().contains("empty"));
    }

    #[test]
    fn rejects_missing_angle_bracket() {
        let err = parse_link_header("https://idx; rel=next").unwrap_err();
        assert!(err.to_string().contains("'<'"));
    }

    #[test]
    fn rejects_unterminated_uri() {
        let err = parse_link_header("<https://idx; rel=next").unwrap_err();
        assert!(err.to_string().contains("unterminated"));
    }

    #[test]
    fn rejects_empty_uri() {
        let err = parse_link_header("<>; rel=next").unwrap_err();
        assert!(err.to_string().contains("empty URI"));
    }

    #[test]
    fn rejects_missing_param_name() {
        let err = parse_link_header("<u>; =val").unwrap_err();
        assert!(err.to_string().contains("param-name"));
    }

    #[test]
    fn rejects_missing_param_value_after_equals() {
        let err = parse_link_header("<u>; rel=").unwrap_err();
        assert!(err.to_string().contains("param value"));
    }

    #[test]
    fn rejects_unterminated_quoted_value() {
        let err = parse_link_header("<u>; rel=\"unterm").unwrap_err();
        assert!(err.to_string().contains("unterminated"));
    }

    #[test]
    fn pep691_metadata_link_shape() {
        // PEP 658 / 691 emit `rel=metadata` to point at the wheel's
        // METADATA twin without downloading the wheel.
        let h = "<https://files.pythonhosted.org/abc.metadata>; rel=\"metadata\"; type=\"text/plain\"";
        let l = parse_link_header(h).unwrap();
        assert_eq!(l[0].rel(), Some("metadata"));
        assert_eq!(l[0].uri, "https://files.pythonhosted.org/abc.metadata");
    }

    #[test]
    fn github_pagination_shape() {
        let h = "<https://api.github.com/x?page=2>; rel=\"next\", <https://api.github.com/x?page=10>; rel=\"last\"";
        let l = parse_link_header(h).unwrap();
        assert_eq!(l.len(), 2);
        assert_eq!(find_rel(&l, "next").unwrap().uri, "https://api.github.com/x?page=2");
        assert_eq!(find_rel(&l, "last").unwrap().uri, "https://api.github.com/x?page=10");
    }

    #[test]
    fn first_occurrence_of_duplicate_param_wins() {
        // Defensive: duplicate keys shouldn't crash. Keep first.
        let h = "<u>; rel=\"next\"; rel=\"prev\"";
        let l = parse_link_header(h).unwrap();
        assert_eq!(l[0].rel(), Some("next"));
    }
}
