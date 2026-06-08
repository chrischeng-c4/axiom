---
id: projects-meter-src-plugin-rs
fill_sections: [overview, source, changes]
capability_refs:
  - id: agent-use-first-cli
    role: primary
    gap: json-default-report-envelope-and-findings
    claim: json-default-report-envelope-and-findings
    coverage: full
    rationale: "Source template implements meter agent-facing CLI, runner, or report surfaces."
---

# Standardized projects/meter/src/plugin.rs

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/meter/src/plugin.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `AsyncHookFuture` | projects/meter/src/plugin.rs | type | pub | 47 |  |
| `FilterPlugin` | projects/meter/src/plugin.rs | struct | pub | 481 |  |
| `HookError` | projects/meter/src/plugin.rs | enum | pub | 52 |  |
| `HookResult` | projects/meter/src/plugin.rs | type | pub | 43 |  |
| `HookSpec` | projects/meter/src/plugin.rs | enum | pub | 81 |  |
| `LoggingPlugin` | projects/meter/src/plugin.rs | struct | pub | 382 |  |
| `PluginConfig` | projects/meter/src/plugin.rs | struct | pub | 184 |  |
| `PluginManager` | projects/meter/src/plugin.rs | struct | pub | 228 |  |
| `TimeoutPlugin` | projects/meter/src/plugin.rs | struct | pub | 443 |  |
| `exclude` | projects/meter/src/plugin.rs | function | pub | 497 | exclude(mut self, tag: impl Into<String>) -> Self |
| `get` | projects/meter/src/plugin.rs | function | pub | 216 | get(&self, key: &str) -> Option<&str> |
| `get` | projects/meter/src/plugin.rs | function | pub | 269 | get(&self, name: &str) -> Option<&Arc<dyn Plugin>> |
| `has` | projects/meter/src/plugin.rs | function | pub | 279 | has(&self, name: &str) -> bool |
| `hook_collection_finish` | projects/meter/src/plugin.rs | function | pub | 317 | hook_collection_finish(&self, items: &[TestMeta]) |
| `hook_collection_start` | projects/meter/src/plugin.rs | function | pub | 310 | hook_collection_start(&self) |
| `hook_configure` | projects/meter/src/plugin.rs | function | pub | 303 | hook_configure(&self, config: &mut PluginConfig) |
| `hook_error` | projects/meter/src/plugin.rs | function | pub | 359 | hook_error(&self, error: &str) |
| `hook_modify_items` | projects/meter/src/plugin.rs | function | pub | 366 | hook_modify_items(&self, items: Vec<TestMeta>) -> Vec<TestMeta> |
| `hook_session_finish` | projects/meter/src/plugin.rs | function | pub | 331 | hook_session_finish(&self, summary: &TestSummary) |
| `hook_session_start` | projects/meter/src/plugin.rs | function | pub | 324 | hook_session_start(&self) |
| `hook_test_finish` | projects/meter/src/plugin.rs | function | pub | 345 | hook_test_finish(&self, test: &TestMeta, result: &TestResult) |
| `hook_test_skipped` | projects/meter/src/plugin.rs | function | pub | 352 | hook_test_skipped(&self, test: &TestMeta, reason: &str) |
| `hook_test_start` | projects/meter/src/plugin.rs | function | pub | 338 | hook_test_start(&self, test: &TestMeta) |
| `include` | projects/meter/src/plugin.rs | function | pub | 492 | include(mut self, tag: impl Into<String>) -> Self |
| `is_empty` | projects/meter/src/plugin.rs | function | pub | 289 | is_empty(&self) -> bool |
| `len` | projects/meter/src/plugin.rs | function | pub | 284 | len(&self) -> usize |
| `names` | projects/meter/src/plugin.rs | function | pub | 274 | names(&self) -> Vec<&'static str> |
| `new` | projects/meter/src/plugin.rs | function | pub | 200 | new() -> Self |
| `new` | projects/meter/src/plugin.rs | function | pub | 235 | new() -> Self |
| `new` | projects/meter/src/plugin.rs | function | pub | 388 | new(verbose: bool) -> Self |
| `new` | projects/meter/src/plugin.rs | function | pub | 449 | new(default_timeout: f64) -> Self |
| `new` | projects/meter/src/plugin.rs | function | pub | 488 | new() -> Self |
| `register` | projects/meter/src/plugin.rs | function | pub | 242 | register(&mut self, plugin: Arc<dyn Plugin>) -> HookResult<()> |
| `set` | projects/meter/src/plugin.rs | function | pub | 211 | set(&mut self, key: impl Into<String>, value: impl Into<String>) |
| `unregister` | projects/meter/src/plugin.rs | function | pub | 258 | unregister(&mut self, name: &str) -> HookResult<()> |
## Source
<!-- type: source lang: rust -->
<!-- source-from-target: strip-managed-markers -->

<!-- source-snapshot: path=projects/meter/src/plugin.rs -->
````rust
//! Plugin system for cclab-probe
//!
//! Provides a hook-based plugin architecture inspired by pytest's pluggy system.
//! Supports both synchronous and asynchronous hooks with priority ordering.
//!
//! # Example
//!
//! ```ignore
//! use meter::plugin::{Plugin, PluginManager, HookSpec};
//!
//! struct MyPlugin;
//!
//! impl Plugin for MyPlugin {
//!     fn name(&self) -> &'static str { "my-plugin" }
//!
//!     fn probe_configure(&self, config: &mut RunnerConfig) {
//!         config.parallel = true;
//!     }
//! }
//!
//! let mut manager = PluginManager::new();
//! manager.register(Box::new(MyPlugin));
//! ```

use std::any::Any;
use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

#[cfg(test)]
use crate::runner::{Language, TestType};
use crate::runner::{TestMeta, TestResult, TestSummary};

// ============================================================================
// Hook Specifications
// ============================================================================

/// Result type for hook execution
pub type HookResult<T> = Result<T, HookError>;

/// Future type for async hooks
pub type AsyncHookFuture<'a, T> = Pin<Box<dyn Future<Output = HookResult<T>> + Send + 'a>>;

/// Errors that can occur during hook execution
#[derive(Debug, Clone)]
pub enum HookError {
    /// Hook execution failed
    ExecutionFailed(String),
    /// Hook timed out
    Timeout(String),
    /// Plugin not found
    PluginNotFound(String),
    /// Invalid hook arguments
    InvalidArguments(String),
}

impl std::fmt::Display for HookError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HookError::ExecutionFailed(msg) => write!(f, "Hook execution failed: {}", msg),
            HookError::Timeout(msg) => write!(f, "Hook timed out: {}", msg),
            HookError::PluginNotFound(name) => write!(f, "Plugin not found: {}", name),
            HookError::InvalidArguments(msg) => write!(f, "Invalid hook arguments: {}", msg),
        }
    }
}

impl std::error::Error for HookError {}

/// Hook specification types for the plugin system
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum HookSpec {
    /// Called when the runner is being configured
    Configure,
    /// Called before test collection begins
    CollectionStart,
    /// Called after test collection completes
    CollectionFinish,
    /// Called before a test session starts
    SessionStart,
    /// Called after a test session ends
    SessionFinish,
    /// Called before each test runs
    TestStart,
    /// Called after each test completes
    TestFinish,
    /// Called when a test is skipped
    TestSkipped,
    /// Called when an error occurs
    Error,
    /// Called to modify test items after collection
    ModifyItems,
}

impl std::fmt::Display for HookSpec {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HookSpec::Configure => write!(f, "probe_configure"),
            HookSpec::CollectionStart => write!(f, "probe_collection_start"),
            HookSpec::CollectionFinish => write!(f, "probe_collection_finish"),
            HookSpec::SessionStart => write!(f, "probe_session_start"),
            HookSpec::SessionFinish => write!(f, "probe_session_finish"),
            HookSpec::TestStart => write!(f, "probe_test_start"),
            HookSpec::TestFinish => write!(f, "probe_test_finish"),
            HookSpec::TestSkipped => write!(f, "probe_test_skipped"),
            HookSpec::Error => write!(f, "probe_error"),
            HookSpec::ModifyItems => write!(f, "probe_modify_items"),
        }
    }
}

// ============================================================================
// Plugin Trait
// ============================================================================

/// Trait for implementing probe plugins
///
/// Plugins can implement any subset of hook methods. Default implementations
/// are provided that do nothing, allowing plugins to only implement the hooks
/// they care about.
pub trait Plugin: Send + Sync {
    /// Get the plugin name (must be unique)
    fn name(&self) -> &'static str;

    /// Get the plugin priority (lower = earlier execution)
    fn priority(&self) -> i32 {
        0
    }

    /// Called during runner configuration
    fn probe_configure(&self, _config: &mut PluginConfig) {}

    /// Called before test collection starts
    fn probe_collection_start(&self) {}

    /// Called after test collection finishes
    fn probe_collection_finish(&self, _items: &[TestMeta]) {}

    /// Called before a test session starts
    fn probe_session_start(&self) {}

    /// Called after a test session ends
    fn probe_session_finish(&self, _summary: &TestSummary) {}

    /// Called before each test runs
    fn probe_test_start(&self, _test: &TestMeta) {}

    /// Called after each test completes
    fn probe_test_finish(&self, _test: &TestMeta, _result: &TestResult) {}

    /// Called when a test is skipped
    fn probe_test_skipped(&self, _test: &TestMeta, _reason: &str) {}

    /// Called when an error occurs
    fn probe_error(&self, _error: &str) {}

    /// Called to modify test items after collection (return modified items)
    fn probe_modify_items(&self, items: Vec<TestMeta>) -> Vec<TestMeta> {
        items
    }

    /// Get plugin metadata as Any for downcasting
    fn as_any(&self) -> &dyn Any;
}

// ============================================================================
// Plugin Configuration
// ============================================================================

/// Configuration passed to plugins during the configure hook
#[derive(Debug, Clone, Default)]
pub struct PluginConfig {
    /// Whether to run tests in parallel
    pub parallel: bool,
    /// Number of parallel workers
    pub workers: usize,
    /// Test timeout in seconds
    pub timeout: Option<f64>,
    /// Verbose output
    pub verbose: bool,
    /// Extra plugin-specific settings
    pub extra: HashMap<String, String>,
}

impl PluginConfig {
    /// Create a new plugin config with defaults
    pub fn new() -> Self {
        Self {
            parallel: false,
            workers: 1,
            timeout: None,
            verbose: false,
            extra: HashMap::new(),
        }
    }

    /// Set a custom configuration value
    pub fn set(&mut self, key: impl Into<String>, value: impl Into<String>) {
        self.extra.insert(key.into(), value.into());
    }

    /// Get a custom configuration value
    pub fn get(&self, key: &str) -> Option<&str> {
        self.extra.get(key).map(|s| s.as_str())
    }
}

// ============================================================================
// Plugin Manager
// ============================================================================

/// Manages plugin registration and hook execution
#[derive(Default)]
pub struct PluginManager {
    plugins: Vec<Arc<dyn Plugin>>,
}

impl PluginManager {
    /// Create a new plugin manager
    pub fn new() -> Self {
        Self {
            plugins: Vec::new(),
        }
    }

    /// Register a plugin
    pub fn register(&mut self, plugin: Arc<dyn Plugin>) -> HookResult<()> {
        // Check for duplicate names
        let name = plugin.name();
        if self.plugins.iter().any(|p| p.name() == name) {
            return Err(HookError::ExecutionFailed(format!(
                "Plugin '{}' is already registered",
                name
            )));
        }

        self.plugins.push(plugin);
        self.sort_by_priority();
        Ok(())
    }

    /// Unregister a plugin by name
    pub fn unregister(&mut self, name: &str) -> HookResult<()> {
        let idx = self
            .plugins
            .iter()
            .position(|p| p.name() == name)
            .ok_or_else(|| HookError::PluginNotFound(name.to_string()))?;
        self.plugins.remove(idx);
        Ok(())
    }

    /// Get a plugin by name
    pub fn get(&self, name: &str) -> Option<&Arc<dyn Plugin>> {
        self.plugins.iter().find(|p| p.name() == name)
    }

    /// Get all registered plugin names
    pub fn names(&self) -> Vec<&'static str> {
        self.plugins.iter().map(|p| p.name()).collect()
    }

    /// Check if a plugin is registered
    pub fn has(&self, name: &str) -> bool {
        self.plugins.iter().any(|p| p.name() == name)
    }

    /// Get the number of registered plugins
    pub fn len(&self) -> usize {
        self.plugins.len()
    }

    /// Check if no plugins are registered
    pub fn is_empty(&self) -> bool {
        self.plugins.is_empty()
    }

    /// Sort plugins by priority (lower = earlier)
    fn sort_by_priority(&mut self) {
        self.plugins.sort_by_key(|p| p.priority());
    }

    // ========================================================================
    // Hook Execution Methods
    // ========================================================================

    /// Execute the configure hook
    pub fn hook_configure(&self, config: &mut PluginConfig) {
        for plugin in &self.plugins {
            plugin.probe_configure(config);
        }
    }

    /// Execute the collection_start hook
    pub fn hook_collection_start(&self) {
        for plugin in &self.plugins {
            plugin.probe_collection_start();
        }
    }

    /// Execute the collection_finish hook
    pub fn hook_collection_finish(&self, items: &[TestMeta]) {
        for plugin in &self.plugins {
            plugin.probe_collection_finish(items);
        }
    }

    /// Execute the session_start hook
    pub fn hook_session_start(&self) {
        for plugin in &self.plugins {
            plugin.probe_session_start();
        }
    }

    /// Execute the session_finish hook
    pub fn hook_session_finish(&self, summary: &TestSummary) {
        for plugin in &self.plugins {
            plugin.probe_session_finish(summary);
        }
    }

    /// Execute the test_start hook
    pub fn hook_test_start(&self, test: &TestMeta) {
        for plugin in &self.plugins {
            plugin.probe_test_start(test);
        }
    }

    /// Execute the test_finish hook
    pub fn hook_test_finish(&self, test: &TestMeta, result: &TestResult) {
        for plugin in &self.plugins {
            plugin.probe_test_finish(test, result);
        }
    }

    /// Execute the test_skipped hook
    pub fn hook_test_skipped(&self, test: &TestMeta, reason: &str) {
        for plugin in &self.plugins {
            plugin.probe_test_skipped(test, reason);
        }
    }

    /// Execute the error hook
    pub fn hook_error(&self, error: &str) {
        for plugin in &self.plugins {
            plugin.probe_error(error);
        }
    }

    /// Execute the modify_items hook (chained transformation)
    pub fn hook_modify_items(&self, items: Vec<TestMeta>) -> Vec<TestMeta> {
        let mut result = items;
        for plugin in &self.plugins {
            result = plugin.probe_modify_items(result);
        }
        result
    }
}

// ============================================================================
// Built-in Plugins
// ============================================================================

/// Logging plugin that logs test lifecycle events
#[derive(Debug, Default)]
pub struct LoggingPlugin {
    verbose: bool,
}

impl LoggingPlugin {
    pub fn new(verbose: bool) -> Self {
        Self { verbose }
    }
}

impl Plugin for LoggingPlugin {
    fn name(&self) -> &'static str {
        "logging"
    }

    fn priority(&self) -> i32 {
        -100 // Run early to log everything
    }

    fn probe_session_start(&self) {
        tracing::info!("Test session started");
    }

    fn probe_session_finish(&self, summary: &TestSummary) {
        tracing::info!(
            passed = summary.passed,
            failed = summary.failed,
            skipped = summary.skipped,
            "Test session finished"
        );
    }

    fn probe_test_start(&self, test: &TestMeta) {
        if self.verbose {
            tracing::debug!(name = %test.name, "Test starting");
        }
    }

    fn probe_test_finish(&self, test: &TestMeta, result: &TestResult) {
        tracing::info!(
            name = %test.name,
            status = ?result.status,
            duration_ms = result.duration_ms,
            "Test completed"
        );
    }

    fn probe_error(&self, error: &str) {
        tracing::error!(error = %error, "Test error");
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

/// Timeout plugin that enforces test timeouts
#[derive(Debug)]
pub struct TimeoutPlugin {
    default_timeout: f64,
}

impl TimeoutPlugin {
    pub fn new(default_timeout: f64) -> Self {
        Self { default_timeout }
    }
}

impl Default for TimeoutPlugin {
    fn default() -> Self {
        Self::new(30.0) // 30 seconds default
    }
}

impl Plugin for TimeoutPlugin {
    fn name(&self) -> &'static str {
        "timeout"
    }

    fn probe_configure(&self, config: &mut PluginConfig) {
        if config.timeout.is_none() {
            config.timeout = Some(self.default_timeout);
        }
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

/// Filter plugin that filters tests by markers/tags
#[derive(Debug, Default)]
pub struct FilterPlugin {
    include_tags: Vec<String>,
    exclude_tags: Vec<String>,
}

impl FilterPlugin {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn include(mut self, tag: impl Into<String>) -> Self {
        self.include_tags.push(tag.into());
        self
    }

    pub fn exclude(mut self, tag: impl Into<String>) -> Self {
        self.exclude_tags.push(tag.into());
        self
    }
}

impl Plugin for FilterPlugin {
    fn name(&self) -> &'static str {
        "filter"
    }

    fn probe_modify_items(&self, items: Vec<TestMeta>) -> Vec<TestMeta> {
        items
            .into_iter()
            .filter(|test| {
                // Check include tags (if any specified, test must have at least one)
                if !self.include_tags.is_empty() {
                    let has_include = test.tags.iter().any(|t| self.include_tags.contains(t));
                    if !has_include {
                        return false;
                    }
                }

                // Check exclude tags (test must not have any)
                let has_exclude = test.tags.iter().any(|t| self.exclude_tags.contains(t));
                !has_exclude
            })
            .collect()
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    struct TestPlugin {
        name: &'static str,
        priority: i32,
        configured: std::sync::atomic::AtomicBool,
    }

    impl TestPlugin {
        fn new(name: &'static str, priority: i32) -> Self {
            Self {
                name,
                priority,
                configured: std::sync::atomic::AtomicBool::new(false),
            }
        }

        fn was_configured(&self) -> bool {
            self.configured.load(std::sync::atomic::Ordering::SeqCst)
        }
    }

    impl Plugin for TestPlugin {
        fn name(&self) -> &'static str {
            self.name
        }

        fn priority(&self) -> i32 {
            self.priority
        }

        fn probe_configure(&self, _config: &mut PluginConfig) {
            self.configured
                .store(true, std::sync::atomic::Ordering::SeqCst);
        }

        fn as_any(&self) -> &dyn Any {
            self
        }
    }

    #[test]
    fn test_plugin_registration() {
        let mut manager = PluginManager::new();

        let plugin = Arc::new(TestPlugin::new("test-plugin", 0));
        assert!(manager.register(plugin).is_ok());

        assert!(manager.has("test-plugin"));
        assert_eq!(manager.len(), 1);
        assert_eq!(manager.names(), vec!["test-plugin"]);
    }

    #[test]
    fn test_duplicate_plugin_registration() {
        let mut manager = PluginManager::new();

        let plugin1 = Arc::new(TestPlugin::new("test-plugin", 0));
        let plugin2 = Arc::new(TestPlugin::new("test-plugin", 0));

        assert!(manager.register(plugin1).is_ok());
        assert!(manager.register(plugin2).is_err());
    }

    #[test]
    fn test_plugin_priority_ordering() {
        let mut manager = PluginManager::new();

        manager
            .register(Arc::new(TestPlugin::new("low", 10)))
            .unwrap();
        manager
            .register(Arc::new(TestPlugin::new("high", -10)))
            .unwrap();
        manager
            .register(Arc::new(TestPlugin::new("mid", 0)))
            .unwrap();

        let names = manager.names();
        assert_eq!(names, vec!["high", "mid", "low"]);
    }

    #[test]
    fn test_configure_hook() {
        let mut manager = PluginManager::new();

        let plugin = Arc::new(TestPlugin::new("test-plugin", 0));
        let plugin_ref = plugin.clone();
        manager.register(plugin).unwrap();

        let mut config = PluginConfig::new();
        manager.hook_configure(&mut config);

        assert!(plugin_ref.was_configured());
    }

    #[test]
    fn test_filter_plugin() {
        let filter = FilterPlugin::new().include("unit").exclude("slow");

        let items = vec![
            TestMeta {
                name: "test1".to_string(),
                full_name: "mod.test1".to_string(),
                test_type: TestType::Unit,
                language: Language::Python,
                timeout: None,
                tags: vec!["unit".to_string()],
                skip_reason: None,
                file_path: Some("test.py".to_string()),
                line_number: Some(1),
            },
            TestMeta {
                name: "test2".to_string(),
                full_name: "mod.test2".to_string(),
                test_type: TestType::Unit,
                language: Language::Python,
                timeout: None,
                tags: vec!["integration".to_string()],
                skip_reason: None,
                file_path: Some("test.py".to_string()),
                line_number: Some(10),
            },
            TestMeta {
                name: "test3".to_string(),
                full_name: "mod.test3".to_string(),
                test_type: TestType::Unit,
                language: Language::Python,
                timeout: None,
                tags: vec!["unit".to_string(), "slow".to_string()],
                skip_reason: None,
                file_path: Some("test.py".to_string()),
                line_number: Some(20),
            },
        ];

        let filtered = filter.probe_modify_items(items);
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].name, "test1");
    }

    #[test]
    fn test_logging_plugin() {
        let plugin = LoggingPlugin::new(true);
        assert_eq!(plugin.name(), "logging");
        assert_eq!(plugin.priority(), -100);
    }

    #[test]
    fn test_timeout_plugin() {
        let plugin = TimeoutPlugin::new(60.0);
        let mut config = PluginConfig::new();

        plugin.probe_configure(&mut config);

        assert_eq!(config.timeout, Some(60.0));
    }

    #[test]
    fn test_plugin_unregister() {
        let mut manager = PluginManager::new();

        manager
            .register(Arc::new(TestPlugin::new("plugin1", 0)))
            .unwrap();
        manager
            .register(Arc::new(TestPlugin::new("plugin2", 0)))
            .unwrap();

        assert_eq!(manager.len(), 2);

        manager.unregister("plugin1").unwrap();

        assert_eq!(manager.len(), 1);
        assert!(!manager.has("plugin1"));
        assert!(manager.has("plugin2"));
    }

    #[test]
    fn test_hook_spec_display() {
        assert_eq!(HookSpec::Configure.to_string(), "probe_configure");
        assert_eq!(HookSpec::TestStart.to_string(), "probe_test_start");
        assert_eq!(HookSpec::SessionFinish.to_string(), "probe_session_finish");
    }
}
````

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/meter/src/plugin.rs
    action: modify
    section: source
    impl_mode: codegen
    description: |
      Source template for `projects/meter/src/plugin.rs` captured during meter full-codegen standardization.
```
