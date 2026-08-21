use serde::Deserialize;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::error::MambaError;

/// Top-level mamba.toml configuration (#250, #251, #1134).
///
/// Supports two formats:
///
/// **Richer format** (project mode):
/// ```toml
/// [project]
/// name = "my-app"
/// version = "0.1.0"
///
/// [crates.pg]
/// path = "../cclab-pg-mamba"
/// expose = ["query", "execute"]
/// module = "mambalibs.pg"
/// ```
///
/// **Flat format** (backward compatible):
/// ```toml
/// entry_point = "src/main.py"
/// [crates]
/// cclab-schema-mamba = "0.1.0"
/// [expose]
/// cclab-schema-mamba = ["BaseModel", "Field"]
/// ```
#[derive(Debug, Clone, Deserialize)]
pub struct MambaConfig {
    /// Project metadata.  Optional for flat-format configs that omit `[project]`.
    #[serde(default)]
    pub project: ProjectConfig,

    /// Top-level entry point (flat format).
    /// In the richer format, use `project.entry_point` instead.
    #[serde(default)]
    pub entry_point: Option<String>,

    /// Native Rust crates exposed to Mamba scripts as modules (#1014).
    ///
    /// Accepts either:
    /// - A plain string value (interpreted as a version constraint), or
    /// - A full `CrateConfig` table with path/version/expose/module fields.
    #[serde(default)]
    pub crates: HashMap<String, CrateEntry>,

    /// Per-crate symbol expose lists (flat format).
    ///
    /// In the richer format, expose lists live inside each `CrateConfig`.
    /// This top-level field provides backward compatibility with the flat format.
    #[serde(default)]
    pub expose: HashMap<String, Vec<String>>,

    #[serde(default)]
    pub build: BuildConfig,

    /// Search paths for multi-package resolution (#1014).
    #[serde(default)]
    pub paths: PathsConfig,
}

/// Project metadata.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct ProjectConfig {
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub version: String,
    /// Entry-point source file (relative to the config file's directory).
    /// Used by the compiler driver and CLI to locate the main script.
    #[serde(default)]
    pub entry_point: Option<String>,
}

/// A crate dependency that accepts either a plain version string or a full
/// config table.
///
/// ```toml
/// # Plain string (flat format):
/// cclab-schema-mamba = "0.1.0"
///
/// # Full table (richer format):
/// [crates.pg]
/// path = "../cclab-pg-mamba"
/// expose = ["query", "execute"]
/// ```
#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum CrateEntry {
    /// A plain version string, e.g. `cclab-schema-mamba = "0.1.0"`.
    Version(String),
    /// A full crate config table.
    Config(CrateConfig),
}

impl CrateEntry {
    /// Convert to a `CrateConfig`, filling in the version for plain strings.
    pub fn to_crate_config(&self) -> CrateConfig {
        match self {
            CrateEntry::Version(v) => CrateConfig {
                crate_name: None,
                version: Some(v.clone()),
                path: None,
                expose: Vec::new(),
                module: None,
            },
            CrateEntry::Config(c) => c.clone(),
        }
    }

    /// Returns the expose list for this entry.
    pub fn expose_list(&self) -> &[String] {
        match self {
            CrateEntry::Version(_) => &[],
            CrateEntry::Config(c) => &c.expose,
        }
    }
}

/// Native crate dependency specification (#1014).
///
/// Each entry in `[crates]` declares a Rust crate whose symbols are exposed
/// to Mamba scripts via the `MAMBA_MODULES` registry.
///
/// Example `mamba.toml`:
/// ```toml
/// [crates.pg]
/// path = "../cclab-pg-mamba"
/// expose = ["query", "execute", "connect"]
/// module = "mambalibs.pg"
/// ```
#[derive(Debug, Clone, Deserialize)]
pub struct CrateConfig {
    /// Rust crate name.  Defaults to the TOML table key if omitted.
    #[serde(rename = "crate")]
    pub crate_name: Option<String>,
    /// Registry version constraint (semver range).  Required unless `path` is set.
    pub version: Option<String>,
    /// Local path to the crate source.  Required unless `version` is set.
    pub path: Option<PathBuf>,
    /// Whitelist of symbols exported to Mamba scripts.  Must be non-empty.
    #[serde(default)]
    pub expose: Vec<String>,
    /// Mamba import path prefix under which the symbols are exposed.
    /// Defaults to the TOML table key if omitted.
    pub module: Option<String>,
}

impl CrateConfig {
    /// Effective crate name: `crate_name` override or the map key.
    pub fn effective_crate_name<'a>(&'a self, key: &'a str) -> &'a str {
        self.crate_name.as_deref().unwrap_or(key)
    }

    /// Effective module import path: `module` override or the map key.
    pub fn effective_module<'a>(&'a self, key: &'a str) -> &'a str {
        self.module.as_deref().unwrap_or(key)
    }
}

/// Path search configuration (#1014).
#[derive(Debug, Clone, Default, Deserialize)]
pub struct PathsConfig {
    /// Additional directories searched for Mamba packages and modules.
    #[serde(default)]
    pub search: Vec<PathBuf>,
}

/// Build settings.
#[derive(Debug, Clone, Deserialize)]
pub struct BuildConfig {
    #[serde(default = "default_target")]
    pub target: String,
    #[serde(default = "default_opt_level")]
    pub opt_level: u8,
}

fn default_target() -> String {
    "native".to_string()
}
fn default_opt_level() -> u8 {
    2
}

impl Default for BuildConfig {
    fn default() -> Self {
        Self {
            target: default_target(),
            opt_level: default_opt_level(),
        }
    }
}

impl MambaConfig {
    /// Walk up from `start_dir` looking for `mamba.toml`.
    ///
    /// Returns `(config, config_path)` for the first file found, or `None` if
    /// the filesystem root is reached without finding one.
    // @spec .aw/changes/1134-mamba-dual-config/groups/unify-mamba-config/specs/1134-mamba-dual-config-spec.md#R1
    // @spec .aw/changes/1134-mamba-dual-config/groups/unify-mamba-config/specs/1134-mamba-dual-config-spec.md#R3
    pub fn discover(start_dir: &Path) -> Option<(Self, PathBuf)> {
        let mut dir = start_dir;
        loop {
            let candidate = dir.join("mamba.toml");
            if candidate.exists() {
                let text = std::fs::read_to_string(&candidate).ok()?;
                let cfg: MambaConfig = toml::from_str(&text).ok()?;
                // Skip validation for discover -- the file may be a minimal
                // flat-format config without a [project] table.
                return Some((cfg, candidate));
            }
            dir = dir.parent()?;
        }
    }

    /// Parse a mamba.toml file (#251).
    // @spec .aw/changes/1134-mamba-dual-config/groups/unify-mamba-config/specs/1134-mamba-dual-config-spec.md#R2
    // @spec .aw/changes/1134-mamba-dual-config/groups/unify-mamba-config/specs/1134-mamba-dual-config-spec.md#R3
    pub fn from_file(path: &Path) -> crate::error::Result<Self> {
        let content = std::fs::read_to_string(path)?;
        Self::from_str(&content)
    }

    /// Parse from a TOML string.
    // @spec .aw/changes/1134-mamba-dual-config/groups/unify-mamba-config/specs/1134-mamba-dual-config-spec.md#R2
    // @spec .aw/changes/1134-mamba-dual-config/groups/unify-mamba-config/specs/1134-mamba-dual-config-spec.md#R3
    pub fn from_str(content: &str) -> crate::error::Result<Self> {
        let config: Self = toml::from_str(content)
            .map_err(|e| MambaError::Other(format!("config parse error: {e}")))?;
        config.validate()?;
        Ok(config)
    }

    /// Entry-point source file, if set.
    ///
    /// Checks the top-level `entry_point` field first (flat format),
    /// then falls back to `project.entry_point` (richer format).
    // @spec .aw/changes/1134-mamba-dual-config/groups/unify-mamba-config/specs/1134-mamba-dual-config-spec.md#R3
    pub fn entry_point(&self) -> Option<&str> {
        self.entry_point
            .as_deref()
            .or(self.project.entry_point.as_deref())
    }

    /// Returns `true` if `symbol` from `crate_name` may be imported.
    ///
    /// Checks both the top-level `[expose]` table (flat format) and
    /// per-crate `expose` lists (richer format).
    ///
    /// An absent or empty expose list means "expose everything".
    // @spec .aw/changes/1134-mamba-dual-config/groups/unify-mamba-config/specs/1134-mamba-dual-config-spec.md#R2
    // @spec .aw/changes/1134-mamba-dual-config/groups/unify-mamba-config/specs/1134-mamba-dual-config-spec.md#R3
    pub fn is_symbol_exposed(&self, crate_name: &str, symbol: &str) -> bool {
        // Check top-level expose map first (flat format).
        if let Some(list) = self.expose.get(crate_name) {
            if !list.is_empty() {
                return list.iter().any(|s| s == symbol);
            }
            // Empty list means expose everything.
            return true;
        }

        // Check per-crate expose list (richer format).
        if let Some(entry) = self.crates.get(crate_name) {
            let expose = entry.expose_list();
            if !expose.is_empty() {
                return expose.iter().any(|s| s == symbol);
            }
        }

        // No expose restrictions found -- allow everything.
        true
    }

    /// Check whether expose filtering should be applied for the given key.
    ///
    /// Returns `true` if the key has a non-empty expose list in either
    /// the top-level `[expose]` table or a `CrateEntry::Config` with
    /// a non-empty expose list.
    // @spec .aw/changes/1134-mamba-dual-config/groups/unify-mamba-config/specs/1134-mamba-dual-config-spec.md#R2
    // @spec .aw/changes/1134-mamba-dual-config/groups/unify-mamba-config/specs/1134-mamba-dual-config-spec.md#R3
    pub fn has_crate_expose(&self, key: &str) -> bool {
        // Top-level expose map (flat format).
        if self.expose.contains_key(key) {
            return true;
        }
        // Per-crate expose list (richer format) -- only Config entries
        // with non-empty expose lists count.
        if let Some(entry) = self.crates.get(key) {
            if !entry.expose_list().is_empty() {
                return true;
            }
        }
        false
    }

    /// Returns `true` if a `[project]` table with non-empty name was provided.
    fn has_project(&self) -> bool {
        !self.project.name.is_empty()
    }

    /// Validate config fields (#251, #270, #1014, #1134).
    ///
    /// Validation is lenient for the flat format: `[project]` is optional,
    /// and plain-string crate entries skip the expose/version-or-path checks.
    // @spec .aw/changes/1134-mamba-dual-config/groups/unify-mamba-config/specs/1134-mamba-dual-config-spec.md#R5
    pub fn validate(&self) -> crate::error::Result<()> {
        // Validate project metadata only when a [project] table was provided.
        if self.has_project() {
            // Validate semver (#270)
            if semver::Version::parse(&self.project.version).is_err() {
                return Err(MambaError::Other(format!(
                    "project.version `{}` is not valid semver",
                    self.project.version
                )));
            }
        }

        // Validate build target
        match self.build.target.as_str() {
            "native" | "wasm32" => {}
            other => {
                return Err(MambaError::Other(format!(
                    "build.target `{other}` is invalid; expected \"native\" or \"wasm32\""
                )));
            }
        }

        // Validate opt_level
        if self.build.opt_level > 3 {
            return Err(MambaError::Other(format!(
                "build.opt_level {} is out of range (0-3)",
                self.build.opt_level
            )));
        }

        // Validate rich crate configs (skip validation for plain version strings).
        for (name, entry) in &self.crates {
            if let CrateEntry::Config(cr) = entry {
                if cr.expose.is_empty() {
                    return Err(MambaError::Other(format!(
                        "crate `{name}`: expose list must not be empty"
                    )));
                }
                if cr.version.is_none() && cr.path.is_none() {
                    return Err(MambaError::Other(format!(
                        "crate `{name}`: must specify either `version` or `path`"
                    )));
                }
                // Validate dep version semver if present (#270)
                if let Some(ver) = &cr.version {
                    if semver::VersionReq::parse(ver).is_err() {
                        return Err(MambaError::Other(format!(
                            "crate `{name}`: version `{ver}` is not valid semver"
                        )));
                    }
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── Richer format (project mode) tests ──────────────────────────────────

    #[test]
    fn test_basic_config_with_crates() {
        let toml = r#"
[project]
name = "my-app"
version = "0.1.0"

[crates.fast_math]
crate = "fast-math"
version = "1.2"
expose = ["sqrt", "sin", "cos"]
module = "math_ext"

[build]
target = "native"
opt_level = 2
"#;
        let config = MambaConfig::from_str(toml).unwrap();
        assert_eq!(config.project.name, "my-app");
        assert_eq!(config.project.version, "0.1.0");
        assert_eq!(config.crates.len(), 1);
        assert_eq!(config.build.opt_level, 2);
        let cr = match &config.crates["fast_math"] {
            CrateEntry::Config(c) => c,
            _ => panic!("expected CrateEntry::Config"),
        };
        assert_eq!(cr.effective_crate_name("fast_math"), "fast-math");
        assert_eq!(cr.effective_module("fast_math"), "math_ext");
    }

    #[test]
    fn test_crate_defaults_name_from_key() {
        let toml = r#"
[project]
name = "my-app"
version = "0.1.0"

[crates.pg]
path = "../cclab-pg-mamba"
expose = ["connect", "query"]
"#;
        let config = MambaConfig::from_str(toml).unwrap();
        let cr = match &config.crates["pg"] {
            CrateEntry::Config(c) => c,
            _ => panic!("expected CrateEntry::Config"),
        };
        assert_eq!(cr.effective_crate_name("pg"), "pg");
        assert_eq!(cr.effective_module("pg"), "pg");
    }

    #[test]
    fn test_paths_config() {
        let toml = r#"
[project]
name = "my-app"
version = "0.1.0"

[paths]
search = ["./packages", "../shared"]
"#;
        let config = MambaConfig::from_str(toml).unwrap();
        assert_eq!(config.paths.search.len(), 2);
    }

    #[test]
    fn test_minimal_config() {
        let toml = r#"
[project]
name = "hello"
version = "1.0.0"
"#;
        let config = MambaConfig::from_str(toml).unwrap();
        assert_eq!(config.build.target, "native");
        assert_eq!(config.build.opt_level, 2);
        assert!(config.crates.is_empty());
        assert!(config.paths.search.is_empty());
    }

    #[test]
    fn test_invalid_semver() {
        let toml = r#"
[project]
name = "bad"
version = "not-a-version"
"#;
        let result = MambaConfig::from_str(toml);
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_target() {
        let toml = r#"
[project]
name = "bad"
version = "1.0.0"

[build]
target = "arm64"
"#;
        let result = MambaConfig::from_str(toml);
        assert!(result.is_err());
    }

    #[test]
    fn test_empty_expose() {
        let toml = r#"
[project]
name = "bad"
version = "1.0.0"

[crates.foo]
crate = "foo"
version = "1.0"
expose = []
"#;
        let result = MambaConfig::from_str(toml);
        assert!(result.is_err());
    }

    #[test]
    fn test_local_path_crate() {
        let toml = r#"
[project]
name = "local"
version = "0.1.0"

[crates.mylib]
crate = "mylib"
path = "../mylib"
expose = ["helper"]
"#;
        let config = MambaConfig::from_str(toml).unwrap();
        match &config.crates["mylib"] {
            CrateEntry::Config(c) => assert!(c.path.is_some()),
            _ => panic!("expected CrateEntry::Config"),
        }
    }

    #[test]
    fn test_missing_version_and_path() {
        let toml = r#"
[project]
name = "bad"
version = "1.0.0"

[crates.foo]
expose = ["func"]
"#;
        let result = MambaConfig::from_str(toml);
        assert!(result.is_err());
    }

    // ── Flat format (backward compat #1134) ──────────────────────────────────

    #[test]
    fn flat_format_entry_point_and_crates() {
        let toml = r#"
entry_point = "src/main.py"
[crates]
cclab-schema-mamba = "0.1.0"
"#;
        let config = MambaConfig::from_str(toml).unwrap();
        assert_eq!(config.entry_point(), Some("src/main.py"));
        assert!(config.crates.contains_key("cclab-schema-mamba"));
        match &config.crates["cclab-schema-mamba"] {
            CrateEntry::Version(v) => assert_eq!(v, "0.1.0"),
            _ => panic!("expected CrateEntry::Version for flat crate"),
        }
    }

    #[test]
    fn flat_format_entry_point_only() {
        let toml = r#"entry_point = "app.py""#;
        let config = MambaConfig::from_str(toml).unwrap();
        assert_eq!(config.entry_point(), Some("app.py"));
        assert!(config.crates.is_empty());
    }

    #[test]
    fn flat_format_with_expose() {
        let toml = r#"
entry_point = "app.py"
[crates]
cclab-schema-mamba = "0.1.0"
[expose]
cclab-schema-mamba = ["BaseModel", "Field"]
"#;
        let config = MambaConfig::from_str(toml).unwrap();
        assert_eq!(config.entry_point(), Some("app.py"));
        assert!(config.crates.contains_key("cclab-schema-mamba"));
        let exposed = config.expose.get("cclab-schema-mamba").unwrap();
        assert!(exposed.contains(&"BaseModel".to_string()));
        assert!(exposed.contains(&"Field".to_string()));
    }

    #[test]
    fn flat_format_is_symbol_exposed_blocks_unlisted() {
        let toml = r#"
entry_point = "app.py"
[crates]
cclab-schema-mamba = "0.1.0"
[expose]
cclab-schema-mamba = ["BaseModel"]
"#;
        let cfg = MambaConfig::from_str(toml).unwrap();
        assert!(cfg.is_symbol_exposed("cclab-schema-mamba", "BaseModel"));
        assert!(
            !cfg.is_symbol_exposed("cclab-schema-mamba", "Field"),
            "Field should be blocked in flat-format expose"
        );
    }

    #[test]
    fn flat_format_has_crate_expose() {
        let toml = r#"
entry_point = "app.py"
[expose]
cclab-schema-mamba = ["BaseModel"]
"#;
        let cfg = MambaConfig::from_str(toml).unwrap();
        assert!(cfg.has_crate_expose("cclab-schema-mamba"));
        assert!(!cfg.has_crate_expose("pgkit-binding"));
    }

    #[test]
    fn flat_format_discover() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(
            dir.path().join("mamba.toml"),
            "entry_point = \"src/main.py\"\n[crates]\ncclab-schema-mamba = \"0.1.0\"\n",
        )
        .unwrap();

        let (cfg, found_path) =
            MambaConfig::discover(dir.path()).expect("should find flat mamba.toml");
        assert_eq!(cfg.entry_point(), Some("src/main.py"));
        assert!(cfg.crates.contains_key("cclab-schema-mamba"));
        assert_eq!(found_path, dir.path().join("mamba.toml"));
    }

    // ── Entry point precedence ───────────────────────────────────────────────

    #[test]
    fn entry_point_top_level_takes_precedence() {
        let toml = r#"
entry_point = "top.py"
[project]
name = "test"
version = "1.0.0"
entry_point = "project.py"
"#;
        let config = MambaConfig::from_str(toml).unwrap();
        // Top-level entry_point takes precedence over project.entry_point.
        assert_eq!(config.entry_point(), Some("top.py"));
    }

    #[test]
    fn entry_point_falls_back_to_project() {
        let toml = r#"
[project]
name = "test"
version = "1.0.0"
entry_point = "project.py"
"#;
        let config = MambaConfig::from_str(toml).unwrap();
        assert_eq!(config.entry_point(), Some("project.py"));
    }

    // ── #1134 regression: Conductor mamba.toml format ────────────────────────

    /// Regression test for #1134: Conductor's mamba.toml with [project] and
    /// [crates.cclab-schema-mamba] sub-tables caused a TOML parse error when
    /// the CLI used driver/config.rs which expected flat crates = version_string.
    /// The unified MambaConfig in config/schema.rs now handles both formats.
    #[test]
    fn issue_1134_conductor_toml_format_no_parse_error() {
        // This is the format that Conductor's mamba.toml uses.
        // Before #1134, this caused: "invalid type: map, expected a string"
        let toml = r#"
[project]
name = "conductor"
version = "0.1.0"
entry_point = "src/main.py"

[crates.cclab-schema-mamba]
version = "0.1.0"
expose = ["BaseModel", "Field"]
"#;
        let config = MambaConfig::from_str(toml)
            .expect("Conductor mamba.toml format should parse without error (#1134)");
        assert_eq!(config.project.name, "conductor");
        assert_eq!(config.entry_point(), Some("src/main.py"));
        assert!(config.crates.contains_key("cclab-schema-mamba"));
        match &config.crates["cclab-schema-mamba"] {
            CrateEntry::Config(c) => {
                assert!(c.expose.contains(&"BaseModel".to_string()));
                assert!(c.expose.contains(&"Field".to_string()));
            }
            CrateEntry::Version(_) => {
                panic!("Expected CrateEntry::Config for structured crate entry")
            }
        }
    }
}
