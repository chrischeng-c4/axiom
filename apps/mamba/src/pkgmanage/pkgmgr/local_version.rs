// PEP 440 local-version-label parser (Tick 117).
//
// PEP 440 §"Local version identifiers" allows a `+<local>` suffix on a
// public version string. The label is a sequence of dot-separated
// segments where each segment is either all-digits (numeric) or
// alphanumeric. Examples:
//
//     1.2.3+cuda12.cu118
//     2.0.0+cpu
//     1.5.0+abi3.musllinux
//
// The existing `pep440` comparator in this crate strips the `+local`
// suffix and ignores it. That's correct for resolution against PyPI —
// public indexes do not host distinct artifacts that differ only in
// local label. However, **direct-URL pins** (e.g. PyTorch CUDA wheels
// at download.pytorch.org/whl/cu121/) absolutely do distinguish wheels
// by local label, and the resolver needs to compare them when both
// candidates carry one. This module supplies the typed comparator
// without touching the existing `pep440::Pep440Version` semantics.
//
// PEP 440 §"Local version identifiers" comparison rule (paraphrased):
//
//   * A version with NO local label sorts BELOW the same version WITH one.
//   * Between two local labels, compare segment-by-segment:
//       - numeric vs numeric: integer order
//       - alphanumeric vs alphanumeric: byte-wise lex order
//       - numeric vs alphanumeric: numeric sorts ABOVE alphanumeric
//   * Shorter label sorts below a longer one when all shared segments
//     compare equal.

use std::cmp::Ordering;

use crate::pkgmanage::pkgmgr::types::IndexError;

/// One dot-separated segment of a local version label.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LocalSegment {
    Numeric(u64),
    Alphanumeric(String),
}

/// A parsed local version label (the part after `+`).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct LocalVersionLabel {
    /// Original label text exactly as written (lowercased per PEP 440).
    pub raw: String,
    /// Segments in order; comparison uses PEP 440 §"Local version
    /// identifiers" rules.
    pub segments: Vec<LocalSegment>,
}

impl LocalVersionLabel {
    /// Parse a label string (without the leading `+`). The empty string
    /// and a string consisting only of separators are rejected.
    pub fn parse(label: &str) -> Result<Self, IndexError> {
        if label.is_empty() {
            return Err(parse_err("local version label is empty"));
        }
        let lowered = label.to_ascii_lowercase();
        let mut segments = Vec::new();
        for part in lowered.split('.') {
            if part.is_empty() {
                return Err(parse_err(
                    "local version label has an empty segment (leading, trailing, or doubled `.`)",
                ));
            }
            // PEP 440 forbids any non-alphanumeric character inside a
            // local segment. The dot is the only legal separator.
            if !part.bytes().all(|b| b.is_ascii_alphanumeric()) {
                return Err(parse_err(format!(
                    "local version segment `{part}` contains a non-alphanumeric character"
                )));
            }
            // All-digit segments become Numeric; otherwise Alphanumeric.
            if part.bytes().all(|b| b.is_ascii_digit()) {
                let n: u64 = part.parse().map_err(|_| {
                    parse_err(format!("local version segment `{part}` overflows u64"))
                })?;
                segments.push(LocalSegment::Numeric(n));
            } else {
                segments.push(LocalSegment::Alphanumeric(part.to_string()));
            }
        }
        Ok(Self {
            raw: lowered,
            segments,
        })
    }

    /// Pull the `+local` suffix off a full PEP 440 string. Returns
    /// `(public_part, Some(label))` when present, `(input, None)` when
    /// no `+` is present. Errors when the label is empty or malformed.
    pub fn split_from_version(version: &str) -> Result<(&str, Option<Self>), IndexError> {
        match version.split_once('+') {
            None => Ok((version, None)),
            Some((public, label)) => {
                let parsed = Self::parse(label)?;
                Ok((public, Some(parsed)))
            }
        }
    }

    /// True when no segments are present (only possible via
    /// `LocalVersionLabel::default()`, since `parse` rejects empty).
    pub fn is_empty(&self) -> bool {
        self.segments.is_empty()
    }
}

fn parse_err(detail: impl Into<String>) -> IndexError {
    IndexError::ParseError {
        url: "<local-version>".to_string(),
        detail: detail.into(),
    }
}

impl Ord for LocalVersionLabel {
    fn cmp(&self, other: &Self) -> Ordering {
        // Segment-by-segment comparison up to the shared length.
        for (a, b) in self.segments.iter().zip(other.segments.iter()) {
            match (a, b) {
                (LocalSegment::Numeric(x), LocalSegment::Numeric(y)) => match x.cmp(y) {
                    Ordering::Equal => continue,
                    other => return other,
                },
                (LocalSegment::Alphanumeric(x), LocalSegment::Alphanumeric(y)) => match x.cmp(y) {
                    Ordering::Equal => continue,
                    other => return other,
                },
                // PEP 440: numeric sorts ABOVE alphanumeric.
                (LocalSegment::Numeric(_), LocalSegment::Alphanumeric(_)) => {
                    return Ordering::Greater
                }
                (LocalSegment::Alphanumeric(_), LocalSegment::Numeric(_)) => return Ordering::Less,
            }
        }
        // Tie among shared prefix — shorter sorts below longer.
        self.segments.len().cmp(&other.segments.len())
    }
}

impl PartialOrd for LocalVersionLabel {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// Compare two PEP 440 version strings with full local-label semantics.
/// The public part is compared by the existing `pep440::parse` ordering;
/// the local label tie-breaker applies only when the public parts are
/// equal. A version with no local label sorts BELOW the same public
/// version with one.
pub fn cmp_with_local(a: &str, b: &str) -> Result<Ordering, IndexError> {
    let (a_pub, a_loc) = LocalVersionLabel::split_from_version(a)?;
    let (b_pub, b_loc) = LocalVersionLabel::split_from_version(b)?;
    let pa = crate::pkgmanage::pkgmgr::pep440::parse(a_pub)
        .ok_or_else(|| parse_err(format!("public part `{a_pub}` is not PEP 440 parseable")))?;
    let pb = crate::pkgmanage::pkgmgr::pep440::parse(b_pub)
        .ok_or_else(|| parse_err(format!("public part `{b_pub}` is not PEP 440 parseable")))?;
    match pa.cmp(&pb) {
        Ordering::Equal => Ok(match (a_loc, b_loc) {
            (None, None) => Ordering::Equal,
            (None, Some(_)) => Ordering::Less,
            (Some(_), None) => Ordering::Greater,
            (Some(la), Some(lb)) => la.cmp(&lb),
        }),
        ord => Ok(ord),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn label(s: &str) -> LocalVersionLabel {
        LocalVersionLabel::parse(s).unwrap_or_else(|e| panic!("parse {s:?}: {e:?}"))
    }

    #[test]
    fn parses_single_numeric_segment() {
        let l = label("1");
        assert_eq!(l.segments, vec![LocalSegment::Numeric(1)]);
    }

    #[test]
    fn parses_single_alphanumeric_segment() {
        let l = label("cu118");
        assert_eq!(l.segments, vec![LocalSegment::Alphanumeric("cu118".into())]);
    }

    #[test]
    fn parses_dot_separated_segments() {
        let l = label("cuda12.cu118");
        assert_eq!(
            l.segments,
            vec![
                LocalSegment::Alphanumeric("cuda12".into()),
                LocalSegment::Alphanumeric("cu118".into()),
            ]
        );
    }

    #[test]
    fn lowercases_input() {
        let l = label("CUDA12.CU118");
        assert_eq!(l.raw, "cuda12.cu118");
    }

    #[test]
    fn rejects_empty_string() {
        let err = LocalVersionLabel::parse("").unwrap_err();
        let msg = format!("{err:?}");
        assert!(msg.contains("empty"));
    }

    #[test]
    fn rejects_leading_dot() {
        assert!(LocalVersionLabel::parse(".cu118").is_err());
    }

    #[test]
    fn rejects_trailing_dot() {
        assert!(LocalVersionLabel::parse("cu118.").is_err());
    }

    #[test]
    fn rejects_doubled_dot() {
        assert!(LocalVersionLabel::parse("cuda..118").is_err());
    }

    #[test]
    fn rejects_non_alphanumeric() {
        assert!(LocalVersionLabel::parse("cu-118").is_err());
        assert!(LocalVersionLabel::parse("cu_118").is_err());
        assert!(LocalVersionLabel::parse("cu+118").is_err());
    }

    #[test]
    fn split_from_version_no_local() {
        let (pub_part, loc) = LocalVersionLabel::split_from_version("1.2.3").unwrap();
        assert_eq!(pub_part, "1.2.3");
        assert!(loc.is_none());
    }

    #[test]
    fn split_from_version_with_local() {
        let (pub_part, loc) = LocalVersionLabel::split_from_version("1.2.3+cu118").unwrap();
        assert_eq!(pub_part, "1.2.3");
        let l = loc.unwrap();
        assert_eq!(l.raw, "cu118");
        assert_eq!(l.segments, vec![LocalSegment::Alphanumeric("cu118".into())]);
    }

    #[test]
    fn split_from_version_rejects_empty_label() {
        // `1.2.3+` is malformed per PEP 440.
        assert!(LocalVersionLabel::split_from_version("1.2.3+").is_err());
    }

    #[test]
    fn ord_numeric_vs_numeric() {
        assert!(label("1") < label("2"));
        assert!(label("10") > label("9"));
    }

    #[test]
    fn ord_alpha_vs_alpha() {
        assert!(label("cu118") < label("cu121"));
        assert!(label("cpu") < label("cuda"));
    }

    #[test]
    fn ord_numeric_above_alphanumeric() {
        // PEP 440: within local labels, numeric sorts ABOVE alphanumeric.
        assert!(label("1") > label("a"));
        assert!(label("0") > label("zzz"));
    }

    #[test]
    fn ord_shorter_below_longer() {
        // Same shared prefix, the shorter sorts below.
        assert!(label("cu118") < label("cu118.0"));
    }

    #[test]
    fn cmp_with_local_no_labels_either_side() {
        assert_eq!(cmp_with_local("1.2.3", "1.2.3").unwrap(), Ordering::Equal);
        assert_eq!(cmp_with_local("1.2.3", "1.2.4").unwrap(), Ordering::Less);
    }

    #[test]
    fn cmp_with_local_label_only_on_one_side() {
        // No-local sorts below with-local at the same public version.
        assert_eq!(
            cmp_with_local("1.2.3", "1.2.3+cu118").unwrap(),
            Ordering::Less
        );
        assert_eq!(
            cmp_with_local("1.2.3+cu118", "1.2.3").unwrap(),
            Ordering::Greater
        );
    }

    #[test]
    fn cmp_with_local_both_sides_have_labels() {
        // cu118 < cu121, so the full versions sort that way too.
        assert_eq!(
            cmp_with_local("1.2.3+cu118", "1.2.3+cu121").unwrap(),
            Ordering::Less
        );
    }

    #[test]
    fn cmp_with_local_public_wins_over_label() {
        // Public 1.2.4 beats 1.2.3 regardless of label.
        assert_eq!(
            cmp_with_local("1.2.3+zzz", "1.2.4+aaa").unwrap(),
            Ordering::Less
        );
    }

    #[test]
    fn cmp_with_local_errors_on_malformed_public() {
        assert!(cmp_with_local("not.a.version+cu118", "1.2.3").is_err());
    }

    #[test]
    fn realistic_pytorch_cuda_labels() {
        // PyTorch publishes wheels at /whl/cu118/, /whl/cu121/, /whl/cu124/
        // — uv must sort these by their local labels when pinning.
        let order: Vec<&str> = vec!["2.1.0+cpu", "2.1.0+cu118", "2.1.0+cu121", "2.1.0+cu124"];
        for win in order.windows(2) {
            assert_eq!(
                cmp_with_local(win[0], win[1]).unwrap(),
                Ordering::Less,
                "expected {} < {}",
                win[0],
                win[1]
            );
        }
    }

    #[test]
    fn realistic_mixed_numeric_alpha_segments() {
        // The "cuda12.118" mix exercises both segment kinds in one label.
        assert!(label("cuda12.118") > label("cuda12.cu118"));
        // Same shared first segment (alpha), then numeric > alpha tie-break.
    }
}
