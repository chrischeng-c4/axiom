---
id: sdd-ui-viewer-types
fill_sections: [source, overview, schema, changes]
capability_refs:
  - id: aw-core-client-model-workitem-first-artifact-lifecycle
    role: primary
    gap: core-concept-model-and-invariants
    claim: core-concept-model-and-invariants
    coverage: full
    rationale: "Core model/parser TDs define AW Core domain nouns, invariants, and artifact structure."
---

# Viewer Manager Data Types

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/agentic-workflow/src/ui/viewer/manager.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `ViewerManager` | projects/agentic-workflow/src/ui/viewer/manager.rs | struct | pub | 5 |  |
| `annotations_path` | projects/agentic-workflow/src/ui/viewer/manager.rs | function | pub | 104 | annotations_path(&self) -> &Path |
| `change_dir` | projects/agentic-workflow/src/ui/viewer/manager.rs | function | pub | 46 | change_dir(&self) -> &Path |
| `change_exists` | projects/agentic-workflow/src/ui/viewer/manager.rs | function | pub | 41 | change_exists(&self) -> bool |
| `generate_project_tree` | projects/agentic-workflow/src/ui/viewer/manager.rs | function | pub | 174 | generate_project_tree(&self) -> Result<FileNode, ViewerError> |
| `list_files` | projects/agentic-workflow/src/ui/viewer/manager.rs | function | pub | 134 | list_files(&self) -> Vec<FileInfo> |
| `load_annotations` | projects/agentic-workflow/src/ui/viewer/manager.rs | function | pub | 94 | load_annotations(&self) -> AnnotationResult<AnnotationStore> |
| `load_file` | projects/agentic-workflow/src/ui/viewer/manager.rs | function | pub | 54 | load_file(&self, filename: &str) -> Result<FileLoadResponse, ViewerError> |
| `new` | projects/agentic-workflow/src/ui/viewer/manager.rs | function | pub | 17 | new(change_id: &str, project_root: &Path) -> Self |
| `new_project_manager` | projects/agentic-workflow/src/ui/viewer/manager.rs | function | pub | 29 | new_project_manager(project_root: &Path) -> Self |
| `save_annotations` | projects/agentic-workflow/src/ui/viewer/manager.rs | function | pub | 99 | save_annotations(&self, store: &AnnotationStore) -> AnnotationResult<()> |
| `update_phase` | projects/agentic-workflow/src/ui/viewer/manager.rs | function | pub | 285 | update_phase(&self, new_phase: &str) -> Result<(), ViewerError> |
## Schema
<!-- type: schema lang: yaml -->

```yaml
definitions:
  FileLoadResponse:
    type: object
    required: [content, annotations, exists]
    description: Response from loading a file, includes content and annotations.
    properties:
      content:
        type: string
        description: "Rendered HTML content."
      annotations:
        type: array
        items: { type: object }
        x-rust-type: "Vec<crate::models::Annotation>"
        description: "Annotations for this file."
      exists:
        type: boolean
        description: "Whether the file exists."
    x-rust-struct:
      derive: [Debug, Clone, "serde::Serialize"]

  FileInfo:
    type: object
    required: [name, exists]
    description: Information about a file in the change directory.
    properties:
      name:
        type: string
        description: "Filename."
      exists:
        type: boolean
        description: "Whether the file exists."
    x-rust-struct:
      derive: [Debug, Clone, "serde::Serialize"]

  FileNode:
    type: object
    required: [id, name, path, is_directory]
    description: Hierarchical node for project-level tree view.
    properties:
      id:
        type: string
        description: "Unique identifier for the node."
      name:
        type: string
        description: "Display name of the file or directory."
      path:
        type: string
        description: "Relative path from the genesis root."
      is_directory:
        type: boolean
        description: "Whether this is a directory."
      children:
        type: array
        items: { type: object }
        x-rust-type: "Option<Vec<FileNode>>"
        x-serde-skip-if: "Option::is_none"
        description: "Child nodes (None if not a directory)."
    x-rust-struct:
      derive: [Debug, Clone, "serde::Serialize", "serde::Deserialize"]

  ViewerError:
    type: object
    description: Errors that can occur in the viewer manager.
    x-rust-enum:
      derive: [Debug, "thiserror::Error"]
      variants:
        - name: PathTraversal
          kind: tuple
          error: "Path traversal attempt detected: {0}"
          fields:
            - { rust_type: String }
        - name: FileNotAllowed
          kind: tuple
          error: "File not allowed: {0}"
          fields:
            - { rust_type: String }
        - name: FileRead
          kind: tuple
          error: "Failed to read file '{0}': {1}"
          fields:
            - { rust_type: String }
            - { rust_type: "std::io::Error" }
        - name: Annotation
          kind: tuple
          error: "Annotation error: {0}"
          fields:
            - { rust_type: AnnotationError, error_from: true }
        - name: ChangeNotFound
          kind: tuple
          error: "Change not found: {0}"
          fields:
            - { rust_type: String }
        - name: IoError
          kind: tuple
          error: "IO error: {0}"
          fields:
            - { rust_type: "std::io::Error", error_from: true }
```

## Source
<!-- type: source lang: rust -->

```rust
/// Viewer manager for handling file operations
pub struct ViewerManager {
    /// Change ID being viewed
    change_id: String,
    /// Path to the change directory
    change_dir: PathBuf,
    /// Path to annotations file
    annotations_path: PathBuf,
}

impl ViewerManager {
    /// Create a new viewer manager for a change
    pub fn new(change_id: &str, project_root: &Path) -> Self {
        let change_dir = project_root.join(".aw/changes").join(change_id);
        let annotations_path = change_dir.join("annotations.json");

        Self {
            change_id: change_id.to_string(),
            change_dir,
            annotations_path,
        }
    }

    /// Create a new viewer manager for project-level browsing
    pub fn new_project_manager(project_root: &Path) -> Self {
        let sdd_dir = crate::shared::workspace::workspace_path(project_root);
        let annotations_path = sdd_dir.join("project_annotations.json");

        Self {
            change_id: "project".to_string(),
            change_dir: sdd_dir,
            annotations_path,
        }
    }

    /// Check if the change directory exists
    pub fn change_exists(&self) -> bool {
        self.change_dir.exists() && self.change_dir.is_dir()
    }

    /// Get the change directory path
    pub fn change_dir(&self) -> &Path {
        &self.change_dir
    }

    /// Load a file and its annotations
    ///
    /// Returns rendered HTML content and associated annotations.
    /// Returns a "not found" placeholder if the file doesn't exist.
    pub fn load_file(&self, filename: &str) -> Result<FileLoadResponse, ViewerError> {
        // Validate filename to prevent path traversal
        self.validate_filename(filename)?;

        let file_path = self.change_dir.join(filename);
        let annotations = self.load_annotations()?;

        if !file_path.exists() {
            return Ok(FileLoadResponse {
                content: render_not_found_html(filename),
                annotations: annotations
                    .for_file(filename)
                    .into_iter()
                    .cloned()
                    .collect(),
                exists: false,
            });
        }

        let content = fs::read_to_string(&file_path)
            .map_err(|e| ViewerError::FileRead(filename.to_string(), e))?;

        let html = if filename.ends_with(".yaml") || filename.ends_with(".yml") {
            render_yaml_to_html(&content)
        } else {
            render_markdown_to_html(&content)
        };

        Ok(FileLoadResponse {
            content: html,
            annotations: annotations
                .for_file(filename)
                .into_iter()
                .cloned()
                .collect(),
            exists: true,
        })
    }

    /// Load annotations from the annotations file
    pub fn load_annotations(&self) -> AnnotationResult<AnnotationStore> {
        AnnotationStore::load(&self.annotations_path, &self.change_id)
    }

    /// Save annotations to the annotations file
    pub fn save_annotations(&self, store: &AnnotationStore) -> AnnotationResult<()> {
        store.save(&self.annotations_path)
    }

    /// Get the annotations file path
    pub fn annotations_path(&self) -> &Path {
        &self.annotations_path
    }

    /// Validate that a filename is allowed and doesn't contain path traversal
    fn validate_filename(&self, filename: &str) -> Result<(), ViewerError> {
        // Check for path traversal attempts (but allow specs/ prefix)
        if filename.contains("..") || filename.contains('\\') {
            return Err(ViewerError::PathTraversal(filename.to_string()));
        }

        // Check if it's a base file or specs file
        let is_base_file = BASE_ALLOWED_FILES.contains(&filename);
        let is_specs_file =
            filename.starts_with("specs/") && filename.ends_with(".md") && !filename.contains("..");

        if !is_base_file && !is_specs_file {
            return Err(ViewerError::FileNotAllowed(filename.to_string()));
        }

        // Double-check the resolved path is within change_dir
        let resolved = self.change_dir.join(filename);
        if !resolved.starts_with(&self.change_dir) {
            return Err(ViewerError::PathTraversal(filename.to_string()));
        }

        Ok(())
    }

    /// List all available files in the change directory
    pub fn list_files(&self) -> Vec<FileInfo> {
        let mut files: Vec<FileInfo> = BASE_ALLOWED_FILES
            .iter()
            .map(|&name| {
                let path = self.change_dir.join(name);
                FileInfo {
                    name: name.to_string(),
                    exists: path.exists(),
                }
            })
            .collect();

        // Add specs/*.md files
        let specs_dir = self.change_dir.join("specs");
        if specs_dir.exists() && specs_dir.is_dir() {
            if let Ok(entries) = fs::read_dir(&specs_dir) {
                for entry in entries.filter_map(|e| e.ok()) {
                    let path = entry.path();
                    if path.extension().map_or(false, |e| e == "md") {
                        if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                            // Skip skeleton files
                            if !name.starts_with('_') {
                                files.push(FileInfo {
                                    name: format!("specs/{}", name),
                                    exists: true,
                                });
                            }
                        }
                    }
                }
            }
        }

        files
    }

    /// Generate a tree structure for project-level browsing
    ///
    /// Scans the cclab/ directory and creates a hierarchical tree of files
    /// including specs/, knowledge/, archive/, config, and project.md
    pub fn generate_project_tree(&self) -> Result<FileNode, ViewerError> {
        let sdd_dir = &self.change_dir;

        // Root node representing the genesis directory
        let mut root = FileNode {
            id: "genesis-root".to_string(),
            name: "SDD".to_string(),
            path: ".".to_string(),
            is_directory: true,
            children: Some(Vec::new()),
        };

        // Directories to include in the tree
        let dirs_to_scan = vec!["specs", "knowledge", "archive", "changes"];

        if let Some(ref mut children) = root.children {
            for dir_name in dirs_to_scan {
                let dir_path = sdd_dir.join(dir_name);
                if dir_path.exists() && dir_path.is_dir() {
                    if let Ok(node) = self.scan_directory(&dir_path, dir_name, &dir_path) {
                        children.push(node);
                    }
                }
            }

            // Add top-level files (project.md, config.yaml, etc.)
            let top_level_files = vec!["project.md", "config.yaml"];
            for file_name in top_level_files {
                let file_path = sdd_dir.join(file_name);
                if file_path.exists() && file_path.is_file() {
                    let node = FileNode {
                        id: format!("file-{}", file_name),
                        name: file_name.to_string(),
                        path: file_name.to_string(),
                        is_directory: false,
                        children: None,
                    };
                    children.push(node);
                }
            }
        }

        Ok(root)
    }

    /// Recursively scan a directory and build a tree node
    fn scan_directory(
        &self,
        dir_path: &Path,
        dir_name: &str,
        root_dir: &Path,
    ) -> Result<FileNode, ViewerError> {
        let mut node = FileNode {
            id: format!("dir-{}", dir_name),
            name: dir_name.to_string(),
            path: dir_path
                .strip_prefix(root_dir)
                .ok()
                .and_then(|p| p.to_str())
                .unwrap_or(dir_name)
                .to_string(),
            is_directory: true,
            children: Some(Vec::new()),
        };

        if let Ok(entries) = fs::read_dir(dir_path) {
            if let Some(ref mut children) = node.children {
                for entry in entries.filter_map(|e| e.ok()) {
                    let path = entry.path();

                    if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                        // Skip hidden files
                        if name.starts_with('.') {
                            continue;
                        }

                        if path.is_dir() {
                            // Recursively scan subdirectories
                            if let Ok(child_node) = self.scan_directory(&path, name, root_dir) {
                                children.push(child_node);
                            }
                        } else if path
                            .extension()
                            .map_or(false, |e| e == "md" || e == "yaml" || e == "yml")
                        {
                            // Include markdown and YAML files
                            let relative_path = path
                                .strip_prefix(root_dir)
                                .ok()
                                .and_then(|p| p.to_str())
                                .unwrap_or(name)
                                .to_string();

                            let child_node = FileNode {
                                id: format!("file-{}", relative_path.replace('/', "-")),
                                name: name.to_string(),
                                path: relative_path,
                                is_directory: false,
                                children: None,
                            };
                            children.push(child_node);
                        }
                    }
                }
            }
        }

        Ok(node)
    }

    /// Update the phase in STATE.yaml
    pub fn update_phase(&self, new_phase: &str) -> Result<(), ViewerError> {
        let state_path = self.change_dir.join("STATE.yaml");

        if !state_path.exists() {
            return Err(
                std::io::Error::new(std::io::ErrorKind::NotFound, "STATE.yaml not found").into(),
            );
        }

        let content = fs::read_to_string(&state_path)?;

        // Simple regex-like replacement for phase field
        let updated = content
            .lines()
            .map(|line| {
                if line.starts_with("phase:") {
                    format!("phase: {}", new_phase)
                } else {
                    line.to_string()
                }
            })
            .collect::<Vec<_>>()
            .join("\n");

        // Atomic write
        let temp_path = state_path.with_extension("yaml.tmp");
        fs::write(&temp_path, &updated)?;
        fs::rename(&temp_path, &state_path)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn setup_test_change(dir: &TempDir, change_id: &str) -> PathBuf {
        let change_dir = dir.path().join(".aw/changes").join(change_id);
        fs::create_dir_all(&change_dir).unwrap();
        change_dir
    }

    #[test]
    fn test_viewer_manager_new() {
        let temp_dir = TempDir::new().unwrap();
        let manager = ViewerManager::new("my-change", temp_dir.path());

        assert_eq!(manager.change_id, "my-change");
        assert!(manager.change_dir.ends_with(".aw/changes/my-change"));
    }

    #[test]
    fn test_change_exists() {
        let temp_dir = TempDir::new().unwrap();
        setup_test_change(&temp_dir, "existing-change");

        let manager = ViewerManager::new("existing-change", temp_dir.path());
        assert!(manager.change_exists());

        let manager2 = ViewerManager::new("nonexistent", temp_dir.path());
        assert!(!manager2.change_exists());
    }

    #[test]
    fn test_load_file_existing() {
        let temp_dir = TempDir::new().unwrap();
        let change_dir = setup_test_change(&temp_dir, "test-change");

        fs::write(
            change_dir.join("proposal.md"),
            "# My Proposal\n\nThis is content.",
        )
        .unwrap();

        let manager = ViewerManager::new("test-change", temp_dir.path());
        let response = manager.load_file("proposal.md").unwrap();

        assert!(response.exists);
        assert!(response.content.contains("<h1"));
        assert!(response.content.contains("My Proposal"));
    }

    #[test]
    fn test_load_file_missing() {
        let temp_dir = TempDir::new().unwrap();
        setup_test_change(&temp_dir, "test-change");

        let manager = ViewerManager::new("test-change", temp_dir.path());
        let response = manager.load_file("CHALLENGE.md").unwrap();

        assert!(!response.exists);
        assert!(response.content.contains("File not found"));
        assert!(response.content.contains("CHALLENGE.md"));
    }

    #[test]
    fn test_load_yaml_file() {
        let temp_dir = TempDir::new().unwrap();
        let change_dir = setup_test_change(&temp_dir, "test-change");

        fs::write(
            change_dir.join("STATE.yaml"),
            "phase: Proposed\nchange_id: test",
        )
        .unwrap();

        let manager = ViewerManager::new("test-change", temp_dir.path());
        let response = manager.load_file("STATE.yaml").unwrap();

        assert!(response.exists);
        assert!(response.content.contains("language-yaml"));
        assert!(response.content.contains("phase: Proposed"));
    }

    #[test]
    fn test_path_traversal_rejected() {
        let temp_dir = TempDir::new().unwrap();
        setup_test_change(&temp_dir, "test-change");

        let manager = ViewerManager::new("test-change", temp_dir.path());

        // Test various path traversal attempts
        assert!(matches!(
            manager.load_file("../secret.txt"),
            Err(ViewerError::PathTraversal(_))
        ));
        assert!(matches!(
            manager.load_file("..\\secret.txt"),
            Err(ViewerError::PathTraversal(_))
        ));
        // foo/bar.txt is not a path traversal, but it's not allowed (not specs/*.md)
        assert!(matches!(
            manager.load_file("foo/bar.txt"),
            Err(ViewerError::FileNotAllowed(_))
        ));
    }

    #[test]
    fn test_file_not_allowed() {
        let temp_dir = TempDir::new().unwrap();
        setup_test_change(&temp_dir, "test-change");

        let manager = ViewerManager::new("test-change", temp_dir.path());

        assert!(matches!(
            manager.load_file("secret.txt"),
            Err(ViewerError::FileNotAllowed(_))
        ));
        assert!(matches!(
            manager.load_file("config.toml"),
            Err(ViewerError::FileNotAllowed(_))
        ));
    }

    #[test]
    fn test_list_files() {
        let temp_dir = TempDir::new().unwrap();
        let change_dir = setup_test_change(&temp_dir, "test-change");

        fs::write(change_dir.join("proposal.md"), "content").unwrap();
        fs::write(change_dir.join("tasks.md"), "content").unwrap();

        let manager = ViewerManager::new("test-change", temp_dir.path());
        let files = manager.list_files();

        assert_eq!(files.len(), 4); // All BASE_ALLOWED_FILES (no specs dir)

        let proposal = files.iter().find(|f| f.name == "proposal.md").unwrap();
        assert!(proposal.exists);

        let challenge = files.iter().find(|f| f.name == "CHALLENGE.md").unwrap();
        assert!(!challenge.exists);
    }

    #[test]
    fn test_load_file_with_annotations() {
        let temp_dir = TempDir::new().unwrap();
        let change_dir = setup_test_change(&temp_dir, "test-change");

        fs::write(change_dir.join("proposal.md"), "# Heading").unwrap();

        // Create annotations
        let mut store = AnnotationStore::new("test-change");
        store.add(crate::models::Annotation::new(
            "proposal.md",
            "heading",
            "comment",
            "user",
        ));
        store.add(crate::models::Annotation::new(
            "CHALLENGE.md",
            "issue",
            "other",
            "user",
        ));

        let annotations_path = change_dir.join("annotations.json");
        store.save(&annotations_path).unwrap();

        let manager = ViewerManager::new("test-change", temp_dir.path());
        let response = manager.load_file("proposal.md").unwrap();

        assert_eq!(response.annotations.len(), 1);
        assert_eq!(response.annotations[0].file, "proposal.md");
    }

    #[test]
    fn test_validate_filename_empty() {
        let temp_dir = TempDir::new().unwrap();
        setup_test_change(&temp_dir, "test-change");

        let manager = ViewerManager::new("test-change", temp_dir.path());

        // Empty string should fail
        assert!(manager.validate_filename("").is_err());
    }
}
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/ui/viewer/manager.rs
    action: modify
    section: schema
    impl_mode: codegen
    replaces:
      - FileLoadResponse
      - FileInfo
      - FileNode
      - ViewerError
    description: |
      Codegen replaces the 4 type declarations only. ViewerError's
      Display impl is auto-generated by the thiserror derive macro
      from the per-variant `#[error("...")]` attrs — no separate impl
      block needed.
  - path: projects/agentic-workflow/src/ui/viewer/manager.rs
    action: modify
    section: source
    impl_mode: codegen
    description: |
      Regenerate ViewerManager behavior, helper functions, and tests from the
      source section until a richer behavior template can replace the raw source.
```

# Reviews

## Review 1
<!-- type: review lang: markdown -->
**Verdict:** approved

- [overview] Accurately enumerates the 4 codegen-tracked types and the three patterns. Hand-written boundary explicit.
- [schema] Schema correct: 3 structs use established conventions; ViewerError uses thiserror derive + per-variant error templates + error_from on the two foreign-error fields. Multi-field tuple variant FileRead exercises the join logic.
- [changes] Two-entry split partitions codegen (4 types) from hand-written (const, ViewerManager + impl, tests).
