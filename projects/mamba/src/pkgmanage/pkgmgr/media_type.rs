// media_type.rs — RFC 9110 §8.3 media-type and §12.5.1 Accept parsers.
//
// PEP 691 simple-API content negotiation requires both:
//
//   * Reading `Content-Type:` on responses to dispatch between the
//     legacy PEP 503 HTML parser and the PEP 691 JSON parser.
//   * Writing / parsing `Accept:` on requests with q-values so a mirror
//     can choose the response shape it prefers.
//
// Grammar (RFC 9110 §8.3 + §12.5.1 + RFC 9110 §5.6 tokens):
//
//   media-type = type "/" subtype *( OWS ";" OWS parameter )
//   type       = token
//   subtype    = token
//   parameter  = parameter-name BWS "=" BWS parameter-value
//   parameter-value = token / quoted-string
//   Accept     = #( media-range [ accept-params ] )
//   media-range = ( "*/*" / type "/*" / type "/" subtype )
//                 *( OWS ";" OWS parameter )
//   accept-params = ";" OWS "q=" qvalue *( OWS ";" OWS accept-ext )
//
// `type` and `subtype` are lower-cased on parse (RFC 9110 §8.3.1 says
// they're case-insensitive); parameter NAMES are lower-cased;
// parameter VALUES are preserved verbatim (the RFC reserves case
// significance for value strings).

use std::collections::BTreeMap;

use crate::pkgmanage::pkgmgr::types::IndexError;

/// One parsed media-type. Used for both `Content-Type` (single value)
/// and as one element of an `Accept` list.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MediaType {
    pub type_: String,
    pub subtype: String,
    pub params: BTreeMap<String, String>,
}

impl MediaType {
    /// Convenience constructor for tests and producers.
    pub fn new(type_: &str, subtype: &str) -> Self {
        Self {
            type_: type_.to_ascii_lowercase(),
            subtype: subtype.to_ascii_lowercase(),
            params: BTreeMap::new(),
        }
    }

    /// `charset=` parameter, lower-cased for ASCII charsets. Returns
    /// `None` if absent.
    pub fn charset(&self) -> Option<&str> {
        self.params.get("charset").map(String::as_str)
    }

    /// Accept q-value (§12.4.2). Defaults to 1.0 when absent.
    /// Out-of-range / malformed values default to 1.0 too — matching
    /// uv / curl's permissive behavior; callers wanting strict
    /// validation can read params["q"] directly.
    pub fn q(&self) -> f32 {
        match self.params.get("q") {
            Some(v) => v.parse::<f32>().unwrap_or(1.0).clamp(0.0, 1.0),
            None => 1.0,
        }
    }

    /// True iff this media-type matches `other` exactly OR via
    /// wildcards (`*/*`, `type/*`). Used by simple-API negotiation to
    /// decide whether a server response satisfies the Accept clause
    /// the client sent.
    pub fn matches(&self, other: &MediaType) -> bool {
        let type_ok = self.type_ == "*" || other.type_ == "*" || self.type_ == other.type_;
        let sub_ok = self.subtype == "*" || other.subtype == "*" || self.subtype == other.subtype;
        type_ok && sub_ok
    }
}

/// Parse a `Content-Type:` (or any single media-type) header value.
pub fn parse_media_type(header: &str) -> Result<MediaType, IndexError> {
    let s = header.trim();
    if s.is_empty() {
        return Err(pe("empty media-type"));
    }
    parse_one(s)
}

/// Parse an `Accept:` header value (comma-separated media-ranges).
/// Returns one `MediaType` per element. Empty inputs are rejected.
pub fn parse_accept(header: &str) -> Result<Vec<MediaType>, IndexError> {
    let s = header.trim();
    if s.is_empty() {
        return Err(pe("empty Accept header"));
    }
    let mut out = Vec::new();
    for elem in split_top_level_commas(s) {
        let elem = elem.trim();
        if elem.is_empty() {
            // Tolerate trailing commas — RFC 9110 §5.6.1 #rule.
            continue;
        }
        out.push(parse_one(elem)?);
    }
    if out.is_empty() {
        return Err(pe("Accept header contained no media-ranges"));
    }
    Ok(out)
}

/// Split on `,` but skip commas inside quoted-strings — Accept
/// parameter values can legally carry commas.
fn split_top_level_commas(s: &str) -> Vec<&str> {
    let bytes = s.as_bytes();
    let mut out = Vec::new();
    let mut start = 0;
    let mut in_quote = false;
    let mut i = 0;
    while i < bytes.len() {
        match bytes[i] {
            b'"' => {
                in_quote = !in_quote;
                i += 1;
            }
            b'\\' if in_quote => {
                // Skip the escaped char so an escaped `"` doesn't flip
                // us out of quote-mode.
                i += 2.min(bytes.len() - i);
            }
            b',' if !in_quote => {
                out.push(&s[start..i]);
                start = i + 1;
                i += 1;
            }
            _ => i += 1,
        }
    }
    out.push(&s[start..]);
    out
}

fn parse_one(s: &str) -> Result<MediaType, IndexError> {
    let bytes = s.as_bytes();
    // Type segment.
    let mut i = 0;
    while i < bytes.len() && is_tchar(bytes[i]) {
        i += 1;
    }
    if i == 0 || i == bytes.len() || bytes[i] != b'/' {
        return Err(pe(&format!(
            "expected 'type/subtype' in media-type, got {s:?}"
        )));
    }
    let type_ = s[..i].to_ascii_lowercase();
    i += 1; // consume '/'.
    let subtype_start = i;
    while i < bytes.len() && is_tchar(bytes[i]) {
        i += 1;
    }
    if i == subtype_start {
        return Err(pe(&format!("empty subtype in media-type {s:?}")));
    }
    let subtype = s[subtype_start..i].to_ascii_lowercase();
    let mut params: BTreeMap<String, String> = BTreeMap::new();
    while i < bytes.len() {
        // OWS.
        while i < bytes.len() && (bytes[i] == b' ' || bytes[i] == b'\t') {
            i += 1;
        }
        if i == bytes.len() {
            break;
        }
        if bytes[i] != b';' {
            return Err(pe(&format!(
                "expected ';' between parameters in {s:?}, found {:?}",
                bytes[i] as char
            )));
        }
        i += 1; // consume ';'.
        while i < bytes.len() && (bytes[i] == b' ' || bytes[i] == b'\t') {
            i += 1;
        }
        let name_start = i;
        while i < bytes.len() && is_tchar(bytes[i]) {
            i += 1;
        }
        if i == name_start {
            return Err(pe(&format!("expected parameter name in {s:?}")));
        }
        let name = s[name_start..i].to_ascii_lowercase();
        while i < bytes.len() && (bytes[i] == b' ' || bytes[i] == b'\t') {
            i += 1;
        }
        if i == bytes.len() || bytes[i] != b'=' {
            return Err(pe(&format!(
                "expected '=' after parameter name {name:?} in {s:?}"
            )));
        }
        i += 1; // consume '='.
        while i < bytes.len() && (bytes[i] == b' ' || bytes[i] == b'\t') {
            i += 1;
        }
        let value = if i < bytes.len() && bytes[i] == b'"' {
            read_quoted(s, &mut i)?
        } else {
            let v_start = i;
            while i < bytes.len() && is_tchar(bytes[i]) {
                i += 1;
            }
            if i == v_start {
                return Err(pe(&format!("expected parameter value in {s:?}")));
            }
            s[v_start..i].to_string()
        };
        // First occurrence wins for duplicates (matches most HTTP
        // libraries).
        params.entry(name).or_insert(value);
    }
    Ok(MediaType {
        type_,
        subtype,
        params,
    })
}

fn read_quoted(s: &str, i: &mut usize) -> Result<String, IndexError> {
    let bytes = s.as_bytes();
    // Caller already verified bytes[*i] == b'"'.
    *i += 1;
    let mut out = String::new();
    while *i < bytes.len() {
        match bytes[*i] {
            b'\\' => {
                *i += 1;
                if *i >= bytes.len() {
                    return Err(pe("dangling '\\' in quoted parameter value"));
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
    Err(pe("unterminated quoted parameter value"))
}

fn is_tchar(b: u8) -> bool {
    matches!(
        b,
        b'!' | b'#'
            | b'$'
            | b'%'
            | b'&'
            | b'\''
            | b'*'
            | b'+'
            | b'-'
            | b'.'
            | b'^'
            | b'_'
            | b'`'
            | b'|'
            | b'~'
    ) || b.is_ascii_alphanumeric()
}

fn pe(msg: &str) -> IndexError {
    IndexError::ParseError {
        url: "Content-Type".into(),
        detail: msg.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mt_with(type_: &str, sub: &str, params: &[(&str, &str)]) -> MediaType {
        let mut m = MediaType::new(type_, sub);
        for (k, v) in params {
            m.params.insert((*k).to_string(), (*v).to_string());
        }
        m
    }

    #[test]
    fn parses_simple_content_type() {
        assert_eq!(
            parse_media_type("text/html").unwrap(),
            mt_with("text", "html", &[])
        );
    }

    #[test]
    fn parses_pep691_json_content_type() {
        // PEP 691 official MIME type — the whole reason this module
        // exists. Mirrors emit this on JSON simple-API responses.
        let m = parse_media_type("application/vnd.pypi.simple.v1+json").unwrap();
        assert_eq!(m.type_, "application");
        assert_eq!(m.subtype, "vnd.pypi.simple.v1+json");
    }

    #[test]
    fn parses_with_charset_parameter() {
        let m = parse_media_type("text/html; charset=UTF-8").unwrap();
        assert_eq!(m.charset(), Some("UTF-8"));
    }

    #[test]
    fn lowercases_type_subtype_and_param_names() {
        let m = parse_media_type("APPLICATION/JSON; Charset=UTF-8").unwrap();
        assert_eq!(m.type_, "application");
        assert_eq!(m.subtype, "json");
        // Param NAME lowered, VALUE preserved.
        assert_eq!(m.params.get("charset").map(String::as_str), Some("UTF-8"));
    }

    #[test]
    fn parses_quoted_parameter_value() {
        let m = parse_media_type("text/html; boundary=\"---a;b---\"").unwrap();
        assert_eq!(
            m.params.get("boundary").map(String::as_str),
            Some("---a;b---")
        );
    }

    #[test]
    fn parses_quoted_value_with_escape() {
        let m = parse_media_type("text/x; q=\"a\\\"b\"").unwrap();
        assert_eq!(m.params.get("q").map(String::as_str), Some(r#"a"b"#));
    }

    #[test]
    fn tolerates_whitespace_around_parameters() {
        let m = parse_media_type("text/html  ;  charset = utf-8  ").unwrap();
        assert_eq!(m.charset(), Some("utf-8"));
    }

    #[test]
    fn rejects_missing_slash() {
        let err = parse_media_type("text").unwrap_err();
        assert!(err.to_string().contains("type/subtype"));
    }

    #[test]
    fn rejects_empty_subtype() {
        let err = parse_media_type("text/").unwrap_err();
        assert!(err.to_string().contains("empty subtype"));
    }

    #[test]
    fn rejects_empty_input() {
        let err = parse_media_type("   ").unwrap_err();
        assert!(err.to_string().contains("empty"));
    }

    #[test]
    fn rejects_missing_parameter_value() {
        let err = parse_media_type("text/html; charset").unwrap_err();
        assert!(err.to_string().contains("expected '='"));
    }

    #[test]
    fn rejects_unterminated_quoted_value() {
        let err = parse_media_type("text/html; q=\"unterm").unwrap_err();
        assert!(err.to_string().contains("unterminated"));
    }

    #[test]
    fn parses_accept_list_with_q_values() {
        let l = parse_accept("text/html, application/json;q=0.9, */*;q=0.1").unwrap();
        assert_eq!(l.len(), 3);
        assert_eq!(l[0].q(), 1.0);
        assert_eq!(l[1].subtype, "json");
        assert!((l[1].q() - 0.9).abs() < 1e-6);
        assert!((l[2].q() - 0.1).abs() < 1e-6);
        // Wildcard.
        assert_eq!(l[2].type_, "*");
        assert_eq!(l[2].subtype, "*");
    }

    #[test]
    fn accept_default_q_is_one() {
        let l = parse_accept("text/html").unwrap();
        assert_eq!(l[0].q(), 1.0);
    }

    #[test]
    fn accept_clamps_q_out_of_range() {
        // RFC says 0.000–1.000; we clamp rather than error so a
        // misbehaving server doesn't break negotiation.
        let l = parse_accept("text/html;q=2.0").unwrap();
        assert_eq!(l[0].q(), 1.0);
        let l = parse_accept("text/html;q=-0.5").unwrap();
        assert_eq!(l[0].q(), 0.0);
    }

    #[test]
    fn accept_malformed_q_defaults_to_one() {
        let l = parse_accept("text/html;q=banana").unwrap();
        assert_eq!(l[0].q(), 1.0);
    }

    #[test]
    fn accept_tolerates_trailing_comma() {
        let l = parse_accept("text/html, application/json,").unwrap();
        assert_eq!(l.len(), 2);
    }

    #[test]
    fn accept_with_quoted_value_keeps_internal_commas() {
        // RFC 9110 §5.6.1: commas inside quoted-string don't split.
        let l = parse_accept("text/x; n=\"a,b\", text/y").unwrap();
        assert_eq!(l.len(), 2);
        assert_eq!(l[0].params.get("n").map(String::as_str), Some("a,b"));
    }

    #[test]
    fn matches_exact() {
        let a = MediaType::new("application", "json");
        let b = MediaType::new("application", "json");
        assert!(a.matches(&b));
    }

    #[test]
    fn matches_subtype_wildcard() {
        let a = MediaType::new("application", "*");
        let b = MediaType::new("application", "json");
        assert!(a.matches(&b));
        assert!(b.matches(&a));
    }

    #[test]
    fn matches_full_wildcard() {
        let star = MediaType::new("*", "*");
        let m = MediaType::new("text", "html");
        assert!(star.matches(&m));
        assert!(m.matches(&star));
    }

    #[test]
    fn does_not_match_different_types() {
        let a = MediaType::new("application", "json");
        let b = MediaType::new("text", "html");
        assert!(!a.matches(&b));
    }

    #[test]
    fn pep691_negotiation_round_trip() {
        // The real use-case: parse the Accept the client sent, then
        // verify the response's Content-Type satisfies one of them.
        let accept = parse_accept(
            "application/vnd.pypi.simple.v1+json, application/vnd.pypi.simple.v1+html;q=0.5, text/html;q=0.1",
        )
        .unwrap();
        let resp = parse_media_type("application/vnd.pypi.simple.v1+json; charset=utf-8").unwrap();
        assert!(accept.iter().any(|a| a.matches(&resp)));
    }

    #[test]
    fn accept_rejects_empty_input() {
        let err = parse_accept("   ").unwrap_err();
        assert!(err.to_string().contains("empty"));
    }
}
