// simple_url.rs — PEP 503 simple-API URL construction and parsing.
//
// PEP 503 defines a single canonical shape for project URLs on a simple
// index:
//
//     <base>/<normalized-project-name>/
//
// The trailing slash is REQUIRED; legacy clients break on its absence.
// The project name segment must be PEP 503-normalized (lowercased,
// `-_.` runs collapsed to a single `-`, leading/trailing separators
// trimmed).
//
// uv canonicalises every index URL through this normalizer so that
// `https://pypi.org/simple`, `https://pypi.org/simple/`, and
// `https://pypi.org/simple//` all hash to the same cache key. Mamba's
// fetcher does the same now via `normalize_index_url`.

use crate::pkgmanage::pkgmgr::name_normalize::pep503_normalize;
use crate::pkgmanage::pkgmgr::types::IndexError;

/// Build the canonical PEP 503 URL for one project on a given index.
///
/// `base` may or may not end in `/` and is otherwise passed through —
/// callers are responsible for picking the right index root (the
/// `/simple/` suffix is part of the convention but not enforced here:
/// some mirrors expose the index at a different path).
pub fn normalize_index_url(base: &str, project: &str) -> String {
    let trimmed_base = base.trim_end_matches('/');
    let name = pep503_normalize(project);
    format!("{trimmed_base}/{name}/")
}

/// Result of parsing an arbitrary URL as a simple-API project page.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SimpleProjectUrl {
    /// Everything up to (but excluding) the project segment, without
    /// trailing slash.
    pub base: String,
    /// The project name as it appeared in the URL — already
    /// PEP 503-normalized on output regardless of input casing.
    pub project: String,
}

/// Split a project-page URL into `(base, project)`. The URL must use
/// `http` or `https`, have a non-empty project segment, and end in
/// either `/` or no trailing path beyond the project (we tolerate the
/// missing slash and normalize it away).
pub fn parse_simple_api_url(url: &str) -> Result<SimpleProjectUrl, IndexError> {
    let lowered = url.to_ascii_lowercase();
    if !(lowered.starts_with("http://") || lowered.starts_with("https://")) {
        return Err(IndexError::ParseError {
            url: url.to_string(),
            detail: "simple-API URL must use http or https".into(),
        });
    }
    let scheme_end = url.find("://").unwrap() + 3;
    let after_scheme = &url[scheme_end..];
    let path_start = match after_scheme.find('/') {
        Some(i) => i,
        None => {
            return Err(IndexError::ParseError {
                url: url.to_string(),
                detail: "simple-API URL has no path / project segment".into(),
            });
        }
    };
    let full_path = &after_scheme[path_start..];
    // Drop any trailing slashes from the path so we can split off the
    // last non-empty segment as the project.
    let trimmed = full_path.trim_end_matches('/');
    let last_slash = match trimmed.rfind('/') {
        Some(i) => i,
        None => {
            return Err(IndexError::ParseError {
                url: url.to_string(),
                detail: "simple-API URL must include a project segment".into(),
            });
        }
    };
    let project_raw = &trimmed[last_slash + 1..];
    if project_raw.is_empty() {
        return Err(IndexError::ParseError {
            url: url.to_string(),
            detail: "simple-API URL must include a project segment".into(),
        });
    }
    let base_path = &trimmed[..last_slash];
    let base = format!(
        "{scheme_and_host}{base_path}",
        scheme_and_host = &url[..scheme_end + path_start]
    );
    Ok(SimpleProjectUrl {
        base,
        project: pep503_normalize(project_raw),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalize_pypi_canonical() {
        let u = normalize_index_url("https://pypi.org/simple", "Requests");
        assert_eq!(u, "https://pypi.org/simple/requests/");
    }

    #[test]
    fn normalize_trailing_slash_on_base_is_idempotent() {
        let a = normalize_index_url("https://pypi.org/simple", "x");
        let b = normalize_index_url("https://pypi.org/simple/", "x");
        let c = normalize_index_url("https://pypi.org/simple//", "x");
        assert_eq!(a, b);
        assert_eq!(b, c);
    }

    #[test]
    fn normalize_applies_pep503_to_project_segment() {
        let u = normalize_index_url("https://pypi.org/simple", "Pillow_PIL");
        assert_eq!(u, "https://pypi.org/simple/pillow-pil/");
    }

    #[test]
    fn normalize_collapses_separator_runs() {
        let u = normalize_index_url("https://pypi.org/simple", "Zope.Interface");
        assert_eq!(u, "https://pypi.org/simple/zope-interface/");
    }

    #[test]
    fn normalize_handles_mirror_subpath() {
        let u = normalize_index_url(
            "https://gitlab.example.com/api/v4/projects/123/packages/pypi/simple",
            "MyPkg",
        );
        assert_eq!(
            u,
            "https://gitlab.example.com/api/v4/projects/123/packages/pypi/simple/mypkg/"
        );
    }

    #[test]
    fn parse_simple_pypi_page() {
        let r = parse_simple_api_url("https://pypi.org/simple/requests/").unwrap();
        assert_eq!(r.base, "https://pypi.org/simple");
        assert_eq!(r.project, "requests");
    }

    #[test]
    fn parse_tolerates_missing_trailing_slash() {
        let r = parse_simple_api_url("https://pypi.org/simple/requests").unwrap();
        assert_eq!(r.base, "https://pypi.org/simple");
        assert_eq!(r.project, "requests");
    }

    #[test]
    fn parse_normalizes_project_name_in_url() {
        let r = parse_simple_api_url("https://pypi.org/simple/Pillow_PIL/").unwrap();
        assert_eq!(r.project, "pillow-pil");
    }

    #[test]
    fn parse_handles_mirror_subpath() {
        let r = parse_simple_api_url(
            "https://gitlab.example.com/api/v4/projects/123/packages/pypi/simple/mypkg/",
        )
        .unwrap();
        assert_eq!(
            r.base,
            "https://gitlab.example.com/api/v4/projects/123/packages/pypi/simple"
        );
        assert_eq!(r.project, "mypkg");
    }

    #[test]
    fn parse_rejects_non_http_scheme() {
        let err = parse_simple_api_url("ftp://pypi.org/simple/x/").unwrap_err();
        assert!(err.to_string().contains("http or https"));
    }

    #[test]
    fn parse_rejects_no_path() {
        let err = parse_simple_api_url("https://pypi.org").unwrap_err();
        assert!(err.to_string().contains("no path"));
    }

    #[test]
    fn parse_rejects_empty_project() {
        let err = parse_simple_api_url("https://pypi.org/").unwrap_err();
        assert!(err.to_string().contains("project segment"));
    }

    #[test]
    fn parse_treats_last_segment_as_project_unconditionally() {
        // `https://pypi.org/simple/` has only one path segment after the
        // host; we treat it as project="simple" / base="https://pypi.org".
        // Callers that want to gate "must look like a project page" are
        // responsible for that check — this parser stays mechanical.
        let r = parse_simple_api_url("https://pypi.org/simple/").unwrap();
        assert_eq!(r.base, "https://pypi.org");
        assert_eq!(r.project, "simple");
    }

    #[test]
    fn parse_then_normalize_round_trip() {
        let url = "https://pypi.org/simple/Requests";
        let r = parse_simple_api_url(url).unwrap();
        let rebuilt = normalize_index_url(&r.base, &r.project);
        assert_eq!(rebuilt, "https://pypi.org/simple/requests/");
    }

    #[test]
    fn parse_http_scheme_accepted() {
        let r = parse_simple_api_url("http://internal-index.example.com/simple/x/").unwrap();
        assert_eq!(r.project, "x");
    }

    #[test]
    fn case_in_host_preserved_but_project_normalized() {
        let r = parse_simple_api_url("https://INDEX.example.com/simple/MyPkg/").unwrap();
        // Scheme + host casing is left to the caller / DNS layer —
        // we only normalize the PEP 503 project segment.
        assert_eq!(r.base, "https://INDEX.example.com/simple");
        assert_eq!(r.project, "mypkg");
    }
}
