//! Configuration system for Argus type checker
//!
//! Reads configuration from pyproject.toml [tool.cclab_lens] section
//! and supports per-directory overrides.
//!
//! ## Python Environment Configuration
//!
//! The `[tool.cclab_lens.python]` section supports:
//! - `search_paths`: Additional directories to search for modules
//! - `venv_path`: Path to the virtual environment to use
//! - `ignore_site_packages`: Whether to ignore site-packages

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// Python environment configuration for module resolution
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(default)]
pub struct PythonEnvConfig {
    /// Additional directories to search for modules
    pub search_paths: Vec<PathBuf>,

    /// Path to the virtual environment to use (overrides auto-detection)
    pub venv_path: Option<PathBuf>,

    /// Whether to ignore site-packages (default: false)
    #[serde(default)]
    pub ignore_site_packages: bool,
}

/// Configuration for Argus type checker
#[derive(Debug, Clone, Deserialize, Default)]
#[serde(default)]
pub struct ArgusConfig {
    /// Python version to check against (e.g., "3.10")
    pub python_version: Option<String>,

    /// Python environment configuration for module resolution
    #[serde(default)]
    pub python: PythonEnvConfig,

    /// Enable strict mode (like mypy --strict)
    pub strict: bool,

    /// Enable strict optional checking (None must be explicit)
    pub strict_optional: bool,

    /// Warn about returning Any from typed function
    pub warn_return_any: bool,

    /// Warn about unused ignores
    pub warn_unused_ignores: bool,

    /// Check untyped functions
    pub check_untyped_defs: bool,

    /// Disallow untyped decorators
    pub disallow_untyped_decorators: bool,

    /// Disallow incomplete function definitions
    pub disallow_incomplete_defs: bool,

    /// Disallow untyped function definitions
    pub disallow_untyped_defs: bool,

    /// Paths to exclude from analysis
    pub exclude: Vec<String>,

    /// Paths to include (overrides exclude)
    pub include: Vec<String>,

    /// Per-directory overrides
    #[serde(default)]
    pub overrides: Vec<OverrideConfig>,

    /// Custom type stub paths
    pub stub_paths: Vec<PathBuf>,

    /// Plugins to enable
    pub plugins: Vec<String>,

    // === Typeshed configuration ===
    /// Custom path to a local typeshed copy (takes precedence over downloads)
    pub typeshed_path: Option<PathBuf>,

    /// Directory to store downloaded typeshed stubs (default: ~/.cache/cclab_lens)
    pub typeshed_cache_dir: Option<PathBuf>,

    /// Disable network requests for typeshed downloads (offline mode)
    #[serde(default)]
    pub typeshed_offline: bool,

    /// Cache TTL in days for typeshed stubs (default: 7)
    #[serde(default = "default_typeshed_ttl")]
    pub typeshed_ttl_days: u32,

    /// Optional commit hash to pin typeshed version
    pub typeshed_commit: Option<String>,

    /// Stub precedence order: "local", "typeshed", "bundled" (default: local > typeshed > bundled)
    #[serde(default = "default_stub_precedence")]
    pub stub_precedence: Vec<String>,
}

fn default_typeshed_ttl() -> u32 {
    7
}

fn default_stub_precedence() -> Vec<String> {
    vec![
        "local".to_string(),
        "typeshed".to_string(),
        "bundled".to_string(),
    ]
}

/// Per-directory configuration override
#[derive(Debug, Clone, Deserialize, Default)]
#[serde(default)]
pub struct OverrideConfig {
    /// Glob pattern to match files (e.g., "tests/**/*.py")
    pub pattern: String,

    /// Enable strict mode for matching files
    pub strict: Option<bool>,

    /// Check untyped defs in matching files
    pub check_untyped_defs: Option<bool>,

    /// Disallow untyped defs in matching files
    pub disallow_untyped_defs: Option<bool>,

    /// Ignore missing imports for matching files
    pub ignore_missing_imports: Option<bool>,
}

/// pyproject.toml structure
#[derive(Debug, Deserialize)]
struct PyProject {
    tool: Option<ToolSection>,
}

#[derive(Debug, Deserialize)]
struct ToolSection {
    cclab_lens: Option<ArgusConfig>,
}

impl ArgusConfig {
    /// Create a new config with defaults
    pub fn new() -> Self {
        Self::default()
    }

    /// Get the Python version for stub resolution (defaults to "3.11")
    pub fn python_version_or_default(&self) -> String {
        self.python_version
            .clone()
            .unwrap_or_else(|| "3.11".to_string())
    }

    /// Get the typeshed cache directory (defaults to ~/.cache/cclab_lens)
    pub fn typeshed_cache_dir_or_default(&self) -> PathBuf {
        self.typeshed_cache_dir.clone().unwrap_or_else(|| {
            if let Ok(home) = std::env::var("HOME") {
                PathBuf::from(home).join(".cache").join("cclab_lens")
            } else if let Ok(cache) = std::env::var("XDG_CACHE_HOME") {
                PathBuf::from(cache).join("cclab_lens")
            } else {
                PathBuf::from(".cclab_lens-cache")
            }
        })
    }

    /// Create a strict configuration
    pub fn strict() -> Self {
        Self {
            strict: true,
            strict_optional: true,
            warn_return_any: true,
            warn_unused_ignores: true,
            check_untyped_defs: true,
            disallow_untyped_decorators: true,
            disallow_incomplete_defs: true,
            disallow_untyped_defs: true,
            ..Default::default()
        }
    }

    /// Load config from pyproject.toml in the given directory
    pub fn from_pyproject(dir: &Path) -> Self {
        let pyproject_path = dir.join("pyproject.toml");
        if pyproject_path.exists() {
            if let Ok(contents) = std::fs::read_to_string(&pyproject_path) {
                if let Ok(pyproject) = toml::from_str::<PyProject>(&contents) {
                    if let Some(tool) = pyproject.tool {
                        if let Some(config) = tool.cclab_lens {
                            return config;
                        }
                    }
                }
            }
        }
        Self::default()
    }

    /// Find and load config from pyproject.toml by searching up the directory tree
    pub fn discover(start: &Path) -> Self {
        let mut current = start.to_path_buf();
        loop {
            let config = Self::from_pyproject(&current);
            // If we found a config with non-default values, use it
            if config.python_version.is_some()
                || config.strict
                || !config.exclude.is_empty()
                || !config.overrides.is_empty()
            {
                return config;
            }

            // Move up to parent directory
            if let Some(parent) = current.parent() {
                current = parent.to_path_buf();
            } else {
                break;
            }
        }
        Self::default()
    }

    /// Get effective config for a specific file path
    /// Applies override rules based on glob patterns
    pub fn effective_for(&self, file_path: &Path) -> EffectiveConfig {
        let mut effective = EffectiveConfig {
            strict: self.strict,
            strict_optional: self.strict_optional,
            warn_return_any: self.warn_return_any,
            check_untyped_defs: self.check_untyped_defs,
            disallow_untyped_defs: self.disallow_untyped_defs,
            ignore_missing_imports: false,
        };

        // Apply matching overrides
        let file_str = file_path.to_string_lossy();
        for override_config in &self.overrides {
            if glob_matches(&override_config.pattern, &file_str) {
                if let Some(strict) = override_config.strict {
                    effective.strict = strict;
                }
                if let Some(check) = override_config.check_untyped_defs {
                    effective.check_untyped_defs = check;
                }
                if let Some(disallow) = override_config.disallow_untyped_defs {
                    effective.disallow_untyped_defs = disallow;
                }
                if let Some(ignore) = override_config.ignore_missing_imports {
                    effective.ignore_missing_imports = ignore;
                }
            }
        }

        effective
    }

    /// Check if a path should be excluded from analysis
    pub fn should_exclude(&self, path: &Path) -> bool {
        let path_str = path.to_string_lossy();

        // Check explicit excludes
        for pattern in &self.exclude {
            if glob_matches(pattern, &path_str) {
                // Check if explicitly included
                for include_pattern in &self.include {
                    if glob_matches(include_pattern, &path_str) {
                        return false;
                    }
                }
                return true;
            }
        }

        false
    }
}

/// Effective configuration for a specific file
#[derive(Debug, Clone)]
pub struct EffectiveConfig {
    pub strict: bool,
    pub strict_optional: bool,
    pub warn_return_any: bool,
    pub check_untyped_defs: bool,
    pub disallow_untyped_defs: bool,
    pub ignore_missing_imports: bool,
}

/// Simple glob pattern matching
/// Supports * (any characters) and ** (any path segments)
fn glob_matches(pattern: &str, path: &str) -> bool {
    let pattern = pattern.replace("**", "\x00").replace('*', "[^/]*");
    let pattern = pattern.replace('\x00', ".*");
    let regex_pattern = format!("^{}$", pattern);

    if let Ok(re) = regex_lite::Regex::new(&regex_pattern) {
        re.is_match(path)
    } else {
        // Fallback: simple contains check
        path.contains(pattern.trim_matches('*'))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = ArgusConfig::new();
        assert!(!config.strict);
        assert!(!config.strict_optional);
        assert!(config.exclude.is_empty());
    }

    #[test]
    fn test_strict_config() {
        let config = ArgusConfig::strict();
        assert!(config.strict);
        assert!(config.strict_optional);
        assert!(config.warn_return_any);
        assert!(config.disallow_untyped_defs);
    }

    #[test]
    fn test_glob_matching() {
        assert!(glob_matches("*.py", "test.py"));
        assert!(glob_matches("tests/*.py", "tests/test_main.py"));
        assert!(glob_matches("tests/**/*.py", "tests/unit/test_main.py"));
        assert!(!glob_matches("*.py", "src/test.txt"));
    }

    #[test]
    fn test_effective_config_with_override() {
        let config = ArgusConfig {
            strict: false,
            overrides: vec![OverrideConfig {
                pattern: "tests/**/*.py".to_string(),
                check_untyped_defs: Some(false),
                ignore_missing_imports: Some(true),
                ..Default::default()
            }],
            ..Default::default()
        };

        let effective = config.effective_for(Path::new("tests/unit/test_main.py"));
        assert!(!effective.check_untyped_defs);
        assert!(effective.ignore_missing_imports);

        let effective_src = config.effective_for(Path::new("src/main.py"));
        assert!(!effective_src.check_untyped_defs); // Default
        assert!(!effective_src.ignore_missing_imports);
    }

    #[test]
    fn test_should_exclude() {
        let config = ArgusConfig {
            exclude: vec!["venv/**".to_string(), "__pycache__/**".to_string()],
            include: vec!["venv/important.py".to_string()],
            ..Default::default()
        };

        assert!(config.should_exclude(Path::new("venv/lib/site-packages/foo.py")));
        assert!(!config.should_exclude(Path::new("venv/important.py"))); // Explicitly included
        assert!(!config.should_exclude(Path::new("src/main.py")));
    }

    #[test]
    fn test_parse_pyproject_toml() {
        let toml_content = r#"
[tool.cclab_lens]
python_version = "3.10"
strict = true
exclude = ["venv/**", "__pycache__/**"]

[[tool.cclab_lens.overrides]]
pattern = "tests/**/*.py"
check_untyped_defs = false
"#;

        let pyproject: PyProject = toml::from_str(toml_content).unwrap();
        let config = pyproject.tool.unwrap().cclab_lens.unwrap();

        assert_eq!(config.python_version, Some("3.10".to_string()));
        assert!(config.strict);
        assert_eq!(config.exclude.len(), 2);
        assert_eq!(config.overrides.len(), 1);
        assert_eq!(config.overrides[0].pattern, "tests/**/*.py");
    }

    #[test]
    fn test_python_env_config_default() {
        let config = PythonEnvConfig::default();
        assert!(config.search_paths.is_empty());
        assert!(config.venv_path.is_none());
        assert!(!config.ignore_site_packages);
    }

    #[test]
    fn test_parse_python_env_config() {
        let toml_content = r#"
[tool.cclab_lens]
python_version = "3.11"

[tool.cclab_lens.python]
search_paths = ["./lib", "./src"]
venv_path = "./custom_env"
ignore_site_packages = true
"#;

        let pyproject: PyProject = toml::from_str(toml_content).unwrap();
        let config = pyproject.tool.unwrap().cclab_lens.unwrap();

        assert_eq!(config.python_version, Some("3.11".to_string()));
        assert_eq!(config.python.search_paths.len(), 2);
        assert_eq!(config.python.search_paths[0], PathBuf::from("./lib"));
        assert_eq!(config.python.search_paths[1], PathBuf::from("./src"));
        assert_eq!(config.python.venv_path, Some(PathBuf::from("./custom_env")));
        assert!(config.python.ignore_site_packages);
    }

    #[test]
    fn test_parse_python_env_config_partial() {
        let toml_content = r#"
[tool.cclab_lens]
python_version = "3.10"

[tool.cclab_lens.python]
venv_path = ".venv"
"#;

        let pyproject: PyProject = toml::from_str(toml_content).unwrap();
        let config = pyproject.tool.unwrap().cclab_lens.unwrap();

        assert!(config.python.search_paths.is_empty());
        assert_eq!(config.python.venv_path, Some(PathBuf::from(".venv")));
        assert!(!config.python.ignore_site_packages);
    }
}
