// REQ: R5 — PEP 440-compliant version ordering for "latest" selection.
//
// Implements a minimal PEP 440 comparator covering:
//   - Release segments: 1.2.3 split on '.' with integer comparison
//   - Pre-release: a/b/rc with ordering alpha < beta < rc
//   - Post-release: .postN (numeric)
//   - Dev-release: .devN (numeric)
//
// Ordering rules (lowest to highest):
//   dev < alpha < beta < rc < release < post
//
// No support for local-version segments ("+local") — stripped and ignored.

use std::cmp::Ordering;

/// Pre-release phase, ordered lowest to highest.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
enum PrePhase {
    Alpha,
    Beta,
    Rc,
}

/// All components needed to compare two PEP 440 version strings.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Pep440Version {
    /// Release segments, e.g. [1, 2, 3] for "1.2.3".
    release: Vec<u64>,
    /// Dev release number: Some(N) for ".devN", None means not a dev release.
    /// Dev releases sort BELOW pre-releases and the base release.
    dev: Option<u64>,
    /// Pre-release: Some((phase, N)) for "a1"/"b2"/"rc3", None means not a pre-release.
    pre: Option<(PrePhase, u64)>,
    /// Post-release number: Some(N) for ".postN".
    post: Option<u64>,
}

impl Pep440Version {
    /// Read-only view of the release segments (e.g. `[1, 2, 3]` for "1.2.3").
    /// Exposed for consumers like `pip_check` that need wildcard-prefix
    /// matching against `==X.Y.*` specifiers.
    pub(crate) fn release_segments(&self) -> &[u64] {
        &self.release
    }
}

/// Parse a PEP 440 version string into a `Pep440Version`.
///
/// Returns `None` when the string is entirely unparseable (no release digits).
/// Best-effort: unknown suffixes are ignored.
pub(crate) fn parse(version: &str) -> Option<Pep440Version> {
    // Strip local version segment (everything after '+').
    let without_local = version.split('+').next().unwrap_or(version);

    // Lowercase for case-insensitive pre-release tags.
    let s = without_local.to_lowercase();

    // Split on '!' for epoch — we ignore epoch (treat as 0) but consume it.
    let s = if let Some((_epoch, rest)) = s.split_once('!') { rest } else { &s };

    // Extract dev suffix: ".dev<N>" at the end.
    let (s, dev) = if let Some((base, dev_str)) = s.rsplit_once(".dev") {
        let dev_n: u64 = dev_str.parse().unwrap_or(0);
        (base, Some(dev_n))
    } else if let Some(base) = s.strip_suffix(".dev") {
        (base, Some(0))
    } else {
        (s, None)
    };

    // Extract post suffix: ".post<N>" at the end.
    let (s, post) = if let Some((base, post_str)) = s.rsplit_once(".post") {
        let post_n: u64 = post_str.parse().unwrap_or(0);
        (base, Some(post_n))
    } else if let Some(base) = s.strip_suffix(".post") {
        (base, Some(0))
    } else {
        (s, None)
    };

    // Extract pre-release suffix: the last segment may end with a/b/rc + digits.
    // e.g. "1.0.0a1" → release=[1,0,0], pre=(Alpha,1)
    //      "1.0.0rc2" → release=[1,0,0], pre=(Rc,2)
    //      "1.2.3b4"  → release=[1,2,3], pre=(Beta,4)
    let (release_str, pre) = parse_pre_suffix(s);

    // Parse release segments. Every non-empty segment must be a valid
    // u64 — silently coercing "garbage" to 0 would let the resolver
    // accept malformed specifiers like ">=garbage" as ">=0".
    let mut release: Vec<u64> = Vec::new();
    for seg in release_str.split('.') {
        if seg.is_empty() {
            continue;
        }
        match seg.parse::<u64>() {
            Ok(n) => release.push(n),
            Err(_) => return None,
        }
    }

    if release.is_empty() {
        return None;
    }

    Some(Pep440Version { release, dev, pre, post })
}

/// Split a version string at the pre-release suffix (a/b/rc + digits).
///
/// Returns `(release_part, Option<(PrePhase, N)>)`.
fn parse_pre_suffix(s: &str) -> (&str, Option<(PrePhase, u64)>) {
    // Check for "rc" first (2 chars) before "a"/"b" (1 char).
    for (tag, phase) in [("rc", PrePhase::Rc), ("b", PrePhase::Beta), ("a", PrePhase::Alpha)] {
        if let Some((base, num_str)) = s.rsplit_once(tag) {
            // Ensure num_str is all digits (could be empty → 0) and base ends with digit or '.'.
            if num_str.chars().all(|c| c.is_ascii_digit()) {
                let last_char = base.chars().last();
                if last_char.map(|c| c.is_ascii_digit()).unwrap_or(false) {
                    let n: u64 = if num_str.is_empty() { 0 } else { num_str.parse().unwrap_or(0) };
                    return (base, Some((phase, n)));
                }
            }
        }
    }
    (s, None)
}

impl Pep440Version {
    /// Compare two release segment vectors, treating missing trailing segments as 0.
    fn cmp_release(a: &[u64], b: &[u64]) -> Ordering {
        let len = a.len().max(b.len());
        for i in 0..len {
            let av = a.get(i).copied().unwrap_or(0);
            let bv = b.get(i).copied().unwrap_or(0);
            match av.cmp(&bv) {
                Ordering::Equal => continue,
                other => return other,
            }
        }
        Ordering::Equal
    }

    /// Numeric sort key for the pre-release phase component.
    ///
    /// Returns a tuple (phase_rank, pre_n) where phase_rank puts the release
    /// above pre-releases and below post-releases. Values:
    ///
    /// | State       | phase_rank |
    /// |-------------|------------|
    /// | dev         | -2         |
    /// | alpha       | -1 (pre)   |
    /// | beta        | -1 (pre)   |
    /// | rc          | -1 (pre)   |
    /// | release     |  0         |
    /// | post        | +1         |
    ///
    /// Pre-phases are further distinguished by (PrePhase, n).
    fn rank(&self) -> (i32, Option<&PrePhase>, u64, Option<u64>) {
        if self.dev.is_some() {
            return (-2, None, 0, self.dev);
        }
        if let Some((ref phase, n)) = self.pre {
            return (-1, Some(phase), n, None);
        }
        if let Some(n) = self.post {
            return (1, None, n, None);
        }
        (0, None, 0, None)
    }
}

impl PartialOrd for Pep440Version {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Pep440Version {
    fn cmp(&self, other: &Self) -> Ordering {
        // First compare release segments.
        let rel = Self::cmp_release(&self.release, &other.release);
        if rel != Ordering::Equal {
            return rel;
        }

        // Same release — compare rank (dev < pre < release < post).
        let (sr, sp, sn, sd) = self.rank();
        let (or_, op, on, od) = other.rank();

        match sr.cmp(&or_) {
            Ordering::Equal => {}
            other => return other,
        }

        // Same rank tier — compare sub-fields.
        if sr == -2 {
            // Both dev — compare dev number.
            return sd.unwrap_or(0).cmp(&od.unwrap_or(0));
        }
        if sr == -1 {
            // Both pre-release — compare phase then number.
            match sp.cmp(&op) {
                Ordering::Equal => {}
                other => return other,
            }
            return sn.cmp(&on);
        }
        if sr == 1 {
            // Both post — compare post number.
            return sn.cmp(&on);
        }
        // Both plain release — already equal by release segment comparison.
        Ordering::Equal
    }
}

/// Sort a slice of version strings by PEP 440 ordering, newest first.
///
/// Versions that cannot be parsed (no release segments) are placed at the end.
pub(crate) fn sort_versions_newest_first(versions: &mut Vec<String>) {
    versions.sort_by(|a, b| {
        let pa = parse(a);
        let pb = parse(b);
        match (pa, pb) {
            (Some(va), Some(vb)) => vb.cmp(&va), // descending
            (Some(_), None) => Ordering::Less,   // parseable < unparseable
            (None, Some(_)) => Ordering::Greater,
            (None, None) => b.cmp(a),            // both unparseable: lex desc
        }
    });
}

// ---------------------------------------------------------------------------
// Tests (D2)
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // REQ: D2-test-a — pre-releases rank below release; 1.0.1 is latest
    #[test]
    fn test_pep440_sort_pre_releases_below_release() {
        let mut versions =
            vec!["1.0.0".to_string(), "1.0.0a1".to_string(), "1.0.0rc1".to_string(), "1.0.1".to_string()];
        sort_versions_newest_first(&mut versions);
        // Latest must be 1.0.1; pre-releases ranked below the full release 1.0.0
        assert_eq!(versions[0], "1.0.1", "1.0.1 must be newest");
        // 1.0.0 must come before pre-releases (1.0.0 > 1.0.0rc1 > 1.0.0a1)
        let idx_100 = versions.iter().position(|v| v == "1.0.0").unwrap();
        let idx_rc = versions.iter().position(|v| v == "1.0.0rc1").unwrap();
        let idx_a = versions.iter().position(|v| v == "1.0.0a1").unwrap();
        assert!(idx_100 < idx_rc, "1.0.0 must rank above 1.0.0rc1");
        assert!(idx_rc < idx_a, "1.0.0rc1 must rank above 1.0.0a1");
    }

    // REQ: D2-test-b — numeric segment comparison (1.10 > 1.2); post-release > base
    #[test]
    fn test_pep440_sort_numeric_segments_and_post() {
        let mut versions =
            vec!["1.2".to_string(), "1.10".to_string(), "1.2.post1".to_string()];
        sort_versions_newest_first(&mut versions);
        // 1.10 > 1.2.post1 > 1.2
        assert_eq!(versions[0], "1.10", "1.10 must be newest (10 > 2 numerically)");
        let idx_post = versions.iter().position(|v| v == "1.2.post1").unwrap();
        let idx_base = versions.iter().position(|v| v == "1.2").unwrap();
        assert!(idx_post < idx_base, "1.2.post1 must rank above 1.2");
    }

    // REQ: tick-116 test-coverage — beta phase + alpha<beta<rc strict ordering + release-pad equality
    #[test]
    fn test_pep440_sort_beta_phase_and_pad_equality() {
        // Covers previously-untested beta branch + ensures full alpha<beta<rc ordering
        // and that missing trailing release segments pad to 0 (1.0 ≡ 1.0.0).
        let mut versions = vec![
            "1.0".to_string(),
            "1.0.0".to_string(),
            "1.0.0a1".to_string(),
            "1.0.0b1".to_string(),
            "1.0.0rc1".to_string(),
        ];
        sort_versions_newest_first(&mut versions);
        let idx_rc = versions.iter().position(|v| v == "1.0.0rc1").unwrap();
        let idx_b = versions.iter().position(|v| v == "1.0.0b1").unwrap();
        let idx_a = versions.iter().position(|v| v == "1.0.0a1").unwrap();
        assert!(idx_rc < idx_b, "rc must rank above beta");
        assert!(idx_b < idx_a, "beta must rank above alpha");
        // Both 1.0 and 1.0.0 pad to [1,0,0] — must tie as equal-rank releases, both above rc
        let idx_pad = versions.iter().position(|v| v == "1.0").unwrap();
        let idx_full = versions.iter().position(|v| v == "1.0.0").unwrap();
        assert!(idx_pad < idx_rc && idx_full < idx_rc, "1.0 and 1.0.0 must both outrank pre-releases");
    }

    // REQ: dev-release ranks below pre-release; local-version + epoch + unparseable edges
    #[test]
    fn test_pep440_sort_dev_local_epoch_and_unparseable() {
        let mut versions = vec![
            "1.0.0".to_string(),
            "1.0.0a1".to_string(),
            "1.0.0.dev1".to_string(),
            "1.0.0+deadbeef".to_string(), // local — strips to 1.0.0, ties with "1.0.0"
            "1!0.1".to_string(),           // epoch — stripped, becomes 0.1
            "not-a-version".to_string(),   // unparseable — goes to end
        ];
        sort_versions_newest_first(&mut versions);
        // Rank: 1.0.0 == 1.0.0+deadbeef > 1.0.0a1 > 1.0.0.dev1 > 0.1 (epoch-stripped) > unparseable
        assert_eq!(versions.last().unwrap(), "not-a-version", "unparseable must sink to end");
        let idx_a1 = versions.iter().position(|v| v == "1.0.0a1").unwrap();
        let idx_dev = versions.iter().position(|v| v == "1.0.0.dev1").unwrap();
        assert!(idx_a1 < idx_dev, "alpha must rank above dev (dev < alpha)");
        let idx_epoch = versions.iter().position(|v| v == "1!0.1").unwrap();
        assert!(idx_dev < idx_epoch, "1.0.0.dev1 must rank above epoch-stripped 0.1");
    }
}
