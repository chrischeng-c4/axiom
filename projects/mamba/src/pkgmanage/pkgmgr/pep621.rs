// PEP 621 `[project]` table reader (Tick 115).
//
// Parses the `[project]` table from a pyproject.toml file into a typed
// `ProjectTable`. Dependency lines flow through `requirement_string` so
// downstream consumers (pyproject_deps, workspace, init_scaffold) all
// see normalized names and structurally split extras / specifiers /
// markers.
//
// Spec: <https://packaging.python.org/en/latest/specifications/declaring-project-metadata/>
//
// Fields covered:
//   name, version, description, requires-python,
//   dependencies, optional-dependencies, dynamic,
//   authors, maintainers, license, readme, keywords, classifiers
//
// Authors and maintainers preserve both `name` and `email` keys when
// the TOML uses the inline-table form; the legacy string-only form is
// also accepted (treated as name-only).

use std::collections::BTreeMap;

use crate::pkgmanage::pkgmgr::name_normalize::pep503_normalize;
use crate::pkgmanage::pkgmgr::requirement_string::Requirement;
use crate::pkgmanage::pkgmgr::types::IndexError;

/// Author or maintainer entry — either or both of `name` / `email`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Person {
    pub name: Option<String>,
    pub email: Option<String>,
}

/// Readme reference — either a file path, an inline text+content-type, or
/// `None` if absent.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Readme {
    File(String),
    Inline { text: String, content_type: String },
}

/// License declaration. PEP 639 introduces SPDX expressions; this reader
/// keeps the legacy `{ file = ... }` / `{ text = ... }` shapes plus the
/// SPDX-expression shorthand string.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum License {
    File(String),
    Text(String),
    SpdxExpression(String),
}

/// PEP 621 `[project]` table.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ProjectTable {
    /// PEP 503-normalized distribution name.
    pub name: String,
    /// Raw distribution name as written in pyproject.toml.
    pub raw_name: String,
    /// Static version. Absent when the field is listed in `dynamic`.
    pub version: Option<String>,
    pub description: Option<String>,
    pub requires_python: Option<String>,
    pub dependencies: Vec<Requirement>,
    /// `[project.optional-dependencies]` — group name → requirements.
    pub optional_dependencies: BTreeMap<String, Vec<Requirement>>,
    /// Fields the build backend will fill in at build time.
    pub dynamic: Vec<String>,
    pub authors: Vec<Person>,
    pub maintainers: Vec<Person>,
    pub license: Option<License>,
    pub readme: Option<Readme>,
    pub keywords: Vec<String>,
    pub classifiers: Vec<String>,
}

impl ProjectTable {
    /// Parse a pyproject.toml source string and extract its `[project]`
    /// table. Missing table is an error; only valid surface is enforced.
    pub fn parse(src: &str) -> Result<Self, IndexError> {
        let root: toml::Value = toml::from_str(src).map_err(|e| parse_err(e.to_string()))?;
        let project = root
            .get("project")
            .ok_or_else(|| parse_err("pyproject.toml has no [project] table"))?;
        Self::from_value(project)
    }

    /// Build from an already-parsed TOML value pointing at the `[project]`
    /// table (useful when the caller has already loaded pyproject.toml).
    pub fn from_value(project: &toml::Value) -> Result<Self, IndexError> {
        let table = project
            .as_table()
            .ok_or_else(|| parse_err("[project] is not a table"))?;

        let raw_name = take_string(table, "name")?
            .ok_or_else(|| parse_err("[project] is missing required field `name`"))?;
        let name = pep503_normalize(&raw_name);
        if name.is_empty() {
            return Err(parse_err("[project].name normalizes to an empty string"));
        }

        let dynamic = take_string_array(table, "dynamic")?;
        let version = take_string(table, "version")?;
        // PEP 621 §"version": MUST appear either as a static string or in
        // the `dynamic` array — never both, never neither.
        let version_is_dynamic = dynamic.iter().any(|s| s == "version");
        match (&version, version_is_dynamic) {
            (Some(_), true) => {
                return Err(parse_err(
                    "[project].version is both static and listed in `dynamic`",
                ))
            }
            (None, false) => {
                return Err(parse_err(
                    "[project].version is missing and not listed in `dynamic`",
                ))
            }
            _ => {}
        }

        let dependencies = take_requirement_array(table, "dependencies")?;
        let optional_dependencies = take_optional_dependencies(table)?;

        let authors = take_person_array(table, "authors")?;
        let maintainers = take_person_array(table, "maintainers")?;
        let license = take_license(table)?;
        let readme = take_readme(table)?;
        let keywords = take_string_array(table, "keywords")?;
        let classifiers = take_string_array(table, "classifiers")?;

        Ok(Self {
            name,
            raw_name,
            version,
            description: take_string(table, "description")?,
            requires_python: take_string(table, "requires-python")?,
            dependencies,
            optional_dependencies,
            dynamic,
            authors,
            maintainers,
            license,
            readme,
            keywords,
            classifiers,
        })
    }

    /// Look up an extras group, applying PEP 685 normalization to the key.
    pub fn extras_group(&self, name: &str) -> Option<&[Requirement]> {
        let normalized = pep685_normalize(name);
        self.optional_dependencies
            .iter()
            .find(|(k, _)| pep685_normalize(k) == normalized)
            .map(|(_, v)| v.as_slice())
    }
}

fn parse_err(detail: impl Into<String>) -> IndexError {
    IndexError::ParseError {
        url: "pyproject.toml".to_string(),
        detail: detail.into(),
    }
}

fn take_string(table: &toml::value::Table, key: &str) -> Result<Option<String>, IndexError> {
    match table.get(key) {
        None => Ok(None),
        Some(toml::Value::String(s)) => Ok(Some(s.clone())),
        Some(other) => Err(parse_err(format!(
            "[project].{key} must be a string, got {}",
            other.type_str()
        ))),
    }
}

fn take_string_array(table: &toml::value::Table, key: &str) -> Result<Vec<String>, IndexError> {
    match table.get(key) {
        None => Ok(Vec::new()),
        Some(toml::Value::Array(arr)) => {
            let mut out = Vec::with_capacity(arr.len());
            for (i, v) in arr.iter().enumerate() {
                let s = v.as_str().ok_or_else(|| {
                    parse_err(format!(
                        "[project].{key}[{i}] must be a string, got {}",
                        v.type_str()
                    ))
                })?;
                out.push(s.to_string());
            }
            Ok(out)
        }
        Some(other) => Err(parse_err(format!(
            "[project].{key} must be an array of strings, got {}",
            other.type_str()
        ))),
    }
}

fn take_requirement_array(
    table: &toml::value::Table,
    key: &str,
) -> Result<Vec<Requirement>, IndexError> {
    let lines = take_string_array(table, key)?;
    let mut out = Vec::with_capacity(lines.len());
    for line in lines {
        out.push(Requirement::parse(&line)?);
    }
    Ok(out)
}

fn take_optional_dependencies(
    table: &toml::value::Table,
) -> Result<BTreeMap<String, Vec<Requirement>>, IndexError> {
    let Some(value) = table.get("optional-dependencies") else {
        return Ok(BTreeMap::new());
    };
    let inner = value
        .as_table()
        .ok_or_else(|| parse_err("[project.optional-dependencies] must be a table"))?;
    let mut out = BTreeMap::new();
    for (group, deps_value) in inner.iter() {
        let arr = deps_value.as_array().ok_or_else(|| {
            parse_err(format!(
                "[project.optional-dependencies].{group} must be an array of requirement strings"
            ))
        })?;
        let mut reqs = Vec::with_capacity(arr.len());
        for (i, v) in arr.iter().enumerate() {
            let line = v.as_str().ok_or_else(|| {
                parse_err(format!(
                    "[project.optional-dependencies].{group}[{i}] must be a string"
                ))
            })?;
            reqs.push(Requirement::parse(line)?);
        }
        out.insert(group.clone(), reqs);
    }
    Ok(out)
}

fn take_person_array(table: &toml::value::Table, key: &str) -> Result<Vec<Person>, IndexError> {
    let Some(value) = table.get(key) else {
        return Ok(Vec::new());
    };
    let arr = value
        .as_array()
        .ok_or_else(|| parse_err(format!("[project].{key} must be an array")))?;
    let mut out = Vec::with_capacity(arr.len());
    for (i, v) in arr.iter().enumerate() {
        let person = match v {
            toml::Value::String(s) => Person {
                name: Some(s.clone()),
                email: None,
            },
            toml::Value::Table(t) => {
                let name = t
                    .get("name")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string());
                let email = t
                    .get("email")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string());
                if name.is_none() && email.is_none() {
                    return Err(parse_err(format!(
                        "[project].{key}[{i}] must have at least `name` or `email`"
                    )));
                }
                Person { name, email }
            }
            other => {
                return Err(parse_err(format!(
                    "[project].{key}[{i}] must be a string or inline table, got {}",
                    other.type_str()
                )))
            }
        };
        out.push(person);
    }
    Ok(out)
}

fn take_license(table: &toml::value::Table) -> Result<Option<License>, IndexError> {
    let Some(value) = table.get("license") else {
        return Ok(None);
    };
    match value {
        toml::Value::String(s) => Ok(Some(License::SpdxExpression(s.clone()))),
        toml::Value::Table(t) => {
            let has_file = t.contains_key("file");
            let has_text = t.contains_key("text");
            match (has_file, has_text) {
                (true, true) => Err(parse_err("[project].license sets both `file` and `text`")),
                (true, false) => {
                    let file = t
                        .get("file")
                        .and_then(|v| v.as_str())
                        .ok_or_else(|| parse_err("[project].license.file must be a string"))?;
                    Ok(Some(License::File(file.to_string())))
                }
                (false, true) => {
                    let text = t
                        .get("text")
                        .and_then(|v| v.as_str())
                        .ok_or_else(|| parse_err("[project].license.text must be a string"))?;
                    Ok(Some(License::Text(text.to_string())))
                }
                (false, false) => Err(parse_err(
                    "[project].license table must set `file` or `text`",
                )),
            }
        }
        other => Err(parse_err(format!(
            "[project].license must be a string or table, got {}",
            other.type_str()
        ))),
    }
}

fn take_readme(table: &toml::value::Table) -> Result<Option<Readme>, IndexError> {
    let Some(value) = table.get("readme") else {
        return Ok(None);
    };
    match value {
        toml::Value::String(s) => Ok(Some(Readme::File(s.clone()))),
        toml::Value::Table(t) => {
            let file = t
                .get("file")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());
            let text = t
                .get("text")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());
            let content_type = t
                .get("content-type")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());
            match (file, text, content_type) {
                (Some(f), None, _) => Ok(Some(Readme::File(f))),
                (None, Some(t), Some(ct)) => Ok(Some(Readme::Inline {
                    text: t,
                    content_type: ct,
                })),
                (None, Some(_), None) => {
                    Err(parse_err("[project].readme.text requires `content-type`"))
                }
                (Some(_), Some(_), _) => {
                    Err(parse_err("[project].readme sets both `file` and `text`"))
                }
                (None, None, _) => Err(parse_err(
                    "[project].readme table must set `file` or `text`",
                )),
            }
        }
        other => Err(parse_err(format!(
            "[project].readme must be a string or table, got {}",
            other.type_str()
        ))),
    }
}

fn pep685_normalize(name: &str) -> String {
    let mut out = String::with_capacity(name.len());
    let mut last_dash = false;
    for c in name.chars() {
        let lc = c.to_ascii_lowercase();
        if matches!(lc, '.' | '_' | '-') {
            if !last_dash {
                out.push('-');
            }
            last_dash = true;
        } else {
            out.push(lc);
            last_dash = false;
        }
    }
    out.trim_matches('-').to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(src: &str) -> ProjectTable {
        ProjectTable::parse(src).unwrap_or_else(|e| panic!("parse failed: {e:?}"))
    }

    const MINIMAL: &str = r#"
[project]
name = "demo-pkg"
version = "0.1.0"
"#;

    #[test]
    fn parses_minimal_project() {
        let p = parse(MINIMAL);
        assert_eq!(p.name, "demo-pkg");
        assert_eq!(p.raw_name, "demo-pkg");
        assert_eq!(p.version.as_deref(), Some("0.1.0"));
        assert!(p.dependencies.is_empty());
    }

    #[test]
    fn normalizes_name() {
        let src = r#"
[project]
name = "My.Pkg_Name"
version = "1"
"#;
        let p = parse(src);
        assert_eq!(p.name, "my-pkg-name");
        assert_eq!(p.raw_name, "My.Pkg_Name");
    }

    #[test]
    fn missing_project_table_errors() {
        let err = ProjectTable::parse(
            r#"[build-system]
requires = ["setuptools"]"#,
        )
        .unwrap_err();
        let msg = format!("{err:?}");
        assert!(msg.contains("[project] table"));
    }

    #[test]
    fn missing_name_errors() {
        let err = ProjectTable::parse(
            r#"[project]
version = "1.0"
"#,
        )
        .unwrap_err();
        let msg = format!("{err:?}");
        assert!(msg.contains("name"));
    }

    #[test]
    fn missing_version_errors_when_not_dynamic() {
        let err = ProjectTable::parse(
            r#"[project]
name = "x"
"#,
        )
        .unwrap_err();
        let msg = format!("{err:?}");
        assert!(msg.contains("version"));
    }

    #[test]
    fn dynamic_version_ok() {
        let src = r#"
[project]
name = "x"
dynamic = ["version"]
"#;
        let p = parse(src);
        assert_eq!(p.version, None);
        assert_eq!(p.dynamic, vec!["version".to_string()]);
    }

    #[test]
    fn version_static_and_dynamic_errors() {
        let err = ProjectTable::parse(
            r#"
[project]
name = "x"
version = "1"
dynamic = ["version"]
"#,
        )
        .unwrap_err();
        let msg = format!("{err:?}");
        assert!(msg.contains("both static and listed in `dynamic`"));
    }

    #[test]
    fn parses_dependencies_through_requirement_string() {
        let src = r#"
[project]
name = "x"
version = "1"
dependencies = [
    "requests >= 2.31",
    "Django[argon2,bcrypt] >= 4.2",
]
"#;
        let p = parse(src);
        assert_eq!(p.dependencies.len(), 2);
        assert_eq!(p.dependencies[0].name, "requests");
        assert_eq!(p.dependencies[0].specifier.as_deref(), Some(">= 2.31"));
        assert_eq!(p.dependencies[1].name, "django");
        assert!(p.dependencies[1].extras.contains("argon2"));
        assert!(p.dependencies[1].extras.contains("bcrypt"));
    }

    #[test]
    fn rejects_invalid_dependency_line() {
        let err = ProjectTable::parse(
            r#"
[project]
name = "x"
version = "1"
dependencies = ["[broken"]
"#,
        )
        .unwrap_err();
        let msg = format!("{err:?}");
        assert!(msg.contains("letter or digit") || msg.contains("name"));
    }

    #[test]
    fn parses_optional_dependencies() {
        let src = r#"
[project]
name = "x"
version = "1"

[project.optional-dependencies]
test = ["pytest>=7", "pytest-cov"]
docs = ["sphinx"]
"#;
        let p = parse(src);
        assert_eq!(p.optional_dependencies.len(), 2);
        assert_eq!(p.optional_dependencies["test"].len(), 2);
        assert_eq!(p.optional_dependencies["test"][0].name, "pytest");
        assert_eq!(p.optional_dependencies["docs"][0].name, "sphinx");
    }

    #[test]
    fn extras_group_lookup_normalizes() {
        let src = r#"
[project]
name = "x"
version = "1"

[project.optional-dependencies]
"Dev_Tools" = ["mypy"]
"#;
        let p = parse(src);
        // PEP 685 normalization: "Dev_Tools" → "dev-tools"
        let group = p.extras_group("dev-tools").unwrap();
        assert_eq!(group[0].name, "mypy");
        // Round-trip lookup via the unnormalized form.
        let group2 = p.extras_group("Dev.Tools").unwrap();
        assert_eq!(group2[0].name, "mypy");
    }

    #[test]
    fn parses_authors_inline_table_form() {
        let src = r#"
[project]
name = "x"
version = "1"
authors = [
    { name = "Alice", email = "alice@example.com" },
    { email = "bob@example.com" },
]
"#;
        let p = parse(src);
        assert_eq!(p.authors.len(), 2);
        assert_eq!(p.authors[0].name.as_deref(), Some("Alice"));
        assert_eq!(p.authors[0].email.as_deref(), Some("alice@example.com"));
        assert_eq!(p.authors[1].name, None);
        assert_eq!(p.authors[1].email.as_deref(), Some("bob@example.com"));
    }

    #[test]
    fn parses_authors_string_form() {
        let src = r#"
[project]
name = "x"
version = "1"
authors = ["Alice"]
"#;
        let p = parse(src);
        assert_eq!(p.authors.len(), 1);
        assert_eq!(p.authors[0].name.as_deref(), Some("Alice"));
        assert_eq!(p.authors[0].email, None);
    }

    #[test]
    fn rejects_empty_person_entry() {
        let err = ProjectTable::parse(
            r#"
[project]
name = "x"
version = "1"
authors = [{}]
"#,
        )
        .unwrap_err();
        let msg = format!("{err:?}");
        assert!(msg.contains("name") || msg.contains("email"));
    }

    #[test]
    fn parses_license_spdx_expression() {
        let src = r#"
[project]
name = "x"
version = "1"
license = "MIT OR Apache-2.0"
"#;
        let p = parse(src);
        assert_eq!(
            p.license,
            Some(License::SpdxExpression("MIT OR Apache-2.0".to_string()))
        );
    }

    #[test]
    fn parses_license_file() {
        let src = r#"
[project]
name = "x"
version = "1"
license = { file = "LICENSE.txt" }
"#;
        let p = parse(src);
        assert_eq!(p.license, Some(License::File("LICENSE.txt".to_string())));
    }

    #[test]
    fn parses_license_text() {
        let src = r#"
[project]
name = "x"
version = "1"
license = { text = "Proprietary" }
"#;
        let p = parse(src);
        assert_eq!(p.license, Some(License::Text("Proprietary".to_string())));
    }

    #[test]
    fn license_with_both_file_and_text_errors() {
        let err = ProjectTable::parse(
            r#"
[project]
name = "x"
version = "1"
license = { file = "LICENSE", text = "x" }
"#,
        )
        .unwrap_err();
        let msg = format!("{err:?}");
        assert!(msg.contains("both `file` and `text`"));
    }

    #[test]
    fn parses_readme_string_form() {
        let src = r#"
[project]
name = "x"
version = "1"
readme = "README.md"
"#;
        let p = parse(src);
        assert_eq!(p.readme, Some(Readme::File("README.md".to_string())));
    }

    #[test]
    fn parses_readme_inline_form() {
        let src = r##"
[project]
name = "x"
version = "1"
readme = { text = "# Hello", content-type = "text/markdown" }
"##;
        let p = parse(src);
        assert_eq!(
            p.readme,
            Some(Readme::Inline {
                text: "# Hello".to_string(),
                content_type: "text/markdown".to_string(),
            })
        );
    }

    #[test]
    fn readme_text_without_content_type_errors() {
        let err = ProjectTable::parse(
            r#"
[project]
name = "x"
version = "1"
readme = { text = "no content type" }
"#,
        )
        .unwrap_err();
        let msg = format!("{err:?}");
        assert!(msg.contains("content-type"));
    }

    #[test]
    fn parses_keywords_and_classifiers() {
        let src = r#"
[project]
name = "x"
version = "1"
keywords = ["http", "client"]
classifiers = [
    "Development Status :: 5 - Production/Stable",
    "Programming Language :: Python :: 3",
]
"#;
        let p = parse(src);
        assert_eq!(p.keywords, vec!["http", "client"]);
        assert_eq!(p.classifiers.len(), 2);
        assert!(p.classifiers[0].starts_with("Development Status"));
    }

    #[test]
    fn realistic_pyproject_round_trip() {
        // Approximates the requests project's pyproject.toml.
        let src = r#"
[project]
name = "requests"
version = "2.31.0"
description = "Python HTTP for Humans."
requires-python = ">=3.7"
authors = [{ name = "Kenneth Reitz", email = "me@kennethreitz.org" }]
license = "Apache-2.0"
readme = "README.md"
dependencies = [
    "charset-normalizer>=2,<4",
    "idna>=2.5,<4",
    "urllib3>=1.21.1,<3",
    "certifi>=2017.4.17",
]

[project.optional-dependencies]
socks = ["PySocks>=1.5.6, !=1.5.7"]
use_chardet_on_py3 = ["chardet>=3.0.2,<6"]
"#;
        let p = parse(src);
        assert_eq!(p.name, "requests");
        assert_eq!(p.version.as_deref(), Some("2.31.0"));
        assert_eq!(p.requires_python.as_deref(), Some(">=3.7"));
        assert_eq!(p.authors[0].name.as_deref(), Some("Kenneth Reitz"));
        assert_eq!(p.dependencies.len(), 4);
        assert_eq!(p.dependencies[0].name, "charset-normalizer");
        assert_eq!(p.dependencies[2].name, "urllib3");
        assert_eq!(p.dependencies[2].specifier.as_deref(), Some(">=1.21.1,<3"));
        assert_eq!(p.optional_dependencies.len(), 2);
        let socks = p.extras_group("socks").unwrap();
        assert_eq!(socks[0].name, "pysocks");
    }

    #[test]
    fn rejects_wrong_type_for_name() {
        let err = ProjectTable::parse(
            r#"
[project]
name = 123
version = "1"
"#,
        )
        .unwrap_err();
        let msg = format!("{err:?}");
        assert!(msg.contains("string"));
    }

    #[test]
    fn rejects_optional_dependencies_not_a_table() {
        let err = ProjectTable::parse(
            r#"
[project]
name = "x"
version = "1"
optional-dependencies = ["bad"]
"#,
        )
        .unwrap_err();
        let msg = format!("{err:?}");
        assert!(msg.contains("table"));
    }
}
