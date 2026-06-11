//! Persistent index storage path resolution.
//!
//! Resolves the persistent storage directory for Lens code indexes at
//! `{project_dir}/cclab/.index/`. Indexes are stored locally within each
//! project for portability and easy cleanup.

use std::path::{Path, PathBuf};

/// Resolve the persistent Lens storage root for a project.
///
/// Returns `{project_root}/cclab/.index/`.
pub fn resolve_lens_storage(project_root: &Path) -> std::io::Result<PathBuf> {
    let canonical = project_root.canonicalize()?;
    Ok(canonical.join("cclab").join(".index"))
}

/// Resolve a module-specific index path within the Lens storage directory.
///
/// Returns `{project_root}/cclab/.index/{module_name}.idx`
pub fn resolve_module_index(project_root: &Path, module_name: &str) -> std::io::Result<PathBuf> {
    let root = resolve_lens_storage(project_root)?;
    Ok(root.join(format!("{}.idx", module_name)))
}

/// Resolve the PID file path for the daemon.
///
/// Returns `{project_root}/cclab/.index/daemon.pid`
pub fn resolve_pid_file(project_root: &Path) -> std::io::Result<PathBuf> {
    let root = resolve_lens_storage(project_root)?;
    Ok(root.join("daemon.pid"))
}

/// Resolve the socket path for the daemon.
///
/// Returns `{project_root}/cclab/.index/daemon.sock`
pub fn resolve_socket_path(project_root: &Path) -> std::io::Result<PathBuf> {
    let root = resolve_lens_storage(project_root)?;
    Ok(root.join("daemon.sock"))
}

/// Resolve the persistent AST index cache directory.
///
/// Returns `{project_root}/cclab/.index/cache/`
pub fn resolve_cache_dir(project_root: &Path) -> std::io::Result<PathBuf> {
    let root = resolve_lens_storage(project_root)?;
    Ok(root.join("cache"))
}

/// Resolve per-scope cache directory (#1127).
///
/// Returns `{project_root}/cclab/.index/scopes/{scope_id}/cache/`
pub fn resolve_scope_cache_dir(project_root: &Path, scope_id: &str) -> std::io::Result<PathBuf> {
    let root = resolve_lens_storage(project_root)?;
    Ok(root.join("scopes").join(scope_id).join("cache"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_resolve_lens_storage() {
        let temp = TempDir::new().unwrap();
        let path = resolve_lens_storage(temp.path()).unwrap();

        assert!(path.to_string_lossy().ends_with("cclab/.index"));
        // Use canonical path for comparison to handle macOS /var -> /private/var symlinks
        let canonical_temp = temp.path().canonicalize().unwrap();
        assert!(path.starts_with(&canonical_temp));
    }

    #[test]
    fn test_resolve_same_path_gives_same_result() {
        let temp = TempDir::new().unwrap();
        let path1 = resolve_lens_storage(temp.path()).unwrap();
        let path2 = resolve_lens_storage(temp.path()).unwrap();
        assert_eq!(path1, path2);
    }

    #[test]
    fn test_resolve_different_paths_give_different_results() {
        let temp1 = TempDir::new().unwrap();
        let temp2 = TempDir::new().unwrap();
        let path1 = resolve_lens_storage(temp1.path()).unwrap();
        let path2 = resolve_lens_storage(temp2.path()).unwrap();
        assert_ne!(path1, path2);
    }

    #[test]
    fn test_resolve_module_index() {
        let temp = TempDir::new().unwrap();
        let path = resolve_module_index(temp.path(), "backend").unwrap();

        assert!(path.to_string_lossy().ends_with("cclab/.index/backend.idx"));
    }

    #[test]
    fn test_resolve_pid_file() {
        let temp = TempDir::new().unwrap();
        let path = resolve_pid_file(temp.path()).unwrap();

        assert!(path.to_string_lossy().ends_with("cclab/.index/daemon.pid"));
    }

    #[test]
    fn test_resolve_socket_path() {
        let temp = TempDir::new().unwrap();
        let path = resolve_socket_path(temp.path()).unwrap();

        assert!(path.to_string_lossy().ends_with("cclab/.index/daemon.sock"));
    }

    #[test]
    fn test_resolve_cache_dir() {
        let temp = TempDir::new().unwrap();
        let path = resolve_cache_dir(temp.path()).unwrap();

        assert!(path.to_string_lossy().ends_with("cclab/.index/cache"));
    }
}
