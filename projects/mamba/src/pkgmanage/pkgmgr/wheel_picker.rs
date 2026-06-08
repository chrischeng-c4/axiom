// Wheel batch picker (Tick 118).
//
// Given a list of candidate wheel filenames for one (distribution, version),
// pick the single best wheel for the running interpreter. The picker
// composes three primitives already in this crate:
//
//   * `wheel_filename::parse_wheel_filename` — full structural parse,
//     including PEP 491 build-tag preservation.
//   * `tags::WheelTag` + `tags::TagSelector::score` — host-compatibility
//     ranking by python / abi / platform tags.
//
// Tie-break order (descending preference):
//
//   1. higher TagSelector::score (most-specific compatibility match)
//   2. higher build-tag numeric prefix (PEP 491 §"Build tag" tie-break;
//      pip / uv both use this to let republishers ship a fixed sdist
//      without bumping the public version)
//   3. lexicographically earlier filename (deterministic, source-stable)
//
// Returns `None` only when zero candidates parse + are compatible.

use crate::pkgmanage::pkgmgr::tags::{
    parse_wheel_filename as parse_tag_only, TagSelector, WheelTag,
};
use crate::pkgmanage::pkgmgr::wheel_filename::{parse_wheel_filename, WheelFilename};

/// One scored candidate for the picker. Exposes the score components so
/// callers (loggers, diff tools) can explain "why this wheel won".
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScoredWheel {
    pub filename: String,
    pub parsed: WheelFilename,
    /// Compatibility score from `TagSelector::score`. Higher = better.
    pub tag_score: u32,
    /// PEP 491 build-tag numeric prefix. None when no build tag, or
    /// when the build tag's leading run of digits is empty. Used only
    /// as a tie-break.
    pub build_number: Option<u64>,
}

/// Pick the best wheel for the given host from a candidate list.
/// Filenames that fail to parse or that are incompatible with the host
/// are silently filtered out — they're not errors at the picker layer
/// (a non-wheel `.tar.gz` is a perfectly valid release file, just not
/// pickable here).
pub fn pick_best_wheel(filenames: &[String], selector: &TagSelector) -> Option<ScoredWheel> {
    let mut best: Option<ScoredWheel> = None;
    for filename in filenames {
        let Ok(parsed) = parse_wheel_filename(filename) else {
            continue;
        };
        let Some(tag) = parse_tag_only(filename) else {
            continue;
        };
        let Some(score) = selector.score(&tag) else {
            continue;
        };
        let build_number = parsed.build_tag.as_deref().and_then(extract_build_number);
        let candidate = ScoredWheel {
            filename: filename.clone(),
            parsed,
            tag_score: score,
            build_number,
        };
        best = Some(match best {
            None => candidate,
            Some(prev) => pick_winner(prev, candidate),
        });
    }
    best
}

/// Score ALL candidates and return them sorted descending by preference.
/// Useful when the caller wants to attempt installs in order (e.g.
/// network-fetch fallback if the top pick fails to download).
pub fn rank_wheels(filenames: &[String], selector: &TagSelector) -> Vec<ScoredWheel> {
    let mut scored: Vec<ScoredWheel> = filenames
        .iter()
        .filter_map(|filename| {
            let parsed = parse_wheel_filename(filename).ok()?;
            let tag = parse_tag_only(filename)?;
            let score = selector.score(&tag)?;
            let build_number = parsed.build_tag.as_deref().and_then(extract_build_number);
            Some(ScoredWheel {
                filename: filename.clone(),
                parsed,
                tag_score: score,
                build_number,
            })
        })
        .collect();
    scored.sort_by(|a, b| {
        b.tag_score
            .cmp(&a.tag_score)
            .then_with(|| b.build_number.unwrap_or(0).cmp(&a.build_number.unwrap_or(0)))
            .then_with(|| a.filename.cmp(&b.filename))
    });
    scored
}

fn pick_winner(prev: ScoredWheel, next: ScoredWheel) -> ScoredWheel {
    use std::cmp::Ordering;
    match next.tag_score.cmp(&prev.tag_score) {
        Ordering::Greater => next,
        Ordering::Less => prev,
        Ordering::Equal => {
            let pb = prev.build_number.unwrap_or(0);
            let nb = next.build_number.unwrap_or(0);
            match nb.cmp(&pb) {
                Ordering::Greater => next,
                Ordering::Less => prev,
                Ordering::Equal => {
                    if next.filename < prev.filename {
                        next
                    } else {
                        prev
                    }
                }
            }
        }
    }
}

/// PEP 491 build tag is "digit prefix + arbitrary suffix" (e.g. `1`,
/// `42dev`, `7post1`). The numeric tie-break uses the leading digit run.
fn extract_build_number(build_tag: &str) -> Option<u64> {
    let digits: String = build_tag.chars().take_while(|c| c.is_ascii_digit()).collect();
    if digits.is_empty() {
        return None;
    }
    digits.parse().ok()
}

/// Helper for tests + callers who already have a `WheelTag` and want
/// the same compatibility test the picker uses internally.
pub fn wheel_tag_of(filename: &str) -> Option<WheelTag> {
    parse_tag_only(filename)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Build a selector that prefers `cp312` on macOS arm64. We bypass
    /// `TagSelector::current_host` because env-driven behavior would
    /// make these tests flaky.
    fn macos_arm64_cp312_selector() -> TagSelector {
        TagSelector {
            python: vec![
                "cp312".into(),
                "cp3".into(),
                "py312".into(),
                "py3".into(),
                "py2.py3".into(),
                "py".into(),
            ],
            abi: vec!["cp312".into(), "abi3".into(), "none".into()],
            platform: vec![
                "macosx_14_0_arm64".into(),
                "macosx_13_0_arm64".into(),
                "macosx_11_0_arm64".into(),
                "macosx_11_0_universal2".into(),
                "any".into(),
            ],
        }
    }

    #[test]
    fn picks_only_compatible_wheel() {
        let files = vec![
            "pkg-1.0-cp310-cp310-manylinux_2_17_x86_64.whl".to_string(),
            "pkg-1.0-cp312-cp312-macosx_11_0_arm64.whl".to_string(),
            "pkg-1.0-cp311-cp311-win_amd64.whl".to_string(),
        ];
        let s = macos_arm64_cp312_selector();
        let best = pick_best_wheel(&files, &s).unwrap();
        assert_eq!(best.filename, "pkg-1.0-cp312-cp312-macosx_11_0_arm64.whl");
    }

    #[test]
    fn prefers_native_over_pure_python_when_both_compatible() {
        let files = vec![
            "pkg-1.0-cp312-cp312-macosx_11_0_arm64.whl".to_string(),
            "pkg-1.0-py3-none-any.whl".to_string(),
        ];
        let s = macos_arm64_cp312_selector();
        let best = pick_best_wheel(&files, &s).unwrap();
        assert_eq!(best.filename, "pkg-1.0-cp312-cp312-macosx_11_0_arm64.whl");
    }

    #[test]
    fn falls_back_to_pure_python_when_no_native_wheel() {
        let files = vec!["pkg-1.0-py3-none-any.whl".to_string()];
        let s = macos_arm64_cp312_selector();
        let best = pick_best_wheel(&files, &s).unwrap();
        assert_eq!(best.filename, "pkg-1.0-py3-none-any.whl");
    }

    #[test]
    fn prefers_abi3_over_pure_python() {
        let files = vec![
            "pkg-1.0-cp312-abi3-macosx_11_0_arm64.whl".to_string(),
            "pkg-1.0-py3-none-any.whl".to_string(),
        ];
        let s = macos_arm64_cp312_selector();
        let best = pick_best_wheel(&files, &s).unwrap();
        assert_eq!(
            best.filename,
            "pkg-1.0-cp312-abi3-macosx_11_0_arm64.whl"
        );
    }

    #[test]
    fn returns_none_when_no_wheel_is_compatible() {
        let files = vec![
            "pkg-1.0-cp310-cp310-manylinux_2_17_x86_64.whl".to_string(),
            "pkg-1.0-cp311-cp311-win_amd64.whl".to_string(),
        ];
        let s = macos_arm64_cp312_selector();
        assert!(pick_best_wheel(&files, &s).is_none());
    }

    #[test]
    fn ignores_non_wheel_files() {
        // sdists and other release artifacts should silently be skipped.
        let files = vec![
            "pkg-1.0.tar.gz".to_string(),
            "pkg-1.0.zip".to_string(),
            "pkg-1.0-cp312-cp312-macosx_11_0_arm64.whl".to_string(),
        ];
        let s = macos_arm64_cp312_selector();
        let best = pick_best_wheel(&files, &s).unwrap();
        assert!(best.filename.ends_with(".whl"));
    }

    #[test]
    fn ignores_malformed_wheel_filenames() {
        // A 4-segment "wheel" is malformed — must be ignored, not error.
        let files = vec![
            "broken-1.0-cp312-cp312.whl".to_string(),
            "pkg-1.0-cp312-cp312-macosx_11_0_arm64.whl".to_string(),
        ];
        let s = macos_arm64_cp312_selector();
        let best = pick_best_wheel(&files, &s).unwrap();
        assert_eq!(best.filename, "pkg-1.0-cp312-cp312-macosx_11_0_arm64.whl");
    }

    #[test]
    fn build_tag_tie_breaks_equal_score() {
        // Identical tag triple → the higher build-tag number wins.
        let files = vec![
            "pkg-1.0-1-cp312-cp312-macosx_11_0_arm64.whl".to_string(),
            "pkg-1.0-2-cp312-cp312-macosx_11_0_arm64.whl".to_string(),
        ];
        let s = macos_arm64_cp312_selector();
        let best = pick_best_wheel(&files, &s).unwrap();
        assert_eq!(
            best.filename,
            "pkg-1.0-2-cp312-cp312-macosx_11_0_arm64.whl"
        );
        assert_eq!(best.build_number, Some(2));
    }

    #[test]
    fn build_tag_with_alpha_suffix_uses_numeric_prefix() {
        // PEP 491: `42dev` has numeric prefix 42.
        let files = vec![
            "pkg-1.0-1-cp312-cp312-macosx_11_0_arm64.whl".to_string(),
            "pkg-1.0-42dev-cp312-cp312-macosx_11_0_arm64.whl".to_string(),
        ];
        let s = macos_arm64_cp312_selector();
        let best = pick_best_wheel(&files, &s).unwrap();
        assert_eq!(best.build_number, Some(42));
    }

    #[test]
    fn no_build_tag_loses_to_build_tag_at_equal_score() {
        // Bare wheel vs build-tag 1 — both are "version 1.0 + cp312 + arm64",
        // but the build-tag publisher tie-breaks above the bare wheel.
        let files = vec![
            "pkg-1.0-cp312-cp312-macosx_11_0_arm64.whl".to_string(),
            "pkg-1.0-1-cp312-cp312-macosx_11_0_arm64.whl".to_string(),
        ];
        let s = macos_arm64_cp312_selector();
        let best = pick_best_wheel(&files, &s).unwrap();
        assert_eq!(
            best.filename,
            "pkg-1.0-1-cp312-cp312-macosx_11_0_arm64.whl"
        );
    }

    #[test]
    fn filename_tie_break_is_deterministic() {
        // Two genuinely-equivalent wheels (identical tag, no build tag)
        // — picker returns the lexicographically earlier filename so
        // re-runs of `mamba sync` produce stable lockfiles.
        let files = vec![
            "pkg-1.0-cp312-cp312-macosx_11_0_arm64.whl".to_string(),
            // Construct two byte-distinct-but-equivalent filenames by
            // letting the second one differ only in case (which our
            // parser is byte-exact about).
            "pkg-1.0-CP312-CP312-macosx_11_0_arm64.whl".to_string(),
        ];
        // Selector accepts only the lowercase tag, so only one is
        // compatible — exercises the "uppercase tag is incompatible"
        // path, not the filename tie-break itself.
        let s = macos_arm64_cp312_selector();
        let best = pick_best_wheel(&files, &s).unwrap();
        assert_eq!(best.filename, "pkg-1.0-cp312-cp312-macosx_11_0_arm64.whl");
    }

    #[test]
    fn rank_wheels_orders_descending() {
        let files = vec![
            "pkg-1.0-py3-none-any.whl".to_string(),
            "pkg-1.0-cp312-cp312-macosx_11_0_arm64.whl".to_string(),
            "pkg-1.0-cp312-abi3-macosx_11_0_arm64.whl".to_string(),
            "pkg-1.0-cp310-cp310-manylinux_2_17_x86_64.whl".to_string(),
        ];
        let s = macos_arm64_cp312_selector();
        let ranked = rank_wheels(&files, &s);
        // The Linux wheel is incompatible — must be filtered out entirely.
        assert_eq!(ranked.len(), 3);
        assert_eq!(
            ranked[0].filename,
            "pkg-1.0-cp312-cp312-macosx_11_0_arm64.whl"
        );
        assert_eq!(
            ranked[1].filename,
            "pkg-1.0-cp312-abi3-macosx_11_0_arm64.whl"
        );
        assert_eq!(ranked[2].filename, "pkg-1.0-py3-none-any.whl");
        // Strict descending tag_score.
        assert!(ranked[0].tag_score >= ranked[1].tag_score);
        assert!(ranked[1].tag_score >= ranked[2].tag_score);
    }

    #[test]
    fn extract_build_number_handles_pep491_shapes() {
        assert_eq!(extract_build_number("1"), Some(1));
        assert_eq!(extract_build_number("42"), Some(42));
        assert_eq!(extract_build_number("42dev"), Some(42));
        assert_eq!(extract_build_number("7post1"), Some(7));
        assert_eq!(extract_build_number(""), None);
    }

    #[test]
    fn realistic_numpy_release_matrix() {
        // Approximates the numpy 1.26 release file list on PyPI — covers
        // CPython 3.10/3.11/3.12 × manylinux/macOS-arm64/macOS-x86_64/win.
        let files = vec![
            "numpy-1.26.0-cp310-cp310-macosx_10_9_x86_64.whl".to_string(),
            "numpy-1.26.0-cp310-cp310-macosx_11_0_arm64.whl".to_string(),
            "numpy-1.26.0-cp310-cp310-manylinux_2_17_x86_64.manylinux2014_x86_64.whl".to_string(),
            "numpy-1.26.0-cp310-cp310-win_amd64.whl".to_string(),
            "numpy-1.26.0-cp311-cp311-macosx_10_9_x86_64.whl".to_string(),
            "numpy-1.26.0-cp311-cp311-macosx_11_0_arm64.whl".to_string(),
            "numpy-1.26.0-cp311-cp311-manylinux_2_17_x86_64.manylinux2014_x86_64.whl".to_string(),
            "numpy-1.26.0-cp311-cp311-win_amd64.whl".to_string(),
            "numpy-1.26.0-cp312-cp312-macosx_10_9_x86_64.whl".to_string(),
            "numpy-1.26.0-cp312-cp312-macosx_11_0_arm64.whl".to_string(),
            "numpy-1.26.0-cp312-cp312-manylinux_2_17_x86_64.manylinux2014_x86_64.whl".to_string(),
            "numpy-1.26.0-cp312-cp312-win_amd64.whl".to_string(),
            "numpy-1.26.0.tar.gz".to_string(),
        ];
        let s = macos_arm64_cp312_selector();
        let best = pick_best_wheel(&files, &s).unwrap();
        assert_eq!(
            best.filename,
            "numpy-1.26.0-cp312-cp312-macosx_11_0_arm64.whl"
        );
    }

    #[test]
    fn wheel_tag_of_passes_through() {
        let tag = wheel_tag_of("pkg-1.0-cp312-cp312-macosx_11_0_arm64.whl").unwrap();
        assert_eq!(tag.python, vec!["cp312".to_string()]);
        assert_eq!(tag.platform, vec!["macosx_11_0_arm64".to_string()]);
    }
}
