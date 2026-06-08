// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/interfaces/spec_ir/orchestrator.md#source
// CODEGEN-BEGIN
//! Codegen orchestration: detect YAML IR, choose pipeline strategy.
//!
//! Implements genesis-codegen-orchestration spec:
//! - R1: YAML Detection — scan `spec_ir/` for `.yaml` manifests
//! - R2: Lens Invocation — collect manifest paths for Lens codegen
//! - R3: Fallback Logic — legacy pipeline or error when no IR found

use std::path::{Path, PathBuf};

/// Strategy resolved by the orchestrator.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/spec_ir/orchestrator.md#schema
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CodegenStrategy {
    /// YAML IR files found — invoke Lens with these paths.
    YamlPipeline { manifest_paths: Vec<PathBuf> },
    /// No YAML IR, but legacy fallback is allowed.
    LegacyFallback,
    /// No YAML IR and legacy is disabled — error state.
    NoIrError,
}
/// Detect YAML IR manifest files under `<change_dir>/spec_ir/`.
///
/// Returns a sorted list of `.yaml` file paths found in the directory.
/// Returns an empty vec if the directory does not exist.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/spec_ir/orchestrator.md#source
pub fn detect_yaml_ir(change_dir: &Path) -> Vec<PathBuf> {
    let spec_ir_dir = change_dir.join("spec_ir");
    if !spec_ir_dir.is_dir() {
        return Vec::new();
    }

    let mut paths: Vec<PathBuf> = std::fs::read_dir(&spec_ir_dir)
        .into_iter()
        .flatten()
        .filter_map(|entry| entry.ok())
        .filter(|entry| {
            let path = entry.path();
            // Skip symlinks for safety
            if let Ok(meta) = std::fs::symlink_metadata(&path) {
                if meta.file_type().is_symlink() {
                    return false;
                }
            }
            path.extension()
                .map(|ext| ext == "yaml" || ext == "yml")
                .unwrap_or(false)
        })
        .map(|entry| entry.path())
        .collect();

    paths.sort();
    paths
}

/// Resolve the codegen strategy for a change directory.
///
/// 1. If `spec_ir/*.yaml` files exist → `YamlPipeline` (R1 + R2)
/// 2. If no YAML IR and `legacy_allowed` → `LegacyFallback` (R3)
/// 3. If no YAML IR and legacy disabled → `NoIrError` (R3)
/// @spec projects/agentic-workflow/tech-design/core/interfaces/spec_ir/orchestrator.md#source
pub fn resolve_strategy(change_dir: &Path, legacy_allowed: bool) -> CodegenStrategy {
    let manifests = detect_yaml_ir(change_dir);

    if !manifests.is_empty() {
        CodegenStrategy::YamlPipeline {
            manifest_paths: manifests,
        }
    } else if legacy_allowed {
        CodegenStrategy::LegacyFallback
    } else {
        CodegenStrategy::NoIrError
    }
}

/// Check whether a change directory has any YAML IR manifests.
///
/// Convenience wrapper around `detect_yaml_ir` for boolean checks.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/spec_ir/orchestrator.md#source
pub fn has_yaml_ir(change_dir: &Path) -> bool {
    !detect_yaml_ir(change_dir).is_empty()
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn setup_change_dir() -> (TempDir, PathBuf) {
        let tmp = TempDir::new().unwrap();
        let change_dir = tmp.path().to_path_buf();
        (tmp, change_dir)
    }

    fn write_manifest(change_dir: &Path, filename: &str) {
        let spec_ir_dir = change_dir.join("spec_ir");
        std::fs::create_dir_all(&spec_ir_dir).unwrap();
        let content = format!(
            "apiVersion: cclab.dev/v1\nkind: Api\nmetadata:\n  name: {}\n  change_id: test\nspec: {{}}\n",
            filename.trim_end_matches(".yaml")
        );
        std::fs::write(spec_ir_dir.join(filename), content).unwrap();
    }

    // -- R1: YAML Detection --

    #[test]
    fn test_detect_yaml_ir_no_dir() {
        let (_tmp, change_dir) = setup_change_dir();
        assert!(detect_yaml_ir(&change_dir).is_empty());
    }

    #[test]
    fn test_detect_yaml_ir_empty_dir() {
        let (_tmp, change_dir) = setup_change_dir();
        std::fs::create_dir_all(change_dir.join("spec_ir")).unwrap();
        assert!(detect_yaml_ir(&change_dir).is_empty());
    }

    #[test]
    fn test_detect_yaml_ir_finds_manifests() {
        let (_tmp, change_dir) = setup_change_dir();
        write_manifest(&change_dir, "api.yaml");
        write_manifest(&change_dir, "flowchart.yaml");

        let paths = detect_yaml_ir(&change_dir);
        assert_eq!(paths.len(), 2);
        assert!(paths[0]
            .file_name()
            .unwrap()
            .to_str()
            .unwrap()
            .ends_with(".yaml"));
    }

    #[test]
    fn test_detect_yaml_ir_ignores_non_yaml() {
        let (_tmp, change_dir) = setup_change_dir();
        let spec_ir_dir = change_dir.join("spec_ir");
        std::fs::create_dir_all(&spec_ir_dir).unwrap();
        std::fs::write(spec_ir_dir.join("readme.md"), "# Not a manifest").unwrap();
        std::fs::write(spec_ir_dir.join("data.json"), "{}").unwrap();
        write_manifest(&change_dir, "api.yaml");

        let paths = detect_yaml_ir(&change_dir);
        assert_eq!(paths.len(), 1);
    }

    #[test]
    fn test_detect_yaml_ir_sorted() {
        let (_tmp, change_dir) = setup_change_dir();
        write_manifest(&change_dir, "z_last.yaml");
        write_manifest(&change_dir, "a_first.yaml");

        let paths = detect_yaml_ir(&change_dir);
        assert_eq!(paths.len(), 2);
        let names: Vec<&str> = paths
            .iter()
            .map(|p| p.file_name().unwrap().to_str().unwrap())
            .collect();
        assert_eq!(names, vec!["a_first.yaml", "z_last.yaml"]);
    }

    // -- R2: Lens Invocation (YamlPipeline strategy) --

    #[test]
    fn test_resolve_strategy_yaml_pipeline() {
        let (_tmp, change_dir) = setup_change_dir();
        write_manifest(&change_dir, "user-service.yaml");

        let strategy = resolve_strategy(&change_dir, false);
        match strategy {
            CodegenStrategy::YamlPipeline { manifest_paths } => {
                assert_eq!(manifest_paths.len(), 1);
            }
            other => panic!("Expected YamlPipeline, got {:?}", other),
        }
    }

    #[test]
    fn test_resolve_strategy_yaml_pipeline_ignores_legacy_flag() {
        let (_tmp, change_dir) = setup_change_dir();
        write_manifest(&change_dir, "api.yaml");

        // Even with legacy_allowed=true, YAML IR takes priority
        let strategy = resolve_strategy(&change_dir, true);
        assert!(matches!(strategy, CodegenStrategy::YamlPipeline { .. }));
    }

    // -- R3: Fallback Logic --

    #[test]
    fn test_resolve_strategy_legacy_fallback() {
        let (_tmp, change_dir) = setup_change_dir();
        // No spec_ir dir at all
        let strategy = resolve_strategy(&change_dir, true);
        assert_eq!(strategy, CodegenStrategy::LegacyFallback);
    }

    #[test]
    fn test_resolve_strategy_no_ir_error() {
        let (_tmp, change_dir) = setup_change_dir();
        let strategy = resolve_strategy(&change_dir, false);
        assert_eq!(strategy, CodegenStrategy::NoIrError);
    }

    #[test]
    fn test_resolve_strategy_empty_spec_ir_legacy() {
        let (_tmp, change_dir) = setup_change_dir();
        std::fs::create_dir_all(change_dir.join("spec_ir")).unwrap();
        let strategy = resolve_strategy(&change_dir, true);
        assert_eq!(strategy, CodegenStrategy::LegacyFallback);
    }

    #[test]
    fn test_resolve_strategy_empty_spec_ir_no_legacy() {
        let (_tmp, change_dir) = setup_change_dir();
        std::fs::create_dir_all(change_dir.join("spec_ir")).unwrap();
        let strategy = resolve_strategy(&change_dir, false);
        assert_eq!(strategy, CodegenStrategy::NoIrError);
    }

    // -- has_yaml_ir convenience --

    #[test]
    fn test_has_yaml_ir_true() {
        let (_tmp, change_dir) = setup_change_dir();
        write_manifest(&change_dir, "service.yaml");
        assert!(has_yaml_ir(&change_dir));
    }

    #[test]
    fn test_has_yaml_ir_false() {
        let (_tmp, change_dir) = setup_change_dir();
        assert!(!has_yaml_ir(&change_dir));
    }
}

// CODEGEN-END
