// `uv pip list` / `uv pip show` / `uv pip freeze` — installed-package
// inventory (Tick 44).
//
// Pure data layer + a thin filesystem walker.
//
//   * `enumerate_installed(site_packages)` walks a Python `site-packages`
//     directory, locates `*.dist-info/METADATA` files (PEP 376), parses
//     each, and returns an `InstalledDist` row.
//   * `render_freeze`, `render_list`, `render_show` produce the same
//     ASCII output `pip freeze` / `pip list` / `pip show` emit, so
//     downstream tooling (CI scripts, snapshot tests, etc.) can be
//     migrated to mamba's flag-compatible verbs without diffs.
//
// Format notes:
//   * METADATA is RFC 822-style (PEP 241 + PEP 685): one header per line,
//     case-insensitive names, repeated `Requires-Dist` lines, and an
//     optional blank-line-separated long-description body. We parse just
//     enough headers to feed `pip show` faithfully.
//   * `pip list --format=columns` (the default) pads the Package column
//     to the widest name, then a single space, then the Version column.
//   * `pip freeze` emits `name==version` sorted by PEP 503-normalized
//     name; editable installs use `-e <url>` but we don't carry those
//     in this Tick (deferred to the editable-install reader Tick).

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use crate::pkgmanage::pkgmgr::name_normalize::pep503_normalize;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InstalledDist {
    /// PEP 503 normalized name (`requests`, `my-pkg`).
    pub canonical_name: String,
    /// Display name as recorded in METADATA (`Requests`, `My-Pkg`).
    pub name: String,
    pub version: String,
    /// Absolute path to the `*.dist-info` directory.
    pub dist_info: PathBuf,
    /// Optional one-line summary from `Summary:`.
    pub summary: Option<String>,
    /// All `Requires-Dist:` values, verbatim, in declaration order.
    pub requires: Vec<String>,
    /// `Home-page:` URL or first `Project-URL:` value as a fallback.
    pub home_page: Option<String>,
    /// `Author:` value (concatenated with `Author-email:` when present).
    pub author: Option<String>,
    /// `License:` value, if any.
    pub license: Option<String>,
}

/// Knobs for the column / freeze emitters.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ListOptions {
    /// Print the leading `Package   Version` header line (default true).
    pub include_header: bool,
    /// Sort the output ascending by version instead of by name.
    pub sort_by_version: bool,
}

impl Default for ListOptions {
    fn default() -> Self {
        ListOptions {
            include_header: true,
            sort_by_version: false,
        }
    }
}

/// Walk a `site-packages` directory and return the parsed dist-info rows.
/// Missing dir → empty vec (`pip list` against a non-existent venv is
/// a soft no-op).
pub fn enumerate_installed(site_packages: &Path) -> Vec<InstalledDist> {
    let entries = match std::fs::read_dir(site_packages) {
        Ok(e) => e,
        Err(_) => return Vec::new(),
    };
    let mut out = Vec::new();
    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let name = match path.file_name().and_then(|n| n.to_str()) {
            Some(n) => n,
            None => continue,
        };
        if !name.ends_with(".dist-info") {
            continue;
        }
        let metadata_path = path.join("METADATA");
        let body = match std::fs::read_to_string(&metadata_path) {
            Ok(b) => b,
            Err(_) => continue,
        };
        if let Some(dist) = parse_metadata(&body, &path) {
            out.push(dist);
        }
    }
    out.sort_by(|a, b| a.canonical_name.cmp(&b.canonical_name));
    out
}

/// Parse a single METADATA body into an `InstalledDist`. Returns `None`
/// when the `Name:` or `Version:` header is missing — both are required
/// per PEP 241.
pub fn parse_metadata(src: &str, dist_info: &Path) -> Option<InstalledDist> {
    let mut name: Option<String> = None;
    let mut version: Option<String> = None;
    let mut summary: Option<String> = None;
    let mut home_page: Option<String> = None;
    let mut author: Option<String> = None;
    let mut author_email: Option<String> = None;
    let mut license: Option<String> = None;
    let mut requires: Vec<String> = Vec::new();
    let mut project_url_fallback: Option<String> = None;

    for line in src.lines() {
        if line.is_empty() {
            // Header section ends at the first blank line.
            break;
        }
        if !line.contains(':') {
            // Continuation line of the previous header (RFC 822 folding);
            // we ignore for the headers we currently track.
            continue;
        }
        let (key, value) = match line.split_once(':') {
            Some(kv) => kv,
            None => continue,
        };
        let value = value.trim();
        match key.to_ascii_lowercase().as_str() {
            "name" => name = Some(value.to_string()),
            "version" => version = Some(value.to_string()),
            "summary" => summary = Some(value.to_string()),
            "home-page" => home_page = Some(value.to_string()),
            "author" => author = Some(value.to_string()),
            "author-email" => author_email = Some(value.to_string()),
            "license" => license = Some(value.to_string()),
            "requires-dist" => requires.push(value.to_string()),
            "project-url" => {
                if project_url_fallback.is_none() {
                    if let Some((_label, url)) = value.split_once(',') {
                        project_url_fallback = Some(url.trim().to_string());
                    } else {
                        project_url_fallback = Some(value.to_string());
                    }
                }
            }
            _ => {}
        }
    }

    let name = name?;
    let version = version?;
    let home_page = home_page.or(project_url_fallback);
    let author = match (author, author_email) {
        (Some(a), Some(e)) => Some(format!("{a} <{e}>")),
        (Some(a), None) => Some(a),
        (None, Some(e)) => Some(e),
        (None, None) => None,
    };

    Some(InstalledDist {
        canonical_name: pep503_normalize(&name),
        name,
        version,
        dist_info: dist_info.to_path_buf(),
        summary,
        requires,
        home_page,
        author,
        license,
    })
}

/// pip's `freeze` output: `name==version\n` lines sorted by canonical
/// name. Output ends with a single trailing newline.
pub fn render_freeze(dists: &[InstalledDist]) -> String {
    let mut rows: Vec<&InstalledDist> = dists.iter().collect();
    rows.sort_by(|a, b| a.canonical_name.cmp(&b.canonical_name));
    let mut out = String::new();
    for d in rows {
        out.push_str(&format!("{}=={}\n", d.name, d.version));
    }
    out
}

/// pip's `list --format=columns` output. Two columns padded to the
/// widest entry. Header row written when `opts.include_header` is true.
pub fn render_list(dists: &[InstalledDist], opts: &ListOptions) -> String {
    if dists.is_empty() {
        return String::new();
    }
    let mut rows: Vec<&InstalledDist> = dists.iter().collect();
    if opts.sort_by_version {
        rows.sort_by(|a, b| a.version.cmp(&b.version));
    } else {
        rows.sort_by(|a, b| a.canonical_name.cmp(&b.canonical_name));
    }
    let name_width = rows
        .iter()
        .map(|d| d.name.len())
        .max()
        .unwrap_or(0)
        .max("Package".len());
    let version_width = rows
        .iter()
        .map(|d| d.version.len())
        .max()
        .unwrap_or(0)
        .max("Version".len());

    let mut out = String::new();
    if opts.include_header {
        out.push_str(&format!(
            "{:nw$} {:vw$}\n",
            "Package",
            "Version",
            nw = name_width,
            vw = version_width
        ));
        out.push_str(&format!(
            "{} {}\n",
            "-".repeat(name_width),
            "-".repeat(version_width)
        ));
    }
    for d in rows {
        out.push_str(&format!(
            "{:nw$} {:vw$}\n",
            d.name,
            d.version,
            nw = name_width,
            vw = version_width
        ));
    }
    out
}

/// pip's `show <name>` output for a single dist. Multiline RFC 822-shaped
/// body. Always ends with a single trailing newline.
pub fn render_show(dist: &InstalledDist) -> String {
    let mut out = String::new();
    out.push_str(&format!("Name: {}\n", dist.name));
    out.push_str(&format!("Version: {}\n", dist.version));
    if let Some(s) = &dist.summary {
        out.push_str(&format!("Summary: {s}\n"));
    }
    if let Some(h) = &dist.home_page {
        out.push_str(&format!("Home-page: {h}\n"));
    }
    if let Some(a) = &dist.author {
        out.push_str(&format!("Author: {a}\n"));
    }
    if let Some(l) = &dist.license {
        out.push_str(&format!("License: {l}\n"));
    }
    out.push_str(&format!("Location: {}\n", dist.dist_info.display()));
    if !dist.requires.is_empty() {
        out.push_str("Requires: ");
        // pip lists just the project-name prefixes, comma-separated.
        let names: Vec<String> = dist
            .requires
            .iter()
            .map(|r| extract_requirement_name(r))
            .collect();
        out.push_str(&names.join(", "));
        out.push('\n');
    }
    out
}

/// Lookup a single dist by name (PEP 503 normalized match). Returns
/// `None` when nothing in `dists` matches.
pub fn find_by_name<'a>(dists: &'a [InstalledDist], name: &str) -> Option<&'a InstalledDist> {
    let key = pep503_normalize(name);
    dists.iter().find(|d| d.canonical_name == key)
}

/// Group a list of dists by their PEP 503 canonical name so callers can
/// detect duplicates (e.g. a venv with both `Foo-1.0.dist-info` and
/// `foo-1.1.dist-info`).
pub fn group_by_canonical(dists: &[InstalledDist]) -> BTreeMap<String, Vec<&InstalledDist>> {
    let mut out: BTreeMap<String, Vec<&InstalledDist>> = BTreeMap::new();
    for d in dists {
        out.entry(d.canonical_name.clone()).or_default().push(d);
    }
    out
}

fn extract_requirement_name(req: &str) -> String {
    // PEP 508: stop at the first non-name char. Names are `[A-Za-z0-9_.-]+`.
    let mut end = 0;
    for (i, c) in req.char_indices() {
        if c.is_ascii_alphanumeric() || c == '-' || c == '_' || c == '.' {
            end = i + c.len_utf8();
        } else {
            break;
        }
    }
    req[..end].to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn fake_dist(name: &str, version: &str) -> InstalledDist {
        InstalledDist {
            canonical_name: pep503_normalize(name),
            name: name.into(),
            version: version.into(),
            dist_info: PathBuf::from(format!("/fake/{name}-{version}.dist-info")),
            summary: None,
            requires: Vec::new(),
            home_page: None,
            author: None,
            license: None,
        }
    }

    #[test]
    fn parse_metadata_extracts_required_headers() {
        let body = "\
Metadata-Version: 2.1
Name: Requests
Version: 2.31.0
Summary: Python HTTP for Humans.
Home-page: https://requests.readthedocs.io
Author: Kenneth Reitz
Author-email: me@kennethreitz.org
License: Apache 2.0
Requires-Dist: charset-normalizer (<4,>=2)
Requires-Dist: idna (<4,>=2.5)
Requires-Python: >=3.7

Long description follows...
";
        let dist = parse_metadata(body, Path::new("/x/requests-2.31.0.dist-info")).unwrap();
        assert_eq!(dist.name, "Requests");
        assert_eq!(dist.canonical_name, "requests");
        assert_eq!(dist.version, "2.31.0");
        assert_eq!(dist.summary.as_deref(), Some("Python HTTP for Humans."));
        assert_eq!(
            dist.author.as_deref(),
            Some("Kenneth Reitz <me@kennethreitz.org>")
        );
        assert_eq!(dist.license.as_deref(), Some("Apache 2.0"));
        assert_eq!(dist.requires.len(), 2);
        assert_eq!(
            dist.home_page.as_deref(),
            Some("https://requests.readthedocs.io")
        );
    }

    #[test]
    fn parse_metadata_fallback_project_url_when_home_page_missing() {
        let body = "\
Name: x
Version: 1
Project-URL: docs, https://example.org/docs
";
        let d = parse_metadata(body, Path::new("/x.dist-info")).unwrap();
        assert_eq!(d.home_page.as_deref(), Some("https://example.org/docs"));
    }

    #[test]
    fn parse_metadata_rejects_missing_required() {
        assert!(parse_metadata("Name: x\n", Path::new("/")).is_none());
        assert!(parse_metadata("Version: 1\n", Path::new("/")).is_none());
        assert!(parse_metadata("", Path::new("/")).is_none());
    }

    #[test]
    fn parse_metadata_handles_email_only_author() {
        let body = "Name: x\nVersion: 1\nAuthor-email: x@y\n";
        let d = parse_metadata(body, Path::new("/")).unwrap();
        assert_eq!(d.author.as_deref(), Some("x@y"));
    }

    #[test]
    fn enumerate_installed_walks_site_packages() {
        let tmp = tempfile::tempdir().unwrap();
        let dist_dir = tmp.path().join("requests-2.31.0.dist-info");
        fs::create_dir(&dist_dir).unwrap();
        fs::write(
            dist_dir.join("METADATA"),
            "Name: Requests\nVersion: 2.31.0\n",
        )
        .unwrap();
        let other = tmp.path().join("urllib3-2.1.0.dist-info");
        fs::create_dir(&other).unwrap();
        fs::write(other.join("METADATA"), "Name: urllib3\nVersion: 2.1.0\n").unwrap();
        // Stray non-dist-info dir must be ignored.
        fs::create_dir(tmp.path().join("__pycache__")).unwrap();

        let dists = enumerate_installed(tmp.path());
        assert_eq!(dists.len(), 2);
        assert_eq!(dists[0].canonical_name, "requests");
        assert_eq!(dists[1].canonical_name, "urllib3");
    }

    #[test]
    fn enumerate_missing_dir_is_empty() {
        let nowhere = std::env::temp_dir().join("definitely-does-not-exist-mamba-test");
        let _ = std::fs::remove_dir_all(&nowhere);
        assert!(enumerate_installed(&nowhere).is_empty());
    }

    #[test]
    fn enumerate_skips_dist_info_without_metadata() {
        let tmp = tempfile::tempdir().unwrap();
        let dir = tmp.path().join("broken-1.0.dist-info");
        fs::create_dir(&dir).unwrap();
        // No METADATA file written.
        assert!(enumerate_installed(tmp.path()).is_empty());
    }

    #[test]
    fn render_freeze_emits_alphabetized_pins() {
        let dists = vec![
            fake_dist("Zeta", "1.0"),
            fake_dist("alpha", "2.0"),
            fake_dist("My-Pkg", "3.0"),
        ];
        assert_eq!(
            render_freeze(&dists),
            "alpha==2.0\nMy-Pkg==3.0\nZeta==1.0\n"
        );
    }

    #[test]
    fn render_list_pads_columns_to_widest_value() {
        let dists = vec![fake_dist("a", "1.0"), fake_dist("longer-name", "11.22.33")];
        let out = render_list(&dists, &ListOptions::default());
        // Both columns must be at least as wide as the widest entry +
        // the header.
        assert!(out.lines().next().unwrap().starts_with("Package"));
        // Underline row uses dashes matching column widths.
        let underline = out.lines().nth(1).unwrap();
        assert!(underline.starts_with("---"));
        assert!(out.contains("longer-name 11.22.33"));
    }

    #[test]
    fn render_list_omits_header_when_disabled() {
        let dists = vec![fake_dist("a", "1.0")];
        let out = render_list(
            &dists,
            &ListOptions {
                include_header: false,
                ..Default::default()
            },
        );
        assert!(!out.contains("Package"));
        assert!(out.contains("a"));
    }

    #[test]
    fn render_list_sort_by_version() {
        let dists = vec![
            fake_dist("a", "9.0"),
            fake_dist("b", "1.0"),
            fake_dist("c", "5.0"),
        ];
        let out = render_list(
            &dists,
            &ListOptions {
                include_header: false,
                sort_by_version: true,
            },
        );
        let body_lines: Vec<&str> = out.lines().collect();
        // First listed = version 1.0 (b), then 5.0, then 9.0.
        assert!(body_lines[0].starts_with("b "));
        assert!(body_lines[1].starts_with("c "));
        assert!(body_lines[2].starts_with("a "));
    }

    #[test]
    fn render_show_emits_pip_compatible_body() {
        let mut d = fake_dist("Requests", "2.31.0");
        d.summary = Some("Python HTTP for Humans.".into());
        d.home_page = Some("https://requests.readthedocs.io".into());
        d.author = Some("Kenneth Reitz <me@kennethreitz.org>".into());
        d.license = Some("Apache 2.0".into());
        d.requires = vec![
            "charset-normalizer (<4,>=2)".into(),
            "idna (<4,>=2.5)".into(),
        ];

        let out = render_show(&d);
        assert!(out.contains("Name: Requests"));
        assert!(out.contains("Version: 2.31.0"));
        assert!(out.contains("Summary: Python HTTP for Humans."));
        assert!(out.contains("Home-page: https://requests.readthedocs.io"));
        assert!(out.contains("Author: Kenneth Reitz <me@kennethreitz.org>"));
        assert!(out.contains("License: Apache 2.0"));
        // Requires section lists project names only (no version spec).
        assert!(out.contains("Requires: charset-normalizer, idna"));
    }

    #[test]
    fn render_show_omits_unset_optional_fields() {
        let d = fake_dist("x", "1.0");
        let out = render_show(&d);
        assert!(!out.contains("Summary:"));
        assert!(!out.contains("Author:"));
        assert!(!out.contains("Requires:"));
        // Name, Version, Location always present.
        assert!(out.contains("Name: x"));
        assert!(out.contains("Version: 1.0"));
        assert!(out.contains("Location:"));
    }

    #[test]
    fn find_by_name_uses_pep503_match() {
        let dists = vec![fake_dist("My-Pkg", "1.0")];
        assert!(find_by_name(&dists, "my_pkg").is_some());
        assert!(find_by_name(&dists, "MY.PKG").is_some());
        assert!(find_by_name(&dists, "other").is_none());
    }

    #[test]
    fn group_by_canonical_collects_duplicates() {
        let dists = vec![
            fake_dist("Foo", "1.0"),
            fake_dist("foo", "2.0"),
            fake_dist("bar", "1.0"),
        ];
        let groups = group_by_canonical(&dists);
        assert_eq!(groups.get("foo").unwrap().len(), 2);
        assert_eq!(groups.get("bar").unwrap().len(), 1);
    }

    #[test]
    fn extract_requirement_name_stops_at_specifier() {
        assert_eq!(
            extract_requirement_name("charset-normalizer (<4,>=2)"),
            "charset-normalizer"
        );
        assert_eq!(extract_requirement_name("idna>=2.5"), "idna");
        assert_eq!(
            extract_requirement_name("flask ; python_version>='3.7'"),
            "flask"
        );
        assert_eq!(extract_requirement_name("a.b.c"), "a.b.c");
    }

    #[test]
    fn freeze_then_render_round_trip_matches_alphabetical_canonical_order() {
        let dists = vec![fake_dist("Zeta", "1.0"), fake_dist("alpha", "2.0")];
        let body = render_freeze(&dists);
        // Re-sort the expected canonical order: alpha (a) < zeta (z).
        let lines: Vec<&str> = body.lines().collect();
        assert_eq!(lines, vec!["alpha==2.0", "Zeta==1.0"]);
    }

    #[test]
    fn empty_input_yields_empty_freeze_and_list() {
        assert_eq!(render_freeze(&[]), "");
        assert_eq!(render_list(&[], &ListOptions::default()), "");
    }
}
