// cache_control.rs — HTTP `Cache-Control` header parser (RFC 9111 §5.2).
//
// uv treats PyPI's simple-API and JSON-API responses as cacheable per the
// standard HTTP cache semantics, with the index server's `Cache-Control`
// directives controlling freshness. This module gives mamba the same
// parsing surface so the cache layer can honour `max-age`, `no-cache`,
// `immutable`, and the rest of the documented directive set.
//
// Grammar per RFC 9111 §5.2 + RFC 9110 §5.5 (the field-value ABNF):
//
//     Cache-Control = 1#cache-directive
//     cache-directive = token [ "=" ( token / quoted-string ) ]
//
// Multiple headers stack: `Cache-Control: max-age=60` followed by
// `Cache-Control: must-revalidate` is equivalent to `max-age=60,
// must-revalidate`. Callers can either concatenate with `, ` and call
// `parse_cache_control` once, or feed each line through the same parser
// and merge the results with `CacheControl::merge`.

use std::collections::BTreeMap;

use crate::pkgmanage::pkgmgr::types::IndexError;

/// Parsed Cache-Control directives. Fields are `Option` for the
/// integer-valued ones and `bool` for the no-argument flags. Unknown
/// extension directives land in `extensions`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CacheControl {
    pub max_age: Option<u64>,
    pub s_maxage: Option<u64>,
    pub stale_while_revalidate: Option<u64>,
    pub no_cache: bool,
    pub no_store: bool,
    pub must_revalidate: bool,
    pub proxy_revalidate: bool,
    pub public: bool,
    pub private: bool,
    pub immutable: bool,
    pub no_transform: bool,
    pub only_if_cached: bool,
    /// Directives we don't model individually. Key is lowercased directive
    /// name; value is the raw argument string (empty when bare).
    pub extensions: BTreeMap<String, String>,
}

impl CacheControl {
    /// Merge another header's directives into self. Boolean flags OR in;
    /// integer fields take the *minimum* (the stricter / safer choice for
    /// a cache). Extension keys overwrite.
    pub fn merge(&mut self, other: CacheControl) {
        fn min_opt(a: Option<u64>, b: Option<u64>) -> Option<u64> {
            match (a, b) {
                (Some(x), Some(y)) => Some(x.min(y)),
                (Some(x), None) | (None, Some(x)) => Some(x),
                (None, None) => None,
            }
        }
        self.max_age = min_opt(self.max_age, other.max_age);
        self.s_maxage = min_opt(self.s_maxage, other.s_maxage);
        self.stale_while_revalidate =
            min_opt(self.stale_while_revalidate, other.stale_while_revalidate);
        self.no_cache |= other.no_cache;
        self.no_store |= other.no_store;
        self.must_revalidate |= other.must_revalidate;
        self.proxy_revalidate |= other.proxy_revalidate;
        self.public |= other.public;
        self.private |= other.private;
        self.immutable |= other.immutable;
        self.no_transform |= other.no_transform;
        self.only_if_cached |= other.only_if_cached;
        for (k, v) in other.extensions {
            self.extensions.insert(k, v);
        }
    }
}

/// Parse a comma-separated `Cache-Control` field value. Whitespace
/// surrounding directives and the `=` are tolerated. Quoted-string values
/// have surrounding quotes stripped and `\` escapes processed.
pub fn parse_cache_control(header: &str) -> Result<CacheControl, IndexError> {
    let mut cc = CacheControl::default();
    let directives = split_directives(header)?;
    for (name, value) in directives {
        let key = name.to_ascii_lowercase();
        match key.as_str() {
            "max-age" => cc.max_age = Some(require_uint(&key, value.as_deref())?),
            "s-maxage" => cc.s_maxage = Some(require_uint(&key, value.as_deref())?),
            "stale-while-revalidate" => {
                cc.stale_while_revalidate = Some(require_uint(&key, value.as_deref())?);
            }
            "no-cache" => cc.no_cache = true,
            "no-store" => cc.no_store = true,
            "must-revalidate" => cc.must_revalidate = true,
            "proxy-revalidate" => cc.proxy_revalidate = true,
            "public" => cc.public = true,
            "private" => cc.private = true,
            "immutable" => cc.immutable = true,
            "no-transform" => cc.no_transform = true,
            "only-if-cached" => cc.only_if_cached = true,
            _ => {
                cc.extensions.insert(key, value.unwrap_or_default());
            }
        }
    }
    Ok(cc)
}

fn require_uint(name: &str, value: Option<&str>) -> Result<u64, IndexError> {
    let v = value.ok_or_else(|| IndexError::ParseError {
        url: "Cache-Control".into(),
        detail: format!("{name} requires an integer argument"),
    })?;
    v.parse::<u64>().map_err(|e| IndexError::ParseError {
        url: "Cache-Control".into(),
        detail: format!("{name}={v:?} is not a valid unsigned integer: {e}"),
    })
}

// ---- tokenizer ----------------------------------------------------------

fn split_directives(header: &str) -> Result<Vec<(String, Option<String>)>, IndexError> {
    let bytes = header.as_bytes();
    let mut i = 0usize;
    let mut out = Vec::new();
    while i < bytes.len() {
        // Skip optional whitespace and any empty list elements (the
        // RFC 9110 §5.6.1 `#rule` ABNF permits `,,` and leading/trailing
        // bare commas).
        while i < bytes.len() && (bytes[i] == b' ' || bytes[i] == b'\t' || bytes[i] == b',') {
            i += 1;
        }
        if i >= bytes.len() {
            break;
        }
        // Read a token (directive name).
        let start = i;
        while i < bytes.len() && is_token_byte(bytes[i]) {
            i += 1;
        }
        if i == start {
            return Err(IndexError::ParseError {
                url: "Cache-Control".into(),
                detail: format!("expected directive name at byte {start}"),
            });
        }
        let name = std::str::from_utf8(&bytes[start..i])
            .map_err(|_| IndexError::ParseError {
                url: "Cache-Control".into(),
                detail: "non-UTF-8 directive name".into(),
            })?
            .to_string();

        // Skip whitespace, then optional `= value`.
        while i < bytes.len() && (bytes[i] == b' ' || bytes[i] == b'\t') {
            i += 1;
        }
        let mut value: Option<String> = None;
        if i < bytes.len() && bytes[i] == b'=' {
            i += 1;
            while i < bytes.len() && (bytes[i] == b' ' || bytes[i] == b'\t') {
                i += 1;
            }
            if i < bytes.len() && bytes[i] == b'"' {
                value = Some(read_quoted_string(bytes, &mut i)?);
            } else {
                let vstart = i;
                while i < bytes.len() && is_token_byte(bytes[i]) {
                    i += 1;
                }
                if i == vstart {
                    return Err(IndexError::ParseError {
                        url: "Cache-Control".into(),
                        detail: format!("expected value after '=' in {name}"),
                    });
                }
                value = Some(
                    std::str::from_utf8(&bytes[vstart..i])
                        .map_err(|_| IndexError::ParseError {
                            url: "Cache-Control".into(),
                            detail: "non-UTF-8 directive value".into(),
                        })?
                        .to_string(),
                );
            }
        }

        // Skip whitespace and expect either `,` or end-of-input.
        while i < bytes.len() && (bytes[i] == b' ' || bytes[i] == b'\t') {
            i += 1;
        }
        if i < bytes.len() {
            if bytes[i] == b',' {
                i += 1;
            } else {
                return Err(IndexError::ParseError {
                    url: "Cache-Control".into(),
                    detail: format!(
                        "expected ',' or end-of-input after directive {name}, found {:?}",
                        bytes[i] as char
                    ),
                });
            }
        }
        out.push((name, value));
    }
    Ok(out)
}

fn read_quoted_string(bytes: &[u8], i: &mut usize) -> Result<String, IndexError> {
    // Caller positioned *i at the opening `"`.
    debug_assert_eq!(bytes[*i], b'"');
    *i += 1;
    let mut out = String::new();
    while *i < bytes.len() {
        let b = bytes[*i];
        if b == b'\\' {
            *i += 1;
            if *i >= bytes.len() {
                return Err(IndexError::ParseError {
                    url: "Cache-Control".into(),
                    detail: "dangling '\\' in quoted-string".into(),
                });
            }
            out.push(bytes[*i] as char);
            *i += 1;
            continue;
        }
        if b == b'"' {
            *i += 1;
            return Ok(out);
        }
        out.push(b as char);
        *i += 1;
    }
    Err(IndexError::ParseError {
        url: "Cache-Control".into(),
        detail: "unterminated quoted-string".into(),
    })
}

fn is_token_byte(b: u8) -> bool {
    // RFC 9110 §5.6.2 token: 1*<any VCHAR except delimiters>.
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
            | b'0'..=b'9'
            | b'A'..=b'Z'
            | b'a'..=b'z'
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_max_age() {
        let cc = parse_cache_control("max-age=60").unwrap();
        assert_eq!(cc.max_age, Some(60));
    }

    #[test]
    fn parses_no_cache_flag() {
        let cc = parse_cache_control("no-cache").unwrap();
        assert!(cc.no_cache);
    }

    #[test]
    fn parses_compound_directive_list() {
        let cc = parse_cache_control("public, max-age=3600, immutable").unwrap();
        assert!(cc.public);
        assert_eq!(cc.max_age, Some(3600));
        assert!(cc.immutable);
        assert!(!cc.private);
    }

    #[test]
    fn parses_quoted_string_value() {
        let cc = parse_cache_control(r#"private="set-cookie""#).unwrap();
        assert!(cc.private);
        // The argument to private is absorbed as part of the flag in our
        // simple model — note we just set the flag and discard the
        // field-name list (cache modeling treats this as opaque).
    }

    #[test]
    fn parses_extension_directive() {
        let cc = parse_cache_control("x-pypi-tag=mirror-eu-1").unwrap();
        assert_eq!(
            cc.extensions.get("x-pypi-tag").map(String::as_str),
            Some("mirror-eu-1")
        );
    }

    #[test]
    fn parses_bare_extension_directive() {
        let cc = parse_cache_control("x-bare").unwrap();
        assert_eq!(cc.extensions.get("x-bare").map(String::as_str), Some(""));
    }

    #[test]
    fn case_insensitive_directive_names() {
        let cc = parse_cache_control("MAX-AGE=10, No-Store, Public").unwrap();
        assert_eq!(cc.max_age, Some(10));
        assert!(cc.no_store);
        assert!(cc.public);
    }

    #[test]
    fn whitespace_around_equals_tolerated() {
        let cc = parse_cache_control("max-age = 120").unwrap();
        assert_eq!(cc.max_age, Some(120));
    }

    #[test]
    fn rejects_missing_integer_argument() {
        let err = parse_cache_control("max-age").unwrap_err();
        assert!(err.to_string().contains("max-age requires"));
    }

    #[test]
    fn rejects_non_numeric_max_age() {
        let err = parse_cache_control("max-age=forever").unwrap_err();
        assert!(err.to_string().contains("not a valid unsigned"));
    }

    #[test]
    fn rejects_negative_max_age() {
        // `-` is not a valid token byte, so the parser bails before
        // reaching integer parse — the failure is about value shape.
        let err = parse_cache_control("max-age=-5").unwrap_err();
        let s = err.to_string();
        assert!(
            s.contains("expected value") || s.contains("not a valid"),
            "got {s}"
        );
    }

    #[test]
    fn rejects_unterminated_quoted_string() {
        let err = parse_cache_control(r#"private="oops"#).unwrap_err();
        assert!(err.to_string().contains("unterminated"));
    }

    #[test]
    fn quoted_string_with_escape() {
        let cc = parse_cache_control(r#"x-note="hello \"world\"""#).unwrap();
        assert_eq!(
            cc.extensions.get("x-note").map(String::as_str),
            Some(r#"hello "world""#)
        );
    }

    #[test]
    fn handles_all_standard_flags() {
        let cc = parse_cache_control(
            "no-cache, no-store, must-revalidate, proxy-revalidate, public, private, \
             immutable, no-transform, only-if-cached",
        )
        .unwrap();
        assert!(cc.no_cache);
        assert!(cc.no_store);
        assert!(cc.must_revalidate);
        assert!(cc.proxy_revalidate);
        assert!(cc.public);
        assert!(cc.private);
        assert!(cc.immutable);
        assert!(cc.no_transform);
        assert!(cc.only_if_cached);
    }

    #[test]
    fn merge_min_for_max_age_or_for_flags() {
        let mut a = parse_cache_control("max-age=600, public").unwrap();
        let b = parse_cache_control("max-age=60, must-revalidate").unwrap();
        a.merge(b);
        assert_eq!(a.max_age, Some(60));
        assert!(a.public);
        assert!(a.must_revalidate);
    }

    #[test]
    fn merge_keeps_existing_when_other_absent() {
        let mut a = parse_cache_control("max-age=600").unwrap();
        let b = parse_cache_control("public").unwrap();
        a.merge(b);
        assert_eq!(a.max_age, Some(600));
        assert!(a.public);
    }

    #[test]
    fn empty_header_parses_as_default() {
        let cc = parse_cache_control("").unwrap();
        assert_eq!(cc, CacheControl::default());
    }

    #[test]
    fn whitespace_only_parses_as_default() {
        let cc = parse_cache_control("   ").unwrap();
        assert_eq!(cc, CacheControl::default());
    }

    #[test]
    fn stale_while_revalidate_parsed() {
        let cc = parse_cache_control("max-age=300, stale-while-revalidate=60").unwrap();
        assert_eq!(cc.max_age, Some(300));
        assert_eq!(cc.stale_while_revalidate, Some(60));
    }

    #[test]
    fn s_maxage_parsed() {
        let cc = parse_cache_control("max-age=60, s-maxage=120").unwrap();
        assert_eq!(cc.max_age, Some(60));
        assert_eq!(cc.s_maxage, Some(120));
    }

    #[test]
    fn trailing_comma_tolerated() {
        // RFC 9110 §5.6.1 `#rule` ABNF permits empty list elements (the
        // famed `1#element = element *( OWS "," OWS [ element ] )`).
        // Senders sometimes emit `max-age=60,` — accept it silently.
        let cc = parse_cache_control("max-age=60,").unwrap();
        assert_eq!(cc.max_age, Some(60));
    }

    #[test]
    fn leading_comma_tolerated() {
        let cc = parse_cache_control(", max-age=60").unwrap();
        assert_eq!(cc.max_age, Some(60));
    }
}
