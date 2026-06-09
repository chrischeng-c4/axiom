//! Python environment detection and configuration
//!
//! This module provides automatic detection of Python virtual environments
//! and configuration of search paths for module resolution.
//!
//! ## Detection Priority
//! 1. Explicit configuration in `pyproject.toml` (`[tool.cclab_lens.python]`)
//! 2. `PYTHONPATH` environment variable
//! 3. Automatic detection of local virtual environments
//! 4. System Python interpreter paths

use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use super::config::ArgusConfig;

/// Type of virtual environment detected
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum VenvType {
    /// Standard venv created with `python -m venv`
    Venv,
    /// Poetry managed environment
    Poetry,
    /// Pipenv managed environment
    Pipenv,
    /// Conda environment
    Conda,
    /// Unknown or custom environment
    Unknown,
}

impl std::fmt::Display for VenvType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            VenvType::Venv => write!(f, "venv"),
            VenvType::Poetry => write!(f, "poetry"),
            VenvType::Pipenv => write!(f, "pipenv"),
            VenvType::Conda => write!(f, "conda"),
            VenvType::Unknown => write!(f, "unknown"),
        }
    }
}

/// Information about a detected virtual environment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectedEnv {
    /// Path to the virtual environment
    pub path: PathBuf,
    /// Type of virtual environment
    pub env_type: VenvType,
    /// Path to the site-packages directory (if found)
    pub site_packages: Option<PathBuf>,
}

/// Comprehensive environment information for Python module resolution
#[derive(Debug, Clone, Default)]
pub struct EnvInfo {
    /// Active virtual environment (from config or auto-detected)
    pub active_venv: Option<DetectedEnv>,
    /// All detected virtual environments in the project
    pub detected_envs: Vec<DetectedEnv>,
    /// Combined search paths in priority order
    pub search_paths: Vec<PathBuf>,
    /// Python version (e.g., "3.11")
    pub python_version: Option<String>,
}

/// Detect Python environment for a project
///
/// This function implements the configuration priority:
/// 1. Explicit `[tool.cclab_lens.python]` configuration
/// 2. `PYTHONPATH` environment variable
/// 3. Auto-detected virtual environments
pub fn detect_python_environment(project_root: &Path) -> EnvInfo {
    let config = ArgusConfig::from_pyproject(project_root);
    detect_with_config(project_root, &config)
}

/// Detect Python environment with a pre-loaded configuration
pub fn detect_with_config(project_root: &Path, config: &ArgusConfig) -> EnvInfo {
    let mut info = EnvInfo {
        python_version: config.python_version.clone(),
        ..Default::default()
    };

    // Detect all virtual environments first
    info.detected_envs = detect_all_venvs(project_root);

    // Priority 1: Explicit venv_path configuration
    if let Some(ref venv_path) = config.python.venv_path {
        let venv_abs = if venv_path.is_absolute() {
            venv_path.clone()
        } else {
            project_root.join(venv_path)
        };

        if venv_abs.exists() {
            let site_packages = find_site_packages(&venv_abs, config.python_version.as_deref());
            // Try to determine the venv type by checking if it matches any detected env
            let env_type = info
                .detected_envs
                .iter()
                .find(|e| e.path == venv_abs)
                .map(|e| e.env_type.clone())
                .unwrap_or(VenvType::Venv); // Default to Venv if not in detected list

            info.active_venv = Some(DetectedEnv {
                path: venv_abs,
                env_type,
                site_packages,
            });
        }
    }

    // If no explicit config, use first detected env
    if info.active_venv.is_none() && !info.detected_envs.is_empty() {
        info.active_venv = Some(info.detected_envs[0].clone());
    }

    // Build search paths in priority order
    let mut search_paths = Vec::new();

    // 1. Explicit search_paths from config
    for path in &config.python.search_paths {
        let abs_path = if path.is_absolute() {
            path.clone()
        } else {
            project_root.join(path)
        };
        if abs_path.exists() {
            search_paths.push(abs_path);
        }
    }

    // 2. PYTHONPATH environment variable
    if let Ok(pythonpath) = env::var("PYTHONPATH") {
        for path_str in pythonpath.split(':') {
            let path = PathBuf::from(path_str);
            if path.exists() && !search_paths.contains(&path) {
                search_paths.push(path);
            }
        }
    }

    // 3. Project root
    if !search_paths.contains(&project_root.to_path_buf()) {
        search_paths.push(project_root.to_path_buf());
    }

    // 4. Site-packages from active venv (unless ignored)
    if !config.python.ignore_site_packages {
        if let Some(ref venv) = info.active_venv {
            if let Some(ref site_packages) = venv.site_packages {
                if !search_paths.contains(site_packages) {
                    search_paths.push(site_packages.clone());
                }
            }
        }
    }

    info.search_paths = search_paths;
    info
}

/// Detect all virtual environments in a project directory
pub fn detect_all_venvs(project_root: &Path) -> Vec<DetectedEnv> {
    let mut envs = Vec::new();

    // Check VIRTUAL_ENV environment variable first
    if let Ok(venv_path) = env::var("VIRTUAL_ENV") {
        let path = PathBuf::from(&venv_path);
        if path.exists() {
            let site_packages = find_site_packages(&path, None);
            envs.push(DetectedEnv {
                path,
                env_type: VenvType::Venv,
                site_packages,
            });
        }
    }

    // Check common venv directory names
    let common_venv_dirs = [".venv", "venv", "env", ".env"];
    for dir_name in common_venv_dirs {
        let venv_path = project_root.join(dir_name);
        if is_venv_directory(&venv_path) {
            // Avoid duplicates if already added via VIRTUAL_ENV
            if !envs.iter().any(|e| e.path == venv_path) {
                let site_packages = find_site_packages(&venv_path, None);
                envs.push(DetectedEnv {
                    path: venv_path,
                    env_type: VenvType::Venv,
                    site_packages,
                });
            }
        }
    }

    // Check for Poetry managed environment
    if project_root.join("poetry.lock").exists() {
        // Poetry stores envs in a different location, try to find it
        if let Some(poetry_env) = find_poetry_venv(project_root) {
            if !envs.iter().any(|e| e.path == poetry_env) {
                let site_packages = find_site_packages(&poetry_env, None);
                envs.push(DetectedEnv {
                    path: poetry_env,
                    env_type: VenvType::Poetry,
                    site_packages,
                });
            }
        }
    }

    // Check for Pipenv
    if project_root.join("Pipfile").exists() || project_root.join("Pipfile.lock").exists() {
        if let Some(pipenv_path) = find_pipenv_venv(project_root) {
            if !envs.iter().any(|e| e.path == pipenv_path) {
                let site_packages = find_site_packages(&pipenv_path, None);
                envs.push(DetectedEnv {
                    path: pipenv_path,
                    env_type: VenvType::Pipenv,
                    site_packages,
                });
            }
        }
    }

    envs
}

/// Check if a directory is a Python virtual environment
pub fn is_venv_directory(path: &Path) -> bool {
    if !path.is_dir() {
        return false;
    }

    // Check for pyvenv.cfg (modern venvs)
    if path.join("pyvenv.cfg").exists() {
        return true;
    }

    // Check for bin/python or Scripts/python.exe (cross-platform)
    let has_python_unix = path.join("bin").join("python").exists();
    let has_python_windows = path.join("Scripts").join("python.exe").exists();

    // Check for lib/pythonX.Y/site-packages structure
    let has_lib = path.join("lib").exists() || path.join("Lib").exists();

    (has_python_unix || has_python_windows) && has_lib
}

/// Find site-packages directory within a virtual environment
pub fn find_site_packages(venv_path: &Path, python_version: Option<&str>) -> Option<PathBuf> {
    // Try Unix-style paths first (lib/pythonX.Y/site-packages)
    let lib_dir = venv_path.join("lib");
    if lib_dir.exists() {
        // If we have a specific version, try that first
        if let Some(version) = python_version {
            let specific_path = lib_dir
                .join(format!("python{}", version))
                .join("site-packages");
            if specific_path.exists() {
                return Some(specific_path);
            }
        }

        // Otherwise, find any pythonX.Y directory
        if let Ok(entries) = fs::read_dir(&lib_dir) {
            for entry in entries.flatten() {
                let name = entry.file_name();
                let name_str = name.to_string_lossy();
                if name_str.starts_with("python") {
                    let site_packages = entry.path().join("site-packages");
                    if site_packages.exists() {
                        return Some(site_packages);
                    }
                }
            }
        }
    }

    // Try Windows-style path (Lib/site-packages)
    let windows_site_packages = venv_path.join("Lib").join("site-packages");
    if windows_site_packages.exists() {
        return Some(windows_site_packages);
    }

    None
}

/// Find Poetry's virtual environment for a project
fn find_poetry_venv(project_root: &Path) -> Option<PathBuf> {
    // Poetry creates venvs in {cache-dir}/virtualenvs/{project-name}-{hash}-py{version}
    // We can also check for in-project venv if poetry.toml has virtualenvs.in-project = true

    // First check for in-project venv
    let in_project_venv = project_root.join(".venv");
    if is_venv_directory(&in_project_venv) {
        return Some(in_project_venv);
    }

    // Try to find poetry cache directory
    let poetry_cache = get_poetry_cache_dir()?;
    let virtualenvs_dir = poetry_cache.join("virtualenvs");

    if !virtualenvs_dir.exists() {
        return None;
    }

    // Get project name from pyproject.toml or directory name
    let project_name = project_root
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown");

    // Find matching venv (poetry uses {name}-{hash}-py{version} pattern)
    if let Ok(entries) = fs::read_dir(&virtualenvs_dir) {
        for entry in entries.flatten() {
            let name = entry.file_name();
            let name_str = name.to_string_lossy();
            if name_str.starts_with(project_name) && is_venv_directory(&entry.path()) {
                return Some(entry.path());
            }
        }
    }

    None
}

/// Get Poetry's cache directory
fn get_poetry_cache_dir() -> Option<PathBuf> {
    // Check POETRY_CACHE_DIR environment variable
    if let Ok(cache_dir) = env::var("POETRY_CACHE_DIR") {
        return Some(PathBuf::from(cache_dir));
    }

    // Default locations
    #[cfg(target_os = "macos")]
    {
        if let Ok(home) = env::var("HOME") {
            return Some(PathBuf::from(home).join("Library/Caches/pypoetry"));
        }
    }

    #[cfg(target_os = "linux")]
    {
        if let Ok(xdg_cache) = env::var("XDG_CACHE_HOME") {
            return Some(PathBuf::from(xdg_cache).join("pypoetry"));
        }
        if let Ok(home) = env::var("HOME") {
            return Some(PathBuf::from(home).join(".cache/pypoetry"));
        }
    }

    #[cfg(target_os = "windows")]
    {
        if let Ok(local_app_data) = env::var("LOCALAPPDATA") {
            return Some(PathBuf::from(local_app_data).join("pypoetry/Cache"));
        }
    }

    None
}

/// Find Pipenv's virtual environment for a project
fn find_pipenv_venv(project_root: &Path) -> Option<PathBuf> {
    // Pipenv creates venvs in {WORKON_HOME}/{project-name}-{hash}
    // Default WORKON_HOME is ~/.local/share/virtualenvs on Linux/macOS

    // Check WORKON_HOME first
    let workon_home = if let Ok(home) = env::var("WORKON_HOME") {
        PathBuf::from(home)
    } else if let Ok(home) = env::var("HOME") {
        PathBuf::from(home).join(".local/share/virtualenvs")
    } else {
        return None;
    };

    if !workon_home.exists() {
        return None;
    }

    // Get project name from directory
    let project_name = project_root
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown");

    // Find matching venv
    if let Ok(entries) = fs::read_dir(&workon_home) {
        for entry in entries.flatten() {
            let name = entry.file_name();
            let name_str = name.to_string_lossy();
            if name_str.starts_with(project_name) && is_venv_directory(&entry.path()) {
                return Some(entry.path());
            }
        }
    }

    None
}

/// Get the Python version from a virtual environment
pub fn get_venv_python_version(venv_path: &Path) -> Option<String> {
    // Try to read from pyvenv.cfg
    let pyvenv_cfg = venv_path.join("pyvenv.cfg");
    if pyvenv_cfg.exists() {
        if let Ok(content) = fs::read_to_string(&pyvenv_cfg) {
            for line in content.lines() {
                if let Some(version) = line.strip_prefix("version = ") {
                    // Extract major.minor from full version (e.g., "3.11.4" -> "3.11")
                    let parts: Vec<&str> = version.trim().split('.').collect();
                    if parts.len() >= 2 {
                        return Some(format!("{}.{}", parts[0], parts[1]));
                    }
                }
            }
        }
    }

    // Fallback: try to detect from lib directory name
    let lib_dir = venv_path.join("lib");
    if let Ok(entries) = fs::read_dir(&lib_dir) {
        for entry in entries.flatten() {
            let name = entry.file_name();
            let name_str = name.to_string_lossy();
            if let Some(version) = name_str.strip_prefix("python") {
                return Some(version.to_string());
            }
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::super::config::PythonEnvConfig;
    use super::*;
    use std::fs;

    #[test]
    fn test_venv_type_display() {
        assert_eq!(VenvType::Venv.to_string(), "venv");
        assert_eq!(VenvType::Poetry.to_string(), "poetry");
        assert_eq!(VenvType::Pipenv.to_string(), "pipenv");
        assert_eq!(VenvType::Conda.to_string(), "conda");
        assert_eq!(VenvType::Unknown.to_string(), "unknown");
    }

    #[test]
    fn test_is_venv_directory_with_pyvenv_cfg() {
        let temp_dir = env::temp_dir().join("cclab_lens_test_venv_cfg");
        let _ = fs::remove_dir_all(&temp_dir);
        fs::create_dir_all(&temp_dir).unwrap();

        // Not a venv initially
        assert!(!is_venv_directory(&temp_dir));

        // Add pyvenv.cfg
        fs::write(temp_dir.join("pyvenv.cfg"), "version = 3.11.0").unwrap();
        assert!(is_venv_directory(&temp_dir));

        // Cleanup
        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_is_venv_directory_with_structure() {
        let temp_dir = env::temp_dir().join("cclab_lens_test_venv_struct");
        let _ = fs::remove_dir_all(&temp_dir);
        fs::create_dir_all(&temp_dir).unwrap();

        // Not a venv initially
        assert!(!is_venv_directory(&temp_dir));

        // Add venv structure (Unix-style)
        fs::create_dir_all(temp_dir.join("bin")).unwrap();
        fs::create_dir_all(temp_dir.join("lib/python3.11/site-packages")).unwrap();
        fs::write(temp_dir.join("bin/python"), "").unwrap();

        assert!(is_venv_directory(&temp_dir));

        // Cleanup
        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_find_site_packages() {
        let temp_dir = env::temp_dir().join("cclab_lens_test_site_packages");
        let _ = fs::remove_dir_all(&temp_dir);
        fs::create_dir_all(&temp_dir).unwrap();

        // Create Unix-style structure
        let site_packages_path = temp_dir.join("lib/python3.11/site-packages");
        fs::create_dir_all(&site_packages_path).unwrap();

        let found = find_site_packages(&temp_dir, None);
        assert!(found.is_some());
        assert!(found.unwrap().ends_with("site-packages"));

        // Test with specific version
        let found_specific = find_site_packages(&temp_dir, Some("3.11"));
        assert!(found_specific.is_some());

        // Cleanup
        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_find_site_packages_windows_style() {
        let temp_dir = env::temp_dir().join("cclab_lens_test_site_packages_win");
        let _ = fs::remove_dir_all(&temp_dir);
        fs::create_dir_all(&temp_dir).unwrap();

        // Create Windows-style structure
        let site_packages_path = temp_dir.join("Lib/site-packages");
        fs::create_dir_all(&site_packages_path).unwrap();

        let found = find_site_packages(&temp_dir, None);
        assert!(found.is_some());
        assert!(found.unwrap().ends_with("site-packages"));

        // Cleanup
        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_detect_all_venvs_common_names() {
        let temp_dir = env::temp_dir().join("cclab_lens_test_detect_venvs");
        let _ = fs::remove_dir_all(&temp_dir);
        fs::create_dir_all(&temp_dir).unwrap();

        // Create .venv directory with structure
        let venv_path = temp_dir.join(".venv");
        fs::create_dir_all(venv_path.join("bin")).unwrap();
        fs::create_dir_all(venv_path.join("lib")).unwrap();
        fs::write(venv_path.join("pyvenv.cfg"), "version = 3.11.0").unwrap();

        let envs = detect_all_venvs(&temp_dir);
        assert!(!envs.is_empty());
        assert!(envs.iter().any(|e| e.path.ends_with(".venv")));

        // Cleanup
        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_detect_python_environment_with_config() {
        let temp_dir = env::temp_dir().join("cclab_lens_test_env_config");
        let _ = fs::remove_dir_all(&temp_dir);
        fs::create_dir_all(&temp_dir).unwrap();

        // Create a venv
        let venv_path = temp_dir.join("custom_venv");
        fs::create_dir_all(venv_path.join("lib/python3.10/site-packages")).unwrap();
        fs::write(venv_path.join("pyvenv.cfg"), "version = 3.10.0").unwrap();

        // Create config with explicit venv_path
        let config = ArgusConfig {
            python_version: Some("3.10".to_string()),
            python: PythonEnvConfig {
                venv_path: Some(PathBuf::from("custom_venv")),
                ..Default::default()
            },
            ..Default::default()
        };

        let info = detect_with_config(&temp_dir, &config);
        assert!(info.active_venv.is_some());
        assert!(info
            .active_venv
            .as_ref()
            .unwrap()
            .path
            .ends_with("custom_venv"));
        assert_eq!(info.python_version, Some("3.10".to_string()));

        // Cleanup
        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_detect_python_environment_with_search_paths() {
        let temp_dir = env::temp_dir().join("cclab_lens_test_search_paths");
        let _ = fs::remove_dir_all(&temp_dir);
        fs::create_dir_all(&temp_dir).unwrap();

        // Create source directories
        fs::create_dir_all(temp_dir.join("src")).unwrap();
        fs::create_dir_all(temp_dir.join("lib")).unwrap();

        let config = ArgusConfig {
            python: PythonEnvConfig {
                search_paths: vec![PathBuf::from("src"), PathBuf::from("lib")],
                ..Default::default()
            },
            ..Default::default()
        };

        let info = detect_with_config(&temp_dir, &config);

        // Should include the configured search paths
        assert!(info.search_paths.iter().any(|p| p.ends_with("src")));
        assert!(info.search_paths.iter().any(|p| p.ends_with("lib")));

        // Cleanup
        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_get_venv_python_version_from_pyvenv_cfg() {
        let temp_dir = env::temp_dir().join("cclab_lens_test_py_version");
        let _ = fs::remove_dir_all(&temp_dir);
        fs::create_dir_all(&temp_dir).unwrap();

        fs::write(
            temp_dir.join("pyvenv.cfg"),
            "home = /usr/bin\nversion = 3.11.4\n",
        )
        .unwrap();

        let version = get_venv_python_version(&temp_dir);
        assert_eq!(version, Some("3.11".to_string()));

        // Cleanup
        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_detect_poetry_project() {
        let temp_dir = env::temp_dir().join("cclab_lens_test_poetry");
        let _ = fs::remove_dir_all(&temp_dir);
        fs::create_dir_all(&temp_dir).unwrap();

        // Create poetry.lock (marker for poetry project)
        fs::write(temp_dir.join("poetry.lock"), "[[package]]").unwrap();

        // Create in-project venv
        let venv_path = temp_dir.join(".venv");
        fs::create_dir_all(venv_path.join("lib")).unwrap();
        fs::write(venv_path.join("pyvenv.cfg"), "version = 3.11.0").unwrap();

        let envs = detect_all_venvs(&temp_dir);

        // Should find the venv (Poetry type detected when poetry.lock exists)
        assert!(!envs.is_empty());

        // Cleanup
        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_ignore_site_packages() {
        let temp_dir = env::temp_dir().join("cclab_lens_test_ignore_site");
        let _ = fs::remove_dir_all(&temp_dir);
        fs::create_dir_all(&temp_dir).unwrap();

        // Create a venv
        let venv_path = temp_dir.join(".venv");
        let site_packages = venv_path.join("lib/python3.11/site-packages");
        fs::create_dir_all(&site_packages).unwrap();
        fs::write(venv_path.join("pyvenv.cfg"), "version = 3.11.0").unwrap();

        // Config with ignore_site_packages = true
        let config = ArgusConfig {
            python: PythonEnvConfig {
                ignore_site_packages: true,
                ..Default::default()
            },
            ..Default::default()
        };

        let info = detect_with_config(&temp_dir, &config);

        // Should NOT include site-packages in search paths
        assert!(!info
            .search_paths
            .iter()
            .any(|p| p.ends_with("site-packages")));

        // Cleanup
        let _ = fs::remove_dir_all(&temp_dir);
    }

    /// Acceptance Criteria: WHEN PYTHONPATH is set THEN include in search paths
    /// Spec: python-env.md#acceptance-criteria
    #[test]
    fn test_pythonpath_in_search_paths() {
        let temp_dir = env::temp_dir().join("cclab_lens_test_pythonpath");
        let _ = fs::remove_dir_all(&temp_dir);
        fs::create_dir_all(&temp_dir).unwrap();

        // Create the directory that PYTHONPATH will point to
        let extra_lib = temp_dir.join("extra_lib");
        fs::create_dir_all(&extra_lib).unwrap();

        // We need to temporarily set PYTHONPATH for this test
        // Note: This test manipulates env vars which could affect other tests if run in parallel
        let original_pythonpath = env::var("PYTHONPATH").ok();
        env::set_var("PYTHONPATH", extra_lib.to_string_lossy().to_string());

        let config = ArgusConfig::default();
        let info = detect_with_config(&temp_dir, &config);

        // PYTHONPATH should be included in search paths
        assert!(
            info.search_paths.iter().any(|p| p == &extra_lib),
            "PYTHONPATH '{}' should be in search_paths: {:?}",
            extra_lib.display(),
            info.search_paths
        );

        // Restore original PYTHONPATH
        match original_pythonpath {
            Some(val) => env::set_var("PYTHONPATH", val),
            None => env::remove_var("PYTHONPATH"),
        }

        // Cleanup
        let _ = fs::remove_dir_all(&temp_dir);
    }

    /// Acceptance Criteria: Configuration priority test
    /// Spec: python-env.md#r1-configuration-priority
    #[test]
    fn test_config_priority_explicit_over_auto() {
        let temp_dir = env::temp_dir().join("cclab_lens_test_config_priority");
        let _ = fs::remove_dir_all(&temp_dir);
        fs::create_dir_all(&temp_dir).unwrap();

        // Create two venvs: one that will be auto-detected, one configured explicitly
        let auto_venv = temp_dir.join(".venv");
        fs::create_dir_all(auto_venv.join("lib/python3.11/site-packages")).unwrap();
        fs::write(auto_venv.join("pyvenv.cfg"), "version = 3.11.0").unwrap();

        let explicit_venv = temp_dir.join("my_custom_env");
        fs::create_dir_all(explicit_venv.join("lib/python3.10/site-packages")).unwrap();
        fs::write(explicit_venv.join("pyvenv.cfg"), "version = 3.10.0").unwrap();

        // Explicit config should take priority over auto-detected .venv
        let config = ArgusConfig {
            python: PythonEnvConfig {
                venv_path: Some(PathBuf::from("my_custom_env")),
                ..Default::default()
            },
            ..Default::default()
        };

        let info = detect_with_config(&temp_dir, &config);

        // Active venv should be the explicitly configured one
        assert!(info.active_venv.is_some());
        assert!(
            info.active_venv
                .as_ref()
                .unwrap()
                .path
                .ends_with("my_custom_env"),
            "Expected my_custom_env, got: {:?}",
            info.active_venv.as_ref().unwrap().path
        );

        // Cleanup
        let _ = fs::remove_dir_all(&temp_dir);
    }

    /// Test detecting Pipenv managed environment
    #[test]
    fn test_detect_pipenv_project() {
        let temp_dir = env::temp_dir().join("cclab_lens_test_pipenv");
        let _ = fs::remove_dir_all(&temp_dir);
        fs::create_dir_all(&temp_dir).unwrap();

        // Create Pipfile (marker for pipenv project)
        fs::write(temp_dir.join("Pipfile"), "[packages]").unwrap();

        // Create in-project venv style
        let venv_path = temp_dir.join(".venv");
        fs::create_dir_all(venv_path.join("lib")).unwrap();
        fs::write(venv_path.join("pyvenv.cfg"), "version = 3.11.0").unwrap();

        let envs = detect_all_venvs(&temp_dir);

        // Should find at least one environment
        assert!(!envs.is_empty());

        // Cleanup
        let _ = fs::remove_dir_all(&temp_dir);
    }

    /// Test that site_packages is included in active venv info
    #[test]
    fn test_active_venv_has_site_packages() {
        let temp_dir = env::temp_dir().join("cclab_lens_test_active_site_pkgs");
        let _ = fs::remove_dir_all(&temp_dir);
        fs::create_dir_all(&temp_dir).unwrap();

        // Create a venv with site-packages
        let venv_path = temp_dir.join(".venv");
        let site_packages = venv_path.join("lib/python3.11/site-packages");
        fs::create_dir_all(&site_packages).unwrap();
        fs::write(venv_path.join("pyvenv.cfg"), "version = 3.11.0").unwrap();

        // Create a dummy package in site-packages
        fs::create_dir_all(site_packages.join("requests")).unwrap();
        fs::write(site_packages.join("requests/__init__.py"), "").unwrap();

        let config = ArgusConfig::default();
        let info = detect_with_config(&temp_dir, &config);

        // Should have active venv with site-packages
        assert!(info.active_venv.is_some());
        let active = info.active_venv.as_ref().unwrap();
        assert!(active.site_packages.is_some());
        assert!(active
            .site_packages
            .as_ref()
            .unwrap()
            .ends_with("site-packages"));

        // Site-packages should be in search paths
        assert!(
            info.search_paths
                .iter()
                .any(|p| p.ends_with("site-packages")),
            "site-packages should be in search_paths: {:?}",
            info.search_paths
        );

        // Cleanup
        let _ = fs::remove_dir_all(&temp_dir);
    }
}
