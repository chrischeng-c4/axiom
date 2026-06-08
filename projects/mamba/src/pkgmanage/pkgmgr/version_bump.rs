// `uv version` — PEP 440 version bump + pyproject.toml writeback (Tick 43).
//
// Pure data layer. Mirrors uv's `version` subcommand surface:
//
//   $ uv version --bump minor      # 1.2.3 -> 1.3.0
//   $ uv version --bump patch      # 1.2.3 -> 1.2.4
//   $ uv version --bump alpha      # 1.2.3 -> 1.2.4a1
//   $ uv version --bump beta       # 1.2.3a1 -> 1.2.3b1
//   $ uv version --bump rc         # 1.2.3b1 -> 1.2.3rc1
//   $ uv version --bump post       # 1.2.3 -> 1.2.3.post1
//   $ uv version --bump dev        # 1.2.3 -> 1.2.3.dev1
//   $ uv version --bump release    # 1.2.3a1 -> 1.2.3 (drop pre/dev)
//   $ uv version 2.0.0             # set explicit
//
// The module exposes:
//   * `Version` — round-trippable PEP 440 representation with
//     epoch, release segments, pre / post / dev, and local part.
//   * `parse_version` / `Version::render` — canonical form.
//   * `BumpKind` — every uv-equivalent bump.
//   * `bump(version, kind)` — pure transition function.
//   * `read_pyproject_version(toml_src)` — extract `[project].version`.
//   * `write_pyproject_version(toml_src, new)` — minimal-diff
//     pyproject.toml rewrite (re-uses the line layout when possible).

use std::cmp::Ordering;

/// PEP 440 pre-release phase. Order: alpha < beta < rc.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PrePhase {
    Alpha,
    Beta,
    Rc,
}

impl PrePhase {
    fn tag(self) -> &'static str {
        match self {
            PrePhase::Alpha => "a",
            PrePhase::Beta => "b",
            PrePhase::Rc => "rc",
        }
    }
    fn rank(self) -> u8 {
        match self {
            PrePhase::Alpha => 0,
            PrePhase::Beta => 1,
            PrePhase::Rc => 2,
        }
    }
}

/// Round-trippable PEP 440 version. Sufficient surface to parse, bump,
/// render, and compare uv-style.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Version {
    pub epoch: u64,
    pub release: Vec<u64>,
    pub pre: Option<(PrePhase, u64)>,
    pub post: Option<u64>,
    pub dev: Option<u64>,
    pub local: Option<String>,
}

impl Version {
    pub fn new(release: Vec<u64>) -> Self {
        Version {
            epoch: 0,
            release,
            pre: None,
            post: None,
            dev: None,
            local: None,
        }
    }

    /// Canonical PEP 440 string form. Round-trips through `parse_version`.
    pub fn render(&self) -> String {
        let mut s = String::new();
        if self.epoch > 0 {
            s.push_str(&format!("{}!", self.epoch));
        }
        let parts: Vec<String> = self.release.iter().map(|n| n.to_string()).collect();
        s.push_str(&parts.join("."));
        if let Some((phase, n)) = self.pre {
            s.push_str(phase.tag());
            s.push_str(&n.to_string());
        }
        if let Some(p) = self.post {
            s.push_str(&format!(".post{p}"));
        }
        if let Some(d) = self.dev {
            s.push_str(&format!(".dev{d}"));
        }
        if let Some(local) = &self.local {
            s.push('+');
            s.push_str(local);
        }
        s
    }
}

impl PartialOrd for Version {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Version {
    fn cmp(&self, other: &Self) -> Ordering {
        self.epoch
            .cmp(&other.epoch)
            .then_with(|| cmp_release(&self.release, &other.release))
            .then_with(|| pre_rank(self).cmp(&pre_rank(other)))
            .then_with(|| self.post.unwrap_or(0).cmp(&other.post.unwrap_or(0)))
            .then_with(|| dev_rank(self).cmp(&dev_rank(other)))
    }
}

fn cmp_release(a: &[u64], b: &[u64]) -> Ordering {
    let n = a.len().max(b.len());
    for i in 0..n {
        let av = a.get(i).copied().unwrap_or(0);
        let bv = b.get(i).copied().unwrap_or(0);
        match av.cmp(&bv) {
            Ordering::Equal => continue,
            other => return other,
        }
    }
    Ordering::Equal
}

fn pre_rank(v: &Version) -> (u8, u8, u64) {
    // dev<pre<release<post; encode as (cohort, phase_rank, num).
    if let Some((phase, n)) = v.pre {
        (1, phase.rank(), n)
    } else if v.dev.is_some() && v.pre.is_none() && v.post.is_none() {
        // Bare dev release sorts BEFORE pre releases.
        (0, 0, 0)
    } else {
        (2, 0, 0)
    }
}

fn dev_rank(v: &Version) -> u64 {
    v.dev.unwrap_or(u64::MAX)
}

/// Parse a PEP 440 version string into a `Version`. Returns `None` for
/// anything that doesn't have at least one release-segment digit.
pub fn parse_version(src: &str) -> Option<Version> {
    let src = src.trim();
    if src.is_empty() {
        return None;
    }
    // Local segment after `+`.
    let (head, local) = match src.split_once('+') {
        Some((h, l)) => (h, Some(l.to_string())),
        None => (src, None),
    };
    let head = head.to_lowercase();

    // Epoch before `!`.
    let (epoch, after_epoch) = match head.split_once('!') {
        Some((e, rest)) => {
            let n: u64 = e.parse().ok()?;
            (n, rest.to_string())
        }
        None => (0, head),
    };

    // Strip dev suffix.
    let (after_epoch, dev) = strip_numbered_suffix(&after_epoch, ".dev");
    // Strip post suffix.
    let (after_epoch, post) = strip_numbered_suffix(&after_epoch, ".post");
    // Strip pre suffix (rc first, then beta, then alpha).
    let (release_str, pre) = strip_pre_suffix(&after_epoch);

    let release_str = release_str.trim_matches('.');
    if release_str.is_empty() {
        return None;
    }
    let mut release = Vec::new();
    for seg in release_str.split('.') {
        if seg.is_empty() {
            continue;
        }
        let n: u64 = seg.parse().ok()?;
        release.push(n);
    }
    if release.is_empty() {
        return None;
    }
    Some(Version {
        epoch,
        release,
        pre,
        post,
        dev,
        local,
    })
}

fn strip_numbered_suffix(s: &str, marker: &str) -> (String, Option<u64>) {
    if let Some((base, rest)) = s.rsplit_once(marker) {
        if rest.chars().all(|c| c.is_ascii_digit()) {
            let n = if rest.is_empty() { 0 } else { rest.parse().unwrap_or(0) };
            return (base.to_string(), Some(n));
        }
    }
    (s.to_string(), None)
}

fn strip_pre_suffix(s: &str) -> (String, Option<(PrePhase, u64)>) {
    for (tag, phase) in [
        ("rc", PrePhase::Rc),
        ("b", PrePhase::Beta),
        ("a", PrePhase::Alpha),
    ] {
        if let Some((base, rest)) = s.rsplit_once(tag) {
            if rest.chars().all(|c| c.is_ascii_digit()) {
                if let Some(last) = base.chars().last() {
                    if last.is_ascii_digit() {
                        let n = if rest.is_empty() { 0 } else { rest.parse().unwrap_or(0) };
                        return (base.to_string(), Some((phase, n)));
                    }
                }
            }
        }
    }
    (s.to_string(), None)
}

/// Every uv-equivalent bump operation. Each variant operates on a
/// freshly-parsed `Version` and returns a new `Version` per PEP 440
/// reset rules.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BumpKind {
    /// `1.2.3` → `2.0.0`. Clears pre/post/dev/local.
    Major,
    /// `1.2.3` → `1.3.0`. Clears pre/post/dev/local.
    Minor,
    /// `1.2.3` → `1.2.4`. Clears pre/post/dev/local.
    Patch,
    /// `1.2.3` → `1.2.4a1`, `1.2.4a1` → `1.2.4a2`.
    Alpha,
    /// `1.2.3` → `1.2.4b1`, `1.2.3a4` → `1.2.3b1`, `1.2.3b1` → `1.2.3b2`.
    Beta,
    /// `1.2.3` → `1.2.4rc1`, `1.2.3a4` / `1.2.3b1` → `1.2.3rc1`,
    /// `1.2.3rc1` → `1.2.3rc2`.
    Rc,
    /// `1.2.3` → `1.2.3.post1`, `1.2.3.post1` → `1.2.3.post2`. Pre is preserved.
    Post,
    /// `1.2.3` → `1.2.3.dev1`, `1.2.3.dev1` → `1.2.3.dev2`.
    Dev,
    /// Drop pre/dev — `1.2.3a4.dev2` → `1.2.3`. Post is preserved.
    Release,
}

/// Apply a bump. Pure: the input is not mutated.
pub fn bump(version: &Version, kind: BumpKind) -> Version {
    let mut v = version.clone();
    match kind {
        BumpKind::Major => bump_release(&mut v, 0),
        BumpKind::Minor => bump_release(&mut v, 1),
        BumpKind::Patch => bump_release(&mut v, 2),
        BumpKind::Alpha => bump_pre(&mut v, PrePhase::Alpha),
        BumpKind::Beta => bump_pre(&mut v, PrePhase::Beta),
        BumpKind::Rc => bump_pre(&mut v, PrePhase::Rc),
        BumpKind::Post => {
            v.post = Some(v.post.unwrap_or(0) + 1);
            v.dev = None;
        }
        BumpKind::Dev => {
            v.dev = Some(v.dev.unwrap_or(0) + 1);
        }
        BumpKind::Release => {
            v.pre = None;
            v.dev = None;
        }
    }
    v
}

fn bump_release(v: &mut Version, idx: usize) {
    while v.release.len() <= idx {
        v.release.push(0);
    }
    v.release[idx] += 1;
    for slot in v.release.iter_mut().skip(idx + 1) {
        *slot = 0;
    }
    v.pre = None;
    v.post = None;
    v.dev = None;
    v.local = None;
}

fn bump_pre(v: &mut Version, target: PrePhase) {
    let new_pre = match v.pre {
        // Same phase: increment counter.
        Some((p, n)) if p == target => (p, n + 1),
        // Different phase: pre rank must go up. If trying to go BACKWARDS
        // (e.g., from b1 to alpha), uv raises; we do too by bumping patch
        // + restarting at <target>1, matching the "1.2.3 -> 1.2.4a1" rule.
        Some((p, _)) if p.rank() > target.rank() => {
            // Move to next patch then alpha1/beta1/rc1.
            bump_release(v, 2);
            (target, 1)
        }
        // Promoting: keep release, switch phase, restart counter at 1.
        Some(_) => (target, 1),
        // No pre yet: bump patch first to enforce monotonicity, then attach.
        None => {
            bump_release(v, 2);
            (target, 1)
        }
    };
    v.pre = Some(new_pre);
    v.dev = None;
    v.post = None;
}

/// Read the `version = "..."` value from a pyproject.toml. Looks only at
/// the `[project]` table per PEP 621; ignores `[tool.poetry].version`
/// etc. so we never silently bump the wrong field.
pub fn read_pyproject_version(toml_src: &str) -> Option<String> {
    let doc: toml::Value = toml::from_str(toml_src).ok()?;
    let project = doc.get("project")?.as_table()?;
    project.get("version")?.as_str().map(|s| s.to_string())
}

/// Rewrite the `[project].version = "<old>"` line in-place. Returns the
/// new toml body. Preserves the original line's leading whitespace and
/// quote style; uses double quotes when the original isn't found.
///
/// Returns `Err` when the `[project]` table is absent or has no
/// `version` key — caller must surface this rather than blindly add.
pub fn write_pyproject_version(toml_src: &str, new_version: &str) -> Result<String, &'static str> {
    let _ = read_pyproject_version(toml_src).ok_or("pyproject.toml: [project].version missing")?;

    let mut out = String::with_capacity(toml_src.len() + 8);
    let mut in_project = false;
    let mut replaced = false;

    for line in toml_src.split_inclusive('\n') {
        let stripped = line.trim_start();
        if stripped.starts_with('[') {
            // New table — close any prior membership.
            in_project = stripped.starts_with("[project]")
                && !stripped.starts_with("[project.");
        }
        if in_project && !replaced {
            if let Some(rebuilt) = replace_version_assignment(line, new_version) {
                out.push_str(&rebuilt);
                replaced = true;
                continue;
            }
        }
        out.push_str(line);
    }
    if !replaced {
        return Err("pyproject.toml: [project].version line not found");
    }
    Ok(out)
}

fn replace_version_assignment(line: &str, new_version: &str) -> Option<String> {
    let trimmed = line.trim_start();
    if !trimmed.starts_with("version") {
        return None;
    }
    let after = trimmed.strip_prefix("version")?.trim_start();
    if !after.starts_with('=') {
        return None;
    }
    let lead = &line[..line.len() - trimmed.len()];
    let trailing_nl = if line.ends_with('\n') { "\n" } else { "" };
    Some(format!("{lead}version = \"{new_version}\"{trailing_nl}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn v(s: &str) -> Version {
        parse_version(s).unwrap_or_else(|| panic!("could not parse: {s}"))
    }

    #[test]
    fn parse_render_roundtrip_basic() {
        for s in ["1.0.0", "1.2.3", "0.1.0", "10.20.30"] {
            assert_eq!(v(s).render(), s);
        }
    }

    #[test]
    fn parse_render_roundtrip_prerelease() {
        assert_eq!(v("1.2.3a1").render(), "1.2.3a1");
        assert_eq!(v("1.2.3b2").render(), "1.2.3b2");
        assert_eq!(v("1.2.3rc4").render(), "1.2.3rc4");
    }

    #[test]
    fn parse_render_roundtrip_post_dev_epoch_local() {
        assert_eq!(v("1.2.3.post1").render(), "1.2.3.post1");
        assert_eq!(v("1.2.3.dev2").render(), "1.2.3.dev2");
        assert_eq!(v("2!1.0.0").render(), "2!1.0.0");
        assert_eq!(v("1.0.0+abc.123").render(), "1.0.0+abc.123");
    }

    #[test]
    fn parse_handles_mixed_case_pre_tags() {
        assert_eq!(v("1.0.0RC1").render(), "1.0.0rc1");
        assert_eq!(v("1.0.0A1").render(), "1.0.0a1");
    }

    #[test]
    fn parse_rejects_garbage() {
        assert!(parse_version("").is_none());
        assert!(parse_version("hello").is_none());
        assert!(parse_version(".").is_none());
    }

    #[test]
    fn bump_major_zeros_lower_segments_and_clears_pre() {
        let out = bump(&v("1.2.3a4.dev5"), BumpKind::Major);
        assert_eq!(out.render(), "2.0.0");
    }

    #[test]
    fn bump_minor_zeros_patch_and_clears_pre() {
        let out = bump(&v("1.2.3rc1"), BumpKind::Minor);
        assert_eq!(out.render(), "1.3.0");
    }

    #[test]
    fn bump_patch_increments_third_segment() {
        let out = bump(&v("1.2.3"), BumpKind::Patch);
        assert_eq!(out.render(), "1.2.4");
    }

    #[test]
    fn bump_patch_from_short_release_grows_segments() {
        let out = bump(&v("1.2"), BumpKind::Patch);
        assert_eq!(out.render(), "1.2.1");
    }

    #[test]
    fn bump_alpha_from_release_bumps_patch_first() {
        let out = bump(&v("1.2.3"), BumpKind::Alpha);
        assert_eq!(out.render(), "1.2.4a1");
    }

    #[test]
    fn bump_alpha_on_alpha_increments_counter() {
        let out = bump(&v("1.2.3a1"), BumpKind::Alpha);
        assert_eq!(out.render(), "1.2.3a2");
    }

    #[test]
    fn bump_beta_from_alpha_promotes_and_resets_counter() {
        let out = bump(&v("1.2.3a5"), BumpKind::Beta);
        assert_eq!(out.render(), "1.2.3b1");
    }

    #[test]
    fn bump_rc_from_beta_promotes_and_resets_counter() {
        let out = bump(&v("1.2.3b2"), BumpKind::Rc);
        assert_eq!(out.render(), "1.2.3rc1");
    }

    #[test]
    fn bump_alpha_from_beta_moves_to_next_patch() {
        // Can't go beta -> alpha on the same release; advance patch.
        let out = bump(&v("1.2.3b2"), BumpKind::Alpha);
        assert_eq!(out.render(), "1.2.4a1");
    }

    #[test]
    fn bump_post_attaches_post1_then_increments() {
        let one = bump(&v("1.2.3"), BumpKind::Post);
        assert_eq!(one.render(), "1.2.3.post1");
        let two = bump(&one, BumpKind::Post);
        assert_eq!(two.render(), "1.2.3.post2");
    }

    #[test]
    fn bump_dev_attaches_dev1_then_increments() {
        let one = bump(&v("1.2.3"), BumpKind::Dev);
        assert_eq!(one.render(), "1.2.3.dev1");
        let two = bump(&one, BumpKind::Dev);
        assert_eq!(two.render(), "1.2.3.dev2");
    }

    #[test]
    fn bump_release_drops_pre_and_dev_but_keeps_post() {
        let out = bump(&v("1.2.3a4.post1.dev2"), BumpKind::Release);
        assert_eq!(out.render(), "1.2.3.post1");
    }

    #[test]
    fn version_ordering_obeys_pep440() {
        assert!(v("1.0.0a1") < v("1.0.0b1"));
        assert!(v("1.0.0b1") < v("1.0.0rc1"));
        assert!(v("1.0.0rc1") < v("1.0.0"));
        assert!(v("1.0.0") < v("1.0.0.post1"));
        assert!(v("1.0.0.dev1") < v("1.0.0a1"));
        assert!(v("1.0.0") < v("1.0.1"));
        assert!(v("1!1.0.0") > v("0!9999.0.0"));
    }

    #[test]
    fn read_pyproject_version_pulls_from_project_table() {
        let src = "\
[project]
name = \"demo\"
version = \"1.2.3\"
";
        assert_eq!(read_pyproject_version(src), Some("1.2.3".into()));
    }

    #[test]
    fn read_pyproject_version_ignores_tool_tables() {
        let src = "\
[tool.poetry]
version = \"0.1.0\"
";
        assert_eq!(read_pyproject_version(src), None);
    }

    #[test]
    fn write_pyproject_version_minimal_diff() {
        let src = "\
[project]
name = \"demo\"
version = \"1.2.3\"
description = \"x\"
";
        let out = write_pyproject_version(src, "1.2.4").unwrap();
        assert!(out.contains("version = \"1.2.4\""));
        // Surrounding lines preserved.
        assert!(out.contains("name = \"demo\""));
        assert!(out.contains("description = \"x\""));
    }

    #[test]
    fn write_pyproject_version_preserves_leading_whitespace() {
        let src = "[project]\n  version = \"1.0.0\"\n";
        let out = write_pyproject_version(src, "1.0.1").unwrap();
        // Original leading 2-space indent retained.
        assert!(out.contains("  version = \"1.0.1\""));
    }

    #[test]
    fn write_pyproject_version_only_touches_project_table() {
        let src = "\
[tool.poetry]
version = \"0.1.0\"

[project]
version = \"1.2.3\"
";
        let out = write_pyproject_version(src, "1.2.4").unwrap();
        // Only the project version was rewritten.
        assert!(out.contains("[tool.poetry]\nversion = \"0.1.0\""));
        assert!(out.contains("[project]\nversion = \"1.2.4\""));
    }

    #[test]
    fn write_pyproject_version_errors_when_missing() {
        let src = "[project]\nname = \"x\"\n";
        assert!(write_pyproject_version(src, "1.0.0").is_err());
    }

    #[test]
    fn write_pyproject_version_errors_when_project_table_missing() {
        let src = "[tool.poetry]\nversion = \"0.1.0\"\n";
        assert!(write_pyproject_version(src, "1.0.0").is_err());
    }

    #[test]
    fn bump_then_writeback_roundtrip_is_clean() {
        let src = "[project]\nname = \"demo\"\nversion = \"1.2.3\"\n";
        let cur = read_pyproject_version(src).unwrap();
        let next = bump(&parse_version(&cur).unwrap(), BumpKind::Minor).render();
        let out = write_pyproject_version(src, &next).unwrap();
        assert_eq!(read_pyproject_version(&out).as_deref(), Some("1.3.0"));
    }

    #[test]
    fn epoch_segments_render_with_bang() {
        assert_eq!(v("2!1.0.0").epoch, 2);
        assert_eq!(v("2!1.0.0").render(), "2!1.0.0");
    }

    #[test]
    fn local_segment_is_preserved_through_bump_when_no_release_change() {
        // The Post bump preserves pre + release, but PEP 440 says local
        // metadata is excluded from public releases — confirm Post still
        // carries it as data (we don't strip), but Patch resets it.
        let pinned = v("1.0.0+ci");
        let post = bump(&pinned, BumpKind::Post);
        // PEP 440 canonical form puts local segment last: release post local.
        assert_eq!(post.render(), "1.0.0.post1+ci");
        let patch = bump(&pinned, BumpKind::Patch);
        assert_eq!(patch.render(), "1.0.1");
    }
}
