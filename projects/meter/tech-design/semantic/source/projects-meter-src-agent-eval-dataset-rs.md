---
id: projects-meter-src-agent-eval-dataset-rs
fill_sections: [overview, source, changes]
capability_refs:
  - id: agent-use-first-cli
    role: primary
    gap: json-default-report-envelope-and-findings
    claim: json-default-report-envelope-and-findings
    coverage: full
    rationale: "Source template implements meter agent-facing CLI, runner, or report surfaces."
---

# Standardized projects/meter/src/agent_eval/dataset.rs

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/meter/src/agent_eval/dataset.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `DatasetGitIntegration` | projects/meter/src/agent_eval/dataset.rs | struct | pub | 284 |  |
| `DatasetMetadata` | projects/meter/src/agent_eval/dataset.rs | struct | pub | 15 |  |
| `DatasetSnapshot` | projects/meter/src/agent_eval/dataset.rs | struct | pub | 44 |  |
| `GoldenDataset` | projects/meter/src/agent_eval/dataset.rs | struct | pub | 108 |  |
| `commit_dataset` | projects/meter/src/agent_eval/dataset.rs | function | pub | 338 | commit_dataset(file_path: &Path, commit_message: &str) -> io::Result<()> |
| `delete` | projects/meter/src/agent_eval/dataset.rs | function | pub | 220 | delete(&self, name: &str) -> io::Result<()> |
| `exists` | projects/meter/src/agent_eval/dataset.rs | function | pub | 212 | exists(&self, name: &str) -> bool |
| `get_current_commit` | projects/meter/src/agent_eval/dataset.rs | function | pub | 292 | get_current_commit() -> Option<String> |
| `is_empty` | projects/meter/src/agent_eval/dataset.rs | function | pub | 101 | is_empty(&self) -> bool |
| `len` | projects/meter/src/agent_eval/dataset.rs | function | pub | 96 | len(&self) -> usize |
| `list` | projects/meter/src/agent_eval/dataset.rs | function | pub | 229 | list(&self) -> io::Result<Vec<String>> |
| `load` | projects/meter/src/agent_eval/dataset.rs | function | pub | 180 | load(&self, name: &str) -> io::Result<DatasetSnapshot> |
| `load_test_cases` | projects/meter/src/agent_eval/dataset.rs | function | pub | 200 | load_test_cases(&self, name: &str) -> io::Result<Vec<AgentTestCase>> |
| `new` | projects/meter/src/agent_eval/dataset.rs | function | pub | 55 | new(name: impl Into<String>, test_cases: Vec<AgentTestCase>) -> Self |
| `new` | projects/meter/src/agent_eval/dataset.rs | function | pub | 119 | new(root_dir: impl AsRef<Path>) -> io::Result<Self> |
| `root_dir` | projects/meter/src/agent_eval/dataset.rs | function | pub | 277 | root_dir(&self) -> &Path |
| `save` | projects/meter/src/agent_eval/dataset.rs | function | pub | 138 | save(&self, name: &str, snapshot: &DatasetSnapshot) -> io::Result<PathBuf> |
| `save_test_cases` | projects/meter/src/agent_eval/dataset.rs | function | pub | 164 | save_test_cases(         &self,         name: &str,         test_cases: Vec<AgentTestCase>,     ) -> io::Result<PathBuf> |
| `save_with_git_tracking` | projects/meter/src/agent_eval/dataset.rs | function | pub | 317 | save_with_git_tracking(         dataset: &GoldenDataset,         name: &str,         test_cases: Vec<AgentTestCase>,     ) -> io::Result<PathBuf> |
| `update` | projects/meter/src/agent_eval/dataset.rs | function | pub | 259 | update(&self, name: &str, test_cases: Vec<AgentTestCase>) -> io::Result<PathBuf> |
| `with_description` | projects/meter/src/agent_eval/dataset.rs | function | pub | 72 | with_description(mut self, description: impl Into<String>) -> Self |
| `with_git_commit` | projects/meter/src/agent_eval/dataset.rs | function | pub | 84 | with_git_commit(mut self, commit: impl Into<String>) -> Self |
| `with_tags` | projects/meter/src/agent_eval/dataset.rs | function | pub | 90 | with_tags(mut self, tags: Vec<String>) -> Self |
| `with_version` | projects/meter/src/agent_eval/dataset.rs | function | pub | 78 | with_version(mut self, version: impl Into<String>) -> Self |
## Source
<!-- type: rust-source-unit lang: rust -->

````rust
//! Golden dataset management for agent evaluation test cases

use crate::agent_eval::test_case::AgentTestCase;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

/// Metadata for a golden dataset
#[derive(Debug, Clone, Serialize, Deserialize)]
/// @spec projects/meter/tech-design/semantic/source/projects-meter-src-agent-eval-dataset-rs.md#source
pub struct DatasetMetadata {
    /// Dataset name
    pub name: String,

    /// Dataset version
    pub version: String,

    /// Creation timestamp
    pub created_at: String,

    /// Last updated timestamp
    pub updated_at: String,

    /// Description
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Git commit hash when created
    #[serde(skip_serializing_if = "Option::is_none")]
    pub git_commit: Option<String>,

    /// Tags for categorization
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<String>,
}

/// A golden dataset containing test cases
#[derive(Debug, Clone, Serialize, Deserialize)]
/// @spec projects/meter/tech-design/semantic/source/projects-meter-src-agent-eval-dataset-rs.md#source
pub struct DatasetSnapshot {
    /// Metadata
    pub metadata: DatasetMetadata,

    /// Test cases
    pub test_cases: Vec<AgentTestCase>,
}

/// @spec projects/meter/tech-design/semantic/source/projects-meter-src-agent-eval-dataset-rs.md#source
impl DatasetSnapshot {
    /// Create a new dataset snapshot
    pub fn new(name: impl Into<String>, test_cases: Vec<AgentTestCase>) -> Self {
        let now = Utc::now().to_rfc3339();
        Self {
            metadata: DatasetMetadata {
                name: name.into(),
                version: "1.0.0".to_string(),
                created_at: now.clone(),
                updated_at: now,
                description: None,
                git_commit: None,
                tags: Vec::new(),
            },
            test_cases,
        }
    }

    /// Set description
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.metadata.description = Some(description.into());
        self
    }

    /// Set version
    pub fn with_version(mut self, version: impl Into<String>) -> Self {
        self.metadata.version = version.into();
        self
    }

    /// Set git commit
    pub fn with_git_commit(mut self, commit: impl Into<String>) -> Self {
        self.metadata.git_commit = Some(commit.into());
        self
    }

    /// Set tags
    pub fn with_tags(mut self, tags: Vec<String>) -> Self {
        self.metadata.tags = tags;
        self
    }

    /// Get number of test cases
    pub fn len(&self) -> usize {
        self.test_cases.len()
    }

    /// Check if dataset is empty
    pub fn is_empty(&self) -> bool {
        self.test_cases.is_empty()
    }
}

/// Golden dataset manager for storing and loading test case datasets
/// @spec projects/meter/tech-design/semantic/source/projects-meter-src-agent-eval-dataset-rs.md#source
pub struct GoldenDataset {
    /// Root directory for datasets
    root_dir: PathBuf,
}

/// @spec projects/meter/tech-design/semantic/source/projects-meter-src-agent-eval-dataset-rs.md#source
impl GoldenDataset {
    /// Create a new golden dataset manager
    ///
    /// # Arguments
    /// * `root_dir` - Root directory for storing datasets (e.g., ".agent_eval/datasets")
    pub fn new(root_dir: impl AsRef<Path>) -> io::Result<Self> {
        let root_dir = root_dir.as_ref().to_path_buf();

        // Create directory if it doesn't exist
        if !root_dir.exists() {
            fs::create_dir_all(&root_dir)?;
        }

        Ok(Self { root_dir })
    }

    /// Save a dataset snapshot
    ///
    /// # Arguments
    /// * `name` - Dataset name (used as filename)
    /// * `snapshot` - Dataset snapshot to save
    ///
    /// # Returns
    /// Path to the saved file
    pub fn save(&self, name: &str, snapshot: &DatasetSnapshot) -> io::Result<PathBuf> {
        let file_path = self.get_dataset_path(name);

        // Ensure parent directory exists
        if let Some(parent) = file_path.parent() {
            fs::create_dir_all(parent)?;
        }

        // Serialize to YAML
        let yaml = serde_yaml::to_string(snapshot)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

        // Write to file
        fs::write(&file_path, yaml)?;

        Ok(file_path)
    }

    /// Save test cases as a new dataset
    ///
    /// # Arguments
    /// * `name` - Dataset name
    /// * `test_cases` - Test cases to save
    ///
    /// # Returns
    /// Path to the saved file
    pub fn save_test_cases(
        &self,
        name: &str,
        test_cases: Vec<AgentTestCase>,
    ) -> io::Result<PathBuf> {
        let snapshot = DatasetSnapshot::new(name, test_cases);
        self.save(name, &snapshot)
    }

    /// Load a dataset snapshot
    ///
    /// # Arguments
    /// * `name` - Dataset name
    ///
    /// # Returns
    /// Dataset snapshot
    pub fn load(&self, name: &str) -> io::Result<DatasetSnapshot> {
        let file_path = self.get_dataset_path(name);

        // Read file
        let yaml = fs::read_to_string(&file_path)?;

        // Deserialize from YAML
        let snapshot: DatasetSnapshot = serde_yaml::from_str(&yaml)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

        Ok(snapshot)
    }

    /// Load test cases from a dataset
    ///
    /// # Arguments
    /// * `name` - Dataset name
    ///
    /// # Returns
    /// Test cases
    pub fn load_test_cases(&self, name: &str) -> io::Result<Vec<AgentTestCase>> {
        let snapshot = self.load(name)?;
        Ok(snapshot.test_cases)
    }

    /// Check if a dataset exists
    ///
    /// # Arguments
    /// * `name` - Dataset name
    ///
    /// # Returns
    /// True if dataset exists
    pub fn exists(&self, name: &str) -> bool {
        self.get_dataset_path(name).exists()
    }

    /// Delete a dataset
    ///
    /// # Arguments
    /// * `name` - Dataset name
    pub fn delete(&self, name: &str) -> io::Result<()> {
        let file_path = self.get_dataset_path(name);
        fs::remove_file(file_path)
    }

    /// List all available datasets
    ///
    /// # Returns
    /// Vector of dataset names
    pub fn list(&self) -> io::Result<Vec<String>> {
        let mut datasets = Vec::new();

        if !self.root_dir.exists() {
            return Ok(datasets);
        }

        for entry in fs::read_dir(&self.root_dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("yaml") {
                if let Some(name) = path.file_stem().and_then(|s| s.to_str()) {
                    datasets.push(name.to_string());
                }
            }
        }

        datasets.sort();
        Ok(datasets)
    }

    /// Update an existing dataset
    ///
    /// # Arguments
    /// * `name` - Dataset name
    /// * `test_cases` - New test cases
    ///
    /// # Returns
    /// Path to the updated file
    pub fn update(&self, name: &str, test_cases: Vec<AgentTestCase>) -> io::Result<PathBuf> {
        // Load existing snapshot to preserve metadata
        let mut snapshot = self.load(name)?;

        // Update test cases and timestamp
        snapshot.test_cases = test_cases;
        snapshot.metadata.updated_at = Utc::now().to_rfc3339();

        // Save updated snapshot
        self.save(name, &snapshot)
    }

    /// Get the file path for a dataset
    fn get_dataset_path(&self, name: &str) -> PathBuf {
        self.root_dir.join(format!("{}.yaml", name))
    }

    /// Get the root directory
    pub fn root_dir(&self) -> &Path {
        &self.root_dir
    }
}

/// Git integration for dataset versioning
/// @spec projects/meter/tech-design/semantic/source/projects-meter-src-agent-eval-dataset-rs.md#source
pub struct DatasetGitIntegration;

/// @spec projects/meter/tech-design/semantic/source/projects-meter-src-agent-eval-dataset-rs.md#source
impl DatasetGitIntegration {
    /// Get current git commit hash
    ///
    /// # Returns
    /// Current commit hash or None if not in a git repo
    pub fn get_current_commit() -> Option<String> {
        use std::process::Command;

        let output = Command::new("git")
            .args(&["rev-parse", "HEAD"])
            .output()
            .ok()?;

        if output.status.success() {
            let commit = String::from_utf8(output.stdout).ok()?;
            Some(commit.trim().to_string())
        } else {
            None
        }
    }

    /// Save dataset with git commit tracking
    ///
    /// # Arguments
    /// * `dataset` - Golden dataset manager
    /// * `name` - Dataset name
    /// * `test_cases` - Test cases to save
    ///
    /// # Returns
    /// Path to the saved file
    pub fn save_with_git_tracking(
        dataset: &GoldenDataset,
        name: &str,
        test_cases: Vec<AgentTestCase>,
    ) -> io::Result<PathBuf> {
        let git_commit = Self::get_current_commit();

        let snapshot = DatasetSnapshot::new(name, test_cases)
            .with_git_commit(git_commit.unwrap_or_else(|| "unknown".to_string()));

        dataset.save(name, &snapshot)
    }

    /// Stage and commit a dataset file
    ///
    /// # Arguments
    /// * `file_path` - Path to the dataset file
    /// * `commit_message` - Commit message
    ///
    /// # Returns
    /// Success or error
    pub fn commit_dataset(file_path: &Path, commit_message: &str) -> io::Result<()> {
        use std::process::Command;

        // Stage file
        let status = Command::new("git")
            .args(&["add", file_path.to_str().unwrap_or("")])
            .status()?;

        if !status.success() {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                "Failed to stage dataset file",
            ));
        }

        // Commit
        let status = Command::new("git")
            .args(&["commit", "-m", commit_message])
            .status()?;

        if !status.success() {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                "Failed to commit dataset",
            ));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn create_test_cases() -> Vec<AgentTestCase> {
        vec![
            AgentTestCase::new("test-001", "Test 1", "Input 1").with_expected_output("Output 1"),
            AgentTestCase::new("test-002", "Test 2", "Input 2")
                .with_expected_output_regex(r"Output.*"),
        ]
    }

    #[test]
    fn test_dataset_snapshot_creation() {
        let test_cases = create_test_cases();
        let snapshot = DatasetSnapshot::new("test_dataset", test_cases.clone());

        assert_eq!(snapshot.metadata.name, "test_dataset");
        assert_eq!(snapshot.metadata.version, "1.0.0");
        assert_eq!(snapshot.test_cases.len(), 2);
        assert!(!snapshot.is_empty());
    }

    #[test]
    fn test_dataset_snapshot_builders() {
        let test_cases = create_test_cases();
        let snapshot = DatasetSnapshot::new("test_dataset", test_cases)
            .with_description("Test dataset")
            .with_version("2.0.0")
            .with_git_commit("abc123")
            .with_tags(vec!["unit".to_string(), "regression".to_string()]);

        assert_eq!(
            snapshot.metadata.description,
            Some("Test dataset".to_string())
        );
        assert_eq!(snapshot.metadata.version, "2.0.0");
        assert_eq!(snapshot.metadata.git_commit, Some("abc123".to_string()));
        assert_eq!(snapshot.metadata.tags, vec!["unit", "regression"]);
    }

    #[test]
    fn test_golden_dataset_save_and_load() {
        let temp_dir = TempDir::new().unwrap();
        let dataset = GoldenDataset::new(temp_dir.path()).unwrap();

        let test_cases = create_test_cases();
        let snapshot = DatasetSnapshot::new("test_dataset", test_cases.clone());

        // Save
        let file_path = dataset.save("test_dataset", &snapshot).unwrap();
        assert!(file_path.exists());

        // Load
        let loaded = dataset.load("test_dataset").unwrap();
        assert_eq!(loaded.metadata.name, "test_dataset");
        assert_eq!(loaded.test_cases.len(), 2);
        assert_eq!(loaded.test_cases[0].id, "test-001");
    }

    #[test]
    fn test_golden_dataset_save_test_cases() {
        let temp_dir = TempDir::new().unwrap();
        let dataset = GoldenDataset::new(temp_dir.path()).unwrap();

        let test_cases = create_test_cases();
        let file_path = dataset
            .save_test_cases("simple_dataset", test_cases.clone())
            .unwrap();
        assert!(file_path.exists());

        // Load and verify
        let loaded_cases = dataset.load_test_cases("simple_dataset").unwrap();
        assert_eq!(loaded_cases.len(), 2);
        assert_eq!(loaded_cases[0].id, "test-001");
    }

    #[test]
    fn test_golden_dataset_exists() {
        let temp_dir = TempDir::new().unwrap();
        let dataset = GoldenDataset::new(temp_dir.path()).unwrap();

        assert!(!dataset.exists("nonexistent"));

        let test_cases = create_test_cases();
        dataset.save_test_cases("existing", test_cases).unwrap();

        assert!(dataset.exists("existing"));
    }

    #[test]
    fn test_golden_dataset_delete() {
        let temp_dir = TempDir::new().unwrap();
        let dataset = GoldenDataset::new(temp_dir.path()).unwrap();

        let test_cases = create_test_cases();
        dataset.save_test_cases("to_delete", test_cases).unwrap();
        assert!(dataset.exists("to_delete"));

        dataset.delete("to_delete").unwrap();
        assert!(!dataset.exists("to_delete"));
    }

    #[test]
    fn test_golden_dataset_list() {
        let temp_dir = TempDir::new().unwrap();
        let dataset = GoldenDataset::new(temp_dir.path()).unwrap();

        let test_cases = create_test_cases();
        dataset
            .save_test_cases("dataset1", test_cases.clone())
            .unwrap();
        dataset
            .save_test_cases("dataset2", test_cases.clone())
            .unwrap();
        dataset.save_test_cases("dataset3", test_cases).unwrap();

        let list = dataset.list().unwrap();
        assert_eq!(list.len(), 3);
        assert!(list.contains(&"dataset1".to_string()));
        assert!(list.contains(&"dataset2".to_string()));
        assert!(list.contains(&"dataset3".to_string()));
    }

    #[test]
    fn test_golden_dataset_update() {
        let temp_dir = TempDir::new().unwrap();
        let dataset = GoldenDataset::new(temp_dir.path()).unwrap();

        // Create initial dataset
        let test_cases = create_test_cases();
        dataset.save_test_cases("to_update", test_cases).unwrap();

        let original = dataset.load("to_update").unwrap();
        let original_updated_at = original.metadata.updated_at.clone();

        // Wait a moment to ensure timestamp changes
        std::thread::sleep(std::time::Duration::from_millis(10));

        // Update with new test cases
        let new_test_cases =
            vec![AgentTestCase::new("test-003", "Test 3", "Input 3")
                .with_expected_output("Output 3")];
        dataset.update("to_update", new_test_cases).unwrap();

        // Verify update
        let updated = dataset.load("to_update").unwrap();
        assert_eq!(updated.test_cases.len(), 1);
        assert_eq!(updated.test_cases[0].id, "test-003");
        assert_ne!(updated.metadata.updated_at, original_updated_at);
    }

    #[test]
    fn test_yaml_serialization() {
        let test_cases = create_test_cases();
        let snapshot =
            DatasetSnapshot::new("yaml_test", test_cases).with_description("YAML test dataset");

        let yaml = serde_yaml::to_string(&snapshot).unwrap();

        // Verify YAML contains expected fields
        assert!(yaml.contains("name: yaml_test"));
        assert!(yaml.contains("version: 1.0.0"));
        assert!(yaml.contains("description: YAML test dataset"));
        assert!(yaml.contains("test_cases:"));

        // Verify deserialization works
        let deserialized: DatasetSnapshot = serde_yaml::from_str(&yaml).unwrap();
        assert_eq!(deserialized.metadata.name, "yaml_test");
        assert_eq!(deserialized.test_cases.len(), 2);
    }

    #[test]
    fn test_git_integration_get_commit() {
        // This test may fail in non-git environments
        // The function should return None gracefully
        let commit = DatasetGitIntegration::get_current_commit();

        // If we're in a git repo, commit should be 40 hex chars
        if let Some(hash) = commit {
            assert_eq!(hash.len(), 40);
            assert!(hash.chars().all(|c| c.is_ascii_hexdigit()));
        }
    }

    #[test]
    fn test_git_integration_save_with_tracking() {
        let temp_dir = TempDir::new().unwrap();
        let dataset = GoldenDataset::new(temp_dir.path()).unwrap();

        let test_cases = create_test_cases();
        let file_path =
            DatasetGitIntegration::save_with_git_tracking(&dataset, "git_tracked", test_cases)
                .unwrap();

        assert!(file_path.exists());

        // Load and verify git commit is set
        let loaded = dataset.load("git_tracked").unwrap();
        assert!(loaded.metadata.git_commit.is_some());
    }
}
````

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/meter/src/agent_eval/dataset.rs
    action: modify
    section: rust-source-unit
    impl_mode: codegen
    description: |
      Source template for `projects/meter/src/agent_eval/dataset.rs` captured during meter full-codegen standardization.
```
