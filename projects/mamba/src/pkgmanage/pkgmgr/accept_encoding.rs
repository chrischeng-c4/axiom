// accept_encoding.rs — Accept-Encoding / Content-Encoding header helpers.
//
// Per RFC 9110 §12.5.3 (Accept-Encoding) and §8.4 (Content-Encoding),
// HTTP clients negotiate compression by listing acceptable codings:
//
//   Accept-Encoding: gzip, deflate;q=0.5, br;q=1.0, identity;q=0
//
// And the server reports the chosen layering on the response:
//
//   Content-Encoding: br
//   Content-Encoding: gzip, br   # outer-first layering
//
// Package indexes typically serve PEP 691 JSON responses with `gzip`
// or `br` compression. We need three operations:
//
//   * `build_accept_encoding(codings)` — render an ordered list with
//     standard q-values so callers can configure reqwest middleware.
//
//   * `parse_accept_encoding(header)` — useful when replaying cached
//     transcripts or implementing a mirror-side proxy; q-values are
//     extracted and the list is sorted in preference order.
//
//   * `parse_content_encoding(header)` — split a response header into
//     its layering, outer-first. RFC 9110 §8.4 explicitly defines the
//     order: the first coding listed was applied first (so the client
//     decodes in reverse). We preserve that order.
//
// All errors funnel through `IndexError::ParseError { url: "", ... }`.

use crate::pkgmanage::pkgmgr::types::IndexError;

/// One entry in an Accept-Encoding header, after q-value parsing.
#[derive(Debug, Clone, PartialEq)]
pub struct AcceptEncoding {
    /// The coding name, lowercased per RFC 9110 §12.5.3 (case-insensitive).
    pub coding: String,
    /// Quality value in `[0.0, 1.0]`. Absent q defaults to 1.0.
    pub q: f32,
}

impl AcceptEncoding {
    pub fn new(coding: &str) -> Self {
        Self {
            coding: coding.to_ascii_lowercase(),
            q: 1.0,
        }
    }

    pub fn with_q(coding: &str, q: f32) -> Self {
        Self {
            coding: coding.to_ascii_lowercase(),
            q: q.clamp(0.0, 1.0),
        }
    }

    /// `true` if this entry says "do not send this coding" (q=0).
    pub fn is_rejected(&self) -> bool {
        self.q == 0.0
    }
}

/// Render an Accept-Encoding request header value from an ordered
/// preference list. Each entry is emitted with its q-value unless
/// q == 1.0 (the default).
pub fn build_accept_encoding(entries: &[AcceptEncoding]) -> String {
    entries
        .iter()
        .map(|e| {
            if (e.q - 1.0).abs() < f32::EPSILON {
                e.coding.clone()
            } else {
                format!("{}; q={}", e.coding, format_q(e.q))
            }
        })
        .collect::<Vec<_>>()
        .join(", ")
}

fn format_q(q: f32) -> String {
    // RFC 9110 §12.4.2: up to three decimal places, trailing zeros
    // optional. Trim trailing zeros and trailing dot.
    let formatted = format!("{q:.3}");
    let trimmed = formatted.trim_end_matches('0').trim_end_matches('.');
    if trimmed.is_empty() {
        "0".into()
    } else {
        trimmed.to_string()
    }
}

/// Parse an Accept-Encoding header. Returns entries in preference
/// order: q=1.0 entries first (in declaration order), then sorted
/// descending by q. Stable: equal-q entries keep declaration order.
pub fn parse_accept_encoding(header: &str) -> Result<Vec<AcceptEncoding>, IndexError> {
    let trimmed = header.trim();
    if trimmed.is_empty() {
        return Ok(Vec::new());
    }

    let mut entries: Vec<(usize, AcceptEncoding)> = Vec::new();
    for (idx, raw) in trimmed.split(',').enumerate() {
        let part = raw.trim();
        if part.is_empty() {
            continue;
        }
        let mut params = part.split(';').map(str::trim);
        let coding = params
            .next()
            .ok_or_else(|| IndexError::ParseError {
                url: String::new(),
                detail: format!("Accept-Encoding entry missing coding: {part:?}"),
            })?
            .to_ascii_lowercase();
        if coding.is_empty() {
            return Err(IndexError::ParseError {
                url: String::new(),
                detail: format!("Accept-Encoding entry has empty coding: {part:?}"),
            });
        }
        let mut q = 1.0f32;
        for param in params {
            if let Some(qv) = param.strip_prefix("q=").or_else(|| param.strip_prefix("Q=")) {
                q = qv.parse::<f32>().map_err(|_| IndexError::ParseError {
                    url: String::new(),
                    detail: format!("Accept-Encoding q-value not numeric: {qv:?}"),
                })?;
                if !(0.0..=1.0).contains(&q) {
                    return Err(IndexError::ParseError {
                        url: String::new(),
                        detail: format!("Accept-Encoding q-value out of range: {q}"),
                    });
                }
            }
            // Unknown parameters are ignored per RFC 9110 §12.5.3.
        }
        entries.push((idx, AcceptEncoding { coding, q }));
    }

    // Sort descending by q, breaking ties by declaration index.
    entries.sort_by(|a, b| {
        b.1.q.partial_cmp(&a.1.q).unwrap_or(std::cmp::Ordering::Equal)
            .then(a.0.cmp(&b.0))
    });
    Ok(entries.into_iter().map(|(_, e)| e).collect())
}

/// Parse a Content-Encoding response header. Returns the codings in
/// the order they were applied (outer-first per RFC 9110 §8.4). Empty
/// header returns an empty vec (interpreted as "identity / no coding").
pub fn parse_content_encoding(header: &str) -> Result<Vec<String>, IndexError> {
    let trimmed = header.trim();
    if trimmed.is_empty() {
        return Ok(Vec::new());
    }
    let mut out = Vec::new();
    for raw in trimmed.split(',') {
        let part = raw.trim();
        if part.is_empty() {
            return Err(IndexError::ParseError {
                url: String::new(),
                detail: format!("Content-Encoding has empty coding: {trimmed:?}"),
            });
        }
        if part.contains(';') || part.contains(' ') {
            return Err(IndexError::ParseError {
                url: String::new(),
                detail: format!(
                    "Content-Encoding coding must not contain parameters: {part:?}"
                ),
            });
        }
        out.push(part.to_ascii_lowercase());
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn err_detail(err: IndexError) -> String {
        match err {
            IndexError::ParseError { detail, .. } => detail,
            other => panic!("expected ParseError, got {other:?}"),
        }
    }

    // ---- build_accept_encoding ---------------------------------------

    #[test]
    fn build_omits_q_when_default() {
        let header = build_accept_encoding(&[AcceptEncoding::new("gzip")]);
        assert_eq!(header, "gzip");
    }

    #[test]
    fn build_emits_q_for_nondefault() {
        let header = build_accept_encoding(&[AcceptEncoding::with_q("gzip", 0.5)]);
        assert_eq!(header, "gzip; q=0.5");
    }

    #[test]
    fn build_lowercases_coding() {
        let header = build_accept_encoding(&[AcceptEncoding::new("GZIP")]);
        assert_eq!(header, "gzip");
    }

    #[test]
    fn build_joins_multiple_entries() {
        let header = build_accept_encoding(&[
            AcceptEncoding::new("br"),
            AcceptEncoding::with_q("gzip", 0.5),
            AcceptEncoding::with_q("identity", 0.0),
        ]);
        assert_eq!(header, "br, gzip; q=0.5, identity; q=0");
    }

    #[test]
    fn build_clamps_q_to_unit_interval() {
        let header = build_accept_encoding(&[
            AcceptEncoding::with_q("gzip", -1.0),
            AcceptEncoding::with_q("br", 2.0),
        ]);
        // -1 clamped to 0 → "gzip; q=0"; 2 clamped to 1 → "br" (omitted)
        assert_eq!(header, "gzip; q=0, br");
    }

    #[test]
    fn format_q_trims_trailing_zeros() {
        assert_eq!(format_q(0.5), "0.5");
        assert_eq!(format_q(0.25), "0.25");
        assert_eq!(format_q(0.125), "0.125");
        assert_eq!(format_q(1.0), "1");
        assert_eq!(format_q(0.0), "0");
    }

    // ---- parse_accept_encoding ---------------------------------------

    #[test]
    fn parse_single_coding() {
        let entries = parse_accept_encoding("gzip").unwrap();
        assert_eq!(entries, vec![AcceptEncoding::new("gzip")]);
    }

    #[test]
    fn parse_multiple_codings_sorts_by_q_desc() {
        let entries =
            parse_accept_encoding("gzip;q=0.5, br;q=1.0, identity;q=0").unwrap();
        assert_eq!(
            entries,
            vec![
                AcceptEncoding::with_q("br", 1.0),
                AcceptEncoding::with_q("gzip", 0.5),
                AcceptEncoding::with_q("identity", 0.0),
            ]
        );
    }

    #[test]
    fn parse_default_q_is_one() {
        let entries = parse_accept_encoding("gzip, br").unwrap();
        // Equal q (=1) preserves declaration order.
        assert_eq!(entries[0].coding, "gzip");
        assert_eq!(entries[1].coding, "br");
        assert!((entries[0].q - 1.0).abs() < 1e-6);
    }

    #[test]
    fn parse_case_insensitive_coding() {
        let entries = parse_accept_encoding("GZIP, Br;q=0.5").unwrap();
        assert_eq!(entries[0].coding, "gzip");
        assert_eq!(entries[1].coding, "br");
    }

    #[test]
    fn parse_case_insensitive_q_param() {
        // RFC 9110: parameter names are case-insensitive.
        let entries = parse_accept_encoding("gzip;Q=0.5").unwrap();
        assert!((entries[0].q - 0.5).abs() < 1e-6);
    }

    #[test]
    fn parse_empty_header_is_empty_vec() {
        assert!(parse_accept_encoding("").unwrap().is_empty());
        assert!(parse_accept_encoding("   ").unwrap().is_empty());
    }

    #[test]
    fn parse_tolerates_extra_whitespace() {
        let entries = parse_accept_encoding("   gzip  ;  q=0.5  ,  br  ").unwrap();
        // br first (q=1 default > 0.5)
        assert_eq!(entries[0].coding, "br");
        assert_eq!(entries[1].coding, "gzip");
    }

    #[test]
    fn parse_ignores_unknown_parameters() {
        // RFC 9110 §12.5.3: ignore unknown params.
        let entries = parse_accept_encoding("gzip; foo=bar; q=0.7").unwrap();
        assert_eq!(entries[0].coding, "gzip");
        assert!((entries[0].q - 0.7).abs() < 1e-6);
    }

    #[test]
    fn parse_rejects_non_numeric_q() {
        let err = parse_accept_encoding("gzip;q=high").unwrap_err();
        assert!(err_detail(err).contains("q-value not numeric"));
    }

    #[test]
    fn parse_rejects_q_out_of_range() {
        let err = parse_accept_encoding("gzip;q=2.0").unwrap_err();
        assert!(err_detail(err).contains("out of range"));
    }

    #[test]
    fn parse_rejects_empty_coding_in_entry() {
        let err = parse_accept_encoding(";q=0.5").unwrap_err();
        assert!(err_detail(err).contains("empty coding"));
    }

    #[test]
    fn parse_skips_empty_comma_entries() {
        // "gzip, , br" → ["gzip", "br"]
        let entries = parse_accept_encoding("gzip, , br").unwrap();
        assert_eq!(entries.len(), 2);
    }

    #[test]
    fn parse_wildcard_passes_through() {
        let entries = parse_accept_encoding("*;q=0.1").unwrap();
        assert_eq!(entries[0].coding, "*");
        assert!((entries[0].q - 0.1).abs() < 1e-6);
    }

    #[test]
    fn round_trip_via_parse_then_build() {
        let entries = parse_accept_encoding("br, gzip;q=0.5, identity;q=0").unwrap();
        let header = build_accept_encoding(&entries);
        assert_eq!(header, "br, gzip; q=0.5, identity; q=0");
    }

    // ---- parse_content_encoding --------------------------------------

    #[test]
    fn parse_content_single() {
        let codings = parse_content_encoding("gzip").unwrap();
        assert_eq!(codings, vec!["gzip".to_string()]);
    }

    #[test]
    fn parse_content_layered_outer_first() {
        let codings = parse_content_encoding("gzip, br").unwrap();
        // gzip was applied first, br applied second (outer).
        // Per RFC 9110 §8.4 the header is "outer-last first" — sorry,
        // it's: "If multiple encodings have been applied to the
        // representation, the content codings are listed in the order
        // in which they were applied." So gzip applied first means
        // gzip first; the receiver decodes in reverse (br, then gzip).
        assert_eq!(codings, vec!["gzip".to_string(), "br".to_string()]);
    }

    #[test]
    fn parse_content_empty_returns_empty() {
        assert!(parse_content_encoding("").unwrap().is_empty());
        assert!(parse_content_encoding("   ").unwrap().is_empty());
    }

    #[test]
    fn parse_content_lowercases_codings() {
        let codings = parse_content_encoding("GZIP, BR").unwrap();
        assert_eq!(codings, vec!["gzip".to_string(), "br".to_string()]);
    }

    #[test]
    fn parse_content_rejects_empty_inner_entry() {
        let err = parse_content_encoding("gzip, , br").unwrap_err();
        assert!(err_detail(err).contains("empty coding"));
    }

    #[test]
    fn parse_content_rejects_parameters() {
        let err = parse_content_encoding("gzip;q=0.5").unwrap_err();
        assert!(err_detail(err).contains("must not contain parameters"));
    }

    #[test]
    fn realistic_index_request_header() {
        let entries = vec![
            AcceptEncoding::new("br"),
            AcceptEncoding::new("gzip"),
            AcceptEncoding::with_q("identity", 0.0),
        ];
        let header = build_accept_encoding(&entries);
        assert_eq!(header, "br, gzip, identity; q=0");
    }

    #[test]
    fn realistic_index_response_header() {
        let codings = parse_content_encoding("br").unwrap();
        assert_eq!(codings, vec!["br".to_string()]);
    }

    #[test]
    fn is_rejected_helper() {
        assert!(AcceptEncoding::with_q("identity", 0.0).is_rejected());
        assert!(!AcceptEncoding::new("gzip").is_rejected());
    }
}
