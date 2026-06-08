// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/interfaces/spec_ir/migration.md#source
// CODEGEN-BEGIN
//! Migration architecture: detect legacy vs YAML IR flow, enforce per-change scoping.
//!
//! Implements migration-architecture spec:
//! - R1: Legacy Path Detection — identify legacy vs YAML IR flow
//! - R2: YAML Path Enforcement — new changes default to YAML IR
//! - R3: Deprecation Warnings — warn when legacy tools are used
//! - R4: Dual-Path Support — per-change scoping, no mixing
//!
//! ## Design: manifest-presence detection
//!
//! Flow detection uses **manifest-presence** (YAML files inside `spec_ir/`),
//! not mere directory-presence. An empty `spec_ir/` directory is treated as
//! Legacy because no manifests have been generated yet. This avoids false
//! positives from partially initialized changes.
//!
//! ## Integration points
//!
//! Call sites for production enforcement:
//! - `run_change` implement flow → call [`guard_tool_invocation`] before
//!   dispatching legacy generate spec-generation tools
//! - Change bootstrap → call [`should_use_yaml_ir_for_new_change`] to set
//!   the default pipeline
//! - MCP tool handlers → call [`analyze`] and surface `deprecation_warning`
//!   in tool responses when present

use std::path::Path;

use super::orchestrator;

/// The pipeline flow detected for a change.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/spec_ir/migration.md#schema
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FlowKind {
    /// New YAML IR pipeline (spec_ir/ directory exists with manifests).
    YamlIr,
    /// Legacy relay pipeline (no spec_ir/ directory).
    Legacy,
}

/// Result of migration analysis for a change directory.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/spec_ir/migration.md#schema
#[derive(Debug, Clone)]
pub struct MigrationResult {
    /// Which flow this change uses.
    pub flow: FlowKind,
    /// Deprecation warning message, if any (R3).
    pub deprecation_warning: Option<String>,
}
/// Detect which pipeline flow a change is using (R1).
///
/// - If `spec_ir/` contains YAML manifests → `YamlIr`
/// - Otherwise → `Legacy`
/// @spec projects/agentic-workflow/tech-design/core/interfaces/spec_ir/migration.md#source
pub fn detect_flow(change_dir: &Path) -> FlowKind {
    if orchestrator::has_yaml_ir(change_dir) {
        FlowKind::YamlIr
    } else {
        FlowKind::Legacy
    }
}

/// Analyze a change for migration status (R1 + R3).
///
/// Returns the detected flow and any deprecation warnings.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/spec_ir/migration.md#source
pub fn analyze(change_dir: &Path) -> MigrationResult {
    let flow = detect_flow(change_dir);
    let deprecation_warning = if flow == FlowKind::Legacy {
        Some(
            "This change uses the legacy relay pipeline. \
             New changes should use YAML IR (spec_ir/). \
             See .aw/tech-design/sdd/spec-to-code/ for migration details."
                .to_string(),
        )
    } else {
        None
    };

    MigrationResult {
        flow,
        deprecation_warning,
    }
}

/// Check if a legacy tool invocation should be rejected (R4).
///
/// Returns an error message if the change already has YAML IR manifests
/// and a legacy tool is being used — mixing flows is not allowed.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/spec_ir/migration.md#source
pub fn check_mixed_flow(change_dir: &Path, using_legacy_tool: bool) -> Result<(), String> {
    if using_legacy_tool && orchestrator::has_yaml_ir(change_dir) {
        Err(
            "Cannot use legacy generate tools on a change that already has YAML IR manifests. \
             Mixing flows is not supported. Remove spec_ir/ to revert to legacy, \
             or use YAML IR tools instead."
                .to_string(),
        )
    } else {
        Ok(())
    }
}

/// Check if a new change should default to YAML IR (R2).
///
/// Always returns true — all new changes use the YAML IR pipeline.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/spec_ir/migration.md#source
pub fn should_use_yaml_ir_for_new_change() -> bool {
    true
}

/// Guard for tool invocations — combines R3 (warnings) and R4 (mixed rejection).
///
/// Call this before invoking any legacy generate tool on a change directory.
/// Returns `Ok(warning)` if invocation is allowed (with optional deprecation
/// warning), or `Err(message)` if the invocation is rejected due to flow mixing.
///
/// This is the primary integration point for MCP tool handlers and the
/// run_change implement flow.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/spec_ir/migration.md#source
pub fn guard_tool_invocation(
    change_dir: &Path,
    is_legacy_tool: bool,
) -> Result<Option<String>, String> {
    // R4: reject mixed flows first
    check_mixed_flow(change_dir, is_legacy_tool)?;

    // R3: emit deprecation warning for legacy usage
    if is_legacy_tool {
        let result = analyze(change_dir);
        Ok(result.deprecation_warning)
    } else {
        Ok(None)
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn setup_change_dir() -> (TempDir, std::path::PathBuf) {
        let tmp = TempDir::new().unwrap();
        let change_dir = tmp.path().to_path_buf();
        (tmp, change_dir)
    }

    fn write_manifest(change_dir: &Path) {
        let spec_ir_dir = change_dir.join("spec_ir");
        std::fs::create_dir_all(&spec_ir_dir).unwrap();
        std::fs::write(
            spec_ir_dir.join("api.yaml"),
            "apiVersion: cclab.dev/v1\nkind: Api\nmetadata:\n  name: test\n  change_id: t\nspec: {}\n",
        )
        .unwrap();
    }

    // -- R1: Legacy Path Detection --

    #[test]
    fn test_detect_flow_yaml_ir() {
        let (_tmp, change_dir) = setup_change_dir();
        write_manifest(&change_dir);
        assert_eq!(detect_flow(&change_dir), FlowKind::YamlIr);
    }

    #[test]
    fn test_detect_flow_legacy() {
        let (_tmp, change_dir) = setup_change_dir();
        assert_eq!(detect_flow(&change_dir), FlowKind::Legacy);
    }

    #[test]
    fn test_detect_flow_empty_spec_ir_is_legacy() {
        let (_tmp, change_dir) = setup_change_dir();
        std::fs::create_dir_all(change_dir.join("spec_ir")).unwrap();
        assert_eq!(detect_flow(&change_dir), FlowKind::Legacy);
    }

    // -- R2: YAML Path Enforcement --

    #[test]
    fn test_new_changes_default_to_yaml_ir() {
        assert!(should_use_yaml_ir_for_new_change());
    }

    // -- R3: Deprecation Warnings --

    #[test]
    fn test_analyze_legacy_emits_warning() {
        let (_tmp, change_dir) = setup_change_dir();
        let result = analyze(&change_dir);
        assert_eq!(result.flow, FlowKind::Legacy);
        assert!(result.deprecation_warning.is_some());
        assert!(result.deprecation_warning.unwrap().contains("legacy"));
    }

    #[test]
    fn test_analyze_yaml_ir_no_warning() {
        let (_tmp, change_dir) = setup_change_dir();
        write_manifest(&change_dir);
        let result = analyze(&change_dir);
        assert_eq!(result.flow, FlowKind::YamlIr);
        assert!(result.deprecation_warning.is_none());
    }

    // -- R4: Dual-Path Support (no mixing) --

    #[test]
    fn test_mixed_flow_rejected() {
        let (_tmp, change_dir) = setup_change_dir();
        write_manifest(&change_dir);
        let result = check_mixed_flow(&change_dir, true);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .contains("Mixing flows is not supported"));
    }

    #[test]
    fn test_legacy_tool_on_legacy_change_ok() {
        let (_tmp, change_dir) = setup_change_dir();
        assert!(check_mixed_flow(&change_dir, true).is_ok());
    }

    #[test]
    fn test_yaml_tool_on_yaml_change_ok() {
        let (_tmp, change_dir) = setup_change_dir();
        write_manifest(&change_dir);
        assert!(check_mixed_flow(&change_dir, false).is_ok());
    }

    #[test]
    fn test_non_legacy_tool_on_legacy_change_ok() {
        let (_tmp, change_dir) = setup_change_dir();
        assert!(check_mixed_flow(&change_dir, false).is_ok());
    }

    // -- guard_tool_invocation (R3+R4 combined) --

    #[test]
    fn test_guard_legacy_tool_on_legacy_change_warns() {
        let (_tmp, change_dir) = setup_change_dir();
        let result = guard_tool_invocation(&change_dir, true);
        assert!(result.is_ok());
        let warning = result.unwrap();
        assert!(warning.is_some());
        assert!(warning.unwrap().contains("legacy"));
    }

    #[test]
    fn test_guard_legacy_tool_on_yaml_change_rejects() {
        let (_tmp, change_dir) = setup_change_dir();
        write_manifest(&change_dir);
        let result = guard_tool_invocation(&change_dir, true);
        assert!(result.is_err());
    }

    #[test]
    fn test_guard_yaml_tool_no_warning() {
        let (_tmp, change_dir) = setup_change_dir();
        let result = guard_tool_invocation(&change_dir, false);
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }
}

// CODEGEN-END
