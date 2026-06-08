// `[tool.uv.sources]` parser (Tick 56).
//
// uv lets a project redirect individual dependencies away from the
// configured indexes to alternate origins via `[tool.uv.sources]` in
// pyproject.toml. Each key is the dependency name; the value is an
// inline table whose fields decide the source kind:
//
//   ruff = { git = "https://github.com/astral-sh/ruff", tag = "v0.4.0" }
//   ruff = { git = "...", rev = "abc123" }
//   ruff = { git = "...", branch = "main" }
//   local-pkg = { path = "../local-pkg" }
//   local-pkg = { path = "../local-pkg", editable = true }
//   thing = { url = "https://example.com/thing-1.0.tar.gz" }
//   member = { workspace = true }
//   member = { index = "private-index" }
//
// This module turns that table into a typed map. It is pure-data: no
// I/O, no resolver lookup. Marker / extra filtering is deferred — uv
// permits `marker` and `extra` keys here but the basic shape is what
// downstream resolver / installer code needs first.

use std::collections::BTreeMap;

/// One entry's typed source override.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UvSource {
    /// `{ git = "...", rev/tag/branch = "...", subdirectory? = "..." }`.
    Git {
        url: String,
        reference: Option<GitReference>,
        subdirectory: Option<String>,
    },
    /// `{ path = "...", editable? = true, subdirectory? = "..." }`.
    Path {
        path: String,
        editable: bool,
        subdirectory: Option<String>,
    },
    /// `{ url = "..." }`. Direct URL to an sdist or wheel.
    Url { url: String },
    /// `{ workspace = true }`. Resolves from the same workspace.
    Workspace,
    /// `{ index = "name" }`. Pin the dep to a specific index.
    Index { name: String },
}

/// Which kind of git anchor the user supplied. uv accepts at most one
/// of `rev`, `tag`, or `branch` per source.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GitReference {
    Rev(String),
    Tag(String),
    Branch(String),
}

/// Reasons a `[tool.uv.sources]` table may fail to parse.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UvSourcesError {
    /// Body is not valid TOML.
    InvalidToml(String),
    /// A field has the wrong type (e.g. `git` set to an integer).
    WrongType {
        package: String,
        field: &'static str,
    },
    /// Entry is missing a required field.
    MissingField {
        package: String,
        detail: String,
    },
    /// More than one source kind is set on the same entry, or none.
    ConflictingKind {
        package: String,
        detail: String,
    },
}

impl std::fmt::Display for UvSourcesError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UvSourcesError::InvalidToml(s) => write!(f, "invalid toml: {s}"),
            UvSourcesError::WrongType { package, field } => {
                write!(f, "[tool.uv.sources] {package}.{field}: wrong type")
            }
            UvSourcesError::MissingField { package, detail } => {
                write!(f, "[tool.uv.sources] {package}: missing field — {detail}")
            }
            UvSourcesError::ConflictingKind { package, detail } => {
                write!(f, "[tool.uv.sources] {package}: {detail}")
            }
        }
    }
}

impl std::error::Error for UvSourcesError {}

/// Parse `[tool.uv.sources]` out of a `pyproject.toml` body string.
/// Returns an empty map when the table is absent.
pub fn parse_uv_sources(toml_src: &str) -> Result<BTreeMap<String, UvSource>, UvSourcesError> {
    let doc: toml::Value = toml_src
        .parse()
        .map_err(|e: toml::de::Error| UvSourcesError::InvalidToml(e.to_string()))?;

    let Some(table) = doc
        .get("tool")
        .and_then(|t| t.get("uv"))
        .and_then(|t| t.get("sources"))
    else {
        return Ok(BTreeMap::new());
    };

    let table = table
        .as_table()
        .ok_or_else(|| UvSourcesError::WrongType {
            package: String::new(),
            field: "sources",
        })?;

    let mut out = BTreeMap::new();
    for (name, value) in table {
        let entry = value.as_table().ok_or_else(|| UvSourcesError::WrongType {
            package: name.clone(),
            field: "entry",
        })?;
        out.insert(name.clone(), decode_entry(name, entry)?);
    }
    Ok(out)
}

fn decode_entry(
    name: &str,
    entry: &toml::value::Table,
) -> Result<UvSource, UvSourcesError> {
    let has_git = entry.contains_key("git");
    let has_path = entry.contains_key("path");
    let has_url = entry.contains_key("url");
    let has_workspace = entry.contains_key("workspace");
    let has_index = entry.contains_key("index");

    let kind_count = [has_git, has_path, has_url, has_workspace, has_index]
        .iter()
        .filter(|x| **x)
        .count();
    if kind_count == 0 {
        return Err(UvSourcesError::ConflictingKind {
            package: name.into(),
            detail: "must declare one of git / path / url / workspace / index".into(),
        });
    }
    if kind_count > 1 {
        return Err(UvSourcesError::ConflictingKind {
            package: name.into(),
            detail: "git / path / url / workspace / index are mutually exclusive".into(),
        });
    }

    if has_git {
        decode_git(name, entry)
    } else if has_path {
        decode_path(name, entry)
    } else if has_url {
        decode_url(name, entry)
    } else if has_workspace {
        decode_workspace(name, entry)
    } else {
        decode_index(name, entry)
    }
}

fn decode_git(name: &str, entry: &toml::value::Table) -> Result<UvSource, UvSourcesError> {
    let url = required_string(name, entry, "git")?;
    let has_rev = entry.contains_key("rev");
    let has_tag = entry.contains_key("tag");
    let has_branch = entry.contains_key("branch");
    if [has_rev, has_tag, has_branch]
        .iter()
        .filter(|b| **b)
        .count()
        > 1
    {
        return Err(UvSourcesError::ConflictingKind {
            package: name.into(),
            detail: "rev / tag / branch are mutually exclusive".into(),
        });
    }
    let reference = if has_rev {
        Some(GitReference::Rev(required_string(name, entry, "rev")?))
    } else if has_tag {
        Some(GitReference::Tag(required_string(name, entry, "tag")?))
    } else if has_branch {
        Some(GitReference::Branch(required_string(
            name, entry, "branch",
        )?))
    } else {
        None
    };
    let subdirectory = optional_string(name, entry, "subdirectory")?;
    Ok(UvSource::Git {
        url,
        reference,
        subdirectory,
    })
}

fn decode_path(name: &str, entry: &toml::value::Table) -> Result<UvSource, UvSourcesError> {
    let path = required_string(name, entry, "path")?;
    let editable = optional_bool(name, entry, "editable")?.unwrap_or(false);
    let subdirectory = optional_string(name, entry, "subdirectory")?;
    Ok(UvSource::Path {
        path,
        editable,
        subdirectory,
    })
}

fn decode_url(name: &str, entry: &toml::value::Table) -> Result<UvSource, UvSourcesError> {
    let url = required_string(name, entry, "url")?;
    Ok(UvSource::Url { url })
}

fn decode_workspace(name: &str, entry: &toml::value::Table) -> Result<UvSource, UvSourcesError> {
    let v = entry.get("workspace").expect("checked");
    let b = v.as_bool().ok_or_else(|| UvSourcesError::WrongType {
        package: name.into(),
        field: "workspace",
    })?;
    if !b {
        return Err(UvSourcesError::ConflictingKind {
            package: name.into(),
            detail: "workspace = false is not a valid source declaration".into(),
        });
    }
    Ok(UvSource::Workspace)
}

fn decode_index(name: &str, entry: &toml::value::Table) -> Result<UvSource, UvSourcesError> {
    let idx_name = required_string(name, entry, "index")?;
    Ok(UvSource::Index { name: idx_name })
}

fn required_string(
    package: &str,
    entry: &toml::value::Table,
    field: &'static str,
) -> Result<String, UvSourcesError> {
    let v = entry.get(field).ok_or_else(|| UvSourcesError::MissingField {
        package: package.into(),
        detail: field.into(),
    })?;
    let s = v.as_str().ok_or_else(|| UvSourcesError::WrongType {
        package: package.into(),
        field,
    })?;
    Ok(s.to_string())
}

fn optional_string(
    package: &str,
    entry: &toml::value::Table,
    field: &'static str,
) -> Result<Option<String>, UvSourcesError> {
    let Some(v) = entry.get(field) else {
        return Ok(None);
    };
    let s = v.as_str().ok_or_else(|| UvSourcesError::WrongType {
        package: package.into(),
        field,
    })?;
    Ok(Some(s.to_string()))
}

fn optional_bool(
    package: &str,
    entry: &toml::value::Table,
    field: &'static str,
) -> Result<Option<bool>, UvSourcesError> {
    let Some(v) = entry.get(field) else {
        return Ok(None);
    };
    let b = v.as_bool().ok_or_else(|| UvSourcesError::WrongType {
        package: package.into(),
        field,
    })?;
    Ok(Some(b))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_document_returns_empty_map() {
        let src = r#"[project]
name = "x"
version = "0.0.0"
"#;
        let got = parse_uv_sources(src).unwrap();
        assert!(got.is_empty());
    }

    #[test]
    fn invalid_toml_is_an_error() {
        let err = parse_uv_sources("not = = valid").unwrap_err();
        assert!(matches!(err, UvSourcesError::InvalidToml(_)));
    }

    #[test]
    fn parses_git_source_with_tag() {
        let src = r#"[tool.uv.sources]
ruff = { git = "https://github.com/astral-sh/ruff", tag = "v0.4.0" }
"#;
        let got = parse_uv_sources(src).unwrap();
        assert_eq!(
            got.get("ruff"),
            Some(&UvSource::Git {
                url: "https://github.com/astral-sh/ruff".into(),
                reference: Some(GitReference::Tag("v0.4.0".into())),
                subdirectory: None,
            })
        );
    }

    #[test]
    fn parses_git_source_with_rev_and_subdirectory() {
        let src = r#"[tool.uv.sources]
pkg = { git = "https://example.com/repo", rev = "abc123", subdirectory = "subdir" }
"#;
        let got = parse_uv_sources(src).unwrap();
        let entry = got.get("pkg").unwrap();
        match entry {
            UvSource::Git {
                url,
                reference,
                subdirectory,
            } => {
                assert_eq!(url, "https://example.com/repo");
                assert_eq!(reference.as_ref(), Some(&GitReference::Rev("abc123".into())));
                assert_eq!(subdirectory.as_deref(), Some("subdir"));
            }
            other => panic!("unexpected: {other:?}"),
        }
    }

    #[test]
    fn parses_git_source_with_branch() {
        let src = r#"[tool.uv.sources]
pkg = { git = "https://example.com/repo", branch = "main" }
"#;
        let got = parse_uv_sources(src).unwrap();
        if let UvSource::Git { reference, .. } = got.get("pkg").unwrap() {
            assert_eq!(
                reference.as_ref(),
                Some(&GitReference::Branch("main".into()))
            );
        } else {
            panic!("expected Git");
        }
    }

    #[test]
    fn git_without_anchor_keeps_reference_none() {
        let src = r#"[tool.uv.sources]
pkg = { git = "https://example.com/repo" }
"#;
        let got = parse_uv_sources(src).unwrap();
        if let UvSource::Git { reference, .. } = got.get("pkg").unwrap() {
            assert!(reference.is_none());
        } else {
            panic!("expected Git");
        }
    }

    #[test]
    fn rejects_git_with_two_anchors() {
        let src = r#"[tool.uv.sources]
pkg = { git = "https://example.com/repo", tag = "v1", branch = "main" }
"#;
        let err = parse_uv_sources(src).unwrap_err();
        assert!(matches!(err, UvSourcesError::ConflictingKind { .. }));
    }

    #[test]
    fn parses_path_source_default_non_editable() {
        let src = r#"[tool.uv.sources]
local = { path = "../local-pkg" }
"#;
        let got = parse_uv_sources(src).unwrap();
        assert_eq!(
            got.get("local"),
            Some(&UvSource::Path {
                path: "../local-pkg".into(),
                editable: false,
                subdirectory: None,
            })
        );
    }

    #[test]
    fn parses_editable_path_source() {
        let src = r#"[tool.uv.sources]
local = { path = "../local-pkg", editable = true }
"#;
        let got = parse_uv_sources(src).unwrap();
        if let UvSource::Path { editable, .. } = got.get("local").unwrap() {
            assert!(*editable);
        } else {
            panic!("expected Path");
        }
    }

    #[test]
    fn parses_url_source() {
        let src = r#"[tool.uv.sources]
thing = { url = "https://example.com/thing-1.0.tar.gz" }
"#;
        let got = parse_uv_sources(src).unwrap();
        assert_eq!(
            got.get("thing"),
            Some(&UvSource::Url {
                url: "https://example.com/thing-1.0.tar.gz".into(),
            })
        );
    }

    #[test]
    fn parses_workspace_true() {
        let src = r#"[tool.uv.sources]
member = { workspace = true }
"#;
        let got = parse_uv_sources(src).unwrap();
        assert_eq!(got.get("member"), Some(&UvSource::Workspace));
    }

    #[test]
    fn rejects_workspace_false() {
        let src = r#"[tool.uv.sources]
member = { workspace = false }
"#;
        let err = parse_uv_sources(src).unwrap_err();
        assert!(matches!(err, UvSourcesError::ConflictingKind { .. }));
    }

    #[test]
    fn parses_index_pinning() {
        let src = r#"[tool.uv.sources]
pkg = { index = "private" }
"#;
        let got = parse_uv_sources(src).unwrap();
        assert_eq!(
            got.get("pkg"),
            Some(&UvSource::Index {
                name: "private".into(),
            })
        );
    }

    #[test]
    fn rejects_entry_with_no_kind() {
        let src = r#"[tool.uv.sources]
pkg = { tag = "v1" }
"#;
        let err = parse_uv_sources(src).unwrap_err();
        assert!(matches!(err, UvSourcesError::ConflictingKind { .. }));
    }

    #[test]
    fn rejects_entry_with_two_kinds() {
        let src = r#"[tool.uv.sources]
pkg = { git = "https://example.com/r", path = "../local" }
"#;
        let err = parse_uv_sources(src).unwrap_err();
        assert!(matches!(err, UvSourcesError::ConflictingKind { .. }));
    }

    #[test]
    fn rejects_wrong_field_type() {
        let src = r#"[tool.uv.sources]
pkg = { git = 42 }
"#;
        let err = parse_uv_sources(src).unwrap_err();
        assert!(matches!(err, UvSourcesError::WrongType { field: "git", .. }));
    }

    #[test]
    fn parses_multiple_entries_alphabetized_by_btreemap() {
        let src = r#"[tool.uv.sources]
zzz = { url = "https://e.com/z" }
aaa = { url = "https://e.com/a" }
mmm = { url = "https://e.com/m" }
"#;
        let got = parse_uv_sources(src).unwrap();
        let keys: Vec<&String> = got.keys().collect();
        assert_eq!(keys, vec!["aaa", "mmm", "zzz"]);
    }

    #[test]
    fn ignores_other_tool_sections() {
        let src = r#"[tool.black]
line-length = 100

[tool.uv.sources]
ruff = { git = "https://github.com/astral-sh/ruff", tag = "v0.4.0" }

[tool.uv]
dev-dependencies = ["pytest"]
"#;
        let got = parse_uv_sources(src).unwrap();
        assert_eq!(got.len(), 1);
        assert!(got.contains_key("ruff"));
    }

    #[test]
    fn missing_required_field_reports_package_name() {
        let src = r#"[tool.uv.sources]
pkg = { git = "https://example.com/r", rev = 1 }
"#;
        let err = parse_uv_sources(src).unwrap_err();
        match err {
            UvSourcesError::WrongType { package, field } => {
                assert_eq!(package, "pkg");
                assert_eq!(field, "rev");
            }
            other => panic!("unexpected: {other:?}"),
        }
    }
}
