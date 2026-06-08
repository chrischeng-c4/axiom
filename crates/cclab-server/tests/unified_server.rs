//! E2E integration tests for unified server architecture

use std::path::PathBuf;
use std::sync::Arc;

use cclab_server::LensHandlerPool;

#[tokio::test]
async fn test_lens_pool_creation() {
    let pool = LensHandlerPool::new();
    assert_eq!(pool.handler_count().await, 0);
}

#[tokio::test]
async fn test_pool_get_handler_creates_new() {
    let pool = LensHandlerPool::new();
    // Use a path that exists to avoid issues - current directory canonicalizes to same path
    let path = std::env::current_dir().expect("Failed to get current dir");

    // First access should create handler
    let handler = pool
        .get_handler(&path)
        .await
        .expect("Failed to get handler");
    assert_eq!(pool.handler_count().await, 1);

    // Verify handler is Arc<RequestHandler>
    // After first access and storage in pool, strong count should be 1
    // (one in pool, none held by us - we're accessing it through .get_handler return)
    assert!(Arc::strong_count(&handler) >= 1);
}

#[tokio::test]
async fn test_pool_returns_same_instance() {
    let pool = LensHandlerPool::new();
    let path = std::env::current_dir().expect("Failed to get current dir");

    let handler1 = pool
        .get_handler(&path)
        .await
        .expect("Failed to get handler 1");
    let handler2 = pool
        .get_handler(&path)
        .await
        .expect("Failed to get handler 2");

    // Should return same Arc
    assert!(Arc::ptr_eq(&handler1, &handler2));
    assert_eq!(pool.handler_count().await, 1);
}

#[tokio::test]
async fn test_pool_different_projects() {
    let pool = LensHandlerPool::new();
    // Create two distinct paths
    let path1 = PathBuf::from("/tmp/project-1");
    let path2 = PathBuf::from("/tmp/project-2");

    let _handler1 = pool
        .get_handler(&path1)
        .await
        .expect("Failed to get handler 1");
    let _handler2 = pool
        .get_handler(&path2)
        .await
        .expect("Failed to get handler 2");

    // Should create separate handlers for different paths
    assert_eq!(pool.handler_count().await, 2);
}

#[tokio::test]
async fn test_pool_remove_handler() {
    let pool = LensHandlerPool::new();
    let path = PathBuf::from("/tmp/project-remove-test");

    let _handler = pool
        .get_handler(&path)
        .await
        .expect("Failed to get handler");
    assert!(pool.has_handler(&path).await);

    pool.remove_handler(&path).await;
    assert!(!pool.has_handler(&path).await);
    assert_eq!(pool.handler_count().await, 0);
}

#[tokio::test]
async fn test_pool_list_projects() {
    let pool = LensHandlerPool::new();
    let path1 = PathBuf::from("/tmp/project-list-1");
    let path2 = PathBuf::from("/tmp/project-list-2");

    let _handler1 = pool
        .get_handler(&path1)
        .await
        .expect("Failed to get handler 1");
    let _handler2 = pool
        .get_handler(&path2)
        .await
        .expect("Failed to get handler 2");

    let projects = pool.list_projects().await;
    assert_eq!(projects.len(), 2);
}

#[tokio::test]
async fn test_acceptance_unified_architecture() {
    // Test that we can:
    // 1. Create a pool
    // 2. Get handlers for multiple projects
    // 3. Verify separation of concerns

    let pool = Arc::new(LensHandlerPool::new());

    // Get handlers for two projects
    let project1_path = PathBuf::from("/tmp/acceptance-project-1");
    let project2_path = PathBuf::from("/tmp/acceptance-project-2");

    let handler1 = pool
        .get_handler(&project1_path)
        .await
        .expect("Failed to get project1 handler");
    let handler2 = pool
        .get_handler(&project2_path)
        .await
        .expect("Failed to get project2 handler");

    // Verify they are different instances
    assert!(!Arc::ptr_eq(&handler1, &handler2));

    // Verify pool tracks both
    assert_eq!(pool.handler_count().await, 2);
}

#[tokio::test]
async fn test_document_override_integration() {
    // Test that handlers support document overrides for LSP unsaved changes
    let pool = LensHandlerPool::new();
    let project_path = PathBuf::from("/tmp/override-test-project");

    let handler = pool
        .get_handler(&project_path)
        .await
        .expect("Failed to get handler");

    // Set a document override
    let file_path = PathBuf::from("test.py");
    let content = "def hello(): pass";
    handler
        .set_document_override(&file_path, content.to_string())
        .await;

    // Retrieve the override
    let retrieved = handler
        .get_document_content(&file_path)
        .await
        .expect("Failed to get document");
    assert_eq!(retrieved, content);

    // Remove the override
    handler.remove_document_override(&file_path).await;
    let result = handler.get_document_content(&file_path).await;
    assert!(
        result.is_err(),
        "Document should not be found after removal"
    );
}
