// pkgmanage::lockfile — top-level mamba.lock (project-wide).
//
// Distinct from `pkgmanage::pkgmgr::lockfile`, which is the resolver's pure
// schema. This module owns the user-facing `mamba.lock` shape written next to
// `mamba.toml` and adapts legacy/top-level fields into the resolver schema for
// read-only package-manager commands such as `mamba export` and `mamba tree`.

use std::path::Path;

use anyhow::{bail, Context, Result};

use crate::pkgmanage::pkgmgr::lockfile::{
    Lockfile, Package, SourceRef, SourceRefKind, MAX_SUPPORTED_FORMAT_VERSION,
};

pub fn read_user_lockfile(path: &Path) -> Result<Lockfile> {
    let src = std::fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
    parse_user_lockfile(&src).with_context(|| format!("parse {}", path.display()))
}

pub fn parse_user_lockfile(src: &str) -> Result<Lockfile> {
    let doc: toml::Value = src.parse().context("parse TOML")?;
    let format_version = doc
        .get("format_version")
        .and_then(|v| v.as_integer())
        .context("mamba.lock missing integer `format_version`")?;
    if format_version < 0 || format_version as u32 > MAX_SUPPORTED_FORMAT_VERSION {
        bail!(
            "unsupported format_version {} (max supported {})",
            format_version,
            MAX_SUPPORTED_FORMAT_VERSION
        );
    }
    let input_hash = doc
        .get("input_hash")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    let packages = doc
        .get("package")
        .and_then(|v| v.as_array())
        .map(|entries| {
            entries
                .iter()
                .map(parse_package)
                .collect::<Result<Vec<Package>>>()
        })
        .transpose()?
        .unwrap_or_default();

    Ok(Lockfile {
        format_version: format_version as u32,
        input_hash,
        packages,
    })
}

fn parse_package(entry: &toml::Value) -> Result<Package> {
    let tbl = entry
        .as_table()
        .context("mamba.lock package entry is not a table")?;
    let name = required_str(tbl, "name")?.to_string();
    let version = required_str(tbl, "version")?.to_string();
    let sha256 = optional_str(tbl, "sha256").unwrap_or_default();
    let url = optional_str(tbl, "url").unwrap_or_default();
    let source_raw = optional_str(tbl, "source").unwrap_or_default();
    let source = if !url.is_empty() {
        url.clone()
    } else {
        source_raw
    };
    let dependencies = tbl
        .get("dependencies")
        .and_then(|v| v.as_array())
        .map(|deps| {
            deps.iter()
                .filter_map(|v| v.as_str())
                .map(dependency_name)
                .collect()
        })
        .unwrap_or_default();
    let markers = optional_str(tbl, "markers").or_else(|| optional_str(tbl, "marker"));
    let source_ref = parse_source_ref(tbl);

    Ok(Package {
        name,
        version,
        sha256,
        source,
        dependencies,
        markers,
        source_ref,
    })
}

fn parse_source_ref(tbl: &toml::map::Map<String, toml::Value>) -> Option<SourceRef> {
    if let Some(source_ref) = tbl.get("source_ref").and_then(|v| v.as_table()) {
        let kind = source_ref
            .get("kind")
            .and_then(|v| v.as_str())
            .and_then(parse_source_kind)?;
        return Some(SourceRef {
            kind,
            path: optional_str(source_ref, "path"),
            url: optional_str(source_ref, "url"),
            rev: optional_str(source_ref, "rev"),
        });
    }

    let source_kind = optional_str(tbl, "source_kind")?;
    match source_kind.as_str() {
        "direct_file" | "path" | "editable" => Some(SourceRef {
            kind: SourceRefKind::Path,
            path: optional_str(tbl, "path"),
            url: None,
            rev: None,
        }),
        "git" => Some(SourceRef {
            kind: SourceRefKind::Git,
            path: None,
            url: optional_str(tbl, "url"),
            rev: optional_str(tbl, "rev"),
        }),
        "registry" => Some(SourceRef {
            kind: SourceRefKind::Registry,
            path: None,
            url: optional_str(tbl, "url"),
            rev: None,
        }),
        _ => None,
    }
}

fn parse_source_kind(raw: &str) -> Option<SourceRefKind> {
    match raw {
        "registry" => Some(SourceRefKind::Registry),
        "path" | "direct_file" | "editable" => Some(SourceRefKind::Path),
        "git" => Some(SourceRefKind::Git),
        _ => None,
    }
}

fn required_str<'a>(tbl: &'a toml::map::Map<String, toml::Value>, key: &str) -> Result<&'a str> {
    tbl.get(key)
        .and_then(|v| v.as_str())
        .with_context(|| format!("mamba.lock package missing `{key}`"))
}

fn optional_str(tbl: &toml::map::Map<String, toml::Value>, key: &str) -> Option<String> {
    tbl.get(key)
        .and_then(|v| v.as_str())
        .map(ToString::to_string)
}

fn dependency_name(spec: &str) -> String {
    let trimmed = spec.trim();
    let head = trimmed
        .split_once(';')
        .map(|(name, _)| name)
        .unwrap_or(trimmed)
        .trim();
    let name_end = head
        .char_indices()
        .find_map(|(idx, ch)| {
            if matches!(ch, '<' | '>' | '=' | '!' | '~') || ch.is_whitespace() || ch == '[' {
                Some(idx)
            } else {
                None
            }
        })
        .unwrap_or(head.len());
    head[..name_end].trim().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn user_lockfile_parser_keeps_url_as_registry_source() {
        let lock = parse_user_lockfile(
            r#"
format_version = 1
input_hash = "x"

[[package]]
name = "demo"
version = "1.0.0"
sha256 = "abc"
url = "https://example.test/demo-1.0.0.whl"
source = "pypi://demo/1.0.0"
dependencies = ["child==2.0.0"]
"#,
        )
        .unwrap();
        assert_eq!(
            lock.packages[0].source,
            "https://example.test/demo-1.0.0.whl"
        );
        assert_eq!(lock.packages[0].dependencies, vec!["child"]);
    }
}
