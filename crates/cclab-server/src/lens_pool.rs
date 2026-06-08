//! Lens handler pool for multi-project support
//!
//! Each project gets its own RequestHandler instance that performs in-process
//! code analysis without spawning external daemons.

use agentic_workflow::models::SddConfig;
use agentic_workflow::server::RequestHandler;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::RwLock;

/// Pool of Lens handlers, one per project path
pub struct LensHandlerPool {
    handlers: Arc<RwLock<HashMap<PathBuf, Arc<RequestHandler>>>>,
}

impl Default for LensHandlerPool {
    fn default() -> Self {
        Self::new()
    }
}

impl LensHandlerPool {
    /// Create a new empty handler pool
    pub fn new() -> Self {
        Self {
            handlers: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Get or create a handler for the given project path
    ///
    /// Handlers are lazily initialized on first access.
    pub async fn get_handler(&self, project_path: &Path) -> Result<Arc<RequestHandler>, String> {
        let canonical = canonical_project_key(project_path);

        // Check cache first
        {
            let handlers = self.handlers.read().await;
            if let Some(handler) = handlers.get(&canonical) {
                return Ok(handler.clone());
            }
        }

        // Create new handler
        let handler = Arc::new(RequestHandler::new(canonical.clone())?);

        // Insert into cache
        {
            let mut handlers = self.handlers.write().await;
            handlers.insert(canonical, handler.clone());
        }

        Ok(handler)
    }

    /// Check if a handler exists for the given project path
    pub async fn has_handler(&self, project_path: &Path) -> bool {
        let canonical = canonical_project_key(project_path);
        let handlers = self.handlers.read().await;
        handlers.contains_key(&canonical)
    }

    /// Remove a handler for the given project path
    pub async fn remove_handler(&self, project_path: &Path) {
        let canonical = canonical_project_key(project_path);
        let mut handlers = self.handlers.write().await;
        handlers.remove(&canonical);
    }

    /// Get the number of active handlers
    pub async fn handler_count(&self) -> usize {
        let handlers = self.handlers.read().await;
        handlers.len()
    }

    /// List all project paths with active handlers
    pub async fn list_projects(&self) -> Vec<PathBuf> {
        let handlers = self.handlers.read().await;
        handlers.keys().cloned().collect()
    }

    /// Initialize handlers for multiple projects in background
    /// Returns immediately, initialization happens asynchronously
    pub fn initialize_projects_background(&self, paths: Vec<PathBuf>) {
        if paths.is_empty() {
            return;
        }

        // Clone Arc for the spawned task
        let handlers = Arc::clone(&self.handlers);

        tokio::spawn(async move {
            for path in paths {
                let canonical = canonical_project_key(&path);

                // Skip if already initialized
                {
                    let existing = handlers.read().await;
                    if existing.contains_key(&canonical) {
                        continue;
                    }
                }

                // Try to create handler
                match RequestHandler::new(canonical.clone()) {
                    Ok(handler) => {
                        let handler = Arc::new(handler);
                        {
                            let mut writers = handlers.write().await;
                            // Double-check under write lock to avoid race with get_handler()
                            if !writers.contains_key(&canonical) {
                                writers.insert(canonical.clone(), Arc::clone(&handler));
                                eprintln!(
                                    "  Initialized Lens handler for: {}",
                                    canonical.display()
                                );
                            }
                        }

                        // Proactively index project files
                        let dirs: Vec<PathBuf> = match SddConfig::load(&canonical) {
                            Ok(config) if !config.project.modules.is_empty() => config
                                .project
                                .modules
                                .iter()
                                .map(|m| canonical.join(&m.path))
                                .collect(),
                            _ => vec![canonical.clone()],
                        };

                        let mut total_indexed = 0usize;
                        for dir in &dirs {
                            let count = handler.index_directory(dir).await;
                            total_indexed += count;
                        }
                        eprintln!(
                            "  Lens indexed {} files for: {}",
                            total_indexed,
                            canonical.display()
                        );
                    }
                    Err(e) => {
                        eprintln!(
                            "  Failed to initialize Lens handler for {}: {}",
                            canonical.display(),
                            e
                        );
                    }
                }
            }
            eprintln!("Background Lens initialization complete");
        });
    }
}

fn canonical_project_key(project_path: &Path) -> PathBuf {
    let mut missing = Vec::new();
    let mut current = project_path;

    loop {
        if let Ok(mut canonical) = current.canonicalize() {
            for component in missing.iter().rev() {
                canonical.push(component);
            }
            return canonical;
        }

        if let Some(file_name) = current.file_name() {
            missing.push(file_name.to_owned());
        }

        let Some(parent) = current.parent() else {
            break;
        };
        current = parent;
    }

    project_path.to_path_buf()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[tokio::test]
    async fn test_pool_creation() {
        let pool = LensHandlerPool::new();
        assert_eq!(pool.handler_count().await, 0);
    }

    #[tokio::test]
    async fn test_get_handler_creates_new() {
        let pool = LensHandlerPool::new();
        let path = PathBuf::from(".");

        // First access should create handler
        let _handler = pool
            .get_handler(&path)
            .await
            .expect("Failed to get handler");
        assert!(pool.has_handler(&path).await);
        assert_eq!(pool.handler_count().await, 1);
    }

    #[tokio::test]
    async fn test_get_handler_returns_cached() {
        let pool = LensHandlerPool::new();
        let path = PathBuf::from(".");

        let handler1 = pool
            .get_handler(&path)
            .await
            .expect("Failed to get handler");
        let handler2 = pool
            .get_handler(&path)
            .await
            .expect("Failed to get handler");

        // Should return same Arc
        assert!(Arc::ptr_eq(&handler1, &handler2));
        assert_eq!(pool.handler_count().await, 1);
    }

    #[tokio::test]
    async fn test_remove_handler() {
        let pool = LensHandlerPool::new();
        let path = PathBuf::from(".");

        let _handler = pool
            .get_handler(&path)
            .await
            .expect("Failed to get handler");
        assert!(pool.has_handler(&path).await);

        pool.remove_handler(&path).await;
        assert!(!pool.has_handler(&path).await);
    }
}
