//! Configuration for Argus
//!
//! Parses cclab_lens.toml configuration files.

use serde::Deserialize;
use std::collections::HashSet;
use std::path::Path;

/// Top-level Argus configuration
#[derive(Debug, Clone, Default, Deserialize)]
pub struct ArgusConfig {
    #[serde(default)]
    pub cclab_lens: ArgusSettings,
}

/// Main settings under [cclab_lens]
#[derive(Debug, Clone, Deserialize)]
pub struct ArgusSettings {
    /// Languages to analyze
    #[serde(default)]
    pub languages: Vec<String>,

    /// LSP server port (default: 5007)
    #[serde(default = "default_lsp_port")]
    pub lsp_port: u16,

    /// Python-specific settings
    #[serde(default)]
    pub python: PythonConfig,

    /// TypeScript-specific settings
    #[serde(default)]
    pub typescript: TypeScriptConfig,

    /// Rust-specific settings
    #[serde(default)]
    pub rust: RustConfig,
}

impl Default for ArgusSettings {
    fn default() -> Self {
        Self {
            languages: Vec::new(),
            lsp_port: default_lsp_port(),
            python: PythonConfig::default(),
            typescript: TypeScriptConfig::default(),
            rust: RustConfig::default(),
        }
    }
}

/// Python configuration
#[derive(Debug, Clone, Default, Deserialize)]
pub struct PythonConfig {
    /// Target Python version (e.g., "3.11")
    #[serde(default)]
    pub target_version: Option<String>,

    /// Patterns to exclude
    #[serde(default)]
    pub exclude: Vec<String>,

    /// Lint settings
    #[serde(default)]
    pub lint: LintConfig,

    /// isort-like settings
    #[serde(default)]
    pub isort: IsortConfig,
}

/// TypeScript configuration
#[derive(Debug, Clone, Default, Deserialize)]
pub struct TypeScriptConfig {
    /// Whether TypeScript checking is enabled
    #[serde(default = "default_true")]
    pub enabled: bool,

    /// Patterns to exclude
    #[serde(default)]
    pub exclude: Vec<String>,

    /// Lint settings
    #[serde(default)]
    pub lint: LintConfig,
}

/// Rust configuration
#[derive(Debug, Clone, Default, Deserialize)]
pub struct RustConfig {
    /// Whether Rust checking is enabled
    #[serde(default = "default_true")]
    pub enabled: bool,

    /// Patterns to exclude
    #[serde(default)]
    pub exclude: Vec<String>,

    /// Lint settings
    #[serde(default)]
    pub lint: LintConfig,
}

/// Lint configuration (shared across languages)
#[derive(Debug, Clone, Deserialize)]
pub struct LintConfig {
    /// Whether linting is enabled
    #[serde(default = "default_true")]
    pub enabled: bool,

    /// Rules to enable (e.g., ["PY1", "PY2", "PY4"])
    #[serde(default)]
    pub select: Vec<String>,

    /// Rules to ignore (e.g., ["PY103"])
    #[serde(default)]
    pub ignore: Vec<String>,
}

impl Default for LintConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            select: Vec::new(),
            ignore: Vec::new(),
        }
    }
}

/// Language-agnostic lint config used by checkers
#[derive(Debug, Clone, Default)]
pub struct LanguageConfig {
    /// Rules to ignore
    pub ignore_rules: HashSet<String>,
    /// Rule prefixes to select (empty = all)
    pub select_prefixes: Vec<String>,
}

impl LanguageConfig {
    /// Check if a rule is enabled
    pub fn is_rule_enabled(&self, rule_id: &str) -> bool {
        // If explicitly ignored, skip
        if self.ignore_rules.contains(rule_id) {
            return false;
        }

        // If select is empty, all rules are enabled
        if self.select_prefixes.is_empty() {
            return true;
        }

        // Check if rule matches any select prefix
        self.select_prefixes
            .iter()
            .any(|prefix| rule_id.starts_with(prefix))
    }
}

impl From<&LintConfig> for LanguageConfig {
    fn from(config: &LintConfig) -> Self {
        Self {
            ignore_rules: config.ignore.iter().cloned().collect(),
            select_prefixes: config.select.clone(),
        }
    }
}

/// isort-like configuration
#[derive(Debug, Clone, Default, Deserialize)]
pub struct IsortConfig {
    /// Whether isort is enabled
    #[serde(default = "default_true")]
    pub enabled: bool,

    /// Known first-party packages
    #[serde(default)]
    pub known_first_party: Vec<String>,

    /// Known third-party packages
    #[serde(default)]
    pub known_third_party: Vec<String>,

    /// Known standard library modules (overrides)
    #[serde(default)]
    pub known_standard_library: Vec<String>,
}

fn default_true() -> bool {
    true
}

fn default_lsp_port() -> u16 {
    5007
}

impl ArgusConfig {
    /// Load configuration from a file
    pub fn from_file(path: &Path) -> Result<Self, ConfigError> {
        let content = std::fs::read_to_string(path).map_err(ConfigError::Io)?;
        Self::from_str(&content)
    }

    /// Parse configuration from a string
    pub fn from_str(content: &str) -> Result<Self, ConfigError> {
        toml::from_str(content).map_err(ConfigError::Parse)
    }

    /// Find and load configuration from a directory (looks for cclab_lens.toml)
    pub fn from_directory(dir: &Path) -> Result<Self, ConfigError> {
        let config_path = dir.join("cclab_lens.toml");
        if config_path.exists() {
            Self::from_file(&config_path)
        } else {
            // Try parent directories
            if let Some(parent) = dir.parent() {
                Self::from_directory(parent)
            } else {
                // No config found, use defaults
                Ok(Self::default())
            }
        }
    }

    /// Get the language config for Python
    pub fn python_lint_config(&self) -> LanguageConfig {
        LanguageConfig::from(&self.cclab_lens.python.lint)
    }

    /// Get the language config for TypeScript
    pub fn typescript_lint_config(&self) -> LanguageConfig {
        LanguageConfig::from(&self.cclab_lens.typescript.lint)
    }

    /// Get the language config for Rust
    pub fn rust_lint_config(&self) -> LanguageConfig {
        LanguageConfig::from(&self.cclab_lens.rust.lint)
    }
}

/// Configuration errors
#[derive(Debug)]
pub enum ConfigError {
    Io(std::io::Error),
    Parse(toml::de::Error),
}

impl std::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConfigError::Io(e) => write!(f, "Failed to read config file: {}", e),
            ConfigError::Parse(e) => write!(f, "Failed to parse config: {}", e),
        }
    }
}

impl std::error::Error for ConfigError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_empty_config() {
        let config = ArgusConfig::from_str("").unwrap();
        assert!(config.cclab_lens.python.lint.enabled);
    }

    #[test]
    fn test_parse_full_config() {
        let toml = r#"
[cclab_lens]
languages = ["python", "typescript", "rust"]
lsp_port = 5007

[cclab_lens.python]
target_version = "3.11"
exclude = ["**/migrations/**"]

[cclab_lens.python.lint]
enabled = true
select = ["PY1", "PY2", "PY4", "PY5"]
ignore = ["PY103"]

[cclab_lens.python.isort]
enabled = true
known_first_party = ["cclab_nucleus"]

[cclab_lens.typescript]
enabled = true

[cclab_lens.rust]
enabled = true
"#;

        let config = ArgusConfig::from_str(toml).unwrap();

        assert_eq!(
            config.cclab_lens.languages,
            vec!["python", "typescript", "rust"]
        );
        assert_eq!(config.cclab_lens.lsp_port, 5007);
        assert_eq!(
            config.cclab_lens.python.target_version,
            Some("3.11".to_string())
        );
        assert_eq!(config.cclab_lens.python.exclude, vec!["**/migrations/**"]);
        assert_eq!(
            config.cclab_lens.python.lint.select,
            vec!["PY1", "PY2", "PY4", "PY5"]
        );
        assert_eq!(config.cclab_lens.python.lint.ignore, vec!["PY103"]);
        assert_eq!(
            config.cclab_lens.python.isort.known_first_party,
            vec!["cclab_nucleus"]
        );
        assert!(config.cclab_lens.typescript.enabled);
        assert!(config.cclab_lens.rust.enabled);
    }

    #[test]
    fn test_language_config_rule_filtering() {
        let lint = LintConfig {
            enabled: true,
            select: vec!["PY1".to_string(), "PY2".to_string()],
            ignore: vec!["PY103".to_string()],
        };

        let config = LanguageConfig::from(&lint);

        // PY103 is explicitly ignored
        assert!(!config.is_rule_enabled("PY103"));

        // PY101 matches PY1 prefix
        assert!(config.is_rule_enabled("PY101"));

        // PY201 matches PY2 prefix
        assert!(config.is_rule_enabled("PY201"));

        // PY401 doesn't match any prefix
        assert!(!config.is_rule_enabled("PY401"));
    }

    #[test]
    fn test_language_config_empty_select() {
        let lint = LintConfig {
            enabled: true,
            select: vec![],
            ignore: vec!["PY103".to_string()],
        };

        let config = LanguageConfig::from(&lint);

        // Empty select means all rules enabled (except ignored)
        assert!(config.is_rule_enabled("PY101"));
        assert!(config.is_rule_enabled("PY401"));
        assert!(!config.is_rule_enabled("PY103"));
    }
}
