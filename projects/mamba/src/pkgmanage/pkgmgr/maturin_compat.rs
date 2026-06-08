// Maturin `[tool.maturin]` pyproject.toml reader (Tick 130).
//
// Maturin is the de-facto PEP 517 build backend for Rust-Python
// extension modules (the PyO3 ecosystem). Its configuration lives
// under the `[tool.maturin]` table in pyproject.toml and tells the
// backend how to compile and lay out the wheel:
//
//   [tool.maturin]
//   bindings        = "pyo3" | "cffi" | "uniffi" | "bin"
//   module-name     = "package._native"     (rename for the cdylib)
//   manifest-path   = "rust/Cargo.toml"      (path to Cargo.toml)
//   python-source   = "python"                (pure-Python src dir)
//   features        = ["abi3-py38"]
//   strip           = true / false
//   skip-auditwheel = true / false
//   include         = ["python/*.pyi"]
//   exclude         = ["tests/**"]
//
// uv reads these fields when classifying a project's build invocation
// — e.g. to know whether the project is a Rust extension (and thus
// whether the wheel build will need a Rust toolchain), and to surface
// `tool.maturin.features` so a user can pass them through to a
// `mamba lock --build-feature` flag.
//
// This module is a pure parser: it returns a typed view of the
// `[tool.maturin]` table. Toolchain invocation belongs in
// `wheel_build.rs` / `pep517.rs`.

use crate::pkgmanage::pkgmgr::types::IndexError;
use toml::Value;

const DETAIL: &str = "<pyproject.toml [tool.maturin]>";

/// Recognized binding flavours. Anything else is preserved verbatim
/// under `Other(_)` so we don't lock out future maturin extensions.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MaturinBindings {
    PyO3,
    Cffi,
    UniFfi,
    /// Standalone executable (no Python module).
    Bin,
    Other(String),
}

impl MaturinBindings {
    pub fn from_str(s: &str) -> Self {
        match s.to_ascii_lowercase().as_str() {
            "pyo3" => MaturinBindings::PyO3,
            "cffi" => MaturinBindings::Cffi,
            "uniffi" => MaturinBindings::UniFfi,
            "bin" => MaturinBindings::Bin,
            _ => MaturinBindings::Other(s.to_string()),
        }
    }
}

/// Typed view of `[tool.maturin]`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct MaturinConfig {
    pub bindings: Option<MaturinBindings>,
    pub module_name: Option<String>,
    pub manifest_path: Option<String>,
    pub python_source: Option<String>,
    pub features: Vec<String>,
    pub strip: Option<bool>,
    pub skip_auditwheel: Option<bool>,
    pub include: Vec<String>,
    pub exclude: Vec<String>,
}

impl MaturinConfig {
    /// True when none of the recognized keys were present.
    pub fn is_empty(&self) -> bool {
        self.bindings.is_none()
            && self.module_name.is_none()
            && self.manifest_path.is_none()
            && self.python_source.is_none()
            && self.features.is_empty()
            && self.strip.is_none()
            && self.skip_auditwheel.is_none()
            && self.include.is_empty()
            && self.exclude.is_empty()
    }

    /// True when this config describes a project that produces a
    /// Python extension module (anything except `bindings = "bin"`).
    pub fn is_python_extension(&self) -> bool {
        match &self.bindings {
            Some(MaturinBindings::Bin) => false,
            _ => true,
        }
    }
}

/// Parse the entire pyproject.toml text and extract `[tool.maturin]`.
/// Returns `Ok(None)` when the file has no `[tool.maturin]` table —
/// that means the project is not using maturin and the caller should
/// fall back to PEP 517 generic handling.
pub fn parse_maturin_config(pyproject_src: &str) -> Result<Option<MaturinConfig>, IndexError> {
    let doc: Value = toml::from_str(pyproject_src).map_err(|e| IndexError::ParseError {
        url: DETAIL.into(),
        detail: format!("pyproject.toml parse error: {e}"),
    })?;
    let table = match doc
        .get("tool")
        .and_then(|t| t.as_table())
        .and_then(|t| t.get("maturin"))
        .and_then(|m| m.as_table())
    {
        Some(t) => t,
        None => return Ok(None),
    };

    let mut out = MaturinConfig::default();

    if let Some(b) = table.get("bindings").and_then(|v| v.as_str()) {
        out.bindings = Some(MaturinBindings::from_str(b));
    }
    if let Some(m) = table.get("module-name").and_then(|v| v.as_str()) {
        out.module_name = Some(m.to_string());
    }
    if let Some(p) = table.get("manifest-path").and_then(|v| v.as_str()) {
        out.manifest_path = Some(p.to_string());
    }
    if let Some(p) = table.get("python-source").and_then(|v| v.as_str()) {
        out.python_source = Some(p.to_string());
    }
    if let Some(arr) = table.get("features").and_then(|v| v.as_array()) {
        out.features = string_array(arr, "features")?;
    }
    if let Some(b) = table.get("strip").and_then(|v| v.as_bool()) {
        out.strip = Some(b);
    }
    if let Some(b) = table.get("skip-auditwheel").and_then(|v| v.as_bool()) {
        out.skip_auditwheel = Some(b);
    }
    if let Some(arr) = table.get("include").and_then(|v| v.as_array()) {
        out.include = string_array(arr, "include")?;
    }
    if let Some(arr) = table.get("exclude").and_then(|v| v.as_array()) {
        out.exclude = string_array(arr, "exclude")?;
    }

    Ok(Some(out))
}

fn string_array(arr: &[Value], field: &str) -> Result<Vec<String>, IndexError> {
    let mut out = Vec::with_capacity(arr.len());
    for v in arr {
        match v.as_str() {
            Some(s) => out.push(s.to_string()),
            None => {
                return Err(IndexError::ParseError {
                    url: DETAIL.into(),
                    detail: format!("[tool.maturin].{field} entries must be strings"),
                })
            }
        }
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pyproject_without_maturin_table_returns_none() {
        let src = r#"
[project]
name = "foo"
version = "1.0"
"#;
        let r = parse_maturin_config(src).unwrap();
        assert!(r.is_none());
    }

    #[test]
    fn empty_maturin_table_returns_empty_config() {
        let src = r#"
[tool.maturin]
"#;
        let r = parse_maturin_config(src).unwrap().unwrap();
        assert!(r.is_empty());
    }

    #[test]
    fn parses_pyo3_extension_config() {
        let src = r#"
[tool.maturin]
bindings = "pyo3"
module-name = "foo._native"
manifest-path = "rust/Cargo.toml"
python-source = "python"
features = ["abi3-py38", "extension-module"]
strip = true
"#;
        let r = parse_maturin_config(src).unwrap().unwrap();
        assert_eq!(r.bindings, Some(MaturinBindings::PyO3));
        assert_eq!(r.module_name.as_deref(), Some("foo._native"));
        assert_eq!(r.manifest_path.as_deref(), Some("rust/Cargo.toml"));
        assert_eq!(r.python_source.as_deref(), Some("python"));
        assert_eq!(r.features, vec!["abi3-py38", "extension-module"]);
        assert_eq!(r.strip, Some(true));
        assert!(r.is_python_extension());
    }

    #[test]
    fn parses_cffi_and_uniffi_bindings() {
        let cffi = parse_maturin_config("[tool.maturin]\nbindings = \"cffi\"")
            .unwrap()
            .unwrap();
        let uniffi = parse_maturin_config("[tool.maturin]\nbindings = \"uniffi\"")
            .unwrap()
            .unwrap();
        assert_eq!(cffi.bindings, Some(MaturinBindings::Cffi));
        assert_eq!(uniffi.bindings, Some(MaturinBindings::UniFfi));
        assert!(cffi.is_python_extension());
        assert!(uniffi.is_python_extension());
    }

    #[test]
    fn bin_bindings_not_classified_as_extension() {
        let src = "[tool.maturin]\nbindings = \"bin\"";
        let r = parse_maturin_config(src).unwrap().unwrap();
        assert_eq!(r.bindings, Some(MaturinBindings::Bin));
        assert!(!r.is_python_extension());
    }

    #[test]
    fn unknown_binding_value_preserved_verbatim() {
        let src = "[tool.maturin]\nbindings = \"future-bindings-2030\"";
        let r = parse_maturin_config(src).unwrap().unwrap();
        match r.bindings {
            Some(MaturinBindings::Other(s)) => assert_eq!(s, "future-bindings-2030"),
            other => panic!("unexpected bindings: {other:?}"),
        }
    }

    #[test]
    fn binding_value_is_case_insensitive() {
        let src = "[tool.maturin]\nbindings = \"PyO3\"";
        let r = parse_maturin_config(src).unwrap().unwrap();
        assert_eq!(r.bindings, Some(MaturinBindings::PyO3));
    }

    #[test]
    fn include_and_exclude_arrays_round_trip() {
        let src = r#"
[tool.maturin]
include = ["python/*.pyi", "py.typed"]
exclude = ["tests/**", "docs/**"]
"#;
        let r = parse_maturin_config(src).unwrap().unwrap();
        assert_eq!(r.include, vec!["python/*.pyi", "py.typed"]);
        assert_eq!(r.exclude, vec!["tests/**", "docs/**"]);
    }

    #[test]
    fn non_string_in_features_is_rejected() {
        let src = "[tool.maturin]\nfeatures = [\"ok\", 42]";
        let err = parse_maturin_config(src).unwrap_err();
        let msg = format!("{err:?}");
        assert!(msg.contains("features"));
    }

    #[test]
    fn boolean_fields_round_trip() {
        let src = "[tool.maturin]\nstrip = false\nskip-auditwheel = true";
        let r = parse_maturin_config(src).unwrap().unwrap();
        assert_eq!(r.strip, Some(false));
        assert_eq!(r.skip_auditwheel, Some(true));
    }

    #[test]
    fn realistic_pyo3_pyproject_corpus() {
        // Pattern lifted from a real pyo3-based pyproject.toml
        // (cryptography-style structured Rust package).
        let src = r#"
[build-system]
requires = ["maturin>=1.5"]
build-backend = "maturin"

[project]
name = "demo_native"
version = "0.1.0"

[tool.maturin]
bindings = "pyo3"
module-name = "demo_native._core"
manifest-path = "Cargo.toml"
python-source = "python"
features = ["pyo3/extension-module", "pyo3/abi3-py39"]
strip = true
include = ["py.typed"]
"#;
        let r = parse_maturin_config(src).unwrap().unwrap();
        assert_eq!(r.bindings, Some(MaturinBindings::PyO3));
        assert_eq!(r.module_name.as_deref(), Some("demo_native._core"));
        assert_eq!(r.python_source.as_deref(), Some("python"));
        assert_eq!(r.features, vec!["pyo3/extension-module", "pyo3/abi3-py39"]);
        assert_eq!(r.strip, Some(true));
        assert_eq!(r.include, vec!["py.typed"]);
        assert!(r.is_python_extension());
    }

    #[test]
    fn parse_error_surfaces_location() {
        let err = parse_maturin_config("this is not valid toml = =").unwrap_err();
        let msg = format!("{err:?}");
        assert!(msg.contains("pyproject.toml parse error"));
    }
}
