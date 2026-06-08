// yanked.rs — PEP 592 file-level "yanked" marker.
//
// PEP 592 lets a publisher mark a release file as **yanked** without
// removing it from the index. Yanked files are visible to legacy
// clients (so reproducible installs keep working) but the resolver
// must treat them as last-resort: a yanked file may only be selected
// when the user pinned it via an *exact* `==` specifier; under any
// range or "latest" selection, a yanked file is invisible.
//
// PEP 592 + PEP 691 + PEP 503 give us three on-wire shapes:
//
//   * PEP 691 JSON     — `"yanked": true | false | "<reason>"` on
//                        each file record. `false` (or absent) means
//                        not yanked; `true` means yanked w/o reason;
//                        a string means yanked WITH that reason.
//   * PEP 503 HTML     — `data-yanked` attribute on the `<a>` tag.
//                        Attribute *present* (even with empty value)
//                        means yanked; attribute *value* is the
//                        reason. Attribute absent means not yanked.
//   * PEP 691 sentinel — `"yanked": ""` was briefly emitted by some
//                        indexes to mean "yanked, no reason". PEP
//                        treats it as equivalent to `true`.
//
// We unify those into a typed enum + two read-side parsers.

use serde::{Deserialize, Serialize};

use crate::pkgmanage::pkgmgr::types::IndexError;

/// Per-file yanked status.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(tag = "kind", content = "reason", rename_all = "snake_case")]
pub enum YankedStatus {
    /// Default — file is available.
    #[default]
    NotYanked,
    /// File is yanked. No reason was provided.
    Yanked,
    /// File is yanked with a publisher-supplied human-readable
    /// reason. The reason is intended for display to the user; it
    /// must not affect resolver decisions.
    YankedWithReason(String),
}

impl YankedStatus {
    /// True iff this file is yanked (with or without a reason).
    pub fn is_yanked(&self) -> bool {
        !matches!(self, YankedStatus::NotYanked)
    }

    /// The reason, if any. `Yanked` (no reason) and `NotYanked` both
    /// return `None`.
    pub fn reason(&self) -> Option<&str> {
        match self {
            YankedStatus::YankedWithReason(r) => Some(r.as_str()),
            _ => None,
        }
    }

    /// Resolver gate: should the resolver consider this file under
    /// `selection_mode`? Yanked files are eligible **only** when the
    /// user pinned an exact version (per PEP 592 §"Resolvers must
    /// not implicitly select a yanked file").
    pub fn is_selectable_under(&self, selection_mode: SelectionMode) -> bool {
        match (self, selection_mode) {
            (YankedStatus::NotYanked, _) => true,
            (_, SelectionMode::ExactPin) => true,
            _ => false,
        }
    }
}

/// How the resolver is choosing a file. Used to gate yanked
/// availability.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SelectionMode {
    /// The user's specifier is `==<exact-version>`; yanked files are
    /// eligible.
    ExactPin,
    /// Any non-exact selection: range, latest, "best wheel for this
    /// platform", etc. Yanked files must be skipped.
    Range,
}

/// Parse the `yanked` field of a PEP 691 JSON file record.
/// Accepts:
///   * absent / null / `false`        → NotYanked
///   * `true`                         → Yanked
///   * `""` (empty string)            → Yanked (legacy sentinel)
///   * `"<reason>"` (non-empty str)   → YankedWithReason(reason)
///
/// Any other JSON shape (number, array, object) is rejected with a
/// typed `ParseError` so the resolver can refuse the index.
pub fn parse_yanked_json(value: Option<&serde_json::Value>) -> Result<YankedStatus, IndexError> {
    match value {
        None | Some(serde_json::Value::Null) | Some(serde_json::Value::Bool(false)) => {
            Ok(YankedStatus::NotYanked)
        }
        Some(serde_json::Value::Bool(true)) => Ok(YankedStatus::Yanked),
        Some(serde_json::Value::String(s)) => {
            if s.is_empty() {
                Ok(YankedStatus::Yanked)
            } else {
                Ok(YankedStatus::YankedWithReason(s.clone()))
            }
        }
        Some(other) => Err(IndexError::ParseError {
            url: String::new(),
            detail: format!(
                "PEP 592 `yanked` must be bool or string, got {}",
                json_kind(other)
            ),
        }),
    }
}

/// Parse a PEP 503 HTML `data-yanked` attribute value.
///
/// Per the PEP 503 yanked addendum:
///   * Attribute absent (`None`)           → NotYanked
///   * Attribute present, empty / whitespace → Yanked
///   * Attribute present, non-empty value  → YankedWithReason(value)
///
/// Whitespace-only values collapse to `Yanked` (no reason) because
/// PyPI's HTML escapes empty attribute values inconsistently and
/// "yanked because: <space>" is never what a publisher means.
pub fn parse_yanked_html_attr(attr_value: Option<&str>) -> YankedStatus {
    match attr_value {
        None => YankedStatus::NotYanked,
        Some(s) if s.trim().is_empty() => YankedStatus::Yanked,
        Some(s) => YankedStatus::YankedWithReason(s.to_string()),
    }
}

fn json_kind(v: &serde_json::Value) -> &'static str {
    match v {
        serde_json::Value::Null => "null",
        serde_json::Value::Bool(_) => "bool",
        serde_json::Value::Number(_) => "number",
        serde_json::Value::String(_) => "string",
        serde_json::Value::Array(_) => "array",
        serde_json::Value::Object(_) => "object",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn err_detail(err: IndexError) -> String {
        match err {
            IndexError::ParseError { detail, .. } => detail,
            other => panic!("expected ParseError, got {other:?}"),
        }
    }

    // ---- parse_yanked_json --------------------------------------------

    #[test]
    fn json_absent_is_not_yanked() {
        assert_eq!(parse_yanked_json(None).unwrap(), YankedStatus::NotYanked);
    }

    #[test]
    fn json_null_is_not_yanked() {
        assert_eq!(
            parse_yanked_json(Some(&json!(null))).unwrap(),
            YankedStatus::NotYanked
        );
    }

    #[test]
    fn json_false_is_not_yanked() {
        assert_eq!(
            parse_yanked_json(Some(&json!(false))).unwrap(),
            YankedStatus::NotYanked
        );
    }

    #[test]
    fn json_true_is_yanked() {
        assert_eq!(
            parse_yanked_json(Some(&json!(true))).unwrap(),
            YankedStatus::Yanked
        );
    }

    #[test]
    fn json_empty_string_is_yanked_no_reason() {
        // Legacy sentinel — PEP treats `""` as equivalent to `true`.
        assert_eq!(
            parse_yanked_json(Some(&json!(""))).unwrap(),
            YankedStatus::Yanked
        );
    }

    #[test]
    fn json_non_empty_string_is_yanked_with_reason() {
        let v = json!("Critical CVE-2024-12345");
        let s = parse_yanked_json(Some(&v)).unwrap();
        match s {
            YankedStatus::YankedWithReason(r) => assert_eq!(r, "Critical CVE-2024-12345"),
            other => panic!("expected YankedWithReason, got {other:?}"),
        }
    }

    #[test]
    fn json_number_is_rejected() {
        let err = parse_yanked_json(Some(&json!(1))).unwrap_err();
        let detail = err_detail(err);
        assert!(detail.contains("must be bool or string"));
        assert!(detail.contains("number"));
    }

    #[test]
    fn json_array_is_rejected() {
        let err = parse_yanked_json(Some(&json!([1, 2]))).unwrap_err();
        assert!(err_detail(err).contains("array"));
    }

    #[test]
    fn json_object_is_rejected() {
        let err = parse_yanked_json(Some(&json!({"reason": "x"}))).unwrap_err();
        assert!(err_detail(err).contains("object"));
    }

    // ---- parse_yanked_html_attr ---------------------------------------

    #[test]
    fn html_attr_absent_is_not_yanked() {
        assert_eq!(parse_yanked_html_attr(None), YankedStatus::NotYanked);
    }

    #[test]
    fn html_attr_empty_string_is_yanked_no_reason() {
        assert_eq!(parse_yanked_html_attr(Some("")), YankedStatus::Yanked);
    }

    #[test]
    fn html_attr_whitespace_only_is_yanked_no_reason() {
        // PyPI's HTML occasionally emits `data-yanked="  "` for the
        // legacy "yanked but no reason" case.
        assert_eq!(parse_yanked_html_attr(Some("  \t  ")), YankedStatus::Yanked);
    }

    #[test]
    fn html_attr_with_reason() {
        let s = parse_yanked_html_attr(Some("CVE-2024-12345"));
        assert_eq!(s, YankedStatus::YankedWithReason("CVE-2024-12345".into()));
    }

    #[test]
    fn html_attr_leading_whitespace_in_reason_preserved() {
        // The HTML attribute parser should not normalize the reason
        // beyond detecting the empty-marker case; downstream display
        // can trim if it wants.
        let s = parse_yanked_html_attr(Some("  why  "));
        assert_eq!(s, YankedStatus::YankedWithReason("  why  ".into()));
    }

    // ---- YankedStatus helpers -----------------------------------------

    #[test]
    fn is_yanked_for_each_variant() {
        assert!(!YankedStatus::NotYanked.is_yanked());
        assert!(YankedStatus::Yanked.is_yanked());
        assert!(YankedStatus::YankedWithReason("x".into()).is_yanked());
    }

    #[test]
    fn reason_only_for_with_reason_variant() {
        assert_eq!(YankedStatus::NotYanked.reason(), None);
        assert_eq!(YankedStatus::Yanked.reason(), None);
        assert_eq!(
            YankedStatus::YankedWithReason("CVE".into()).reason(),
            Some("CVE")
        );
    }

    #[test]
    fn default_is_not_yanked() {
        assert_eq!(YankedStatus::default(), YankedStatus::NotYanked);
    }

    // ---- is_selectable_under (resolver gate) --------------------------

    #[test]
    fn not_yanked_is_always_selectable() {
        assert!(YankedStatus::NotYanked.is_selectable_under(SelectionMode::Range));
        assert!(YankedStatus::NotYanked.is_selectable_under(SelectionMode::ExactPin));
    }

    #[test]
    fn yanked_is_only_selectable_under_exact_pin() {
        assert!(!YankedStatus::Yanked.is_selectable_under(SelectionMode::Range));
        assert!(YankedStatus::Yanked.is_selectable_under(SelectionMode::ExactPin));
    }

    #[test]
    fn yanked_with_reason_follows_same_rule() {
        let s = YankedStatus::YankedWithReason("x".into());
        assert!(!s.is_selectable_under(SelectionMode::Range));
        assert!(s.is_selectable_under(SelectionMode::ExactPin));
    }

    // ---- realistic mixed workflow -------------------------------------

    #[test]
    fn realistic_pypi_file_record_workflow() {
        // 1. Parse the JSON file record.
        let file_record = json!({
            "filename": "broken-1.0-py3-none-any.whl",
            "url": "https://pypi.example/broken-1.0-py3-none-any.whl",
            "hashes": {"sha256": "abc"},
            "yanked": "Security: CVE-2024-99999 (use 1.0.1 instead)"
        });
        let yanked_field = file_record.get("yanked");
        let status = parse_yanked_json(yanked_field).unwrap();

        assert!(status.is_yanked());
        assert_eq!(
            status.reason(),
            Some("Security: CVE-2024-99999 (use 1.0.1 instead)")
        );

        // 2. Resolver under "latest" → skip this file.
        assert!(!status.is_selectable_under(SelectionMode::Range));

        // 3. User pinned `broken == 1.0` exactly → file IS eligible.
        assert!(status.is_selectable_under(SelectionMode::ExactPin));
    }

    // ---- serde round-trip ---------------------------------------------

    #[test]
    fn serde_roundtrip_each_variant() {
        for s in [
            YankedStatus::NotYanked,
            YankedStatus::Yanked,
            YankedStatus::YankedWithReason("CVE".into()),
        ] {
            let serialized = serde_json::to_string(&s).unwrap();
            let back: YankedStatus = serde_json::from_str(&serialized).unwrap();
            assert_eq!(back, s);
        }
    }
}
