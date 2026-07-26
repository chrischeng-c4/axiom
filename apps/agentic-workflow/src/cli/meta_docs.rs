// SPEC-MANAGED: apps/agentic-workflow/tech-design/surface/interfaces/src/meta_docs.md#source
// @spec apps/agentic-workflow/tech-design/surface/specs/aw-meta-doc-ownership-matrix.md#logic
// CODEGEN-BEGIN
//! Canonical repository/project META-doc ownership matrix and validator.
//!
//! The matrix is the semantic source for placement, fact ownership,
//! inheritance, required headings, the CONTRIBUTING projection, and the
//! existing root-doc tests. The sibling `meta` module consumes this contract
//! for the `aw meta init|sync|check` producer/checker surface.

use serde::Serialize;
#[cfg(test)]
use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

pub const META_DOC_MATRIX_START: &str = "<!-- aw:meta-doc-matrix:start -->";
pub const META_DOC_MATRIX_END: &str = "<!-- aw:meta-doc-matrix:end -->";

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum MetaDocLayer {
    Repository,
    Project,
}

impl MetaDocLayer {
    pub const fn label(self) -> &'static str {
        match self {
            Self::Repository => "repo",
            Self::Project => "project",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct MetaDocContract {
    pub layer: MetaDocLayer,
    pub filename: &'static str,
    pub fact_owner: &'static str,
    pub required_headings: &'static [&'static str],
    pub inherits_from: &'static str,
}

const REPO_AGENT_HEADINGS: &[&str] = &["## Agentic Workflow CLI Surface"];
const REPO_CLAUDE_HEADINGS: &[&str] = &["## Claude Runtime Adapter"];
const REPO_README_HEADINGS: &[&str] = &["## Contributing"];
const REPO_CONTRIBUTING_HEADINGS: &[&str] = &["## Meta-doc content contract"];
const PROJECT_README_HEADINGS: &[&str] = &["## Brief", "## Contributing", "## Capability Contract"];
const PROJECT_CONTRIBUTING_HEADINGS: &[&str] = &[
    "## Brief",
    "## Authoritative Inputs",
    "## Local Workflow",
    "## Verification",
];
const PROJECT_CAPABILITIES_HEADINGS: &[&str] =
    &["## Brief", "## Capabilities", "### Capability Index"];

/// One fact-ownership row per META-doc and layer.
///
/// `CAPABILITIES.md` is a project-layer contract. In a single-product
/// repository the repository root is also the project root, so the project
/// row applies there; otherwise a root `CAPABILITIES.md` is forbidden.
pub const META_DOC_OWNERSHIP_MATRIX: &[MetaDocContract] = &[
    MetaDocContract {
        layer: MetaDocLayer::Repository,
        filename: "AGENTS.md",
        fact_owner: "Codex checkout operations; CLAUDE projection plus the fixed Codex whitelist",
        required_headings: REPO_AGENT_HEADINGS,
        inherits_from: "none",
    },
    MetaDocContract {
        layer: MetaDocLayer::Repository,
        filename: "CLAUDE.md",
        fact_owner: "Claude-only adapter importing AGENTS.md and generated Claude rule projections",
        required_headings: REPO_CLAUDE_HEADINGS,
        inherits_from: "AGENTS.md + .claude/rules projections",
    },
    MetaDocContract {
        layer: MetaDocLayer::Repository,
        filename: "README.md",
        fact_owner: "repository identity, inventory, install, and discovery entrypoints",
        required_headings: REPO_README_HEADINGS,
        inherits_from: "none",
    },
    MetaDocContract {
        layer: MetaDocLayer::Repository,
        filename: "CONTRIBUTING.md",
        fact_owner: "repo-wide authoring contracts, CLI conventions, and META-doc taxonomy",
        required_headings: REPO_CONTRIBUTING_HEADINGS,
        inherits_from: "none",
    },
    MetaDocContract {
        layer: MetaDocLayer::Project,
        filename: "README.md",
        fact_owner:
            "project identity and brief projections linking local contribution and goal contracts",
        required_headings: PROJECT_README_HEADINGS,
        inherits_from: "repo README + CONTRIBUTING",
    },
    MetaDocContract {
        layer: MetaDocLayer::Project,
        filename: "CONTRIBUTING.md",
        fact_owner: "project-local authoring, verification, migration, and contribution rules",
        required_headings: PROJECT_CONTRIBUTING_HEADINGS,
        inherits_from: "repo CONTRIBUTING",
    },
    MetaDocContract {
        layer: MetaDocLayer::Project,
        filename: "CAPABILITIES.md",
        fact_owner: "project product promises, work roots, and required verification",
        required_headings: PROJECT_CAPABILITIES_HEADINGS,
        inherits_from: "repo capability schema policy",
    },
];

pub fn meta_doc_contract(layer: MetaDocLayer, filename: &str) -> Option<&'static MetaDocContract> {
    META_DOC_OWNERSHIP_MATRIX
        .iter()
        .find(|contract| contract.layer == layer && contract.filename == filename)
}

pub fn allowed_repo_meta_doc_filenames(repository_is_product: bool) -> BTreeSet<&'static str> {
    META_DOC_OWNERSHIP_MATRIX
        .iter()
        .filter(|contract| {
            contract.layer == MetaDocLayer::Repository
                || (repository_is_product && contract.layer == MetaDocLayer::Project)
        })
        .map(|contract| contract.filename)
        .collect()
}

pub fn is_repo_only_agent_doc(filename: &str) -> bool {
    meta_doc_contract(MetaDocLayer::Repository, filename).is_some()
        && meta_doc_contract(MetaDocLayer::Project, filename).is_none()
        && matches!(filename, "AGENTS.md" | "CLAUDE.md")
}

fn rendered_headings(headings: &[&str]) -> String {
    headings
        .iter()
        .map(|heading| format!("`{heading}`"))
        .collect::<Vec<_>>()
        .join("<br>")
}

/// Render the exact ownership table embedded in CONTRIBUTING.md.
pub fn render_meta_doc_ownership_table() -> String {
    let mut out = String::from(
        "| Layer | Doc | Fact owner | Required headings | Inherits |\n\
         |---|---|---|---|---|\n",
    );
    for contract in META_DOC_OWNERSHIP_MATRIX {
        let doc = match contract.layer {
            MetaDocLayer::Repository => format!("`/{}`", contract.filename),
            MetaDocLayer::Project => format!("`<project>/{}`", contract.filename),
        };
        out.push_str(&format!(
            "| {} | {} | {} | {} | {} |\n",
            contract.layer.label(),
            doc,
            contract.fact_owner,
            rendered_headings(contract.required_headings),
            contract.inherits_from,
        ));
    }
    out
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum MetaFindingSeverity {
    Blocker,
    Drift,
    Advisory,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct MetaDocFinding {
    pub code: String,
    pub axis: String,
    pub severity: MetaFindingSeverity,
    pub path: String,
    pub message: String,
    pub remediation: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct MetaDocValidationReport {
    pub clean: bool,
    pub findings: Vec<MetaDocFinding>,
}

fn display_path(repository_root: &Path, path: &Path) -> String {
    path.strip_prefix(repository_root)
        .unwrap_or(path)
        .to_string_lossy()
        .replace('\\', "/")
}

fn resolved_project_root(repository_root: &Path, project_root: &Path) -> PathBuf {
    if project_root.is_absolute() {
        project_root.to_path_buf()
    } else {
        repository_root.join(project_root)
    }
}

fn has_heading(document: &str, heading: &str) -> bool {
    document.lines().any(|line| line.trim_end() == heading)
}

fn validate_contracts_at(
    repository_root: &Path,
    document_root: &Path,
    layer: MetaDocLayer,
    findings: &mut Vec<MetaDocFinding>,
) {
    for contract in META_DOC_OWNERSHIP_MATRIX
        .iter()
        .filter(|contract| contract.layer == layer)
    {
        let path = document_root.join(contract.filename);
        let relative = display_path(repository_root, &path);
        if !path.is_file() {
            findings.push(MetaDocFinding {
                code: "meta_doc_missing".to_string(),
                axis: "documents".to_string(),
                severity: MetaFindingSeverity::Blocker,
                path: relative.clone(),
                message: format!(
                    "{}-layer {} is required by the META-doc ownership matrix",
                    layer.label(),
                    contract.filename
                ),
                remediation: format!(
                    "Create {relative} from the {}-layer META-doc skeleton.",
                    layer.label()
                ),
            });
            continue;
        }
        let document = match fs::read_to_string(&path) {
            Ok(document) => document,
            Err(error) => {
                findings.push(MetaDocFinding {
                    code: "meta_doc_unreadable".to_string(),
                    axis: "documents".to_string(),
                    severity: MetaFindingSeverity::Blocker,
                    path: relative.clone(),
                    message: format!("cannot read {relative}: {error}"),
                    remediation: format!(
                        "Repair file permissions or encoding for {relative}, then rerun the META-doc check."
                    ),
                });
                continue;
            }
        };
        for heading in contract.required_headings {
            if has_heading(&document, heading) {
                continue;
            }
            findings.push(MetaDocFinding {
                code: "meta_doc_section_missing".to_string(),
                axis: "schema".to_string(),
                severity: MetaFindingSeverity::Blocker,
                path: relative.clone(),
                message: format!(
                    "{}-layer {} is missing canonical heading `{heading}`",
                    layer.label(),
                    contract.filename
                ),
                remediation: format!(
                    "Add `{heading}` to {relative} using the {}-layer skeleton; keep inherited repo facts linked rather than copied.",
                    layer.label()
                ),
            });
        }
    }
}

fn uppercase_meta_filename(filename: &str) -> bool {
    if filename == "LICENSE" {
        return true;
    }
    let Some(stem) = filename.strip_suffix(".md") else {
        return false;
    };
    !stem.is_empty()
        && stem.chars().all(|character| {
            character.is_ascii_uppercase()
                || character.is_ascii_digit()
                || matches!(character, '-' | '_')
        })
}

fn validate_repository_allowlist(
    repository_root: &Path,
    repository_is_product: bool,
    findings: &mut Vec<MetaDocFinding>,
) {
    let allowed = allowed_repo_meta_doc_filenames(repository_is_product);
    let entries = match fs::read_dir(repository_root) {
        Ok(entries) => entries,
        Err(error) => {
            findings.push(MetaDocFinding {
                code: "meta_doc_repository_unreadable".to_string(),
                axis: "placement".to_string(),
                severity: MetaFindingSeverity::Blocker,
                path: ".".to_string(),
                message: format!("cannot read repository root: {error}"),
                remediation: "Repair repository-root permissions and rerun the META-doc check."
                    .to_string(),
            });
            return;
        }
    };
    for entry in entries.filter_map(Result::ok) {
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        let Some(filename) = path.file_name().and_then(|name| name.to_str()) else {
            continue;
        };
        if filename.starts_with('.') || filename == "LICENSE" {
            continue;
        }
        if filename == "CAPABILITIES.md" && !repository_is_product {
            findings.push(MetaDocFinding {
                code: "root_capabilities_requires_product".to_string(),
                axis: "placement".to_string(),
                severity: MetaFindingSeverity::Blocker,
                path: filename.to_string(),
                message: "root CAPABILITIES.md is project-layer state, but this repository root is not declared as a product"
                    .to_string(),
                remediation: "Move the capability contract to the owning project root, or explicitly classify the repository root as a product before keeping root CAPABILITIES.md."
                    .to_string(),
            });
        } else if (filename.ends_with(".md") || uppercase_meta_filename(filename))
            && !allowed.contains(filename)
        {
            findings.push(MetaDocFinding {
                code: "unexpected_root_meta_doc".to_string(),
                axis: "placement".to_string(),
                severity: MetaFindingSeverity::Blocker,
                path: filename.to_string(),
                message: format!(
                    "{filename} is not owned by the repository-layer META-doc matrix"
                ),
                remediation: "Move project facts to the owning project README/CONTRIBUTING/CAPABILITIES file and scoped conventions next to the tree they govern."
                    .to_string(),
            });
        }
    }
}

fn validate_project_agent_doc_placement(
    repository_root: &Path,
    project_root: &Path,
    findings: &mut Vec<MetaDocFinding>,
) {
    for entry in WalkDir::new(project_root)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|entry| entry.file_type().is_file())
    {
        let Some(filename) = entry.path().file_name().and_then(|name| name.to_str()) else {
            continue;
        };
        if !is_repo_only_agent_doc(filename) {
            continue;
        }
        let path = display_path(repository_root, entry.path());
        let project = display_path(repository_root, project_root);
        findings.push(MetaDocFinding {
            code: "project_agent_doc_forbidden".to_string(),
            axis: "placement".to_string(),
            severity: MetaFindingSeverity::Blocker,
            path: path.clone(),
            message: format!(
                "{path} is a live project-layer {filename}; AGENTS.md and CLAUDE.md belong only to the repository root"
            ),
            remediation: format!(
                "Move project-specific agent facts to {project}/CONTRIBUTING.md or generated CLI guidance, then remove {path}."
            ),
        });
    }
}

/// Validate one repository and its explicitly selected project roots.
///
/// A repository product applies both repository- and project-layer contracts
/// at the root. A monorepo applies project contracts only to `project_roots`.
pub fn validate_meta_doc_layout(
    repository_root: &Path,
    repository_is_product: bool,
    project_roots: &[PathBuf],
) -> MetaDocValidationReport {
    let mut findings = Vec::new();
    validate_repository_allowlist(repository_root, repository_is_product, &mut findings);
    validate_contracts_at(
        repository_root,
        repository_root,
        MetaDocLayer::Repository,
        &mut findings,
    );
    if repository_is_product {
        validate_contracts_at(
            repository_root,
            repository_root,
            MetaDocLayer::Project,
            &mut findings,
        );
    }

    let mut seen = BTreeSet::new();
    for project_root in project_roots {
        let project_root = resolved_project_root(repository_root, project_root);
        if project_root == repository_root || !seen.insert(project_root.clone()) {
            continue;
        }
        validate_contracts_at(
            repository_root,
            &project_root,
            MetaDocLayer::Project,
            &mut findings,
        );
        validate_project_agent_doc_placement(repository_root, &project_root, &mut findings);
    }

    findings.sort_by(|left, right| {
        (&left.path, &left.code, &left.message).cmp(&(&right.path, &right.code, &right.message))
    });
    MetaDocValidationReport {
        clean: findings.is_empty(),
        findings,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn add_contract_headings(
        docs: &mut BTreeMap<PathBuf, BTreeSet<&'static str>>,
        root: &Path,
        layer: MetaDocLayer,
    ) {
        for contract in META_DOC_OWNERSHIP_MATRIX
            .iter()
            .filter(|contract| contract.layer == layer)
        {
            docs.entry(root.join(contract.filename))
                .or_default()
                .extend(contract.required_headings.iter().copied());
        }
    }

    fn write_valid_layout(
        repository_root: &Path,
        repository_is_product: bool,
        project_roots: &[PathBuf],
    ) {
        let mut docs = BTreeMap::new();
        add_contract_headings(&mut docs, repository_root, MetaDocLayer::Repository);
        if repository_is_product {
            add_contract_headings(&mut docs, repository_root, MetaDocLayer::Project);
        }
        for project in project_roots {
            let project = resolved_project_root(repository_root, project);
            add_contract_headings(&mut docs, &project, MetaDocLayer::Project);
        }
        for (path, headings) in docs {
            fs::create_dir_all(path.parent().expect("document parent")).unwrap();
            let mut body = String::from("# Fixture\n\n");
            for heading in headings {
                body.push_str(heading);
                body.push_str("\n\nFixture content.\n\n");
            }
            fs::write(path, body).unwrap();
        }
    }

    #[test]
    fn meta_doc_ownership_matrix_keys_are_unique_and_complete() {
        let keys = META_DOC_OWNERSHIP_MATRIX
            .iter()
            .map(|contract| (contract.layer, contract.filename))
            .collect::<BTreeSet<_>>();

        assert_eq!(keys.len(), META_DOC_OWNERSHIP_MATRIX.len());
        assert_eq!(
            keys,
            BTreeSet::from([
                (MetaDocLayer::Repository, "AGENTS.md"),
                (MetaDocLayer::Repository, "CLAUDE.md"),
                (MetaDocLayer::Repository, "CONTRIBUTING.md"),
                (MetaDocLayer::Repository, "README.md"),
                (MetaDocLayer::Project, "CAPABILITIES.md"),
                (MetaDocLayer::Project, "CONTRIBUTING.md"),
                (MetaDocLayer::Project, "README.md"),
            ])
        );
    }

    #[test]
    fn meta_doc_ownership_monorepo_and_project_layout_pass() {
        let temp = tempfile::tempdir().unwrap();
        let projects = vec![PathBuf::from("apps/demo")];
        write_valid_layout(temp.path(), false, &projects);

        let report = validate_meta_doc_layout(temp.path(), false, &projects);

        assert!(report.clean, "{:#?}", report.findings);
    }

    #[test]
    fn meta_doc_ownership_single_product_root_applies_both_layers() {
        let temp = tempfile::tempdir().unwrap();
        write_valid_layout(temp.path(), true, &[]);

        let report = validate_meta_doc_layout(temp.path(), true, &[]);

        assert!(report.clean, "{:#?}", report.findings);
        assert!(temp.path().join("CAPABILITIES.md").is_file());
    }

    #[test]
    fn meta_doc_ownership_project_agent_docs_have_actionable_remediation() {
        let temp = tempfile::tempdir().unwrap();
        let projects = vec![PathBuf::from("apps/demo")];
        write_valid_layout(temp.path(), false, &projects);
        for filename in ["AGENTS.md", "CLAUDE.md"] {
            fs::write(
                temp.path().join("apps/demo").join(filename),
                "# Local agent rules\n",
            )
            .unwrap();
        }

        let report = validate_meta_doc_layout(temp.path(), false, &projects);
        let findings = report
            .findings
            .iter()
            .filter(|finding| finding.code == "project_agent_doc_forbidden")
            .collect::<Vec<_>>();

        assert_eq!(findings.len(), 2);
        assert_eq!(findings[0].path, "apps/demo/AGENTS.md");
        assert_eq!(findings[1].path, "apps/demo/CLAUDE.md");
        assert!(findings
            .iter()
            .all(|finding| finding.remediation.contains("apps/demo/CONTRIBUTING.md")));
        assert!(findings
            .iter()
            .all(|finding| finding.remediation.contains("generated CLI guidance")));
    }

    #[test]
    fn meta_doc_ownership_root_capabilities_requires_product_classification() {
        let temp = tempfile::tempdir().unwrap();
        write_valid_layout(temp.path(), false, &[]);
        fs::write(temp.path().join("CAPABILITIES.md"), "# Stray contract\n").unwrap();

        let report = validate_meta_doc_layout(temp.path(), false, &[]);

        assert!(report
            .findings
            .iter()
            .any(|finding| finding.code == "root_capabilities_requires_product"));
    }

    #[test]
    fn meta_doc_ownership_missing_section_names_exact_skeleton_heading() {
        let temp = tempfile::tempdir().unwrap();
        let projects = vec![PathBuf::from("projects/demo")];
        write_valid_layout(temp.path(), false, &projects);
        fs::write(
            temp.path().join("projects/demo/CONTRIBUTING.md"),
            "# Demo Contributing\n\n## Brief\n\nBrief.\n",
        )
        .unwrap();

        let report = validate_meta_doc_layout(temp.path(), false, &projects);

        assert!(report.findings.iter().any(|finding| {
            finding.code == "meta_doc_section_missing"
                && finding.path == "projects/demo/CONTRIBUTING.md"
                && finding.message.contains("## Verification")
        }));
    }

    #[test]
    fn meta_doc_ownership_contributing_projection_matches_matrix() {
        let repository_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(Path::parent)
            .expect("repository root")
            .to_path_buf();
        let contributing = fs::read_to_string(repository_root.join("CONTRIBUTING.md")).unwrap();
        let start = contributing
            .find(META_DOC_MATRIX_START)
            .expect("META-doc matrix start marker")
            + META_DOC_MATRIX_START.len();
        let end = contributing[start..]
            .find(META_DOC_MATRIX_END)
            .map(|offset| start + offset)
            .expect("META-doc matrix end marker");

        assert_eq!(
            contributing[start..end].trim(),
            render_meta_doc_ownership_table().trim()
        );
    }
}
// CODEGEN-END
