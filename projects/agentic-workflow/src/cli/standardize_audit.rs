// SPEC-MANAGED: projects/agentic-workflow/tech-design/surface/interfaces/src/standardize_audit.md#source
// CODEGEN-BEGIN
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

use super::StandardizeActionKind;

const AUDIT_DIR: &str = ".aw/standardize/audit";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
/// @spec projects/agentic-workflow/tech-design/surface/interfaces/src/standardize_audit.md#source
pub(crate) enum PreservationSurfaceKind {
    Route,
    Command,
    Api,
    Doc,
    GeneratedSource,
    Behavior,
    Accessibility,
    Operations,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// @spec projects/agentic-workflow/tech-design/surface/interfaces/src/standardize_audit.md#source
pub(crate) struct PreservationSurface {
    pub kind: PreservationSurfaceKind,
    pub name: String,
    pub preserve: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
/// @spec projects/agentic-workflow/tech-design/surface/interfaces/src/standardize_audit.md#source
pub(crate) enum ModernizationRisk {
    Low,
    Medium,
    High,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// @spec projects/agentic-workflow/tech-design/surface/interfaces/src/standardize_audit.md#source
pub(crate) struct SafeModernizationLever {
    pub name: String,
    pub risk: ModernizationRisk,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// @spec projects/agentic-workflow/tech-design/surface/interfaces/src/standardize_audit.md#source
pub(crate) struct PreservationAudit {
    pub project: String,
    pub scope: Option<String>,
    pub surfaces: Vec<PreservationSurface>,
    pub quality_debt: Vec<String>,
    pub safe_levers: Vec<SafeModernizationLever>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
/// @spec projects/agentic-workflow/tech-design/surface/interfaces/src/standardize_audit.md#source
pub(crate) struct StandardizeAuditDecision {
    pub audit_required: bool,
    pub audit_path: String,
    pub surfaces_to_preserve: Vec<String>,
}

/// @spec projects/agentic-workflow/tech-design/surface/interfaces/src/standardize_audit.md#source
pub(crate) fn audit_path(project_root: &Path, project: &str) -> PathBuf {
    project_root
        .join(AUDIT_DIR)
        .join(format!("{}.json", sanitize_project_key(project)))
}

/// @spec projects/agentic-workflow/tech-design/surface/interfaces/src/standardize_audit.md#source
pub(crate) fn evaluate_audit_decision(
    project_root: &Path,
    project: &str,
    scopes: &[String],
    action_kind: StandardizeActionKind,
) -> StandardizeAuditDecision {
    let path = audit_path(project_root, project);
    let quality_work = is_quality_changing_action(action_kind);
    let audit_required = quality_work && !path.exists();
    StandardizeAuditDecision {
        audit_required,
        audit_path: path.display().to_string(),
        surfaces_to_preserve: preservation_surface_names(project, scopes),
    }
}

/// @spec projects/agentic-workflow/tech-design/surface/interfaces/src/standardize_audit.md#source
pub(crate) fn fixture_audit(project: &str, scopes: &[String]) -> PreservationAudit {
    PreservationAudit {
        project: project.to_string(),
        scope: (!scopes.is_empty()).then(|| scopes.join(",")),
        surfaces: vec![
            PreservationSurface {
                kind: PreservationSurfaceKind::Route,
                name: "routes".to_string(),
                preserve: "preserve externally visible navigation and endpoint paths before quality changes".to_string(),
            },
            PreservationSurface {
                kind: PreservationSurfaceKind::Command,
                name: "commands".to_string(),
                preserve: "preserve CLI command names, arguments, and output contracts before quality changes".to_string(),
            },
            PreservationSurface {
                kind: PreservationSurfaceKind::GeneratedSource,
                name: "source ownership".to_string(),
                preserve: "preserve CODEGEN and HANDWRITE boundaries while retiring quality debt".to_string(),
            },
            PreservationSurface {
                kind: PreservationSurfaceKind::Api,
                name: "public API".to_string(),
                preserve: "preserve public API and serialized envelope contracts before quality changes"
                    .to_string(),
            },
            PreservationSurface {
                kind: PreservationSurfaceKind::Doc,
                name: "documentation".to_string(),
                preserve: "preserve documented capability, CLI, and workflow contracts before quality changes"
                    .to_string(),
            },
        ],
        quality_debt: vec!["record known quality debt before selecting modernization work".to_string()],
        safe_levers: vec![
            SafeModernizationLever {
                name: "documentation-only cleanup".to_string(),
                risk: ModernizationRisk::Low,
            },
            SafeModernizationLever {
                name: "localized source ownership repair".to_string(),
                risk: ModernizationRisk::Medium,
            },
        ],
    }
}

fn is_quality_changing_action(kind: StandardizeActionKind) -> bool {
    matches!(
        kind,
        StandardizeActionKind::RegenDrift
            | StandardizeActionKind::PromoteHandwrite
            | StandardizeActionKind::SemanticGap
            | StandardizeActionKind::GeneratorPrimitiveGap
            | StandardizeActionKind::IssueMarkerGap
            | StandardizeActionKind::FixSpecRule
            | StandardizeActionKind::FoldShadow
            | StandardizeActionKind::ClaimCode
    )
}

fn preservation_surface_names(project: &str, scopes: &[String]) -> Vec<String> {
    if scopes.is_empty() {
        vec![
            format!("{project}:routes"),
            format!("{project}:commands"),
            format!("{project}:public-contracts"),
            format!("{project}:docs"),
            format!("{project}:generated-source"),
        ]
    } else {
        scopes
            .iter()
            .flat_map(|scope| {
                [
                    format!("{scope}:routes"),
                    format!("{scope}:commands"),
                    format!("{scope}:public-contracts"),
                    format!("{scope}:docs"),
                    format!("{scope}:generated-source"),
                ]
            })
            .collect()
    }
}

fn sanitize_project_key(project: &str) -> String {
    project
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || ch == '-' || ch == '_' {
                ch
            } else {
                '-'
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn quality_action_requires_missing_audit() {
        let root = tempfile::tempdir().unwrap();
        let decision = evaluate_audit_decision(
            root.path(),
            "demo",
            &[],
            StandardizeActionKind::PromoteHandwrite,
        );

        assert!(decision.audit_required);
        assert!(decision
            .audit_path
            .ends_with(".aw/standardize/audit/demo.json"));
        assert!(decision
            .surfaces_to_preserve
            .contains(&"demo:routes".to_string()));
    }

    #[test]
    fn existing_audit_allows_action() {
        let root = tempfile::tempdir().unwrap();
        let path = audit_path(root.path(), "demo");
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(&path, "{}").unwrap();

        let decision = evaluate_audit_decision(
            root.path(),
            "demo",
            &[],
            StandardizeActionKind::PromoteHandwrite,
        );

        assert!(!decision.audit_required);
    }

    #[test]
    fn fixture_records_route_and_command_surfaces() {
        let audit = fixture_audit("demo", &["src/**".to_string()]);

        assert!(audit
            .surfaces
            .iter()
            .any(|surface| surface.kind == PreservationSurfaceKind::Route));
        assert!(audit
            .surfaces
            .iter()
            .any(|surface| surface.kind == PreservationSurfaceKind::Command));
        assert!(audit
            .surfaces
            .iter()
            .any(|surface| surface.kind == PreservationSurfaceKind::Api));
        assert!(audit
            .surfaces
            .iter()
            .any(|surface| surface.kind == PreservationSurfaceKind::Doc));
        assert_eq!(audit.scope.as_deref(), Some("src/**"));
    }
}
// CODEGEN-END
