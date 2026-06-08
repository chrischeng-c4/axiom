// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/validate/source/projects-sdd-src-validate-rules-r6a_loose_root_file-rs.md#source
// CODEGEN-BEGIN
//! R6a — reject loose `.md` files at a crate spec root or directly under
//! `interfaces/`.
//!
//! Spec roots under `crates/{crate}/` accept only
//! `README.md` as a loose top-level file; everything else must live in a
//! canonical subdir (`interfaces/`, `logic/`, `config/`, `tools/`, `skills/`,
//! `generate/`). Files directly inside `interfaces/` are also rejected — they
//! must be under a protocol subdir (`mcp/`, `cli/`, `rest/`, etc.).
//!
//! This rule fires per-file because the runner walks `.md` files; a
//! disallowed loose file produces one finding pointing at it.

use crate::validate::{Finding, Rule, RuleId, RuleReport};
use std::path::Path;

#[derive(Debug, Default, Clone)]
/// @spec projects/agentic-workflow/tech-design/core/validate/source/projects-sdd-src-validate-rules-r6a_loose_root_file-rs.md#source
pub struct LooseRootFileRule {}

/// @spec projects/agentic-workflow/tech-design/core/validate/source/projects-sdd-src-validate-rules-r6a_loose_root_file-rs.md#source
impl Rule for LooseRootFileRule {
    fn id(&self) -> RuleId {
        RuleId::LooseRootFile
    }

    fn check(&self, spec_path: &Path, content: &str, report: &mut RuleReport) {
        if is_codegen_fixture_spec(content) {
            return;
        }
        let Some((_, rel)) = locate_in_crate_spec_root(spec_path) else {
            return;
        };
        let parts: Vec<&str> = rel.split('/').collect();
        if parts.is_empty() {
            return;
        }

        // Loose .md at crate spec root, e.g. projects/agentic-workflow/foo.md
        if parts.len() == 1 && parts[0].ends_with(".md") && parts[0] != "README.md" {
            report.push(Finding::error(
                RuleId::LooseRootFile,
                spec_path,
                format!(
                    "loose file `{}` at crate spec root — only README.md is allowed; \
                         move under interfaces/, logic/, config/, tools/, skills/, or generate/",
                    parts[0]
                ),
            ));
            return;
        }

        // Loose file directly under interfaces/, e.g. projects/agentic-workflow/interfaces/foo.md
        if parts.len() == 2 && parts[0] == "interfaces" && parts[1].ends_with(".md") {
            report.push(Finding::error(
                RuleId::LooseRootFile,
                spec_path,
                format!(
                    "loose file `interfaces/{}` — files in interfaces/ must live under a \
                         protocol subdir (mcp/, cli/, rest/, etc.)",
                    parts[1]
                ),
            ));
        }
    }
}

/// If `spec_path` lives under a `crates/{crate}/`, `projects/{project}`, or
/// project-local `{project}/tech_design/` TD path, return
/// `(name, path_relative_to_root)` with forward-slash separators.
///
/// Anchors on the rightmost `crates`/`projects` segment so absolute host paths
/// whose filesystem also contains a `projects/` segment are resolved relative
/// to the TD root, not the host's `~/projects/`.
///
/// `agentic-workflow` preserves former Score and SDD specs under `cli/` and
/// `core/`; treat those as spec roots for structural validation.
///
/// Returns `None` for paths outside those segment shapes and for legacy
/// `projects/{project}/specs/...` roots.
pub(super) fn locate_in_crate_spec_root(spec_path: &Path) -> Option<(String, String)> {
    let s = spec_path
        .to_string_lossy()
        .replace(std::path::MAIN_SEPARATOR, "/");
    let parts: Vec<&str> = s.split('/').collect();
    if let Some(td_idx) = parts
        .iter()
        .enumerate()
        .rev()
        .find(|(i, p)| {
            (**p == "tech_design" || **p == "tech-design") && *i > 0 && i + 1 < parts.len()
        })
        .map(|(i, _)| i)
    {
        let mut root_name = parts.get(td_idx - 1)?.to_string();
        let mut rel_start = td_idx + 1;
        if root_name == "agentic-workflow"
            && matches!(parts.get(rel_start), Some(&"core" | &"surface"))
        {
            root_name = format!("{}/{}", root_name, parts[rel_start]);
            rel_start += 1;
        }
        let rel_parts = &parts[rel_start..];
        if !root_name.is_empty()
            && !root_name.starts_with('.')
            && !rel_parts.is_empty()
            && rel_parts.first() != Some(&"specs")
        {
            return Some((root_name, rel_parts.join("/")));
        }
    }

    let root_idx = parts
        .iter()
        .enumerate()
        .rev()
        .find(|(i, part)| (**part == "crates" || **part == "projects") && i + 1 < parts.len())
        .map(|(i, _)| i)?;
    let mut root_name = parts.get(root_idx + 1)?.to_string();
    let mut rel_start = root_idx + 2;
    if root_name == "agentic-workflow" {
        if matches!(parts.get(rel_start), Some(&"tech_design" | &"tech-design")) {
            rel_start += 1;
        }
        if matches!(parts.get(rel_start), Some(&"core" | &"surface")) {
            root_name = format!("{}/{}", root_name, parts[rel_start]);
            rel_start += 1;
        }
    }
    let rel_parts = &parts[rel_start..];
    if root_name.is_empty() || rel_parts.is_empty() || rel_parts.first() == Some(&"specs") {
        return None;
    }
    let rel = rel_parts.join("/");
    Some((root_name, rel))
}

/// Return true when the spec is a codegen fixture rather than a canonical TD.
/// @spec projects/agentic-workflow/tech-design/core/validate/source/projects-sdd-src-validate-rules-r6a_loose_root_file-rs.md#source
pub fn is_codegen_fixture_spec(content: &str) -> bool {
    let trimmed = content.trim_start();
    if !trimmed.starts_with("---") {
        return false;
    }
    let after = &trimmed[3..];
    let Some(end) = after.find("\n---") else {
        return false;
    };
    let frontmatter = &after[..end];
    frontmatter.lines().any(|line| {
        let l = line.trim();
        l == "type: codegen-fixture"
            || l == "type: \"codegen-fixture\""
            || (l.starts_with("fixture_for:") && l.len() > "fixture_for:".len())
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn run(path: &str) -> RuleReport {
        let mut r = RuleReport::new();
        LooseRootFileRule {}.check(&PathBuf::from(path), "", &mut r);
        r
    }

    #[test]
    fn loose_md_at_crate_root_flagged() {
        let r = run("projects/agentic-workflow/tech-design/core/state.md");
        assert_eq!(r.findings.len(), 1);
        assert_eq!(r.findings[0].rule, RuleId::LooseRootFile);
        assert!(r.findings[0].message.contains("state.md"));
    }

    #[test]
    fn configured_base_crate_root_flagged() {
        let r = run("docs/td/projects/agentic-workflow/state.md");
        assert_eq!(r.findings.len(), 1);
        assert!(r.findings[0].message.contains("state.md"));
    }

    #[test]
    fn readme_at_crate_root_is_ok() {
        let r = run("projects/agentic-workflow/tech-design/core/README.md");
        assert!(r.is_empty());
    }

    #[test]
    fn file_inside_canonical_subdir_is_ok() {
        let r = run("projects/agentic-workflow/tech-design/core/logic/state.md");
        assert!(r.is_empty());
    }

    #[test]
    fn loose_file_directly_under_interfaces_flagged() {
        let r = run("projects/agentic-workflow/tech-design/core/interfaces/commands.md");
        assert_eq!(r.findings.len(), 1);
        assert!(r.findings[0].message.contains("interfaces/commands.md"));
    }

    #[test]
    fn absolute_host_projects_prefix_uses_inner_project_root() {
        let r = run(
            "/Users/chrischeng/projects/cclab/project-aw/projects/agentic-workflow/tech-design/core/state.md",
        );
        assert_eq!(r.findings.len(), 1);
        assert_eq!(r.findings[0].rule, RuleId::LooseRootFile);
        assert!(r.findings[0].message.contains("state.md"));
    }

    #[test]
    fn absolute_host_projects_prefix_keeps_canonical_subdir_clean() {
        let r = run(
            "/Users/chrischeng/projects/cclab/project-aw/projects/agentic-workflow/tech-design/core/logic/state.md",
        );
        assert!(r.is_empty());
    }

    #[test]
    fn absolute_host_projects_prefix_rejects_direct_interfaces_file() {
        let r = run(
            "/Users/chrischeng/projects/cclab/project-aw/projects/agentic-workflow/tech-design/core/interfaces/commands.md",
        );
        assert_eq!(r.findings.len(), 1);
        assert!(r.findings[0].message.contains("interfaces/commands.md"));
    }

    #[test]
    fn file_under_interface_protocol_subdir_is_ok() {
        let r = run("projects/agentic-workflow/tech-design/core/interfaces/cli/commands.md");
        assert!(r.is_empty());
    }

    #[test]
    fn project_specs_path_is_not_checked() {
        let r = run("projects/agentic-workflow/tech-design/surface/specs/foo.md");
        assert!(r.is_empty());
    }

    #[test]
    fn unrelated_path_is_ignored() {
        let r = run("/tmp/spec.md");
        assert!(r.is_empty());
    }
}

// CODEGEN-END
