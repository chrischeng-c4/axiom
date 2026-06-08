// `uv pip check` — installed-dist dependency consistency (Tick 52).
//
// Given a snapshot of installed distributions (Tick 44 `pip_inventory`),
// walk every `Requires-Dist`/`Requires` line and verify that:
//   1. The named dependency is also installed.
//   2. The installed version satisfies every PEP 440 specifier listed.
//
// Reports a `Vec<CheckIssue>`. Pure-data — no filesystem, no resolver.
// `marker` strings are checked when present but only with the very
// minimal `extra == "..."` rule (markers that filter on the current
// environment are evaluated by the installer; uv `pip check` likewise
// reports all dep edges).

use std::cmp::Ordering;

use crate::pkgmanage::pkgmgr::name_normalize::pep503_normalize;
use crate::pkgmanage::pkgmgr::pep440;
use crate::pkgmanage::pkgmgr::pip_inventory::InstalledDist;
use crate::pkgmanage::pkgmgr::requirements_parse::parse_one_line;
use crate::pkgmanage::pkgmgr::requirements_parse::RequirementLine;

/// One consistency issue detected by `check_consistency`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CheckIssue {
    pub kind: CheckIssueKind,
    /// Package that *has* the broken requirement.
    pub package: String,
    /// Package that is missing or mismatched.
    pub dependency: String,
    /// Raw requirement text as it appeared in the parent's METADATA.
    pub requirement: String,
    /// Human-readable diagnosis.
    pub detail: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CheckIssueKind {
    /// Required package is not installed at all.
    Missing,
    /// Installed but the version doesn't satisfy the specifier.
    VersionMismatch,
    /// The METADATA `Requires-Dist` line itself failed to parse.
    BrokenRequirement,
}

/// Top-level entry point. Iterates every installed dist's
/// `requires` list and returns every issue in source order
/// (parent dist insertion order, then per-line order).
pub fn check_consistency(installed: &[InstalledDist]) -> Vec<CheckIssue> {
    let by_norm: std::collections::HashMap<String, &InstalledDist> = installed
        .iter()
        .map(|d| (pep503_normalize(&d.name), d))
        .collect();

    let mut issues = Vec::new();
    for dist in installed {
        for raw in &dist.requires {
            if should_skip_marker(raw) {
                continue;
            }
            match parse_one_line(raw) {
                Ok(RequirementLine::Package(p)) => {
                    let target_key = pep503_normalize(&p.name);
                    let Some(target) = by_norm.get(&target_key) else {
                        issues.push(CheckIssue {
                            kind: CheckIssueKind::Missing,
                            package: dist.name.clone(),
                            dependency: p.name.clone(),
                            requirement: raw.clone(),
                            detail: format!(
                                "{} requires {}, but it is not installed",
                                dist.name, p.name
                            ),
                        });
                        continue;
                    };
                    if !p.specifiers.is_empty()
                        && !version_satisfies_all(&target.version, &p.specifiers)
                    {
                        issues.push(CheckIssue {
                            kind: CheckIssueKind::VersionMismatch,
                            package: dist.name.clone(),
                            dependency: target.name.clone(),
                            requirement: raw.clone(),
                            detail: format!(
                                "{} requires {} {}, but {} {} is installed",
                                dist.name,
                                p.name,
                                p.specifiers.join(","),
                                target.name,
                                target.version
                            ),
                        });
                    }
                }
                Ok(_) | Err(_) => {
                    issues.push(CheckIssue {
                        kind: CheckIssueKind::BrokenRequirement,
                        package: dist.name.clone(),
                        dependency: String::new(),
                        requirement: raw.clone(),
                        detail: format!(
                            "{} has an unparseable requirement line: {}",
                            dist.name, raw
                        ),
                    });
                }
            }
        }
    }

    issues
}

/// Filter out marker-tagged requirements that wouldn't apply to a normal
/// runtime install. We conservatively skip *only* `; extra == "..."`
/// markers — extras are not enabled by default in `pip check`.
fn should_skip_marker(raw: &str) -> bool {
    let Some(idx) = raw.find(';') else {
        return false;
    };
    let marker = raw[idx + 1..].trim();
    // Most pragmatic rule: any marker that *requires* an extra is skipped.
    marker.contains("extra ==") || marker.contains("extra==")
}

/// Returns true iff `version` satisfies every specifier in `specs`.
pub fn version_satisfies_all(version: &str, specs: &[String]) -> bool {
    specs.iter().all(|s| version_satisfies(version, s))
}

/// Evaluate one PEP 440 specifier (`>=1.0`, `==1.2.*`, `~=1.4`) against a
/// concrete version string.
///
/// Recognised operators: `===`, `==`, `!=`, `~=`, `>=`, `<=`, `>`, `<`.
/// Unknown specifiers conservatively return `false` so we surface them
/// as VersionMismatch rather than silently accept.
pub fn version_satisfies(version: &str, spec: &str) -> bool {
    let Some((op, target_str)) = split_specifier(spec) else {
        return false;
    };
    let target_str = target_str.trim();

    if op == "===" {
        return version.trim() == target_str;
    }

    let Some(actual) = pep440::parse(version) else {
        return false;
    };

    // For `==X.Y.*` style wildcard (PEP 440), strip the trailing `.*` and
    // require a prefix match on the release vector.
    if op == "==" {
        if let Some(prefix) = target_str.strip_suffix(".*") {
            return prefix_match(&actual, prefix);
        }
    }
    if op == "!=" {
        if let Some(prefix) = target_str.strip_suffix(".*") {
            return !prefix_match(&actual, prefix);
        }
    }

    let Some(target) = pep440::parse(target_str) else {
        return false;
    };

    match op {
        "==" => actual.cmp(&target) == Ordering::Equal,
        "!=" => actual.cmp(&target) != Ordering::Equal,
        ">=" => actual.cmp(&target) != Ordering::Less,
        "<=" => actual.cmp(&target) != Ordering::Greater,
        ">" => actual.cmp(&target) == Ordering::Greater,
        "<" => actual.cmp(&target) == Ordering::Less,
        "~=" => compatible_release(&actual, target_str),
        _ => false,
    }
}

fn split_specifier(spec: &str) -> Option<(&str, &str)> {
    // Order matters: 3-char ops first, then 2-char, then 1-char.
    for op in ["===", "==", "!=", "~=", ">=", "<=", ">", "<"] {
        if let Some(rest) = spec.strip_prefix(op) {
            return Some((op, rest));
        }
    }
    None
}

fn prefix_match(actual: &pep440::Pep440Version, prefix: &str) -> bool {
    // `==1.2.*` means: the first two release segments of actual must
    // equal `[1, 2]`. Pre/post/dev are ignored for the wildcard match.
    let want: Vec<u64> = prefix
        .split('.')
        .filter(|s| !s.is_empty())
        .map(|s| s.parse::<u64>().unwrap_or(u64::MAX))
        .collect();
    if want.contains(&u64::MAX) {
        return false;
    }
    let release = actual.release_segments();
    if want.len() > release.len() {
        return false;
    }
    for (i, w) in want.iter().enumerate() {
        if release[i] != *w {
            return false;
        }
    }
    true
}

fn compatible_release(actual: &pep440::Pep440Version, target_str: &str) -> bool {
    // `~=X.Y` → `>=X.Y, <X+1`
    // `~=X.Y.Z` → `>=X.Y.Z, <X.Y+1`
    // `~=X` is invalid by PEP 440 — treat as false.
    let segs: Vec<&str> = target_str.split('.').collect();
    if segs.len() < 2 {
        return false;
    }
    let Some(target) = pep440::parse(target_str) else {
        return false;
    };
    if actual.cmp(&target) == Ordering::Less {
        return false;
    }

    // Build the upper bound: drop the last segment, increment the new last.
    let parsed_segs: Result<Vec<u64>, _> =
        segs.iter().map(|s| s.parse::<u64>()).collect();
    let Ok(mut nums) = parsed_segs else {
        return false;
    };
    nums.pop();
    if let Some(last) = nums.last_mut() {
        *last += 1;
    } else {
        return false;
    }
    let upper_str = nums
        .iter()
        .map(|n| n.to_string())
        .collect::<Vec<_>>()
        .join(".");
    let Some(upper) = pep440::parse(&upper_str) else {
        return false;
    };
    actual.cmp(&upper) == Ordering::Less
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn dist(name: &str, version: &str, requires: &[&str]) -> InstalledDist {
        InstalledDist {
            canonical_name: pep503_normalize(name),
            name: name.to_string(),
            version: version.to_string(),
            dist_info: PathBuf::from(format!("/site/{}-{}.dist-info", name, version)),
            summary: None,
            requires: requires.iter().map(|s| s.to_string()).collect(),
            home_page: None,
            author: None,
            license: None,
        }
    }

    // ---- version_satisfies ---------------------------------------------

    #[test]
    fn satisfies_eq() {
        assert!(version_satisfies("1.2.3", "==1.2.3"));
        assert!(!version_satisfies("1.2.4", "==1.2.3"));
    }

    #[test]
    fn satisfies_neq() {
        assert!(version_satisfies("1.2.4", "!=1.2.3"));
        assert!(!version_satisfies("1.2.3", "!=1.2.3"));
    }

    #[test]
    fn satisfies_ge_le() {
        assert!(version_satisfies("2.0.0", ">=1.0.0"));
        assert!(version_satisfies("1.0.0", ">=1.0.0"));
        assert!(!version_satisfies("0.9.0", ">=1.0.0"));
        assert!(version_satisfies("1.0.0", "<=1.0.0"));
        assert!(!version_satisfies("1.0.1", "<=1.0.0"));
    }

    #[test]
    fn satisfies_strict() {
        assert!(version_satisfies("2.0.0", ">1.0.0"));
        assert!(!version_satisfies("1.0.0", ">1.0.0"));
        assert!(version_satisfies("0.9.0", "<1.0.0"));
        assert!(!version_satisfies("1.0.0", "<1.0.0"));
    }

    #[test]
    fn satisfies_tilde_eq_minor() {
        // ~=1.4 → >=1.4, <2
        assert!(version_satisfies("1.4.0", "~=1.4"));
        assert!(version_satisfies("1.99.9", "~=1.4"));
        assert!(!version_satisfies("2.0.0", "~=1.4"));
        assert!(!version_satisfies("1.3.9", "~=1.4"));
    }

    #[test]
    fn satisfies_tilde_eq_patch() {
        // ~=1.4.2 → >=1.4.2, <1.5
        assert!(version_satisfies("1.4.2", "~=1.4.2"));
        assert!(version_satisfies("1.4.99", "~=1.4.2"));
        assert!(!version_satisfies("1.5.0", "~=1.4.2"));
        assert!(!version_satisfies("1.4.1", "~=1.4.2"));
    }

    #[test]
    fn satisfies_wildcard_eq() {
        assert!(version_satisfies("1.2.3", "==1.2.*"));
        assert!(version_satisfies("1.2.99", "==1.2.*"));
        assert!(!version_satisfies("1.3.0", "==1.2.*"));
    }

    #[test]
    fn satisfies_wildcard_neq() {
        assert!(!version_satisfies("1.2.3", "!=1.2.*"));
        assert!(version_satisfies("1.3.0", "!=1.2.*"));
    }

    #[test]
    fn satisfies_arbitrary_equality() {
        assert!(version_satisfies("1.2.3-beta", "===1.2.3-beta"));
        assert!(!version_satisfies("1.2.3", "===1.2.3-beta"));
    }

    #[test]
    fn satisfies_all_compound() {
        let specs = vec![">=1.0".to_string(), "<2.0".to_string()];
        assert!(version_satisfies_all("1.5.0", &specs));
        assert!(!version_satisfies_all("2.0.1", &specs));
    }

    #[test]
    fn unknown_operator_rejects() {
        assert!(!version_satisfies("1.0", "garbage"));
    }

    #[test]
    fn unparseable_version_rejects() {
        assert!(!version_satisfies("not-a-version", ">=1.0"));
    }

    // ---- check_consistency ----------------------------------------------

    #[test]
    fn empty_input_yields_no_issues() {
        assert!(check_consistency(&[]).is_empty());
    }

    #[test]
    fn happy_path_satisfied() {
        let d = vec![
            dist("requests", "2.31.0", &["urllib3>=1.21,<3", "charset_normalizer<4"]),
            dist("urllib3", "2.0.7", &[]),
            dist("charset_normalizer", "3.3.0", &[]),
        ];
        assert!(check_consistency(&d).is_empty());
    }

    #[test]
    fn flags_missing_dependency() {
        let d = vec![dist("requests", "2.31.0", &["urllib3>=1.21"])];
        let issues = check_consistency(&d);
        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].kind, CheckIssueKind::Missing);
        assert_eq!(issues[0].package, "requests");
        assert_eq!(issues[0].dependency, "urllib3");
        assert!(issues[0].detail.contains("not installed"));
    }

    #[test]
    fn flags_version_mismatch() {
        let d = vec![
            dist("a", "1.0.0", &["b>=2.0"]),
            dist("b", "1.0.0", &[]),
        ];
        let issues = check_consistency(&d);
        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].kind, CheckIssueKind::VersionMismatch);
        assert_eq!(issues[0].package, "a");
        // Detail format: "a requires b >=2.0, but b 1.0.0 is installed".
        assert!(issues[0].detail.contains("b 1.0.0"));
        assert!(issues[0].detail.contains(">=2.0"));
    }

    #[test]
    fn skips_extra_marker_requirements() {
        let d = vec![dist(
            "requests",
            "2.31.0",
            &["chardet>=3 ; extra == 'use_chardet_on_py3'"],
        )];
        // No `chardet` installed but the requirement is gated by extra,
        // so no issue is raised.
        assert!(check_consistency(&d).is_empty());
    }

    #[test]
    fn normalised_name_match_succeeds() {
        // Required name and installed name differ by PEP 503 normalization.
        let d = vec![
            dist("My-Pkg", "1.0", &["my_pkg-dep"]),
            dist("My.Pkg.Dep", "0.1", &[]),
        ];
        assert!(check_consistency(&d).is_empty());
    }

    #[test]
    fn flags_broken_requirement_line() {
        let d = vec![dist("bad", "1.0", &["@@@malformed@@"])];
        let issues = check_consistency(&d);
        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].kind, CheckIssueKind::BrokenRequirement);
    }

    #[test]
    fn reports_issues_in_source_order() {
        let d = vec![
            dist("a", "1.0", &["missing_one"]),
            dist("b", "1.0", &["missing_two"]),
        ];
        let issues = check_consistency(&d);
        assert_eq!(issues.len(), 2);
        assert_eq!(issues[0].dependency, "missing_one");
        assert_eq!(issues[1].dependency, "missing_two");
    }

    #[test]
    fn marker_without_extra_does_not_skip() {
        // python_version markers are not enough to suppress the check —
        // pip check / uv pip check report these as if active.
        let d = vec![dist(
            "a",
            "1.0",
            &["nonexistent>=1 ; python_version >= '3.8'"],
        )];
        let issues = check_consistency(&d);
        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].kind, CheckIssueKind::Missing);
    }
}
