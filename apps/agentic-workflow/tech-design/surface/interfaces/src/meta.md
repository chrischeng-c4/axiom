---
id: apps-agentic-workflow-src-cli-meta-rs
summary: Canonical source unit for the aw meta init, sync, and check producer control plane.
capability_refs:
  - id: aw-core-client-model-workitem-first-artifact-lifecycle
    role: primary
    gap: meta-doc-init-sync-check
    claim: meta-doc-init-sync-check-source
    coverage: full
    rationale: "The source unit owns the single producer registry, marker reconciliation, scope resolution, and chainable output."
---

# Standardized apps/agentic-workflow/src/cli/meta.rs

## Source
<!-- type: source lang: rust -->
<!-- source-from-target: strip-handwrite -->

<!-- source-snapshot: path=apps/agentic-workflow/src/cli/meta.rs -->
~~~~~rust
// SPEC-MANAGED: apps/agentic-workflow/tech-design/surface/interfaces/src/meta.md#source
// @spec apps/agentic-workflow/tech-design/surface/specs/aw-meta-doc-producer.md#logic
// CODEGEN-BEGIN
//! `aw meta` — initialize, synchronize, and check repository/project META-docs.
//!
//! Every writable projection is registered in [`META_DOC_PRODUCERS`]. The
//! public commands and the legacy greenfield installer use the same pure
//! reconciliation functions, so no second producer can diverge from this
//! marker contract.

use crate::cli::doc_mirror;
use crate::cli::meta_docs::{
    meta_doc_contract, render_meta_doc_ownership_table, validate_meta_doc_layout, MetaDocFinding,
    MetaDocLayer, META_DOC_MATRIX_END, META_DOC_MATRIX_START,
};
use crate::services::project_registry;
use crate::Result;
use anyhow::{bail, Context};
use clap::{Args, Subcommand};
use serde::Serialize;
use std::fs;
use std::path::{Component, Path, PathBuf};

/// The binary's build-time snapshot of the CLAUDE template. #1912: never
/// read this directly for rendering — call [`resolve_claude_template`],
/// which prefers the live checkout copy when one is available so a stale
/// installed binary's embedded snapshot can never overwrite newer
/// checkout-authored prose. This constant is the last-resort fallback for
/// checkouts (or installs) where the checkout copy genuinely isn't present.
const EMBEDDED_CLAUDE_TEMPLATE: &str =
    include_str!("../../templates/cli/mainthread/CLAUDE.md.tmpl");
/// Repository-relative path to the checkout's own copy of the template
/// [`EMBEDDED_CLAUDE_TEMPLATE`] embeds at build time (#1912 R2).
const CLAUDE_TEMPLATE_CHECKOUT_RELATIVE: &str =
    "apps/agentic-workflow/templates/cli/mainthread/CLAUDE.md.tmpl";
const AW_START_MARKER: &str = "<!-- aw:start -->";
const AW_END_MARKER: &str = "<!-- aw:end -->";
const REPO_README_START: &str = "<!-- aw:meta:repo-readme:start -->";
const REPO_README_END: &str = "<!-- aw:meta:repo-readme:end -->";
const REPO_CONTRIBUTING_START: &str = "<!-- aw:meta:repo-contributing:start -->";
const REPO_CONTRIBUTING_END: &str = "<!-- aw:meta:repo-contributing:end -->";
const PROJECT_README_START: &str = "<!-- aw:meta:project-readme:start -->";
const PROJECT_README_END: &str = "<!-- aw:meta:project-readme:end -->";
const PROJECT_CONTRIBUTING_START: &str = "<!-- aw:meta:project-contributing:start -->";
const PROJECT_CONTRIBUTING_END: &str = "<!-- aw:meta:project-contributing:end -->";
const PROJECT_CAPABILITIES_START: &str = "<!-- aw:meta:project-capabilities:start -->";
const PROJECT_CAPABILITIES_END: &str = "<!-- aw:meta:project-capabilities:end -->";

#[derive(Debug, Args, Clone)]
pub struct MetaArgs {
    #[command(subcommand)]
    pub command: MetaCommand,
}

#[derive(Debug, Subcommand, Clone)]
pub enum MetaCommand {
    /// Create missing META-docs and reconcile every registered managed block.
    Init(MetaScopeArgs),
    /// Refresh every registered managed block without rewriting human regions.
    Sync(MetaScopeArgs),
    /// Read-only META-doc ownership and managed-block drift check.
    Check(MetaScopeArgs),
}

#[derive(Debug, Args, Clone, Default)]
pub struct MetaScopeArgs {
    /// Restrict project-layer docs to a configured project name (repeatable).
    #[arg(long = "project", value_name = "NAME")]
    pub projects: Vec<String>,
    /// Add an unregistered repository-relative project root (repeatable).
    #[arg(long = "project-path", value_name = "PATH")]
    pub project_paths: Vec<PathBuf>,
    /// Treat the repository root as the product root and create project META-docs there.
    #[arg(long)]
    pub repository_product: bool,
    /// #1912 R3: override the content-regression guard when a projection
    /// write that would delete existing content is a deliberate change,
    /// not stale-binary skew. The guard fires only when the installed
    /// binary is provably behind the checkout AND no live checkout
    /// template copy is available to render from instead.
    #[arg(long = "force-stale")]
    pub force_stale: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ProducerKind {
    RepoClaude,
    RepoAgents,
    RepoReadme,
    RepoProjectsTable,
    RepoContributing,
    RepoTraitTable,
    RepoMetaMatrix,
    ProjectReadme,
    ProjectContributing,
    ProjectCapabilities,
}

/// One AW-owned marker block. `(layer, filename, block_id)` is unique.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct MetaDocProducer {
    pub block_id: &'static str,
    pub layer: MetaDocLayer,
    pub filename: &'static str,
    pub start_marker: &'static str,
    pub end_marker: &'static str,
    pub kind: ProducerKind,
}

pub const META_DOC_PRODUCERS: &[MetaDocProducer] = &[
    MetaDocProducer {
        block_id: "repo-claude-guidance",
        layer: MetaDocLayer::Repository,
        filename: "CLAUDE.md",
        start_marker: AW_START_MARKER,
        end_marker: AW_END_MARKER,
        kind: ProducerKind::RepoClaude,
    },
    MetaDocProducer {
        block_id: "repo-agents-guidance",
        layer: MetaDocLayer::Repository,
        filename: "AGENTS.md",
        start_marker: AW_START_MARKER,
        end_marker: AW_END_MARKER,
        kind: ProducerKind::RepoAgents,
    },
    MetaDocProducer {
        block_id: "repo-readme-skeleton",
        layer: MetaDocLayer::Repository,
        filename: "README.md",
        start_marker: REPO_README_START,
        end_marker: REPO_README_END,
        kind: ProducerKind::RepoReadme,
    },
    MetaDocProducer {
        block_id: "repo-projects-table",
        layer: MetaDocLayer::Repository,
        filename: "README.md",
        start_marker: doc_mirror::PROJECTS_TABLE_START,
        end_marker: doc_mirror::PROJECTS_TABLE_END,
        kind: ProducerKind::RepoProjectsTable,
    },
    MetaDocProducer {
        block_id: "repo-contributing-skeleton",
        layer: MetaDocLayer::Repository,
        filename: "CONTRIBUTING.md",
        start_marker: REPO_CONTRIBUTING_START,
        end_marker: REPO_CONTRIBUTING_END,
        kind: ProducerKind::RepoContributing,
    },
    MetaDocProducer {
        block_id: "repo-trait-table",
        layer: MetaDocLayer::Repository,
        filename: "CONTRIBUTING.md",
        start_marker: doc_mirror::TRAIT_TABLE_START,
        end_marker: doc_mirror::TRAIT_TABLE_END,
        kind: ProducerKind::RepoTraitTable,
    },
    MetaDocProducer {
        block_id: "repo-meta-doc-matrix",
        layer: MetaDocLayer::Repository,
        filename: "CONTRIBUTING.md",
        start_marker: META_DOC_MATRIX_START,
        end_marker: META_DOC_MATRIX_END,
        kind: ProducerKind::RepoMetaMatrix,
    },
    MetaDocProducer {
        block_id: "project-readme-skeleton",
        layer: MetaDocLayer::Project,
        filename: "README.md",
        start_marker: PROJECT_README_START,
        end_marker: PROJECT_README_END,
        kind: ProducerKind::ProjectReadme,
    },
    MetaDocProducer {
        block_id: "project-contributing-skeleton",
        layer: MetaDocLayer::Project,
        filename: "CONTRIBUTING.md",
        start_marker: PROJECT_CONTRIBUTING_START,
        end_marker: PROJECT_CONTRIBUTING_END,
        kind: ProducerKind::ProjectContributing,
    },
    MetaDocProducer {
        block_id: "project-capabilities-skeleton",
        layer: MetaDocLayer::Project,
        filename: "CAPABILITIES.md",
        start_marker: PROJECT_CAPABILITIES_START,
        end_marker: PROJECT_CAPABILITIES_END,
        kind: ProducerKind::ProjectCapabilities,
    },
];

#[derive(Debug, Clone, PartialEq, Eq)]
struct ProjectTarget {
    name: String,
    path: PathBuf,
}

#[derive(Debug, Clone)]
struct MetaScope {
    repository_is_product: bool,
    projects: Vec<ProjectTarget>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct MetaDocChange {
    pub path: String,
    pub block: String,
    pub status: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct NextCommand {
    command: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct TerminalMarker {
    status: &'static str,
}

#[derive(Debug, Clone, Serialize)]
struct MetaCommandOutput {
    schema_version: &'static str,
    action: &'static str,
    status: &'static str,
    repository_root: String,
    repository_is_product: bool,
    projects: Vec<String>,
    changes: Vec<MetaDocChange>,
    findings: Vec<MetaDocFinding>,
    /// #1912 R4: `Some(source_version)` when the running binary is provably
    /// behind the checkout's declared source version (semver) or its own
    /// embedded CLAUDE template snapshot has fallen behind the checkout's
    /// live copy (content-precise — catches drift with no version bump).
    /// `check` routes findings to the rebuild/upgrade remediation instead
    /// of `aw meta sync` whenever this is set.
    #[serde(skip_serializing_if = "Option::is_none")]
    binary_stale: Option<String>,
    next: Option<NextCommand>,
    terminal: Option<TerminalMarker>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ApplyMode {
    Init,
    Sync,
    Check,
}

pub fn run(args: MetaArgs) -> Result<()> {
    let root = crate::find_project_root()?;
    run_at_root(&root, args)
}

fn run_at_root(root: &Path, args: MetaArgs) -> Result<()> {
    let (mode, scope_args) = match args.command {
        MetaCommand::Init(args) => (ApplyMode::Init, args),
        MetaCommand::Sync(args) => (ApplyMode::Sync, args),
        MetaCommand::Check(args) => (ApplyMode::Check, args),
    };
    let scope = resolve_scope(root, &scope_args)?;
    let output = execute(root, &scope, &scope_args, mode)?;
    println!("{}", serde_json::to_string_pretty(&output)?);
    if mode == ApplyMode::Check && !output.findings.is_empty() {
        bail!(
            "META-doc drift detected in {} file/block finding(s); run {}",
            output.findings.len(),
            output
                .next
                .as_ref()
                .map(|next| next.command.as_str())
                .unwrap_or("aw meta sync")
        );
    }
    Ok(())
}

fn execute(
    root: &Path,
    scope: &MetaScope,
    scope_args: &MetaScopeArgs,
    mode: ApplyMode,
) -> Result<MetaCommandOutput> {
    execute_with_binary_version(root, scope, scope_args, mode, env!("AW_BUILD_VERSION"))
}

/// #1912: [`execute`] with an injectable running-binary version, so R4's
/// stale-binary diagnosis is independently testable (mirrors the injected
/// -inputs pattern in `drift::gate_decision_at`).
fn execute_with_binary_version(
    root: &Path,
    scope: &MetaScope,
    scope_args: &MetaScopeArgs,
    mode: ApplyMode,
    binary_version: &str,
) -> Result<MetaCommandOutput> {
    let mut changes = Vec::new();
    let mut projection_findings = Vec::new();
    // #2186: materialize scoped project skeletons before the repository
    // Projects table reads their `## Brief` sections. This keeps `meta init`
    // bootstrap-safe while leaving `check` read-only.
    for project in &scope.projects {
        reconcile_layer(
            root,
            &project.path,
            &project.name,
            MetaDocLayer::Project,
            mode,
            scope_args.force_stale,
            &mut changes,
            &mut projection_findings,
        )?;
    }
    reconcile_layer(
        root,
        root,
        root_name(root),
        MetaDocLayer::Repository,
        mode,
        scope_args.force_stale,
        &mut changes,
        &mut projection_findings,
    )?;
    if scope.repository_is_product {
        reconcile_layer(
            root,
            root,
            root_name(root),
            MetaDocLayer::Project,
            mode,
            scope_args.force_stale,
            &mut changes,
            &mut projection_findings,
        )?;
    }

    let project_paths = scope
        .projects
        .iter()
        .map(|project| project.path.clone())
        .collect::<Vec<_>>();
    let mut findings =
        validate_meta_doc_layout(root, scope.repository_is_product, &project_paths).findings;
    findings.extend(projection_findings);
    findings.sort_by(|left, right| {
        (&left.path, &left.code, &left.message).cmp(&(&right.path, &right.code, &right.message))
    });
    findings.dedup();

    // #1912 R4: content-precise (catches an unbumped-version template edit)
    // and semver-coarse (#1417's existing signal) stale-binary detection.
    // `check` must never invite a destructive `aw meta sync` when the
    // running binary itself is the reason projections look drifted.
    let binary_stale = embedded_template_is_stale(root)
        .then(|| "checkout HEAD".to_string())
        .or_else(|| crate::cli::drift::binary_behind_checkout_source_version(root, binary_version));

    let scope_suffix = render_scope_suffix(scope_args);
    let (action, status, next, terminal) = match mode {
        ApplyMode::Init => (
            "meta_init",
            "initialized",
            Some(NextCommand {
                command: format!("aw meta check{scope_suffix}"),
            }),
            None,
        ),
        ApplyMode::Sync => (
            "meta_sync",
            "synchronized",
            Some(NextCommand {
                command: format!("aw meta check{scope_suffix}"),
            }),
            None,
        ),
        ApplyMode::Check if findings.is_empty() => (
            "meta_check",
            "clean",
            None,
            Some(TerminalMarker { status: "done" }),
        ),
        ApplyMode::Check if binary_stale.is_some() => {
            let staleness = match binary_stale.as_deref() {
                Some("checkout HEAD") => "behind the checkout's live CLAUDE template".to_string(),
                Some(source_version) => format!("behind checkout source v{source_version}"),
                None => "behind the checkout".to_string(),
            };
            for finding in findings.iter_mut() {
                finding.remediation = format!(
                    "Installed aw binary is {staleness}; rebuild \
                     (`cargo install --path apps/agentic-workflow`) or run `aw upgrade`, then \
                     re-run `aw meta check` instead of `aw meta sync` from this binary."
                );
            }
            (
                "meta_check",
                "binary_stale",
                Some(NextCommand {
                    command: "cargo install --path apps/agentic-workflow".to_string(),
                }),
                None,
            )
        }
        ApplyMode::Check => (
            "meta_check",
            "drift",
            Some(NextCommand {
                command: format!("aw meta sync{scope_suffix}"),
            }),
            None,
        ),
    };
    Ok(MetaCommandOutput {
        schema_version: "aw.meta.v1",
        action,
        status,
        repository_root: root.display().to_string(),
        repository_is_product: scope.repository_is_product,
        projects: scope
            .projects
            .iter()
            .map(|project| display_path(root, &project.path))
            .collect(),
        binary_stale,
        changes,
        findings,
        next,
        terminal,
    })
}

fn resolve_scope(root: &Path, args: &MetaScopeArgs) -> Result<MetaScope> {
    let rows = project_registry::load_project_config_rows(root)?;
    let mut projects = Vec::new();
    if args.projects.is_empty() && args.project_paths.is_empty() {
        for row in rows {
            if row.path.trim().is_empty() {
                continue;
            }
            projects.push(ProjectTarget {
                name: row.name,
                path: resolve_relative_project_path(root, Path::new(&row.path))?,
            });
        }
    } else {
        for requested in &args.projects {
            let row = rows
                .iter()
                .find(|row| row.matches(requested))
                .ok_or_else(|| anyhow::anyhow!("project `{requested}` has no AW config row"))?;
            projects.push(ProjectTarget {
                name: row.name.clone(),
                path: resolve_relative_project_path(root, Path::new(&row.path))?,
            });
        }
        for path in &args.project_paths {
            projects.push(ProjectTarget {
                name: path
                    .file_name()
                    .and_then(|name| name.to_str())
                    .filter(|name| !name.is_empty())
                    .unwrap_or("project")
                    .to_string(),
                path: resolve_relative_project_path(root, path)?,
            });
        }
    }
    projects.sort_by(|left, right| left.path.cmp(&right.path));
    projects.dedup_by(|left, right| left.path == right.path);
    projects.retain(|project| project.path != root);
    Ok(MetaScope {
        repository_is_product: args.repository_product,
        projects,
    })
}

fn resolve_relative_project_path(root: &Path, path: &Path) -> Result<PathBuf> {
    if path.is_absolute()
        || path
            .components()
            .any(|component| matches!(component, Component::ParentDir | Component::RootDir))
    {
        bail!(
            "META-doc project path must stay repository-relative: {}",
            path.display()
        );
    }
    Ok(root.join(path))
}

fn render_scope_suffix(args: &MetaScopeArgs) -> String {
    let mut parts = Vec::new();
    for project in &args.projects {
        parts.push(format!(" --project {project}"));
    }
    for path in &args.project_paths {
        parts.push(format!(" --project-path {}", path.display()));
    }
    if args.repository_product {
        parts.push(" --repository-product".to_string());
    }
    parts.concat()
}

fn root_name(root: &Path) -> &str {
    root.file_name()
        .and_then(|name| name.to_str())
        .filter(|name| !name.is_empty())
        .unwrap_or("Repository")
}

fn humanize(value: &str) -> String {
    value
        .split(['-', '_'])
        .filter(|part| !part.is_empty())
        .map(|part| {
            let mut chars = part.chars();
            match chars.next() {
                Some(first) => format!("{}{}", first.to_uppercase(), chars.as_str()),
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn display_path(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .to_string_lossy()
        .replace('\\', "/")
}

fn reconcile_layer(
    repository_root: &Path,
    document_root: &Path,
    target_name: &str,
    layer: MetaDocLayer,
    mode: ApplyMode,
    force_stale: bool,
    changes: &mut Vec<MetaDocChange>,
    findings: &mut Vec<MetaDocFinding>,
) -> Result<()> {
    for producer in META_DOC_PRODUCERS
        .iter()
        .filter(|producer| producer.layer == layer)
    {
        reconcile_producer(
            repository_root,
            document_root,
            target_name,
            producer,
            mode,
            force_stale,
            changes,
            findings,
        )?;
    }
    Ok(())
}

fn reconcile_producer(
    repository_root: &Path,
    document_root: &Path,
    target_name: &str,
    producer: &MetaDocProducer,
    mode: ApplyMode,
    force_stale: bool,
    changes: &mut Vec<MetaDocChange>,
    findings: &mut Vec<MetaDocFinding>,
) -> Result<()> {
    let path = document_root.join(producer.filename);
    let relative = display_path(repository_root, &path);
    let existing = fs::read_to_string(&path).ok();
    let desired = desired_document(
        existing.as_deref(),
        target_name,
        producer,
        document_root,
        force_stale,
    )?;
    match desired {
        DesiredDocument::Unchanged => changes.push(MetaDocChange {
            path: relative,
            block: producer.block_id.to_string(),
            status: "unchanged".to_string(),
        }),
        DesiredDocument::Malformed(message) => findings.push(MetaDocFinding {
            code: "managed_block_malformed".to_string(),
            path: relative.clone(),
            message,
            remediation: format!(
                "Repair marker pair `{}` / `{}` in {relative}, then run `aw meta sync`.",
                producer.start_marker, producer.end_marker
            ),
        }),
        DesiredDocument::SkewBlocked(message) => findings.push(MetaDocFinding {
            code: "content_regression_blocked".to_string(),
            path: relative.clone(),
            message,
            remediation: format!(
                "Rebuild the aw binary (`cargo install --path apps/agentic-workflow`) or run \
                 `aw upgrade`, then retry; or pass `--force-stale` to {relative} block `{}` once \
                 the deletion is confirmed deliberate.",
                producer.block_id
            ),
        }),
        DesiredDocument::Changed(_) if mode == ApplyMode::Check => {
            findings.push(MetaDocFinding {
                code: if existing.is_some() {
                    "managed_block_stale".to_string()
                } else {
                    "managed_block_missing".to_string()
                },
                path: relative.clone(),
                message: format!(
                    "{} is not the canonical `{}` projection",
                    relative, producer.block_id
                ),
                remediation: format!(
                    "Run `aw meta sync` to repair {relative} block `{}`.",
                    producer.block_id
                ),
            });
        }
        DesiredDocument::Changed(content) => {
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent)
                    .with_context(|| format!("create META-doc directory {}", parent.display()))?;
            }
            fs::write(&path, content)
                .with_context(|| format!("write META-doc {}", path.display()))?;
            changes.push(MetaDocChange {
                path: relative,
                block: producer.block_id.to_string(),
                status: if existing.is_some() {
                    "updated".to_string()
                } else {
                    "created".to_string()
                },
            });
        }
    }
    Ok(())
}

enum DesiredDocument {
    Unchanged,
    Changed(String),
    Malformed(String),
    /// #1912 R3: the write was refused because the render source is
    /// provably stale (installed binary confirmed behind checkout, no
    /// live checkout template available to render from instead) and it
    /// would delete existing projection content. `--force-stale` bypasses
    /// this and falls through to `Changed`.
    SkewBlocked(String),
}

fn desired_document(
    existing: Option<&str>,
    target_name: &str,
    producer: &MetaDocProducer,
    document_root: &Path,
    force_stale: bool,
) -> Result<DesiredDocument> {
    if matches!(
        producer.kind,
        ProducerKind::RepoClaude | ProducerKind::RepoAgents
    ) {
        if let Some(existing) = existing {
            let has_start = existing.contains(producer.start_marker);
            let has_end = existing.contains(producer.end_marker);
            if has_start != has_end {
                return Ok(DesiredDocument::Malformed(format!(
                    "{} has an unmatched managed marker for block `{}`",
                    producer.filename, producer.block_id
                )));
            }
        }
        // #1912 R2: prefer the live checkout copy of the template over the
        // binary's embedded snapshot whenever one is present, eliminating
        // the stale-binary skew window for the self-hosting repo entirely.
        let (template, from_checkout) = resolve_claude_template(document_root);
        let desired = render_agent_document(existing, producer.kind, &template);

        // #1912 R3: defense in depth. Only reachable when no checkout copy
        // was available to render from (R2 already covers the case where
        // one exists) — if the installed binary is also provably behind
        // the checkout and this write would delete existing content,
        // refuse rather than trust the embedded snapshot.
        if !from_checkout && !force_stale {
            if let (Some(existing), Some(source_version)) = (
                existing,
                crate::cli::drift::binary_behind_checkout_source_version(
                    document_root,
                    env!("AW_BUILD_VERSION"),
                ),
            ) {
                let deleted = lines_removed(existing, &desired);
                if !deleted.is_empty() {
                    return Ok(DesiredDocument::SkewBlocked(format!(
                        "{} write would delete {} existing line(s) while the installed aw \
                         binary is behind checkout source v{source_version} and no live \
                         checkout template is available to render from instead",
                        producer.filename,
                        deleted.len()
                    )));
                }
            }
        }

        return Ok(match existing {
            Some(current) if current == desired => DesiredDocument::Unchanged,
            _ => DesiredDocument::Changed(desired),
        });
    }
    if matches!(
        producer.kind,
        ProducerKind::RepoProjectsTable | ProducerKind::RepoTraitTable
    ) {
        let Some(existing) = existing else {
            return Ok(DesiredDocument::Unchanged);
        };
        let has_start = existing.contains(producer.start_marker);
        let has_end = existing.contains(producer.end_marker);
        if has_start != has_end {
            return Ok(DesiredDocument::Malformed(format!(
                "{} has an unmatched managed marker for block `{}`",
                producer.filename, producer.block_id
            )));
        }
        if !has_start {
            return Ok(DesiredDocument::Unchanged);
        }
        let desired = match producer.kind {
            ProducerKind::RepoProjectsTable => {
                doc_mirror::upsert_projects_table(document_root, existing)?
            }
            ProducerKind::RepoTraitTable => doc_mirror::upsert_trait_table(existing),
            _ => unreachable!(),
        };
        return Ok(if existing == desired {
            DesiredDocument::Unchanged
        } else {
            DesiredDocument::Changed(desired)
        });
    }
    let block = render_block(target_name, producer);
    let Some(existing) = existing else {
        if producer.kind == ProducerKind::RepoMetaMatrix {
            return Ok(DesiredDocument::Unchanged);
        }
        let title = render_title(target_name, producer.kind);
        return Ok(DesiredDocument::Changed(format!("{title}\n\n{block}\n")));
    };
    let has_start = existing.contains(producer.start_marker);
    let has_end = existing.contains(producer.end_marker);
    if has_start != has_end {
        return Ok(DesiredDocument::Malformed(format!(
            "{} has an unmatched managed marker for block `{}`",
            producer.filename, producer.block_id
        )));
    }
    if has_start {
        let desired =
            replace_marker_block(existing, producer.start_marker, producer.end_marker, &block)?;
        return Ok(if existing == desired {
            DesiredDocument::Unchanged
        } else {
            DesiredDocument::Changed(desired)
        });
    }
    let contract = meta_doc_contract(producer.layer, producer.filename)
        .expect("every producer must have an ownership-matrix contract");
    if producer.kind != ProducerKind::RepoMetaMatrix
        && contract
            .required_headings
            .iter()
            .all(|heading| existing.lines().any(|line| line.trim_end() == *heading))
    {
        return Ok(DesiredDocument::Unchanged);
    }
    let separator = if existing.ends_with("\n\n") {
        ""
    } else if existing.ends_with('\n') {
        "\n"
    } else {
        "\n\n"
    };
    Ok(DesiredDocument::Changed(format!(
        "{existing}{separator}{block}\n"
    )))
}

fn render_title(target_name: &str, kind: ProducerKind) -> String {
    let name = humanize(target_name);
    match kind {
        ProducerKind::RepoReadme | ProducerKind::ProjectReadme => format!("# {name}"),
        ProducerKind::RepoContributing => "# Contributing".to_string(),
        ProducerKind::ProjectContributing => format!("# {name} Contributing"),
        ProducerKind::ProjectCapabilities => format!("# {name} Capabilities"),
        ProducerKind::RepoClaude
        | ProducerKind::RepoAgents
        | ProducerKind::RepoProjectsTable
        | ProducerKind::RepoTraitTable
        | ProducerKind::RepoMetaMatrix => unreachable!(),
    }
}

fn render_block(target_name: &str, producer: &MetaDocProducer) -> String {
    let name = humanize(target_name);
    let inner = match producer.kind {
        ProducerKind::RepoReadme => {
            "## Contributing\n\nRepository-wide authoring rules live in [CONTRIBUTING.md](CONTRIBUTING.md)."
                .to_string()
        }
        ProducerKind::RepoContributing => format!(
            "## Meta-doc content contract\n\nRepository and project META-doc ownership is generated from the Agentic Workflow matrix.\n\n{META_DOC_MATRIX_START}\n{}{META_DOC_MATRIX_END}",
            render_meta_doc_ownership_table()
        ),
        ProducerKind::RepoMetaMatrix => render_meta_doc_ownership_table(),
        ProducerKind::ProjectReadme => format!(
            "## Brief\n\nDescribe the agent-readable purpose of {name}.\n\n## Contributing\n\nProject-local authoring and verification rules live in [CONTRIBUTING.md](CONTRIBUTING.md).\n\n## Capability Contract\n\nProduct promises and work roots live in [CAPABILITIES.md](CAPABILITIES.md)."
        ),
        ProducerKind::ProjectContributing => format!(
            "## Brief\n\nProject-local contribution contract for {name}.\n\n## Authoritative Inputs\n\n- Product promises and work roots: [CAPABILITIES.md](CAPABILITIES.md)\n- Project orientation: [README.md](README.md)\n\n## Local Workflow\n\nFollow repository-level agent guidance and keep project-specific rules here.\n\n## Verification\n\nList the narrow commands that prove changes to {name}."
        ),
        ProducerKind::ProjectCapabilities => format!(
            "## Brief\n\nMachine-readable capability contract for {name}.\n\n## Capabilities\n\n### Capability Index\n\n| Capability | Root WI | Impl | Verification | Maturity | Production | Notes |\n|---|---:|---|---|---|---|---|"
        ),
        ProducerKind::RepoClaude
        | ProducerKind::RepoAgents
        | ProducerKind::RepoProjectsTable
        | ProducerKind::RepoTraitTable => unreachable!(),
    };
    format!(
        "{}\n{}\n{}",
        producer.start_marker,
        inner.trim_end(),
        producer.end_marker
    )
}

fn replace_marker_block(
    existing: &str,
    start_marker: &str,
    end_marker: &str,
    replacement: &str,
) -> Result<String> {
    let start = existing
        .find(start_marker)
        .ok_or_else(|| anyhow::anyhow!("missing managed start marker `{start_marker}`"))?;
    let after_start = start + start_marker.len();
    let end = existing[after_start..]
        .find(end_marker)
        .map(|offset| after_start + offset + end_marker.len())
        .ok_or_else(|| anyhow::anyhow!("missing managed end marker `{end_marker}`"))?;
    Ok(format!(
        "{}{}{}",
        &existing[..start],
        replacement,
        &existing[end..]
    ))
}

/// #1912 R2: resolve the CLAUDE template text to render from, preferring
/// the live checkout copy at [`CLAUDE_TEMPLATE_CHECKOUT_RELATIVE`]
/// (relative to `document_root`, the resolved repository root for this
/// producer's layer) over the binary's embedded `include_str!` snapshot.
/// Returns `(text, from_checkout)`.
fn resolve_claude_template(document_root: &Path) -> (String, bool) {
    let checkout_path = document_root.join(CLAUDE_TEMPLATE_CHECKOUT_RELATIVE);
    match fs::read_to_string(&checkout_path) {
        Ok(text) => (text, true),
        Err(_) => (EMBEDDED_CLAUDE_TEMPLATE.to_string(), false),
    }
}

/// #1912 R4: `true` when the checkout's live template (if present) renders
/// differently from this binary's embedded snapshot — a content-precise
/// staleness signal independent of semver, catching a template edit that
/// landed without a package version bump (the true root cause of the
/// 2026-07-17 incident: #1417's semver-behind gate alone cannot see it).
fn embedded_template_is_stale(document_root: &Path) -> bool {
    let checkout_path = document_root.join(CLAUDE_TEMPLATE_CHECKOUT_RELATIVE);
    let Ok(checkout_text) = fs::read_to_string(&checkout_path) else {
        return false;
    };
    rendered_claude_document(&checkout_text) != rendered_claude_document(EMBEDDED_CLAUDE_TEMPLATE)
}

/// #1912 R3: lines present in `existing` but missing from `desired` —
/// i.e. content the write is about to delete. Blank lines are ignored
/// (formatting noise, not content).
fn lines_removed(existing: &str, desired: &str) -> Vec<String> {
    let desired_lines: std::collections::HashSet<&str> = desired.lines().collect();
    existing
        .lines()
        .filter(|line| !line.trim().is_empty())
        .filter(|line| !desired_lines.contains(line))
        .map(str::to_string)
        .collect()
}

fn split_claude_template(template: &str) -> (&str, &str, &str) {
    let start = template
        .find(AW_START_MARKER)
        .expect("CLAUDE template must have aw:start");
    let end = template
        .find(AW_END_MARKER)
        .map(|index| index + AW_END_MARKER.len())
        .expect("CLAUDE template must have aw:end");
    (&template[..start], &template[start..end], &template[end..])
}

fn rendered_claude_section(template: &str) -> String {
    let (_, section, _) = split_claude_template(template);
    doc_mirror::render_cli_tables(section)
}

fn rendered_claude_document(template: &str) -> String {
    let (before, section, after) = split_claude_template(template);
    format!("{before}{}{after}", doc_mirror::render_cli_tables(section))
}

fn upsert_agent_section(existing: &str, section: &str) -> String {
    if existing.contains(AW_START_MARKER) && existing.contains(AW_END_MARKER) {
        return replace_marker_block(existing, AW_START_MARKER, AW_END_MARKER, section)
            .expect("validated agent marker pair");
    }
    if let Some(first_newline) = existing.find('\n') {
        if existing[..first_newline].starts_with('#') {
            return format!(
                "{}\n\n{}{}",
                &existing[..first_newline],
                section,
                &existing[first_newline..]
            );
        }
    }
    format!("{section}\n\n{existing}")
}

fn render_agent_document(existing: Option<&str>, kind: ProducerKind, template: &str) -> String {
    let claude_section = rendered_claude_section(template);
    let (section, fresh) = match kind {
        ProducerKind::RepoClaude => (claude_section, rendered_claude_document(template)),
        ProducerKind::RepoAgents => {
            let section = doc_mirror::agents_block_from_claude_block(&claude_section);
            let fresh = format!("{}\n\n{}\n", doc_mirror::AGENTS_TITLE, section);
            (section, fresh)
        }
        _ => unreachable!(),
    };
    existing
        .map(|existing| upsert_agent_section(existing, &section))
        .unwrap_or(fresh)
}

/// Legacy `aw new` delegates all single-product META-doc projection here.
pub(crate) fn sync_repository_product_docs(root: &Path) -> Result<Vec<MetaDocChange>> {
    let mut changes = Vec::new();
    let mut findings = Vec::new();
    reconcile_layer(
        root,
        root,
        root_name(root),
        MetaDocLayer::Repository,
        ApplyMode::Sync,
        false,
        &mut changes,
        &mut findings,
    )?;
    reconcile_layer(
        root,
        root,
        root_name(root),
        MetaDocLayer::Project,
        ApplyMode::Sync,
        false,
        &mut changes,
        &mut findings,
    )?;
    if let Some(finding) = findings.first() {
        bail!("{}: {}", finding.path, finding.message);
    }
    Ok(changes)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;

    fn args_for(path: &str, repository_product: bool) -> MetaScopeArgs {
        MetaScopeArgs {
            projects: Vec::new(),
            project_paths: vec![PathBuf::from(path)],
            repository_product,
            force_stale: false,
        }
    }

    #[test]
    fn meta_producer_registry_is_unique_and_covers_matrix_rows() {
        let keys = META_DOC_PRODUCERS
            .iter()
            .map(|producer| (producer.layer, producer.filename, producer.block_id))
            .collect::<BTreeSet<_>>();
        assert_eq!(keys.len(), META_DOC_PRODUCERS.len());
        for producer in META_DOC_PRODUCERS {
            assert!(meta_doc_contract(producer.layer, producer.filename).is_some());
        }
    }

    #[test]
    fn meta_init_creates_fresh_repo_and_project_skeletons() {
        let temp = tempfile::tempdir().unwrap();
        let args = args_for("apps/demo", false);
        let scope = resolve_scope(temp.path(), &args).unwrap();
        let output = execute(temp.path(), &scope, &args, ApplyMode::Init).unwrap();
        assert!(output.findings.is_empty(), "{:#?}", output.findings);
        for path in [
            "AGENTS.md",
            "CLAUDE.md",
            "README.md",
            "CONTRIBUTING.md",
            "apps/demo/README.md",
            "apps/demo/CONTRIBUTING.md",
            "apps/demo/CAPABILITIES.md",
        ] {
            assert!(temp.path().join(path).is_file(), "missing {path}");
        }
        assert_eq!(
            output.next.unwrap().command,
            "aw meta check --project-path apps/demo"
        );
    }

    #[test]
    fn meta_init_bootstraps_configured_project_before_repo_table_projection() {
        let temp = tempfile::tempdir().unwrap();
        let project = temp.path().join("apps/workbench");
        fs::create_dir_all(&project).unwrap();
        fs::write(
            temp.path().join("aw.toml"),
            "[agentic_workflow.projects]\ndiscover = [\"apps/*/aw.toml\"]\n",
        )
        .unwrap();
        fs::write(
            project.join("aw.toml"),
            "[project]\nname = \"workbench\"\ncap_path = \"CAPABILITIES.md\"\nlabel = \"app:workbench\"\n",
        )
        .unwrap();
        fs::write(
            temp.path().join("README.md"),
            format!(
                "# Demo\n\n{}\n{}\n",
                doc_mirror::PROJECTS_TABLE_START,
                doc_mirror::PROJECTS_TABLE_END
            ),
        )
        .unwrap();
        let args = MetaScopeArgs {
            projects: vec!["workbench".to_string()],
            project_paths: Vec::new(),
            repository_product: false,
            force_stale: false,
        };

        let scope = resolve_scope(temp.path(), &args).unwrap();
        let output = execute(temp.path(), &scope, &args, ApplyMode::Init).unwrap();

        assert!(output.findings.is_empty(), "{:#?}", output.findings);
        assert!(project.join("README.md").is_file());
        assert!(project.join("CONTRIBUTING.md").is_file());
        assert!(project.join("CAPABILITIES.md").is_file());
        let root_readme = fs::read_to_string(temp.path().join("README.md")).unwrap();
        assert!(root_readme.contains("[workbench](apps/workbench/README.md)"));
        assert_eq!(
            output.next.unwrap().command,
            "aw meta check --project workbench"
        );
    }

    #[test]
    fn meta_sync_is_byte_idempotent_and_preserves_human_regions() {
        let temp = tempfile::tempdir().unwrap();
        let args = args_for("apps/demo", false);
        let scope = resolve_scope(temp.path(), &args).unwrap();
        execute(temp.path(), &scope, &args, ApplyMode::Init).unwrap();
        let readme = temp.path().join("apps/demo/README.md");
        let original = fs::read_to_string(&readme).unwrap();
        let with_human = format!("{original}\n## Human Notes\n\nKeep this exact.\n");
        fs::write(&readme, &with_human).unwrap();
        let agents = temp.path().join("AGENTS.md");
        let original_agents = fs::read_to_string(&agents).unwrap();
        let agents_with_human = format!(
            "{original_agents}\n## SDD Workflow\n\nKeep this legacy-named section exact.\n"
        );
        fs::write(&agents, &agents_with_human).unwrap();
        execute(temp.path(), &scope, &args, ApplyMode::Sync).unwrap();
        let once = fs::read_to_string(&readme).unwrap();
        execute(temp.path(), &scope, &args, ApplyMode::Sync).unwrap();
        let twice = fs::read_to_string(&readme).unwrap();
        assert_eq!(once, twice);
        assert!(twice.ends_with("## Human Notes\n\nKeep this exact.\n"));
        assert!(fs::read_to_string(agents)
            .unwrap()
            .ends_with("## SDD Workflow\n\nKeep this legacy-named section exact.\n"));
    }

    #[test]
    fn meta_check_detects_every_tampered_managed_block_and_sync_repairs_it() {
        let temp = tempfile::tempdir().unwrap();
        let args = args_for("apps/demo", false);
        let scope = resolve_scope(temp.path(), &args).unwrap();
        execute(temp.path(), &scope, &args, ApplyMode::Init).unwrap();
        let targets = [
            ("AGENTS.md", AW_START_MARKER),
            ("CLAUDE.md", AW_START_MARKER),
            ("README.md", REPO_README_START),
            ("CONTRIBUTING.md", REPO_CONTRIBUTING_START),
            ("CONTRIBUTING.md", META_DOC_MATRIX_START),
            ("apps/demo/README.md", PROJECT_README_START),
            ("apps/demo/CONTRIBUTING.md", PROJECT_CONTRIBUTING_START),
            ("apps/demo/CAPABILITIES.md", PROJECT_CAPABILITIES_START),
        ];
        for (path, marker) in targets {
            let path = temp.path().join(path);
            let body = fs::read_to_string(&path).unwrap();
            fs::write(
                &path,
                body.replacen(marker, &format!("{marker}\nTAMPERED"), 1),
            )
            .unwrap();
        }
        let drift = execute(temp.path(), &scope, &args, ApplyMode::Check).unwrap();
        assert_eq!(
            drift
                .findings
                .iter()
                .filter(|finding| finding.code == "managed_block_stale")
                .count(),
            8
        );
        assert_eq!(
            drift.next.unwrap().command,
            "aw meta sync --project-path apps/demo"
        );
        execute(temp.path(), &scope, &args, ApplyMode::Sync).unwrap();
        let clean = execute(temp.path(), &scope, &args, ApplyMode::Check).unwrap();
        assert!(clean.findings.is_empty(), "{:#?}", clean.findings);
        assert_eq!(clean.terminal.unwrap().status, "done");
    }

    #[test]
    fn meta_check_is_read_only_on_drift() {
        let temp = tempfile::tempdir().unwrap();
        let args = args_for("apps/demo", false);
        let scope = resolve_scope(temp.path(), &args).unwrap();
        execute(temp.path(), &scope, &args, ApplyMode::Init).unwrap();
        let path = temp.path().join("apps/demo/CAPABILITIES.md");
        let tampered = fs::read_to_string(&path)
            .unwrap()
            .replace("Machine-readable", "Tampered");
        fs::write(&path, &tampered).unwrap();
        let output = execute(temp.path(), &scope, &args, ApplyMode::Check).unwrap();
        assert!(!output.findings.is_empty());
        assert_eq!(fs::read_to_string(path).unwrap(), tampered);
    }

    #[test]
    fn meta_check_reports_unmatched_agent_marker_without_rewriting() {
        let temp = tempfile::tempdir().unwrap();
        let args = args_for("apps/demo", false);
        let scope = resolve_scope(temp.path(), &args).unwrap();
        execute(temp.path(), &scope, &args, ApplyMode::Init).unwrap();
        let path = temp.path().join("AGENTS.md");
        let malformed = fs::read_to_string(&path)
            .unwrap()
            .replacen(AW_END_MARKER, "", 1);
        fs::write(&path, &malformed).unwrap();

        let output = execute(temp.path(), &scope, &args, ApplyMode::Check).unwrap();

        assert!(output.findings.iter().any(|finding| {
            finding.code == "managed_block_malformed" && finding.path == "AGENTS.md"
        }));
        assert_eq!(fs::read_to_string(path).unwrap(), malformed);
    }

    #[test]
    fn meta_check_command_returns_nonzero_error_with_sync_remediation() {
        let temp = tempfile::tempdir().unwrap();
        let args = args_for("apps/demo", false);
        let scope = resolve_scope(temp.path(), &args).unwrap();
        execute(temp.path(), &scope, &args, ApplyMode::Init).unwrap();
        let path = temp.path().join("apps/demo/CAPABILITIES.md");
        let tampered = fs::read_to_string(&path)
            .unwrap()
            .replace("Machine-readable", "Tampered");
        fs::write(&path, &tampered).unwrap();

        let error = run_at_root(
            temp.path(),
            MetaArgs {
                command: MetaCommand::Check(args),
            },
        )
        .unwrap_err();

        assert!(error
            .to_string()
            .contains("aw meta sync --project-path apps/demo"));
        assert_eq!(fs::read_to_string(path).unwrap(), tampered);
    }

    #[test]
    fn meta_repository_product_applies_both_layers_at_root() {
        let temp = tempfile::tempdir().unwrap();
        let args = MetaScopeArgs {
            repository_product: true,
            ..MetaScopeArgs::default()
        };
        let scope = resolve_scope(temp.path(), &args).unwrap();
        let output = execute(temp.path(), &scope, &args, ApplyMode::Init).unwrap();
        assert!(output.findings.is_empty(), "{:#?}", output.findings);
        assert!(temp.path().join("CAPABILITIES.md").is_file());
        let readme = fs::read_to_string(temp.path().join("README.md")).unwrap();
        assert!(readme.contains(REPO_README_START));
        assert!(readme.contains(PROJECT_README_START));
    }

    #[test]
    fn legacy_agent_projector_uses_same_registry_and_runtime_whitelist() {
        let temp = tempfile::tempdir().unwrap();
        sync_repository_product_docs(temp.path()).unwrap();
        let claude = fs::read_to_string(temp.path().join("CLAUDE.md")).unwrap();
        let agents = fs::read_to_string(temp.path().join("AGENTS.md")).unwrap();
        assert!(agents.contains(doc_mirror::CODEX_TRANSLATE_PARAGRAPH));
        assert!(!claude.contains(doc_mirror::CODEX_TRANSLATE_PARAGRAPH));
        assert!(claude.contains("| `aw meta` |"));
        assert!(agents.contains("| `aw meta` |"));
    }

    // -- #1912: stale-binary reprojection must never destroy newer content --

    fn write_checkout_claude_template(root: &Path, text: &str) {
        let checkout_dir = root.join("apps/agentic-workflow/templates/cli/mainthread");
        fs::create_dir_all(&checkout_dir).unwrap();
        fs::write(checkout_dir.join("CLAUDE.md.tmpl"), text).unwrap();
    }

    fn checkout_template_with_extra_prose(marker_line: &str) -> String {
        // Simulate a #1847-shaped prose edit: one new line inserted right
        // after `<!-- aw:start -->`, absent from the embedded snapshot this
        // test binary compiled.
        EMBEDDED_CLAUDE_TEMPLATE.replacen(
            AW_START_MARKER,
            &format!("{AW_START_MARKER}\n{marker_line}"),
            1,
        )
    }

    #[test]
    fn resolve_claude_template_prefers_checkout_copy_when_present() {
        let temp = tempfile::tempdir().unwrap();
        let newer = checkout_template_with_extra_prose("NEWER PROSE #1847");
        write_checkout_claude_template(temp.path(), &newer);

        let (text, from_checkout) = resolve_claude_template(temp.path());

        assert!(from_checkout);
        assert_eq!(text, newer);
    }

    #[test]
    fn resolve_claude_template_falls_back_to_embedded_when_checkout_copy_absent() {
        let temp = tempfile::tempdir().unwrap();

        let (text, from_checkout) = resolve_claude_template(temp.path());

        assert!(!from_checkout);
        assert_eq!(text, EMBEDDED_CLAUDE_TEMPLATE);
    }

    #[test]
    fn meta_sync_reproduces_2026_07_17_incident_shape_and_preserves_newer_content() {
        // AC1, red-then-green: an installed binary's embedded CLAUDE.md.tmpl
        // snapshot predates the checkout's template (exactly the
        // `de033a5fe875` incident shape). #1912 R2 must render from the
        // live checkout copy, so projections built from it survive
        // byte-for-byte across sync instead of being reprojected from the
        // (older) embedded snapshot.
        let temp = tempfile::tempdir().unwrap();
        let newer_template = checkout_template_with_extra_prose("NEWER PROSE #1847/#1848/#1859");
        write_checkout_claude_template(temp.path(), &newer_template);

        let args = MetaScopeArgs {
            repository_product: true,
            ..MetaScopeArgs::default()
        };
        let scope = resolve_scope(temp.path(), &args).unwrap();
        execute(temp.path(), &scope, &args, ApplyMode::Init).unwrap();

        let claude = fs::read_to_string(temp.path().join("CLAUDE.md")).unwrap();
        let agents = fs::read_to_string(temp.path().join("AGENTS.md")).unwrap();
        assert!(
            claude.contains("NEWER PROSE #1847/#1848/#1859"),
            "CLAUDE.md must be projected from the checkout template, not the embedded snapshot"
        );
        assert!(agents.contains("NEWER PROSE #1847/#1848/#1859"));
        // Sanity: prove this is a meaningful, non-vacuous distinguishing
        // fixture — the embedded-only render genuinely lacks the line, so
        // a pre-fix (embedded-only) sync really would have destroyed it.
        assert!(!rendered_claude_document(EMBEDDED_CLAUDE_TEMPLATE)
            .contains("NEWER PROSE #1847/#1848/#1859"));

        // Sync must be byte-idempotent (AC2) and must not regress the
        // checkout-sourced content it just wrote.
        execute(temp.path(), &scope, &args, ApplyMode::Sync).unwrap();
        let claude_after_sync = fs::read_to_string(temp.path().join("CLAUDE.md")).unwrap();
        let agents_after_sync = fs::read_to_string(temp.path().join("AGENTS.md")).unwrap();
        assert_eq!(claude, claude_after_sync);
        assert_eq!(agents, agents_after_sync);
    }

    #[test]
    fn lines_removed_reports_only_non_blank_deleted_lines() {
        let existing = "kept\nremoved\n\nalso removed\n";
        let desired = "kept\n\n";
        let removed = lines_removed(existing, desired);
        assert_eq!(
            removed,
            vec!["removed".to_string(), "also removed".to_string()]
        );
    }

    #[test]
    fn lines_removed_empty_when_nothing_deleted() {
        let existing = "kept\nstill kept\n";
        let desired = "still kept\nkept\nnew line\n";
        assert!(lines_removed(existing, desired).is_empty());
    }

    #[test]
    fn embedded_template_is_stale_true_when_checkout_differs() {
        let temp = tempfile::tempdir().unwrap();
        let newer = checkout_template_with_extra_prose("DRIFTED PROSE");
        write_checkout_claude_template(temp.path(), &newer);

        assert!(embedded_template_is_stale(temp.path()));
    }

    #[test]
    fn embedded_template_is_stale_false_when_checkout_matches() {
        let temp = tempfile::tempdir().unwrap();
        write_checkout_claude_template(temp.path(), EMBEDDED_CLAUDE_TEMPLATE);

        assert!(!embedded_template_is_stale(temp.path()));
    }

    #[test]
    fn embedded_template_is_stale_false_when_no_checkout_copy() {
        let temp = tempfile::tempdir().unwrap();
        assert!(!embedded_template_is_stale(temp.path()));
    }

    #[test]
    fn meta_sync_blocks_deletion_from_stale_binary_with_no_checkout_template() {
        // #1912 R3 defense in depth: no checkout template copy is available
        // (so R2's primary fix can't engage), but the installed binary is
        // provably behind the checkout (semver) and the embedded-sourced
        // rewrite would delete existing projection content — refuse.
        let temp = tempfile::tempdir().unwrap();
        let aw_dir = temp.path().join("apps/agentic-workflow");
        fs::create_dir_all(&aw_dir).unwrap();
        fs::write(
            aw_dir.join("Cargo.toml"),
            "[package]\nname = \"agentic-workflow\"\nversion = \"99.99.99\"\n",
        )
        .unwrap();

        let args = args_for("apps/demo", false);
        let scope = resolve_scope(temp.path(), &args).unwrap();
        execute(temp.path(), &scope, &args, ApplyMode::Init).unwrap();

        // Simulate content that used to be synced from a fixed checkout
        // template and is no longer reproducible from the embedded
        // snapshot alone.
        let claude_path = temp.path().join("CLAUDE.md");
        let with_extra = fs::read_to_string(&claude_path).unwrap().replacen(
            AW_START_MARKER,
            &format!("{AW_START_MARKER}\nPREVIOUSLY SYNCED PROSE"),
            1,
        );
        fs::write(&claude_path, &with_extra).unwrap();

        let error = run_at_root(
            temp.path(),
            MetaArgs {
                command: MetaCommand::Sync(args.clone()),
            },
        )
        .unwrap();
        let _ = error;
        assert_eq!(
            fs::read_to_string(&claude_path).unwrap(),
            with_extra,
            "blocked write must leave the existing projection untouched"
        );

        // `--force-stale` bypasses the guard.
        let forced = MetaScopeArgs {
            force_stale: true,
            ..args
        };
        let scope = resolve_scope(temp.path(), &forced).unwrap();
        execute(temp.path(), &scope, &forced, ApplyMode::Sync).unwrap();
        assert!(
            !fs::read_to_string(&claude_path)
                .unwrap()
                .contains("PREVIOUSLY SYNCED PROSE"),
            "--force-stale must let the embedded-sourced rewrite proceed"
        );
    }

    #[test]
    fn meta_check_names_rebuild_remediation_when_embedded_template_is_stale() {
        // AC3 / R4: when the embedded template snapshot is provably behind
        // the checkout's live copy, `aw meta check` must not invite a
        // destructive `aw meta sync` from this binary.
        let temp = tempfile::tempdir().unwrap();
        let args = args_for("apps/demo", false);
        let scope = resolve_scope(temp.path(), &args).unwrap();
        execute(temp.path(), &scope, &args, ApplyMode::Init).unwrap();

        let newer = checkout_template_with_extra_prose("CHECKOUT-ONLY PROSE");
        write_checkout_claude_template(temp.path(), &newer);

        let output = execute(temp.path(), &scope, &args, ApplyMode::Check).unwrap();

        assert!(!output.findings.is_empty(), "{:#?}", output.findings);
        assert_eq!(output.binary_stale.as_deref(), Some("checkout HEAD"));
        assert_eq!(
            output.next.unwrap().command,
            "cargo install --path apps/agentic-workflow"
        );
        for finding in &output.findings {
            assert!(
                !finding.remediation.starts_with("Run `aw meta sync"),
                "remediation must not recommend `aw meta sync` as the fix: {:#?}",
                finding
            );
            assert!(
                finding.remediation.contains("aw upgrade")
                    || finding.remediation.contains("cargo install"),
                "{:#?}",
                finding
            );
        }

        let error = run_at_root(
            temp.path(),
            MetaArgs {
                command: MetaCommand::Check(args),
            },
        )
        .unwrap_err();
        assert!(
            error.to_string().contains("cargo install") || error.to_string().contains("aw upgrade"),
            "{error}"
        );
        assert!(
            !error.to_string().starts_with("Run `aw meta sync"),
            "{error}"
        );
    }
}
// CODEGEN-END
~~~~~

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: apps/agentic-workflow/src/cli/meta.rs
    action: create
    section: source
    impl_mode: codegen
    description: |
      Issue #1498: implement the one public META-doc producer/checker registry,
      preserve human bytes outside owned markers, emit chainable remediation,
      and let aw new delegate instead of retaining a competing projector.
  - path: apps/agentic-workflow/src/cli/meta.rs
    action: modify
    section: source
    impl_mode: codegen
    description: |
      Issue #1912: prefer the live checkout copy of the CLAUDE template over
      the binary's embedded include_str! snapshot when rendering repo-claude
      and repo-agents projections (R2), eliminating the stale-binary
      reprojection skew window that destroyed newer projection content on
      2026-07-17; add a content-regression guard that refuses a projection
      write that would delete existing content when the embedded fallback is
      used and the binary is provably behind the checkout, with a
      --force-stale override (R3); and let aw meta check distinguish
      binary-stale drift (remediation: rebuild/upgrade) from genuine drift
      (remediation: aw meta sync) via a binary_stale output field (R4).
```
