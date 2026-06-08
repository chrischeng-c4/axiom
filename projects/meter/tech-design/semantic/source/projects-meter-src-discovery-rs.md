---
id: projects-meter-src-discovery-rs
fill_sections: [overview, source, changes]
capability_refs:
  - id: agent-use-first-cli
    role: primary
    gap: json-default-report-envelope-and-findings
    claim: json-default-report-envelope-and-findings
    coverage: full
    rationale: "Source template implements meter agent-facing CLI, runner, or report surfaces."
---

# Standardized projects/meter/src/discovery.rs

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/meter/src/discovery.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `BenchmarkRegistry` | projects/meter/src/discovery.rs | struct | pub | 214 |  |
| `DiscoveryConfig` | projects/meter/src/discovery.rs | struct | pub | 25 |  |
| `DiscoveryStats` | projects/meter/src/discovery.rs | struct | pub | 157 |  |
| `FileInfo` | projects/meter/src/discovery.rs | struct | pub | 56 |  |
| `FileType` | projects/meter/src/discovery.rs | enum | pub | 17 |  |
| `TestRegistry` | projects/meter/src/discovery.rs | struct | pub | 170 |  |
| `clear` | projects/meter/src/discovery.rs | function | pub | 199 | clear(&mut self) |
| `clear` | projects/meter/src/discovery.rs | function | pub | 243 | clear(&mut self) |
| `count` | projects/meter/src/discovery.rs | function | pub | 195 | count(&self) -> usize |
| `count` | projects/meter/src/discovery.rs | function | pub | 239 | count(&self) -> usize |
| `filter_by_pattern` | projects/meter/src/discovery.rs | function | pub | 190 | filter_by_pattern(&mut self, pattern: &str) -> &mut Self |
| `filter_by_pattern` | projects/meter/src/discovery.rs | function | pub | 234 | filter_by_pattern(&mut self, pattern: &str) -> &mut Self |
| `filter_files` | projects/meter/src/discovery.rs | function | pub | 347 | filter_files(files: Vec<FileInfo>, pattern: &str) -> Vec<FileInfo> |
| `from_path` | projects/meter/src/discovery.rs | function | pub | 82 | from_path(path: &Path, root: &Path) -> Result<Self, String> |
| `from_path_with_language` | projects/meter/src/discovery.rs | function | pub | 87 | from_path_with_language(         path: &Path,         root: &Path,         language: Option<Language>,     ) -> Result<Self, String> |
| `get_all` | projects/meter/src/discovery.rs | function | pub | 186 | get_all(&self) -> &[FileInfo] |
| `get_all` | projects/meter/src/discovery.rs | function | pub | 230 | get_all(&self) -> &[FileInfo] |
| `matches_pattern` | projects/meter/src/discovery.rs | function | pub | 140 | matches_pattern(&self, pattern: &str) -> bool |
| `new` | projects/meter/src/discovery.rs | function | pub | 176 | new() -> Self |
| `new` | projects/meter/src/discovery.rs | function | pub | 220 | new() -> Self |
| `register` | projects/meter/src/discovery.rs | function | pub | 180 | register(&mut self, file: FileInfo) |
| `register` | projects/meter/src/discovery.rs | function | pub | 224 | register(&mut self, file: FileInfo) |
| `walk_files` | projects/meter/src/discovery.rs | function | pub | 257 | walk_files(config: &DiscoveryConfig) -> Result<Vec<FileInfo>, String> |
## Source
<!-- type: source lang: rust -->
<!-- source-from-target: strip-managed-markers -->

<!-- source-snapshot: path=projects/meter/src/discovery.rs -->
````rust
// Discovery module for dbtest CLI
//
// This module provides fast file-system discovery using the jwalk crate,
// storing file paths and metadata for lazy loading during execution.
// jwalk provides parallel directory traversal for improved performance.

use crate::runner::Language;
use jwalk::{Parallelism, WalkDir};
use std::path::{Path, PathBuf};
use std::time::Instant;

/// File type classification
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileType {
    Test,
    Benchmark,
}

/// Configuration for file discovery
#[derive(Debug, Clone)]
pub struct DiscoveryConfig {
    /// Root path to start discovery
    pub root_path: PathBuf,
    /// File patterns to match (e.g., ["test_*.py", "bench_*.py"])
    pub patterns: Vec<String>,
    /// Directories to exclude (e.g., ["__pycache__", ".git"])
    pub exclusions: Vec<String>,
    /// Maximum directory depth
    pub max_depth: usize,
    /// Number of parallel threads for discovery (default: available CPU cores or 4)
    pub num_threads: usize,
}

impl Default for DiscoveryConfig {
    fn default() -> Self {
        Self {
            root_path: PathBuf::from("tests/"),
            patterns: vec!["test_*.py".to_string(), "bench_*.py".to_string()],
            exclusions: vec!["__pycache__".to_string(), ".git".to_string()],
            max_depth: 10,
            num_threads: std::thread::available_parallelism()
                .map(|n| n.get())
                .unwrap_or(4),
        }
    }
}

/// File information discovered by walkdir
#[derive(Debug, Clone)]
pub struct FileInfo {
    /// Absolute path to the file
    pub path: PathBuf,
    /// Module name (derived from path)
    pub module_name: String,
    /// File type (Test or Benchmark)
    pub file_type: FileType,
    /// Programming language detected from file extension
    pub language: Language,
}

impl FileInfo {
    /// Detect language from file extension
    fn detect_language(path: &Path) -> Language {
        match path.extension().and_then(|e| e.to_str()) {
            Some("py") => Language::Python,
            Some("rs") => Language::Rust,
            Some("ts") | Some("tsx") | Some("js") | Some("jsx") | Some("mjs") | Some("mts") => {
                Language::TypeScript
            }
            _ => Language::Python, // Default to Python for backwards compatibility
        }
    }

    /// Create FileInfo from path
    pub fn from_path(path: &Path, root: &Path) -> Result<Self, String> {
        Self::from_path_with_language(path, root, None)
    }

    /// Create FileInfo from path with explicit language (or auto-detect)
    pub fn from_path_with_language(
        path: &Path,
        root: &Path,
        language: Option<Language>,
    ) -> Result<Self, String> {
        let file_name = path
            .file_name()
            .and_then(|n| n.to_str())
            .ok_or_else(|| format!("Invalid file name: {:?}", path))?;

        // Determine file type based on file name patterns
        let file_type = if file_name.starts_with("test_")
            || file_name.ends_with("_test.rs")
            || file_name.ends_with(".test.ts")
            || file_name.ends_with(".test.js")
            || file_name.ends_with(".spec.ts")
            || file_name.ends_with(".spec.js")
        {
            FileType::Test
        } else if file_name.starts_with("bench_")
            || file_name.ends_with("_bench.rs")
            || file_name.ends_with(".bench.ts")
            || file_name.ends_with(".bench.js")
        {
            FileType::Benchmark
        } else {
            return Err(format!("Unknown file type: {}", file_name));
        };

        // Detect language from extension or use provided
        let detected_language = language.unwrap_or_else(|| Self::detect_language(path));

        // Generate module name (e.g., "tests.mongo.unit.test_document")
        let rel_path = path
            .strip_prefix(root)
            .map_err(|e| format!("Failed to strip prefix: {}", e))?;

        let module_name = rel_path
            .with_extension("")
            .components()
            .map(|c| c.as_os_str().to_string_lossy().to_string())
            .collect::<Vec<_>>()
            .join(".");

        Ok(Self {
            path: path.to_path_buf(),
            module_name,
            file_type,
            language: detected_language,
        })
    }

    /// Check if file matches a glob pattern
    pub fn matches_pattern(&self, pattern: &str) -> bool {
        let file_name = self.path.file_name().and_then(|n| n.to_str()).unwrap_or("");

        // Simple glob matching (supports * wildcard)
        if let Some(star_pos) = pattern.find('*') {
            let prefix = &pattern[..star_pos];
            let suffix = &pattern[star_pos + 1..];
            file_name.starts_with(prefix) && file_name.ends_with(suffix)
        } else {
            file_name == pattern
        }
    }
}

/// Statistics from discovery process
#[derive(Debug, Clone)]
pub struct DiscoveryStats {
    pub files_found: usize,
    pub filtered_count: usize,
    pub discovery_time_ms: u64,
    /// Total entries scanned during discovery
    pub entries_scanned: usize,
    /// Number of parallel threads used
    pub num_threads_used: usize,
}

/// Registry for test files
#[derive(Debug, Clone)]
pub struct TestRegistry {
    files: Vec<FileInfo>,
}

impl TestRegistry {
    pub fn new() -> Self {
        Self { files: Vec::new() }
    }

    pub fn register(&mut self, file: FileInfo) {
        if file.file_type == FileType::Test {
            self.files.push(file);
        }
    }

    pub fn get_all(&self) -> &[FileInfo] {
        &self.files
    }

    pub fn filter_by_pattern(&mut self, pattern: &str) -> &mut Self {
        self.files.retain(|f| f.matches_pattern(pattern));
        self
    }

    pub fn count(&self) -> usize {
        self.files.len()
    }

    pub fn clear(&mut self) {
        self.files.clear();
    }
}

impl Default for TestRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Registry for benchmark files
#[derive(Debug, Clone)]
pub struct BenchmarkRegistry {
    files: Vec<FileInfo>,
}

impl BenchmarkRegistry {
    pub fn new() -> Self {
        Self { files: Vec::new() }
    }

    pub fn register(&mut self, file: FileInfo) {
        if file.file_type == FileType::Benchmark {
            self.files.push(file);
        }
    }

    pub fn get_all(&self) -> &[FileInfo] {
        &self.files
    }

    pub fn filter_by_pattern(&mut self, pattern: &str) -> &mut Self {
        self.files.retain(|f| f.matches_pattern(pattern));
        self
    }

    pub fn count(&self) -> usize {
        self.files.len()
    }

    pub fn clear(&mut self) {
        self.files.clear();
    }
}

impl Default for BenchmarkRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Walk file system and discover test/benchmark files using parallel traversal
pub fn walk_files(config: &DiscoveryConfig) -> Result<Vec<FileInfo>, String> {
    let start = Instant::now();
    let mut files = Vec::new();
    let mut entries_scanned = 0usize;

    // Clone config for use in closure
    let exclusions = config.exclusions.clone();

    // Determine parallelism strategy based on num_threads
    let parallelism = if config.num_threads <= 1 {
        Parallelism::Serial
    } else {
        Parallelism::RayonNewPool(config.num_threads)
    };

    // Create parallel walker with jwalk
    let walker = WalkDir::new(&config.root_path)
        .follow_links(false)
        .max_depth(config.max_depth)
        .parallelism(parallelism)
        .skip_hidden(false)
        .process_read_dir(move |_depth, _path, _read_dir_state, children| {
            // Filter out excluded directories during traversal for better performance
            children.retain(|dir_entry_result| {
                if let Ok(dir_entry) = dir_entry_result {
                    if dir_entry.file_type().is_dir() {
                        let name = dir_entry.file_name().to_string_lossy();
                        !exclusions.iter().any(|ex| name.contains(ex))
                    } else {
                        true
                    }
                } else {
                    true
                }
            });
        });

    // Process entries
    for entry_result in walker {
        let entry = entry_result.map_err(|e| format!("Walk error: {}", e))?;
        entries_scanned += 1;

        // Skip directories
        if entry.file_type().is_dir() {
            continue;
        }

        // Get filename
        let file_name = entry.file_name().to_string_lossy().to_string();

        // Check if matches any pattern
        let matches_pattern = config
            .patterns
            .iter()
            .any(|pattern| pattern_matches(&file_name, pattern));

        if matches_pattern {
            let path = entry.path();
            match FileInfo::from_path(&path, &config.root_path) {
                Ok(file_info) => files.push(file_info),
                Err(e) => tracing::warn!("Failed to create FileInfo: {}", e),
            }
        }
    }

    let elapsed = start.elapsed().as_millis() as u64;
    tracing::debug!(
        "Parallel discovery completed: {} files found, {} entries scanned in {}ms using {} threads",
        files.len(),
        entries_scanned,
        elapsed,
        config.num_threads
    );

    Ok(files)
}

/// Simple glob pattern matching
fn pattern_matches(text: &str, pattern: &str) -> bool {
    if let Some(star_pos) = pattern.find('*') {
        let prefix = &pattern[..star_pos];
        let suffix = &pattern[star_pos + 1..];
        text.starts_with(prefix) && text.ends_with(suffix)
    } else {
        text == pattern
    }
}

/// Filter files by pattern
pub fn filter_files(files: Vec<FileInfo>, pattern: &str) -> Vec<FileInfo> {
    files
        .into_iter()
        .filter(|f| f.matches_pattern(pattern))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pattern_matches() {
        assert!(pattern_matches("test_foo.py", "test_*.py"));
        assert!(pattern_matches("bench_insert.py", "bench_*.py"));
        assert!(!pattern_matches("foo_test.py", "test_*.py"));
        assert!(pattern_matches("exact.py", "exact.py"));
        assert!(!pattern_matches("exact.py", "other.py"));
    }

    #[test]
    fn test_file_info_matches_pattern() {
        let path = PathBuf::from("/tmp/test_example.py");
        let root = PathBuf::from("/tmp");
        let file_info = FileInfo::from_path(&path, &root).unwrap();

        assert!(file_info.matches_pattern("test_*.py"));
        assert!(!file_info.matches_pattern("bench_*.py"));
        assert!(file_info.matches_pattern("test_example.py"));
    }

    #[test]
    fn test_test_registry() {
        let mut registry = TestRegistry::new();

        let path1 = PathBuf::from("/tmp/test_one.py");
        let path2 = PathBuf::from("/tmp/test_two.py");
        let root = PathBuf::from("/tmp");

        let file1 = FileInfo::from_path(&path1, &root).unwrap();
        let file2 = FileInfo::from_path(&path2, &root).unwrap();

        registry.register(file1);
        registry.register(file2);

        assert_eq!(registry.count(), 2);

        registry.filter_by_pattern("test_one.py");
        assert_eq!(registry.count(), 1);
    }

    #[test]
    fn test_benchmark_registry() {
        let mut registry = BenchmarkRegistry::new();

        let path1 = PathBuf::from("/tmp/bench_insert.py");
        let path2 = PathBuf::from("/tmp/bench_find.py");
        let root = PathBuf::from("/tmp");

        let file1 = FileInfo::from_path(&path1, &root).unwrap();
        let file2 = FileInfo::from_path(&path2, &root).unwrap();

        registry.register(file1);
        registry.register(file2);

        assert_eq!(registry.count(), 2);

        registry.filter_by_pattern("bench_insert.py");
        assert_eq!(registry.count(), 1);
    }

    #[test]
    fn test_filter_files() {
        let root = PathBuf::from("/tmp");
        let files = vec![
            FileInfo::from_path(&PathBuf::from("/tmp/test_foo.py"), &root).unwrap(),
            FileInfo::from_path(&PathBuf::from("/tmp/test_bar.py"), &root).unwrap(),
        ];

        let filtered = filter_files(files, "test_foo.py");
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].module_name, "test_foo");
    }

    #[test]
    fn test_parallel_discovery() {
        // Use a path relative to the workspace root
        let root = std::env::current_dir()
            .unwrap()
            .ancestors()
            .find(|p| p.join("tests").exists())
            .map(|p| p.join("tests"))
            .unwrap_or_else(|| PathBuf::from("tests/"));

        let config = DiscoveryConfig {
            root_path: root,
            num_threads: 4,
            ..Default::default()
        };

        let result = walk_files(&config);
        // Should complete discovery without error
        assert!(result.is_ok(), "Discovery should complete successfully");
        // In a real project with tests, files.len() > 0
    }

    #[test]
    fn test_single_thread_compatibility() {
        // Use a path relative to the workspace root
        let root = std::env::current_dir()
            .unwrap()
            .ancestors()
            .find(|p| p.join("tests").exists())
            .map(|p| p.join("tests"))
            .unwrap_or_else(|| PathBuf::from("tests/"));

        let config = DiscoveryConfig {
            root_path: root,
            num_threads: 1,
            ..Default::default()
        };

        let result = walk_files(&config);
        // Should work with single thread
        assert!(
            result.is_ok(),
            "Single-threaded discovery should complete successfully"
        );
    }
}
````

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/meter/src/discovery.rs
    action: modify
    section: source
    impl_mode: codegen
    description: |
      Source template for `projects/meter/src/discovery.rs` captured during meter full-codegen standardization.
```
