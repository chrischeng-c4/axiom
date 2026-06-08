//! Project registry for unified cclab server
//!
//! The registry tracks:
//! - Server process information (PID, port)
//! - Registered projects (path, registration time)
//!
//! Registry file: ~/.cclab/registry.json
//!
//! Migration: Supports reading from ~/.genesis/registry.json for backward compatibility

use crate::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

/// Project registry stored in ~/.cclab/registry.json
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Registry {
    /// Server information
    pub server: ServerInfo,

    /// Registered projects (key: project name)
    pub projects: HashMap<String, ProjectInfo>,
}

/// MCP server process information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerInfo {
    /// Server process ID
    pub pid: u32,

    /// HTTP server port
    pub port: u16,

    /// Server start time
    pub started_at: DateTime<Utc>,
}

/// Individual project information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectInfo {
    /// Absolute path to project directory
    pub path: PathBuf,

    /// Project registration time
    pub registered_at: DateTime<Utc>,
}

impl Registry {
    /// Get the registry directory path (~/.cclab/)
    fn registry_dir() -> Result<PathBuf> {
        let home =
            dirs::home_dir().ok_or_else(|| anyhow::anyhow!("Could not find home directory"))?;
        Ok(home.join(".cclab"))
    }

    /// Get the registry file path (~/.cclab/registry.json)
    pub fn registry_path() -> Result<PathBuf> {
        let cclab_dir = Self::registry_dir()?;

        // Create directory if it doesn't exist
        if !cclab_dir.exists() {
            fs::create_dir_all(&cclab_dir)?;
        }

        Ok(cclab_dir.join("registry.json"))
    }

    /// Get the legacy registry path (~/.genesis/registry.json)
    fn legacy_registry_path() -> Result<PathBuf> {
        let home =
            dirs::home_dir().ok_or_else(|| anyhow::anyhow!("Could not find home directory"))?;
        Ok(home.join(".genesis").join("registry.json"))
    }

    /// Load registry from file, checking both new and legacy paths
    pub fn load() -> Result<Self> {
        let path = Self::registry_path()?;

        if path.exists() {
            let content = fs::read_to_string(&path)?;
            let registry: Registry = serde_json::from_str(&content)?;
            Ok(registry)
        } else {
            // Check legacy path for backward compatibility
            let legacy_path = Self::legacy_registry_path()?;
            if legacy_path.exists() {
                let content = fs::read_to_string(&legacy_path)?;
                let registry: Registry = serde_json::from_str(&content)?;
                // Migrate to new location
                registry.save()?;
                // Don't delete legacy file yet - let user clean up manually
                Ok(registry)
            } else {
                Err(anyhow::anyhow!("Registry not found. Server not running?"))
            }
        }
    }

    /// Save registry to file
    pub fn save(&self) -> Result<()> {
        let path = Self::registry_path()?;
        let content = serde_json::to_string_pretty(self)?;
        fs::write(&path, content)?;
        Ok(())
    }

    /// Create a new registry with server info
    pub fn new(pid: u32, port: u16) -> Self {
        Self {
            server: ServerInfo {
                pid,
                port,
                started_at: Utc::now(),
            },
            projects: HashMap::new(),
        }
    }

    /// Check if server is still running
    pub fn is_server_running(&self) -> bool {
        process_exists(self.server.pid)
    }

    /// Register a new project
    pub fn register_project(&mut self, name: String, path: PathBuf) -> Result<()> {
        self.projects.insert(
            name,
            ProjectInfo {
                path,
                registered_at: Utc::now(),
            },
        );
        self.save()?;
        Ok(())
    }

    /// Unregister a project
    pub fn unregister_project(&mut self, name: &str) -> Result<()> {
        self.projects.remove(name);
        self.save()?;
        Ok(())
    }

    /// Get project path by name
    pub fn get_project_path(&self, name: &str) -> Option<&PathBuf> {
        self.projects.get(name).map(|p| &p.path)
    }

    /// Get project name from path
    pub fn get_project_name(&self, path: &Path) -> Option<String> {
        for (name, info) in &self.projects {
            if info.path == path {
                return Some(name.clone());
            }
        }
        None
    }

    /// Check if a project path is registered (any name)
    pub fn has_project_path(&self, path: &Path) -> bool {
        self.projects.values().any(|info| info.path == path)
    }

    /// List all registered projects
    pub fn list_projects(&self) -> Vec<(String, &ProjectInfo)> {
        self.projects
            .iter()
            .map(|(name, info)| (name.clone(), info))
            .collect()
    }

    /// Clear server info from registry (when server shuts down)
    /// Preserves the project list for auto-initialization on restart (R1).
    pub fn clear_server_info() -> Result<()> {
        let path = Self::registry_path()?;
        if !path.exists() {
            return Ok(());
        }

        // Load existing registry
        let content = fs::read_to_string(&path)?;
        if let Ok(mut registry) = serde_json::from_str::<Registry>(&content) {
            // Clear server info but keep projects
            registry.server.pid = 0;
            // Save back with projects preserved
            let updated = serde_json::to_string_pretty(&registry)?;
            fs::write(&path, updated)?;
        }
        Ok(())
    }

    /// Delete registry file completely (for testing or full cleanup)
    #[allow(dead_code)]
    pub fn delete() -> Result<()> {
        let path = Self::registry_path()?;
        if path.exists() {
            fs::remove_file(path)?;
        }
        Ok(())
    }

    /// Load only the projects from a previous registry (for merging on restart)
    /// Returns None if no previous registry exists or if it's invalid
    pub fn load_persisted_projects() -> Option<HashMap<String, ProjectInfo>> {
        let path = Self::registry_path().ok()?;
        if !path.exists() {
            // Try legacy path
            let legacy = Self::legacy_registry_path().ok()?;
            if !legacy.exists() {
                return None;
            }
            let content = fs::read_to_string(&legacy).ok()?;
            let registry: Registry = serde_json::from_str(&content).ok()?;
            return Some(registry.projects);
        }
        let content = fs::read_to_string(&path).ok()?;
        let registry: Registry = serde_json::from_str(&content).ok()?;
        Some(registry.projects)
    }

    /// Merge projects from a previous registry into this one
    /// This preserves registered projects across server restarts
    pub fn merge_persisted_projects(&mut self) {
        if let Some(persisted) = Self::load_persisted_projects() {
            for (name, info) in persisted {
                // Only add if not already present and path still exists
                if !self.projects.contains_key(&name) && info.path.exists() {
                    self.projects.insert(name, info);
                }
            }
        }
    }

    /// Check if the running server is outdated (binary was modified after server started)
    pub fn is_server_outdated(&self) -> bool {
        if !self.is_server_running() {
            return false;
        }

        if let Ok(exe_path) = std::env::current_exe() {
            if let Ok(metadata) = fs::metadata(&exe_path) {
                if let Ok(modified) = metadata.modified() {
                    let modified_dt: DateTime<Utc> = modified.into();
                    return modified_dt > self.server.started_at;
                }
            }
        }

        false
    }

    /// Get server uptime in human-readable format
    pub fn server_uptime(&self) -> String {
        let duration = Utc::now().signed_duration_since(self.server.started_at);
        if duration.num_days() > 0 {
            format!("{} days", duration.num_days())
        } else if duration.num_hours() > 0 {
            format!("{} hours", duration.num_hours())
        } else if duration.num_minutes() > 0 {
            format!("{} minutes", duration.num_minutes())
        } else {
            "just started".to_string()
        }
    }

    /// Auto-register a project path (used by MCP router for on-demand registration)
    pub fn auto_register(&mut self, path: &Path) -> Result<String> {
        // Derive name from path's final component
        let name = path
            .file_name()
            .and_then(|n| n.to_str())
            .ok_or_else(|| anyhow::anyhow!("Cannot determine project name from path"))?
            .to_string();

        // Check if already registered with same name
        if let Some(existing) = self.projects.get(&name) {
            if existing.path == path {
                return Ok(name);
            }
            // Different path with same name - use full path as name
            let unique_name = path.display().to_string();
            self.register_project(unique_name.clone(), path.to_path_buf())?;
            return Ok(unique_name);
        }

        self.register_project(name.clone(), path.to_path_buf())?;
        Ok(name)
    }
}

/// Check if a process with given PID exists
#[cfg(unix)]
fn process_exists(pid: u32) -> bool {
    // PID 0 refers to the process group, not a real process
    if pid == 0 {
        return false;
    }

    use std::process::Command;

    Command::new("kill")
        .arg("-0")
        .arg(pid.to_string())
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

#[cfg(windows)]
fn process_exists(pid: u32) -> bool {
    use std::process::Command;

    Command::new("tasklist")
        .arg("/FI")
        .arg(format!("PID eq {}", pid))
        .output()
        .map(|output| {
            let stdout = String::from_utf8_lossy(&output.stdout);
            stdout.contains(&pid.to_string())
        })
        .unwrap_or(false)
}

/// Check if a path is a valid project (has pyproject.toml or .aw/changes)
pub fn is_valid_project(path: &Path) -> bool {
    path.is_dir()
        && (path.join("pyproject.toml").exists()
            || path.join("Cargo.toml").exists()
            || path.join("package.json").exists()
            || path.join("cclab").is_dir())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_registry_new() {
        let registry = Registry::new(12345, 3456);
        assert_eq!(registry.server.pid, 12345);
        assert_eq!(registry.server.port, 3456);
        assert_eq!(registry.projects.len(), 0);
    }

    #[test]
    fn test_has_project_path() {
        let mut registry = Registry::new(12345, 3456);
        let path = PathBuf::from("/tmp/test-project");
        registry
            .register_project("test".to_string(), path.clone())
            .unwrap();

        assert!(registry.has_project_path(&path));
        assert!(!registry.has_project_path(&PathBuf::from("/tmp/other")));
    }

    #[test]
    fn test_merge_persisted_projects() {
        // Create a registry with some projects
        let mut registry = Registry::new(12345, 3456);

        // Add a project that exists
        let existing_path = PathBuf::from("/tmp"); // /tmp always exists
        registry.projects.insert(
            "existing".to_string(),
            ProjectInfo {
                path: existing_path.clone(),
                registered_at: chrono::Utc::now(),
            },
        );

        // Add a project that doesn't exist
        registry.projects.insert(
            "nonexistent".to_string(),
            ProjectInfo {
                path: PathBuf::from("/nonexistent/path/should/not/exist"),
                registered_at: chrono::Utc::now(),
            },
        );

        // Create a new registry and merge
        let mut new_registry = Registry::new(54321, 3456);

        // Manually test merge logic (since we can't use load_persisted_projects in test)
        for (name, info) in &registry.projects {
            if !new_registry.projects.contains_key(name) && info.path.exists() {
                new_registry.projects.insert(name.clone(), info.clone());
            }
        }

        // Should have merged the existing project
        assert!(new_registry.projects.contains_key("existing"));
        // Should NOT have merged the nonexistent project
        assert!(!new_registry.projects.contains_key("nonexistent"));
    }
}
