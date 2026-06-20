// content_disposition.rs — RFC 6266 + RFC 5987 Content-Disposition parser.
//
// PyPI itself just redirects to a `.whl` whose URL path already
// carries the canonical filename, but several mirrors (devpi, GitLab
// pkg registry, JFrog, Cloudflare CDN) serve wheels behind opaque URLs
// like `/api/files/abcdef0123` and set a `Content-Disposition:` header
// to hand the client the real filename:
//
//     Content-Disposition: attachment; filename="requests-2.31.0-…whl"
//     Content-Disposition: attachment; filename*=UTF-8''req%C3%BBests-…
//
// Without parsing this, our cache layer would pick `abcdef0123` as the
// filename and store the wheel without a recognisable name; uv does
// the same RFC 6266 + 5987 dance to pin filenames.
//
// Grammar (RFC 6266 §4.1, RFC 5987 §3.2):
//
//   content-disposition = disposition-type *( OWS ";" OWS disp-parm )
//   disposition-type    = "inline" / "attachment" / extension-token
//   disp-parm           = filename-parm / disp-ext-parm
//   filename-parm       = "filename" "=" value
//                       / "filename*" "=" ext-value
//   ext-value           = charset "'" [ language ] "'" value-chars
//   charset             = "UTF-8" / "ISO-8859-1" / mime-charset
//
// We accept both plain `filename=…` and RFC 5987 `filename*=…` forms.
// When both are present, the RFC says callers MUST prefer `filename*`
// (only that form can carry non-ASCII), so this module exposes
// `effective_filename()` to return the right one.

use std::collections::BTreeMap;

use crate::pkgmanage::pkgmgr::types::IndexError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContentDisposition {
    /// `inline`, `attachment`, or an extension token (all lower-cased
    /// per RFC 6266 §4.1).
    pub disposition: String,
    /// Plain ASCII `filename=…` parameter, if present.
    pub filename: Option<String>,
    /// RFC 5987 `filename*=charset''pct-encoded` parameter, already
    /// decoded into a UTF-8 String. Charset is restricted to `UTF-8`
    /// and `ISO-8859-1` (the two the RFC mandates support for).
    pub filename_star: Option<String>,
    /// Any extension parameters we didn't promote to a typed field.
    pub extensions: BTreeMap<String, String>,
}

impl ContentDisposition {
    /// Best filename per RFC 6266 §4.3 — prefer `filename*` if both
    /// are present, fall back to plain `filename=`, then to None.
    pub fn effective_filename(&self) -> Option<&str> {
        self.filename_star.as_deref().or(self.filename.as_deref())
    }
}

pub fn parse_content_disposition(header: &str) -> Result<ContentDisposition, IndexError> {
    let s = header.trim();
    if s.is_empty() {
        return Err(pe("empty Content-Disposition header"));
    }
    let bytes = s.as_bytes();
    let mut i = 0;
    // disposition-type token.
    while i < bytes.len() && is_tchar(bytes[i]) {
        i += 1;
    }
    if i == 0 {
        return Err(pe(&format!("expected disposition-type token, got {s:?}")));
    }
    let disposition = s[..i].to_ascii_lowercase();

    let mut filename: Option<String> = None;
    let mut filename_star: Option<String> = None;
    let mut extensions: BTreeMap<String, String> = BTreeMap::new();

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
                "expected ';' between parameters, found {:?}",
                bytes[i] as char
            )));
        }
        i += 1; // consume ';'.
        while i < bytes.len() && (bytes[i] == b' ' || bytes[i] == b'\t') {
            i += 1;
        }
        // Parameter name (may end in `*` for the ext-value form).
        let name_start = i;
        while i < bytes.len() && (is_tchar(bytes[i]) || bytes[i] == b'*') {
            i += 1;
        }
        if i == name_start {
            return Err(pe("expected parameter name"));
        }
        let name = s[name_start..i].to_ascii_lowercase();
        while i < bytes.len() && (bytes[i] == b' ' || bytes[i] == b'\t') {
            i += 1;
        }
        if i == bytes.len() || bytes[i] != b'=' {
            return Err(pe(&format!("expected '=' after parameter name {name:?}")));
        }
        i += 1; // consume '='.
        while i < bytes.len() && (bytes[i] == b' ' || bytes[i] == b'\t') {
            i += 1;
        }
        // Value: quoted-string, ext-value, or bare token.
        let value = if i < bytes.len() && bytes[i] == b'"' {
            read_quoted(s, &mut i)?
        } else {
            let v_start = i;
            // ext-value characters: token chars plus `'`, `%`,
            // and (per RFC 5987) the small set of "attr-char"
            // punctuation. We're permissive and accept anything
            // that's not `;`, whitespace, or end-of-string.
            while i < bytes.len() && bytes[i] != b';' && bytes[i] != b' ' && bytes[i] != b'\t' {
                i += 1;
            }
            if i == v_start {
                return Err(pe(&format!("expected parameter value for {name:?}")));
            }
            s[v_start..i].to_string()
        };
        match name.as_str() {
            "filename" => filename = Some(value),
            "filename*" => filename_star = Some(decode_ext_value(&value)?),
            _ => {
                extensions.entry(name).or_insert(value);
            }
        }
    }
    Ok(ContentDisposition {
        disposition,
        filename,
        filename_star,
        extensions,
    })
}

/// Decode an RFC 5987 ext-value: `charset'language'pct-encoded`.
fn decode_ext_value(raw: &str) -> Result<String, IndexError> {
    let parts: Vec<&str> = raw.splitn(3, '\'').collect();
    if parts.len() != 3 {
        return Err(pe(&format!(
            "expected RFC 5987 'charset\\'lang\\'value' shape, got {raw:?}"
        )));
    }
    let charset_raw = parts[0];
    let charset_upper = charset_raw.to_ascii_uppercase();
    let encoded = parts[2];
    let bytes = pct_decode(encoded)?;
    match charset_upper.as_str() {
        "UTF-8" => String::from_utf8(bytes).map_err(|e| pe(&format!("invalid UTF-8: {e}"))),
        "ISO-8859-1" => {
            // ISO-8859-1 → Unicode is the identity per code point.
            Ok(bytes.into_iter().map(|b| b as char).collect())
        }
        _ => Err(pe(&format!(
            "unsupported RFC 5987 charset {charset_raw:?} (only UTF-8 and ISO-8859-1 are RFC-required)"
        ))),
    }
}

fn pct_decode(s: &str) -> Result<Vec<u8>, IndexError> {
    let bytes = s.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' {
            if i + 2 >= bytes.len() {
                return Err(pe("incomplete percent-escape"));
            }
            let hi = from_hex(bytes[i + 1])?;
            let lo = from_hex(bytes[i + 2])?;
            out.push((hi << 4) | lo);
            i += 3;
        } else {
            out.push(bytes[i]);
            i += 1;
        }
    }
    Ok(out)
}

fn from_hex(b: u8) -> Result<u8, IndexError> {
    match b {
        b'0'..=b'9' => Ok(b - b'0'),
        b'a'..=b'f' => Ok(b - b'a' + 10),
        b'A'..=b'F' => Ok(b - b'A' + 10),
        _ => Err(pe(&format!(
            "non-hex digit in percent-escape: {:?}",
            b as char
        ))),
    }
}

fn read_quoted(s: &str, i: &mut usize) -> Result<String, IndexError> {
    let bytes = s.as_bytes();
    *i += 1; // consume opening '"'.
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
        url: "Content-Disposition".into(),
        detail: msg.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_attachment_with_plain_filename() {
        let cd =
            parse_content_disposition("attachment; filename=\"requests-2.31.0-py3-none-any.whl\"")
                .unwrap();
        assert_eq!(cd.disposition, "attachment");
        assert_eq!(
            cd.filename.as_deref(),
            Some("requests-2.31.0-py3-none-any.whl")
        );
        assert_eq!(cd.filename_star, None);
    }

    #[test]
    fn parses_inline_disposition() {
        let cd = parse_content_disposition("inline").unwrap();
        assert_eq!(cd.disposition, "inline");
        assert_eq!(cd.filename, None);
    }

    #[test]
    fn lowercases_disposition_token() {
        let cd = parse_content_disposition("ATTACHMENT; filename=\"x.whl\"").unwrap();
        assert_eq!(cd.disposition, "attachment");
    }

    #[test]
    fn lowercases_parameter_names() {
        let cd = parse_content_disposition("attachment; FileName=\"x.whl\"").unwrap();
        assert_eq!(cd.filename.as_deref(), Some("x.whl"));
    }

    #[test]
    fn parses_unquoted_filename() {
        let cd = parse_content_disposition("attachment; filename=plain.whl").unwrap();
        assert_eq!(cd.filename.as_deref(), Some("plain.whl"));
    }

    #[test]
    fn parses_rfc5987_utf8_filename_star() {
        // `filename*=UTF-8''req%C3%BBests-1.0.whl` → "reqûests-1.0.whl"
        let cd = parse_content_disposition("attachment; filename*=UTF-8''req%C3%BBests-1.0.whl")
            .unwrap();
        assert_eq!(cd.filename_star.as_deref(), Some("reqûests-1.0.whl"));
    }

    #[test]
    fn parses_rfc5987_iso_8859_1_filename_star() {
        // `filename*=ISO-8859-1''na%EFve-1.0.whl` → "naïve-1.0.whl"
        let cd =
            parse_content_disposition("attachment; filename*=ISO-8859-1''na%EFve-1.0.whl").unwrap();
        assert_eq!(cd.filename_star.as_deref(), Some("naïve-1.0.whl"));
    }

    #[test]
    fn rfc5987_charset_case_insensitive() {
        let cd = parse_content_disposition("attachment; filename*=utf-8''r%C3%BBn.whl").unwrap();
        assert_eq!(cd.filename_star.as_deref(), Some("rûn.whl"));
    }

    #[test]
    fn rfc5987_language_segment_tolerated() {
        // `filename*=UTF-8'en'na%C3%AFve-1.0.whl`
        let cd =
            parse_content_disposition("attachment; filename*=UTF-8'en'na%C3%AFve-1.0.whl").unwrap();
        assert_eq!(cd.filename_star.as_deref(), Some("naïve-1.0.whl"));
    }

    #[test]
    fn both_filename_forms_present_star_wins() {
        let cd = parse_content_disposition(
            "attachment; filename=\"fallback.whl\"; filename*=UTF-8''real-1.0.whl",
        )
        .unwrap();
        assert_eq!(cd.filename.as_deref(), Some("fallback.whl"));
        assert_eq!(cd.filename_star.as_deref(), Some("real-1.0.whl"));
        assert_eq!(cd.effective_filename(), Some("real-1.0.whl"));
    }

    #[test]
    fn effective_filename_falls_back_to_plain() {
        let cd = parse_content_disposition("attachment; filename=\"x.whl\"").unwrap();
        assert_eq!(cd.effective_filename(), Some("x.whl"));
    }

    #[test]
    fn effective_filename_none_when_no_param() {
        let cd = parse_content_disposition("inline").unwrap();
        assert_eq!(cd.effective_filename(), None);
    }

    #[test]
    fn extension_parameter_preserved() {
        let cd = parse_content_disposition(
            "attachment; filename=\"x.whl\"; modification-date=\"Wed, 12 Feb 2025 16:00:00 GMT\"",
        )
        .unwrap();
        assert_eq!(
            cd.extensions.get("modification-date").map(String::as_str),
            Some("Wed, 12 Feb 2025 16:00:00 GMT")
        );
    }

    #[test]
    fn tolerates_whitespace_between_parameters() {
        let cd = parse_content_disposition(
            "attachment  ;  filename = \"x.whl\"  ;  filename* = UTF-8''y.whl",
        )
        .unwrap();
        assert_eq!(cd.filename.as_deref(), Some("x.whl"));
        assert_eq!(cd.filename_star.as_deref(), Some("y.whl"));
    }

    #[test]
    fn rejects_empty_header() {
        let err = parse_content_disposition("   ").unwrap_err();
        assert!(err.to_string().contains("empty"));
    }

    #[test]
    fn rejects_missing_disposition_token() {
        let err = parse_content_disposition("; filename=\"x.whl\"").unwrap_err();
        assert!(err.to_string().contains("disposition-type"));
    }

    #[test]
    fn rejects_unterminated_quoted_value() {
        let err = parse_content_disposition("attachment; filename=\"unterm").unwrap_err();
        assert!(err.to_string().contains("unterminated"));
    }

    #[test]
    fn rejects_missing_equals() {
        let err = parse_content_disposition("attachment; filename").unwrap_err();
        assert!(err.to_string().contains("expected '='"));
    }

    #[test]
    fn rejects_rfc5987_unsupported_charset() {
        let err = parse_content_disposition(
            "attachment; filename*=Shift_JIS''%E6%97%A5%E6%9C%AC%E8%AA%9E.whl",
        )
        .unwrap_err();
        assert!(err.to_string().contains("unsupported"));
        assert!(err.to_string().contains("Shift_JIS"));
    }

    #[test]
    fn rejects_rfc5987_incomplete_shape() {
        // Missing the second `'` (no language segment separator).
        let err = parse_content_disposition("attachment; filename*=UTF-8noseparator").unwrap_err();
        assert!(err.to_string().contains("RFC 5987"));
    }

    #[test]
    fn rejects_percent_escape_with_non_hex() {
        let err =
            parse_content_disposition("attachment; filename*=UTF-8''bad%ZZchar.whl").unwrap_err();
        assert!(err.to_string().contains("non-hex"));
    }

    #[test]
    fn rejects_truncated_percent_escape() {
        let err = parse_content_disposition("attachment; filename*=UTF-8''trunc%").unwrap_err();
        assert!(err.to_string().contains("incomplete"));
    }

    #[test]
    fn quoted_filename_unescapes_backslash_quote() {
        // `filename="weird\\\"name.whl"` → `weird"name.whl`
        let cd = parse_content_disposition("attachment; filename=\"weird\\\"name.whl\"").unwrap();
        assert_eq!(cd.filename.as_deref(), Some(r#"weird"name.whl"#));
    }

    #[test]
    fn gitlab_pkg_registry_shape() {
        // Real GitLab-style header for a JFrog-fronted wheel.
        let header = "attachment; filename=\"foo-1.0.0-py3-none-any.whl\"; filename*=UTF-8''foo-1.0.0-py3-none-any.whl";
        let cd = parse_content_disposition(header).unwrap();
        assert_eq!(cd.disposition, "attachment");
        assert_eq!(cd.effective_filename(), Some("foo-1.0.0-py3-none-any.whl"));
    }
}
