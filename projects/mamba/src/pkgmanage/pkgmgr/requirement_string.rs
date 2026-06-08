// PEP 508 requirement-string head parser (Tick 113).
//
// Splits a requirement string into typed parts:
//
//   name[extras] <version-specifier> ; <marker>
//   name[extras] @ <url>             ; <marker>
//
// The specifier and marker tails are captured as opaque strings — downstream
// evaluators already exist in this crate:
//
//   * pep440::parse for individual `==1.2.3` / `>=2,<3` clauses
//   * markers::evaluate for the `; python_version >= "3.10"` tail
//
// Strict parts:
//   * Name validated by PEP 503 character set (letterOrDigit-led,
//     `[A-Za-z0-9._-]*` body) before PEP 503 normalization.
//   * Extras qualifier delegated to `extras_spec::ExtrasSpec::parse_qualifier`.
//   * Quote-balance and bracket-balance enforced.
//   * `@` URL form and version-specifier form are mutually exclusive.
//
// Lenient parts:
//   * Whitespace is permitted between every token, matching pip / uv.
//   * Version-specifier text is not re-parsed here; we just capture it so
//     the resolver can run pep440 over each clause.

use crate::pkgmanage::pkgmgr::extras_spec::ExtrasSpec;
use crate::pkgmanage::pkgmgr::name_normalize::pep503_normalize;
use crate::pkgmanage::pkgmgr::types::IndexError;

/// A PEP 508 requirement parsed into structural parts.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Requirement {
    /// PEP 503-normalized distribution name.
    pub name: String,
    /// Original distribution name as written (preserved for diagnostics
    /// and round-tripping the user's spelling).
    pub raw_name: String,
    /// Extras qualifier (`[a,b]`) — empty when no extras were given.
    pub extras: ExtrasSpec,
    /// Version-specifier clause text, e.g. `>=1.2,<2`. None for URL or
    /// bare-name requirements.
    pub specifier: Option<String>,
    /// Direct-URL reference text after `@`. None for version-specifier
    /// or bare-name requirements.
    pub url: Option<String>,
    /// Environment-marker tail text (without the leading `;`). None
    /// when no marker was supplied.
    pub marker: Option<String>,
}

impl Requirement {
    /// Parse a single PEP 508 requirement string.
    pub fn parse(src: &str) -> Result<Self, IndexError> {
        let trimmed = src.trim();
        if trimmed.is_empty() {
            return Err(parse_err("empty requirement string"));
        }

        // Split marker tail at the first top-level unquoted `;`. PEP 508
        // markers may contain `;` only inside string literals, so we track
        // quote state.
        let (head, marker) = split_marker(trimmed)?;
        let head = head.trim();

        // The name occupies the leading identifier; anything from the first
        // non-identifier character onward is `[extras]`, `@ url`, or version
        // specifier.
        let (raw_name, after_name) = split_name(head)?;
        validate_name_chars(&raw_name)?;
        let name = pep503_normalize(&raw_name);
        if name.is_empty() {
            return Err(parse_err("requirement name is empty after normalization"));
        }

        let mut rest = after_name.trim_start();
        let extras = if rest.starts_with('[') {
            let (qualifier, tail) = split_extras_qualifier(rest)?;
            rest = tail.trim_start();
            ExtrasSpec::parse_qualifier(qualifier)?
        } else {
            ExtrasSpec::default()
        };

        let (specifier, url) = split_specifier_or_url(rest)?;

        Ok(Self {
            name,
            raw_name,
            extras,
            specifier,
            url,
            marker,
        })
    }
}

fn parse_err(detail: impl Into<String>) -> IndexError {
    IndexError::ParseError {
        url: "<requirement>".to_string(),
        detail: detail.into(),
    }
}

/// Split at the first unquoted top-level `;`. Returns (head, marker?).
fn split_marker(src: &str) -> Result<(&str, Option<String>), IndexError> {
    let bytes = src.as_bytes();
    let mut in_squote = false;
    let mut in_dquote = false;
    for (i, &b) in bytes.iter().enumerate() {
        match b {
            b'\'' if !in_dquote => in_squote = !in_squote,
            b'"' if !in_squote => in_dquote = !in_dquote,
            b';' if !in_squote && !in_dquote => {
                let head = &src[..i];
                let tail = src[i + 1..].trim();
                if tail.is_empty() {
                    return Err(parse_err("marker `;` separator with empty marker tail"));
                }
                return Ok((head, Some(tail.to_string())));
            }
            _ => {}
        }
    }
    if in_squote || in_dquote {
        return Err(parse_err("unterminated quoted string in requirement"));
    }
    Ok((src, None))
}

/// Pull the leading identifier off `src`. The identifier must start with a
/// letterOrDigit and continues across `[A-Za-z0-9._-]`. Returns
/// (raw_name, remainder).
fn split_name(src: &str) -> Result<(String, &str), IndexError> {
    let bytes = src.as_bytes();
    if bytes.is_empty() {
        return Err(parse_err("requirement is missing a name"));
    }
    if !bytes[0].is_ascii_alphanumeric() {
        return Err(parse_err(format!(
            "requirement name must start with a letter or digit, got {:?}",
            src.chars().next().unwrap()
        )));
    }
    let mut end = 0;
    while end < bytes.len() && is_name_char(bytes[end]) {
        end += 1;
    }
    let raw = src[..end].to_string();
    Ok((raw, &src[end..]))
}

fn is_name_char(b: u8) -> bool {
    b.is_ascii_alphanumeric() || matches!(b, b'.' | b'_' | b'-')
}

fn validate_name_chars(name: &str) -> Result<(), IndexError> {
    if name.is_empty() {
        return Err(parse_err("requirement name is empty"));
    }
    if !name.as_bytes()[0].is_ascii_alphanumeric() {
        return Err(parse_err(
            "requirement name must start with a letter or digit",
        ));
    }
    for b in name.bytes() {
        if !is_name_char(b) {
            return Err(parse_err(format!(
                "requirement name contains invalid character {:?}",
                b as char
            )));
        }
    }
    Ok(())
}

/// Slice off `[ ... ]` starting at `src[0] == '['`. Returns (qualifier
/// including brackets, remainder).
fn split_extras_qualifier(src: &str) -> Result<(&str, &str), IndexError> {
    debug_assert!(src.starts_with('['));
    let bytes = src.as_bytes();
    let mut depth = 0usize;
    for (i, &b) in bytes.iter().enumerate() {
        match b {
            b'[' => depth += 1,
            b']' => {
                depth -= 1;
                if depth == 0 {
                    let qualifier = &src[..=i];
                    return Ok((qualifier, &src[i + 1..]));
                }
            }
            _ => {}
        }
    }
    Err(parse_err("unterminated `[extras]` qualifier"))
}

/// Decide whether the remainder is a URL spec (`@ <url>`) or a version
/// specifier. Returns (specifier?, url?). At most one is set.
fn split_specifier_or_url(rest: &str) -> Result<(Option<String>, Option<String>), IndexError> {
    let r = rest.trim();
    if r.is_empty() {
        return Ok((None, None));
    }
    if let Some(stripped) = r.strip_prefix('@') {
        let url = stripped.trim();
        if url.is_empty() {
            return Err(parse_err("`@` direct-URL requirement is missing the URL"));
        }
        return Ok((None, Some(url.to_string())));
    }
    // Any non-empty remainder that does not start with `@` is treated as
    // a version specifier. We do not validate clause shape here — that's
    // pep440's job — but we reject obviously broken inputs that contain
    // a stray `@` further in.
    if r.contains('@') {
        return Err(parse_err(
            "version specifier may not contain `@`; use `name @ url` form instead",
        ));
    }
    Ok((Some(r.to_string()), None))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn req(s: &str) -> Requirement {
        Requirement::parse(s).unwrap_or_else(|e| panic!("parse failed for {s:?}: {e:?}"))
    }

    #[test]
    fn parses_bare_name() {
        let r = req("requests");
        assert_eq!(r.name, "requests");
        assert_eq!(r.raw_name, "requests");
        assert!(r.extras.is_empty());
        assert_eq!(r.specifier, None);
        assert_eq!(r.url, None);
        assert_eq!(r.marker, None);
    }

    #[test]
    fn normalizes_name_per_pep503() {
        let r = req("Requests");
        assert_eq!(r.name, "requests");
        assert_eq!(r.raw_name, "Requests");

        let r = req("my_pkg");
        assert_eq!(r.name, "my-pkg");
        assert_eq!(r.raw_name, "my_pkg");
    }

    #[test]
    fn parses_simple_version_specifier() {
        let r = req("requests>=2.31");
        assert_eq!(r.name, "requests");
        assert_eq!(r.specifier.as_deref(), Some(">=2.31"));
    }

    #[test]
    fn parses_compound_specifier_with_whitespace() {
        let r = req("requests >= 2.31 , <3");
        assert_eq!(r.specifier.as_deref(), Some(">= 2.31 , <3"));
    }

    #[test]
    fn parses_exact_pin() {
        let r = req("requests==2.31.0");
        assert_eq!(r.specifier.as_deref(), Some("==2.31.0"));
    }

    #[test]
    fn parses_extras_only() {
        let r = req("requests[socks]");
        assert_eq!(r.name, "requests");
        assert!(r.extras.contains("socks"));
        assert_eq!(r.specifier, None);
    }

    #[test]
    fn parses_extras_with_specifier() {
        let r = req("requests[socks,security]>=2.31");
        assert_eq!(r.name, "requests");
        assert!(r.extras.contains("socks"));
        assert!(r.extras.contains("security"));
        assert_eq!(r.specifier.as_deref(), Some(">=2.31"));
    }

    #[test]
    fn parses_django_style_extras() {
        let r = req("Django[argon2,bcrypt]>=4.2");
        assert_eq!(r.name, "django");
        assert!(r.extras.contains("argon2"));
        assert!(r.extras.contains("bcrypt"));
    }

    #[test]
    fn parses_url_form() {
        let r = req("requests @ https://example.com/requests-2.31.0.tar.gz");
        assert_eq!(r.name, "requests");
        assert_eq!(
            r.url.as_deref(),
            Some("https://example.com/requests-2.31.0.tar.gz")
        );
        assert_eq!(r.specifier, None);
    }

    #[test]
    fn parses_url_form_with_extras() {
        let r = req("requests[socks] @ https://example.com/x.whl");
        assert!(r.extras.contains("socks"));
        assert_eq!(r.url.as_deref(), Some("https://example.com/x.whl"));
    }

    #[test]
    fn parses_marker_tail() {
        let r = req("requests >= 2.31 ; python_version >= \"3.10\"");
        assert_eq!(r.specifier.as_deref(), Some(">= 2.31"));
        assert_eq!(r.marker.as_deref(), Some("python_version >= \"3.10\""));
    }

    #[test]
    fn parses_marker_with_semicolon_inside_string_literal() {
        // PEP 508 markers compare against quoted strings; a `;` inside such
        // a string must not be treated as the marker separator.
        let r = req("requests >= 2.31 ; sys_platform == \"linux;weird\"");
        assert_eq!(
            r.marker.as_deref(),
            Some("sys_platform == \"linux;weird\"")
        );
    }

    #[test]
    fn parses_url_with_marker() {
        let r = req("pkg @ https://x/y.whl ; python_version >= \"3.10\"");
        assert_eq!(r.url.as_deref(), Some("https://x/y.whl"));
        assert_eq!(r.marker.as_deref(), Some("python_version >= \"3.10\""));
    }

    #[test]
    fn rejects_empty_string() {
        let err = Requirement::parse("").unwrap_err();
        let msg = format!("{err:?}");
        assert!(msg.contains("empty"));
    }

    #[test]
    fn rejects_whitespace_only() {
        assert!(Requirement::parse("   ").is_err());
    }

    #[test]
    fn rejects_leading_digit_only_ok_but_special_char_not() {
        // Digits are letterOrDigit-led — allowed.
        let r = req("7zip");
        assert_eq!(r.name, "7zip");
    }

    #[test]
    fn rejects_leading_underscore() {
        let err = Requirement::parse("_private").unwrap_err();
        let msg = format!("{err:?}");
        assert!(msg.contains("letter or digit"));
    }

    #[test]
    fn rejects_leading_dot() {
        assert!(Requirement::parse(".pkg").is_err());
    }

    #[test]
    fn rejects_unbalanced_extras_bracket() {
        let err = Requirement::parse("requests[socks").unwrap_err();
        let msg = format!("{err:?}");
        assert!(msg.contains("unterminated"));
    }

    #[test]
    fn rejects_empty_url() {
        assert!(Requirement::parse("pkg @").is_err());
        assert!(Requirement::parse("pkg @   ").is_err());
    }

    #[test]
    fn rejects_marker_with_empty_tail() {
        assert!(Requirement::parse("pkg;").is_err());
        assert!(Requirement::parse("pkg;   ").is_err());
    }

    #[test]
    fn rejects_unterminated_quote_in_head() {
        // The opening quote is reached BEFORE the `;` separator, so the
        // marker-splitter follows quote state. The trailing `;` is then
        // swallowed as if inside the quote, leaving us unterminated at
        // end-of-string — surfaced as a parse error.
        assert!(
            Requirement::parse("pkg \"oops ; python_version >= \"3.10\"").is_err()
        );
    }

    #[test]
    fn rejects_at_inside_version_specifier() {
        // `pkg>=1@2` is nonsense — must use `name @ url` form.
        let err = Requirement::parse("pkg>=1@2").unwrap_err();
        let msg = format!("{err:?}");
        assert!(msg.contains("@"));
    }

    #[test]
    fn whitespace_between_every_token_is_tolerated() {
        let r = req("  requests   [  socks  ,  security  ]   >=  2.31   ;  python_version  >=  \"3.10\"  ");
        assert_eq!(r.name, "requests");
        assert!(r.extras.contains("socks"));
        assert!(r.extras.contains("security"));
        assert_eq!(r.specifier.as_deref(), Some(">=  2.31"));
        assert_eq!(
            r.marker.as_deref(),
            Some("python_version  >=  \"3.10\"")
        );
    }

    #[test]
    fn extras_apply_pep685_normalization() {
        // ExtrasSpec normalizes its members per PEP 685; ensure the
        // pipeline propagates that.
        let r = req("pkg[Dev_Tools, Foo.Bar]");
        assert!(r.extras.contains("dev-tools"));
        assert!(r.extras.contains("foo-bar"));
    }

    #[test]
    fn round_trips_through_render_for_extras() {
        let r = req("pkg[c, b, a]");
        // ExtrasSpec uses BTreeSet under the hood — canonical sort order.
        assert_eq!(r.extras.render(), "[a,b,c]");
    }

    #[test]
    fn marker_is_optional_with_specifier() {
        let r = req("pkg>=1");
        assert_eq!(r.marker, None);
    }

    #[test]
    fn marker_is_optional_with_url() {
        let r = req("pkg @ file:///tmp/x.whl");
        assert_eq!(r.marker, None);
        assert_eq!(r.url.as_deref(), Some("file:///tmp/x.whl"));
    }

    #[test]
    fn name_with_dots_is_preserved_raw_and_normalized() {
        let r = req("zope.interface");
        assert_eq!(r.raw_name, "zope.interface");
        assert_eq!(r.name, "zope-interface");
    }

    #[test]
    fn realistic_pyproject_line() {
        // Approximates a typical pyproject.toml dependency entry.
        let r = req("urllib3>=1.21.1,<3 ; python_version >= \"3.7\"");
        assert_eq!(r.name, "urllib3");
        assert_eq!(r.specifier.as_deref(), Some(">=1.21.1,<3"));
        assert_eq!(r.marker.as_deref(), Some("python_version >= \"3.7\""));
    }

    #[test]
    fn realistic_uv_lockfile_line() {
        // uv lockfile-style direct-URL pin with marker.
        let r = req(
            "torch[cuda] @ https://download.pytorch.org/whl/cu121/torch-2.1.0.whl ; sys_platform == \"linux\"",
        );
        assert_eq!(r.name, "torch");
        assert!(r.extras.contains("cuda"));
        assert_eq!(
            r.url.as_deref(),
            Some("https://download.pytorch.org/whl/cu121/torch-2.1.0.whl")
        );
        assert_eq!(r.marker.as_deref(), Some("sys_platform == \"linux\""));
    }
}
