// SPEC-MANAGED: projects/agentic-workflow/tech-design/surface/specs/project-health-governance-report.md#source
// CODEGEN-BEGIN
use serde::{Deserialize, Serialize};
use std::path::Path;

/// @spec projects/agentic-workflow/tech-design/surface/specs/project-health-governance-report.md#cli
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RegenerabilityAuthority {
    GeneratorAuthoritative,
    ExternalAdvisory,
}

/// @spec projects/agentic-workflow/tech-design/surface/specs/project-health-governance-report.md#cli
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RegenerabilityPolicy {
    pub authority: RegenerabilityAuthority,
    pub full_required: bool,
    pub reason: String,
}

/// @spec projects/agentic-workflow/tech-design/surface/specs/project-health-governance-report.md#logic
impl RegenerabilityPolicy {
    pub fn required_for_production(&self) -> bool {
        self.full_required || self.authority == RegenerabilityAuthority::GeneratorAuthoritative
    }
}

#[derive(Debug, Deserialize, Default)]
struct ConfigFile {
    #[serde(default)]
    projects: Vec<ProjectRow>,
}

#[derive(Debug, Deserialize, Default)]
struct ProjectRow {
    name: String,
    #[serde(default)]
    aliases: Vec<String>,
    #[serde(default)]
    regenerability: Option<ProjectRegenerabilityConfig>,
}

#[derive(Debug, Deserialize, Default)]
struct ProjectRegenerabilityConfig {
    #[serde(default)]
    authority: Option<RegenerabilityAuthority>,
    #[serde(default)]
    full_required: Option<bool>,
    #[serde(default)]
    reason: Option<String>,
}

/// @spec projects/agentic-workflow/tech-design/surface/specs/project-health-governance-report.md#logic
pub fn resolve_regenerability_policy(project: Option<&str>) -> RegenerabilityPolicy {
    if let Some(project) = project.filter(|project| !project.is_empty()) {
        if let Ok(project_root) = crate::find_project_root() {
            if let Some(policy) = resolve_regenerability_policy_at(&project_root, project) {
                return policy;
            }
        }
    }
    default_regenerability_policy(project)
}

/// @spec projects/agentic-workflow/tech-design/surface/specs/project-health-governance-report.md#logic
pub fn resolve_regenerability_policy_at(
    project_root: &Path,
    project: &str,
) -> Option<RegenerabilityPolicy> {
    let config_path = project_root.join(".aw").join("config.toml");
    let body = std::fs::read_to_string(config_path).ok()?;
    let parsed = toml::from_str::<ConfigFile>(&body).ok()?;
    let row = parsed
        .projects
        .into_iter()
        .find(|row| row.name == project || row.aliases.iter().any(|alias| alias == project))?;
    let default = default_regenerability_policy(Some(&row.name));
    let Some(config) = row.regenerability else {
        return Some(default);
    };
    let authority = config.authority.unwrap_or(default.authority);
    let full_required = config.full_required.unwrap_or(default.full_required);
    let reason = config.reason.unwrap_or_else(|| {
        format!(
            "configured regenerability policy for project `{}`",
            row.name
        )
    });
    Some(RegenerabilityPolicy {
        authority,
        full_required,
        reason,
    })
}

/// @spec projects/agentic-workflow/tech-design/surface/specs/project-health-governance-report.md#logic
fn default_regenerability_policy(project: Option<&str>) -> RegenerabilityPolicy {
    if matches!(project, Some("agentic-workflow" | "aw")) {
        RegenerabilityPolicy {
            authority: RegenerabilityAuthority::GeneratorAuthoritative,
            full_required: true,
            reason: "agentic-workflow is the generator-authoritative AW implementation project"
                .to_string(),
        }
    } else {
        RegenerabilityPolicy {
            authority: RegenerabilityAuthority::ExternalAdvisory,
            full_required: false,
            reason: "missing regenerability config keeps external generator gaps advisory"
                .to_string(),
        }
    }
}
// CODEGEN-END
