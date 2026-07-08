/// Strip PEP 515 digit-separator underscores from `s` after validating
/// that placement is legal: no leading/trailing underscore, no
/// consecutive underscores. When `allow_leading` is true (used after
/// stripping a radix prefix like `0x`), an underscore may immediately
/// follow the prefix (`0x_FF` → caller passes `_FF`). Returns `None`
/// when placement is invalid.
pub(crate) fn strip_pep515_underscores(s: &str, allow_leading: bool) -> Option<String> {
    if s.is_empty() {
        return None;
    }
    if !allow_leading && s.starts_with('_') {
        return None;
    }
    if s.ends_with('_') {
        return None;
    }
    if s.contains("__") {
        return None;
    }
    Some(s.replace('_', ""))
}

/// Strip PEP 515 underscores from a float literal string. Underscores
/// may appear between digits in the integer part, fractional part, or
/// exponent — but never adjacent to `.`/`e`/`E`/sign characters, and
/// never leading/trailing in any run. Returns `None` if any rule is
/// violated.
pub(crate) fn strip_float_underscores(s: &str) -> Option<String> {
    // Forbidden adjacencies: `_` next to `.`, `e`/`E`, or sign.
    let bytes = s.as_bytes();
    for i in 0..bytes.len() {
        if bytes[i] == b'_' {
            let prev = if i == 0 { None } else { Some(bytes[i - 1]) };
            let next = bytes.get(i + 1).copied();
            let forbidden = |c: Option<u8>| {
                matches!(
                    c,
                    None | Some(b'.')
                        | Some(b'e')
                        | Some(b'E')
                        | Some(b'+')
                        | Some(b'-')
                        | Some(b'_')
                )
            };
            if forbidden(prev) || forbidden(next) {
                return None;
            }
        }
    }
    Some(s.replace('_', ""))
}

/// Parse the textual form of a float as CPython's `float(str)` / `float(bytes)`
/// does: surrounding whitespace ignored, case-insensitive inf/infinity/nan,
/// PEP 515 underscores. Returns None when the text is not a valid float literal.
pub(crate) fn parse_pyfloat_text(text: &str) -> Option<f64> {
    let trimmed = text.trim();
    let lower = trimmed.to_ascii_lowercase();
    if lower == "inf" || lower == "infinity" {
        return Some(f64::INFINITY);
    }
    if lower == "-inf" || lower == "-infinity" {
        return Some(f64::NEG_INFINITY);
    }
    if lower == "nan" || lower == "-nan" {
        return Some(f64::NAN);
    }
    if let Ok(f) = trimmed.parse::<f64>() {
        return Some(f);
    }
    // PEP 515: `1_000.5`, `2_500e-3`, etc. Validate underscore placement, then
    // strip and re-parse.
    if let Some(without) = strip_float_underscores(trimmed) {
        if let Ok(f) = without.parse::<f64>() {
            return Some(f);
        }
    }
    None
}
