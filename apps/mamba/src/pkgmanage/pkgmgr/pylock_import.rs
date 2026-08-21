// PEP 751 `pylock.toml` reader (Tick 55).
//
// Inverse of `pylock_export`: takes a PEP 751-shaped TOML body and
// returns a `PylockDocument` carrying the header fields plus a
// reconstructed `Lockfile`. Pure-data: no I/O.
//
// Behavior:
//   * Accepts only `lock-version = "1.0"` (the only flavor we emit).
//   * `requires-python` and `environments` are surfaced on the result.
//   * `[[packages]]` is mapped to `Lockfile::packages` (one per entry).
//   * The artifact form is detected in priority order: `vcs`, then
//     `directory`, then `wheels` (first entry), then `sdist`. Anything
//     else is a parse error.
//   * `dependencies` is read from either an array of bare strings or
//     an array of inline tables `{ name = "..." }` (matches the writer
//     output).
//   * `format_version` and `input_hash` are not part of PEP 751; the
//     reader synthesizes them (`format_version = 1`, `input_hash = ""`)
//     so the result round-trips through the internal API.

use crate::pkgmanage::pkgmgr::lockfile::{Lockfile, Package, SourceRef, SourceRefKind};

/// Header + packages decoded from a `pylock.toml` body.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PylockDocument {
    /// PEP 751 `lock-version`.
    pub lock_version: String,
    /// PEP 751 `created-by`, when present.
    pub created_by: Option<String>,
    /// PEP 751 `requires-python` specifier, when present.
    pub requires_python: Option<String>,
    /// PEP 751 `environments` — marker expressions describing the
    /// environments the lock is meant to apply to.
    pub environments: Vec<String>,
    /// Internal lockfile shape reconstructed from the document.
    pub lockfile: Lockfile,
}

/// Reasons a pylock body may fail to parse.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PylockParseError {
    /// Body is not valid TOML.
    InvalidToml(String),
    /// `lock-version` is missing or not the supported version.
    UnsupportedLockVersion(String),
    /// A required field is missing (key carries the path).
    MissingField(String),
    /// A field exists but has the wrong shape.
    WrongType(String),
    /// `[[packages]]` lacks any recognized artifact.
    NoArtifact(String),
}

impl std::fmt::Display for PylockParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PylockParseError::InvalidToml(s) => write!(f, "invalid toml: {s}"),
            PylockParseError::UnsupportedLockVersion(s) => {
                write!(f, "unsupported lock-version: {s}")
            }
            PylockParseError::MissingField(s) => write!(f, "missing field: {s}"),
            PylockParseError::WrongType(s) => write!(f, "wrong type: {s}"),
            PylockParseError::NoArtifact(s) => {
                write!(f, "package {s} has no recognized artifact")
            }
        }
    }
}

impl std::error::Error for PylockParseError {}

/// Parse a `pylock.toml` body string into a `PylockDocument`.
pub fn parse_pylock_toml(src: &str) -> Result<PylockDocument, PylockParseError> {
    let doc: toml::Value = src
        .parse()
        .map_err(|e: toml::de::Error| PylockParseError::InvalidToml(e.to_string()))?;
    let table = doc
        .as_table()
        .ok_or_else(|| PylockParseError::WrongType("root".into()))?;

    let lock_version = required_string(table, "lock-version")?;
    if lock_version != "1.0" {
        return Err(PylockParseError::UnsupportedLockVersion(lock_version));
    }
    let created_by = optional_string(table, "created-by")?;
    let requires_python = optional_string(table, "requires-python")?;
    let environments = optional_string_array(table, "environments")?.unwrap_or_default();

    let packages = if let Some(v) = table.get("packages") {
        let arr = v
            .as_array()
            .ok_or_else(|| PylockParseError::WrongType("packages".into()))?;
        let mut out = Vec::with_capacity(arr.len());
        for (idx, entry) in arr.iter().enumerate() {
            let t = entry
                .as_table()
                .ok_or_else(|| PylockParseError::WrongType(format!("packages[{idx}]")))?;
            out.push(decode_package(t)?);
        }
        out
    } else {
        Vec::new()
    };

    Ok(PylockDocument {
        lock_version,
        created_by,
        requires_python,
        environments,
        lockfile: Lockfile {
            format_version: 1,
            input_hash: String::new(),
            packages,
        },
    })
}

fn decode_package(t: &toml::value::Table) -> Result<Package, PylockParseError> {
    let name = required_string(t, "name")?;
    let version = required_string(t, "version")?;
    let markers = optional_string(t, "marker")?;
    let dependencies = decode_dependencies(t)?;

    let (source, sha256, source_ref) = decode_artifact(t, &name)?;

    Ok(Package {
        name,
        version,
        sha256,
        source,
        dependencies,
        markers,
        source_ref: Some(source_ref),
    })
}

fn decode_dependencies(t: &toml::value::Table) -> Result<Vec<String>, PylockParseError> {
    let Some(v) = t.get("dependencies") else {
        return Ok(Vec::new());
    };
    let arr = v
        .as_array()
        .ok_or_else(|| PylockParseError::WrongType("dependencies".into()))?;
    let mut out = Vec::with_capacity(arr.len());
    for (idx, entry) in arr.iter().enumerate() {
        match entry {
            toml::Value::String(s) => out.push(s.clone()),
            toml::Value::Table(tbl) => {
                let name = tbl.get("name").and_then(|v| v.as_str()).ok_or_else(|| {
                    PylockParseError::MissingField(format!("dependencies[{idx}].name"))
                })?;
                out.push(name.to_string());
            }
            _ => {
                return Err(PylockParseError::WrongType(format!("dependencies[{idx}]")));
            }
        }
    }
    Ok(out)
}

fn decode_artifact(
    t: &toml::value::Table,
    pkg_name: &str,
) -> Result<(String, String, SourceRef), PylockParseError> {
    if let Some(v) = t.get("vcs") {
        return decode_vcs(v, pkg_name);
    }
    if let Some(v) = t.get("directory") {
        return decode_directory(v, pkg_name);
    }
    if let Some(v) = t.get("wheels") {
        return decode_wheels(v, pkg_name);
    }
    if let Some(v) = t.get("sdist") {
        return decode_sdist(v, pkg_name);
    }
    Err(PylockParseError::NoArtifact(pkg_name.into()))
}

fn decode_vcs(
    v: &toml::Value,
    pkg_name: &str,
) -> Result<(String, String, SourceRef), PylockParseError> {
    let t = v
        .as_table()
        .ok_or_else(|| PylockParseError::WrongType(format!("packages.{pkg_name}.vcs")))?;
    let kind = t
        .get("type")
        .and_then(|v| v.as_str())
        .ok_or_else(|| PylockParseError::MissingField(format!("packages.{pkg_name}.vcs.type")))?;
    if kind != "git" {
        return Err(PylockParseError::WrongType(format!(
            "packages.{pkg_name}.vcs.type"
        )));
    }
    let url = t.get("url").and_then(|v| v.as_str()).map(String::from);
    let rev = t
        .get("commit-id")
        .and_then(|v| v.as_str())
        .map(String::from);
    Ok((
        url.clone().unwrap_or_default(),
        String::new(),
        SourceRef {
            kind: SourceRefKind::Git,
            path: None,
            url,
            rev,
        },
    ))
}

fn decode_directory(
    v: &toml::Value,
    pkg_name: &str,
) -> Result<(String, String, SourceRef), PylockParseError> {
    let t = v
        .as_table()
        .ok_or_else(|| PylockParseError::WrongType(format!("packages.{pkg_name}.directory")))?;
    let path = t.get("path").and_then(|v| v.as_str()).ok_or_else(|| {
        PylockParseError::MissingField(format!("packages.{pkg_name}.directory.path"))
    })?;
    Ok((
        path.to_string(),
        String::new(),
        SourceRef {
            kind: SourceRefKind::Path,
            path: Some(path.to_string()),
            url: None,
            rev: None,
        },
    ))
}

fn decode_wheels(
    v: &toml::Value,
    pkg_name: &str,
) -> Result<(String, String, SourceRef), PylockParseError> {
    let arr = v
        .as_array()
        .ok_or_else(|| PylockParseError::WrongType(format!("packages.{pkg_name}.wheels")))?;
    let first = arr
        .first()
        .ok_or_else(|| PylockParseError::MissingField(format!("packages.{pkg_name}.wheels[0]")))?;
    let t = first
        .as_table()
        .ok_or_else(|| PylockParseError::WrongType(format!("packages.{pkg_name}.wheels[0]")))?;
    let url = t.get("url").and_then(|v| v.as_str()).ok_or_else(|| {
        PylockParseError::MissingField(format!("packages.{pkg_name}.wheels[0].url"))
    })?;
    let sha = artifact_sha256(t, &format!("packages.{pkg_name}.wheels[0]"))?;
    Ok((
        url.to_string(),
        sha,
        SourceRef {
            kind: SourceRefKind::Registry,
            path: None,
            url: None,
            rev: None,
        },
    ))
}

fn decode_sdist(
    v: &toml::Value,
    pkg_name: &str,
) -> Result<(String, String, SourceRef), PylockParseError> {
    let t = v
        .as_table()
        .ok_or_else(|| PylockParseError::WrongType(format!("packages.{pkg_name}.sdist")))?;
    let url = t
        .get("url")
        .and_then(|v| v.as_str())
        .ok_or_else(|| PylockParseError::MissingField(format!("packages.{pkg_name}.sdist.url")))?;
    let sha = artifact_sha256(t, &format!("packages.{pkg_name}.sdist"))?;
    Ok((
        url.to_string(),
        sha,
        SourceRef {
            kind: SourceRefKind::Registry,
            path: None,
            url: None,
            rev: None,
        },
    ))
}

fn artifact_sha256(t: &toml::value::Table, path: &str) -> Result<String, PylockParseError> {
    let hashes = t
        .get("hashes")
        .and_then(|v| v.as_table())
        .ok_or_else(|| PylockParseError::MissingField(format!("{path}.hashes")))?;
    let sha = hashes
        .get("sha256")
        .and_then(|v| v.as_str())
        .ok_or_else(|| PylockParseError::MissingField(format!("{path}.hashes.sha256")))?;
    Ok(sha.to_string())
}

fn required_string(t: &toml::value::Table, key: &str) -> Result<String, PylockParseError> {
    let v = t
        .get(key)
        .ok_or_else(|| PylockParseError::MissingField(key.into()))?;
    let s = v
        .as_str()
        .ok_or_else(|| PylockParseError::WrongType(key.into()))?;
    Ok(s.to_string())
}

fn optional_string(t: &toml::value::Table, key: &str) -> Result<Option<String>, PylockParseError> {
    match t.get(key) {
        None => Ok(None),
        Some(v) => {
            let s = v
                .as_str()
                .ok_or_else(|| PylockParseError::WrongType(key.into()))?;
            Ok(Some(s.to_string()))
        }
    }
}

fn optional_string_array(
    t: &toml::value::Table,
    key: &str,
) -> Result<Option<Vec<String>>, PylockParseError> {
    let Some(v) = t.get(key) else {
        return Ok(None);
    };
    let arr = v
        .as_array()
        .ok_or_else(|| PylockParseError::WrongType(key.into()))?;
    let mut out = Vec::with_capacity(arr.len());
    for (idx, entry) in arr.iter().enumerate() {
        let s = entry
            .as_str()
            .ok_or_else(|| PylockParseError::WrongType(format!("{key}[{idx}]")))?;
        out.push(s.to_string());
    }
    Ok(Some(out))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pkgmanage::pkgmgr::pylock_export::{render_pylock_toml, PylockOptions};

    fn round_trip_packages(packages: Vec<Package>, opts: PylockOptions) -> PylockDocument {
        let lf = Lockfile {
            format_version: 1,
            input_hash: "ignored".into(),
            packages,
        };
        let body = render_pylock_toml(&lf, &opts);
        parse_pylock_toml(&body).expect("parse")
    }

    fn reg(name: &str, version: &str, url: &str, sha: &str) -> Package {
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
    fn rejects_invalid_toml() {
        let err = parse_pylock_toml("not = valid = toml").unwrap_err();
        assert!(matches!(err, PylockParseError::InvalidToml(_)));
    }

    #[test]
    fn rejects_unsupported_lock_version() {
        let body = "lock-version = \"0.9\"\n";
        let err = parse_pylock_toml(body).unwrap_err();
        assert!(matches!(err, PylockParseError::UnsupportedLockVersion(_)));
    }

    #[test]
    fn reads_header_fields() {
        let lf = Lockfile {
            format_version: 1,
            input_hash: "x".into(),
            packages: vec![],
        };
        let opts = PylockOptions {
            lock_version: "1.0".into(),
            created_by: "uv".into(),
            requires_python: Some(">=3.10".into()),
            environments: vec!["sys_platform == 'linux'".into()],
        };
        let body = render_pylock_toml(&lf, &opts);
        let doc = parse_pylock_toml(&body).unwrap();
        assert_eq!(doc.lock_version, "1.0");
        assert_eq!(doc.created_by.as_deref(), Some("uv"));
        assert_eq!(doc.requires_python.as_deref(), Some(">=3.10"));
        assert_eq!(doc.environments, vec!["sys_platform == 'linux'"]);
        assert!(doc.lockfile.packages.is_empty());
    }

    #[test]
    fn header_only_omits_optional_keys() {
        let body = "lock-version = \"1.0\"\n";
        let doc = parse_pylock_toml(body).unwrap();
        assert_eq!(doc.created_by, None);
        assert_eq!(doc.requires_python, None);
        assert!(doc.environments.is_empty());
    }

    #[test]
    fn round_trips_single_sdist_package() {
        let pkg = reg(
            "click",
            "8.1.7",
            "https://example.com/click-8.1.7.tar.gz",
            "deadbeef",
        );
        let doc = round_trip_packages(vec![pkg.clone()], PylockOptions::default());
        assert_eq!(doc.lockfile.packages.len(), 1);
        let got = &doc.lockfile.packages[0];
        assert_eq!(got.name, "click");
        assert_eq!(got.version, "8.1.7");
        assert_eq!(got.source, pkg.source);
        assert_eq!(got.sha256, "deadbeef");
        assert_eq!(
            got.source_ref.as_ref().unwrap().kind,
            SourceRefKind::Registry
        );
    }

    #[test]
    fn round_trips_wheel_artifact() {
        let pkg = reg(
            "numpy",
            "1.26.4",
            "https://example.com/numpy-1.26.4-cp312-cp312-linux.whl",
            "feedface",
        );
        let doc = round_trip_packages(vec![pkg.clone()], PylockOptions::default());
        let got = &doc.lockfile.packages[0];
        assert_eq!(got.source, pkg.source);
        assert_eq!(got.sha256, "feedface");
        assert_eq!(
            got.source_ref.as_ref().unwrap().kind,
            SourceRefKind::Registry
        );
    }

    #[test]
    fn round_trips_git_vcs_artifact() {
        let pkg = Package {
            name: "thing".into(),
            version: "0.1.0".into(),
            sha256: String::new(),
            source: "https://github.com/example/thing".into(),
            dependencies: vec![],
            markers: None,
            source_ref: Some(SourceRef {
                kind: SourceRefKind::Git,
                path: None,
                url: Some("https://github.com/example/thing".into()),
                rev: Some("abc123".into()),
            }),
        };
        let doc = round_trip_packages(vec![pkg], PylockOptions::default());
        let got = &doc.lockfile.packages[0];
        let sref = got.source_ref.as_ref().unwrap();
        assert_eq!(sref.kind, SourceRefKind::Git);
        assert_eq!(
            sref.url.as_deref(),
            Some("https://github.com/example/thing")
        );
        assert_eq!(sref.rev.as_deref(), Some("abc123"));
    }

    #[test]
    fn round_trips_directory_artifact() {
        let pkg = Package {
            name: "local".into(),
            version: "0.0.0".into(),
            sha256: String::new(),
            source: "./local-pkg".into(),
            dependencies: vec![],
            markers: None,
            source_ref: Some(SourceRef {
                kind: SourceRefKind::Path,
                path: Some("./local-pkg".into()),
                url: None,
                rev: None,
            }),
        };
        let doc = round_trip_packages(vec![pkg], PylockOptions::default());
        let got = &doc.lockfile.packages[0];
        let sref = got.source_ref.as_ref().unwrap();
        assert_eq!(sref.kind, SourceRefKind::Path);
        assert_eq!(sref.path.as_deref(), Some("./local-pkg"));
        assert_eq!(got.source, "./local-pkg");
    }

    #[test]
    fn round_trips_dependencies_and_marker() {
        let mut pkg = reg(
            "requests",
            "2.31.0",
            "https://example.com/requests-2.31.0.tar.gz",
            "cafebabe",
        );
        pkg.dependencies = vec!["urllib3".into(), "charset-normalizer".into()];
        pkg.markers = Some("python_version >= '3.10'".into());
        let doc = round_trip_packages(vec![pkg.clone()], PylockOptions::default());
        let got = &doc.lockfile.packages[0];
        assert_eq!(got.markers.as_deref(), Some("python_version >= '3.10'"));
        let mut deps = got.dependencies.clone();
        deps.sort();
        assert_eq!(deps, vec!["charset-normalizer", "urllib3"]);
    }

    #[test]
    fn accepts_dependencies_as_bare_strings() {
        let body = r#"lock-version = "1.0"

[[packages]]
name = "x"
version = "1.0.0"
dependencies = ["a", "b"]
sdist = { name = "x-1.0.0.tar.gz", url = "https://e.com/x.tgz", hashes = { sha256 = "1" } }
"#;
        let doc = parse_pylock_toml(body).unwrap();
        assert_eq!(doc.lockfile.packages[0].dependencies, vec!["a", "b"]);
    }

    #[test]
    fn missing_version_is_an_error() {
        let body = r#"lock-version = "1.0"

[[packages]]
name = "x"
sdist = { name = "x-1.0.tar.gz", url = "https://e.com/x", hashes = { sha256 = "1" } }
"#;
        let err = parse_pylock_toml(body).unwrap_err();
        assert!(matches!(err, PylockParseError::MissingField(s) if s == "version"));
    }

    #[test]
    fn package_without_artifact_is_an_error() {
        let body = r#"lock-version = "1.0"

[[packages]]
name = "x"
version = "1.0.0"
"#;
        let err = parse_pylock_toml(body).unwrap_err();
        assert!(matches!(err, PylockParseError::NoArtifact(name) if name == "x"));
    }

    #[test]
    fn missing_sha256_is_an_error() {
        let body = r#"lock-version = "1.0"

[[packages]]
name = "x"
version = "1.0.0"
sdist = { name = "x.tar.gz", url = "https://e.com/x", hashes = {} }
"#;
        let err = parse_pylock_toml(body).unwrap_err();
        assert!(matches!(err, PylockParseError::MissingField(s) if s.ends_with("sha256")));
    }

    #[test]
    fn vcs_with_non_git_type_is_an_error() {
        let body = r#"lock-version = "1.0"

[[packages]]
name = "x"
version = "1.0.0"
vcs = { type = "hg", url = "https://e.com/r" }
"#;
        let err = parse_pylock_toml(body).unwrap_err();
        assert!(matches!(err, PylockParseError::WrongType(_)));
    }

    #[test]
    fn empty_packages_is_ok() {
        let body = "lock-version = \"1.0\"\ncreated-by = \"mamba\"\n";
        let doc = parse_pylock_toml(body).unwrap();
        assert!(doc.lockfile.packages.is_empty());
    }

    #[test]
    fn round_trips_multiple_packages_preserving_count() {
        let pkgs = vec![
            reg("a", "1.0", "https://e.com/a-1.0.tar.gz", "01"),
            reg("b", "2.0", "https://e.com/b-2.0.tar.gz", "02"),
            reg("c", "3.0", "https://e.com/c-3.0.tar.gz", "03"),
        ];
        let doc = round_trip_packages(pkgs, PylockOptions::default());
        assert_eq!(doc.lockfile.packages.len(), 3);
        let mut names: Vec<&str> = doc
            .lockfile
            .packages
            .iter()
            .map(|p| p.name.as_str())
            .collect();
        names.sort();
        assert_eq!(names, vec!["a", "b", "c"]);
    }
}
