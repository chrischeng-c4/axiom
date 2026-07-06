// SPEC-MANAGED: projects/agentic-workflow/tech-design/surface/interfaces/src/cb.md#source
// CODEGEN-BEGIN
//! Code-artifact workflow verb implementations inherited by `aw td`.
//!
//! `cb` is the canonical namespace for code generation, code checks, and
//! HANDWRITE marker fill/review flows. The lifecycle phase written by `td gen`
//! is `cb_genned` and the canonical `Lifecycle-Stage:` trailer is `Cb-Gen`.
//!
//! @spec projects/agentic-workflow/tech-design/surface/specs/score-namespaces.md#changes

use anyhow::{Context, Result};
use clap::Args;
#[cfg(test)]
use clap::Subcommand;
use serde::Serialize;
use std::collections::{BTreeMap, BTreeSet};

use crate::cli::td::{self, AuditArgs, AuditGroupBy, GenCodeArgs};

const AW_EC_BEGIN_MARKER: &str = "AW-EC-BEGIN";

// Kept test-only (issue #860 cleanup): the production `CbArgs`/`run`
// dispatcher this enum backed had zero callers (no `aw cb` CLI surface
// remains; `aw td` mounts `run_gen`/`run_gen_source`/`run_check`/`run_claim`/
// `cb_fill::run` directly) and was deleted. This enum is retained only as
// the `#[command(subcommand)]` type for the `TestCbCli` clap-parsing test
// harness below (see `cb_gen_force_regen_parses_without_slug` and friends).
#[cfg(test)]
#[derive(Debug, Subcommand)]
enum CbCommand {
    // Generate implementation code from an approved TD spec.
    Gen(CbGenArgs),
    // Forward-generate a target source file from a per-file rust-source-unit
    // TD (routes through the @spec-injecting lossless item-tree generator).
    GenSource(CbGenSourceArgs),
    // Audit code-space files for CODEGEN drift, MarkerGap, Uncovered,
    // and Handwrite items.
    Check(CbCheckArgs),
    // Adopt existing code into score by generating a TD spec via the
    // fillback pipeline.
    Claim(CbClaimArgs),
    // Fill handwrite marker blocks in generated code (Phase 3).
    Fill(CbFillArgs),
}

// Args for `aw td fill <slug>` — Phase 3 marker-fill workflow.
///
// @spec projects/agentic-workflow/tech-design/surface/specs/score-cb-fill-workflow.md#cli
#[derive(Debug, Args)]
pub struct CbFillArgs {
    // Issue slug identifying the approved tech-design branch.
    pub slug: String,
    // Active TD spec path. Used by brief mode to scope markers to the
    // spec's Changes paths.
    #[arg(long)]
    pub spec_path: Option<String>,
    // Merge mode. When set, --marker is required. Merges the payload at
    // `/tmp/aw/workspaces/<workspace>/payloads/<slug>/<marker>.md` into the
    // matching begin/end marker block.
    #[arg(long)]
    pub apply: bool,
    // Marker identifier (matches the `gap` attribute on the
    // begin-marker line). Required with --apply.
    #[arg(long)]
    pub marker: Option<String>,
    // Emit envelope as pretty-printed JSON.
    #[arg(long)]
    pub json: bool,
    // Force brief mode to re-enumerate even if a dispatch was emitted earlier.
    #[arg(long)]
    pub force: bool,
}

// Args for `aw td code-claim <code-path>`.
#[derive(Debug, Args)]
// @spec projects/agentic-workflow/tech-design/surface/interfaces/src/cb.md#source
pub struct CbClaimArgs {
    // Path to a source file or directory to analyse.
    pub code_path: String,
    // Create `.aw/` workspace directory if it does not already exist.
    #[arg(long)]
    pub init: bool,
    // Skip filing/linking a durable tracker work-item for the adopted code
    // path. Tracker linkage is on by default (issue #925): adopted code
    // needs a durable root — a real work-item the commit/marker can point
    // back to — for traceability closure the same way any other lifecycle
    // artifact does. This opt-out exists for offline or sandboxed runs
    // where no issue backend is configured/reachable; the claim itself
    // (spec write + commit) still completes either way.
    #[arg(long)]
    pub no_issue: bool,
    // Tech-design group name. Inferred from the code path when omitted.
    #[arg(long)]
    pub group: Option<String>,
    // Emit result envelope as JSON.
    #[arg(long)]
    pub json: bool,
    // Suppress all interactive clarification prompts. Required for
    // non-TTY environments such as agent dispatch and CI pipelines.
    // Auto-enabled when stdin is not a terminal.
    // @spec projects/agentic-workflow/tech-design/surface/specs/score-recovery-verbs-non-interactive.md#cli
    #[arg(long)]
    pub non_interactive: bool,
}

// Args for `aw td gen <slug>` or
// `aw td gen --force-regen --project <project>`.
///
// @spec projects/agentic-workflow/tech-design/surface/specs/score-namespaces.md#changes
#[derive(Debug, Args)]
pub struct CbGenArgs {
    // Issue slug identifying the approved tech-design.
    pub slug: Option<String>,
    // Path to the spec file (relative to the current checkout root).
    #[arg(long)]
    pub spec_path: Option<String>,
    // Force-regenerate canonical source TD entries for codegen-owned files
    // under the configured project source scope. This bypasses issue phase
    // changes, commits, and lifecycle dispatch.
    #[arg(long)]
    pub force_regen: bool,
    // Project name whose configured td_path should be force-regenerated.
    // Required with --force-regen.
    #[arg(long)]
    pub project: Option<String>,
    // Workspace name under the selected project. Narrows force-regeneration
    // source roots while keeping the project's td_path and issue routing.
    #[arg(long)]
    pub workspace: Option<String>,
    // Preview force regeneration without writing files.
    #[arg(long)]
    pub dry_run: bool,
    // Verify project sources are byte-equivalent after replaying their TD
    // generation logic in a temporary checkout copy.
    #[arg(long)]
    pub verify: bool,
    // Verify project sources can be rebuilt from TD/spec into an empty
    // temporary source root. Unlike --verify, this does not copy current
    // source files into the temp root before replay.
    #[arg(long)]
    pub verify_cold: bool,
    // With --verify, print a deterministic sample of source sections that
    // still require agent semantic review. Use a ratio such as 0.15.
    #[arg(long)]
    pub semantic_sample: Option<f64>,
    // With --force-regen, refresh AST-derived public API manifests inside
    // canonical source TD Overview sections before replaying code.
    #[arg(long)]
    pub sync_public_api: bool,
}

// Args for `aw td code-check <target>`.
///
// @spec projects/agentic-workflow/tech-design/surface/specs/score-namespaces.md#changes
#[derive(Debug, Args)]
pub struct CbCheckArgs {
    // Issue slug or file/directory path to audit.
    pub target: Option<String>,
    // Output as JSON.
    #[arg(long)]
    pub json: bool,
    // Group findings by gap / file / status.
    #[arg(long, value_enum)]
    pub group_by: Option<AuditGroupBy>,
    // Skip the empty-implementation gate (issue #847) that refuses to
    // complete a terminal code-check whose spec's entire promised
    // implementation is missing from disk. For legitimate spec-only
    // completions.
    #[arg(long)]
    pub allow_empty_impl: bool,
}

// Args for `aw td gen-source --spec <td> --target <rs>`.
#[derive(Debug, Args)]
pub struct CbGenSourceArgs {
    // Repo-relative path to the per-file source TD (with a `## Source`
    // rust-source-unit fence).
    #[arg(long)]
    pub spec: String,
    // Repo-relative path to the target source file to write.
    #[arg(long)]
    pub target: String,
    // Print the generated source to stdout without writing the target.
    #[arg(long)]
    pub dry_run: bool,
}

// Forward-generate a target source file from a per-file rust-source-unit TD,
// reusing the same generator path as codegen (@spec injection + lossless
// item-tree regeneration). The forward inverse of `td gen --force-regen`
// (which syncs TD<-source); this writes source<-TD.
pub fn run_gen_source(args: CbGenSourceArgs) -> Result<()> {
    let root = crate::find_project_root()?;
    let spec_abs = root.join(&args.spec);
    let target_abs = root.join(&args.target);
    let report = crate::generate::apply::run_apply_scoped_targets(
        &spec_abs,
        &root,
        args.dry_run,
        std::slice::from_ref(&target_abs),
    )
    .map_err(|e| anyhow::anyhow!("gen-source apply {} -> {}: {e}", args.spec, args.target))?;
    eprintln!(
        "gen-source {} -> {}: {} block(s) updated, {} file(s) created, wrote={} (dry_run={})",
        args.spec,
        args.target,
        report.total_blocks_updated(),
        report.files_created(),
        report.wrote_files,
        args.dry_run,
    );
    Ok(())
}

// Implementation of `aw td gen`.
///
// Slug mode delegates to the approved-TD lifecycle pipeline. `--force-regen`
// replays canonical source TD entries for codegen-owned files under a
// configured project source scope, without touching issue phase, commits, or
// dispatch envelopes.
///
// @spec projects/agentic-workflow/tech-design/surface/specs/score-namespaces.md#changes
pub async fn run_gen(args: CbGenArgs) -> Result<()> {
    if args.force_regen {
        if args.slug.is_some() || args.spec_path.is_some() {
            anyhow::bail!("--force-regen cannot be combined with slug or --spec-path");
        }
        let Some(project) = args.project.as_deref() else {
            anyhow::bail!("--force-regen requires --project <project>");
        };
        if args.verify && args.verify_cold {
            anyhow::bail!("--verify-cold cannot be combined with --verify");
        }
        if args.verify && args.dry_run {
            anyhow::bail!("--verify cannot be combined with --dry-run");
        }
        if args.verify_cold && args.dry_run {
            anyhow::bail!("--verify-cold cannot be combined with --dry-run");
        }
        if args.verify {
            if args.sync_public_api {
                anyhow::bail!("--sync-public-api cannot be combined with --verify");
            }
            ensure_td_lock_clean_for_project(&std::env::current_dir()?, project)?;
            return run_force_regen_verify(
                project,
                args.workspace.as_deref(),
                args.semantic_sample,
            );
        }
        if args.verify_cold {
            if args.sync_public_api {
                anyhow::bail!("--sync-public-api cannot be combined with --verify-cold");
            }
            if args.semantic_sample.is_some() {
                anyhow::bail!("--semantic-sample is only supported with --verify");
            }
            ensure_td_lock_clean_for_project(&std::env::current_dir()?, project)?;
            return run_force_regen_verify_cold(project, args.workspace.as_deref());
        }
        if args.semantic_sample.is_some() {
            anyhow::bail!("--semantic-sample is only supported with --verify");
        }
        ensure_td_lock_clean_for_project(&std::env::current_dir()?, project)?;
        return run_force_regen(
            args.dry_run,
            project,
            args.workspace.as_deref(),
            args.sync_public_api,
        );
    }
    if args.project.is_some() {
        anyhow::bail!("--project is only supported with --force-regen");
    }
    if args.workspace.is_some() {
        anyhow::bail!("--workspace is only supported with --force-regen --project <project>");
    }
    if args.dry_run {
        anyhow::bail!("--dry-run is only supported with --force-regen");
    }
    if args.verify {
        anyhow::bail!("--verify is only supported with --force-regen");
    }
    if args.verify_cold {
        anyhow::bail!("--verify-cold is only supported with --force-regen");
    }
    if args.semantic_sample.is_some() {
        anyhow::bail!("--semantic-sample is only supported with --force-regen --verify");
    }
    if args.sync_public_api {
        anyhow::bail!("--sync-public-api is only supported with --force-regen");
    }
    let Some(slug) = args.slug else {
        anyhow::bail!("Either specify a slug or use --force-regen --project <project>");
    };
    let td_args = GenCodeArgs {
        slug,
        spec_path: args.spec_path,
    };
    td::run_gen_code(td_args).await
}

fn ensure_td_lock_clean_for_project(root: &std::path::Path, project: &str) -> Result<()> {
    let status = crate::cli::td_lock::check_project_td_lock_at_root(root, project)?;
    if status.clean {
        return Ok(());
    }
    anyhow::bail!(
        "td gen requires a clean TD IR lock before generation: {}",
        status.message
    )
}

fn run_force_regen(
    dry_run: bool,
    project: &str,
    workspace: Option<&str>,
    sync_public_api: bool,
) -> Result<()> {
    use crate::generate::apply::run_apply_scoped_targets;

    let cwd = std::env::current_dir().context("failed to get current directory")?;
    let scope = resolve_project_force_regen_scope(&cwd, project, workspace)?;
    if !scope.td_root.exists() {
        println!("No specs to regenerate.");
        return Ok(());
    }
    let public_api_update_paths = if sync_public_api {
        sync_force_regen_public_api_manifests(&cwd, &scope, dry_run)?
    } else {
        Vec::new()
    };
    let public_api_updates = public_api_update_paths.len();

    let mut specs = Vec::new();
    collect_force_regen_specs(&cwd, &scope, &mut specs)?;
    specs.sort();
    specs.dedup();

    if specs.is_empty() {
        println!("No specs to regenerate.");
        return Ok(());
    }

    let mut updated_files = 0usize;
    let mut created_files = 0usize;
    let mut blocks_updated = 0usize;
    let mut changed_paths = Vec::new();

    for spec in &specs {
        let report = run_apply_scoped_targets(spec, &cwd, dry_run, &scope.source_roots)
            .map_err(|e| anyhow::anyhow!("regeneration failed for {}: {}", spec.display(), e))?;
        updated_files += report.files.iter().filter(|f| f.updated).count();
        created_files += report.files_created();
        blocks_updated += report.total_blocks_updated();
        if !dry_run {
            changed_paths.extend(
                report
                    .files
                    .iter()
                    .filter(|file| file.updated || file.created)
                    .map(|file| cwd.join(&file.path)),
            );
        }
        if dry_run {
            println!(
                "(dry-run) {}: {} block(s) would be updated",
                spec.display(),
                report.total_blocks_updated()
            );
        } else {
            println!(
                "Regenerated {}: {} file(s) updated ({} created, {} CODEGEN blocks)",
                spec.display(),
                report.files.len(),
                report.files_created(),
                report.total_blocks_updated(),
            );
        }
    }
    if !dry_run {
        changed_paths.extend(public_api_update_paths);
        changed_paths.sort();
        changed_paths.dedup();
        format_rust_files(&changed_paths)?;
        commit_force_regen(
            &cwd,
            project,
            workspace,
            specs.len(),
            updated_files,
            created_files,
            blocks_updated,
            public_api_updates,
            &changed_paths,
        )?;
    }

    println!(
        "td gen --force-regen --project {}{}: {} spec(s) from {}, {} file update(s), {} created, {} CODEGEN block(s), {} public API manifest update(s){}",
        project,
        workspace
            .map(|name| format!(" --workspace {name}"))
            .unwrap_or_default(),
        specs.len(),
        scope.td_root.display(),
        updated_files,
        created_files,
        blocks_updated,
        public_api_updates,
        if dry_run { " (dry-run)" } else { "" },
    );

    Ok(())
}

fn run_force_regen_verify(
    project: &str,
    workspace: Option<&str>,
    semantic_sample: Option<f64>,
) -> Result<()> {
    let cwd = std::env::current_dir().context("failed to get current directory")?;
    let scope = resolve_project_force_regen_scope(&cwd, project, workspace)?;
    let mut specs = Vec::new();
    collect_force_regen_specs(&cwd, &scope, &mut specs)?;
    specs.sort();
    specs.dedup();

    if specs.is_empty() {
        println!("No specs to verify.");
        return Ok(());
    }

    let conformance = verify_force_regen_conformance(&cwd, &scope)?;
    let mismatches = force_regen_replay_mismatches(&cwd, &scope, &specs)?;

    if !mismatches.is_empty() {
        anyhow::bail!(
            "td gen --force-regen --project {project}{} --verify failed: {} file(s) differ after TD replay:\n{}",
            workspace
                .map(|name| format!(" --workspace {name}"))
                .unwrap_or_default(),
            mismatches.len(),
            mismatches.join("\n")
        );
    }
    if !conformance.failures.is_empty() {
        anyhow::bail!(
            "td gen --force-regen --project {project}{} --verify failed deterministic conformance: {} finding(s):\n{}",
            workspace
                .map(|name| format!(" --workspace {name}"))
                .unwrap_or_default(),
            conformance.failures.len(),
            conformance.failures.join("\n")
        );
    }

    println!(
        "td gen --force-regen --project {}{} --verify: {} spec(s), {} source root(s), byte-equivalent after TD replay",
        project,
        workspace
            .map(|name| format!(" --workspace {name}"))
            .unwrap_or_default(),
        specs.len(),
        scope.source_roots.len(),
    );
    conformance.print_text(semantic_sample)?;

    Ok(())
}

fn run_force_regen_verify_cold(project: &str, workspace: Option<&str>) -> Result<()> {
    let cwd = std::env::current_dir().context("failed to get current directory")?;
    let summary = force_regen_verify_cold_summary_at(&cwd, project, workspace)?;
    if summary.spec_count == 0 {
        println!("No specs to cold-verify.");
        return Ok(());
    }

    let failures = summary.failures.clone();

    if !failures.is_empty() {
        anyhow::bail!(
            "td gen --force-regen --project {project}{} --verify-cold failed: {} finding(s):\n{}",
            workspace
                .map(|name| format!(" --workspace {name}"))
                .unwrap_or_default(),
            failures.len(),
            failures.join("\n")
        );
    }

    println!(
        "td gen --force-regen --project {}{} --verify-cold: {} spec(s), {} source root(s), rebuilt expected targets from TD only",
        project,
        workspace
            .map(|name| format!(" --workspace {name}"))
            .unwrap_or_default(),
        summary.spec_count,
        summary.source_root_count,
    );
    println!(
        "cold_rebuild: files {}/{}",
        summary.generated_files, summary.expected_files
    );
    println!(
        "codegen_origin: td_ast {}/{} ({:.1}%), source_template {}/{} ({:.1}%), artifact_replay {}/{} ({:.1}%)",
        summary.codegen_origin.td_ast_files,
        summary.codegen_origin.target_files,
        percent_of(
            summary.codegen_origin.td_ast_files,
            summary.codegen_origin.target_files
        ),
        summary.codegen_origin.source_template_files,
        summary.codegen_origin.target_files,
        percent_of(
            summary.codegen_origin.source_template_files,
            summary.codegen_origin.target_files
        ),
        summary.codegen_origin.artifact_replay_files,
        summary.codegen_origin.target_files,
        percent_of(
            summary.codegen_origin.artifact_replay_files,
            summary.codegen_origin.target_files
        )
    );

    Ok(())
}

fn force_regen_verify_cold_summary_at(
    cwd: &std::path::Path,
    project: &str,
    workspace: Option<&str>,
) -> Result<CbColdVerifySummary> {
    let scope = resolve_project_force_regen_scope(cwd, project, workspace)?;
    let canonical_targets = collect_canonical_spec_refs_by_target(cwd, &scope)?;
    let (mut specs, expected_targets) = if canonical_targets.is_empty() {
        let mut specs = Vec::new();
        collect_force_regen_specs_from_td_changes(cwd, &scope, &mut specs)?;
        specs.sort();
        specs.dedup();
        let expected_targets = force_regen_cold_expected_targets(cwd, &scope, &specs)?;
        (specs, expected_targets)
    } else {
        let mut specs = canonical_targets.values().cloned().collect::<Vec<_>>();
        specs.sort();
        specs.dedup();
        (specs, canonical_targets.into_keys().collect())
    };
    specs.retain(|spec| spec.exists());
    let expected_files = expected_targets.len();
    let codegen_origin = codegen_origin_for_cold_targets(cwd, &scope, &specs, &expected_targets)?;

    if specs.is_empty() {
        return Ok(CbColdVerifySummary {
            workspace: workspace.map(str::to_string),
            clean: true,
            spec_count: 0,
            source_root_count: scope.source_roots.len(),
            generated_files: 0,
            expected_files,
            codegen_origin,
            failures: Vec::new(),
        });
    }

    let temp_root = create_force_regen_cold_root(cwd, &scope)?;
    let temp_scope = ForceRegenScope {
        td_root: temp_root.join(
            scope
                .td_root
                .strip_prefix(cwd)
                .context("project td_root must live under the current checkout")?,
        ),
        source_roots: scope
            .source_roots
            .iter()
            .map(|root| {
                root.strip_prefix(cwd)
                    .map(|rel| temp_root.join(rel))
                    .context("project source root must live under the current checkout")
            })
            .collect::<Result<Vec<_>>>()?,
    };
    let temp_specs = specs
        .iter()
        .map(|spec| {
            spec.strip_prefix(cwd)
                .map(|rel| temp_root.join(rel))
                .context("project TD spec must live under the current checkout")
        })
        .collect::<Result<Vec<_>>>()?;
    let snapshot_targets = collect_source_snapshot_targets(cwd, &scope, &specs)?;

    let verify_result = (|| {
        let (_, _, _, changed_paths) =
            run_force_regen_specs(&temp_root, &temp_scope, &temp_specs, false, true)?;
        write_project_root_llms_targets(cwd, &temp_root, &specs, false)?;
        format_rust_files(&changed_paths)?;
        let generated_files =
            count_existing_or_snapshot_targets(&temp_root, &expected_targets, &snapshot_targets);
        let mismatches = compare_cold_rebuild_targets(
            cwd,
            &temp_root,
            &scope.source_roots,
            &expected_targets,
            &snapshot_targets,
        )?;
        Ok::<_, anyhow::Error>((generated_files, mismatches))
    })();
    if std::env::var_os("SCORE_KEEP_FORCE_REGEN_VERIFY_ROOT").is_some() {
        eprintln!(
            "[agentic-workflow] kept force-regen cold root: {}",
            temp_root.display()
        );
    } else {
        std::fs::remove_dir_all(&temp_root).ok();
    }
    let (generated_files, mismatches) = verify_result?;

    let failures = mismatches
        .into_iter()
        .map(|mismatch| format!("{mismatch}: missing after cold TD rebuild"))
        .collect::<Vec<_>>();
    Ok(CbColdVerifySummary {
        workspace: workspace.map(str::to_string),
        clean: failures.is_empty(),
        spec_count: specs.len(),
        source_root_count: scope.source_roots.len(),
        generated_files,
        expected_files,
        codegen_origin,
        failures,
    })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum CbCodegenOriginClass {
    TdAst,
    SourceTemplate,
    ArtifactReplay,
}

fn codegen_origin_for_cold_targets(
    cwd: &std::path::Path,
    scope: &ForceRegenScope,
    specs: &[std::path::PathBuf],
    expected_targets: &BTreeSet<std::path::PathBuf>,
) -> Result<CbCodegenOriginSummary> {
    let mut classes: BTreeMap<std::path::PathBuf, CbCodegenOriginClass> = BTreeMap::new();

    for spec in specs {
        let content = std::fs::read_to_string(spec)
            .with_context(|| format!("failed to read {}", spec.display()))?;
        let class = classify_codegen_origin_spec(&content);
        for target in extract_cold_rebuild_target_paths(&content)
            .into_iter()
            .filter(|target| expected_targets.contains(target))
            .filter(|target| target_is_in_scope(cwd, scope, target))
        {
            classes
                .entry(target)
                .and_modify(|existing| {
                    if class > *existing {
                        *existing = class;
                    }
                })
                .or_insert(class);
        }
    }

    let mut summary = CbCodegenOriginSummary {
        target_files: expected_targets.len(),
        artifact_replay_files: 0,
        source_template_files: 0,
        td_ast_files: 0,
    };

    for target in expected_targets {
        match classes
            .get(target)
            .copied()
            .unwrap_or(CbCodegenOriginClass::TdAst)
        {
            CbCodegenOriginClass::TdAst => {
                summary.td_ast_files += 1;
            }
            CbCodegenOriginClass::SourceTemplate => {
                summary.source_template_files += 1;
            }
            CbCodegenOriginClass::ArtifactReplay => {
                summary.artifact_replay_files += 1;
            }
        }
    }

    Ok(summary)
}

fn classify_codegen_origin_spec(spec_content: &str) -> CbCodegenOriginClass {
    if spec_content.contains("source-from-target") || spec_content.contains("<!-- source-snapshot:")
    {
        CbCodegenOriginClass::ArtifactReplay
    } else if source_section_has_type_marker(spec_content, "type: rust-source-unit")
        || source_section_has_type_marker(spec_content, "type: text-source-unit")
    {
        CbCodegenOriginClass::TdAst
    } else if spec_declares_source_section(spec_content) {
        CbCodegenOriginClass::SourceTemplate
    } else {
        CbCodegenOriginClass::TdAst
    }
}

fn source_section_has_type_marker(spec_content: &str, marker: &str) -> bool {
    let mut in_source = false;
    for line in spec_content.lines() {
        if line.starts_with("## ") {
            let heading = line.trim_start_matches('#').trim();
            in_source = heading.eq_ignore_ascii_case("Source");
            continue;
        }
        if in_source && line.trim().contains(marker) {
            return true;
        }
    }
    false
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
// @spec projects/agentic-workflow/tech-design/surface/specs/project-health-governance-report.md#changes
pub struct CbVerifySummary {
    pub clean: bool,
    pub public_api_covered: usize,
    pub public_api_total: usize,
    pub semantic_review_required: usize,
    pub failures: Vec<String>,
}

#[derive(Debug, Clone, Default, Serialize, PartialEq, Eq)]
// @spec projects/agentic-workflow/tech-design/surface/specs/project-health-governance-report.md#changes
pub struct CbCodegenOriginSummary {
    pub target_files: usize,
    pub td_ast_files: usize,
    pub artifact_replay_files: usize,
    pub source_template_files: usize,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
// @spec projects/agentic-workflow/tech-design/surface/specs/project-health-governance-report.md#changes
pub struct CbColdVerifySummary {
    pub workspace: Option<String>,
    pub clean: bool,
    pub spec_count: usize,
    pub source_root_count: usize,
    pub generated_files: usize,
    pub expected_files: usize,
    pub codegen_origin: CbCodegenOriginSummary,
    pub failures: Vec<String>,
}

fn percent_of(part: usize, total: usize) -> f64 {
    if total == 0 {
        0.0
    } else {
        (part as f64 / total as f64) * 100.0
    }
}

// Return the same deterministic cb force-regeneration verification signals as
// `td gen --force-regen --verify` without printing the verbose CLI report.
// @spec projects/agentic-workflow/tech-design/surface/specs/project-health-governance-report.md#changes
pub fn project_force_regen_verify_summary(project: &str) -> Result<CbVerifySummary> {
    let cwd = std::env::current_dir().context("failed to get current directory")?;
    let scope = resolve_project_force_regen_scope(&cwd, project, None)?;
    if !scope.td_root.exists() {
        return Ok(CbVerifySummary {
            clean: true,
            public_api_covered: 0,
            public_api_total: 0,
            semantic_review_required: 0,
            failures: Vec::new(),
        });
    }

    let mut specs = Vec::new();
    collect_force_regen_specs(&cwd, &scope, &mut specs)?;
    specs.sort();
    specs.dedup();
    if specs.is_empty() {
        return Ok(CbVerifySummary {
            clean: true,
            public_api_covered: 0,
            public_api_total: 0,
            semantic_review_required: 0,
            failures: Vec::new(),
        });
    }

    let report = verify_force_regen_conformance(&cwd, &scope)?;
    let replay_mismatches = force_regen_replay_mismatches_quiet(&cwd, &scope, &specs)?;
    Ok(cb_verify_summary_from_report(report, replay_mismatches))
}

fn cb_verify_summary_from_report(
    report: ForceRegenConformanceReport,
    replay_mismatches: Vec<String>,
) -> CbVerifySummary {
    let semantic_review_required = report.agent_review_units().len();
    let public_api_covered = report.td_semantic_public_symbols;
    let public_api_total = report.public_symbols;
    let mut failures = report.failures;
    failures.extend(
        replay_mismatches
            .into_iter()
            .map(|path| format!("{path}: differs after TD replay")),
    );
    CbVerifySummary {
        clean: failures.is_empty(),
        public_api_covered,
        public_api_total,
        semantic_review_required,
        failures,
    }
}

// Return cold rebuild verification summaries for configured opt-in workspaces.
// @spec projects/agentic-workflow/tech-design/surface/specs/project-health-governance-report.md#changes
pub fn project_force_regen_cold_verify_summary(project: &str) -> Result<Vec<CbColdVerifySummary>> {
    let cwd = std::env::current_dir().context("failed to get current directory")?;
    let workspaces = project_cold_verify_workspaces(&cwd, project)?;
    let mut summaries = Vec::new();
    for workspace in workspaces {
        summaries.push(force_regen_verify_cold_summary_at(
            &cwd,
            project,
            Some(&workspace),
        )?);
    }
    Ok(summaries)
}

// Return configured cold rebuild workspace names without running cold rebuilds.
// @spec projects/agentic-workflow/tech-design/surface/specs/project-health-governance-report.md#changes
pub fn project_force_regen_cold_verify_workspaces(project: &str) -> Result<Vec<String>> {
    let cwd = std::env::current_dir().context("failed to get current directory")?;
    project_cold_verify_workspaces(&cwd, project)
}

fn run_force_regen_specs(
    root: &std::path::Path,
    scope: &ForceRegenScope,
    specs: &[std::path::PathBuf],
    dry_run: bool,
    quiet: bool,
) -> Result<(usize, usize, usize, Vec<std::path::PathBuf>)> {
    use crate::generate::apply::{run_apply_scoped_targets, run_apply_scoped_targets_quiet};

    let mut updated_files = 0usize;
    let mut created_files = 0usize;
    let mut blocks_updated = 0usize;
    let mut changed_paths = Vec::new();

    for spec in specs {
        let rel_spec = spec.strip_prefix(std::env::current_dir()?).unwrap_or(spec);
        let spec_path = if spec.starts_with(root) {
            spec.clone()
        } else {
            root.join(rel_spec)
        };
        let report = if quiet {
            run_apply_scoped_targets_quiet(&spec_path, root, dry_run, &scope.source_roots)
        } else {
            run_apply_scoped_targets(&spec_path, root, dry_run, &scope.source_roots)
        }
        .map_err(|e| anyhow::anyhow!("regeneration failed for {}: {}", spec_path.display(), e))?;
        updated_files += report.files.iter().filter(|f| f.updated).count();
        created_files += report.files_created();
        blocks_updated += report.total_blocks_updated();
        let (llms_updated, llms_created, llms_paths) =
            write_project_root_llms_targets(root, root, &[spec_path], dry_run)?;
        updated_files += llms_updated;
        created_files += llms_created;
        if !dry_run {
            changed_paths.extend(
                report
                    .files
                    .iter()
                    .filter(|file| file.updated || file.created)
                    .map(|file| root.join(&file.path)),
            );
            changed_paths.extend(llms_paths);
        }
    }

    changed_paths.sort();
    changed_paths.dedup();
    Ok((updated_files, created_files, blocks_updated, changed_paths))
}

fn write_project_root_llms_targets(
    render_root: &std::path::Path,
    output_root: &std::path::Path,
    specs: &[std::path::PathBuf],
    dry_run: bool,
) -> Result<(usize, usize, Vec<std::path::PathBuf>)> {
    let mut targets = BTreeSet::new();
    for spec in specs {
        let content = std::fs::read_to_string(spec)
            .with_context(|| format!("failed to read {}", spec.display()))?;
        targets.extend(extract_project_root_llms_target_paths(&content));
    }

    let mut updated = 0usize;
    let mut created = 0usize;
    let mut changed_paths = Vec::new();
    for target in targets {
        let target_rel = target.to_string_lossy().replace('\\', "/");
        let Some(project) =
            crate::cli::standardize::configured_project_name_for_path(render_root, &target_rel)?
        else {
            continue;
        };
        let content = crate::cli::standardize::render_project_llms_txt(render_root, &project)?;
        let path = output_root.join(&target);
        let existed = path.exists();
        let changed = std::fs::read_to_string(&path)
            .map(|existing| existing != content)
            .unwrap_or(true);
        if !changed {
            continue;
        }
        if existed {
            updated += 1;
        } else {
            created += 1;
        }
        if !dry_run {
            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent)
                    .with_context(|| format!("failed to create {}", parent.display()))?;
            }
            std::fs::write(&path, content)
                .with_context(|| format!("failed to write {}", path.display()))?;
            changed_paths.push(path);
        }
    }
    changed_paths.sort();
    changed_paths.dedup();
    Ok((updated, created, changed_paths))
}

fn extract_project_root_llms_target_paths(spec_content: &str) -> Vec<std::path::PathBuf> {
    if !spec_content.contains("project_root_llms") {
        return Vec::new();
    }
    let mut targets = crate::generate::apply::extract_change_entries(spec_content)
        .into_iter()
        .filter(|entry| entry.impl_mode != crate::generate::apply::ImplMode::HandWritten)
        .map(|entry| std::path::PathBuf::from(entry.path))
        .filter(|path| {
            path.file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| name == "llms.txt")
        })
        .collect::<Vec<_>>();
    targets.sort();
    targets.dedup();
    targets
}

fn format_rust_files(paths: &[std::path::PathBuf]) -> Result<()> {
    let rust_files = paths
        .iter()
        .filter(|path| path.extension().and_then(|ext| ext.to_str()) == Some("rs"))
        .cloned()
        .collect::<Vec<_>>();
    if rust_files.is_empty() {
        return Ok(());
    }
    let rustfmt = crate::git::find_rustfmt_bin()
        .context("rustfmt binary not found on PATH or rustup defaults")?;
    for chunk in rust_files.chunks(100) {
        let output = std::process::Command::new(&rustfmt)
            .arg("--edition")
            .arg("2021")
            .arg("--config")
            .arg("skip_children=true")
            .arg("--")
            .args(chunk)
            .output()
            .context("failed to run rustfmt for force-regen output")?;
        if !output.status.success() {
            anyhow::bail!(
                "rustfmt failed for force-regen output: {}",
                String::from_utf8_lossy(&output.stderr).trim()
            );
        }
    }
    Ok(())
}

fn commit_force_regen(
    project_root: &std::path::Path,
    project: &str,
    workspace: Option<&str>,
    spec_count: usize,
    updated_files: usize,
    created_files: usize,
    blocks_updated: usize,
    public_api_updates: usize,
    paths: &[std::path::PathBuf],
) -> Result<()> {
    let workspace_title = workspace.map(|name| format!(":{name}")).unwrap_or_default();
    let workspace_trailer = workspace
        .map(|name| format!("Workspace: {name}\n"))
        .unwrap_or_default();
    let message = format!(
        "cb force-regen({project}{workspace_title})\n\n\
         Lifecycle-Stage: Cb-Force-Regen\n\
         Project: {project}\n\
         {workspace_trailer}\
         Specs: {spec_count}\n\
         Files-Updated: {updated_files}\n\
         Files-Created: {created_files}\n\
         Blocks-Updated: {blocks_updated}\n\
         Public-API-Updates: {public_api_updates}\n"
    );
    crate::git::commit_scoped_paths(project_root, paths, &message)?;
    Ok(())
}

struct ForceRegenScope {
    td_root: std::path::PathBuf,
    source_roots: Vec<std::path::PathBuf>,
}

fn resolve_project_force_regen_scope(
    cwd: &std::path::Path,
    project_name: &str,
    workspace_name: Option<&str>,
) -> Result<ForceRegenScope> {
    let config_path = cwd.join(".aw").join("config.toml");
    if !config_path.exists() {
        anyhow::bail!("td gen --force-regen requires .aw/config.toml project routing");
    }

    let content = std::fs::read_to_string(&config_path)
        .with_context(|| format!("failed to read {}", config_path.display()))?;
    let config: CbGenConfig = toml::from_str(&content)
        .with_context(|| format!("failed to parse {}", config_path.display()))?;

    let Some(project_config) = config
        .projects
        .iter()
        .find(|project| project.matches(project_name))
    else {
        let available = config
            .projects
            .iter()
            .map(|p| p.name.as_str())
            .collect::<Vec<_>>()
            .join(", ");
        anyhow::bail!("unknown td gen project `{project_name}`. Available projects: {available}");
    };

    let td_root =
        crate::services::project_registry::resolve_td_root_from_config(cwd, &project_config.name)
            .map(|resolved| std::path::PathBuf::from(resolved.root))
            .map_err(|err| anyhow::anyhow!("{}", err.message))?;
    let mut source_roots = if let Some(workspace_name) = workspace_name {
        workspace_source_roots(cwd, project_name, project_config, workspace_name)?
    } else {
        project_source_roots(cwd, project_config)
    };
    if source_roots.is_empty() {
        if let Some(workspace_name) = workspace_name {
            anyhow::bail!(
                "td gen project `{project_name}` workspace `{workspace_name}` has no source paths"
            );
        }
        anyhow::bail!("td gen project `{project_name}` has no source path or workspace paths");
    }
    source_roots.sort();
    source_roots.dedup();

    Ok(ForceRegenScope {
        td_root,
        source_roots,
    })
}

#[derive(Debug, serde::Deserialize)]
struct CbGenConfig {
    #[serde(default)]
    projects: Vec<CbGenProject>,
}

#[derive(Debug, serde::Deserialize)]
struct CbGenProject {
    name: String,
    #[serde(default)]
    aliases: Vec<String>,
    path: Option<String>,
    #[serde(default)]
    workspaces: Vec<CbGenWorkspace>,
}

impl CbGenProject {
    fn matches(&self, requested: &str) -> bool {
        self.name == requested || self.aliases.iter().any(|alias| alias == requested)
    }
}

#[derive(Debug, serde::Deserialize)]
struct CbGenWorkspace {
    name: Option<String>,
    #[serde(default)]
    paths: Vec<String>,
    #[serde(default)]
    verify_cold: bool,
}

fn project_source_roots(cwd: &std::path::Path, project: &CbGenProject) -> Vec<std::path::PathBuf> {
    let mut roots = Vec::new();
    if let Some(path) = project.path.as_deref() {
        if !path.is_empty() {
            roots.push(cwd.join(path));
        }
    }
    for workspace in &project.workspaces {
        for pattern in &workspace.paths {
            let root = scope_root_from_pattern(pattern);
            if !root.is_empty() {
                roots.push(cwd.join(root));
            }
        }
    }
    roots
}

fn workspace_source_roots(
    cwd: &std::path::Path,
    project_name: &str,
    project: &CbGenProject,
    workspace_name: &str,
) -> Result<Vec<std::path::PathBuf>> {
    let Some(workspace) = project
        .workspaces
        .iter()
        .find(|workspace| workspace.name.as_deref() == Some(workspace_name))
    else {
        let available = project
            .workspaces
            .iter()
            .filter_map(|workspace| workspace.name.as_deref())
            .collect::<Vec<_>>()
            .join(", ");
        anyhow::bail!(
            "unknown td gen workspace `{workspace_name}` for project `{project_name}`. Available workspaces: {available}"
        );
    };

    let mut roots = Vec::new();
    for pattern in &workspace.paths {
        let root = scope_root_from_pattern(pattern);
        if !root.is_empty() {
            roots.push(cwd.join(root));
        }
    }
    Ok(roots)
}

fn project_cold_verify_workspaces(
    cwd: &std::path::Path,
    project_name: &str,
) -> Result<Vec<String>> {
    let config_path = cwd.join(".aw").join("config.toml");
    if !config_path.exists() {
        anyhow::bail!("cb cold verify requires .aw/config.toml project routing");
    }

    let content = std::fs::read_to_string(&config_path)
        .with_context(|| format!("failed to read {}", config_path.display()))?;
    let config: CbGenConfig = toml::from_str(&content)
        .with_context(|| format!("failed to parse {}", config_path.display()))?;

    let Some(project_config) = config.projects.iter().find(|p| p.name == project_name) else {
        let available = config
            .projects
            .iter()
            .map(|p| p.name.as_str())
            .collect::<Vec<_>>()
            .join(", ");
        anyhow::bail!("unknown td gen project `{project_name}`. Available projects: {available}");
    };

    project_config
        .workspaces
        .iter()
        .filter(|workspace| workspace.verify_cold)
        .map(|workspace| {
            workspace.name.clone().with_context(|| {
                format!("project `{project_name}` has verify_cold workspace without a name")
            })
        })
        .collect()
}

fn scope_root_from_pattern(pattern: &str) -> &str {
    pattern
        .split('*')
        .next()
        .unwrap_or(pattern)
        .trim_end_matches('/')
}

fn should_skip_force_regen_scan_dir(path: &std::path::Path) -> bool {
    let Some(name) = path.file_name().and_then(|name| name.to_str()) else {
        return false;
    };
    matches!(
        name,
        ".aw"
            | ".git"
            | ".hg"
            | ".mypy_cache"
            | ".pytest_cache"
            | ".ruff_cache"
            | ".tox"
            | ".venv"
            | "__pycache__"
            | "build"
            | "coverage"
            | "dist"
            | "e2e-results"
            | "node_modules"
            | "playwright-report"
            | "target"
            | "test-results"
            | "venv"
    )
}

fn collect_force_regen_specs(
    cwd: &std::path::Path,
    scope: &ForceRegenScope,
    out: &mut Vec<std::path::PathBuf>,
) -> Result<()> {
    for source_root in &scope.source_roots {
        collect_spec_managed_refs(cwd, &scope.td_root, source_root, out)?;
    }
    collect_force_regen_specs_from_td_changes(cwd, scope, out)?;
    Ok(())
}

fn collect_spec_managed_refs(
    cwd: &std::path::Path,
    td_root: &std::path::Path,
    dir: &std::path::Path,
    out: &mut Vec<std::path::PathBuf>,
) -> Result<()> {
    if !dir.exists() {
        return Ok(());
    }
    if dir.is_file() {
        collect_spec_managed_refs_from_file(cwd, td_root, dir, out);
        return Ok(());
    }
    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        let file_type = entry.file_type()?;
        if file_type.is_symlink() {
            continue;
        }
        if file_type.is_dir() {
            if should_skip_force_regen_scan_dir(&path) {
                continue;
            }
            collect_spec_managed_refs(cwd, td_root, &path, out)?;
            continue;
        }
        if file_type.is_file() {
            collect_spec_managed_refs_from_file(cwd, td_root, &path, out);
        }
    }
    Ok(())
}

fn collect_spec_managed_refs_from_file(
    cwd: &std::path::Path,
    td_root: &std::path::Path,
    path: &std::path::Path,
    out: &mut Vec<std::path::PathBuf>,
) {
    let Ok(content) = std::fs::read_to_string(path) else {
        return;
    };
    if is_aw_ec_generated_content(&content) {
        return;
    }
    for spec_ref in extract_spec_managed_refs(&content) {
        let spec_path = cwd.join(&spec_ref);
        if spec_path.starts_with(td_root) && spec_path.exists() {
            out.push(spec_path);
        }
    }
}

fn collect_force_regen_specs_from_td_changes(
    cwd: &std::path::Path,
    scope: &ForceRegenScope,
    out: &mut Vec<std::path::PathBuf>,
) -> Result<()> {
    collect_force_regen_specs_from_td_changes_inner(cwd, scope, &scope.td_root, out)
}

fn collect_force_regen_specs_from_td_changes_inner(
    cwd: &std::path::Path,
    scope: &ForceRegenScope,
    dir: &std::path::Path,
    out: &mut Vec<std::path::PathBuf>,
) -> Result<()> {
    if !dir.exists() {
        return Ok(());
    }
    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        let file_type = entry.file_type()?;
        if file_type.is_symlink() {
            continue;
        }
        if file_type.is_dir() {
            if should_skip_force_regen_scan_dir(&path) {
                continue;
            }
            collect_force_regen_specs_from_td_changes_inner(cwd, scope, &path, out)?;
            continue;
        }
        if !file_type.is_file() {
            continue;
        }
        if path.extension().and_then(|ext| ext.to_str()) != Some("md") {
            continue;
        }
        let content = std::fs::read_to_string(&path)
            .with_context(|| format!("failed to read {}", path.display()))?;
        if extract_cold_rebuild_target_paths(&content)
            .into_iter()
            .any(|target| target_is_in_scope(cwd, scope, &target))
        {
            out.push(path);
        }
    }
    Ok(())
}

fn force_regen_cold_expected_targets(
    cwd: &std::path::Path,
    scope: &ForceRegenScope,
    specs: &[std::path::PathBuf],
) -> Result<BTreeSet<std::path::PathBuf>> {
    let mut targets = BTreeSet::new();
    for spec in specs {
        let content = std::fs::read_to_string(spec)
            .with_context(|| format!("failed to read {}", spec.display()))?;
        targets.extend(
            extract_cold_rebuild_target_paths(&content)
                .into_iter()
                .filter(|target| target_is_in_scope(cwd, scope, target)),
        );
    }
    Ok(targets)
}

fn target_is_in_scope(
    cwd: &std::path::Path,
    scope: &ForceRegenScope,
    target_rel: &std::path::Path,
) -> bool {
    let target_path = cwd.join(target_rel);
    scope
        .source_roots
        .iter()
        .any(|source_root| target_path.starts_with(source_root))
}

fn collect_canonical_spec_refs_by_target(
    cwd: &std::path::Path,
    scope: &ForceRegenScope,
) -> Result<BTreeMap<std::path::PathBuf, std::path::PathBuf>> {
    let mut refs = BTreeMap::new();
    for source_root in &scope.source_roots {
        collect_canonical_spec_refs_by_target_inner(cwd, &scope.td_root, source_root, &mut refs)?;
    }
    Ok(refs)
}

fn collect_canonical_spec_refs_by_target_inner(
    cwd: &std::path::Path,
    td_root: &std::path::Path,
    path: &std::path::Path,
    refs: &mut BTreeMap<std::path::PathBuf, std::path::PathBuf>,
) -> Result<()> {
    if !path.exists() {
        return Ok(());
    }
    if path.is_dir() {
        for entry in std::fs::read_dir(path)? {
            let entry = entry?;
            let child = entry.path();
            let file_type = entry.file_type()?;
            if file_type.is_symlink() {
                continue;
            }
            if file_type.is_dir() && should_skip_force_regen_scan_dir(&child) {
                continue;
            }
            collect_canonical_spec_refs_by_target_inner(cwd, td_root, &child, refs)?;
        }
        return Ok(());
    }

    let Ok(content) = std::fs::read_to_string(path) else {
        return Ok(());
    };
    if is_aw_ec_generated_content(&content) {
        return Ok(());
    }
    let Some(spec_ref) = extract_spec_managed_ref(&content) else {
        return Ok(());
    };
    let spec_path = cwd.join(spec_ref);
    if !spec_path.starts_with(td_root) || !spec_path.exists() {
        return Ok(());
    }
    let target_path = path
        .strip_prefix(cwd)
        .with_context(|| format!("source file must live under cwd: {}", path.display()))?
        .to_path_buf();
    let spec_content = std::fs::read_to_string(&spec_path)
        .with_context(|| format!("failed to read {}", spec_path.display()))?;
    if !extract_cold_rebuild_target_paths(&spec_content).contains(&target_path) {
        return Ok(());
    }
    refs.insert(target_path, spec_path);
    Ok(())
}

fn is_aw_ec_generated_content(content: &str) -> bool {
    content.contains(AW_EC_BEGIN_MARKER)
}

fn extract_spec_managed_ref(content: &str) -> Option<String> {
    extract_spec_managed_refs_with_sections(content)
        .into_iter()
        .find(|(_, section)| section.as_deref().is_some_and(is_source_unit_section_name))
        .map(|(spec, _)| spec)
        .or_else(|| extract_spec_managed_refs(content).into_iter().next())
}

fn extract_spec_managed_refs(content: &str) -> Vec<String> {
    let mut refs = extract_spec_managed_refs_with_sections(content)
        .into_iter()
        .map(|(spec, _)| spec)
        .collect::<Vec<_>>();
    refs.sort();
    refs.dedup();
    refs
}

fn extract_spec_managed_refs_with_sections(content: &str) -> Vec<(String, Option<String>)> {
    use crate::generate::marker::parse_codegen_blocks;

    let mut refs = parse_codegen_blocks(content)
        .into_iter()
        .filter_map(|block| parse_spec_managed_path_section(&block.spec_ref))
        .collect::<Vec<_>>();
    if refs.is_empty() {
        refs.extend(content.lines().filter_map(|line| {
            let trimmed = line.trim();
            let spec_ref = trimmed
                .strip_prefix("// SPEC-MANAGED: ")
                .or_else(|| trimmed.strip_prefix("# SPEC-MANAGED: "))
                .or_else(|| {
                    trimmed
                        .strip_prefix("<!-- SPEC-MANAGED: ")
                        .and_then(|s| s.strip_suffix(" -->"))
                })?;
            parse_spec_managed_path_section(spec_ref)
        }));
    }
    refs.sort();
    refs.dedup();
    refs
}

fn parse_spec_managed_path_section(spec_ref: &str) -> Option<(String, Option<String>)> {
    let spec_ref = spec_ref.trim();
    let (path, section) = spec_ref
        .split_once('#')
        .map(|(path, section)| (path.trim(), Some(section.trim())))
        .unwrap_or((spec_ref, None));
    (!path.is_empty()).then(|| {
        (
            path.to_string(),
            section
                .filter(|section| !section.is_empty())
                .map(str::to_string),
        )
    })
}

fn is_source_unit_section_name(section: &str) -> bool {
    matches!(section, "source" | "rust-source-unit" | "text-source-unit")
}

fn sync_force_regen_public_api_manifests(
    cwd: &std::path::Path,
    scope: &ForceRegenScope,
    dry_run: bool,
) -> Result<Vec<std::path::PathBuf>> {
    use crate::fillback::AstAnalyzer;
    use crate::generate::marker::parse_codegen_blocks;

    let mut analyzer = AstAnalyzer::new().context("failed to initialize AST analyzer")?;
    let mut manifests: BTreeMap<std::path::PathBuf, Vec<PublicApiManifestTarget>> = BTreeMap::new();
    let mut seen_targets = BTreeSet::new();
    for path in collect_source_scope_files(scope)? {
        if is_minified_asset_file(&path) || !is_supported_source_file(&path) {
            continue;
        }
        let rel_path = path.strip_prefix(cwd).unwrap_or(&path).to_path_buf();
        let content = std::fs::read_to_string(&path)
            .with_context(|| format!("failed to read {}", path.display()))?;
        let module = analyzer.parse_file(&path, &content).map_err(|err| {
            anyhow::anyhow!("{}: AST parse failed: {}", rel_path.display(), err.reason)
        })?;
        let source_refs = parse_codegen_blocks(&content)
            .into_iter()
            .filter_map(|block| parse_spec_ref(&block.spec_ref))
            .filter(|spec_ref| {
                spec_ref
                    .section
                    .as_deref()
                    .is_some_and(is_source_unit_section_name)
            })
            .collect::<Vec<_>>();
        for spec_ref in source_refs {
            let spec_path = cwd.join(&spec_ref.path);
            if !spec_path.starts_with(&scope.td_root) || !spec_path.exists() {
                continue;
            }
            let target_rel_path = rel_path.to_string_lossy().into_owned();
            if seen_targets.insert((spec_path.clone(), target_rel_path.clone())) {
                manifests
                    .entry(spec_path)
                    .or_default()
                    .push(public_api_manifest_target(
                        &target_rel_path,
                        &module.symbols,
                    ));
            }
        }
    }

    let mut updates = Vec::new();
    for (spec_path, targets) in manifests {
        let spec_content = std::fs::read_to_string(&spec_path)
            .with_context(|| format!("failed to read {}", spec_path.display()))?;
        let updated = upsert_public_api_overview_targets(&spec_content, &targets);
        if updated != spec_content {
            updates.push(spec_path.clone());
            if !dry_run {
                std::fs::write(&spec_path, updated)
                    .with_context(|| format!("failed to write {}", spec_path.display()))?;
            }
        }
    }
    Ok(updates)
}

fn create_force_regen_verify_root(
    cwd: &std::path::Path,
    scope: &ForceRegenScope,
) -> Result<std::path::PathBuf> {
    let temp_root = unique_force_regen_temp_root();
    for source_root in &scope.source_roots {
        let rel = source_root
            .strip_prefix(cwd)
            .context("project source root must live under the current checkout")?;
        copy_tree(source_root, &temp_root.join(rel))?;
    }
    let td_rel = scope
        .td_root
        .strip_prefix(cwd)
        .context("project td_root must live under the current checkout")?;
    copy_tree(&scope.td_root, &temp_root.join(td_rel))?;
    Ok(temp_root)
}

fn force_regen_replay_mismatches(
    cwd: &std::path::Path,
    scope: &ForceRegenScope,
    specs: &[std::path::PathBuf],
) -> Result<Vec<String>> {
    force_regen_replay_mismatches_with_quiet(cwd, scope, specs, false)
}

fn force_regen_replay_mismatches_quiet(
    cwd: &std::path::Path,
    scope: &ForceRegenScope,
    specs: &[std::path::PathBuf],
) -> Result<Vec<String>> {
    force_regen_replay_mismatches_with_quiet(cwd, scope, specs, true)
}

fn force_regen_replay_mismatches_with_quiet(
    cwd: &std::path::Path,
    scope: &ForceRegenScope,
    specs: &[std::path::PathBuf],
    quiet: bool,
) -> Result<Vec<String>> {
    let temp_root = create_force_regen_verify_root(cwd, scope)?;
    let temp_scope = ForceRegenScope {
        td_root: temp_root.join(
            scope
                .td_root
                .strip_prefix(cwd)
                .context("project td_root must live under the current checkout")?,
        ),
        source_roots: scope
            .source_roots
            .iter()
            .map(|root| {
                root.strip_prefix(cwd)
                    .map(|rel| temp_root.join(rel))
                    .context("project source root must live under the current checkout")
            })
            .collect::<Result<Vec<_>>>()?,
    };

    let verify_result = (|| {
        let (_, _, _, changed_paths) =
            run_force_regen_specs(&temp_root, &temp_scope, specs, false, quiet)?;
        format_rust_files(&changed_paths)?;
        compare_source_roots(cwd, &temp_root, &scope.source_roots)
    })();
    if std::env::var_os("SCORE_KEEP_FORCE_REGEN_VERIFY_ROOT").is_some() {
        eprintln!(
            "[td gen] kept force-regen verify root at {}",
            temp_root.display()
        );
    } else {
        std::fs::remove_dir_all(&temp_root).ok();
    }
    verify_result
}

fn create_force_regen_cold_root(
    cwd: &std::path::Path,
    scope: &ForceRegenScope,
) -> Result<std::path::PathBuf> {
    let temp_root = unique_force_regen_temp_root();
    let td_rel = scope
        .td_root
        .strip_prefix(cwd)
        .context("project td_root must live under the current checkout")?;
    copy_tree(&scope.td_root, &temp_root.join(td_rel))?;
    Ok(temp_root)
}

fn unique_force_regen_temp_root() -> std::path::PathBuf {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or_default();
    std::env::temp_dir()
        .join("aw")
        .join("force-regen")
        .join(format!("{}-{nanos}", std::process::id()))
}

fn copy_tree(src: &std::path::Path, dst: &std::path::Path) -> Result<()> {
    let mut visited_dirs = BTreeSet::new();
    copy_tree_inner(src, dst, &mut visited_dirs)
}

fn copy_tree_inner(
    src: &std::path::Path,
    dst: &std::path::Path,
    visited_dirs: &mut BTreeSet<std::path::PathBuf>,
) -> Result<()> {
    if !src.exists() {
        return Ok(());
    }
    let metadata = std::fs::symlink_metadata(src)?;
    let file_type = metadata.file_type();
    if file_type.is_symlink() {
        return Ok(());
    }
    if file_type.is_file() {
        if let Some(parent) = dst.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::copy(src, dst)?;
        return Ok(());
    }
    if !file_type.is_dir() {
        return Ok(());
    }
    let canonical = std::fs::canonicalize(src).unwrap_or_else(|_| src.to_path_buf());
    if !visited_dirs.insert(canonical) {
        return Ok(());
    }
    std::fs::create_dir_all(dst)?;
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let child_src = entry.path();
        let child_dst = dst.join(entry.file_name());
        let file_type = entry.file_type()?;
        if file_type.is_symlink() {
            continue;
        }
        if file_type.is_dir() {
            if should_skip_force_regen_scan_dir(&child_src) {
                continue;
            }
            copy_tree_inner(&child_src, &child_dst, visited_dirs)?;
        } else if file_type.is_file() {
            std::fs::copy(&child_src, &child_dst)?;
        }
    }
    Ok(())
}

fn compare_source_roots(
    original_root: &std::path::Path,
    generated_root: &std::path::Path,
    source_roots: &[std::path::PathBuf],
) -> Result<Vec<String>> {
    let mut mismatches = Vec::new();
    for source_root in source_roots {
        let rel_root = source_root
            .strip_prefix(original_root)
            .context("project source root must live under the current checkout")?;
        let original_files = collect_tree_files(source_root)?;
        let generated_files = collect_tree_files(&generated_root.join(rel_root))?;
        let paths = original_files
            .keys()
            .chain(generated_files.keys())
            .cloned()
            .collect::<std::collections::BTreeSet<_>>();
        for rel_file in paths {
            match (
                original_files.get(&rel_file),
                generated_files.get(&rel_file),
            ) {
                (Some(original), Some(generated)) => {
                    let original_bytes = std::fs::read(original)?;
                    let generated_bytes = std::fs::read(generated)?;
                    if original_bytes != generated_bytes {
                        mismatches.push(format!(
                            "{}",
                            source_root_mismatch_path(rel_root, &rel_file).display()
                        ));
                    }
                }
                (Some(_), None) => {
                    mismatches.push(format!(
                        "{} (missing)",
                        source_root_mismatch_path(rel_root, &rel_file).display()
                    ));
                }
                (None, Some(_)) => {
                    mismatches.push(format!(
                        "{} (extra)",
                        source_root_mismatch_path(rel_root, &rel_file).display()
                    ));
                }
                (None, None) => {}
            }
        }
    }
    Ok(mismatches)
}

fn source_root_mismatch_path(
    rel_root: &std::path::Path,
    rel_file: &std::path::Path,
) -> std::path::PathBuf {
    if rel_file.as_os_str().is_empty() {
        rel_root.to_path_buf()
    } else {
        rel_root.join(rel_file)
    }
}

fn compare_cold_rebuild_targets(
    _original_root: &std::path::Path,
    generated_root: &std::path::Path,
    _source_roots: &[std::path::PathBuf],
    expected_targets: &BTreeSet<std::path::PathBuf>,
    snapshot_targets: &BTreeSet<std::path::PathBuf>,
) -> Result<Vec<String>> {
    let mut mismatches = Vec::new();
    for rel_file in expected_targets {
        let generated = generated_root.join(rel_file);
        if !generated.exists() && !snapshot_targets.contains(rel_file) {
            mismatches.push(format!("{} (missing)", rel_file.display()));
        }
    }
    Ok(mismatches)
}

fn count_existing_or_snapshot_targets(
    root: &std::path::Path,
    targets: &BTreeSet<std::path::PathBuf>,
    snapshot_targets: &BTreeSet<std::path::PathBuf>,
) -> usize {
    targets
        .iter()
        .filter(|target| root.join(target).exists() || snapshot_targets.contains(*target))
        .count()
}

fn collect_source_snapshot_targets(
    cwd: &std::path::Path,
    scope: &ForceRegenScope,
    specs: &[std::path::PathBuf],
) -> Result<BTreeSet<std::path::PathBuf>> {
    let mut targets = BTreeSet::new();
    for spec in specs {
        let content = std::fs::read_to_string(spec)
            .with_context(|| format!("failed to read {}", spec.display()))?;
        for target in extract_source_snapshot_paths(&content) {
            if target_is_in_scope(cwd, scope, &target) {
                targets.insert(target);
            }
        }
    }
    Ok(targets)
}

fn extract_source_snapshot_paths(spec_content: &str) -> Vec<std::path::PathBuf> {
    let mut targets = Vec::new();
    for line in spec_content.lines() {
        let trimmed = line.trim();
        let Some(rest) = trimmed
            .strip_prefix("<!-- source-snapshot:")
            .and_then(|s| s.strip_suffix("-->"))
        else {
            continue;
        };
        let path = rest
            .trim()
            .strip_prefix("path=")
            .unwrap_or_else(|| rest.trim())
            .trim()
            .trim_matches('"')
            .trim_matches('\'')
            .trim_start_matches("./")
            .replace('\\', "/");
        if !path.is_empty() {
            targets.push(std::path::PathBuf::from(path));
        }
    }
    targets.sort();
    targets.dedup();
    targets
}

fn extract_cold_rebuild_target_paths(spec_content: &str) -> Vec<std::path::PathBuf> {
    let mut targets = crate::generate::apply::extract_change_entries(spec_content)
        .into_iter()
        .filter(|entry| entry.impl_mode != crate::generate::apply::ImplMode::HandWritten)
        .map(|entry| std::path::PathBuf::from(entry.path))
        .collect::<Vec<_>>();
    targets.sort();
    targets.dedup();
    targets
}

#[derive(Debug, Clone)]
struct SemanticReviewUnit {
    spec_ref: String,
    target_path: std::path::PathBuf,
    reason: String,
}

#[derive(Debug, Default)]
struct ForceRegenConformanceReport {
    code_files: usize,
    non_code_files: usize,
    managed_code_files: usize,
    managed_spec_refs: usize,
    source_templates: usize,
    deterministic_source_templates: usize,
    target_derived_source_templates: Vec<SemanticReviewUnit>,
    unmanaged_code_files: Vec<SemanticReviewUnit>,
    codegen_blocks: usize,
    ast_files: usize,
    ast_symbols: usize,
    public_symbols: usize,
    td_semantic_public_symbols: usize,
    source_templates_public_api_complete: usize,
    source_templates_public_api_partial: usize,
    source_templates_public_api_empty: usize,
    audit_clean: usize,
    audit_aggregate: usize,
    audit_handwrite: usize,
    failures: Vec<String>,
}

// @spec projects/agentic-workflow/tech-design/surface/interfaces/src/cb.md#source
impl ForceRegenConformanceReport {
    fn deterministic_units(&self) -> usize {
        self.managed_code_files + self.managed_spec_refs + self.codegen_blocks + self.ast_files
    }

    fn agent_review_units(&self) -> Vec<&SemanticReviewUnit> {
        self.target_derived_source_templates
            .iter()
            .chain(self.unmanaged_code_files.iter())
            .collect()
    }

    fn print_text(&self, semantic_sample: Option<f64>) -> Result<()> {
        println!(
            "deterministic_conformance: code_files {}/{}, managed_refs {}, codegen_blocks {}, ast_parse {}/{} files ({} symbols)",
            self.managed_code_files,
            self.code_files,
            self.managed_spec_refs,
            self.codegen_blocks,
            self.ast_files,
            self.code_files,
            self.ast_symbols,
        );
        println!(
            "source_template_conformance: {} deterministic, {} target-derived requiring semantic review, {} unmanaged source file(s)",
            self.deterministic_source_templates,
            self.target_derived_source_templates.len(),
            self.unmanaged_code_files.len(),
        );
        println!(
            "codegen_audit: clean {}, aggregate {}, handwrite {}, deterministic_units {}",
            self.audit_clean,
            self.audit_aggregate,
            self.audit_handwrite,
            self.deterministic_units(),
        );
        if self.source_templates > 0 {
            println!(
                "public_api_semantic_conformance: {}/{} public symbol(s) covered by structured TD sections; source templates complete {}, partial {}, empty-public-api {}",
                self.td_semantic_public_symbols,
                self.public_symbols,
                self.source_templates_public_api_complete,
                self.source_templates_public_api_partial,
                self.source_templates_public_api_empty,
            );
        }
        if self.non_code_files > 0 {
            println!(
                "non_code_files: {} ignored by deterministic source verification",
                self.non_code_files
            );
        }
        let review_units = self.agent_review_units();
        if let Some(ratio) = semantic_sample {
            let sampled = sample_semantic_review_units(&review_units, ratio)?;
            println!(
                "agent_semantic_sample: {}/{} unit(s) at ratio {:.2}",
                sampled.len(),
                review_units.len(),
                ratio,
            );
            for unit in sampled {
                println!(
                    "  - {} -> {} ({})",
                    unit.spec_ref,
                    unit.target_path.display(),
                    unit.reason,
                );
            }
        } else if !review_units.is_empty() {
            let recommended = sample_count(review_units.len(), 0.15);
            println!(
                "agent_review_required: {} unit(s); suggested sample {} at --semantic-sample 0.15",
                review_units.len(),
                recommended,
            );
        }
        Ok(())
    }

    fn enforce_complete_public_api_semantic_conformance(&mut self) {
        if self.public_symbols > self.td_semantic_public_symbols {
            self.failures.push(format!(
                "public API semantic conformance incomplete: {}/{} public symbol(s) covered",
                self.td_semantic_public_symbols, self.public_symbols
            ));
        }
    }

    fn enforce_complete_source_ownership_coverage(&mut self) {
        if self.unmanaged_code_files.is_empty() {
            return;
        }
        let mut targets = self
            .unmanaged_code_files
            .iter()
            .map(|unit| unit.target_path.display().to_string())
            .collect::<Vec<_>>();
        targets.sort();
        targets.truncate(5);
        self.failures.push(format!(
            "deterministic source ownership incomplete: {}/{} source file(s) have CODEGEN or HANDWRITE ownership markers; unmanaged files: {}",
            self.managed_code_files,
            self.code_files,
            targets.join(", ")
        ));
    }
}

fn verify_force_regen_conformance(
    cwd: &std::path::Path,
    scope: &ForceRegenScope,
) -> Result<ForceRegenConformanceReport> {
    use crate::fillback::AstAnalyzer;
    use crate::generate::audit::{audit_file_unified, build_spec_file_index, UnifiedReport};
    use crate::generate::marker::parse_codegen_blocks;

    let mut report = ForceRegenConformanceReport::default();
    let mut analyzer = AstAnalyzer::new().context("failed to initialize AST analyzer")?;
    let spec_index = build_spec_file_index(cwd).context("failed to build TD spec file index")?;
    let source_files = collect_source_scope_files(scope)?;

    for path in source_files {
        if is_minified_asset_file(&path) || !is_supported_source_file(&path) {
            report.non_code_files += 1;
            continue;
        }
        report.code_files += 1;
        let rel_path = path.strip_prefix(cwd).unwrap_or(&path).to_path_buf();
        let content = std::fs::read_to_string(&path)
            .with_context(|| format!("failed to read {}", path.display()))?;
        if is_aw_ec_generated_content(&content) {
            report.non_code_files += 1;
            continue;
        }

        let blocks = parse_codegen_blocks(&content);
        let handwrite_owned = has_handwrite_ownership_marker(&content);
        if blocks.is_empty() && !handwrite_owned {
            report.unmanaged_code_files.push(SemanticReviewUnit {
                spec_ref: "(none)".to_string(),
                target_path: rel_path.clone(),
                reason: "no-ownership-marker".to_string(),
            });
        }
        report.codegen_blocks += blocks.len();
        if !blocks.is_empty() || handwrite_owned {
            report.managed_code_files += 1;
        }
        let parsed_module = match analyzer.parse_file(&path, &content) {
            Ok(module) => {
                report.ast_files += 1;
                report.ast_symbols += module.symbols.len();
                Some(module)
            }
            Err(err) => {
                report.failures.push(format!(
                    "{}: AST parse failed: {}",
                    rel_path.display(),
                    err.reason
                ));
                None
            }
        };
        for block in &blocks {
            if let Some(block_ref) = parse_spec_ref(&block.spec_ref) {
                report.managed_spec_refs += 1;
                let valid = validate_spec_ref(
                    cwd,
                    &scope.td_root,
                    &block_ref,
                    "codegen block",
                    &rel_path,
                    &mut report,
                );
                if valid
                    && block_ref
                        .section
                        .as_deref()
                        .is_some_and(is_source_unit_section_name)
                {
                    classify_source_template(
                        cwd,
                        &block_ref,
                        &rel_path,
                        parsed_module
                            .as_ref()
                            .map(|module| module.symbols.as_slice()),
                        &mut report,
                    )?;
                }
            } else {
                report.failures.push(format!(
                    "{}: malformed CODEGEN block spec ref `{}`",
                    rel_path.display(),
                    block.spec_ref
                ));
            }
        }

        let reports = audit_file_unified(&path, cwd, &spec_index)
            .with_context(|| format!("failed to audit {}", path.display()))?;
        for finding in reports {
            match finding {
                UnifiedReport::Clean { .. } => report.audit_clean += 1,
                UnifiedReport::Aggregate { .. } => report.audit_aggregate += 1,
                UnifiedReport::Handwrite { .. } => report.audit_handwrite += 1,
                UnifiedReport::Drift { file, diff, .. } => report.failures.push(format!(
                    "{}: CODEGEN drift ({})",
                    file.strip_prefix(cwd).unwrap_or(&file).display(),
                    diff
                )),
                UnifiedReport::MarkerGap {
                    file,
                    item_line,
                    line_no,
                    ..
                } => report.failures.push(format!(
                    "{}:{}: CODEGEN item lacks @spec marker: {}",
                    file.strip_prefix(cwd).unwrap_or(&file).display(),
                    line_no,
                    item_line
                )),
                UnifiedReport::Uncovered {
                    file,
                    item_line,
                    line_no,
                    ..
                } => report.failures.push(format!(
                    "{}:{}: spec-claimed pub item outside CODEGEN/HANDWRITE: {}",
                    file.strip_prefix(cwd).unwrap_or(&file).display(),
                    line_no,
                    item_line
                )),
                UnifiedReport::Unresolvable {
                    file,
                    spec_ref,
                    reason,
                } => report.failures.push(format!(
                    "{}: unresolvable SPEC-MANAGED ref `{}` ({})",
                    file.strip_prefix(cwd).unwrap_or(&file).display(),
                    spec_ref,
                    reason
                )),
            }
        }
    }

    report.enforce_complete_source_ownership_coverage();
    report.enforce_complete_public_api_semantic_conformance();
    Ok(report)
}

fn has_handwrite_ownership_marker(content: &str) -> bool {
    content.lines().any(|line| {
        let trimmed = line.trim();
        let body = trimmed
            .strip_prefix("// ")
            .or_else(|| trimmed.strip_prefix("# "))
            .or_else(|| trimmed.strip_prefix("<!-- "))
            .map(|body| body.strip_suffix(" -->").unwrap_or(body))
            .unwrap_or(trimmed);
        body.starts_with("HANDWRITE-BEGIN") || body.starts_with("<HANDWRITE")
    })
}

fn collect_source_scope_files(scope: &ForceRegenScope) -> Result<Vec<std::path::PathBuf>> {
    let mut files = Vec::new();
    let mut visited_dirs = BTreeSet::new();
    for root in &scope.source_roots {
        collect_source_scope_files_inner(root, &mut files, &mut visited_dirs)?;
    }
    files.sort();
    files.dedup();
    Ok(files)
}

fn collect_source_scope_files_inner(
    path: &std::path::Path,
    out: &mut Vec<std::path::PathBuf>,
    visited_dirs: &mut BTreeSet<std::path::PathBuf>,
) -> Result<()> {
    if !path.exists() {
        return Ok(());
    }
    let metadata = std::fs::symlink_metadata(path)?;
    let file_type = metadata.file_type();
    if file_type.is_symlink() {
        return Ok(());
    }
    if file_type.is_file() {
        out.push(path.to_path_buf());
        return Ok(());
    }
    if !file_type.is_dir() {
        return Ok(());
    }
    let canonical = std::fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf());
    if !visited_dirs.insert(canonical) {
        return Ok(());
    }
    for entry in std::fs::read_dir(path)? {
        let entry = entry?;
        let child = entry.path();
        let file_type = entry.file_type()?;
        if file_type.is_symlink() {
            continue;
        }
        if file_type.is_dir() {
            if should_skip_force_regen_scan_dir(&child) {
                continue;
            }
            collect_source_scope_files_inner(&child, out, visited_dirs)?;
        } else if file_type.is_file() {
            out.push(child);
        }
    }
    Ok(())
}

fn is_supported_source_file(path: &std::path::Path) -> bool {
    let Some(ext) = path.extension().and_then(|e| e.to_str()) else {
        return false;
    };
    crate::fillback::SupportedLanguage::from_extension(ext).is_some()
}

fn is_minified_asset_file(path: &std::path::Path) -> bool {
    let Some(file_name) = path.file_name().and_then(|name| name.to_str()) else {
        return false;
    };
    file_name.ends_with(".min.js")
        && path
            .components()
            .any(|component| component.as_os_str() == "assets")
}

#[derive(Debug, Clone)]
struct SpecRef {
    raw: String,
    path: String,
    section: Option<String>,
}

fn parse_spec_ref(raw: &str) -> Option<SpecRef> {
    let raw = raw.trim();
    if raw.is_empty() {
        return None;
    }
    let (path, section) = match raw.split_once('#') {
        Some((path, section)) => (path.trim(), Some(section.trim().to_string())),
        None => (raw, None),
    };
    if path.is_empty() {
        return None;
    }
    Some(SpecRef {
        raw: raw.to_string(),
        path: path.to_string(),
        section: section.filter(|s| !s.is_empty()),
    })
}

fn validate_spec_ref(
    cwd: &std::path::Path,
    td_root: &std::path::Path,
    spec_ref: &SpecRef,
    role: &str,
    rel_path: &std::path::Path,
    report: &mut ForceRegenConformanceReport,
) -> bool {
    if spec_ref.path == "generated" || spec_ref.path.starts_with("generated/") {
        return true;
    }
    let spec_path = cwd.join(&spec_ref.path);
    if !spec_path.starts_with(td_root) {
        report.failures.push(format!(
            "{}: {} ref `{}` is outside project td_path {}",
            rel_path.display(),
            role,
            spec_ref.raw,
            td_root.strip_prefix(cwd).unwrap_or(td_root).display()
        ));
        return false;
    }
    if !spec_path.exists() {
        report.failures.push(format!(
            "{}: {} ref `{}` points at a missing TD spec",
            rel_path.display(),
            role,
            spec_ref.raw
        ));
        return false;
    }
    true
}

fn classify_source_template(
    cwd: &std::path::Path,
    owner_ref: &SpecRef,
    rel_path: &std::path::Path,
    symbols: Option<&[crate::fillback::ast::Symbol]>,
    report: &mut ForceRegenConformanceReport,
) -> Result<()> {
    let spec_path = cwd.join(&owner_ref.path);
    if !spec_path.exists() {
        return Ok(());
    }
    let spec_content = std::fs::read_to_string(&spec_path)
        .with_context(|| format!("failed to read {}", spec_path.display()))?;
    if !spec_declares_source_section(&spec_content) {
        report.failures.push(format!(
            "{}: owner TD `{}` does not declare a source section",
            rel_path.display(),
            owner_ref.path
        ));
        return Ok(());
    }
    report.source_templates += 1;
    let semantic_coverage = symbols
        .map(|symbols| td_public_symbol_semantic_coverage(&spec_content, symbols))
        .transpose()?;
    if let Some(coverage) = &semantic_coverage {
        report.public_symbols += coverage.total_public_symbols;
        report.td_semantic_public_symbols += coverage.covered_public_symbols;
        if coverage.total_public_symbols == 0 {
            report.source_templates_public_api_empty += 1;
        } else if coverage.missing_public_symbols.is_empty() {
            report.source_templates_public_api_complete += 1;
        } else {
            report.source_templates_public_api_partial += 1;
        }
    }
    if spec_content.contains("source-from-target") {
        let reason = semantic_coverage
            .as_ref()
            .map(|coverage| coverage.review_reason())
            .unwrap_or_else(|| "source-from-target; AST semantic coverage unavailable".to_string());
        report
            .target_derived_source_templates
            .push(SemanticReviewUnit {
                spec_ref: owner_ref.raw.clone(),
                target_path: rel_path.to_path_buf(),
                reason,
            });
    } else {
        report.deterministic_source_templates += 1;
    }
    Ok(())
}

fn spec_declares_source_section(spec_content: &str) -> bool {
    spec_content.contains("<!-- type: source")
        || spec_content.contains("<!-- type: rust-source-unit")
        || spec_content.contains("<!-- type: text-source-unit")
        || spec_content.contains("section: source")
        || spec_content.contains("section: rust-source-unit")
        || spec_content.contains("section: text-source-unit")
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct PublicSymbolSemanticCoverage {
    total_public_symbols: usize,
    covered_public_symbols: usize,
    missing_public_symbols: Vec<String>,
}

// @spec projects/agentic-workflow/tech-design/surface/interfaces/src/cb.md#source
impl PublicSymbolSemanticCoverage {
    fn review_reason(&self) -> String {
        if self.total_public_symbols == 0 {
            return "source-from-target; no public AST symbols".to_string();
        }
        if self.missing_public_symbols.is_empty() {
            return format!(
                "source-from-target; public-api-semantic {}/{}",
                self.covered_public_symbols, self.total_public_symbols
            );
        }
        let mut missing = self.missing_public_symbols.clone();
        missing.sort();
        missing.truncate(4);
        format!(
            "source-from-target; public-api-semantic {}/{}; missing {}",
            self.covered_public_symbols,
            self.total_public_symbols,
            missing.join(", ")
        )
    }
}

fn td_public_symbol_semantic_coverage(
    spec_content: &str,
    symbols: &[crate::fillback::ast::Symbol],
) -> Result<PublicSymbolSemanticCoverage> {
    let td_symbols = td_structured_symbol_names(spec_content)?;
    let mut public_symbols = symbols
        .iter()
        .filter(|symbol| symbol.is_public)
        .map(|symbol| symbol.name.clone())
        .collect::<Vec<_>>();
    public_symbols.sort();
    public_symbols.dedup();

    let missing_public_symbols = public_symbols
        .iter()
        .filter(|symbol| !td_symbols.contains(*symbol))
        .cloned()
        .collect::<Vec<_>>();

    Ok(PublicSymbolSemanticCoverage {
        total_public_symbols: public_symbols.len(),
        covered_public_symbols: public_symbols.len() - missing_public_symbols.len(),
        missing_public_symbols,
    })
}

fn td_structured_symbol_names(spec_content: &str) -> Result<BTreeSet<String>> {
    let mut symbols = BTreeSet::new();
    let doc = crate::spec_alignment::parser::parse("(inline)", spec_content);
    for section in doc.sections {
        let section_type = section
            .annotation
            .as_ref()
            .map(|annotation| annotation.section_type.as_str())
            .unwrap_or("");
        if section_type.eq_ignore_ascii_case("schema")
            || section.heading.eq_ignore_ascii_case("Schema")
        {
            for block in &section.code_blocks {
                if block.lang.eq_ignore_ascii_case("yaml")
                    || block.lang.eq_ignore_ascii_case("json")
                {
                    let value = serde_yaml::from_str::<serde_yaml::Value>(&block.content)
                        .with_context(|| {
                            format!(
                                "failed to parse schema block in TD section {}",
                                section.heading
                            )
                        })?;
                    collect_schema_symbol_names(&value, &mut symbols);
                }
            }
        }
        if section_type.eq_ignore_ascii_case("logic")
            || section.heading.to_ascii_lowercase().starts_with("logic")
        {
            if let Some(name) = section
                .heading
                .strip_prefix("Logic:")
                .or_else(|| section.heading.strip_prefix("Logic -"))
            {
                let name = name.trim();
                if !name.is_empty() {
                    symbols.insert(name.to_string());
                }
            }
            for block in &section.code_blocks {
                if block.lang.eq_ignore_ascii_case("yaml")
                    || block.lang.eq_ignore_ascii_case("json")
                {
                    let value = serde_yaml::from_str::<serde_yaml::Value>(&block.content)
                        .with_context(|| {
                            format!(
                                "failed to parse logic block in TD section {}",
                                section.heading
                            )
                        })?;
                    collect_logic_symbol_names(&value, &mut symbols);
                }
            }
        }
        if section_type.eq_ignore_ascii_case("overview")
            || section.heading.eq_ignore_ascii_case("Overview")
        {
            collect_markdown_symbol_names(&section.body, &mut symbols);
            for block in &section.code_blocks {
                if block.lang.eq_ignore_ascii_case("rust") {
                    collect_rust_signature_symbol_names(&block.content, &mut symbols);
                }
            }
        }
    }
    Ok(symbols)
}

fn collect_schema_symbol_names(value: &serde_yaml::Value, out: &mut BTreeSet<String>) {
    let Some(mapping) = value.as_mapping() else {
        return;
    };
    for key in ["definitions", "schemas"] {
        let Some(defs) = mapping
            .get(serde_yaml::Value::String(key.to_string()))
            .and_then(|v| v.as_mapping())
        else {
            continue;
        };
        for name in defs.keys().filter_map(|k| k.as_str()) {
            out.insert(name.to_string());
        }
    }
    if let Some(title) = mapping
        .get(serde_yaml::Value::String("title".to_string()))
        .and_then(|v| v.as_str())
    {
        out.insert(title.to_string());
    }
}

fn collect_logic_symbol_names(value: &serde_yaml::Value, out: &mut BTreeSet<String>) {
    let Some(mapping) = value.as_mapping() else {
        return;
    };
    for key in ["id", "title"] {
        if let Some(name) = mapping
            .get(serde_yaml::Value::String(key.to_string()))
            .and_then(|v| v.as_str())
            .map(str::trim)
            .filter(|name| !name.is_empty())
        {
            out.insert(name.to_string());
        }
    }
}

fn collect_markdown_symbol_names(body: &str, out: &mut BTreeSet<String>) {
    let mut in_symbols_table = false;
    for line in body.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("### ") {
            in_symbols_table = trimmed.eq_ignore_ascii_case("### Symbols");
            continue;
        }
        if in_symbols_table && trimmed.starts_with('|') {
            let first_cell = trimmed
                .trim_matches('|')
                .split('|')
                .next()
                .map(str::trim)
                .unwrap_or("");
            if let Some(name) = backticked_name(first_cell) {
                out.insert(name.to_string());
            }
            continue;
        }
        if trimmed.starts_with("- `") {
            if let Some(name) = backticked_name(trimmed.trim_start_matches("- ")) {
                out.insert(name.to_string());
            }
        }
    }
}

fn collect_rust_signature_symbol_names(content: &str, out: &mut BTreeSet<String>) {
    for line in content.lines() {
        let trimmed = line.trim();
        for prefix in [
            "pub fn ",
            "pub struct ",
            "pub enum ",
            "pub type ",
            "pub const ",
        ] {
            if let Some(rest) = trimmed.strip_prefix(prefix) {
                if let Some(name) = rest
                    .split(|c: char| !(c.is_ascii_alphanumeric() || c == '_'))
                    .find(|segment| !segment.is_empty())
                {
                    out.insert(name.to_string());
                }
            }
        }
    }
}

fn backticked_name(value: &str) -> Option<&str> {
    let value = value.trim();
    let rest = value.strip_prefix('`')?;
    let (name, _) = rest.split_once('`')?;
    if name.is_empty() {
        return None;
    }
    Some(name)
}

#[cfg(test)]
fn upsert_public_api_overview(
    spec_content: &str,
    target_rel_path: &str,
    symbols: &[crate::fillback::ast::Symbol],
) -> String {
    let target = public_api_manifest_target(target_rel_path, symbols);
    upsert_public_api_overview_targets(spec_content, &[target])
}

fn upsert_public_api_overview_targets(
    spec_content: &str,
    targets: &[PublicApiManifestTarget],
) -> String {
    let overview = render_public_api_overview(targets);
    let with_fill_sections = ensure_fill_sections_has_overview(spec_content);
    replace_or_insert_h2_section(&with_fill_sections, "Overview", &overview)
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct PublicApiManifestTarget {
    target_rel_path: String,
    symbols: Vec<PublicApiManifestSymbol>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct PublicApiManifestSymbol {
    name: String,
    kind: String,
    line: usize,
    signature: Option<String>,
}

fn public_api_manifest_target(
    target_rel_path: &str,
    symbols: &[crate::fillback::ast::Symbol],
) -> PublicApiManifestTarget {
    let mut public_symbols = symbols
        .iter()
        .filter(|symbol| symbol.is_public)
        .collect::<Vec<_>>();
    public_symbols.sort_by(|a, b| a.name.cmp(&b.name).then(a.line.cmp(&b.line)));
    PublicApiManifestTarget {
        target_rel_path: target_rel_path.to_string(),
        symbols: public_symbols
            .into_iter()
            .map(|symbol| PublicApiManifestSymbol {
                name: symbol.name.clone(),
                kind: symbol.kind.to_string(),
                line: symbol.line,
                signature: symbol.signature.clone(),
            })
            .collect(),
    }
}

fn render_public_api_overview(targets: &[PublicApiManifestTarget]) -> String {
    let mut out = String::new();
    out.push_str("## Overview\n");
    out.push_str("<!-- type: overview lang: markdown -->\n\n");
    if let [target] = targets {
        out.push_str(&format!(
            "Public API manifest for `{}` generated from AST during Score force-regeneration standardization.\n\n",
            escape_markdown_cell(&target.target_rel_path)
        ));
    } else {
        out.push_str(&format!(
            "Public API manifest for {} target files generated from AST during Score force-regeneration standardization.\n\n",
            targets.len()
        ));
    }
    out.push_str("### Symbols\n\n");
    if targets.iter().all(|target| target.symbols.is_empty()) {
        out.push_str("No public AST symbols.\n");
        return out;
    }
    out.push_str("| Name | Target | Kind | Visibility | Line | Signature |\n");
    out.push_str("|------|--------|------|------------|------|-----------|\n");
    for target in targets {
        for symbol in &target.symbols {
            let signature = symbol
                .signature
                .as_deref()
                .map(escape_markdown_cell)
                .unwrap_or_default();
            out.push_str(&format!(
                "| `{}` | {} | {} | pub | {} | {} |\n",
                escape_markdown_cell(&symbol.name),
                escape_markdown_cell(&target.target_rel_path),
                escape_markdown_cell(&symbol.kind),
                symbol.line,
                signature,
            ));
        }
    }
    out
}

fn ensure_fill_sections_has_overview(content: &str) -> String {
    let mut lines = content.lines().map(str::to_string).collect::<Vec<_>>();
    for line in &mut lines {
        let trimmed = line.trim_start();
        let indent_len = line.len() - trimmed.len();
        let indent = &line[..indent_len];
        let Some(rest) = trimmed.strip_prefix("fill_sections:") else {
            continue;
        };
        let rest = rest.trim();
        let Some(inner) = rest.strip_prefix('[').and_then(|s| s.strip_suffix(']')) else {
            return content.to_string();
        };
        let sections = inner
            .split(',')
            .map(str::trim)
            .filter(|section| !section.is_empty())
            .collect::<Vec<_>>();
        if sections.iter().any(|section| *section == "overview") {
            return content.to_string();
        }
        let mut updated = vec!["overview".to_string()];
        updated.extend(sections.into_iter().map(str::to_string));
        *line = format!("{indent}fill_sections: [{}]", updated.join(", "));
        return lines.join("\n") + trailing_newline(content);
    }
    content.to_string()
}

fn replace_or_insert_h2_section(content: &str, heading: &str, section: &str) -> String {
    let lines = content.lines().collect::<Vec<_>>();
    let wanted = format!("## {heading}");
    let mut h2_positions = Vec::new();
    let mut fence: Option<(char, usize)> = None;
    for (idx, line) in lines.iter().enumerate() {
        let trimmed_start = line.trim_start();
        if let Some((fence_char, fence_len)) = fence {
            let marker_len = trimmed_start
                .chars()
                .take_while(|ch| *ch == fence_char)
                .count();
            if marker_len >= fence_len && trimmed_start[marker_len..].trim().is_empty() {
                fence = None;
            }
            continue;
        }

        let Some(first) = trimmed_start.chars().next() else {
            continue;
        };
        if first == '`' || first == '~' {
            let marker_len = trimmed_start.chars().take_while(|ch| *ch == first).count();
            if marker_len >= 3 {
                fence = Some((first, marker_len));
                continue;
            }
        }

        let trimmed = line.trim();
        if trimmed.starts_with("## ") && !trimmed.starts_with("### ") {
            h2_positions.push(idx);
        }
    }

    if let Some((pos_idx, start)) = h2_positions
        .iter()
        .enumerate()
        .find(|(_, idx)| lines[**idx].trim() == wanted)
    {
        let end = h2_positions
            .get(pos_idx + 1)
            .copied()
            .unwrap_or(lines.len());
        let mut out = Vec::new();
        out.extend_from_slice(&lines[..*start]);
        out.push(section.trim_end());
        out.extend_from_slice(&lines[end..]);
        return out.join("\n") + trailing_newline(content);
    }

    let insert_at = h2_positions.first().copied().unwrap_or(lines.len());
    let mut out = Vec::new();
    out.extend_from_slice(&lines[..insert_at]);
    if out.last().is_some_and(|line| !line.trim().is_empty()) {
        out.push("");
    }
    out.push(section.trim_end());
    if insert_at < lines.len() {
        out.push("");
        out.extend_from_slice(&lines[insert_at..]);
    }
    out.join("\n") + trailing_newline(content)
}

fn escape_markdown_cell(value: &str) -> String {
    value
        .replace('\n', " ")
        .replace('\r', " ")
        .replace('|', "\\|")
}

fn trailing_newline(content: &str) -> &'static str {
    if content.ends_with('\n') {
        "\n"
    } else {
        ""
    }
}

fn sample_semantic_review_units<'a>(
    units: &[&'a SemanticReviewUnit],
    ratio: f64,
) -> Result<Vec<&'a SemanticReviewUnit>> {
    if !(ratio.is_finite() && ratio > 0.0 && ratio <= 1.0) {
        anyhow::bail!("--semantic-sample must be > 0 and <= 1");
    }
    let count = sample_count(units.len(), ratio);
    let mut sorted = units.to_vec();
    sorted.sort_by_key(|unit| stable_sample_hash(&unit.spec_ref));
    sorted.truncate(count);
    sorted.sort_by(|a, b| a.spec_ref.cmp(&b.spec_ref));
    Ok(sorted)
}

fn sample_count(total: usize, ratio: f64) -> usize {
    if total == 0 {
        return 0;
    }
    ((total as f64 * ratio).ceil() as usize).clamp(1, total)
}

fn stable_sample_hash(value: &str) -> u64 {
    let mut hash = 0xcbf29ce484222325u64;
    for byte in value.as_bytes() {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    hash
}

fn collect_tree_files(
    root: &std::path::Path,
) -> Result<std::collections::BTreeMap<std::path::PathBuf, std::path::PathBuf>> {
    let mut files = std::collections::BTreeMap::new();
    let mut visited_dirs = BTreeSet::new();
    collect_tree_files_inner(root, root, &mut files, &mut visited_dirs)?;
    Ok(files)
}

fn collect_tree_files_inner(
    root: &std::path::Path,
    dir: &std::path::Path,
    files: &mut std::collections::BTreeMap<std::path::PathBuf, std::path::PathBuf>,
    visited_dirs: &mut BTreeSet<std::path::PathBuf>,
) -> Result<()> {
    if !dir.exists() {
        return Ok(());
    }
    let metadata = std::fs::symlink_metadata(dir)?;
    let file_type = metadata.file_type();
    if file_type.is_symlink() {
        return Ok(());
    }
    if file_type.is_file() {
        let rel = dir.strip_prefix(root)?.to_path_buf();
        files.insert(rel, dir.to_path_buf());
        return Ok(());
    }
    if !file_type.is_dir() {
        return Ok(());
    }
    let canonical = std::fs::canonicalize(dir).unwrap_or_else(|_| dir.to_path_buf());
    if !visited_dirs.insert(canonical) {
        return Ok(());
    }
    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        let file_type = entry.file_type()?;
        if file_type.is_symlink() {
            continue;
        }
        if file_type.is_dir() {
            if should_skip_force_regen_scan_dir(&path) {
                continue;
            }
            collect_tree_files_inner(root, &path, files, visited_dirs)?;
        } else if file_type.is_file() {
            let rel = path.strip_prefix(root)?.to_path_buf();
            files.insert(rel, path);
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{
        cb_verify_summary_from_report, claim_issue_create_args, claim_issue_title,
        classify_codegen_origin_spec, collect_force_regen_specs, collect_source_scope_files,
        collect_tree_files, commit_cb_claim_trailer, commit_force_regen, compare_source_roots,
        copy_tree, ensure_claim_issue, extract_cold_rebuild_target_paths,
        extract_project_root_llms_target_paths, extract_spec_managed_ref,
        extract_spec_managed_refs, format_rust_files, has_handwrite_ownership_marker,
        is_minified_asset_file, repo_relative_code_path, resolve_project_force_regen_scope,
        run_force_regen_specs, sample_count, sample_semantic_review_units,
        spec_declares_source_section, td_public_symbol_semantic_coverage,
        upsert_public_api_overview, upsert_public_api_overview_targets,
        verify_force_regen_conformance, write_project_root_llms_targets, CbCodegenOriginClass,
        CbCommand, CbGenArgs, ClaimIssueRef, ForceRegenConformanceReport, ForceRegenScope,
        PublicApiManifestSymbol, PublicApiManifestTarget, PublicSymbolSemanticCoverage,
        SemanticReviewUnit,
    };
    use crate::fillback::ast::{Symbol, SymbolKind};
    use clap::Parser;

    #[derive(Debug, Parser)]
    struct TestCbCli {
        #[command(subcommand)]
        command: CbCommand,
    }

    fn git_available() -> bool {
        std::process::Command::new("git")
            .arg("--version")
            .output()
            .map(|out| out.status.success())
            .unwrap_or(false)
    }

    #[test]
    fn codegen_origin_spec_classifies_mixed_routes() {
        let td_ast = "## Changes\n<!-- type: changes lang: yaml -->\n```yaml\nchanges: []\n```";
        assert_eq!(
            classify_codegen_origin_spec(td_ast),
            CbCodegenOriginClass::TdAst
        );

        let source_template =
            "## Source\n<!-- type: source lang: rust -->\n```rust\npub fn demo() {}\n```";
        assert_eq!(
            classify_codegen_origin_spec(source_template),
            CbCodegenOriginClass::SourceTemplate
        );

        let rust_source_unit =
            "## Source\n<!-- type: rust-source-unit lang: rust -->\n```rust\npub fn demo() {}\n```";
        assert_eq!(
            classify_codegen_origin_spec(rust_source_unit),
            CbCodegenOriginClass::TdAst
        );

        let text_source_unit = "## Source\n<!-- type: text-source-unit lang: bash -->\n```bash\n#!/usr/bin/env bash\n```\n";
        assert_eq!(
            classify_codegen_origin_spec(text_source_unit),
            CbCodegenOriginClass::TdAst
        );

        let artifact_replay = "## Source\n<!-- type: source lang: rust -->\n<!-- source-from-target: strip-managed-markers -->";
        assert_eq!(
            classify_codegen_origin_spec(artifact_replay),
            CbCodegenOriginClass::ArtifactReplay
        );
    }

    #[test]
    fn spec_declares_text_source_unit_as_source_section() {
        let text_source_unit = "\
## Source
<!-- type: text-source-unit lang: javascript -->

```javascript
console.log('ok');
```
";

        assert!(spec_declares_source_section(text_source_unit));
    }

    fn init_git_repo(root: &std::path::Path) {
        for args in [
            vec!["init", "-q", "-b", "main"],
            vec!["config", "user.email", "test@example.com"],
            vec!["config", "user.name", "Test"],
            vec!["commit", "--allow-empty", "-m", "init", "-q"],
        ] {
            let out = std::process::Command::new("git")
                .args(&args)
                .current_dir(root)
                .output()
                .expect("git command");
            assert!(
                out.status.success(),
                "git {:?} failed: {}",
                args,
                String::from_utf8_lossy(&out.stderr)
            );
        }
    }

    fn git_stdout(root: &std::path::Path, args: &[&str]) -> String {
        let out = std::process::Command::new("git")
            .args(args)
            .current_dir(root)
            .output()
            .expect("git command");
        assert!(
            out.status.success(),
            "git {:?} failed: {}",
            args,
            String::from_utf8_lossy(&out.stderr)
        );
        String::from_utf8_lossy(&out.stdout).trim().to_string()
    }

    #[test]
    fn cb_gen_force_regen_parses_without_slug() {
        let parsed = TestCbCli::try_parse_from([
            "agentic-workflow",
            "gen",
            "--force-regen",
            "--project",
            "agentic-workflow",
            "--dry-run",
        ])
        .unwrap();
        match parsed.command {
            CbCommand::Gen(CbGenArgs {
                slug,
                force_regen,
                project,
                dry_run,
                ..
            }) => {
                assert!(slug.is_none());
                assert!(force_regen);
                assert_eq!(project.as_deref(), Some("agentic-workflow"));
                assert!(dry_run);
            }
            _ => panic!("expected cb gen"),
        }
    }

    #[test]
    fn cb_gen_force_regen_verify_parses_without_slug() {
        let parsed = TestCbCli::try_parse_from([
            "agentic-workflow",
            "gen",
            "--force-regen",
            "--project",
            "agentic-workflow",
            "--verify",
        ])
        .unwrap();
        match parsed.command {
            CbCommand::Gen(CbGenArgs {
                slug,
                force_regen,
                project,
                verify,
                ..
            }) => {
                assert!(slug.is_none());
                assert!(force_regen);
                assert_eq!(project.as_deref(), Some("agentic-workflow"));
                assert!(verify);
            }
            _ => panic!("expected cb gen"),
        }
    }

    #[test]
    fn cb_gen_force_regen_verify_semantic_sample_parses_without_slug() {
        let parsed = TestCbCli::try_parse_from([
            "agentic-workflow",
            "gen",
            "--force-regen",
            "--project",
            "agentic-workflow",
            "--verify",
            "--semantic-sample",
            "0.15",
        ])
        .unwrap();
        match parsed.command {
            CbCommand::Gen(CbGenArgs {
                slug,
                force_regen,
                project,
                verify,
                semantic_sample,
                ..
            }) => {
                assert!(slug.is_none());
                assert!(force_regen);
                assert_eq!(project.as_deref(), Some("agentic-workflow"));
                assert!(verify);
                assert_eq!(semantic_sample, Some(0.15));
            }
            _ => panic!("expected cb gen"),
        }
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    struct CompletenessFinding {
        code: &'static str,
        line: usize,
        pattern: &'static str,
    }

    fn scan_completeness_placeholders(content: &str) -> Vec<CompletenessFinding> {
        const ALLOWED_MARKERS: [&str; 3] =
            ["HANDWRITE-BEGIN", "generator-gap", "future_work_allowed"];
        const HARD_PATTERNS: [(&str, &str); 6] = [
            ("todo implement", "placeholder_artifact"),
            ("todo: implement", "placeholder_artifact"),
            ("rest omitted", "omitted_content"),
            ("similar pattern omitted", "omitted_content"),
            ("omitted for brevity", "omitted_content"),
            ("...", "omitted_content"),
        ];

        let mut findings = Vec::new();
        for (idx, line) in content.lines().enumerate() {
            if ALLOWED_MARKERS.iter().any(|marker| line.contains(marker)) {
                continue;
            }
            let lowered = line.to_ascii_lowercase();
            for (pattern, code) in HARD_PATTERNS {
                if lowered.contains(pattern) {
                    findings.push(CompletenessFinding {
                        code,
                        line: idx + 1,
                        pattern,
                    });
                    break;
                }
            }
        }
        findings
    }

    #[test]
    fn completeness_placeholder_scanner_contract() {
        let findings = scan_completeness_placeholders(
            r#"
fn generated() {
    // TODO implement text
    // similar pattern omitted from generated prose
    // future_work_allowed TODO implement this after generator primitive lands
}
"#,
        );

        assert_eq!(
            findings,
            vec![
                CompletenessFinding {
                    code: "placeholder_artifact",
                    line: 3,
                    pattern: "todo implement",
                },
                CompletenessFinding {
                    code: "omitted_content",
                    line: 4,
                    pattern: "similar pattern omitted",
                },
            ]
        );
    }

    #[test]
    fn cb_gen_force_regen_sync_public_api_parses_without_slug() {
        let parsed = TestCbCli::try_parse_from([
            "agentic-workflow",
            "gen",
            "--force-regen",
            "--project",
            "agentic-workflow",
            "--sync-public-api",
        ])
        .unwrap();
        match parsed.command {
            CbCommand::Gen(CbGenArgs {
                slug,
                force_regen,
                project,
                sync_public_api,
                ..
            }) => {
                assert!(slug.is_none());
                assert!(force_regen);
                assert_eq!(project.as_deref(), Some("agentic-workflow"));
                assert!(sync_public_api);
            }
            _ => panic!("expected cb gen"),
        }
    }

    #[test]
    fn cb_gen_force_regen_workspace_parses_without_slug() {
        let parsed = TestCbCli::try_parse_from([
            "agentic-workflow",
            "gen",
            "--force-regen",
            "--project",
            "fixture_platform",
            "--workspace",
            "fixture_platform-backend",
            "--verify",
        ])
        .unwrap();
        match parsed.command {
            CbCommand::Gen(CbGenArgs {
                slug,
                force_regen,
                project,
                workspace,
                verify,
                ..
            }) => {
                assert!(slug.is_none());
                assert!(force_regen);
                assert_eq!(project.as_deref(), Some("fixture_platform"));
                assert_eq!(workspace.as_deref(), Some("fixture_platform-backend"));
                assert!(verify);
            }
            _ => panic!("expected cb gen"),
        }
    }

    #[test]
    fn cb_gen_force_regen_workspace_narrows_source_roots() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        std::fs::create_dir_all(root.join(".aw")).unwrap();
        std::fs::write(
            root.join(".aw/config.toml"),
            r#"
[[projects]]
name = "fixture_platform"
path = "examples/fixture_platform"
td_path = "examples/fixture_platform/tech_design"

[[projects.workspaces]]
name = "fixture_platform-backend"
paths = ["examples/fixture_platform/backend/**"]

[[projects.workspaces]]
name = "fixture_platform-frontend"
paths = ["examples/fixture_platform/frontend/**"]
"#,
        )
        .unwrap();

        let scope = resolve_project_force_regen_scope(
            root,
            "fixture_platform",
            Some("fixture_platform-backend"),
        )
        .unwrap();

        assert_eq!(
            scope.td_root,
            root.join("examples/fixture_platform/tech_design")
        );
        assert_eq!(
            scope.source_roots,
            vec![root.join("examples/fixture_platform/backend")]
        );
    }

    #[test]
    fn cb_gen_force_regen_defaults_td_root_to_project_tech_design() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        std::fs::create_dir_all(root.join(".aw")).unwrap();
        std::fs::write(
            root.join(".aw/config.toml"),
            r#"
[[projects]]
name = "fixture_platform"
aliases = ["fp"]
path = "examples/fixture_platform"

[[projects.workspaces]]
name = "fixture_platform-backend"
paths = ["examples/fixture_platform/backend/**"]
"#,
        )
        .unwrap();

        let scope = resolve_project_force_regen_scope(root, "fp", Some("fixture_platform-backend"))
            .unwrap();

        assert_eq!(
            scope.td_root,
            root.join("examples/fixture_platform/tech-design")
        );
        assert_eq!(
            scope.source_roots,
            vec![root.join("examples/fixture_platform/backend")]
        );
    }

    #[test]
    fn cb_force_regen_commit_records_lifecycle_trailers() {
        if !git_available() {
            return;
        }
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        init_git_repo(root);
        let source = root.join("projects/agentic-workflow/src/lib.rs");
        std::fs::create_dir_all(source.parent().unwrap()).unwrap();
        std::fs::write(&source, "pub fn demo() {}\n").unwrap();

        commit_force_regen(
            root,
            "agentic-workflow",
            Some("agentic-workflow"),
            2,
            1,
            0,
            3,
            1,
            &[source],
        )
        .unwrap();

        let log = git_stdout(root, &["log", "-1", "--pretty=%B"]);
        assert!(log.contains("Lifecycle-Stage: Cb-Force-Regen"));
        assert!(log.contains("Project: agentic-workflow"));
        assert!(log.contains("Workspace: agentic-workflow"));
        assert!(log.contains("Specs: 2"));
        assert!(log.contains("Blocks-Updated: 3"));
    }

    #[test]
    fn cb_gen_force_regen_collects_current_spec_managed_refs_only() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        let td_root = root.join("projects/agentic-workflow/tech-design/surface");
        let source_root = root.join("projects/agentic-workflow");
        std::fs::create_dir_all(td_root.join("interfaces/src")).unwrap();
        std::fs::create_dir_all(td_root.join("specs")).unwrap();
        std::fs::create_dir_all(source_root.join("src")).unwrap();
        std::fs::create_dir_all(source_root.join("tests")).unwrap();
        std::fs::write(td_root.join("interfaces/src/lib.md"), "# canonical\n").unwrap();
        std::fs::write(td_root.join("interfaces/src/schema.md"), "# schema\n").unwrap();
        std::fs::write(
            td_root.join("specs/external-contracts.md"),
            "# external contracts\n",
        )
        .unwrap();
        std::fs::write(td_root.join("specs/old-redesign.md"), "# historical\n").unwrap();
        std::fs::write(
            source_root.join("src/lib.rs"),
            "// SPEC-MANAGED: projects/agentic-workflow/tech-design/surface/interfaces/src/lib.md#source\n// CODEGEN-BEGIN\n// CODEGEN-END\n// SPEC-MANAGED: projects/agentic-workflow/tech-design/surface/interfaces/src/schema.md#schema\n// CODEGEN-BEGIN\n// CODEGEN-END\n",
        )
        .unwrap();
        std::fs::write(
            source_root.join("tests/behavior_ec.rs"),
            "// SPEC-MANAGED: projects/agentic-workflow/tech-design/surface/specs/external-contracts.md#demo-contract\n// CODEGEN-BEGIN\n// AW-EC-BEGIN\n// @ec demo-contract\n// AW-EC-END\n// CODEGEN-END\n",
        )
        .unwrap();

        let scope = ForceRegenScope {
            td_root: td_root.clone(),
            source_roots: vec![source_root],
        };
        let mut specs = Vec::new();
        collect_force_regen_specs(root, &scope, &mut specs).unwrap();

        assert_eq!(
            specs,
            vec![
                td_root.join("interfaces/src/lib.md"),
                td_root.join("interfaces/src/schema.md")
            ]
        );
    }

    #[test]
    fn cb_verify_skips_aw_ec_generated_wrappers_outside_td_path() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        let td_root = root.join("projects/demo/tech-design");
        let source_root = root.join("projects/demo");
        std::fs::create_dir_all(&td_root).unwrap();
        std::fs::create_dir_all(source_root.join("tests")).unwrap();
        std::fs::write(td_root.join("app.md"), "# app\n").unwrap();
        std::fs::write(
            source_root.join("tests/behavior_demo_contract.rs"),
            "// SPEC-MANAGED: projects/demo/external-contracts/behavior/demo.md#demo-contract\n// CODEGEN-BEGIN\n// AW-EC-BEGIN\n// @ec demo-contract\n// AW-EC-END\n#[test]\n#[ignore = \"generated EC wrapper\"]\nfn demo_contract() {}\n// CODEGEN-END\n",
        )
        .unwrap();

        let scope = ForceRegenScope {
            td_root,
            source_roots: vec![source_root],
        };
        let report = verify_force_regen_conformance(root, &scope).unwrap();

        assert!(
            report
                .failures
                .iter()
                .all(|failure| !failure.contains("outside project td_path")),
            "unexpected failures: {:?}",
            report.failures
        );
    }

    #[test]
    fn cb_gen_force_regen_skips_dependency_dirs_under_project_root() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        let td_root = root.join("tech-design");
        let source_root = root.to_path_buf();
        std::fs::create_dir_all(&td_root).unwrap();
        std::fs::create_dir_all(root.join("src")).unwrap();
        std::fs::create_dir_all(root.join("node_modules/pkg")).unwrap();
        std::fs::write(td_root.join("app.md"), "# app\n").unwrap();
        std::fs::write(td_root.join("dependency.md"), "# dependency\n").unwrap();
        std::fs::write(
            root.join("src/app.ts"),
            "// SPEC-MANAGED: tech-design/app.md#source\n// CODEGEN-BEGIN\n// CODEGEN-END\n",
        )
        .unwrap();
        std::fs::write(
            root.join("node_modules/pkg/index.ts"),
            "// SPEC-MANAGED: tech-design/dependency.md#source\n// CODEGEN-BEGIN\n// CODEGEN-END\n",
        )
        .unwrap();

        let scope = ForceRegenScope {
            td_root: td_root.clone(),
            source_roots: vec![source_root],
        };
        let mut specs = Vec::new();
        collect_force_regen_specs(root, &scope, &mut specs).unwrap();

        assert_eq!(specs, vec![td_root.join("app.md")]);
    }

    #[test]
    fn cb_gen_force_regen_collects_spec_refs_from_file_source_root() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        let td_root = root.join("projects/cap/tech-design");
        let source_file = root.join("install.sh");
        std::fs::create_dir_all(&td_root).unwrap();
        std::fs::write(td_root.join("install.md"), "# install\n").unwrap();
        std::fs::write(
            &source_file,
            "# SPEC-MANAGED: projects/cap/tech-design/install.md#source\n",
        )
        .unwrap();

        let scope = ForceRegenScope {
            td_root: td_root.clone(),
            source_roots: vec![source_file],
        };
        let mut specs = Vec::new();
        collect_force_regen_specs(root, &scope, &mut specs).unwrap();

        assert_eq!(specs, vec![td_root.join("install.md")]);
    }

    #[test]
    fn cb_gen_force_regen_collects_specs_from_td_changes_without_source_refs() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        let td_root = root.join(".aw/tech-design/projects/demo");
        let source_root = root.join("projects/demo/src");
        std::fs::create_dir_all(td_root.join("semantic")).unwrap();
        std::fs::write(
            td_root.join("semantic/demo-src.md"),
            "\
## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/demo/src/lib.rs
    action: modify
    impl_mode: codegen
  - path: projects/demo/src/main.rs
    action: modify
    impl_mode: hand-written
```
",
        )
        .unwrap();

        let scope = ForceRegenScope {
            td_root: td_root.clone(),
            source_roots: vec![source_root],
        };
        let mut specs = Vec::new();
        collect_force_regen_specs(root, &scope, &mut specs).unwrap();

        assert_eq!(specs, vec![td_root.join("semantic/demo-src.md")]);
    }

    #[test]
    fn cb_gen_force_regen_extracts_spec_managed_ref_without_fragment() {
        let content =
            "// SPEC-MANAGED: projects/agentic-workflow/tech-design/surface/interfaces/src/lib.md#source\n";
        assert_eq!(
            extract_spec_managed_ref(content),
            Some("projects/agentic-workflow/tech-design/surface/interfaces/src/lib.md".to_string())
        );
    }

    #[test]
    fn cb_gen_force_regen_prefers_source_spec_managed_ref_for_canonical_target() {
        let content = "\
// SPEC-MANAGED: projects/agentic-workflow/tech-design/surface/interfaces/src/workflow_guard.md#source
// CODEGEN-BEGIN
// CODEGEN-END
// SPEC-MANAGED: projects/agentic-workflow/tech-design/semantic/agentic-workflow-cli.md#schema
// CODEGEN-BEGIN
// CODEGEN-END
";
        assert_eq!(
            extract_spec_managed_ref(content),
            Some(
                "projects/agentic-workflow/tech-design/surface/interfaces/src/workflow_guard.md"
                    .to_string()
            )
        );
    }

    #[test]
    fn cb_gen_force_regen_extracts_all_spec_managed_refs() {
        let content = "\
// SPEC-MANAGED: projects/agentic-workflow/tech-design/surface/interfaces/src/lib.md#source
// CODEGEN-BEGIN
// CODEGEN-END
// SPEC-MANAGED: projects/agentic-workflow/tech-design/surface/interfaces/src/schema.md#schema
// CODEGEN-BEGIN
// CODEGEN-END
";
        assert_eq!(
            extract_spec_managed_refs(content),
            vec![
                "projects/agentic-workflow/tech-design/surface/interfaces/src/lib.md".to_string(),
                "projects/agentic-workflow/tech-design/surface/interfaces/src/schema.md"
                    .to_string()
            ]
        );
    }

    #[test]
    fn cb_gen_cold_rebuild_targets_ignore_hand_written_changes() {
        let spec = "\
## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/cli/cap.rs
    action: modify
    impl_mode: hand-written
```
";
        assert!(extract_cold_rebuild_target_paths(spec).is_empty());
    }

    #[test]
    fn cb_gen_cold_rebuild_targets_include_codegen_changes() {
        let spec = "\
## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/cli/workflow_guard.rs
    action: modify
    impl_mode: codegen
```
";
        assert_eq!(
            extract_cold_rebuild_target_paths(spec),
            vec![std::path::PathBuf::from(
                "projects/agentic-workflow/src/cli/workflow_guard.rs"
            )]
        );
    }

    #[test]
    fn cb_gen_project_root_llms_targets_require_primitive() {
        let spec = "\
## Schema
<!-- type: schema lang: yaml -->

```yaml
semantic_domain:
  evidence:
    source_units:
      - path: projects/tool/llms.txt
        generator_primitives: [project_root_llms]
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/tool/llms.txt
    action: modify
    impl_mode: codegen
```
";
        assert_eq!(
            extract_project_root_llms_target_paths(spec),
            vec![std::path::PathBuf::from("projects/tool/llms.txt")]
        );

        let without_primitive = spec.replace("project_root_llms", "source_unit");
        assert!(extract_project_root_llms_target_paths(&without_primitive).is_empty());
    }

    #[test]
    fn cb_gen_project_root_llms_emitter_writes_codegen_file() {
        let tmp = tempfile::tempdir().unwrap();
        let config = r#"
[[projects]]
name = "tool"
path = "projects/tool"
label = "project:tool"

[[projects.workspaces]]
name = "tool"
paths = ["projects/tool/**"]
target = "rust"
test_cmd = "cargo test -p tool"
"#;
        std::fs::create_dir_all(tmp.path().join(".aw")).unwrap();
        std::fs::write(tmp.path().join(".aw/config.toml"), config).unwrap();
        std::fs::create_dir_all(tmp.path().join("projects/tool")).unwrap();
        std::fs::write(
            tmp.path().join("projects/tool/Cargo.toml"),
            "[package]\nname = \"tool\"\n\n[[bin]]\nname = \"tool\"\npath = \"src/main.rs\"\n",
        )
        .unwrap();
        let spec_path = tmp
            .path()
            .join("projects/tool/tech-design/semantic/tool-projects-tool.md");
        std::fs::create_dir_all(spec_path.parent().unwrap()).unwrap();
        std::fs::write(
            &spec_path,
            "\
## Schema
<!-- type: schema lang: yaml -->

```yaml
semantic_domain:
  evidence:
    source_units:
      - path: projects/tool/llms.txt
        generator_primitives: [project_root_llms]
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/tool/llms.txt
    action: modify
    impl_mode: codegen
```
",
        )
        .unwrap();

        let (updated, created, changed) =
            write_project_root_llms_targets(tmp.path(), tmp.path(), &[spec_path], false).unwrap();

        assert_eq!(updated, 0);
        assert_eq!(created, 1);
        assert_eq!(changed.len(), 1);
        let generated = std::fs::read_to_string(tmp.path().join("projects/tool/llms.txt")).unwrap();
        assert!(generated.contains("<!-- CODEGEN-BEGIN -->"));
        assert!(generated.contains("## Tech Design"));
        assert!(generated.contains("`cargo test -p tool`"));
    }

    #[test]
    fn cb_gen_force_regen_compare_source_roots_detects_byte_mismatch() {
        let tmp = tempfile::tempdir().unwrap();
        let original = tmp.path().join("original");
        let generated = tmp.path().join("generated");
        let rel = std::path::Path::new("projects/agentic-workflow/src/cli/lib.rs");
        std::fs::create_dir_all(original.join("projects/agentic-workflow/src/cli")).unwrap();
        std::fs::create_dir_all(generated.join("projects/agentic-workflow/src/cli")).unwrap();
        std::fs::write(original.join(rel), "pub fn a() {}\n").unwrap();
        std::fs::write(generated.join(rel), "pub fn b() {}\n").unwrap();

        let mismatches = compare_source_roots(
            &original,
            &generated,
            &[original.join("projects/agentic-workflow")],
        )
        .unwrap();

        assert_eq!(mismatches, vec!["projects/agentic-workflow/src/cli/lib.rs"]);
    }

    #[test]
    fn cb_gen_force_regen_compare_source_roots_handles_file_root() {
        let tmp = tempfile::tempdir().unwrap();
        let original = tmp.path().join("original");
        let generated = tmp.path().join("generated");
        let rel = std::path::Path::new("install.sh");
        std::fs::create_dir_all(&original).unwrap();
        std::fs::create_dir_all(&generated).unwrap();
        std::fs::write(original.join(rel), "#!/bin/sh\nexit 0\n").unwrap();
        std::fs::write(generated.join(rel), "#!/bin/sh\nexit 1\n").unwrap();

        let mismatches =
            compare_source_roots(&original, &generated, &[original.join(rel)]).unwrap();

        assert_eq!(mismatches, vec!["install.sh"]);
    }

    #[test]
    fn cb_gen_force_regen_specs_do_not_format_handwritten_skips() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        let td_root = root.join("tech-design");
        let source_root = root.join("src");
        let source = source_root.join("lib.rs");
        let spec = td_root.join("cap-src.md");
        std::fs::create_dir_all(&td_root).unwrap();
        std::fs::create_dir_all(&source_root).unwrap();
        let original = "pub fn demo(){println!(\"left as authored\");}\n";
        std::fs::write(&source, original).unwrap();
        std::fs::write(
            &spec,
            r#"---
id: cap-src
---

# cap src

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: src/lib.rs
    action: modify
    impl_mode: hand-written
    description: Preserve hand-written source.
```
"#,
        )
        .unwrap();

        let scope = ForceRegenScope {
            td_root,
            source_roots: vec![source_root],
        };
        let (updated, created, blocks, changed_paths) =
            run_force_regen_specs(root, &scope, &[spec], false, true).unwrap();
        format_rust_files(&changed_paths).unwrap();

        assert_eq!((updated, created, blocks), (0, 0, 0));
        assert!(changed_paths.is_empty());
        assert_eq!(std::fs::read_to_string(source).unwrap(), original);
    }

    #[test]
    fn cb_verify_summary_marks_replay_mismatches_unclean() {
        let mut report = ForceRegenConformanceReport::default();
        report.public_symbols = 3;
        report.td_semantic_public_symbols = 3;

        let summary =
            cb_verify_summary_from_report(report, vec!["projects/jet/src/lib.rs".to_string()]);

        assert!(!summary.clean);
        assert_eq!(summary.public_api_covered, 3);
        assert_eq!(summary.public_api_total, 3);
        assert_eq!(
            summary.failures,
            vec!["projects/jet/src/lib.rs: differs after TD replay"]
        );
    }

    #[test]
    fn cb_gen_force_regen_semantic_sample_is_deterministic_and_ceil_based() {
        let units = (0..10)
            .map(|i| SemanticReviewUnit {
                spec_ref: format!("spec-{i}.md#source"),
                target_path: std::path::PathBuf::from(format!("src/file_{i}.rs")),
                reason: "source-from-target".to_string(),
            })
            .collect::<Vec<_>>();

        assert_eq!(sample_count(units.len(), 0.15), 2);
        let unit_refs = units.iter().collect::<Vec<_>>();
        let first = sample_semantic_review_units(&unit_refs, 0.15)
            .unwrap()
            .into_iter()
            .map(|unit| unit.spec_ref.clone())
            .collect::<Vec<_>>();
        let second = sample_semantic_review_units(&unit_refs, 0.15)
            .unwrap()
            .into_iter()
            .map(|unit| unit.spec_ref.clone())
            .collect::<Vec<_>>();

        assert_eq!(first, second);
        assert_eq!(first.len(), 2);
    }

    #[test]
    fn cb_gen_force_regen_treats_minified_viewer_assets_as_non_source() {
        assert!(is_minified_asset_file(std::path::Path::new(
            "projects/agentic-workflow/src/ui/viewer/assets/mermaid.min.js"
        )));
        assert!(!is_minified_asset_file(std::path::Path::new(
            "projects/agentic-workflow/src/ui/viewer/app.js"
        )));
        assert!(!is_minified_asset_file(std::path::Path::new(
            "projects/agentic-workflow/src/ui/viewer/mermaid.min.js"
        )));
    }

    #[test]
    fn cb_gen_force_regen_upserts_public_api_overview_manifest() {
        let spec = "---\nid: demo\nfill_sections: [source, changes]\n---\n\n# Demo\n\n## Source\n<!-- type: source lang: rust -->\n<!-- source-from-target: strip-managed-markers -->\n\n## Changes\n<!-- type: changes lang: yaml -->\n\n```yaml\nchanges: []\n```\n";
        let symbols = vec![
            Symbol {
                name: "run".to_string(),
                kind: SymbolKind::Function,
                signature: Some("pub fn run() -> Result<()>".to_string()),
                line: 12,
                is_public: true,
                ..Default::default()
            },
            Symbol {
                name: "helper".to_string(),
                kind: SymbolKind::Function,
                line: 20,
                is_public: false,
                ..Default::default()
            },
        ];

        let updated =
            upsert_public_api_overview(spec, "projects/agentic-workflow/src/cli/demo.rs", &symbols);

        assert!(updated.contains("fill_sections: [overview, source, changes]"));
        assert!(updated.contains("## Overview\n<!-- type: overview lang: markdown -->"));
        assert!(updated.contains(
            "| `run` | projects/agentic-workflow/src/cli/demo.rs | function | pub | 12 | pub fn run() -> Result<()> |"
        ));
        assert!(!updated.contains("`helper`"));
        assert!(updated.find("## Overview").unwrap() < updated.find("## Source").unwrap());
    }

    #[test]
    fn cb_gen_force_regen_upsert_ignores_headings_inside_code_fences() {
        let spec = r##"---
id: demo
fill_sections: [source, changes]
---

# Demo

## Source
<!-- type: source lang: rust -->

````rust
let spec = r#"
## Overview
<!-- type: overview lang: markdown -->

Some prose.
"#;
````

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes: []
```
"##;
        let symbols = vec![Symbol {
            name: "run".to_string(),
            kind: SymbolKind::Function,
            signature: Some("pub fn run()".to_string()),
            line: 7,
            is_public: true,
            ..Default::default()
        }];

        let updated =
            upsert_public_api_overview(spec, "projects/agentic-workflow/src/cli/demo.rs", &symbols);

        assert!(updated.contains("Some prose."));
        assert!(updated.find("## Overview").unwrap() < updated.find("## Source").unwrap());
        assert_eq!(updated.matches("Public API manifest for").count(), 1);
    }

    #[test]
    fn cb_gen_force_regen_upsert_aggregates_multi_target_public_api_manifest() {
        let spec = "---\nid: demo\nfill_sections: [source, changes]\n---\n\n# Demo\n\n## Source\n";
        let updated = upsert_public_api_overview_targets(
            spec,
            &[
                PublicApiManifestTarget {
                    target_rel_path: "src/a.rs".to_string(),
                    symbols: vec![PublicApiManifestSymbol {
                        name: "Alpha".to_string(),
                        kind: "struct".to_string(),
                        line: 4,
                        signature: None,
                    }],
                },
                PublicApiManifestTarget {
                    target_rel_path: "src/b.rs".to_string(),
                    symbols: vec![PublicApiManifestSymbol {
                        name: "beta".to_string(),
                        kind: "function".to_string(),
                        line: 8,
                        signature: Some("pub fn beta()".to_string()),
                    }],
                },
            ],
        );

        assert!(updated.contains("Public API manifest for 2 target files"));
        assert!(updated.contains("| `Alpha` | src/a.rs | struct | pub | 4 |  |"));
        assert!(updated.contains("| `beta` | src/b.rs | function | pub | 8 | pub fn beta() |"));
    }

    #[test]
    fn cb_gen_force_regen_reports_td_public_symbol_semantic_coverage() {
        let spec = r#"## Schema
<!-- type: schema lang: yaml -->

```yaml
definitions:
  Request:
    type: object
  Response:
    type: object
```

## Logic: handle_request
<!-- type: logic lang: yaml -->

```yaml
id: handle_request
entry: start
nodes: {}
edges: []
```

## Overview
<!-- type: overview lang: markdown -->

### Symbols

| Name | Kind | Visibility |
|------|------|------------|
| `OverviewOnly` | struct | pub |

### Public Signatures

```rust
pub fn signature_only() -> Result<()>
```
"#;
        let symbols = vec![
            Symbol {
                name: "Request".to_string(),
                kind: SymbolKind::Struct,
                is_public: true,
                ..Default::default()
            },
            Symbol {
                name: "handle_request".to_string(),
                kind: SymbolKind::Function,
                is_public: true,
                ..Default::default()
            },
            Symbol {
                name: "internal_helper".to_string(),
                kind: SymbolKind::Function,
                is_public: false,
                ..Default::default()
            },
            Symbol {
                name: "OverviewOnly".to_string(),
                kind: SymbolKind::Struct,
                is_public: true,
                ..Default::default()
            },
            Symbol {
                name: "signature_only".to_string(),
                kind: SymbolKind::Function,
                is_public: true,
                ..Default::default()
            },
            Symbol {
                name: "MissingPublic".to_string(),
                kind: SymbolKind::Struct,
                is_public: true,
                ..Default::default()
            },
        ];

        let coverage = td_public_symbol_semantic_coverage(spec, &symbols).unwrap();

        assert_eq!(
            coverage,
            PublicSymbolSemanticCoverage {
                total_public_symbols: 5,
                covered_public_symbols: 4,
                missing_public_symbols: vec!["MissingPublic".to_string()],
            }
        );
        assert_eq!(
            coverage.review_reason(),
            "source-from-target; public-api-semantic 4/5; missing MissingPublic"
        );
    }

    #[test]
    fn cb_gen_force_regen_fails_incomplete_public_api_semantic_conformance() {
        let mut report = ForceRegenConformanceReport {
            public_symbols: 5,
            td_semantic_public_symbols: 4,
            ..Default::default()
        };

        report.enforce_complete_public_api_semantic_conformance();

        assert_eq!(report.failures.len(), 1);
        assert!(report.failures[0].contains("4/5 public symbol(s) covered"));
    }

    #[test]
    fn cb_gen_force_regen_fails_unmanaged_source_files() {
        let mut report = ForceRegenConformanceReport {
            code_files: 2,
            managed_code_files: 1,
            unmanaged_code_files: vec![SemanticReviewUnit {
                spec_ref: "(none)".to_string(),
                target_path: std::path::PathBuf::from(
                    "projects/agentic-workflow/src/generate/marker.rs",
                ),
                reason: "no-ownership-marker".to_string(),
            }],
            ..Default::default()
        };

        report.enforce_complete_source_ownership_coverage();

        assert_eq!(report.failures.len(), 1);
        assert!(report.failures[0]
            .contains("1/2 source file(s) have CODEGEN or HANDWRITE ownership markers"));
        assert!(report.failures[0].contains("projects/agentic-workflow/src/generate/marker.rs"));
    }

    #[test]
    fn cb_verify_source_scope_collection_skips_dependency_dirs_and_symlink_cycles() {
        let dir = tempfile::tempdir().unwrap();
        let source_root = dir.path().join("projects/jet");
        std::fs::create_dir_all(source_root.join("src")).unwrap();
        std::fs::create_dir_all(source_root.join("node_modules/pkg")).unwrap();
        std::fs::create_dir_all(source_root.join("target/debug")).unwrap();
        std::fs::write(source_root.join("src/lib.rs"), "pub fn owned() {}\n").unwrap();
        std::fs::write(source_root.join("node_modules/pkg/index.ts"), "export {}\n").unwrap();
        std::fs::write(source_root.join("target/debug/build.rs"), "fn main() {}\n").unwrap();
        #[cfg(unix)]
        std::os::unix::fs::symlink(&source_root, source_root.join("cycle")).unwrap();

        let scope = ForceRegenScope {
            td_root: dir.path().join("tech-design"),
            source_roots: vec![source_root.clone()],
        };
        let files = collect_source_scope_files(&scope).unwrap();
        let rel_files = files
            .iter()
            .map(|path| path.strip_prefix(&source_root).unwrap().to_path_buf())
            .collect::<Vec<_>>();

        assert_eq!(rel_files, vec![std::path::PathBuf::from("src/lib.rs")]);

        let copy_root = dir.path().join("copy");
        copy_tree(&source_root, &copy_root).unwrap();
        assert!(copy_root.join("src/lib.rs").is_file());
        assert!(!copy_root.join("node_modules/pkg/index.ts").exists());
        assert!(!copy_root.join("target/debug/build.rs").exists());

        let tree_files = collect_tree_files(&source_root).unwrap();
        assert_eq!(
            tree_files.keys().cloned().collect::<Vec<_>>(),
            vec![std::path::PathBuf::from("src/lib.rs")]
        );
    }

    #[test]
    fn cb_gen_force_regen_accepts_handwrite_ownership_marker() {
        assert!(has_handwrite_ownership_marker(
            "// SPEC-MANAGED: projects/demo/tech-design/source.md#source\n\
             // HANDWRITE-BEGIN gap=\"g\" tracker=\"t\" reason=\"not generated yet\"\n\
             fn main() {}\n\
             // HANDWRITE-END\n"
        ));
        assert!(has_handwrite_ownership_marker(
            "# <HANDWRITE gap=\"g\" tracker=\"t\" reason=\"not generated yet\">\n\
             echo ok\n\
             # </HANDWRITE>\n"
        ));
        assert!(!has_handwrite_ownership_marker(
            "let marker = \"HANDWRITE-BEGIN\";"
        ));
    }

    // ── cb claim: tracker linkage (issue #925) ─────────────────────

    #[test]
    fn claim_issue_title_is_stable_and_derived_from_code_path() {
        assert_eq!(
            claim_issue_title("projects/tool/src/lib.rs"),
            "Adopted (td code-claim): projects/tool/src/lib.rs"
        );
    }

    #[test]
    fn repo_relative_code_path_normalizes_dot_slash_and_absolute_paths() {
        let root = std::path::Path::new("/repo");
        assert_eq!(repo_relative_code_path(root, "./src/lib.rs"), "src/lib.rs");
        assert_eq!(repo_relative_code_path(root, "src/lib.rs"), "src/lib.rs");
        assert_eq!(
            repo_relative_code_path(root, "/repo/src/lib.rs"),
            "src/lib.rs"
        );
    }

    // Mirrors `standardize::gap_issue_create_args_uses_typed_fields_and_bounded_skeleton`
    // (issue #919): the code-claim tracker issue must be filed with typed
    // fields, not a freeform body, so `run_create`'s own structured skeleton
    // fills in the description.
    #[test]
    fn claim_issue_create_args_uses_typed_refactor_fields_and_bounded_skeleton() {
        let title = claim_issue_title("projects/tool/src/lib.rs");
        let args = claim_issue_create_args(&title, "tool".to_string());
        assert_eq!(args.title.as_deref(), Some(title.as_str()));
        assert!(matches!(
            args.issue_type,
            Some(crate::cli::issues::TypeFilter::Refactor)
        ));
        assert_eq!(args.projects, vec!["tool".to_string()]);
        assert!(args.body.is_none(), "no free-form body");
        assert!(args.body_file.is_none());
        assert!(args.draft_path.is_none());
        assert!(!args.json, "json:true would make backend failures fatal");
    }

    // Mirrors `standardize::ensure_gap_issue_reports_recoverable_error_when_backend_unconfigured`
    // (issue #919): a configured project with no issue backend must surface
    // a normal recoverable `Result::Err`, not panic/exit, so `run_claim` can
    // warn-and-proceed.
    #[test]
    fn ensure_claim_issue_reports_recoverable_error_when_backend_unconfigured() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(dir.path().join(".aw")).unwrap();
        std::fs::write(
            dir.path().join(".aw/config.toml"),
            "[[projects]]\n\
             name = \"tool\"\n\
             path = \"projects/tool\"\n\
             label = \"project:tool\"\n\
             \n\
             [[projects.workspaces]]\n\
             name = \"tool\"\n\
             paths = [\"projects/tool/**\"]\n\
             target = \"rust\"\n",
        )
        .unwrap();
        std::fs::create_dir_all(dir.path().join("projects/tool/src")).unwrap();
        std::fs::write(
            dir.path().join("projects/tool/src/lib.rs"),
            "pub fn x() {}\n",
        )
        .unwrap();

        let err = ensure_claim_issue(dir.path(), "projects/tool/src/lib.rs").unwrap_err();
        let msg = err.to_string();
        assert!(
            msg.contains("repo_platform") || msg.contains("issue_platform"),
            "expected a backend-configuration error, got: {msg}"
        );
    }

    #[test]
    fn ensure_claim_issue_errors_when_no_project_configured_for_path() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(dir.path().join("src")).unwrap();
        std::fs::write(dir.path().join("src/lib.rs"), "pub fn x() {}\n").unwrap();

        let err = ensure_claim_issue(dir.path(), "src/lib.rs").unwrap_err();
        assert!(
            err.to_string().contains("no configured project owns"),
            "got: {}",
            err
        );
    }

    fn init_claim_test_git_repo(root: &std::path::Path) {
        let run = |args: &[&str]| {
            std::process::Command::new("git")
                .arg("-C")
                .arg(root)
                .args(args)
                .output()
                .unwrap()
        };
        run(&["init", "--initial-branch=main"]);
        run(&["config", "user.email", "test@example.com"]);
        run(&["config", "user.name", "Test"]);
        run(&["commit", "--allow-empty", "-m", "init"]);
    }

    fn claim_test_git_log_body(root: &std::path::Path) -> String {
        let out = std::process::Command::new("git")
            .arg("-C")
            .arg(root)
            .args(["log", "-1", "--format=%B"])
            .output()
            .unwrap();
        String::from_utf8_lossy(&out.stdout).into_owned()
    }

    #[test]
    fn commit_cb_claim_trailer_without_issue_omits_claim_issue_trailer() {
        if !git_available() {
            return;
        }
        let dir = tempfile::tempdir().unwrap();
        init_claim_test_git_repo(dir.path());
        commit_cb_claim_trailer(dir.path(), "demo-slug", "src/lib.rs", None).unwrap();
        let body = claim_test_git_log_body(dir.path());
        assert!(body.contains("Lifecycle-Stage: Cb-Claim"), "body:\n{body}");
        assert!(!body.contains("Claim-Issue:"), "body:\n{body}");
    }

    #[test]
    fn commit_cb_claim_trailer_with_remote_issue_uses_issue_number() {
        if !git_available() {
            return;
        }
        let dir = tempfile::tempdir().unwrap();
        init_claim_test_git_repo(dir.path());
        let issue_ref = ClaimIssueRef {
            slug: "adopted-lib".to_string(),
            number: Some(42),
        };
        commit_cb_claim_trailer(dir.path(), "demo-slug", "src/lib.rs", Some(&issue_ref)).unwrap();
        let body = claim_test_git_log_body(dir.path());
        assert!(body.contains("Claim-Issue: #42"), "body:\n{body}");
    }

    #[test]
    fn commit_cb_claim_trailer_falls_back_to_slug_without_issue_number() {
        if !git_available() {
            return;
        }
        let dir = tempfile::tempdir().unwrap();
        init_claim_test_git_repo(dir.path());
        let issue_ref = ClaimIssueRef {
            slug: "adopted-lib".to_string(),
            number: None,
        };
        commit_cb_claim_trailer(dir.path(), "demo-slug", "src/lib.rs", Some(&issue_ref)).unwrap();
        let body = claim_test_git_log_body(dir.path());
        assert!(body.contains("Claim-Issue: adopted-lib"), "body:\n{body}");
    }
}

/// True if `target` (an `aw td code-check <target>` argument) does not
/// resolve to an on-disk path relative to `project_root` — i.e. it should be
/// dispatched as a lifecycle slug (terminal code-check) rather than an audit
/// path.
///
/// Shared by `run_check` below and td.rs's `TdCommand::CodeCheck`
/// workflow-guard arm — issue #856d, replacing what were three copies of the
/// same `target_path.is_absolute() -> join -> exists()` check (the third,
/// the dead top-level `aw cb check` dispatcher's own `CbCommand::Check` arm,
/// was removed by issue #860 along with the rest of the unreachable
/// `CbArgs`/`CbCommand`/`run` dispatcher).
pub(crate) fn code_check_target_is_slug(project_root: &std::path::Path, target: &str) -> bool {
    let target_path = std::path::Path::new(target);
    let target_abs = if target_path.is_absolute() {
        target_path.to_path_buf()
    } else {
        project_root.join(target_path)
    };
    !target_abs.exists()
}

// Implementation of `aw td code-check` — delegates to the pre-existing
// audit pipeline. Path defaults to `.` when omitted to match
// the historical audit behaviour.
///
// @spec projects/agentic-workflow/tech-design/surface/specs/score-namespaces.md#changes
pub async fn run_check(args: CbCheckArgs) -> Result<()> {
    let project_root = crate::find_project_root()?;
    if let Some(target) = args.target.as_deref() {
        if code_check_target_is_slug(&project_root, target)
            && run_check_lifecycle_terminal(&project_root, target, args.allow_empty_impl).await?
        {
            return Ok(());
        }
    }

    let td_args = AuditArgs {
        path: args.target,
        json: args.json,
        group_by: args.group_by,
        ready_only: false,
        drift: false,
    };
    td::run_audit(td_args)
}

/// Terminal `aw td code-check <slug>` — advances a fresh `cb_genned` /
/// `cb_filled` (or legacy `td_gen_coded`) issue to `td_merged`, then runs the
/// resumable terminal step sequence (remote closure, `td-<slug>` branch
/// landing, `Cb-CodeCheck` trailer commit, workflow-lock release).
///
/// `td_merged` itself is accepted as a **retry** entry (issue #846): a prior
/// run may have advanced phase + closed the issue via `backend.update` and
/// then failed before finishing the remaining steps (network error on push,
/// git commit failure, ...), which would otherwise strand the issue with
/// `score:locked` forever since every retry hit the phase guard. On retry,
/// the marker gate, the empty-implementation gate, and the phase-advancing
/// `backend.update` are all skipped (they already ran on the attempt that
/// reached `td_merged`), and each remaining step is re-attempted
/// idempotently — see the per-step comments below for how each one avoids
/// redoing already-completed work.
///
/// `allow_empty_impl` skips the empty-implementation gate (issue #847) on a
/// fresh entry; it has no effect on a retry, which never runs that gate.
///
/// Envelopes below are hand-rolled `serde_json::json!` + compact
/// `to_string` rather than td.rs's `TdEnvelope`/`print_envelope` (issue
/// #856e considered this consolidation and deliberately left it out of
/// scope): `TdEnvelope::Done` has no field for the `landing` payload the
/// final envelope carries, and — more importantly — `print_envelope` uses
/// `to_string_pretty` while every `td_no_merge_test.rs` assertion on this
/// path matches compact-JSON substrings with no space after the colon
/// (e.g. `"action":"done"`, `"status":"landed"`); switching to pretty
/// output would break all of them. Extending `TdEnvelope` to also support a
/// compact-with-extra-fields terminal shape is a real refactor of its own,
/// not a drive-by inside #856.
async fn run_check_lifecycle_terminal(
    project_root: &std::path::Path,
    slug: &str,
    allow_empty_impl: bool,
) -> Result<bool> {
    use crate::cli::remote_push::maybe_push_remote;
    use crate::issues::types::{td_phase, ShipStatus};
    use crate::issues::{IssueBackend, IssuePatch, IssueState, LocalBackend};

    let backend = LocalBackend::from_project_root(project_root);
    let Some(issue) = backend.get(slug).await? else {
        // Issue #859 part c: the local issue cache under `/tmp/aw` is
        // ephemeral (cleared on reboot / a fresh checkout) while git history
        // (`Lifecycle-Slug` trailers) and any remote backend issue persist.
        // Returning `Ok(false)` here used to fall through to `run_check`'s
        // `td::run_audit(...)` path lookup, which then misroutes this into
        // an "audit target not found" message about a missing *path* —
        // nothing to do with the real problem (a missing *issue*). Emit an
        // explicit, actionable envelope instead and claim the dispatch
        // (`Ok(true)`) so `run_check` never falls through.
        //
        // This deliberately does not attempt full rehydration via
        // `td::bootstrap_td_issue`: that helper hard-bails when the remote
        // issue's `state != Open`, which is exactly the shape a legitimate
        // terminal *retry* can be in (a prior `aw td code-check` run already
        // closed the remote issue via `backend.update`/`maybe_push_remote`
        // and then failed before finishing its remaining steps) —
        // unconditionally reusing it here would turn a resumable retry into
        // a hard failure. It also carries side effects (workspace/branch
        // activation, a `Td-Hydrate` commit) that don't belong behind a
        // terminal-step entry point. A read-only git-log check is enough to
        // tell the caller which remediation applies.
        let message = if slug_has_lifecycle_history(project_root, slug)? {
            format!(
                "no local work-item '{slug}' found (the local issue cache is ephemeral and may \
                 have been cleared); this slug has prior lifecycle commits in this worktree — \
                 re-run `aw td gen {slug}` (or `aw td create {slug}` if gen was never reached) \
                 to rehydrate the local issue before retrying `aw td code-check {slug}`"
            )
        } else {
            format!(
                "no local work-item '{slug}' found and no lifecycle history for it in this \
                 worktree; nothing to code-check"
            )
        };
        let env = serde_json::json!({
            "action": "error",
            "slug": slug,
            "message": message,
        });
        println!("{}", serde_json::to_string(&env)?);
        return Ok(true);
    };
    let phase = issue.phase.as_deref().unwrap_or("");
    let is_retry = td_phase::is_terminal_code_check_retry(phase);
    if !td_phase::is_terminal_code_checkable(phase) && !is_retry {
        let env = serde_json::json!({
            "action": "error",
            "slug": slug,
            "message": format!(
                "cannot complete code-check: phase is '{}', expected '{}', '{}', legacy '{}', or terminal retry '{}'",
                phase,
                td_phase::CB_FILLED,
                td_phase::CB_GENNED,
                td_phase::LEGACY_TD_GEN_CODED,
                td_phase::TD_MERGED,
            ),
        });
        println!("{}", serde_json::to_string(&env)?);
        return Ok(true);
    }

    // From here the sequence is resumable: a fresh entry advances phase to
    // td_merged (and folds the workflow-lock unlock into that same write —
    // issue #859 part b) below; a retry entry (phase already td_merged)
    // reuses the issue exactly as read above. Every remaining step is
    // self-checking so a partial failure at any point can be recovered by
    // re-running `aw td code-check <slug>`.
    let closed_issue = if is_retry {
        issue
    } else {
        // Scope both terminal gates to this WI's own TD spec (issue #854)
        // instead of the whole worktree / whole `tech_design_path` tree —
        // see `resolve_slug_spec_paths` below. Needed unconditionally
        // (the marker gate, the touched-scope standardization gate, and the
        // empty-implementation gate that follows all consume
        // `slug_spec_paths` / the scope it derives).
        let slug_spec_paths = resolve_slug_spec_paths(project_root, &issue);

        // Issue #932: hoisted out of the `phase != CB_FILLED` marker-gate
        // guard below so it's always available — the touched-scope
        // standardization gate that follows needs this WI's own Changes
        // paths regardless of which phase reached terminal code-check.
        let mut marker_gate_scope: Vec<String> = Vec::new();
        for spec_abs in &slug_spec_paths {
            if let Ok(content) = std::fs::read_to_string(spec_abs) {
                marker_gate_scope.extend(crate::cli::cb_fill::extract_change_paths_from_spec(
                    &content,
                ));
            }
        }

        // Marker gate (issue #859 part a): `aw td fill`'s own apply loop
        // already re-enumerates the whole worktree after every marker write
        // and only advances phase to `cb_filled` once that re-enumeration
        // finds zero unfilled markers (see `run_apply` in cb_fill.rs) — so a
        // fresh entry that reaches this point at `cb_filled` has already had
        // this exact gate proven true by construction. Re-running it here
        // would be a third full/scoped walk of the same already-established
        // fact. Only run it for `cb_genned` / legacy `td_gen_coded` entries,
        // which reach terminal code-check WITHOUT ever going through fill's
        // gate (e.g. a HANDWRITE-marker-free WI that skips `aw td fill`
        // entirely).
        if phase != td_phase::CB_FILLED {
            if let Err(message) =
                crate::cli::cb_fill::run_cb_check_gate_scoped(project_root, &marker_gate_scope)
                    .await
            {
                let env = serde_json::json!({
                    "action": "error",
                    "slug": slug,
                    "message": format!("td code-check gate failed: {}", message),
                });
                println!("{}", serde_json::to_string(&env)?);
                return Ok(true);
            }
        }

        // Touched-scope standardization gate (issue #932): the forward
        // (正流程) loop is the only place that can catch a fresh
        // regenerability regression before it lands — a WI that adds or
        // touches an in-scope file without a CODEGEN/HANDWRITE marker (or
        // leaves a HANDWRITE marker's gap/tracker attrs unfilled) is exactly
        // the kind of drift 標準化 exists to prevent. Scoped to this WI's own
        // touched-file set (same branch-diff ∪ Changes-paths union the
        // marker gate above uses via `cb_fill::resolve_touched_scope`) so
        // pre-existing unmarked files elsewhere in the tree never affect
        // this WI's verdict (no reintroduction of the #854 inherited-marker
        // class). Runs unconditionally (both `cb_filled` and
        // `cb_genned`/legacy entries), unlike the marker gate above.
        if let Some(message) =
            touched_scope_standardization_gate_message(project_root, &issue, &marker_gate_scope)
        {
            let env = serde_json::json!({
                "action": "error",
                "slug": slug,
                "message": message,
            });
            println!("{}", serde_json::to_string(&env)?);
            return Ok(true);
        }

        // Empty-implementation gate (issue #847, restoring the removed `aw
        // td merge` Bug-2 guard): refuse completion when a spec's Changes
        // section lists N create/modify entries and every one of them is
        // missing on disk — the signature of gen-code having been skipped
        // entirely (e.g. a hand-written batch with no scaffold that was
        // never actually implemented). `--allow-empty-impl` overrides for
        // legitimate spec-only completions.
        if allow_empty_impl {
            eprintln!(
                "[td code-check] WARNING: --allow-empty-impl set; skipping empty-implementation gate"
            );
        } else if let Some(message) =
            empty_implementation_gate_message(project_root, slug, &slug_spec_paths)
        {
            let env = serde_json::json!({
                "action": "error",
                "slug": slug,
                "message": message,
            });
            println!("{}", serde_json::to_string(&env)?);
            return Ok(true);
        }

        // Issue #859 part b: fold the workflow-lock projection unlock into
        // this same patch instead of a separate `complete_issue_lock`
        // local write + remote push after the fact — `unlock_projection_for_
        // closed_issue` returns `Ok(None)` (leaving `patch.body` untouched)
        // when the issue body carries no projection block at all, so this
        // is a no-op for issues that never had one. Step 4 below still
        // calls `complete_issue_lock` unconditionally for retry/legacy
        // safety (a prior run that advanced phase here but failed before
        // this fold existed, or before it ran), but that function's own
        // early-return now treats "already unlocked" as a true no-op, so
        // the common case here does not pay for a second write + push.
        let unlocked_body =
            crate::cli::workflow_guard::unlock_projection_for_closed_issue(&issue.body, slug)?;

        let patch = IssuePatch {
            state: Some(IssueState::Closed),
            phase: Some(td_phase::TD_MERGED.to_string()),
            ship_status: Some(ShipStatus::Step1Shipped),
            add_labels: vec![format!("phase:{}", td_phase::TD_MERGED)],
            // Issue #856a: shared with td.rs's own test coverage instead of
            // a second, narrower, drifted inline copy (this one was missing
            // the retired `td_reviewed` / `cb_reviewed` CRRR-phase labels
            // and hardcoded the `td_gen_coded` string literal).
            remove_labels: td::terminal_code_check_labels_to_remove(),
            body: unlocked_body,
            flagged_sections: Some(vec![]),
            validation_errors: Some(vec![]),
            ..Default::default()
        };
        // Issue #859 part b: `update()` already returns the freshly patched
        // `Issue` — reuse it directly instead of a redundant second
        // `backend.get` that would just re-read the same write back off
        // disk.
        backend.update(slug, &patch).await?
    };
    let closed_path = backend.issue_path(&closed_issue);

    // Step 1 — remote closure. `push_through` only *creates* on the remote
    // when no remote issue exists yet (looked up by github_id/gitlab_id once
    // known); otherwise it updates the existing remote issue and refreshes
    // the local copy. Re-running it against an already-closed remote issue
    // is a safe, idempotent write — no separate "already happened" gate.
    maybe_push_remote(project_root, &closed_path, slug).await?;

    // Step 2 — land the `td-<slug>` lifecycle branch (issue #842), ordered
    // before the trailer commit so the trailer (and every implementation
    // commit already on `td-<slug>`) end up reachable from the landing
    // target instead of stranded on the lifecycle branch. No-ops for
    // in-place/off-main lifecycles, which never create `td-<slug>`.
    // Naturally idempotent: `land_td_lifecycle_branch` returns `NoBranch`
    // once a prior run has already deleted the branch, so a retry after the
    // trailer/lock steps failed does not attempt a second merge.
    let landing = match land_td_lifecycle_branch(project_root, slug, &closed_issue) {
        Ok(outcome) => outcome,
        Err(e) => {
            let env = serde_json::json!({
                "action": "error",
                "slug": slug,
                "message": format!(
                    "td code-check landing failed: {}; resolve and re-run `aw td code-check {}`",
                    e, slug
                ),
            });
            println!("{}", serde_json::to_string(&env)?);
            return Ok(true);
        }
    };

    // Step 3 — terminal lifecycle commit. NOT naturally idempotent: calling
    // `commit_cb_code_check_terminal` unconditionally on every retry would
    // spam duplicate --allow-empty commits. Gate it on whether a commit with
    // the exact `Lifecycle-Slug` + `Lifecycle-Stage: Cb-CodeCheck` trailer
    // pair already exists in the log. Runs after landing, so it commits on
    // the landing target (or on the current branch when there was nothing
    // to land).
    if !terminal_commit_already_landed(project_root, slug)? {
        commit_cb_code_check_terminal(project_root, slug, &closed_path)?;
    }

    // Step 4 — lock release. `complete_issue_lock` already no-ops when the
    // lock label / projection is already clear, so it's safe to re-run.
    crate::cli::workflow_guard::complete_issue_lock(project_root, slug, "td").await?;

    let landing_json = match &landing {
        BranchLandingOutcome::NoBranch => serde_json::json!({
            "status": "skipped",
            "branch": null,
            "target": null,
        }),
        BranchLandingOutcome::AlreadyMerged { branch, target } => serde_json::json!({
            "status": "already_merged",
            "branch": branch,
            "target": target,
        }),
        BranchLandingOutcome::Landed { branch, target } => serde_json::json!({
            "status": "landed",
            "branch": branch,
            "target": target,
        }),
    };

    let env = serde_json::json!({
        "action": "done",
        "slug": slug,
        "message": if is_retry {
            "td code-check retry: remaining terminal steps completed; lifecycle closed"
        } else {
            "td code-check passed; lifecycle closed"
        },
        "landing": landing_json,
    });
    println!("{}", serde_json::to_string(&env)?);
    Ok(true)
}

/// Resolve the completing slug's own TD spec file(s) (issue #854): every
/// `.md` ref in `Issue.implements`; else the worktree's uniquely
/// branch-diff-discovered TD spec (`td::discover_worktree_spec`); else the
/// deterministic default path `aw td create` would have used for this issue
/// (`td::default_spec_path_for_issue_in_project`) as a last resort, for
/// issues created before `aw td create`/`aw td claim` started populating
/// `Issue.implements` (#939) or when a worktree already on its base branch
/// has no branch diff to discover from. Both terminal gates below scope to exactly this
/// set instead of the whole `tech_design_path` tree / whole worktree, so an
/// unrelated stale spec or inherited HANDWRITE marker elsewhere in a
/// monorepo checkout can no longer block this WI's own completion.
///
/// An empty result — no source resolves a spec (a fresh issue with a title
/// that derives no spec filename) — is a legitimate "nothing to scope
/// against" outcome. Callers must treat it as pass vacuously, not fall back
/// to a whole-tree scan (that whole-tree fallback is exactly the bug issue
/// #854 fixes). A resolved path that does not exist on disk (the default
/// guess was wrong, or the spec genuinely has no Changes section) is
/// likewise harmless — callers below already skip unreadable spec files.
fn resolve_slug_spec_paths(
    project_root: &std::path::Path,
    issue: &crate::issues::Issue,
) -> Vec<std::path::PathBuf> {
    let mut rels: Vec<String> = issue
        .implements
        .iter()
        .filter(|s| s.ends_with(".md"))
        .cloned()
        .collect();
    if rels.is_empty() {
        if let Some(discovered) = crate::cli::td::discover_worktree_spec(project_root) {
            rels.push(discovered);
        }
    }
    if rels.is_empty() {
        rels.push(crate::cli::td::default_spec_path_for_issue_in_project(
            project_root,
            issue,
            &issue.slug,
        ));
    }
    rels.into_iter().map(|r| project_root.join(r)).collect()
}

/// Resolve the owning project name from an issue's `project:<name>` label,
/// the same convention `aw wi create --project` and the standardize/health
/// surfaces use. Deliberately a small local duplicate of `td.rs`'s
/// equivalent helper rather than widening that module's visibility — this
/// is the only consumer in `cb.rs`.
fn project_label_for_wi(issue: &crate::issues::Issue) -> Option<&str> {
    issue.labels.iter().find_map(|label| {
        let project = label.strip_prefix("project:")?.trim();
        (!project.is_empty()).then_some(project)
    })
}

/// Touched-scope standardization gate (issue #932, the 正流程 forward loop
/// enforcing 標準化): for this WI's own touched-file set — the branch diff
/// against base unioned with the WI's TD `## Changes` paths
/// (`cb_fill::resolve_touched_scope`, the same union the marker gate above
/// consumes) — require every in-scope touched file to carry a CODEGEN or
/// HANDWRITE marker, and every HANDWRITE marker in a touched file to have
/// valid gap/tracker/reason attrs
/// (`standardize::project_touched_scope_standardization`).
///
/// Activation policy: fail-mode only applies once the *rest* of the
/// project's managed inventory (excluding this WI's own touched files) is
/// already at 100% coverage — i.e. this WI would be introducing a fresh
/// regression into an otherwise-standardized project. Below that baseline
/// (a project still mid-標準化 bootstrap), the same violation is
/// warn-only: `Some` is never returned, and a remediation note is emitted
/// to stderr instead so the bootstrap loop is not blocked by pre-existing
/// debt this WI didn't create.
///
/// Vacuous-passes (returns `None`, no stderr warning) when: the issue
/// carries no `project:<name>` label (nothing configured to check against
/// — every pre-#932 WI fixture and any WI created outside `aw wi create
/// --project` falls here); the touched-file set is empty (docs-only WI, or
/// unresolvable branch diff — same rationale as the marker gate's vacuous
/// pass); or the owning project's standardize inventory can't be resolved
/// (e.g. no `.aw/config.toml` workspace scope configured for it) — an
/// unconfigured project must never brick code-check.
fn touched_scope_standardization_gate_message(
    project_root: &std::path::Path,
    issue: &crate::issues::Issue,
    marker_gate_scope: &[String],
) -> Option<String> {
    let project = project_label_for_wi(issue)?;
    let touched = crate::cli::cb_fill::resolve_touched_scope(project_root, marker_gate_scope);
    if touched.is_empty() {
        return None;
    }
    let verdict =
        crate::cli::standardize::project_touched_scope_standardization(project, &touched).ok()?;
    if verdict.unmarked.is_empty() && verdict.attr_gap.is_empty() {
        return None;
    }

    let mut lines = Vec::new();
    for path in &verdict.unmarked {
        lines.push(format!(
            "  - {path}: no CODEGEN/HANDWRITE marker — add one (`// SPEC-MANAGED: <mirror>.md#source` \
             + CODEGEN-BEGIN/END, or HANDWRITE-BEGIN gap=\"...\" tracker=\"...\" reason=\"...\")"
        ));
    }
    for path in &verdict.attr_gap {
        lines.push(format!(
            "  - {path}: HANDWRITE marker missing required gap/tracker attrs — fill them in, \
             or `aw td promote` once the gap-blocker is closed"
        ));
    }
    let message = format!(
        "td code-check touched-scope standardization: {} touched file(s) in project '{}' are not \
         standardized:\n{}",
        lines.len(),
        project,
        lines.join("\n")
    );

    if verdict.baseline_percent >= 100.0 {
        Some(message)
    } else {
        eprintln!(
            "[td code-check] WARNING: {message}\n(project '{project}' managed coverage is below \
             100% excluding this WI's touched files — warning only, not blocking)"
        );
        None
    }
}

/// Empty-implementation gate (issue #847, restoring the removed `aw td
/// merge` "Bug 2" guard byte-for-byte in condition). Scoped to `spec_paths`
/// — the completing slug's own TD spec file(s), resolved by
/// [`resolve_slug_spec_paths`] — instead of walking every `.md` file under
/// the project's tech-design root (issue #854; the removed `run_merge`'s
/// project-wide walk let an unrelated stale spec's 0-of-N signature block a
/// clean slug's code-check). Sums each spec's `action: create`/`modify`
/// Changes entries via [`crate::generate::apply::extract_change_entries_count`]
/// plus the subset missing from disk via
/// [`crate::generate::apply::missing_implementation_paths`].
///
/// Returns `Some(message)` (block) only when the *entire* promised
/// implementation across `spec_paths` is missing (0-of-N,
/// `total_missing == entries_total`, with `entries_total > 0`) — the
/// signature of gen-code having been skipped entirely. Partial presence
/// (some but not all missing) is warn-only to stderr and returns `None`, as
/// does the no-entries case — including when `spec_paths` itself is empty
/// (nothing to scope against, see [`resolve_slug_spec_paths`]).
fn empty_implementation_gate_message(
    project_root: &std::path::Path,
    slug: &str,
    spec_paths: &[std::path::PathBuf],
) -> Option<String> {
    let mut missing_total: Vec<(std::path::PathBuf, Vec<String>)> = Vec::new();
    let mut entries_total = 0usize;
    for spec_abs in spec_paths {
        let Ok(content) = std::fs::read_to_string(spec_abs) else {
            continue;
        };
        let total = crate::generate::apply::extract_change_entries_count(&content);
        entries_total += total;
        let missing = crate::generate::apply::missing_implementation_paths(&content, project_root);
        if !missing.is_empty() {
            missing_total.push((spec_abs.clone(), missing));
        }
    }
    if entries_total == 0 || missing_total.is_empty() {
        return None;
    }
    let total_missing: usize = missing_total.iter().map(|(_, m)| m.len()).sum();
    let block = total_missing == entries_total;
    let mut preview: Vec<String> = Vec::new();
    for (spec, m) in missing_total.iter().take(3) {
        let spec_rel = spec
            .strip_prefix(project_root)
            .unwrap_or(spec)
            .display()
            .to_string();
        for p in m.iter().take(3) {
            preview.push(format!("    {} \u{2192} missing {}", spec_rel, p));
        }
    }
    if block {
        Some(format!(
            "refusing to complete code-check: spec lists {} file(s) but {} are missing on disk \
             (codegen likely skipped; run `aw td gen {}` then implement, \
             or pass --allow-empty-impl for spec-only completions).\n{}",
            entries_total,
            total_missing,
            slug,
            preview.join("\n"),
        ))
    } else {
        eprintln!(
            "[td code-check] WARNING: {} of {} spec-listed files missing on disk:",
            total_missing, entries_total,
        );
        for line in &preview {
            eprintln!("{}", line);
        }
        None
    }
}

/// Outcome of [`land_td_lifecycle_branch`] — reported in the terminal `done`
/// envelope's `landing` field so `aw run` output is auditable (issue #842).
#[derive(Debug, Clone, PartialEq, Eq)]
enum BranchLandingOutcome {
    /// No local `td-<slug>` branch exists — an in-place/off-main lifecycle
    /// never creates one (`td-<slug>` is only provisioned when the
    /// lifecycle launched from `main`), so there is nothing to land.
    NoBranch,
    /// `td-<slug>` existed but its tip was already an ancestor of the
    /// landing target (a previous run already merged it, or it was landed
    /// by other means) — the stale branch was deleted and nothing else
    /// happened.
    AlreadyMerged { branch: String, target: String },
    /// `td-<slug>` was merged into the landing target with a `--no-ff`
    /// commit carrying `Lifecycle-Slug`/`Work-Item` trailers, then deleted.
    /// HEAD ends on `target`.
    Landed { branch: String, target: String },
}

/// Land the `td-<slug>` lifecycle branch (issue #842): resolve the branch's
/// launch target via `merge_target::resolve_merge_target`, merge with
/// `--no-ff` (mirroring the removed `run_merge` behaviour so the merge
/// commit carries lifecycle trailers), and delete the branch. Returns
/// `Ok(BranchLandingOutcome::NoBranch)` immediately when `td-<slug>` does
/// not exist locally — the common case for in-place/off-main lifecycles.
///
/// Errors (dirty tree, unresolved merge conflict, target resolution
/// failure, self-referencing target) are returned as `Err` for the caller
/// to surface as a `td code-check landing failed` error envelope; the
/// dirty-tree check here is a minimal guard naming the offending paths —
/// full dirty-tree semantics is issue #807's scope.
fn land_td_lifecycle_branch(
    project_root: &std::path::Path,
    slug: &str,
    closed_issue: &crate::issues::Issue,
) -> Result<BranchLandingOutcome> {
    let td_branch = format!("td-{}", slug);
    if !crate::branch_switch::branch_exists_local(project_root, &td_branch)? {
        return Ok(BranchLandingOutcome::NoBranch);
    }

    if let Err(e) = crate::branch_switch::ensure_branch_clean(project_root) {
        anyhow::bail!(
            "cannot land '{}': working tree is not clean: {}",
            td_branch,
            e
        );
    }

    // `td-<slug>` is only ever created when the lifecycle launched from
    // `main` (`should_use_td_branch` in td.rs), and every TD/CB verb since
    // then stays on it — so if we're currently sitting on `td-<slug>`
    // itself, `resolve_merge_target`'s current-branch detection (step 3)
    // would self-reference. Feed the deterministic "main" default through
    // as the frontmatter fallback in that case; an explicit
    // `issue.target_branch` override, if ever set, still wins.
    let current = crate::branch_switch::current_branch(project_root)?;
    let frontmatter_branch = closed_issue.target_branch.clone().or_else(|| {
        if current == td_branch {
            Some("main".to_string())
        } else {
            None
        }
    });
    let target =
        crate::cli::merge_target::resolve_merge_target(None, frontmatter_branch, project_root)
            .map_err(|e| anyhow::anyhow!("resolving landing target for '{}': {}", td_branch, e))?;
    if target == td_branch {
        anyhow::bail!(
            "landing target for '{}' resolved to the lifecycle branch itself; refusing to merge a branch into itself",
            td_branch
        );
    }

    crate::branch_switch::switch_or_create_branch(project_root, &target, &target)
        .map_err(|e| anyhow::anyhow!("checking out landing target '{}': {}", target, e))?;

    let git_bin = crate::git::find_git_bin()
        .ok_or_else(|| anyhow::anyhow!("git binary not found on PATH"))?;
    let already_merged = std::process::Command::new(&git_bin)
        .arg("-C")
        .arg(project_root)
        .args(["merge-base", "--is-ancestor", &td_branch, &target])
        .status()
        .context("git merge-base --is-ancestor")?
        .success();
    if already_merged {
        crate::branch_switch::delete_local_branch(project_root, &td_branch)?;
        return Ok(BranchLandingOutcome::AlreadyMerged {
            branch: td_branch,
            target,
        });
    }

    let msg = format!(
        "Merge {} into {} (td code-check)\n\n\
         Lifecycle-Slug: {}\n\
         Work-Item: {}",
        td_branch, target, slug, slug,
    );
    let merge = std::process::Command::new(&git_bin)
        .arg("-C")
        .arg(project_root)
        .args(["merge", "--no-ff", "-m", &msg, &td_branch])
        .output()
        .context("git merge --no-ff")?;
    if !merge.status.success() {
        anyhow::bail!(
            "merge conflict landing '{}' into '{}': {}",
            td_branch,
            target,
            String::from_utf8_lossy(&merge.stderr).trim()
        );
    }

    crate::branch_switch::delete_local_branch(project_root, &td_branch)?;
    Ok(BranchLandingOutcome::Landed {
        branch: td_branch,
        target,
    })
}

/// True if the worktree git log already has a terminal commit for `slug` —
/// one whose message has an exact-line `Lifecycle-Slug: <slug>` AND an
/// exact-line `Lifecycle-Stage: Cb-CodeCheck` (accepting the legacy
/// `Td-Merged` alias via [`lifecycle_trailer::normalize`]). Used by the
/// resumable retry path above to avoid re-committing a duplicate trailer
/// when only the commit step failed on a prior attempt.
///
/// Shares the same exact-line matchers
/// ([`lifecycle_trailer::body_has_slug_trailer`] /
/// [`lifecycle_trailer::body_has_stage_trailer`]) as td.rs's
/// `find_ship_commit_from_log` backfill scan, instead of a second
/// hand-rolled per-line loop (issue #856c).
fn terminal_commit_already_landed(project_root: &std::path::Path, slug: &str) -> Result<bool> {
    use crate::issues::types::lifecycle_trailer;

    let Some(git_bin) = crate::git::find_git_bin() else {
        // No git binary: can't check either way. Treat as "not yet
        // committed" so the commit attempt below runs and surfaces the same
        // "git binary not found" error the fresh-entry path would.
        return Ok(false);
    };
    let slug_line = format!("Lifecycle-Slug: {}", slug);
    let output = std::process::Command::new(&git_bin)
        .arg("-C")
        .arg(project_root)
        .args([
            "log",
            "--format=%B%x1e",
            "--all",
            "--fixed-strings",
            "--grep",
            &slug_line,
        ])
        .output()
        .context("git log failed")?;
    if !output.status.success() {
        return Ok(false);
    }
    let stdout = String::from_utf8_lossy(&output.stdout);
    for entry in stdout.split('\x1e') {
        if lifecycle_trailer::body_has_slug_trailer(entry, slug)
            && lifecycle_trailer::body_has_stage_trailer(entry, lifecycle_trailer::CB_CODE_CHECK)
        {
            return Ok(true);
        }
    }
    Ok(false)
}

/// True if the worktree git log has ANY commit with an exact-line
/// `Lifecycle-Slug: <slug>` trailer, regardless of lifecycle stage. Used by
/// `run_check_lifecycle_terminal` (issue #859 part c) to distinguish "this
/// slug went through the aw lifecycle on this worktree before, its local
/// issue cache was just cleared" (rehydration is the right remediation)
/// from "this slug was never a real work-item here" (a plain not-found is
/// the right remediation) when the local issue is missing. Deliberately
/// read-only and side-effect-free, unlike `td::bootstrap_td_issue`.
fn slug_has_lifecycle_history(project_root: &std::path::Path, slug: &str) -> Result<bool> {
    let Some(git_bin) = crate::git::find_git_bin() else {
        return Ok(false);
    };
    let slug_line = format!("Lifecycle-Slug: {}", slug);
    let output = std::process::Command::new(&git_bin)
        .arg("-C")
        .arg(project_root)
        .args([
            "log",
            "--format=%B%x1e",
            "--all",
            "--fixed-strings",
            "--grep",
            &slug_line,
        ])
        .output()
        .context("git log failed")?;
    if !output.status.success() {
        return Ok(false);
    }
    let stdout = String::from_utf8_lossy(&output.stdout);
    for entry in stdout.split('\x1e') {
        if entry.lines().any(|line| line.trim_end() == slug_line) {
            return Ok(true);
        }
    }
    Ok(false)
}

fn commit_cb_code_check_terminal(
    project_root: &std::path::Path,
    slug: &str,
    issue_path: &std::path::Path,
) -> Result<()> {
    // Issue #856b: reuse td.rs's shared staging + commit-invocation core
    // (`commit_lifecycle_message` / `stage_lifecycle_paths`) instead of a
    // second hand-rolled `git add` + `git commit` implementation. The
    // subject line and trailer schema stay exactly as before — only
    // `find_ship_commit_from_log` / `terminal_commit_already_landed` grep
    // the `Lifecycle-Slug:` / `Lifecycle-Stage:` trailer lines, not the
    // subject — and `--allow-empty` is unconditional here (unlike
    // `commit_lifecycle`'s "only when nothing staged" default): the
    // terminal `Cb-CodeCheck` commit is frequently trailer-only, since the
    // phase advance + label update already landed via a separate
    // `backend.update` write.
    let issue_path_s = issue_path.to_string_lossy();
    let msg = format!(
        "cb({slug}) - code-check passed\n\n\
         Lifecycle-Slug: {slug}\n\
         Work-Item: {slug}\n\
         Lifecycle-Stage: {}",
        crate::issues::types::lifecycle_trailer::CB_CODE_CHECK,
    );
    td::commit_lifecycle_message(
        project_root,
        &[issue_path_s.as_ref()],
        &msg,
        td::LifecycleCommitEmpty::Always,
    )
}

// ── cb claim ────────────────────────────────────────────────────────

// Implementation of `aw td code-claim <code-path>` — recovery verb.
///
// Wraps the fillback pipeline to adopt existing code into the score lifecycle
// in the current checkout.
// @spec projects/agentic-workflow/tech-design/surface/interfaces/src/cb.md#source
pub async fn run_claim(args: CbClaimArgs) -> Result<()> {
    use std::path::PathBuf;

    // 1. Validate code-path exists.
    let code_path = PathBuf::from(&args.code_path);
    if !code_path.exists() {
        let env = serde_json::json!({
            "action": "error",
            "message": format!("code-path not found: {}", args.code_path),
        });
        println!("{}", serde_json::to_string_pretty(&env)?);
        std::process::exit(1);
    }

    // 2. .aw/ presence + --init handling.
    let cwd = std::env::current_dir().context("failed to read cwd")?;
    let project_root = match crate::find_project_root() {
        Ok(p) => p,
        Err(_) => cwd.clone(),
    };
    let score_dir = project_root.join(".aw");
    if !score_dir.exists() {
        if args.init {
            std::fs::create_dir_all(score_dir.join("tech-design"))
                .context("failed to create .aw/tech-design")?;
        } else {
            let env = serde_json::json!({
                "action": "error",
                "message": ".aw/ workspace not found; pass --init to create it",
            });
            println!("{}", serde_json::to_string_pretty(&env)?);
            std::process::exit(1);
        }
    }

    // 3. Run the fillback pipeline. We call the existing `fillback::run`
    //    directly (rather than extracting a `run_core`) because the existing
    //    function already takes `Option<&str>` parameters that match the
    //    flags we expose. This is the simpler-alternative documented in
    //    the spec changes for fillback.rs.
    //
    // When `--non-interactive` is set we export `SCORE_NON_INTERACTIVE=1`
    // so the AST-strategy clarification + overwrite prompts skip with safe
    // defaults. (`std::io::stdin().is_terminal()` already covers piped
    // stdin; this env var is the explicit-override channel.)
    // @spec projects/agentic-workflow/tech-design/surface/specs/score-recovery-verbs-non-interactive.md#logic
    if args.non_interactive {
        std::env::set_var("SCORE_NON_INTERACTIVE", "1");
    }
    let path_str = args.code_path.clone();
    if let Err(e) = crate::cli::fillback::run(Some(&path_str), None, false).await {
        let env = serde_json::json!({
            "action": "error",
            "message": format!("fillback pipeline failed: {}", e),
        });
        println!("{}", serde_json::to_string_pretty(&env)?);
        std::process::exit(1);
    }

    // 4. Tracker linkage (default-on; issue #925). Adopted code needs a
    //    durable tracker root for traceability closure — file (or reuse) a
    //    real work-item through the same `aw wi create` routing issue #919
    //    established for `standardize::ensure_gap_issue`, instead of a
    //    `LocalBackend`-only stub. `--no-issue` is the documented opt-out.
    //    Best-effort either way: a skipped or failed tracker link must
    //    never fail the claim itself (`aw td code-claim` has to keep
    //    working offline / with no issue backend configured).
    let derived_slug = derive_slug_from_path(&code_path);
    let code_path_rel = repo_relative_code_path(&project_root, &args.code_path);
    let claim_issue = if args.no_issue {
        eprintln!(
            "note: --no-issue set; skipping tracker-issue creation for adopted code at {}",
            args.code_path
        );
        None
    } else {
        match ensure_claim_issue(&project_root, &code_path_rel) {
            Ok(issue_ref) => Some(issue_ref),
            Err(e) => {
                eprintln!(
                    "warning: failed to create/link a tracker issue for adopted code at {} \
                     (offline or issue backend unconfigured?): {}",
                    args.code_path, e
                );
                None
            }
        }
    };

    // 5. Commit a Cb-Claim trailer in the current checkout when possible.
    let mut committed = false;
    if let Err(e) = commit_cb_claim_trailer(
        &project_root,
        &derived_slug,
        &args.code_path,
        claim_issue.as_ref(),
    ) {
        eprintln!("warning: failed to commit Cb-Claim trailer: {}", e);
    } else {
        committed = true;
    }

    // 6. Emit result envelope.
    let env = serde_json::json!({
        "action": "done",
        "slug": derived_slug,
        "claim_issue": claim_issue.as_ref().map(|r| r.trailer_value()),
        "message": if committed {
            "td code-claim: spec written; Cb-Claim trailer committed"
        } else {
            "td code-claim: spec written (no trailer committed)"
        },
    });
    println!("{}", serde_json::to_string_pretty(&env)?);
    let _ = args.json;
    let _ = args.group; // group inference handled by fillback's output_dir wiring
    Ok(())
}

// Derive a kebab-case slug from a code path.
fn derive_slug_from_path(p: &std::path::Path) -> String {
    p.file_stem()
        .and_then(|s| s.to_str())
        .map(|s| s.to_lowercase().replace('_', "-"))
        .unwrap_or_else(|| "claim".to_string())
}

// Best-effort project-root-relative form of a `code-claim` path argument,
// for `configured_project_name_for_path` matching (that helper expects a
// forward-slash path rooted at `project_root`, matching its other call
// sites — see `write_project_root_llms_targets` above). `aw td code-claim`
// is invoked with a path relative to the current checkout root in normal
// use (the same convention `derive_slug_from_path` already assumes); this
// only adjusts for a leading `./` or an absolute path that happens to live
// under `project_root`.
fn repo_relative_code_path(project_root: &std::path::Path, code_path: &str) -> String {
    let candidate = std::path::Path::new(code_path);
    let rel = if candidate.is_absolute() {
        candidate
            .strip_prefix(project_root)
            .map(|p| p.to_path_buf())
            .unwrap_or_else(|_| candidate.to_path_buf())
    } else {
        candidate.to_path_buf()
    };
    let mut rel = rel.to_string_lossy().replace('\\', "/");
    while let Some(stripped) = rel.strip_prefix("./") {
        rel = stripped.to_string();
    }
    rel
}

// ── cb claim: tracker linkage (issue #925) ─────────────────────────

// Guards the process-wide cwd while `ensure_claim_issue` drives
// `crate::cli::issues::run` — that entry point resolves its project root
// from `std::env::current_dir()`, not a parameter. Mirrors
// `standardize.rs`'s own `CwdGuard` (issue #919); duplicated here rather
// than reused because that struct is file-private to `standardize.rs` and
// this module already owns its own cwd-guarding convention, matching the
// existing per-module `CWD_LOCK` pattern in `cli/mod.rs` and
// `cli/guard.rs`. Restores the previous cwd on drop, including on
// error/panic unwind.
struct CwdGuard {
    prev: std::path::PathBuf,
    _lock: std::sync::MutexGuard<'static, ()>,
}

impl CwdGuard {
    fn enter(dir: &std::path::Path) -> Result<Self> {
        static LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());
        let lock = LOCK.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
        let prev = std::env::current_dir().context("failed to read current directory")?;
        std::env::set_current_dir(dir)
            .with_context(|| format!("failed to switch cwd to {}", dir.display()))?;
        Ok(Self { prev, _lock: lock })
    }
}

impl Drop for CwdGuard {
    fn drop(&mut self) {
        let _ = std::env::set_current_dir(&self.prev);
    }
}

// Run an async future to completion from sync code that may or may not
// already be inside a tokio runtime (production runs inside `aw`'s
// top-level multi-thread runtime; unit tests call in directly with none).
// Mirrors `standardize.rs`'s `block_on_bridge` (issue #919) for the same
// duplication reason documented on `CwdGuard` above.
fn block_on_bridge<F: std::future::Future>(fut: F) -> F::Output {
    match tokio::runtime::Handle::try_current() {
        Ok(handle) => tokio::task::block_in_place(|| handle.block_on(fut)),
        Err(_) => {
            let rt = tokio::runtime::Runtime::new().expect("failed to create tokio runtime");
            rt.block_on(fut)
        }
    }
}

// Durable tracker reference returned by `ensure_claim_issue` — the created
// (or matched) work-item's slug, plus, when the resolved backend is a
// remote tracker, its published issue number for the commit trailer.
#[derive(Debug)]
struct ClaimIssueRef {
    slug: String,
    number: Option<u64>,
}

impl ClaimIssueRef {
    // The value to place on a `Claim-Issue:` trailer: the published issue
    // number when the backend is remote (`#42`), else the work-item slug
    // (local-backend fallback — see the durability note on
    // `ensure_claim_issue`).
    fn trailer_value(&self) -> String {
        match self.number {
            Some(n) => format!("#{n}"),
            None => self.slug.clone(),
        }
    }
}

// File (or reuse, if one already exists for the same title) a durable
// tracker work-item for code adopted via `aw td code-claim`, routed
// through the real `aw wi create` path (`crate::cli::issues::run`)
// instead of a `LocalBackend`-only stub — the same routing issue #919
// established for `standardize::ensure_gap_issue`. Tracker linkage is
// default-on (see `CbClaimArgs::no_issue` for the opt-out); this function
// is best-effort — the caller (`run_claim`) warns and proceeds on `Err`,
// never fails the claim, since `aw td code-claim` has to keep working
// offline / with no issue backend configured.
//
// `--type refactor`, not `enhancement`: adopting already-written code into
// the spec-driven regenerability lifecycle is maintenance/tech-debt work
// on existing code, not a new user-facing capability — matching
// `IssueType::Refactor`'s "maintenance" bucket in `issues.rs`'s planning
// groups (`groups.maintenance.push(issue)`), as distinct from
// `ensure_gap_issue`'s `enhancement` (a genuine HANDWRITE-coverage gap).
//
// Backend/durability note: this lands in whatever backend
// `.aw/config.toml` resolves for the project that owns `code_path_rel`
// (local → ephemeral `/tmp/aw/...`, exactly like plain `aw wi create`;
// github/gitlab → a durable tracker issue). This function only fixes
// code-claim's routing to the real create path — a marker/attr surface to
// carry the tracker id back onto the adopted source doesn't exist yet
// (the fillback pipeline that writes the spec for adopted code does not
// wrap it in a HANDWRITE marker), so the published id is attached to the
// `Cb-Claim` commit trailer instead (`commit_cb_claim_trailer`); the #932
// gate is expected to pick this up once a marker surface exists.
fn ensure_claim_issue(
    project_root: &std::path::Path,
    code_path_rel: &str,
) -> Result<ClaimIssueRef> {
    let project_name =
        crate::cli::standardize::configured_project_name_for_path(project_root, code_path_rel)?
            .with_context(|| {
                format!(
            "no configured project owns `{code_path_rel}` — cannot file a code-claim tracker issue"
        )
            })?;
    // Pre-resolve the exact `--project` label lookup `aw wi create` performs
    // so a misconfigured project surfaces as a normal `Result::Err` here
    // instead of `run_create`'s own validation-failure path, which hard
    // `std::process::exit`s regardless of `--json` (see
    // `claim_issue_create_args`).
    crate::cli::issues::resolve_project_label(project_root, &project_name).map_err(|e| {
        anyhow::anyhow!(
            "cannot file a code-claim tracker issue for `{code_path_rel}`: {}",
            e.to_envelope_message()
        )
    })?;

    let title = claim_issue_title(code_path_rel);

    let find_existing = |title: &str| {
        let title = title.to_string();
        block_on_bridge(async move {
            let (kind, repo, host) = crate::issues::resolve_default_backend(project_root)?;
            let backend = crate::issues::make_backend(&kind, project_root, repo, host)?;
            let filter = crate::issues::IssueFilter {
                state: None,
                issue_type: Some(crate::issues::IssueType::Refactor),
                label: None,
                author: None,
            };
            let issues = backend.list(&filter).await?;
            Ok::<_, anyhow::Error>(
                issues
                    .into_iter()
                    .filter(|issue| issue.title == title)
                    .max_by(|a, b| a.created_at.cmp(&b.created_at)),
            )
        })
    };

    // Idempotent by title: a re-run of `code-claim` over the same path
    // reuses the already-filed work-item instead of filing a duplicate.
    if let Some(existing) = find_existing(&title)? {
        return Ok(ClaimIssueRef {
            number: existing.github_id.or(existing.gitlab_id),
            slug: existing.slug,
        });
    }

    let issues_args = crate::cli::issues::IssuesArgs {
        command: crate::cli::issues::IssuesCommand::Create(claim_issue_create_args(
            &title,
            project_name,
        )),
    };
    {
        let _cwd = CwdGuard::enter(project_root)?;
        block_on_bridge(crate::cli::issues::run(issues_args))
            .with_context(|| format!("aw wi create failed for code-claim `{code_path_rel}`"))?;
    }

    let created = find_existing(&title)?
        .context("aw wi create reported success but the work-item was not found via list")?;

    Ok(ClaimIssueRef {
        number: created.github_id.or(created.gitlab_id),
        slug: created.slug,
    })
}

// The stable, deterministic title `ensure_claim_issue` files (and
// re-finds) an adopted-code tracker work-item under.
fn claim_issue_title(code_path_rel: &str) -> String {
    format!("Adopted (td code-claim): {code_path_rel}")
}

// Build the exact `CreateArgs` `ensure_claim_issue` hands to the real
// `aw wi create` internal path (`crate::cli::issues::run`) for adopted
// code — typed `--type refactor` + `--project <project_name>` fields, no
// free-form `--body` (so `run_create`'s own canonical structured skeleton
// is what gets filed). Split out from `ensure_claim_issue` so the
// field/skeleton shape is unit-testable without a configured issue
// backend — mirrors `standardize::gap_issue_create_args` (issue #919).
fn claim_issue_create_args(title: &str, project_name: String) -> crate::cli::issues::CreateArgs {
    crate::cli::issues::CreateArgs {
        draft_path: None,
        title: Some(title.to_string()),
        issue_type: Some(crate::cli::issues::TypeFilter::Refactor),
        body: None,
        body_file: None,
        projects: vec![project_name],
        priority: None,
        agent: None,
        remote: false,
        // `false`, not the CLI default `true`: with `json: true`,
        // `run_create`'s validation-failure and remote-backend-create-failure
        // branches hard `std::process::exit` instead of returning
        // `Result::Err`. This call site runs deep inside `aw td code-claim`,
        // not as a direct CLI invocation, and must stay recoverable so a
        // tracker-linkage failure only warns (see `run_claim`) instead of
        // crashing the whole claim.
        json: false,
        repo: None,
    }
}

// Commit a `Lifecycle-Stage: Cb-Claim` trailer in the current checkout.
// Best-effort: a missing git binary or non-git tree returns Err and the
// caller logs a warning. `claim_issue`, when set, carries the published
// tracker issue (issue #925) as a `Claim-Issue:` trailer.
fn commit_cb_claim_trailer(
    checkout_root: &std::path::Path,
    slug: &str,
    code_path: &str,
    claim_issue: Option<&ClaimIssueRef>,
) -> Result<()> {
    let git_bin = crate::git::find_git_bin()
        .ok_or_else(|| anyhow::anyhow!("git binary not found on PATH"))?;
    let _ = std::process::Command::new(&git_bin)
        .arg("-C")
        .arg(checkout_root)
        .args(["add", "-A"])
        .output()?;
    let mut msg = format!(
        "cb({slug}) \u{2014} adopted code at {code_path}\n\n\
         Lifecycle-Slug: {slug}\n\
         Work-Item: {slug}\n\
         Lifecycle-Stage: Cb-Claim\n\
         Claim-Source: {code_path}\n\
         Claim-Type: cb-code"
    );
    if let Some(issue) = claim_issue {
        msg.push_str(&format!("\nClaim-Issue: {}", issue.trailer_value()));
    }
    let commit = std::process::Command::new(&git_bin)
        .arg("-C")
        .arg(checkout_root)
        .args(["commit", "--allow-empty", "-m", &msg])
        .output()?;
    if !commit.status.success() {
        anyhow::bail!(
            "git commit failed: {}",
            String::from_utf8_lossy(&commit.stderr).trim()
        );
    }
    Ok(())
}

// CODEGEN-END
