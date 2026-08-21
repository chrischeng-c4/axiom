// VCS direct-reference URL parser (Tick 121).
//
// pip / uv accept four VCS schemes as PEP 440 Direct References:
//
//   git+<transport>://<location>[@<rev>][#<fragment>]
//   hg+<transport>://<location>[@<rev>][#<fragment>]
//   svn+<transport>://<location>[@<rev>][#<fragment>]
//   bzr+<transport>://<location>[@<rev>][#<fragment>]
//
// where `<transport>` is one of `https`, `http`, `ssh`, `git`, `file`,
// or scheme-specific (`svn` for Subversion's native protocol). The
// fragment is a `&`-joined query-style key=value list, of which
// `subdirectory=<path>` and `egg=<name>` are recognized by every PEP /
// pip / uv consumer.
//
// `direct_url.rs` only decodes `git+` and is embedded inside the
// `Requirement` parser. This module is the reusable primitive used by
// every other call site that needs to recognize a VCS reference:
//   * `lockfile.rs` — source kind classification
//   * `uv_sources.rs` — `[tool.uv.sources].<name> = { git = "..." }`
//   * `requirements_parse.rs` — `pip install -r requirements.txt` lines
//   * `pylock_export.rs` — `pylock.toml` source entries
//
// scp-form normalization: `git@github.com:foo/bar.git` is the legacy
// scp-style shorthand that git understands but is not a URL. uv (like
// pip ≥21.3) accepts it inside the post-`git+` body and rewrites it
// to `ssh://git@github.com/foo/bar.git` before recording it. We mirror
// that rewrite here so downstream code sees a single canonical form.

use crate::pkgmanage::pkgmgr::types::IndexError;
use std::collections::BTreeMap;

const VCS_URL_DETAIL: &str = "<vcs URL>";

/// The four PEP 440 / pip-compatible VCS schemes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum VcsKind {
    Git,
    Hg,
    Svn,
    Bzr,
}

impl VcsKind {
    /// Stable lower-case identifier for use in lockfile source-kind
    /// fields, `direct_url.json` `vcs` keys, and round-trip printing.
    pub fn as_str(self) -> &'static str {
        match self {
            VcsKind::Git => "git",
            VcsKind::Hg => "hg",
            VcsKind::Svn => "svn",
            VcsKind::Bzr => "bzr",
        }
    }

    /// Recognize the VCS prefix at the start of a string.
    /// Returns `(kind, rest_after_plus)` where `rest_after_plus` is the
    /// transport URL with the `<vcs>+` prefix stripped. None when no
    /// recognized VCS prefix is present.
    pub fn strip_prefix(src: &str) -> Option<(Self, &str)> {
        for (kind, prefix) in [
            (VcsKind::Git, "git+"),
            (VcsKind::Hg, "hg+"),
            (VcsKind::Svn, "svn+"),
            (VcsKind::Bzr, "bzr+"),
        ] {
            if let Some(rest) = src.strip_prefix(prefix) {
                return Some((kind, rest));
            }
        }
        None
    }
}

/// Decoded VCS direct-reference URL.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VcsUrl {
    /// Which VCS this URL points at.
    pub kind: VcsKind,
    /// Transport URL with the `<vcs>+` prefix and any `@<rev>` suffix
    /// stripped. e.g. `https://github.com/foo/bar.git`.
    pub url: String,
    /// Revision pin — branch, tag, or full/short commit hash. None when
    /// no `@<rev>` suffix was present (treated as "default branch" by
    /// every implementation).
    pub rev: Option<String>,
    /// Fragment subdirectory hint — points at the project root inside
    /// a monorepo checkout. Mirrors `#subdirectory=...` per pip.
    pub subdirectory: Option<String>,
    /// Legacy `#egg=<name>` hint. pip-deprecated but uv still reads it
    /// for backwards compatibility with old `requirements.txt` files.
    pub egg: Option<String>,
    /// Any unrecognized fragment keys, preserved verbatim. Stable order
    /// (`BTreeMap`) so round-trip prints are deterministic.
    pub extra_fragment: BTreeMap<String, String>,
}

impl VcsUrl {
    /// Parse a `<vcs>+<transport>://...` direct reference.
    ///
    /// Returns `Ok(None)` when `src` has no recognized VCS prefix —
    /// callers should treat this as "not a VCS URL" and fall through
    /// to their existing transport/path handler. Returns `Err` only for
    /// malformed inputs that have a VCS prefix but a broken body
    /// (empty transport URL, unterminated fragment value, etc.).
    pub fn parse(src: &str) -> Result<Option<Self>, IndexError> {
        let Some((kind, rest)) = VcsKind::strip_prefix(src) else {
            return Ok(None);
        };
        Self::parse_body(kind, rest).map(Some)
    }

    /// Parse the body of a VCS URL where the caller has already
    /// identified the VCS kind (e.g. from a `direct_url.json` `vcs`
    /// field rather than a prefix).
    pub fn parse_body(kind: VcsKind, rest: &str) -> Result<Self, IndexError> {
        if rest.is_empty() {
            return Err(IndexError::ParseError {
                url: VCS_URL_DETAIL.into(),
                detail: format!(
                    "{} URL has empty body after {}+ prefix",
                    kind.as_str(),
                    kind.as_str()
                ),
            });
        }

        // Split off the fragment first — anything after the first `#`.
        let (head, fragment) = match rest.find('#') {
            Some(idx) => (&rest[..idx], Some(&rest[idx + 1..])),
            None => (rest, None),
        };

        // Then split off the @<rev> suffix from the transport body.
        // The split must be the LAST `@`, because the userinfo part of
        // an ssh URL (`ssh://git@host/...`) also contains `@`. We split
        // from the right but never inside the scheme part.
        let (transport_url, rev) = split_url_and_rev(head)?;

        if transport_url.is_empty() {
            return Err(IndexError::ParseError {
                url: VCS_URL_DETAIL.into(),
                detail: format!("{} URL has empty transport body", kind.as_str()),
            });
        }

        // Normalize scp-style `git@host:path` to `ssh://git@host/path`.
        let url = normalize_scp_form(transport_url);

        // Decode the fragment dict.
        let (subdirectory, egg, extra_fragment) = match fragment {
            Some(frag) => decode_fragment(frag)?,
            None => (None, None, BTreeMap::new()),
        };

        Ok(VcsUrl {
            kind,
            url,
            rev,
            subdirectory,
            egg,
            extra_fragment,
        })
    }

    /// Round-trip back to the canonical `<vcs>+<transport>://...` form.
    /// Always emits keys in `subdirectory`, `egg`, then `extra_fragment`
    /// alphabetical order; joined with `&` per pip convention.
    pub fn render(&self) -> String {
        let mut out = format!("{}+{}", self.kind.as_str(), self.url);
        if let Some(rev) = &self.rev {
            out.push('@');
            out.push_str(rev);
        }
        let mut frags: Vec<String> = Vec::new();
        if let Some(s) = &self.subdirectory {
            frags.push(format!("subdirectory={s}"));
        }
        if let Some(e) = &self.egg {
            frags.push(format!("egg={e}"));
        }
        for (k, v) in &self.extra_fragment {
            frags.push(format!("{k}={v}"));
        }
        if !frags.is_empty() {
            out.push('#');
            out.push_str(&frags.join("&"));
        }
        out
    }
}

/// Split a URL body into `(url_without_rev, optional_rev)`.
///
/// The tricky case is distinguishing the `@` in `ssh://git@host/path`
/// (userinfo separator) from the `@` in `https://host/path@tag` (rev
/// separator). pip's rule: the rev `@` always appears in the *path*
/// portion of the URL — i.e. after the first `/` that follows `://`.
/// So we scan from the start of the path forward; any earlier `@` is
/// userinfo.
///
/// scp-form (`git@host:path`) has no `://` and no leading `/`; we
/// special-case it as "no rev possible at the userinfo `@`".
fn split_url_and_rev(body: &str) -> Result<(&str, Option<String>), IndexError> {
    let scheme_end = body.find("://").map(|i| i + 3);

    // path_start = index in `body` where the path part begins.
    // For URL-form: the first `/` after `://`.
    // For scp-form (no `://`): no path-`@` is possible, scanning starts
    //   past the userinfo `@` so it's treated as no-rev.
    let path_start = match scheme_end {
        Some(end) => {
            // First `/` after the authority. If absent (e.g. `lp:foo`
            // bzr launchpad shorthand), scan from `end` directly.
            body[end..].find('/').map(|rel| end + rel).unwrap_or(end)
        }
        None => {
            // Scheme-less body. Two variants to disambiguate:
            //   scp-form: `user@host:path[@rev]` — the `@` BEFORE the
            //     first `:` is userinfo, must be skipped.
            //   shorthand: `lp:project[@rev]` (bzr launchpad style) —
            //     no userinfo, the `@` IS the rev separator.
            // Rule: only skip the leading `@` when it appears before
            // the first `:` (i.e. it's a userinfo separator on the
            // authority portion). Otherwise scan from the start.
            match (body.find('@'), body.find(':')) {
                (Some(at), Some(colon)) if at < colon => at + 1,
                _ => 0,
            }
        }
    };

    // Look for `@` in the path portion only. The first match wins —
    // revs themselves can contain `/` (e.g. `release/1.0`) but cannot
    // contain `@` in practice.
    let Some(at_offset_in_path) = body[path_start..].find('@') else {
        return Ok((body, None));
    };
    let split_at = path_start + at_offset_in_path;

    let url = &body[..split_at];
    let rev = body[split_at + 1..].to_string();
    if rev.is_empty() {
        return Err(IndexError::ParseError {
            url: VCS_URL_DETAIL.into(),
            detail: "empty revision after `@`".into(),
        });
    }
    Ok((url, Some(rev)))
}

/// Convert scp-form `user@host:path` to `ssh://user@host/path`.
/// pip ≥21.3 and uv both rewrite scp-form before storing so downstream
/// recorders see a single canonical URL form. We mirror that here.
///
/// Identification: an scp-form URL has no `://`, contains a `:` whose
/// right-hand side does NOT start with `//`, and a `@` before the `:`.
fn normalize_scp_form(url: &str) -> String {
    if url.contains("://") {
        return url.to_string();
    }
    let Some(colon_idx) = url.find(':') else {
        return url.to_string();
    };
    let head = &url[..colon_idx];
    let tail = &url[colon_idx + 1..];
    // Must have a `@` in the head (the userinfo separator) and a
    // non-empty path on the tail. Reject pure host:port.
    if !head.contains('@') {
        return url.to_string();
    }
    // Heuristic: if tail starts with a digit and only digits up to '/' it
    // probably is host:port (uncommon for VCS). Leave alone.
    if tail.chars().next().is_some_and(|c| c.is_ascii_digit())
        && tail
            .split('/')
            .next()
            .is_some_and(|p| p.chars().all(|c| c.is_ascii_digit()))
    {
        return url.to_string();
    }
    format!("ssh://{head}/{tail}")
}

/// Decode a `key=value&key=value` fragment string into
/// `(subdirectory, egg, extras)`. Repeated keys keep the last value
/// (matching pip / browser query-string behavior).
fn decode_fragment(
    frag: &str,
) -> Result<(Option<String>, Option<String>, BTreeMap<String, String>), IndexError> {
    let mut subdirectory: Option<String> = None;
    let mut egg: Option<String> = None;
    let mut extras: BTreeMap<String, String> = BTreeMap::new();

    if frag.is_empty() {
        return Ok((subdirectory, egg, extras));
    }

    for pair in frag.split('&') {
        if pair.is_empty() {
            continue;
        }
        let Some(eq) = pair.find('=') else {
            return Err(IndexError::ParseError {
                url: VCS_URL_DETAIL.into(),
                detail: format!("fragment entry {pair:?} has no `=`"),
            });
        };
        let key = &pair[..eq];
        let value = &pair[eq + 1..];
        if key.is_empty() {
            return Err(IndexError::ParseError {
                url: VCS_URL_DETAIL.into(),
                detail: format!("fragment entry {pair:?} has empty key"),
            });
        }
        match key {
            "subdirectory" => subdirectory = Some(value.to_string()),
            "egg" => egg = Some(value.to_string()),
            other => {
                extras.insert(other.to_string(), value.to_string());
            }
        }
    }

    Ok((subdirectory, egg, extras))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn returns_none_when_no_vcs_prefix() {
        assert!(VcsUrl::parse("https://example.com/foo.tar.gz")
            .unwrap()
            .is_none());
        assert!(VcsUrl::parse("file:///tmp/foo").unwrap().is_none());
        assert!(VcsUrl::parse("foo").unwrap().is_none());
    }

    #[test]
    fn recognizes_all_four_schemes() {
        let cases = [
            (
                "git+https://example.com/foo.git",
                VcsKind::Git,
                "https://example.com/foo.git",
            ),
            (
                "hg+https://example.com/foo",
                VcsKind::Hg,
                "https://example.com/foo",
            ),
            (
                "svn+https://example.com/svn/foo",
                VcsKind::Svn,
                "https://example.com/svn/foo",
            ),
            (
                "bzr+https://example.com/foo",
                VcsKind::Bzr,
                "https://example.com/foo",
            ),
        ];
        for (src, kind, url) in cases {
            let u = VcsUrl::parse(src).unwrap().unwrap();
            assert_eq!(u.kind, kind);
            assert_eq!(u.url, url);
            assert_eq!(u.rev, None);
        }
    }

    #[test]
    fn parses_rev_pin() {
        let u = VcsUrl::parse("git+https://github.com/foo/bar.git@main")
            .unwrap()
            .unwrap();
        assert_eq!(u.url, "https://github.com/foo/bar.git");
        assert_eq!(u.rev.as_deref(), Some("main"));
    }

    #[test]
    fn parses_commit_sha_rev() {
        let u = VcsUrl::parse("git+https://github.com/foo/bar.git@abc123def")
            .unwrap()
            .unwrap();
        assert_eq!(u.rev.as_deref(), Some("abc123def"));
    }

    #[test]
    fn distinguishes_userinfo_at_from_rev_at() {
        // ssh://git@host/path@tag — the LAST @ is the rev.
        let u = VcsUrl::parse("git+ssh://git@github.com/foo/bar.git@v1.2.3")
            .unwrap()
            .unwrap();
        assert_eq!(u.url, "ssh://git@github.com/foo/bar.git");
        assert_eq!(u.rev.as_deref(), Some("v1.2.3"));
    }

    #[test]
    fn no_rev_when_only_userinfo_at_present() {
        let u = VcsUrl::parse("git+ssh://git@github.com/foo/bar.git")
            .unwrap()
            .unwrap();
        assert_eq!(u.url, "ssh://git@github.com/foo/bar.git");
        assert_eq!(u.rev, None);
    }

    #[test]
    fn parses_subdirectory_fragment() {
        let u = VcsUrl::parse("git+https://github.com/foo/bar.git@main#subdirectory=pkgs/sub")
            .unwrap()
            .unwrap();
        assert_eq!(u.rev.as_deref(), Some("main"));
        assert_eq!(u.subdirectory.as_deref(), Some("pkgs/sub"));
        assert_eq!(u.egg, None);
    }

    #[test]
    fn parses_egg_fragment() {
        let u = VcsUrl::parse("git+https://github.com/foo/bar.git@main#egg=bar")
            .unwrap()
            .unwrap();
        assert_eq!(u.egg.as_deref(), Some("bar"));
        assert_eq!(u.subdirectory, None);
    }

    #[test]
    fn parses_combined_fragment_keys() {
        let u = VcsUrl::parse(
            "git+https://github.com/foo/bar.git@main#egg=bar&subdirectory=pkgs/sub&custom=xyz",
        )
        .unwrap()
        .unwrap();
        assert_eq!(u.egg.as_deref(), Some("bar"));
        assert_eq!(u.subdirectory.as_deref(), Some("pkgs/sub"));
        assert_eq!(
            u.extra_fragment.get("custom").map(String::as_str),
            Some("xyz")
        );
    }

    #[test]
    fn rejects_empty_body() {
        assert!(VcsUrl::parse("git+").is_err());
    }

    #[test]
    fn rejects_empty_rev() {
        assert!(VcsUrl::parse("git+https://example.com/foo.git@").is_err());
    }

    #[test]
    fn rejects_fragment_entry_without_equals() {
        assert!(VcsUrl::parse("git+https://example.com/foo.git#brokenkey").is_err());
    }

    #[test]
    fn rejects_fragment_empty_key() {
        assert!(VcsUrl::parse("git+https://example.com/foo.git#=value").is_err());
    }

    #[test]
    fn empty_fragment_is_ok() {
        let u = VcsUrl::parse("git+https://example.com/foo.git#")
            .unwrap()
            .unwrap();
        assert_eq!(u.subdirectory, None);
        assert_eq!(u.egg, None);
        assert!(u.extra_fragment.is_empty());
    }

    #[test]
    fn file_transport_is_supported() {
        let u = VcsUrl::parse("git+file:///srv/repos/foo@v2.0")
            .unwrap()
            .unwrap();
        assert_eq!(u.url, "file:///srv/repos/foo");
        assert_eq!(u.rev.as_deref(), Some("v2.0"));
    }

    #[test]
    fn normalizes_scp_form_to_ssh() {
        // scp-form `git@host:path` (no scheme) -> `ssh://git@host/path`.
        let u = VcsUrl::parse("git+git@github.com:foo/bar.git")
            .unwrap()
            .unwrap();
        assert_eq!(u.url, "ssh://git@github.com/foo/bar.git");
        assert_eq!(u.rev, None);
    }

    #[test]
    fn normalizes_scp_form_with_rev() {
        let u = VcsUrl::parse("git+git@github.com:foo/bar.git@main")
            .unwrap()
            .unwrap();
        assert_eq!(u.url, "ssh://git@github.com/foo/bar.git");
        assert_eq!(u.rev.as_deref(), Some("main"));
    }

    #[test]
    fn does_not_misclassify_https_as_scp_form() {
        // Has `://`, should not be touched.
        let u = VcsUrl::parse("git+https://github.com/foo/bar.git@main")
            .unwrap()
            .unwrap();
        assert_eq!(u.url, "https://github.com/foo/bar.git");
    }

    #[test]
    fn vcs_kind_strip_prefix_misses_unknown() {
        assert!(VcsKind::strip_prefix("fossil+https://example.com/x").is_none());
        assert!(VcsKind::strip_prefix("https://example.com/x").is_none());
    }

    #[test]
    fn vcs_kind_strip_prefix_returns_remainder() {
        let (kind, rest) = VcsKind::strip_prefix("hg+https://example.com/x").unwrap();
        assert_eq!(kind, VcsKind::Hg);
        assert_eq!(rest, "https://example.com/x");
    }

    #[test]
    fn parse_body_takes_pre_classified_kind() {
        // Mirrors `direct_url.json` decode where `vcs: "git"` and `url`
        // are separate fields.
        let v = VcsUrl::parse_body(VcsKind::Git, "https://github.com/foo/bar.git@main").unwrap();
        assert_eq!(v.kind, VcsKind::Git);
        assert_eq!(v.url, "https://github.com/foo/bar.git");
        assert_eq!(v.rev.as_deref(), Some("main"));
    }

    #[test]
    fn round_trip_render_minimal() {
        let original = "hg+https://example.com/foo";
        let v = VcsUrl::parse(original).unwrap().unwrap();
        assert_eq!(v.render(), original);
    }

    #[test]
    fn round_trip_render_with_rev_and_fragment() {
        let original = "git+https://github.com/foo/bar.git@main#subdirectory=pkgs/sub";
        let v = VcsUrl::parse(original).unwrap().unwrap();
        assert_eq!(v.render(), original);
    }

    #[test]
    fn round_trip_render_orders_subdir_before_egg() {
        // Input order: egg first, then subdirectory.
        // Output order: subdirectory, egg, then alphabetical extras.
        let v = VcsUrl::parse(
            "git+https://example.com/x.git@v1#egg=x&subdirectory=pkgs/x&zzz=last&aaa=first",
        )
        .unwrap()
        .unwrap();
        let out = v.render();
        // Canonical form: subdirectory, egg, aaa, zzz.
        assert_eq!(
            out,
            "git+https://example.com/x.git@v1#subdirectory=pkgs/x&egg=x&aaa=first&zzz=last"
        );
    }

    #[test]
    fn rev_with_slash_in_branch_name_is_preserved() {
        // git branches can contain `/` like `release/1.0`.
        let u = VcsUrl::parse("git+https://example.com/x.git@release/1.0")
            .unwrap()
            .unwrap();
        assert_eq!(u.rev.as_deref(), Some("release/1.0"));
    }

    #[test]
    fn vcs_kind_as_str_round_trip() {
        for k in [VcsKind::Git, VcsKind::Hg, VcsKind::Svn, VcsKind::Bzr] {
            let s = k.as_str();
            // Each should match the prefix without the `+`.
            assert!(VcsKind::strip_prefix(&format!("{s}+ignored")).is_some());
        }
    }

    #[test]
    fn realistic_pep440_direct_reference_examples() {
        // A handful of inputs the pip/uv test corpora ship.
        let cases = [
            (
                "git+https://github.com/pallets/flask.git@2.3.3",
                "https://github.com/pallets/flask.git",
                Some("2.3.3"),
                None,
                None,
            ),
            (
                "git+ssh://git@github.com/foo/bar.git@abc123#subdirectory=pkg",
                "ssh://git@github.com/foo/bar.git",
                Some("abc123"),
                Some("pkg"),
                None,
            ),
            (
                "hg+https://bitbucket.org/owner/repo@stable",
                "https://bitbucket.org/owner/repo",
                Some("stable"),
                None,
                None,
            ),
            (
                "svn+https://svn.example.com/repo/trunk@1234",
                "https://svn.example.com/repo/trunk",
                Some("1234"),
                None,
                None,
            ),
            (
                "bzr+lp:project-name@revision",
                "lp:project-name",
                Some("revision"),
                None,
                None,
            ),
        ];
        for (src, want_url, want_rev, want_sub, want_egg) in cases {
            let u = VcsUrl::parse(src).unwrap().unwrap();
            assert_eq!(u.url, want_url, "url mismatch for {src}");
            assert_eq!(u.rev.as_deref(), want_rev, "rev mismatch for {src}");
            assert_eq!(
                u.subdirectory.as_deref(),
                want_sub,
                "sub mismatch for {src}"
            );
            assert_eq!(u.egg.as_deref(), want_egg, "egg mismatch for {src}");
        }
    }

    #[test]
    fn duplicate_fragment_key_takes_last_value() {
        let u = VcsUrl::parse("git+https://x.com/r.git#subdirectory=a&subdirectory=b")
            .unwrap()
            .unwrap();
        assert_eq!(u.subdirectory.as_deref(), Some("b"));
    }
}
