//! File-level caching for incremental analysis
//!
//! This module provides:
//! - Content hashing for change detection
//! - Cached module info storage
//! - Dependency-aware cache invalidation

use std::collections::{HashMap, HashSet};
use std::fs;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use super::check::TypeError;
use super::imports::ModuleInfo;

/// A content hash for change detection
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct ContentHash(pub(crate) u64);

impl ContentHash {
    /// Compute hash from file content
    pub fn from_content(content: &str) -> Self {
        use std::collections::hash_map::DefaultHasher;
        let mut hasher = DefaultHasher::new();
        content.hash(&mut hasher);
        Self(hasher.finish())
    }

    /// Compute hash from file path
    pub fn from_file(path: &Path) -> Option<Self> {
        fs::read_to_string(path)
            .ok()
            .map(|c| Self::from_content(&c))
    }
}

/// Cached module entry
#[derive(Debug, Clone)]
pub struct CacheEntry {
    /// Module name
    pub module_name: String,
    /// File path
    pub path: PathBuf,
    /// Content hash for change detection
    pub content_hash: ContentHash,
    /// Modification time (for quick change detection)
    pub mtime: Option<SystemTime>,
    /// Cached module info
    pub info: ModuleInfo,
    /// Cached type errors
    pub errors: Vec<TypeError>,
    /// Modules this module depends on
    pub dependencies: HashSet<String>,
    /// Whether cross-file propagated types are still valid (R3, R8).
    pub propagation_valid: bool,
}

impl CacheEntry {
    pub fn new(
        module_name: String,
        path: PathBuf,
        content_hash: ContentHash,
        info: ModuleInfo,
    ) -> Self {
        let mtime = fs::metadata(&path).ok().and_then(|m| m.modified().ok());

        Self {
            module_name,
            path,
            content_hash,
            mtime,
            info,
            errors: Vec::new(),
            dependencies: HashSet::new(),
            propagation_valid: false,
        }
    }

    /// Check if file has changed based on mtime (fast check)
    pub fn mtime_changed(&self) -> bool {
        let current_mtime = fs::metadata(&self.path)
            .ok()
            .and_then(|m| m.modified().ok());

        match (self.mtime, current_mtime) {
            (Some(cached), Some(current)) => cached != current,
            _ => true, // Assume changed if we can't determine
        }
    }

    /// Check if file content has changed (slower but accurate)
    pub fn content_changed(&self) -> bool {
        match ContentHash::from_file(&self.path) {
            Some(current) => current != self.content_hash,
            None => true, // File removed or unreadable
        }
    }

    /// Check if this entry needs reanalysis
    pub fn needs_reanalysis(&self) -> bool {
        // Quick mtime check first
        if !self.mtime_changed() {
            return false;
        }
        // Fall back to content hash check
        self.content_changed()
    }
}

/// Module analysis cache
#[derive(Debug, Default)]
pub struct AnalysisCache {
    /// Cached entries by module name
    entries: HashMap<String, CacheEntry>,
    /// Reverse dependency map: module -> modules that depend on it
    reverse_deps: HashMap<String, HashSet<String>>,
}

impl AnalysisCache {
    pub fn new() -> Self {
        Self {
            entries: HashMap::new(),
            reverse_deps: HashMap::new(),
        }
    }

    /// Store a cache entry
    pub fn store(&mut self, entry: CacheEntry) {
        let module_name = entry.module_name.clone();

        // Update reverse dependency map
        for dep in &entry.dependencies {
            self.reverse_deps
                .entry(dep.clone())
                .or_default()
                .insert(module_name.clone());
        }

        self.entries.insert(module_name, entry);
    }

    /// Get a cached entry
    pub fn get(&self, module_name: &str) -> Option<&CacheEntry> {
        self.entries.get(module_name)
    }

    /// Get mutable cached entry
    pub fn get_mut(&mut self, module_name: &str) -> Option<&mut CacheEntry> {
        self.entries.get_mut(module_name)
    }

    /// Check if module is cached
    pub fn has(&self, module_name: &str) -> bool {
        self.entries.contains_key(module_name)
    }

    /// Remove a cached entry
    pub fn remove(&mut self, module_name: &str) -> Option<CacheEntry> {
        if let Some(entry) = self.entries.remove(module_name) {
            // Clean up reverse deps
            for dep in &entry.dependencies {
                if let Some(rdeps) = self.reverse_deps.get_mut(dep) {
                    rdeps.remove(module_name);
                }
            }
            Some(entry)
        } else {
            None
        }
    }

    /// Get modules that need reanalysis due to changes
    pub fn get_changed_modules(&self) -> Vec<String> {
        self.entries
            .iter()
            .filter(|(_, entry)| entry.needs_reanalysis())
            .map(|(name, _)| name.clone())
            .collect()
    }

    /// Get all modules affected by changes to the given module
    /// (including transitive dependents)
    pub fn get_affected_modules(&self, changed: &str) -> HashSet<String> {
        let mut affected = HashSet::new();
        let mut queue = vec![changed.to_string()];

        while let Some(module) = queue.pop() {
            if affected.insert(module.clone()) {
                if let Some(dependents) = self.reverse_deps.get(&module) {
                    for dep in dependents {
                        if !affected.contains(dep) {
                            queue.push(dep.clone());
                        }
                    }
                }
            }
        }

        affected
    }

    /// Invalidate a module and all its dependents
    pub fn invalidate(&mut self, module_name: &str) {
        let affected = self.get_affected_modules(module_name);
        for name in affected {
            self.entries.remove(&name);
        }
    }

    /// Invalidate propagated type information for a module and its dependents (R8).
    ///
    /// Unlike `invalidate()` which removes the entire cache entry, this only
    /// marks the entry for re-propagation while keeping locally-inferred data
    /// intact.  All transitive dependents of `module_name` are also marked.
    pub fn invalidate_propagation(&mut self, module_name: &str) {
        let affected = self.get_affected_modules(module_name);
        for name in affected {
            if let Some(entry) = self.entries.get_mut(&name) {
                entry.propagation_valid = false;
            }
        }
    }

    /// Get all cached module names
    pub fn module_names(&self) -> impl Iterator<Item = &String> {
        self.entries.keys()
    }

    /// Get cache statistics
    pub fn stats(&self) -> CacheStats {
        CacheStats {
            total_entries: self.entries.len(),
            stale_entries: self.get_changed_modules().len(),
        }
    }

    /// Clear the entire cache
    pub fn clear(&mut self) {
        self.entries.clear();
        self.reverse_deps.clear();
    }
}

/// Cache statistics
#[derive(Debug, Clone, Copy)]
pub struct CacheStats {
    pub total_entries: usize,
    pub stale_entries: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn test_content_hash() {
        let hash1 = ContentHash::from_content("hello world");
        let hash2 = ContentHash::from_content("hello world");
        let hash3 = ContentHash::from_content("hello world!");

        assert_eq!(hash1, hash2);
        assert_ne!(hash1, hash3);
    }

    #[test]
    fn test_cache_entry_creation() {
        let path = PathBuf::from("/test/module.py");
        let hash = ContentHash::from_content("content");
        let info = ModuleInfo::new("test_module");

        let entry = CacheEntry::new("test_module".to_string(), path.clone(), hash, info);

        assert_eq!(entry.module_name, "test_module");
        assert_eq!(entry.path, path);
        assert_eq!(entry.content_hash, hash);
    }

    #[test]
    fn test_analysis_cache_store_and_get() {
        let mut cache = AnalysisCache::new();

        let entry = CacheEntry::new(
            "mymodule".to_string(),
            PathBuf::from("/test/mymodule.py"),
            ContentHash::from_content("content"),
            ModuleInfo::new("mymodule"),
        );

        cache.store(entry);

        assert!(cache.has("mymodule"));
        assert!(!cache.has("nonexistent"));

        let retrieved = cache.get("mymodule");
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().module_name, "mymodule");
    }

    #[test]
    fn test_reverse_dependencies() {
        let mut cache = AnalysisCache::new();

        // Create module B that depends on A
        let mut entry_b = CacheEntry::new(
            "b".to_string(),
            PathBuf::from("/test/b.py"),
            ContentHash::from_content("import a"),
            ModuleInfo::new("b"),
        );
        entry_b.dependencies.insert("a".to_string());

        // Create module C that depends on A
        let mut entry_c = CacheEntry::new(
            "c".to_string(),
            PathBuf::from("/test/c.py"),
            ContentHash::from_content("import a"),
            ModuleInfo::new("c"),
        );
        entry_c.dependencies.insert("a".to_string());

        cache.store(entry_b);
        cache.store(entry_c);

        // Check affected modules when A changes
        let affected = cache.get_affected_modules("a");
        assert!(affected.contains("a"));
        assert!(affected.contains("b"));
        assert!(affected.contains("c"));
    }

    #[test]
    fn test_transitive_dependencies() {
        let mut cache = AnalysisCache::new();

        // A -> B -> C (chain of dependencies)
        let entry_a = CacheEntry::new(
            "a".to_string(),
            PathBuf::from("/test/a.py"),
            ContentHash::from_content(""),
            ModuleInfo::new("a"),
        );

        let mut entry_b = CacheEntry::new(
            "b".to_string(),
            PathBuf::from("/test/b.py"),
            ContentHash::from_content("import a"),
            ModuleInfo::new("b"),
        );
        entry_b.dependencies.insert("a".to_string());

        let mut entry_c = CacheEntry::new(
            "c".to_string(),
            PathBuf::from("/test/c.py"),
            ContentHash::from_content("import b"),
            ModuleInfo::new("c"),
        );
        entry_c.dependencies.insert("b".to_string());

        cache.store(entry_a);
        cache.store(entry_b);
        cache.store(entry_c);

        // When A changes, both B and C should be affected
        let affected = cache.get_affected_modules("a");
        assert!(affected.contains("a"));
        assert!(affected.contains("b"));
        assert!(affected.contains("c"));
    }

    #[test]
    fn test_invalidation() {
        let mut cache = AnalysisCache::new();

        let entry_a = CacheEntry::new(
            "a".to_string(),
            PathBuf::from("/test/a.py"),
            ContentHash::from_content(""),
            ModuleInfo::new("a"),
        );

        let mut entry_b = CacheEntry::new(
            "b".to_string(),
            PathBuf::from("/test/b.py"),
            ContentHash::from_content("import a"),
            ModuleInfo::new("b"),
        );
        entry_b.dependencies.insert("a".to_string());

        cache.store(entry_a);
        cache.store(entry_b);

        assert!(cache.has("a"));
        assert!(cache.has("b"));

        // Invalidate A, which should also invalidate B
        cache.invalidate("a");

        assert!(!cache.has("a"));
        assert!(!cache.has("b"));
    }

    #[test]
    fn test_cache_stats() {
        let mut cache = AnalysisCache::new();

        let entry = CacheEntry::new(
            "module".to_string(),
            PathBuf::from("/test/module.py"),
            ContentHash::from_content(""),
            ModuleInfo::new("module"),
        );

        cache.store(entry);

        let stats = cache.stats();
        assert_eq!(stats.total_entries, 1);
    }

    #[test]
    fn test_file_hash() {
        let temp_dir = env::temp_dir().join("cclab_lens_cache_test");
        let _ = fs::remove_dir_all(&temp_dir);
        fs::create_dir_all(&temp_dir).unwrap();

        let file_path = temp_dir.join("test.py");
        fs::write(&file_path, "print('hello')").unwrap();

        let hash1 = ContentHash::from_file(&file_path);
        assert!(hash1.is_some());

        // Same content = same hash
        let hash2 = ContentHash::from_file(&file_path);
        assert_eq!(hash1, hash2);

        // Change content = different hash
        fs::write(&file_path, "print('world')").unwrap();
        let hash3 = ContentHash::from_file(&file_path);
        assert_ne!(hash1, hash3);

        // Cleanup
        let _ = fs::remove_dir_all(&temp_dir);
    }

    /// R3: After propagation, cache entries can be marked with propagation_valid=true.
    #[test]
    fn test_cache_stores_propagated_types() {
        let mut cache = AnalysisCache::new();

        // Create a source module.
        let entry_src = CacheEntry::new(
            "db".to_string(),
            PathBuf::from("/test/db.py"),
            ContentHash::from_content("def get_user(): ..."),
            ModuleInfo::new("db"),
        );
        cache.store(entry_src);

        // Create an importing module that depends on db.
        let mut entry_handler = CacheEntry::new(
            "handler".to_string(),
            PathBuf::from("/test/handler.py"),
            ContentHash::from_content("from db import get_user"),
            ModuleInfo::new("handler"),
        );
        entry_handler.dependencies.insert("db".to_string());
        cache.store(entry_handler);

        // Initially propagation_valid should be false.
        assert!(!cache.get("handler").unwrap().propagation_valid);

        // After propagation, mark as valid.
        if let Some(entry) = cache.get_mut("handler") {
            entry.propagation_valid = true;
        }
        assert!(cache.get("handler").unwrap().propagation_valid);
    }

    /// R8, S6: Changing source file clears propagated bindings in all importers.
    #[test]
    fn test_invalidation_on_dependency_change() {
        let mut cache = AnalysisCache::new();

        // Module A (source).
        let entry_a = CacheEntry::new(
            "a".to_string(),
            PathBuf::from("/test/a.py"),
            ContentHash::from_content("class Foo: pass"),
            ModuleInfo::new("a"),
        );
        cache.store(entry_a);

        // Module B depends on A.
        let mut entry_b = CacheEntry::new(
            "b".to_string(),
            PathBuf::from("/test/b.py"),
            ContentHash::from_content("from a import Foo"),
            ModuleInfo::new("b"),
        );
        entry_b.dependencies.insert("a".to_string());
        entry_b.propagation_valid = true; // mark as propagated
        cache.store(entry_b);

        // Module C depends on B (transitive).
        let mut entry_c = CacheEntry::new(
            "c".to_string(),
            PathBuf::from("/test/c.py"),
            ContentHash::from_content("from b import Foo"),
            ModuleInfo::new("c"),
        );
        entry_c.dependencies.insert("b".to_string());
        entry_c.propagation_valid = true;
        cache.store(entry_c);

        // Verify both are propagation_valid before invalidation.
        assert!(cache.get("b").unwrap().propagation_valid);
        assert!(cache.get("c").unwrap().propagation_valid);

        // Invalidate propagation starting from A.
        cache.invalidate_propagation("a");

        // Both B and C should have propagation_valid=false now.
        assert!(
            !cache
                .get("a")
                .unwrap_or(&CacheEntry::new(
                    "a".to_string(),
                    PathBuf::from("/test/a.py"),
                    ContentHash::from_content(""),
                    ModuleInfo::new("a"),
                ))
                .propagation_valid
        );
        assert!(
            !cache.get("b").unwrap().propagation_valid,
            "B should be invalidated because it depends on A"
        );
        assert!(
            !cache.get("c").unwrap().propagation_valid,
            "C should be invalidated transitively through B"
        );
    }
}
