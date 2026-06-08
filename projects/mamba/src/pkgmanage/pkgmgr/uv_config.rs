// `[tool.uv]` config reader (Tick 58).
//
// uv exposes a handful of resolver / installer knobs under `[tool.uv]`
// in `pyproject.toml`. This module parses them into typed enums + value
// lists so downstream code can match against `enum` variants instead of
// stringly-typed parameters. Pure-data: no resolver lookup, no I/O.
//
// Knobs covered:
//   * resolution     — "highest" | "lowest" | "lowest-direct"
//   * prerelease     — "disallow" | "allow" | "if-necessary"
//                       | "explicit" | "if-necessary-or-explicit"
//   * link-mode      — "clone" | "copy" | "hardlink" | "symlink"
//   * python-preference — "only-managed" | "managed"
//                          | "system" | "only-system"
//   * constraint-dependencies — array of requirement strings
//   * override-dependencies   — array of requirement strings
//   * compile-bytecode        — bool
//   * no-build                — bool
//   * no-binary               — bool
//   * managed                 — bool (project-controlled flag)
//
// Everything is optional; unspecified knobs become `None` so callers
// can layer in their own defaults.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResolutionMode {
    Highest,
    Lowest,
    LowestDirect,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PrereleaseMode {
    Disallow,
    Allow,
    IfNecessary,
    Explicit,
    IfNecessaryOrExplicit,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LinkMode {
    Clone,
    Copy,
    Hardlink,
    Symlink,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PythonPreference {
    OnlyManaged,
    Managed,
    System,
    OnlySystem,
}

/// Decoded `[tool.uv]` block. Every field is optional — absent or
/// outside the table means `None`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct UvConfig {
    pub resolution: Option<ResolutionMode>,
    pub prerelease: Option<PrereleaseMode>,
    pub link_mode: Option<LinkMode>,
    pub python_preference: Option<PythonPreference>,
    pub constraint_dependencies: Vec<String>,
    pub override_dependencies: Vec<String>,
    pub compile_bytecode: Option<bool>,
    pub no_build: Option<bool>,
    pub no_binary: Option<bool>,
    pub managed: Option<bool>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UvConfigError {
    InvalidToml(String),
    WrongType {
        field: String,
    },
    UnknownValue {
        field: String,
        value: String,
    },
}

impl std::fmt::Display for UvConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UvConfigError::InvalidToml(s) => write!(f, "invalid toml: {s}"),
            UvConfigError::WrongType { field } => {
                write!(f, "[tool.uv].{field}: wrong type")
            }
            UvConfigError::UnknownValue { field, value } => {
                write!(f, "[tool.uv].{field}: unknown value {value:?}")
            }
        }
    }
}

impl std::error::Error for UvConfigError {}

/// Parse `[tool.uv]` from a `pyproject.toml` body. Returns the default
/// (all-None) config when the table is absent.
pub fn parse_uv_config(toml_src: &str) -> Result<UvConfig, UvConfigError> {
    let doc: toml::Value = toml_src
        .parse()
        .map_err(|e: toml::de::Error| UvConfigError::InvalidToml(e.to_string()))?;

    let Some(table) = doc.get("tool").and_then(|t| t.get("uv")) else {
        return Ok(UvConfig::default());
    };
    let table = table
        .as_table()
        .ok_or_else(|| UvConfigError::WrongType { field: "uv".into() })?;

    parse_uv_table(table)
}

/// Decode a uv config from an already-extracted TOML table. Shared by
/// `parse_uv_config` (which strips `[tool.uv]`) and `parse_uv_toml`
/// (which feeds in the top-level document of a standalone `uv.toml`).
pub(super) fn parse_uv_table(table: &toml::value::Table) -> Result<UvConfig, UvConfigError> {
    Ok(UvConfig {
        resolution: optional_enum(table, "resolution", |s| match s {
            "highest" => Some(ResolutionMode::Highest),
            "lowest" => Some(ResolutionMode::Lowest),
            "lowest-direct" => Some(ResolutionMode::LowestDirect),
            _ => None,
        })?,
        prerelease: optional_enum(table, "prerelease", |s| match s {
            "disallow" => Some(PrereleaseMode::Disallow),
            "allow" => Some(PrereleaseMode::Allow),
            "if-necessary" => Some(PrereleaseMode::IfNecessary),
            "explicit" => Some(PrereleaseMode::Explicit),
            "if-necessary-or-explicit" => Some(PrereleaseMode::IfNecessaryOrExplicit),
            _ => None,
        })?,
        link_mode: optional_enum(table, "link-mode", |s| match s {
            "clone" => Some(LinkMode::Clone),
            "copy" => Some(LinkMode::Copy),
            "hardlink" => Some(LinkMode::Hardlink),
            "symlink" => Some(LinkMode::Symlink),
            _ => None,
        })?,
        python_preference: optional_enum(table, "python-preference", |s| match s {
            "only-managed" => Some(PythonPreference::OnlyManaged),
            "managed" => Some(PythonPreference::Managed),
            "system" => Some(PythonPreference::System),
            "only-system" => Some(PythonPreference::OnlySystem),
            _ => None,
        })?,
        constraint_dependencies: optional_string_array(table, "constraint-dependencies")?,
        override_dependencies: optional_string_array(table, "override-dependencies")?,
        compile_bytecode: optional_bool(table, "compile-bytecode")?,
        no_build: optional_bool(table, "no-build")?,
        no_binary: optional_bool(table, "no-binary")?,
        managed: optional_bool(table, "managed")?,
    })
}

pub(super) fn optional_enum<T>(
    table: &toml::value::Table,
    field: &str,
    decode: impl Fn(&str) -> Option<T>,
) -> Result<Option<T>, UvConfigError> {
    let Some(v) = table.get(field) else {
        return Ok(None);
    };
    let s = v.as_str().ok_or_else(|| UvConfigError::WrongType {
        field: field.into(),
    })?;
    match decode(s) {
        Some(t) => Ok(Some(t)),
        None => Err(UvConfigError::UnknownValue {
            field: field.into(),
            value: s.into(),
        }),
    }
}

pub(super) fn optional_string_array(
    table: &toml::value::Table,
    field: &str,
) -> Result<Vec<String>, UvConfigError> {
    let Some(v) = table.get(field) else {
        return Ok(Vec::new());
    };
    let arr = v.as_array().ok_or_else(|| UvConfigError::WrongType {
        field: field.into(),
    })?;
    let mut out = Vec::with_capacity(arr.len());
    for (idx, entry) in arr.iter().enumerate() {
        let s = entry.as_str().ok_or_else(|| UvConfigError::WrongType {
            field: format!("{field}[{idx}]"),
        })?;
        out.push(s.to_string());
    }
    Ok(out)
}

pub(super) fn optional_bool(
    table: &toml::value::Table,
    field: &str,
) -> Result<Option<bool>, UvConfigError> {
    let Some(v) = table.get(field) else {
        return Ok(None);
    };
    let b = v.as_bool().ok_or_else(|| UvConfigError::WrongType {
        field: field.into(),
    })?;
    Ok(Some(b))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn missing_table_yields_default() {
        let src = r#"[project]
name = "x"
version = "0.0.0"
"#;
        let cfg = parse_uv_config(src).unwrap();
        assert_eq!(cfg, UvConfig::default());
    }

    #[test]
    fn invalid_toml_is_an_error() {
        let err = parse_uv_config("not = = valid").unwrap_err();
        assert!(matches!(err, UvConfigError::InvalidToml(_)));
    }

    #[test]
    fn parses_resolution_modes() {
        for (raw, expected) in [
            ("highest", ResolutionMode::Highest),
            ("lowest", ResolutionMode::Lowest),
            ("lowest-direct", ResolutionMode::LowestDirect),
        ] {
            let src = format!("[tool.uv]\nresolution = \"{raw}\"\n");
            let cfg = parse_uv_config(&src).unwrap();
            assert_eq!(cfg.resolution, Some(expected));
        }
    }

    #[test]
    fn rejects_unknown_resolution() {
        let src = "[tool.uv]\nresolution = \"bizarre\"\n";
        let err = parse_uv_config(src).unwrap_err();
        assert!(matches!(
            err,
            UvConfigError::UnknownValue { field, .. } if field == "resolution"
        ));
    }

    #[test]
    fn parses_prerelease_modes() {
        for (raw, expected) in [
            ("disallow", PrereleaseMode::Disallow),
            ("allow", PrereleaseMode::Allow),
            ("if-necessary", PrereleaseMode::IfNecessary),
            ("explicit", PrereleaseMode::Explicit),
            ("if-necessary-or-explicit", PrereleaseMode::IfNecessaryOrExplicit),
        ] {
            let src = format!("[tool.uv]\nprerelease = \"{raw}\"\n");
            let cfg = parse_uv_config(&src).unwrap();
            assert_eq!(cfg.prerelease, Some(expected));
        }
    }

    #[test]
    fn parses_link_modes() {
        for (raw, expected) in [
            ("clone", LinkMode::Clone),
            ("copy", LinkMode::Copy),
            ("hardlink", LinkMode::Hardlink),
            ("symlink", LinkMode::Symlink),
        ] {
            let src = format!("[tool.uv]\nlink-mode = \"{raw}\"\n");
            let cfg = parse_uv_config(&src).unwrap();
            assert_eq!(cfg.link_mode, Some(expected));
        }
    }

    #[test]
    fn parses_python_preference() {
        for (raw, expected) in [
            ("only-managed", PythonPreference::OnlyManaged),
            ("managed", PythonPreference::Managed),
            ("system", PythonPreference::System),
            ("only-system", PythonPreference::OnlySystem),
        ] {
            let src = format!("[tool.uv]\npython-preference = \"{raw}\"\n");
            let cfg = parse_uv_config(&src).unwrap();
            assert_eq!(cfg.python_preference, Some(expected));
        }
    }

    #[test]
    fn parses_string_arrays() {
        let src = r#"[tool.uv]
constraint-dependencies = ["pkg-a==1.0", "pkg-b>=2"]
override-dependencies = ["pkg-c<3"]
"#;
        let cfg = parse_uv_config(src).unwrap();
        assert_eq!(cfg.constraint_dependencies, vec!["pkg-a==1.0", "pkg-b>=2"]);
        assert_eq!(cfg.override_dependencies, vec!["pkg-c<3"]);
    }

    #[test]
    fn parses_bool_knobs() {
        let src = r#"[tool.uv]
compile-bytecode = true
no-build = false
no-binary = true
managed = true
"#;
        let cfg = parse_uv_config(src).unwrap();
        assert_eq!(cfg.compile_bytecode, Some(true));
        assert_eq!(cfg.no_build, Some(false));
        assert_eq!(cfg.no_binary, Some(true));
        assert_eq!(cfg.managed, Some(true));
    }

    #[test]
    fn rejects_non_string_array_element() {
        let src = r#"[tool.uv]
constraint-dependencies = ["pkg-a==1.0", 42]
"#;
        let err = parse_uv_config(src).unwrap_err();
        assert!(matches!(err, UvConfigError::WrongType { field } if field.starts_with("constraint-dependencies[")));
    }

    #[test]
    fn rejects_wrong_bool_type() {
        let src = r#"[tool.uv]
compile-bytecode = "yes"
"#;
        let err = parse_uv_config(src).unwrap_err();
        assert!(matches!(
            err,
            UvConfigError::WrongType { field } if field == "compile-bytecode"
        ));
    }

    #[test]
    fn rejects_wrong_enum_type() {
        let src = r#"[tool.uv]
resolution = 42
"#;
        let err = parse_uv_config(src).unwrap_err();
        assert!(matches!(
            err,
            UvConfigError::WrongType { field } if field == "resolution"
        ));
    }

    #[test]
    fn ignores_unrelated_tool_tables() {
        let src = r#"[tool.black]
line-length = 100

[tool.uv]
resolution = "highest"
"#;
        let cfg = parse_uv_config(src).unwrap();
        assert_eq!(cfg.resolution, Some(ResolutionMode::Highest));
    }

    #[test]
    fn partial_table_leaves_other_fields_none() {
        let src = r#"[tool.uv]
link-mode = "symlink"
"#;
        let cfg = parse_uv_config(src).unwrap();
        assert_eq!(cfg.link_mode, Some(LinkMode::Symlink));
        assert_eq!(cfg.resolution, None);
        assert_eq!(cfg.prerelease, None);
        assert_eq!(cfg.python_preference, None);
        assert!(cfg.constraint_dependencies.is_empty());
        assert!(cfg.override_dependencies.is_empty());
    }

    #[test]
    fn empty_uv_table_yields_default() {
        let src = "[tool.uv]\n";
        let cfg = parse_uv_config(src).unwrap();
        assert_eq!(cfg, UvConfig::default());
    }

    #[test]
    fn parses_combined_config() {
        let src = r#"[tool.uv]
resolution = "lowest-direct"
prerelease = "if-necessary"
link-mode = "hardlink"
python-preference = "only-managed"
constraint-dependencies = ["a>=1"]
override-dependencies = ["b<2"]
compile-bytecode = true
no-build = true
no-binary = false
managed = true
"#;
        let cfg = parse_uv_config(src).unwrap();
        assert_eq!(cfg.resolution, Some(ResolutionMode::LowestDirect));
        assert_eq!(cfg.prerelease, Some(PrereleaseMode::IfNecessary));
        assert_eq!(cfg.link_mode, Some(LinkMode::Hardlink));
        assert_eq!(cfg.python_preference, Some(PythonPreference::OnlyManaged));
        assert_eq!(cfg.constraint_dependencies, vec!["a>=1"]);
        assert_eq!(cfg.override_dependencies, vec!["b<2"]);
        assert_eq!(cfg.compile_bytecode, Some(true));
        assert_eq!(cfg.no_build, Some(true));
        assert_eq!(cfg.no_binary, Some(false));
        assert_eq!(cfg.managed, Some(true));
    }
}
