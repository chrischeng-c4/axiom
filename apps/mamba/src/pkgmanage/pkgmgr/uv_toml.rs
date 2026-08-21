// Standalone `uv.toml` reader (Tick 63).
//
// uv accepts a flat `uv.toml` as an alternative (or supplement) to
// `[tool.uv]` in `pyproject.toml`. Same knobs, but the table sits at
// the document root with no `tool.uv` prefix. uv merges the two with
// `uv.toml` taking precedence — this module only does the parsing
// half; merging is a future caller-side concern.
//
// Resolution order at the OS level (consumed by Tick 62 `uv_dirs`):
//   1. Path passed via `--config-file`
//   2. `<cwd>/uv.toml`
//   3. `<workspace-root>/uv.toml`
//   4. `<user-config-dir>/uv/uv.toml`
//
// This module owns parsing only; locating the file is the caller's job.
//
// The `UvConfig` struct + enums + helpers come from `uv_config`. We
// reuse the table walker (`parse_uv_table`) so the two readers stay in
// lock-step — any new knob added there automatically lands here too.
//
// Error type is local because `UvConfigError`'s messages mention
// `[tool.uv].<field>` which is wrong for a flat `uv.toml`. `UvTomlError`
// converts each variant into a `uv.toml: <field>` form for clarity.

use crate::pkgmanage::pkgmgr::uv_config::{parse_uv_table, UvConfig, UvConfigError};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UvTomlError {
    InvalidToml(String),
    WrongType { field: String },
    UnknownValue { field: String, value: String },
    RootIsNotTable,
}

impl std::fmt::Display for UvTomlError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UvTomlError::InvalidToml(s) => write!(f, "invalid uv.toml: {s}"),
            UvTomlError::WrongType { field } => write!(f, "uv.toml: {field}: wrong type"),
            UvTomlError::UnknownValue { field, value } => {
                write!(f, "uv.toml: {field}: unknown value {value:?}")
            }
            UvTomlError::RootIsNotTable => write!(f, "uv.toml: root must be a TOML table"),
        }
    }
}

impl std::error::Error for UvTomlError {}

impl From<UvConfigError> for UvTomlError {
    fn from(err: UvConfigError) -> Self {
        match err {
            UvConfigError::InvalidToml(s) => UvTomlError::InvalidToml(s),
            UvConfigError::WrongType { field } => UvTomlError::WrongType { field },
            UvConfigError::UnknownValue { field, value } => {
                UvTomlError::UnknownValue { field, value }
            }
        }
    }
}

/// Parse a standalone `uv.toml`. Empty input yields the default
/// (all-None) config; a syntactically valid but field-less file is
/// equivalent. Unknown top-level keys are silently ignored so that
/// future uv versions don't break our reader — the same forgiveness
/// `parse_uv_config` already exercises for unrelated `[project]` keys.
pub fn parse_uv_toml(text: &str) -> Result<UvConfig, UvTomlError> {
    let doc: toml::Value = text
        .parse()
        .map_err(|e: toml::de::Error| UvTomlError::InvalidToml(e.to_string()))?;

    let table = doc.as_table().ok_or(UvTomlError::RootIsNotTable)?;

    parse_uv_table(table).map_err(UvTomlError::from)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pkgmanage::pkgmgr::uv_config::{
        LinkMode, PrereleaseMode, PythonPreference, ResolutionMode,
    };

    #[test]
    fn empty_input_yields_default() {
        let cfg = parse_uv_toml("").unwrap();
        assert_eq!(cfg, UvConfig::default());
    }

    #[test]
    fn whitespace_only_input_yields_default() {
        let cfg = parse_uv_toml("\n\n   \n").unwrap();
        assert_eq!(cfg, UvConfig::default());
    }

    #[test]
    fn invalid_toml_is_an_error() {
        let err = parse_uv_toml("not = = valid").unwrap_err();
        assert!(matches!(err, UvTomlError::InvalidToml(_)));
    }

    #[test]
    fn flat_resolution_field() {
        let cfg = parse_uv_toml("resolution = \"lowest-direct\"\n").unwrap();
        assert_eq!(cfg.resolution, Some(ResolutionMode::LowestDirect));
    }

    #[test]
    fn flat_prerelease_field() {
        let cfg = parse_uv_toml("prerelease = \"if-necessary-or-explicit\"\n").unwrap();
        assert_eq!(cfg.prerelease, Some(PrereleaseMode::IfNecessaryOrExplicit));
    }

    #[test]
    fn flat_link_mode_field() {
        let cfg = parse_uv_toml("link-mode = \"hardlink\"\n").unwrap();
        assert_eq!(cfg.link_mode, Some(LinkMode::Hardlink));
    }

    #[test]
    fn flat_python_preference_field() {
        let cfg = parse_uv_toml("python-preference = \"only-system\"\n").unwrap();
        assert_eq!(cfg.python_preference, Some(PythonPreference::OnlySystem));
    }

    #[test]
    fn all_known_knobs_set_at_once() {
        let src = r#"
resolution = "highest"
prerelease = "allow"
link-mode = "copy"
python-preference = "managed"
constraint-dependencies = ["pkg-a==1.0"]
override-dependencies = ["pkg-b<2"]
compile-bytecode = true
no-build = false
no-binary = true
managed = false
"#;
        let cfg = parse_uv_toml(src).unwrap();
        assert_eq!(cfg.resolution, Some(ResolutionMode::Highest));
        assert_eq!(cfg.prerelease, Some(PrereleaseMode::Allow));
        assert_eq!(cfg.link_mode, Some(LinkMode::Copy));
        assert_eq!(cfg.python_preference, Some(PythonPreference::Managed));
        assert_eq!(cfg.constraint_dependencies, vec!["pkg-a==1.0"]);
        assert_eq!(cfg.override_dependencies, vec!["pkg-b<2"]);
        assert_eq!(cfg.compile_bytecode, Some(true));
        assert_eq!(cfg.no_build, Some(false));
        assert_eq!(cfg.no_binary, Some(true));
        assert_eq!(cfg.managed, Some(false));
    }

    #[test]
    fn rejects_unknown_resolution_value() {
        let err = parse_uv_toml("resolution = \"sideways\"\n").unwrap_err();
        match err {
            UvTomlError::UnknownValue { field, value } => {
                assert_eq!(field, "resolution");
                assert_eq!(value, "sideways");
            }
            other => panic!("expected UnknownValue, got {other:?}"),
        }
    }

    #[test]
    fn rejects_wrong_type_on_bool_field() {
        let err = parse_uv_toml("compile-bytecode = \"yes\"\n").unwrap_err();
        match err {
            UvTomlError::WrongType { field } => assert_eq!(field, "compile-bytecode"),
            other => panic!("expected WrongType, got {other:?}"),
        }
    }

    #[test]
    fn rejects_wrong_type_on_string_array_field() {
        let err = parse_uv_toml("constraint-dependencies = 42\n").unwrap_err();
        match err {
            UvTomlError::WrongType { field } => assert_eq!(field, "constraint-dependencies"),
            other => panic!("expected WrongType, got {other:?}"),
        }
    }

    #[test]
    fn rejects_non_string_inside_string_array() {
        let err = parse_uv_toml("constraint-dependencies = [\"ok\", 7]\n").unwrap_err();
        match err {
            UvTomlError::WrongType { field } => {
                assert_eq!(field, "constraint-dependencies[1]");
            }
            other => panic!("expected WrongType, got {other:?}"),
        }
    }

    #[test]
    fn unknown_top_level_keys_are_ignored() {
        // Forward-compat: future uv knobs should not break our reader.
        let src = r#"
resolution = "lowest"
future-knob = "value"
nested = { also = "tolerated" }
"#;
        let cfg = parse_uv_toml(src).unwrap();
        assert_eq!(cfg.resolution, Some(ResolutionMode::Lowest));
    }

    #[test]
    fn display_includes_uv_toml_prefix() {
        let err = UvTomlError::WrongType {
            field: "managed".into(),
        };
        let msg = err.to_string();
        assert!(msg.contains("uv.toml"));
        assert!(msg.contains("managed"));
        assert!(msg.contains("wrong type"));
    }

    #[test]
    fn display_invalid_toml_form() {
        let err = UvTomlError::InvalidToml("expected `=`".into());
        let msg = err.to_string();
        assert!(msg.contains("invalid uv.toml"));
        assert!(msg.contains("expected `=`"));
    }

    #[test]
    fn display_unknown_value_form() {
        let err = UvTomlError::UnknownValue {
            field: "resolution".into(),
            value: "sideways".into(),
        };
        let msg = err.to_string();
        assert!(msg.contains("uv.toml"));
        assert!(msg.contains("resolution"));
        assert!(msg.contains("sideways"));
    }

    #[test]
    fn empty_string_arrays_parse_as_empty_vecs() {
        let src = r#"
constraint-dependencies = []
override-dependencies = []
"#;
        let cfg = parse_uv_toml(src).unwrap();
        assert!(cfg.constraint_dependencies.is_empty());
        assert!(cfg.override_dependencies.is_empty());
    }

    #[test]
    fn from_uv_config_error_invalid_toml_maps_correctly() {
        let from: UvTomlError = UvConfigError::InvalidToml("x".into()).into();
        assert!(matches!(from, UvTomlError::InvalidToml(s) if s == "x"));
    }

    #[test]
    fn from_uv_config_error_wrong_type_maps_correctly() {
        let from: UvTomlError = UvConfigError::WrongType {
            field: "managed".into(),
        }
        .into();
        assert!(matches!(from, UvTomlError::WrongType { field } if field == "managed"));
    }

    #[test]
    fn from_uv_config_error_unknown_value_maps_correctly() {
        let from: UvTomlError = UvConfigError::UnknownValue {
            field: "resolution".into(),
            value: "sideways".into(),
        }
        .into();
        match from {
            UvTomlError::UnknownValue { field, value } => {
                assert_eq!(field, "resolution");
                assert_eq!(value, "sideways");
            }
            other => panic!("expected UnknownValue, got {other:?}"),
        }
    }

    #[test]
    fn round_trip_matches_pyproject_form() {
        // The two readers must agree on the same knob set.
        let flat = parse_uv_toml(
            "resolution = \"highest\"\nlink-mode = \"clone\"\ncompile-bytecode = true\n",
        )
        .unwrap();
        let nested = crate::pkgmanage::pkgmgr::uv_config::parse_uv_config(
            "[tool.uv]\nresolution = \"highest\"\nlink-mode = \"clone\"\ncompile-bytecode = true\n",
        )
        .unwrap();
        assert_eq!(flat, nested);
    }
}
