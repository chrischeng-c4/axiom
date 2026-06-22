// PEP 751 `pylock.toml` writer (Tick 53).
//
// uv supports `uv export --format pylock.toml` to share locks across
// tools. This module turns our internal `Lockfile` into a PEP 751-shaped
// body. Pure-data: no resolver, no I/O.
//
// Reference: PEP 751 — A file format to record Python dependencies for
// installation reproducibility.
//
// Output rules (deterministic):
//   * Header keys emitted in fixed order: `lock-version`, `created-by`,
//     `requires-python`, `environments`.
//   * Packages emitted in PEP 503-normalized-name order.
//   * Inside each `[[packages]]` block keys appear in fixed order: name,
//     version, marker, requires-python, dependencies, sdist, wheels,
//     vcs, directory, archive.
//   * Inline tables use `key = "value"` with double-quoted strings and
//     a single trailing space inside the braces — matches uv's writer.
//
// What this module does NOT cover (deferred):
//   * Multi-artifact per package (sdist + N wheels) — we emit one
//     artifact derived from the `source` URL (sdist *or* wheel depending
//     on the filename extension).
//   * `tools` arrays per artifact — uv writes them for resolver replay,
//     not required for PEP 751 conformance.
//   * `[[packages.dependencies]]` extras / markers — we list bare names.

use crate::pkgmanage::pkgmgr::lockfile::{Lockfile, Package, SourceRefKind};
use crate::pkgmanage::pkgmgr::name_normalize::pep503_normalize;

/// Caller-tunable rendering knobs for the pylock body.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PylockOptions {
    /// PEP 751 `lock-version`. Defaults to `"1.0"`.
    pub lock_version: String,
    /// PEP 751 `created-by`. Defaults to `"mamba"`.
    pub created_by: String,
    /// PEP 751 `requires-python` PEP 440 specifier (optional).
    pub requires_python: Option<String>,
    /// PEP 751 `environments` — PEP 508 marker expressions describing
    /// every environment the lock is meant to apply to.
    pub environments: Vec<String>,
}

impl Default for PylockOptions {
    fn default() -> Self {
        PylockOptions {
            lock_version: "1.0".into(),
            created_by: "mamba".into(),
            requires_python: None,
            environments: Vec::new(),
        }
    }
}

/// Render a `Lockfile` to a PEP 751 `pylock.toml` body.
pub fn render_pylock_toml(lockfile: &Lockfile, opts: &PylockOptions) -> String {
    let mut out = String::new();

    write_kv_string(&mut out, "lock-version", &opts.lock_version);
    write_kv_string(&mut out, "created-by", &opts.created_by);
    if let Some(rp) = &opts.requires_python {
        write_kv_string(&mut out, "requires-python", rp);
    }
    if !opts.environments.is_empty() {
        out.push_str("environments = [\n");
        for env in &opts.environments {
            out.push_str("    \"");
            out.push_str(&escape_string(env));
            out.push_str("\",\n");
        }
        out.push_str("]\n");
    }

    let mut packages: Vec<&Package> = lockfile.packages.iter().collect();
    packages.sort_by(|a, b| pep503_normalize(&a.name).cmp(&pep503_normalize(&b.name)));

    for pkg in packages {
        out.push('\n');
        out.push_str("[[packages]]\n");
        write_kv_string(&mut out, "name", &pkg.name);
        write_kv_string(&mut out, "version", &pkg.version);
        if let Some(m) = &pkg.markers {
            write_kv_string(&mut out, "marker", m);
        }
        if !pkg.dependencies.is_empty() {
            write_dependencies(&mut out, &pkg.dependencies);
        }
        write_artifact(&mut out, pkg);
    }

    out
}

// ---------------------------------------------------------------------------
// Writers
// ---------------------------------------------------------------------------

fn write_kv_string(out: &mut String, key: &str, value: &str) {
    out.push_str(key);
    out.push_str(" = \"");
    out.push_str(&escape_string(value));
    out.push_str("\"\n");
}

fn write_dependencies(out: &mut String, deps: &[String]) {
    out.push_str("dependencies = [\n");
    let mut sorted: Vec<&str> = deps.iter().map(|s| s.as_str()).collect();
    sorted.sort_by_key(|s| pep503_normalize(s));
    for d in sorted {
        out.push_str("    { name = \"");
        out.push_str(&escape_string(d));
        out.push_str("\" },\n");
    }
    out.push_str("]\n");
}

fn write_artifact(out: &mut String, pkg: &Package) {
    let kind = pkg.source_ref.as_ref().map(|s| s.kind);
    match kind {
        Some(SourceRefKind::Git) => write_vcs_artifact(out, pkg),
        Some(SourceRefKind::Path) => write_directory_artifact(out, pkg),
        Some(SourceRefKind::Registry) | None => write_registry_artifact(out, pkg),
    }
}

fn write_registry_artifact(out: &mut String, pkg: &Package) {
    // Decide whether the source URL looks like a wheel or sdist.
    let url = pkg.source.trim();
    let is_wheel = url.ends_with(".whl");
    let header = if is_wheel { "wheels" } else { "sdist" };
    let filename = filename_from_url(url);

    if is_wheel {
        // Wheels live in an array of inline tables.
        out.push_str("wheels = [\n");
        out.push_str("    { name = \"");
        out.push_str(&escape_string(&filename));
        out.push_str("\", url = \"");
        out.push_str(&escape_string(url));
        out.push_str("\", hashes = { sha256 = \"");
        out.push_str(&escape_string(&pkg.sha256));
        out.push_str("\" } },\n");
        out.push_str("]\n");
    } else {
        // Sdist is a single inline table under its own key.
        out.push_str(header);
        out.push_str(" = { name = \"");
        out.push_str(&escape_string(&filename));
        out.push_str("\", url = \"");
        out.push_str(&escape_string(url));
        out.push_str("\", hashes = { sha256 = \"");
        out.push_str(&escape_string(&pkg.sha256));
        out.push_str("\" } }\n");
    }
}

fn write_vcs_artifact(out: &mut String, pkg: &Package) {
    let r = pkg
        .source_ref
        .as_ref()
        .expect("vcs called on no source_ref");
    out.push_str("vcs = { type = \"git\"");
    if let Some(url) = &r.url {
        out.push_str(", url = \"");
        out.push_str(&escape_string(url));
        out.push('"');
    }
    if let Some(rev) = &r.rev {
        out.push_str(", commit-id = \"");
        out.push_str(&escape_string(rev));
        out.push('"');
    }
    out.push_str(" }\n");
}

fn write_directory_artifact(out: &mut String, pkg: &Package) {
    let r = pkg
        .source_ref
        .as_ref()
        .expect("dir called on no source_ref");
    if let Some(path) = &r.path {
        out.push_str("directory = { path = \"");
        out.push_str(&escape_string(path));
        out.push_str("\" }\n");
    }
}

fn filename_from_url(url: &str) -> String {
    url.rsplit('/').next().unwrap_or(url).to_string()
}

/// Minimal TOML-string escape: backslash + double-quote + control chars.
fn escape_string(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '\\' => out.push_str("\\\\"),
            '"' => out.push_str("\\\""),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pkgmanage::pkgmgr::lockfile::{Package, SourceRef, SourceRefKind};

    fn lockfile_with(packages: Vec<Package>) -> Lockfile {
        Lockfile {
            format_version: 1,
            input_hash: "test".into(),
            packages,
        }
    }

    fn registry_pkg(name: &str, version: &str, url: &str, sha: &str) -> Package {
        Package {
            name: name.into(),
            version: version.into(),
            sha256: sha.into(),
            source: url.into(),
            dependencies: vec![],
            markers: None,
            source_ref: Some(SourceRef {
                kind: SourceRefKind::Registry,
                path: None,
                url: None,
                rev: None,
            }),
        }
    }

    #[test]
    fn emits_header_keys_in_order() {
        let lf = lockfile_with(vec![]);
        let opts = PylockOptions {
            lock_version: "1.0".into(),
            created_by: "mamba".into(),
            requires_python: Some(">=3.10".into()),
            environments: vec!["sys_platform == 'linux'".into()],
        };
        let body = render_pylock_toml(&lf, &opts);
        // The four header keys land in fixed order.
        let lv = body.find("lock-version").unwrap();
        let cb = body.find("created-by").unwrap();
        let rp = body.find("requires-python").unwrap();
        let env = body.find("environments").unwrap();
        assert!(lv < cb && cb < rp && rp < env);
    }

    #[test]
    fn default_options_omit_optional_keys() {
        let lf = lockfile_with(vec![]);
        let body = render_pylock_toml(&lf, &PylockOptions::default());
        assert!(body.contains("lock-version = \"1.0\""));
        assert!(body.contains("created-by = \"mamba\""));
        assert!(!body.contains("requires-python"));
        assert!(!body.contains("environments"));
    }

    #[test]
    fn renders_single_registry_sdist_package() {
        let lf = lockfile_with(vec![registry_pkg(
            "click",
            "8.1.7",
            "https://files.pythonhosted.org/packages/abc/click-8.1.7.tar.gz",
            "abcdef",
        )]);
        let body = render_pylock_toml(&lf, &PylockOptions::default());
        assert!(body.contains("[[packages]]"));
        assert!(body.contains("name = \"click\""));
        assert!(body.contains("version = \"8.1.7\""));
        assert!(body.contains("sdist = { name = \"click-8.1.7.tar.gz\""));
        assert!(body.contains("sha256 = \"abcdef\""));
    }

    #[test]
    fn renders_wheel_artifact_inside_wheels_array() {
        let lf = lockfile_with(vec![registry_pkg(
            "click",
            "8.1.7",
            "https://files.pythonhosted.org/packages/abc/click-8.1.7-py3-none-any.whl",
            "feedface",
        )]);
        let body = render_pylock_toml(&lf, &PylockOptions::default());
        assert!(body.contains("wheels = [\n"));
        assert!(body.contains("click-8.1.7-py3-none-any.whl"));
        assert!(body.contains("sha256 = \"feedface\""));
        // No top-level sdist for wheel-only artifact.
        assert!(!body.contains("sdist ="));
    }

    #[test]
    fn packages_are_sorted_by_normalized_name() {
        let lf = lockfile_with(vec![
            registry_pkg(
                "Requests",
                "2.31.0",
                "https://e/Requests-2.31.0.tar.gz",
                "r",
            ),
            registry_pkg("click", "8.1.7", "https://e/click-8.1.7.tar.gz", "c"),
            registry_pkg("my_pkg", "1.0", "https://e/my_pkg-1.0.tar.gz", "m"),
        ]);
        let body = render_pylock_toml(&lf, &PylockOptions::default());
        let click = body.find("name = \"click\"").unwrap();
        let my = body.find("name = \"my_pkg\"").unwrap();
        let req = body.find("name = \"Requests\"").unwrap();
        // PEP 503 order: click < my-pkg < requests.
        assert!(click < my && my < req);
    }

    #[test]
    fn renders_dependencies_sorted_and_inline() {
        let mut pkg = registry_pkg("django", "5.0", "https://e/django-5.0.tar.gz", "deadbeef");
        pkg.dependencies = vec!["sqlparse".into(), "asgiref".into()];
        let lf = lockfile_with(vec![pkg]);
        let body = render_pylock_toml(&lf, &PylockOptions::default());
        let a = body.find("name = \"asgiref\"").unwrap();
        let s = body.find("name = \"sqlparse\"").unwrap();
        assert!(a < s);
        assert!(body.contains("dependencies = ["));
    }

    #[test]
    fn renders_marker_when_present() {
        let mut pkg = registry_pkg("tomli", "2.0.1", "https://e/tomli-2.0.1.tar.gz", "abc");
        pkg.markers = Some("python_version < \"3.11\"".into());
        let lf = lockfile_with(vec![pkg]);
        let body = render_pylock_toml(&lf, &PylockOptions::default());
        assert!(body.contains("marker = \"python_version < \\\"3.11\\\"\""));
    }

    #[test]
    fn renders_git_vcs_artifact() {
        let pkg = Package {
            name: "mypkg".into(),
            version: "0.0.0".into(),
            sha256: "".into(),
            source: "git+https://example.com/m.git@v1".into(),
            dependencies: vec![],
            markers: None,
            source_ref: Some(SourceRef {
                kind: SourceRefKind::Git,
                path: None,
                url: Some("https://example.com/m.git".into()),
                rev: Some("v1".into()),
            }),
        };
        let body = render_pylock_toml(&lockfile_with(vec![pkg]), &PylockOptions::default());
        assert!(body.contains("vcs = { type = \"git\""));
        assert!(body.contains("url = \"https://example.com/m.git\""));
        assert!(body.contains("commit-id = \"v1\""));
        assert!(!body.contains("sdist ="));
        assert!(!body.contains("wheels ="));
    }

    #[test]
    fn renders_directory_artifact() {
        let pkg = Package {
            name: "local".into(),
            version: "0.0.0".into(),
            sha256: "".into(),
            source: "/abs/path".into(),
            dependencies: vec![],
            markers: None,
            source_ref: Some(SourceRef {
                kind: SourceRefKind::Path,
                path: Some("/abs/path".into()),
                url: None,
                rev: None,
            }),
        };
        let body = render_pylock_toml(&lockfile_with(vec![pkg]), &PylockOptions::default());
        assert!(body.contains("directory = { path = \"/abs/path\" }"));
    }

    #[test]
    fn empty_packages_yield_header_only() {
        let body = render_pylock_toml(&lockfile_with(vec![]), &PylockOptions::default());
        assert!(!body.contains("[[packages]]"));
    }

    #[test]
    fn deterministic_round_to_same_string() {
        let mut pkg = registry_pkg("click", "8.1.7", "https://e/click-8.1.7.tar.gz", "ab");
        pkg.dependencies = vec!["colorama".into()];
        pkg.markers = Some("python_version >= '3.8'".into());
        let lf = lockfile_with(vec![pkg]);
        let opts = PylockOptions {
            lock_version: "1.0".into(),
            created_by: "mamba".into(),
            requires_python: Some(">=3.10".into()),
            environments: vec!["sys_platform == 'linux'".into()],
        };
        let a = render_pylock_toml(&lf, &opts);
        let b = render_pylock_toml(&lf, &opts);
        assert_eq!(a, b);
    }

    #[test]
    fn string_escape_handles_quotes_and_backslashes() {
        let mut pkg = registry_pkg("weird", "0.0", "https://e/x.tar.gz", "ab");
        pkg.markers = Some("os_name == \"posix\" and name == 'a\\b'".into());
        let lf = lockfile_with(vec![pkg]);
        let body = render_pylock_toml(&lf, &PylockOptions::default());
        // Quotes inside the marker are escaped with backslash.
        assert!(body.contains("\\\"posix\\\""));
        // Backslash itself doubled.
        assert!(body.contains("a\\\\b"));
    }
}
