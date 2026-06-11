//! Formatter integration — unified interface for external formatters
//!
//! Wraps rustfmt, prettier, gofmt, black, terraform fmt, etc.

pub mod detect;

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Result of formatting a file
#[derive(Debug, Clone)]
pub struct FormatResult {
    /// Original source (before formatting)
    pub original: String,
    /// Formatted source (after formatting)
    pub formatted: String,
    /// Whether the file was changed
    pub changed: bool,
    /// Formatter used
    pub formatter: String,
}

/// Configuration for a single formatter
#[derive(Debug, Clone)]
pub struct FormatterConfig {
    /// Name of the formatter binary
    pub binary_name: String,
    /// Arguments for formatting (reading from stdin, writing to stdout)
    pub format_args: Vec<String>,
    /// Arguments for check mode (exit code indicates if formatting needed)
    pub check_args: Vec<String>,
    /// Whether this formatter reads from stdin
    pub stdin_mode: bool,
}

/// Registry of formatters per language
pub struct FormatterRegistry {
    /// Language -> formatter config
    formatters: HashMap<String, FormatterConfig>,
    /// Cached binary availability
    available: HashMap<String, PathBuf>,
}

impl FormatterRegistry {
    /// Create a new registry with default formatter configs.
    /// Probes for binary availability using `detect::find_binary`.
    pub fn new() -> Self {
        let mut registry = Self {
            formatters: HashMap::new(),
            available: HashMap::new(),
        };

        // Register default formatters
        registry.register(
            "rust",
            FormatterConfig {
                binary_name: "rustfmt".into(),
                format_args: vec!["--edition".into(), "2021".into()],
                check_args: vec!["--check".into(), "--edition".into(), "2021".into()],
                stdin_mode: true,
            },
        );
        registry.register(
            "python",
            FormatterConfig {
                binary_name: "black".into(),
                format_args: vec!["-".into(), "-q".into()],
                check_args: vec!["--check".into(), "-q".into()],
                stdin_mode: true,
            },
        );
        registry.register(
            "go",
            FormatterConfig {
                binary_name: "gofmt".into(),
                format_args: vec![],
                check_args: vec!["-l".into()],
                stdin_mode: true,
            },
        );
        registry.register(
            "javascript",
            FormatterConfig {
                binary_name: "prettier".into(),
                format_args: vec!["--parser".into(), "babel".into()],
                check_args: vec!["--check".into(), "--parser".into(), "babel".into()],
                stdin_mode: true,
            },
        );
        registry.register(
            "typescript",
            FormatterConfig {
                binary_name: "prettier".into(),
                format_args: vec!["--parser".into(), "typescript".into()],
                check_args: vec!["--check".into(), "--parser".into(), "typescript".into()],
                stdin_mode: true,
            },
        );
        registry.register(
            "css",
            FormatterConfig {
                binary_name: "prettier".into(),
                format_args: vec!["--parser".into(), "css".into()],
                check_args: vec!["--check".into(), "--parser".into(), "css".into()],
                stdin_mode: true,
            },
        );
        registry.register(
            "html",
            FormatterConfig {
                binary_name: "prettier".into(),
                format_args: vec!["--parser".into(), "html".into()],
                check_args: vec!["--check".into(), "--parser".into(), "html".into()],
                stdin_mode: true,
            },
        );
        registry.register(
            "hcl",
            FormatterConfig {
                binary_name: "terraform".into(),
                format_args: vec!["fmt".into(), "-".into()],
                check_args: vec!["fmt".into(), "-check".into()],
                stdin_mode: true,
            },
        );
        registry.register(
            "sql",
            FormatterConfig {
                binary_name: "pg_format".into(),
                format_args: vec![],
                check_args: vec![], // pg_format doesn't have check mode
                stdin_mode: true,
            },
        );

        // Detect available binaries
        registry.detect_all();

        registry
    }

    fn register(&mut self, language: &str, config: FormatterConfig) {
        self.formatters.insert(language.to_string(), config);
    }

    fn detect_all(&mut self) {
        for (_, config) in &self.formatters {
            if let Some(path) = detect::find_binary(&config.binary_name) {
                self.available.insert(config.binary_name.clone(), path);
            }
        }
    }

    /// Format source code for a given language.
    /// Returns None if no formatter is available for the language.
    pub fn format(&self, source: &str, language: &str) -> Option<FormatResult> {
        let config = self.formatters.get(language)?;
        let binary_path = self.available.get(&config.binary_name)?;

        let mut cmd = Command::new(binary_path);
        for arg in &config.format_args {
            cmd.arg(arg);
        }

        if config.stdin_mode {
            use std::io::Write;
            cmd.stdin(std::process::Stdio::piped());
            cmd.stdout(std::process::Stdio::piped());
            cmd.stderr(std::process::Stdio::piped());

            let mut child = cmd.spawn().ok()?;
            if let Some(mut stdin) = child.stdin.take() {
                stdin.write_all(source.as_bytes()).ok()?;
            }
            let output = child.wait_with_output().ok()?;
            if output.status.success() {
                let formatted = String::from_utf8_lossy(&output.stdout).to_string();
                let changed = formatted != source;
                Some(FormatResult {
                    original: source.to_string(),
                    formatted,
                    changed,
                    formatter: config.binary_name.clone(),
                })
            } else {
                // Formatter failed, return unchanged
                tracing::warn!(
                    "Formatter {} failed: {}",
                    config.binary_name,
                    String::from_utf8_lossy(&output.stderr)
                );
                None
            }
        } else {
            None
        }
    }

    /// Check if formatting would change the file (without modifying it)
    pub fn format_check(&self, source: &str, language: &str) -> Option<bool> {
        let result = self.format(source, language)?;
        Some(result.changed)
    }

    /// Check if a formatter is available for the language
    pub fn is_available(&self, language: &str) -> bool {
        if let Some(config) = self.formatters.get(language) {
            self.available.contains_key(&config.binary_name)
        } else {
            false
        }
    }

    /// List all registered languages and their availability
    pub fn status(&self) -> Vec<(String, String, bool)> {
        self.formatters
            .iter()
            .map(|(lang, config)| {
                let available = self.available.contains_key(&config.binary_name);
                (lang.clone(), config.binary_name.clone(), available)
            })
            .collect()
    }

    /// Map a file extension to a language name
    pub fn language_for_extension(ext: &str) -> Option<&'static str> {
        match ext {
            "rs" => Some("rust"),
            "py" | "pyi" => Some("python"),
            "go" => Some("go"),
            "js" | "jsx" => Some("javascript"),
            "ts" | "tsx" => Some("typescript"),
            "css" => Some("css"),
            "html" | "htm" => Some("html"),
            "tf" | "tfvars" => Some("hcl"),
            "sql" => Some("sql"),
            _ => None,
        }
    }

    /// Format a file by path (auto-detect language from extension)
    pub fn format_file(&self, path: &Path) -> Option<FormatResult> {
        let ext = path.extension()?.to_str()?;
        let language = Self::language_for_extension(ext)?;
        let source = std::fs::read_to_string(path).ok()?;
        self.format(&source, language)
    }
}

impl Default for FormatterRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_language_for_extension() {
        assert_eq!(
            FormatterRegistry::language_for_extension("rs"),
            Some("rust")
        );
        assert_eq!(
            FormatterRegistry::language_for_extension("py"),
            Some("python")
        );
        assert_eq!(
            FormatterRegistry::language_for_extension("pyi"),
            Some("python")
        );
        assert_eq!(FormatterRegistry::language_for_extension("go"), Some("go"));
        assert_eq!(
            FormatterRegistry::language_for_extension("js"),
            Some("javascript")
        );
        assert_eq!(
            FormatterRegistry::language_for_extension("jsx"),
            Some("javascript")
        );
        assert_eq!(
            FormatterRegistry::language_for_extension("ts"),
            Some("typescript")
        );
        assert_eq!(
            FormatterRegistry::language_for_extension("tsx"),
            Some("typescript")
        );
        assert_eq!(
            FormatterRegistry::language_for_extension("css"),
            Some("css")
        );
        assert_eq!(
            FormatterRegistry::language_for_extension("html"),
            Some("html")
        );
        assert_eq!(
            FormatterRegistry::language_for_extension("htm"),
            Some("html")
        );
        assert_eq!(FormatterRegistry::language_for_extension("tf"), Some("hcl"));
        assert_eq!(
            FormatterRegistry::language_for_extension("tfvars"),
            Some("hcl")
        );
        assert_eq!(
            FormatterRegistry::language_for_extension("sql"),
            Some("sql")
        );
        assert_eq!(FormatterRegistry::language_for_extension("unknown"), None);
    }

    #[test]
    fn test_status_returns_all_registered_formatters() {
        let registry = FormatterRegistry::new();
        let status = registry.status();
        let languages: Vec<&str> = status.iter().map(|(lang, _, _)| lang.as_str()).collect();

        assert!(languages.contains(&"rust"));
        assert!(languages.contains(&"python"));
        assert!(languages.contains(&"go"));
        assert!(languages.contains(&"javascript"));
        assert!(languages.contains(&"typescript"));
        assert!(languages.contains(&"css"));
        assert!(languages.contains(&"html"));
        assert!(languages.contains(&"hcl"));
        assert!(languages.contains(&"sql"));
        assert_eq!(status.len(), 9);
    }

    #[test]
    fn test_is_available_unknown_language() {
        let registry = FormatterRegistry::new();
        assert!(!registry.is_available("brainfuck"));
        assert!(!registry.is_available(""));
        assert!(!registry.is_available("cobol"));
    }

    #[test]
    fn test_format_returns_none_for_unavailable_formatter() {
        let registry = FormatterRegistry {
            formatters: HashMap::new(),
            available: HashMap::new(),
        };
        let result = registry.format("fn main() {}", "rust");
        assert!(result.is_none());
    }
}
