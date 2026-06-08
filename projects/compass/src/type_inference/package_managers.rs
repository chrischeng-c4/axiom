//! Package manager detection and integration (P1 M5.1)
//!
//! Automatically detects and integrates with Python package managers:
//! - uv (modern, fastest) - Priority 1
//! - Poetry (dependency resolution and packaging) - Priority 2
//! - Pipenv (virtual environment management) - Priority 3
//! - pip (standard package installer) - Priority 4 (fallback)
//!
//! Provides:
//! - Automatic project configuration detection
//! - Dependency parsing from lockfiles
//! - Virtual environment discovery
//! - Framework detection enhancement

use std::fs;
use std::path::PathBuf;

// ============================================================================
// Types
// ============================================================================

/// Package manager type
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum PackageManager {
    /// uv - Modern, fast package manager
    Uv,
    /// Poetry - Dependency resolution and packaging
    Poetry,
    /// Pipenv - Virtual environment management
    Pipenv,
    /// pip - Standard package installer
    Pip,
    /// Unknown or not detected
    Unknown,
}

impl PackageManager {
    /// Get human-readable name
    pub fn display_name(&self) -> &str {
        match self {
            Self::Uv => "uv",
            Self::Poetry => "Poetry",
            Self::Pipenv => "Pipenv",
            Self::Pip => "pip",
            Self::Unknown => "Unknown",
        }
    }

    /// Get typical config file name
    pub fn config_file_name(&self) -> &str {
        match self {
            Self::Uv => "pyproject.toml",
            Self::Poetry => "pyproject.toml",
            Self::Pipenv => "Pipfile",
            Self::Pip => "requirements.txt",
            Self::Unknown => "",
        }
    }

    /// Get lock file name
    pub fn lock_file_name(&self) -> Option<&str> {
        match self {
            Self::Uv => Some("uv.lock"),
            Self::Poetry => Some("poetry.lock"),
            Self::Pipenv => Some("Pipfile.lock"),
            Self::Pip => None,
            Self::Unknown => None,
        }
    }
}

/// Dependency information
#[derive(Debug, Clone, PartialEq)]
pub struct Dependency {
    /// Package name
    pub name: String,
    /// Version constraint (e.g., ">=4.0", "^0.100")
    pub version: Option<String>,
    /// Extras (e.g., ["all", "dev"])
    pub extras: Vec<String>,
    /// Development dependency
    pub is_dev: bool,
    /// Optional dependency
    pub is_optional: bool,
}

impl Dependency {
    /// Create a new dependency
    pub fn new(name: String) -> Self {
        Self {
            name,
            version: None,
            extras: Vec::new(),
            is_dev: false,
            is_optional: false,
        }
    }

    /// With version constraint
    pub fn with_version(mut self, version: String) -> Self {
        self.version = Some(version);
        self
    }

    /// With extras
    pub fn with_extras(mut self, extras: Vec<String>) -> Self {
        self.extras = extras;
        self
    }

    /// Mark as dev dependency
    pub fn as_dev(mut self) -> Self {
        self.is_dev = true;
        self
    }

    /// Check if this is a framework package
    pub fn is_framework(&self) -> bool {
        matches!(
            self.name.as_str(),
            "django" | "flask" | "fastapi" | "pydantic" | "sqlalchemy" | "celery"
        )
    }
}

/// Package manager detection result
#[derive(Debug, Clone)]
pub struct PackageManagerDetection {
    /// Detected package manager
    pub manager: PackageManager,
    /// Configuration file path (pyproject.toml, Pipfile, requirements.txt)
    pub config_file: PathBuf,
    /// Lock file path (uv.lock, poetry.lock, Pipfile.lock)
    pub lock_file: Option<PathBuf>,
    /// Virtual environment path (.venv, venv, etc.)
    pub venv_path: Option<PathBuf>,
    /// Parsed dependencies
    pub dependencies: Vec<Dependency>,
    /// Detection confidence (0.0 to 1.0)
    pub confidence: f64,
}

impl PackageManagerDetection {
    /// Create empty detection (Unknown manager)
    pub fn unknown() -> Self {
        Self {
            manager: PackageManager::Unknown,
            config_file: PathBuf::new(),
            lock_file: None,
            venv_path: None,
            dependencies: Vec::new(),
            confidence: 0.0,
        }
    }

    /// Get framework dependencies
    pub fn framework_dependencies(&self) -> Vec<&Dependency> {
        self.dependencies
            .iter()
            .filter(|d| d.is_framework())
            .collect()
    }

    /// Check if a specific package is present
    pub fn has_dependency(&self, name: &str) -> bool {
        self.dependencies.iter().any(|d| d.name == name)
    }

    /// Get dependency by name
    pub fn get_dependency(&self, name: &str) -> Option<&Dependency> {
        self.dependencies.iter().find(|d| d.name == name)
    }
}

// ============================================================================
// Detector
// ============================================================================

/// Package manager detector
pub struct PackageManagerDetector {
    /// Project root directory
    root: PathBuf,
}

impl PackageManagerDetector {
    /// Create a new detector for a project root
    pub fn new(root: PathBuf) -> Self {
        Self { root }
    }

    /// Detect package manager and parse configuration
    ///
    /// Priority order: uv > Poetry > Pipenv > pip
    pub fn detect(&self) -> PackageManagerDetection {
        // 1. Try uv (highest priority)
        if let Some(detection) = self.detect_uv() {
            return detection;
        }

        // 2. Try Poetry
        if let Some(detection) = self.detect_poetry() {
            return detection;
        }

        // 3. Try Pipenv
        if let Some(detection) = self.detect_pipenv() {
            return detection;
        }

        // 4. Fallback to pip
        if let Some(detection) = self.detect_pip() {
            return detection;
        }

        // Nothing found
        PackageManagerDetection::unknown()
    }

    /// Detect uv project
    ///
    /// Looks for: pyproject.toml + uv.lock
    fn detect_uv(&self) -> Option<PackageManagerDetection> {
        let pyproject_path = self.root.join("pyproject.toml");
        let uv_lock_path = self.root.join("uv.lock");

        // Check for uv.lock (strong indicator)
        if !uv_lock_path.exists() {
            return None;
        }

        // Check for pyproject.toml
        if !pyproject_path.exists() {
            return None;
        }

        let content = fs::read_to_string(&pyproject_path).ok()?;

        // Must have [tool.uv] or [project] section
        if !content.contains("[tool.uv]") && !content.contains("[project]") {
            return None;
        }

        // Parse dependencies from pyproject.toml
        let dependencies = self.parse_pyproject_dependencies(&content);

        // Find virtual environment
        let venv_path = self.find_venv();

        Some(PackageManagerDetection {
            manager: PackageManager::Uv,
            config_file: pyproject_path,
            lock_file: Some(uv_lock_path),
            venv_path,
            dependencies,
            confidence: 0.95, // High confidence with lock file
        })
    }

    /// Detect Poetry project
    ///
    /// Looks for: pyproject.toml with [tool.poetry]
    fn detect_poetry(&self) -> Option<PackageManagerDetection> {
        let pyproject_path = self.root.join("pyproject.toml");
        let poetry_lock_path = self.root.join("poetry.lock");

        // Check for pyproject.toml
        if !pyproject_path.exists() {
            return None;
        }

        let content = fs::read_to_string(&pyproject_path).ok()?;

        // Must have [tool.poetry] section
        if !content.contains("[tool.poetry]") {
            return None;
        }

        // Parse dependencies
        let dependencies = self.parse_pyproject_dependencies(&content);

        // Check for lock file
        let has_lock_file = poetry_lock_path.exists();
        let lock_file = if has_lock_file {
            Some(poetry_lock_path)
        } else {
            None
        };

        let venv_path = self.find_venv();

        Some(PackageManagerDetection {
            manager: PackageManager::Poetry,
            config_file: pyproject_path,
            lock_file,
            venv_path,
            dependencies,
            confidence: if has_lock_file { 0.95 } else { 0.85 },
        })
    }

    /// Detect Pipenv project
    ///
    /// Looks for: Pipfile
    fn detect_pipenv(&self) -> Option<PackageManagerDetection> {
        let pipfile_path = self.root.join("Pipfile");
        let pipfile_lock_path = self.root.join("Pipfile.lock");

        if !pipfile_path.exists() {
            return None;
        }

        let content = fs::read_to_string(&pipfile_path).ok()?;

        // Parse Pipfile format
        let dependencies = self.parse_pipfile_dependencies(&content);

        let has_lock_file = pipfile_lock_path.exists();
        let lock_file = if has_lock_file {
            Some(pipfile_lock_path)
        } else {
            None
        };

        let venv_path = self.find_venv();

        Some(PackageManagerDetection {
            manager: PackageManager::Pipenv,
            config_file: pipfile_path,
            lock_file,
            venv_path,
            dependencies,
            confidence: if has_lock_file { 0.90 } else { 0.80 },
        })
    }

    /// Detect pip (requirements.txt)
    ///
    /// Looks for: requirements.txt or requirements/*.txt
    fn detect_pip(&self) -> Option<PackageManagerDetection> {
        // Look for requirements.txt or requirements/*.txt
        let requirements_paths = vec![
            self.root.join("requirements.txt"),
            self.root.join("requirements/base.txt"),
            self.root.join("requirements/prod.txt"),
            self.root.join("requirements/production.txt"),
        ];

        for path in &requirements_paths {
            if path.exists() {
                let content = fs::read_to_string(path).ok()?;
                let dependencies = self.parse_requirements_txt(&content);

                let venv_path = self.find_venv();

                return Some(PackageManagerDetection {
                    manager: PackageManager::Pip,
                    config_file: path.clone(),
                    lock_file: None,
                    venv_path,
                    dependencies,
                    confidence: 0.70, // Lower confidence without lock file
                });
            }
        }

        None
    }

    /// Parse dependencies from pyproject.toml
    ///
    /// Supports both [project.dependencies] and [tool.poetry.dependencies]
    fn parse_pyproject_dependencies(&self, content: &str) -> Vec<Dependency> {
        let mut dependencies = Vec::new();

        // Parse [project] dependencies array
        // Format: dependencies = ["django>=4.0", "fastapi[all]"]
        if let Some(deps_section) = Self::extract_array_section(content, "dependencies") {
            for line in deps_section.lines() {
                if let Some(dep) = Self::parse_dependency_line(line) {
                    dependencies.push(dep);
                }
            }
        }

        // Also check [tool.poetry.dependencies] for Poetry
        if content.contains("[tool.poetry.dependencies]") {
            if let Some(poetry_deps) =
                Self::extract_toml_section(content, "[tool.poetry.dependencies]")
            {
                for line in poetry_deps.lines() {
                    if let Some(dep) = Self::parse_poetry_dependency(line) {
                        dependencies.push(dep);
                    }
                }
            }
        }

        // Check [tool.poetry.dev-dependencies] for dev deps
        if content.contains("[tool.poetry.dev-dependencies]") {
            if let Some(dev_deps) =
                Self::extract_toml_section(content, "[tool.poetry.dev-dependencies]")
            {
                for line in dev_deps.lines() {
                    if let Some(dep) = Self::parse_poetry_dependency(line) {
                        dependencies.push(dep.as_dev());
                    }
                }
            }
        }

        dependencies
    }

    /// Parse dependencies from Pipfile
    ///
    /// Format:
    /// [packages]
    /// django = ">=4.0"
    /// fastapi = {extras = ["all"], version = "^0.100"}
    fn parse_pipfile_dependencies(&self, content: &str) -> Vec<Dependency> {
        let mut dependencies = Vec::new();

        // Parse [packages] section
        if let Some(packages) = Self::extract_toml_section(content, "[packages]") {
            for line in packages.lines() {
                if let Some(dep) = Self::parse_pipfile_dependency(line) {
                    dependencies.push(dep);
                }
            }
        }

        // Parse [dev-packages] section
        if let Some(dev_packages) = Self::extract_toml_section(content, "[dev-packages]") {
            for line in dev_packages.lines() {
                if let Some(dep) = Self::parse_pipfile_dependency(line) {
                    dependencies.push(dep.as_dev());
                }
            }
        }

        dependencies
    }

    /// Parse requirements.txt format
    ///
    /// Supports:
    /// - Simple: django>=4.0
    /// - Extras: fastapi[all]>=0.100
    /// - Comments: # this is a comment
    /// - Editable: -e git+https://...#egg=package
    fn parse_requirements_txt(&self, content: &str) -> Vec<Dependency> {
        let mut dependencies = Vec::new();

        for line in content.lines() {
            let line = line.trim();

            // Skip comments and empty lines
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            // Skip -r, -c flags (include/constraints files)
            if line.starts_with("-r") || line.starts_with("-c") {
                continue;
            }

            // Handle editable installs: -e git+https://...#egg=package
            if line.starts_with("-e") {
                if let Some(egg_idx) = line.find("#egg=") {
                    let name = line[egg_idx + 5..].trim().to_string();
                    dependencies.push(Dependency::new(name));
                }
                continue;
            }

            // Parse normal dependency line
            if let Some(dep) = Self::parse_dependency_line(line) {
                dependencies.push(dep);
            }
        }

        dependencies
    }

    /// Parse a single dependency line
    ///
    /// Formats:
    /// - django>=4.0
    /// - fastapi[all]>=0.100
    /// - package
    pub fn parse_dependency_line(line: &str) -> Option<Dependency> {
        let line = line.trim().trim_matches(|c| c == '"' || c == '\'');

        if line.is_empty() {
            return None;
        }

        // Extract name and version: "package[extra]>=1.0"
        let (name_with_extras, version) = if let Some(op_idx) =
            line.find(|c| c == '=' || c == '>' || c == '<' || c == '~' || c == '^')
        {
            let name_part = line[..op_idx].trim();
            let version_part = line[op_idx..].trim();
            (name_part, Some(version_part.to_string()))
        } else {
            (line, None)
        };

        // Split name and extras: "package[extra1,extra2]"
        let (name, extras) = if let Some(bracket_idx) = name_with_extras.find('[') {
            let name = name_with_extras[..bracket_idx].trim().to_string();
            let extras_str = &name_with_extras[bracket_idx + 1..];
            let extras_end = extras_str.find(']')?;
            let extras: Vec<String> = extras_str[..extras_end]
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect();
            (name, extras)
        } else {
            (name_with_extras.to_string(), Vec::new())
        };

        if name.is_empty() {
            return None;
        }

        let mut dep = Dependency::new(name);
        if let Some(v) = version {
            dep = dep.with_version(v);
        }
        if !extras.is_empty() {
            dep = dep.with_extras(extras);
        }

        Some(dep)
    }

    /// Parse Poetry dependency format: django = "^4.0"
    fn parse_poetry_dependency(line: &str) -> Option<Dependency> {
        let line = line.trim();
        if line.is_empty() {
            return None;
        }

        let parts: Vec<&str> = line.split('=').collect();
        if parts.len() < 2 {
            return None;
        }

        let name = parts[0].trim().to_string();
        let version_part = parts[1].trim().trim_matches(|c| c == '"' || c == '\'');

        // Skip python version requirement
        if name == "python" {
            return None;
        }

        if name.is_empty() {
            return None;
        }

        // Handle inline table: {version = "^1.0", extras = ["all"]}
        if version_part.starts_with('{') {
            // Simple parsing - look for version field
            if let Some(version_start) = version_part.find("version") {
                let version_str = &version_part[version_start..];
                if let Some(quote_start) = version_str.find('"') {
                    let quote_start = version_start + quote_start + 1;
                    if let Some(quote_end) = version_part[quote_start..].find('"') {
                        let version =
                            version_part[quote_start..quote_start + quote_end].to_string();
                        return Some(Dependency::new(name).with_version(version));
                    }
                }
            }
            // No version found in inline table
            return Some(Dependency::new(name));
        }

        Some(Dependency::new(name).with_version(version_part.to_string()))
    }

    /// Parse Pipfile dependency format
    fn parse_pipfile_dependency(line: &str) -> Option<Dependency> {
        // For now, same as Poetry format
        Self::parse_poetry_dependency(line)
    }

    /// Extract array section from TOML: dependencies = [...]
    ///
    /// Handles multi-line arrays:
    /// dependencies = [
    ///     "django>=4.0",
    ///     "fastapi[all]",
    /// ]
    fn extract_array_section(content: &str, key: &str) -> Option<String> {
        let pattern = format!("{} = [", key);
        if let Some(start_idx) = content.find(&pattern) {
            let start = start_idx + pattern.len();

            // Find matching closing bracket, accounting for nested brackets
            let mut depth = 1;
            let mut end = start;
            let chars: Vec<char> = content[start..].chars().collect();

            for (i, ch) in chars.iter().enumerate() {
                match ch {
                    '[' => depth += 1,
                    ']' => {
                        depth -= 1;
                        if depth == 0 {
                            end = start + i;
                            break;
                        }
                    }
                    _ => {}
                }
            }

            if depth == 0 {
                return Some(content[start..end].to_string());
            }
        }
        None
    }

    /// Extract a TOML section by header: [section.name]
    fn extract_toml_section(content: &str, header: &str) -> Option<String> {
        if let Some(start) = content.find(header) {
            let start = start + header.len();
            // Find next section or end of file
            if let Some(end) = content[start..].find("\n[") {
                return Some(content[start..start + end].to_string());
            } else {
                return Some(content[start..].to_string());
            }
        }
        None
    }

    /// Find virtual environment directory
    ///
    /// Checks:
    /// 1. Common venv names: .venv, venv, .virtualenv, env
    /// 2. VIRTUAL_ENV environment variable
    fn find_venv(&self) -> Option<PathBuf> {
        // Check common venv names
        let venv_names = vec![".venv", "venv", ".virtualenv", "env"];

        for name in venv_names {
            let venv_path = self.root.join(name);
            if venv_path.exists() && venv_path.is_dir() {
                // Verify it's a valid venv (has bin/python or Scripts/python.exe)
                let bin_path = venv_path.join("bin/python");
                let scripts_path = venv_path.join("Scripts/python.exe");

                if bin_path.exists() || scripts_path.exists() {
                    return Some(venv_path);
                }
            }
        }

        // Check VIRTUAL_ENV environment variable
        if let Ok(venv_path) = std::env::var("VIRTUAL_ENV") {
            let path = PathBuf::from(venv_path);
            if path.exists() {
                return Some(path);
            }
        }

        None
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_package_manager_display_name() {
        assert_eq!(PackageManager::Uv.display_name(), "uv");
        assert_eq!(PackageManager::Poetry.display_name(), "Poetry");
        assert_eq!(PackageManager::Pipenv.display_name(), "Pipenv");
        assert_eq!(PackageManager::Pip.display_name(), "pip");
    }

    #[test]
    fn test_parse_simple_dependency() {
        let dep = PackageManagerDetector::parse_dependency_line("django>=4.0").unwrap();
        assert_eq!(dep.name, "django");
        assert_eq!(dep.version, Some(">=4.0".to_string()));
        assert!(dep.extras.is_empty());
    }

    #[test]
    fn test_parse_dependency_with_extras() {
        let dep = PackageManagerDetector::parse_dependency_line("fastapi[all]>=0.100").unwrap();
        assert_eq!(dep.name, "fastapi");
        assert_eq!(dep.version, Some(">=0.100".to_string()));
        assert_eq!(dep.extras, vec!["all"]);
    }

    #[test]
    fn test_parse_dependency_multiple_extras() {
        let dep = PackageManagerDetector::parse_dependency_line("package[dev,test]").unwrap();
        assert_eq!(dep.name, "package");
        assert_eq!(dep.extras, vec!["dev", "test"]);
    }

    #[test]
    fn test_parse_dependency_no_version() {
        let dep = PackageManagerDetector::parse_dependency_line("requests").unwrap();
        assert_eq!(dep.name, "requests");
        assert_eq!(dep.version, None);
    }

    #[test]
    fn test_framework_detection() {
        let django = Dependency::new("django".to_string());
        assert!(django.is_framework());

        let fastapi = Dependency::new("fastapi".to_string());
        assert!(fastapi.is_framework());

        let requests = Dependency::new("requests".to_string());
        assert!(!requests.is_framework());
    }

    #[test]
    fn test_dependency_builder() {
        let dep = Dependency::new("django".to_string())
            .with_version(">=4.0".to_string())
            .with_extras(vec!["postgres".to_string()])
            .as_dev();

        assert_eq!(dep.name, "django");
        assert_eq!(dep.version, Some(">=4.0".to_string()));
        assert_eq!(dep.extras, vec!["postgres"]);
        assert!(dep.is_dev);
    }

    #[test]
    fn test_parse_poetry_dependency() {
        let dep = PackageManagerDetector::parse_poetry_dependency("django = \"^4.0\"").unwrap();
        assert_eq!(dep.name, "django");
        assert_eq!(dep.version, Some("^4.0".to_string()));
    }

    #[test]
    fn test_parse_poetry_dependency_skip_python() {
        let dep = PackageManagerDetector::parse_poetry_dependency("python = \"^3.10\"");
        assert!(dep.is_none());
    }

    #[test]
    fn test_detection_unknown() {
        let detection = PackageManagerDetection::unknown();
        assert_eq!(detection.manager, PackageManager::Unknown);
        assert_eq!(detection.confidence, 0.0);
        assert!(detection.dependencies.is_empty());
    }

    #[test]
    fn test_has_dependency() {
        let mut detection = PackageManagerDetection::unknown();
        detection
            .dependencies
            .push(Dependency::new("django".to_string()));

        assert!(detection.has_dependency("django"));
        assert!(!detection.has_dependency("flask"));
    }

    #[test]
    fn test_framework_dependencies() {
        let mut detection = PackageManagerDetection::unknown();
        detection
            .dependencies
            .push(Dependency::new("django".to_string()));
        detection
            .dependencies
            .push(Dependency::new("requests".to_string()));
        detection
            .dependencies
            .push(Dependency::new("fastapi".to_string()));

        let frameworks = detection.framework_dependencies();
        assert_eq!(frameworks.len(), 2);
        assert!(frameworks.iter().any(|d| d.name == "django"));
        assert!(frameworks.iter().any(|d| d.name == "fastapi"));
    }
}
