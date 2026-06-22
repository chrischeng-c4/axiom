// Direct-URL requirements (Tick 32).
//
// PEP 440 §5 lets a requirement skip the index entirely:
//
//   foo @ https://example.test/foo-1.0.tar.gz
//   foo @ git+https://github.com/example/foo.git@v1.2.3
//   foo @ git+ssh://git@example.test:org/foo.git@main#subdirectory=pkgs/foo
//   foo @ file:///abs/path/to/foo
//
// uv recognizes the same shapes (plus a few extras like
// `git+https://…?rev=…`). This module parses the URL half into a
// structured `DirectUrl` so the resolver can decide how to fetch it:
//
//   * Archive       → download + unpack like a normal sdist
//   * Git           → clone, checkout, build
//   * LocalPath     → install from a local directory (PEP 660 editable
//                     candidate)
//
// Pure data layer. No network, no filesystem. We just classify the URL.

use crate::pkgmanage::pkgmgr::types::IndexError;

/// Parsed direct-URL reference.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DirectUrl {
    /// Remote archive (`https://…/foo-1.0.tar.gz`). The resolver
    /// downloads + unpacks like a normal sdist/wheel.
    Archive {
        url: String,
        /// Optional `#subdirectory=path` fragment.
        subdirectory: Option<String>,
    },
    /// Git ref (`git+https://…@ref` or `git+ssh://…@ref`).
    Git {
        /// Transport URL stripped of `git+` prefix and the `@ref`
        /// suffix. e.g. `https://github.com/example/foo.git`.
        url: String,
        /// Ref to check out: branch, tag, or commit SHA. Defaults to
        /// `None` (use whatever the remote HEAD points to).
        rev: Option<String>,
        subdirectory: Option<String>,
    },
    /// Local path (`file://…` or, after future scope, a bare path).
    LocalPath {
        path: String,
        subdirectory: Option<String>,
    },
}

/// One line from a requirements file, parsed into name + direct URL.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DirectRequirement {
    pub name: String,
    pub url: DirectUrl,
}

/// Parse a `name @ url` requirement line.
///
/// Whitespace around `@` is optional but recommended; we accept both
/// `foo@url` and `foo @ url`.
pub fn parse_direct_requirement(line: &str) -> Result<DirectRequirement, IndexError> {
    let line = line.trim();
    let (name, url_part) = split_name_and_url(line)?;
    let url = parse_direct_url(url_part)?;
    Ok(DirectRequirement { name, url })
}

fn split_name_and_url(line: &str) -> Result<(String, &str), IndexError> {
    // Find the first '@' — that delimits name from URL. PEP 440 names
    // are restricted to [A-Za-z0-9._-]; we validate that *after*
    // locating the separator so missing-'@' errors don't get masked
    // by spurious "invalid char" reports from URL bytes.
    let Some(at) = line.find('@') else {
        return Err(IndexError::ParseError {
            url: "<direct requirement>".into(),
            detail: format!("missing '@' separator in {line:?}"),
        });
    };
    for c in line[..at].chars() {
        if !(c.is_ascii_alphanumeric() || c == '.' || c == '-' || c == '_' || c == ' ' || c == '\t')
        {
            return Err(IndexError::ParseError {
                url: "<direct requirement>".into(),
                detail: format!(
                    "unexpected character {c:?} before '@' in {line:?}; \
                     direct-URL requirements look like `name @ url`"
                ),
            });
        }
    }
    let name = line[..at].trim().to_string();
    if name.is_empty() {
        return Err(IndexError::ParseError {
            url: "<direct requirement>".into(),
            detail: format!("missing requirement name in {line:?}"),
        });
    }
    let url_part = line[at + 1..].trim();
    if url_part.is_empty() {
        return Err(IndexError::ParseError {
            url: "<direct requirement>".into(),
            detail: format!("missing URL after '@' in {line:?}"),
        });
    }
    Ok((name, url_part))
}

/// Classify a URL into one of the three `DirectUrl` variants.
pub fn parse_direct_url(raw: &str) -> Result<DirectUrl, IndexError> {
    if let Some(rest) = raw.strip_prefix("git+") {
        return parse_git_url(rest);
    }
    if let Some(rest) = raw.strip_prefix("file://") {
        // file:// URLs have no @rev. Subdirectory still possible.
        let (path, subdirectory) = split_subdirectory_fragment(rest);
        return Ok(DirectUrl::LocalPath {
            path: path.to_string(),
            subdirectory,
        });
    }
    if raw.starts_with("http://") || raw.starts_with("https://") {
        let (url, subdirectory) = split_subdirectory_fragment(raw);
        return Ok(DirectUrl::Archive {
            url: url.to_string(),
            subdirectory,
        });
    }
    if raw.starts_with('/') || raw.starts_with("./") || raw.starts_with("../") {
        // Bare local paths. uv accepts these in `requirements.txt`
        // style; we accept them after `name @` too.
        let (path, subdirectory) = split_subdirectory_fragment(raw);
        return Ok(DirectUrl::LocalPath {
            path: path.to_string(),
            subdirectory,
        });
    }
    Err(IndexError::ParseError {
        url: "<direct requirement>".into(),
        detail: format!(
            "unrecognized direct-URL scheme in {raw:?}; expected one of \
             `git+…`, `file://…`, `http(s)://…`, or a local path"
        ),
    })
}

fn parse_git_url(rest: &str) -> Result<DirectUrl, IndexError> {
    // Strip subdirectory fragment first (it's always at the end).
    let (without_frag, subdirectory) = split_subdirectory_fragment(rest);
    // Split on '@' once, RIGHTMOST, but only if the rightmost '@' is
    // outside any userinfo block. uv's heuristic: the '@rev' is what
    // follows the *last* '@' in the URL, *unless* that '@' precedes
    // the host (i.e. it's an SSH userinfo separator and there's no
    // explicit @rev). Practically: if the substring after the last '@'
    // contains a '/' or ':', it's part of the URL, not a rev.
    if let Some(last_at) = without_frag.rfind('@') {
        let candidate_rev = &without_frag[last_at + 1..];
        let looks_like_rev = !candidate_rev.contains('/') && !candidate_rev.contains(':');
        if looks_like_rev && !candidate_rev.is_empty() {
            let url = without_frag[..last_at].to_string();
            return Ok(DirectUrl::Git {
                url,
                rev: Some(candidate_rev.to_string()),
                subdirectory,
            });
        }
    }
    Ok(DirectUrl::Git {
        url: without_frag.to_string(),
        rev: None,
        subdirectory,
    })
}

fn split_subdirectory_fragment(raw: &str) -> (&str, Option<String>) {
    if let Some(hash) = raw.find('#') {
        let (head, frag) = raw.split_at(hash);
        let frag = &frag[1..]; // drop '#'
                               // Fragment is `key=value[&key=value]…`. We only care about
                               // `subdirectory=…`. uv ignores unknown keys.
        for pair in frag.split('&') {
            if let Some(value) = pair.strip_prefix("subdirectory=") {
                if !value.is_empty() {
                    return (head, Some(value.to_string()));
                }
            }
        }
        return (head, None);
    }
    (raw, None)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_archive_https() {
        let r = parse_direct_requirement("foo @ https://example.test/foo-1.0.tar.gz").unwrap();
        assert_eq!(r.name, "foo");
        assert_eq!(
            r.url,
            DirectUrl::Archive {
                url: "https://example.test/foo-1.0.tar.gz".into(),
                subdirectory: None,
            }
        );
    }

    #[test]
    fn parse_archive_with_subdirectory() {
        let r =
            parse_direct_requirement("foo @ https://example.test/foo.tar.gz#subdirectory=src/foo")
                .unwrap();
        match r.url {
            DirectUrl::Archive { url, subdirectory } => {
                assert_eq!(url, "https://example.test/foo.tar.gz");
                assert_eq!(subdirectory.as_deref(), Some("src/foo"));
            }
            other => panic!("expected Archive, got {other:?}"),
        }
    }

    #[test]
    fn parse_git_https_with_tag() {
        let r = parse_direct_requirement("foo @ git+https://github.com/example/foo.git@v1.2.3")
            .unwrap();
        assert_eq!(
            r.url,
            DirectUrl::Git {
                url: "https://github.com/example/foo.git".into(),
                rev: Some("v1.2.3".into()),
                subdirectory: None,
            }
        );
    }

    #[test]
    fn parse_git_https_without_rev() {
        let r = parse_direct_requirement("foo @ git+https://github.com/example/foo.git").unwrap();
        assert_eq!(
            r.url,
            DirectUrl::Git {
                url: "https://github.com/example/foo.git".into(),
                rev: None,
                subdirectory: None,
            }
        );
    }

    #[test]
    fn parse_git_ssh_with_rev_and_subdirectory() {
        let r = parse_direct_requirement(
            "foo @ git+ssh://git@example.test/org/foo.git@main#subdirectory=pkgs/foo",
        )
        .unwrap();
        match r.url {
            DirectUrl::Git {
                url,
                rev,
                subdirectory,
            } => {
                assert_eq!(url, "ssh://git@example.test/org/foo.git");
                assert_eq!(rev.as_deref(), Some("main"));
                assert_eq!(subdirectory.as_deref(), Some("pkgs/foo"));
            }
            other => panic!("expected Git, got {other:?}"),
        }
    }

    #[test]
    fn parse_git_ssh_userinfo_is_not_rev() {
        // `git@example.test/org/foo.git` — the `git@` here is SSH
        // userinfo. With no `@rev` after, rev should be None.
        let r = parse_direct_requirement("foo @ git+ssh://git@example.test/org/foo.git").unwrap();
        match r.url {
            DirectUrl::Git { url, rev, .. } => {
                assert_eq!(url, "ssh://git@example.test/org/foo.git");
                assert_eq!(rev, None);
            }
            other => panic!("expected Git, got {other:?}"),
        }
    }

    #[test]
    fn parse_file_url() {
        let r = parse_direct_requirement("foo @ file:///abs/path/to/foo").unwrap();
        assert_eq!(
            r.url,
            DirectUrl::LocalPath {
                path: "/abs/path/to/foo".into(),
                subdirectory: None,
            }
        );
    }

    #[test]
    fn parse_bare_relative_path() {
        let r = parse_direct_requirement("foo @ ./vendored/foo").unwrap();
        assert_eq!(
            r.url,
            DirectUrl::LocalPath {
                path: "./vendored/foo".into(),
                subdirectory: None,
            }
        );
    }

    #[test]
    fn parse_handles_missing_separator() {
        let err = parse_direct_requirement("foo https://example.test/").unwrap_err();
        assert!(format!("{err}").contains("missing '@' separator"));
    }

    #[test]
    fn parse_handles_missing_name() {
        let err = parse_direct_requirement(" @ https://example.test/").unwrap_err();
        assert!(format!("{err}").contains("missing requirement name"));
    }

    #[test]
    fn parse_handles_missing_url() {
        let err = parse_direct_requirement("foo @").unwrap_err();
        assert!(format!("{err}").contains("missing URL"));
    }

    #[test]
    fn parse_rejects_unknown_scheme() {
        let err = parse_direct_requirement("foo @ smb://share/path").unwrap_err();
        assert!(format!("{err}").contains("unrecognized direct-URL scheme"));
    }

    #[test]
    fn parse_rejects_invalid_name_chars() {
        // '$' in the name is not a PEP 440 name character.
        let err = parse_direct_requirement("foo$bar @ https://example.test").unwrap_err();
        assert!(format!("{err}").contains("unexpected character"));
    }

    #[test]
    fn fragment_unknown_keys_ignored() {
        let r = parse_direct_requirement(
            "foo @ https://example.test/foo.tar.gz#egg=foo&subdirectory=src",
        )
        .unwrap();
        match r.url {
            DirectUrl::Archive { url, subdirectory } => {
                assert_eq!(url, "https://example.test/foo.tar.gz");
                assert_eq!(subdirectory.as_deref(), Some("src"));
            }
            other => panic!("expected Archive, got {other:?}"),
        }
    }

    #[test]
    fn fragment_without_subdirectory_yields_none() {
        let r = parse_direct_requirement("foo @ https://example.test/foo.tar.gz#egg=foo").unwrap();
        match r.url {
            DirectUrl::Archive { subdirectory, .. } => {
                assert_eq!(subdirectory, None);
            }
            other => panic!("expected Archive, got {other:?}"),
        }
    }
}
