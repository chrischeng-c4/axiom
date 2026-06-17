// SPEC-MANAGED: projects/agentic-workflow/tech-design/surface/interfaces/src/cb.md#source
// CODEGEN-BEGIN
//! `aw cb` CLI — code-artifact workflow verbs.
//!
//! `cb` is the canonical namespace for code generation, code checks, and
//! HANDWRITE marker fill/review flows. The lifecycle phase written by `cb gen`
//! is `cb_genned` and the canonical `Lifecycle-Stage:` trailer is `Cb-Gen`.
//!
//! @spec projects/agentic-workflow/tech-design/surface/specs/score-namespaces.md#changes

use anyhow::{Context, Result};
use clap::{Args, Subcommand};
use serde::Serialize;
use std::collections::{BTreeMap, BTreeSet};

use crate::cli::td::{self, AuditArgs, AuditGroupBy, GenCodeArgs};

const AW_EC_BEGIN_MARKER: &str = "AW-EC-BEGIN";

// Top-level args group for `aw cb ...`.
///
// @spec projects/agentic-workflow/tech-design/surface/specs/score-namespaces.md#changes
#[derive(Debug, Args)]
pub struct CbArgs {
    #[command(subcommand)]
    pub command: CbCommand,
}

// `aw cb` subcommands.
///
// @spec projects/agentic-workflow/tech-design/surface/specs/score-namespaces.md#changes
#[derive(Debug, Subcommand)]
pub enum CbCommand {
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
    // Brief mode: lock the next marker payload in WI projection.
    // `--apply --marker <id>` mode: merge and commit one marker, then
    // lock the next marker or dispatch `aw cb check`.
    // @spec projects/agentic-workflow/tech-design/surface/specs/score-cb-fill-workflow.md#cli
    Fill(CbFillArgs),
    // Review filled HANDWRITE markers (Phase 3 CRRR step 3 of 4).
    // @spec projects/agentic-workflow/tech-design/surface/specs/score-cb-review-revise-crrr.md#cli
    Review(crate::cli::cb_review::CbReviewArgs),
    // Revise flagged markers after `cb review` returns needs-revision
    // (Phase 3 CRRR step 4 of 4).
    // @spec projects/agentic-workflow/tech-design/surface/specs/score-cb-review-revise-crrr.md#cli
    Revise(crate::cli::cb_revise::CbReviseArgs),
    // Terminal escalation after the CB CRRR 2-review ceiling is exceeded.
    // Advances phase to `cb_arbitrated`, commits `Lifecycle-Stage: Cb-Arbitrate`,
    // and prints human guidance for force-merge vs send-back.
    // @spec projects/agentic-workflow/tech-design/surface/specs/score-cb-review-revise-crrr.md#cli
    Arbitrate(crate::cli::cb_arbitrate::CbArbitrateArgs),
}

// Args for `aw cb fill <slug>` — Phase 3 marker-fill workflow.
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
    // Merge mode. When set, --marker is required. Merges
    // `.aw/payloads/<slug>/<marker>.md` into the matching
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
    // Skip the cb review step after marker fill completes; dispatch
    // `aw td merge` directly. Backward-compat path for callers that
    // don't yet need the CB CRRR loop.
    // @spec projects/agentic-workflow/tech-design/surface/specs/score-cb-review-revise-crrr.md#cli
    #[arg(long)]
    pub no_review: bool,
}

// Args for `aw cb claim <code-path>`.
#[derive(Debug, Args)]
// @spec projects/agentic-workflow/tech-design/surface/interfaces/src/cb.md#source
pub struct CbClaimArgs {
    // Path to a source file or directory to analyse.
    pub code_path: String,
    // Create `.aw/` workspace directory if it does not already exist.
    #[arg(long)]
    pub init: bool,
    // Create a minimal issue stub in the temp issue working copy using the
    // derived slug inferred from the code path.
    #[arg(long)]
    pub issue_stub: bool,
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

// Args for `aw cb gen <slug>` or
// `aw cb gen --force-regen --project <project>`.
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

// Args for `aw cb check <target>`.
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
}

// Dispatch for `aw cb ...`.
///
// @spec projects/agentic-workflow/tech-design/surface/specs/score-namespaces.md#changes
pub async fn run(args: CbArgs) -> Result<()> {
    let project_root = crate::find_project_root()?;
    match &args.command {
        CbCommand::Check(_) => {}
        CbCommand::GenSource(_) => {}
        CbCommand::Gen(a) => {
            if let Some(slug) = a.slug.as_deref() {
                crate::cli::workflow_guard::guard_issue_mutation(&project_root, Some(("cb", slug)))
                    .await?;
            } else {
                crate::cli::workflow_guard::guard_issue_mutation(&project_root, None).await?;
            }
        }
        CbCommand::Fill(a) => {
            crate::cli::workflow_guard::guard_issue_mutation(&project_root, Some(("cb", &a.slug)))
                .await?;
        }
        CbCommand::Review(a) => {
            crate::cli::workflow_guard::guard_issue_mutation(&project_root, Some(("cb", &a.slug)))
                .await?;
        }
        CbCommand::Revise(a) => {
            crate::cli::workflow_guard::guard_issue_mutation(&project_root, Some(("cb", &a.slug)))
                .await?;
        }
        CbCommand::Arbitrate(a) => {
            crate::cli::workflow_guard::guard_issue_mutation(&project_root, Some(("cb", &a.slug)))
                .await?;
        }
        CbCommand::Claim(_) => {
            crate::cli::workflow_guard::guard_issue_mutation(&project_root, None).await?;
        }
    }
    match args.command {
        CbCommand::Gen(a) => run_gen(a).await,
        CbCommand::GenSource(a) => run_gen_source(a),
        CbCommand::Check(a) => run_check(a),
        CbCommand::Claim(a) => run_claim(a).await,
        CbCommand::Fill(a) => crate::cli::cb_fill::run(a).await,
        CbCommand::Review(a) => crate::cli::cb_review::run_review(a).await,
        CbCommand::Revise(a) => crate::cli::cb_revise::run_revise(a).await,
        CbCommand::Arbitrate(a) => crate::cli::cb_arbitrate::run_arbitrate(a).await,
    }
}

// Args for `aw cb gen-source --spec <td> --target <rs>`.
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
// item-tree regeneration). The forward inverse of `cb gen --force-regen`
// (which syncs TD<-source); this writes source<-TD.
fn run_gen_source(args: CbGenSourceArgs) -> Result<()> {
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

// Implementation of `aw cb gen`.
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
            return run_force_regen_verify_cold(project, args.workspace.as_deref());
        }
        if args.semantic_sample.is_some() {
            anyhow::bail!("--semantic-sample is only supported with --verify");
        }
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
        "cb gen --force-regen --project {}{}: {} spec(s) from {}, {} file update(s), {} created, {} CODEGEN block(s), {} public API manifest update(s){}",
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
            "cb gen --force-regen --project {project}{} --verify failed: {} file(s) differ after TD replay:\n{}",
            workspace
                .map(|name| format!(" --workspace {name}"))
                .unwrap_or_default(),
            mismatches.len(),
            mismatches.join("\n")
        );
    }
    if !conformance.failures.is_empty() {
        anyhow::bail!(
            "cb gen --force-regen --project {project}{} --verify failed deterministic conformance: {} finding(s):\n{}",
            workspace
                .map(|name| format!(" --workspace {name}"))
                .unwrap_or_default(),
            conformance.failures.len(),
            conformance.failures.join("\n")
        );
    }

    println!(
        "cb gen --force-regen --project {}{} --verify: {} spec(s), {} source root(s), byte-equivalent after TD replay",
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
            "cb gen --force-regen --project {project}{} --verify-cold failed: {} finding(s):\n{}",
            workspace
                .map(|name| format!(" --workspace {name}"))
                .unwrap_or_default(),
            failures.len(),
            failures.join("\n")
        );
    }

    println!(
        "cb gen --force-regen --project {}{} --verify-cold: {} spec(s), {} source root(s), rebuilt expected targets from TD only",
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
// `cb gen --force-regen --verify` without printing the verbose CLI report.
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
        anyhow::bail!("cb gen --force-regen requires .aw/config.toml project routing");
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
        anyhow::bail!("unknown cb gen project `{project_name}`. Available projects: {available}");
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
                "cb gen project `{project_name}` workspace `{workspace_name}` has no source paths"
            );
        }
        anyhow::bail!("cb gen project `{project_name}` has no source path or workspace paths");
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
            "unknown cb gen workspace `{workspace_name}` for project `{project_name}`. Available workspaces: {available}"
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
        anyhow::bail!("unknown cb gen project `{project_name}`. Available projects: {available}");
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
            "[cb gen] kept force-regen verify root at {}",
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
    std::env::temp_dir().join(format!(
        "agentic-workflow-force-regen-verify-{}-{nanos}",
        std::process::id()
    ))
}

fn copy_tree(src: &std::path::Path, dst: &std::path::Path) -> Result<()> {
    if !src.exists() {
        return Ok(());
    }
    if src.is_file() {
        if let Some(parent) = dst.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::copy(src, dst)?;
        return Ok(());
    }
    std::fs::create_dir_all(dst)?;
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let child_src = entry.path();
        let child_dst = dst.join(entry.file_name());
        if child_src.is_dir() {
            copy_tree(&child_src, &child_dst)?;
        } else if child_src.is_file() {
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

    fn enforce_complete_codegen_file_coverage(&mut self) {
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
            "deterministic source coverage incomplete: {}/{} source file(s) have CODEGEN blocks; unmanaged files: {}",
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

        let blocks = parse_codegen_blocks(&content);
        if blocks.is_empty() {
            report.unmanaged_code_files.push(SemanticReviewUnit {
                spec_ref: "(none)".to_string(),
                target_path: rel_path.clone(),
                reason: "no-codegen-block".to_string(),
            });
        }
        report.codegen_blocks += blocks.len();
        if !blocks.is_empty() {
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

    report.enforce_complete_codegen_file_coverage();
    report.enforce_complete_public_api_semantic_conformance();
    Ok(report)
}

fn collect_source_scope_files(scope: &ForceRegenScope) -> Result<Vec<std::path::PathBuf>> {
    let mut files = Vec::new();
    for root in &scope.source_roots {
        collect_source_scope_files_inner(root, &mut files)?;
    }
    files.sort();
    files.dedup();
    Ok(files)
}

fn collect_source_scope_files_inner(
    path: &std::path::Path,
    out: &mut Vec<std::path::PathBuf>,
) -> Result<()> {
    if !path.exists() {
        return Ok(());
    }
    if path.is_file() {
        out.push(path.to_path_buf());
        return Ok(());
    }
    for entry in std::fs::read_dir(path)? {
        let entry = entry?;
        let child = entry.path();
        if child.is_dir() {
            collect_source_scope_files_inner(&child, out)?;
        } else if child.is_file() {
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
    collect_tree_files_inner(root, root, &mut files)?;
    Ok(files)
}

fn collect_tree_files_inner(
    root: &std::path::Path,
    dir: &std::path::Path,
    files: &mut std::collections::BTreeMap<std::path::PathBuf, std::path::PathBuf>,
) -> Result<()> {
    if !dir.exists() {
        return Ok(());
    }
    if dir.is_file() {
        let rel = dir.strip_prefix(root)?.to_path_buf();
        files.insert(rel, dir.to_path_buf());
        return Ok(());
    }
    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            collect_tree_files_inner(root, &path, files)?;
        } else if path.is_file() {
            let rel = path.strip_prefix(root)?.to_path_buf();
            files.insert(rel, path);
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{
        cb_verify_summary_from_report, classify_codegen_origin_spec, collect_force_regen_specs,
        commit_force_regen, compare_source_roots, extract_cold_rebuild_target_paths,
        extract_project_root_llms_target_paths, extract_spec_managed_ref,
        extract_spec_managed_refs, format_rust_files, is_minified_asset_file,
        resolve_project_force_regen_scope, run_force_regen_specs, sample_count,
        sample_semantic_review_units, spec_declares_source_section,
        td_public_symbol_semantic_coverage, upsert_public_api_overview,
        upsert_public_api_overview_targets, write_project_root_llms_targets, CbCodegenOriginClass,
        CbCommand, CbGenArgs, ForceRegenConformanceReport, ForceRegenScope,
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
                reason: "no-codegen-block".to_string(),
            }],
            ..Default::default()
        };

        report.enforce_complete_codegen_file_coverage();

        assert_eq!(report.failures.len(), 1);
        assert!(report.failures[0].contains("1/2 source file(s) have CODEGEN blocks"));
        assert!(report.failures[0].contains("projects/agentic-workflow/src/generate/marker.rs"));
    }
}

// Implementation of `aw cb check` — delegates to the pre-existing
// audit pipeline. Path defaults to `.` when omitted to match
// the historical audit behaviour.
///
// @spec projects/agentic-workflow/tech-design/surface/specs/score-namespaces.md#changes
pub fn run_check(args: CbCheckArgs) -> Result<()> {
    let td_args = AuditArgs {
        path: args.target,
        json: args.json,
        group_by: args.group_by,
        ready_only: false,
        drift: false,
    };
    td::run_audit(td_args)
}

// ── cb claim ────────────────────────────────────────────────────────

// Implementation of `aw cb claim <code-path>` — recovery verb.
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

    // 4. Optional issue stub.
    let derived_slug = derive_slug_from_path(&code_path);
    if args.issue_stub {
        if let Err(e) = create_issue_stub(&project_root, &derived_slug, &args.code_path).await {
            eprintln!("warning: failed to create issue stub: {}", e);
        }
    }

    // 5. Commit a Cb-Claim trailer in the current checkout when possible.
    let mut committed = false;
    if let Err(e) = commit_cb_claim_trailer(&project_root, &derived_slug, &args.code_path) {
        eprintln!("warning: failed to commit Cb-Claim trailer: {}", e);
    } else {
        committed = true;
    }

    // 6. Emit result envelope.
    let env = serde_json::json!({
        "action": "done",
        "slug": derived_slug,
        "message": if committed {
            "cb claim: spec written; Cb-Claim trailer committed"
        } else {
            "cb claim: spec written (no trailer committed)"
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

// Create a minimal issue stub in the temp issue working copy.
async fn create_issue_stub(
    project_root: &std::path::Path,
    slug: &str,
    code_path: &str,
) -> Result<()> {
    use crate::issues::{IssueBackend, IssueState, IssueType, LocalBackend};
    let backend = LocalBackend::from_project_root(project_root);
    if backend.get(slug).await?.is_some() {
        return Ok(()); // skip when an issue already exists for the slug
    }
    let title = format!("Adopted (cb claim): {}", code_path);
    let stub = crate::issues::Issue {
        issue_type: IssueType::Enhancement,
        title: title.clone(),
        state: IssueState::Open,
        id: None,
        github_id: None,
        gitlab_id: None,
        url: None,
        author: None,
        labels: Vec::new(),
        created_at: Some(chrono::Utc::now().to_rfc3339()),
        updated_at: Some(chrono::Utc::now().to_rfc3339()),
        slug: slug.to_string(),
        body: format!("# {}\n\nIssue stub created by `aw cb claim`.\n", title),
        related: Vec::new(),
        implements: Vec::new(),
        phase: None,
        branch: None,
        target_branch: None,
        git_workflow: None,
        change_id: None,
        iteration: None,
        current_task_id: None,
        impl_spec_phase: None,
        task_revisions: None,
        revision_counts: None,
        last_action: None,
        session_id: None,
        validation_errors: Vec::new(),
        review_count: None,
        flagged_sections: None,
        fill_retry_count: None,
        ship_status: None,
        ship_commit: None,
        regen_verified_at: None,
    };
    backend.create(&stub).await?;
    Ok(())
}

// Commit a `Lifecycle-Stage: Cb-Claim` trailer in the current checkout.
// Best-effort: a missing git binary or non-git tree returns Err and the
// caller logs a warning.
fn commit_cb_claim_trailer(
    checkout_root: &std::path::Path,
    slug: &str,
    code_path: &str,
) -> Result<()> {
    let git_bin = crate::git::find_git_bin()
        .ok_or_else(|| anyhow::anyhow!("git binary not found on PATH"))?;
    let _ = std::process::Command::new(&git_bin)
        .arg("-C")
        .arg(checkout_root)
        .args(["add", "-A"])
        .output()?;
    let msg = format!(
        "cb({slug}) \u{2014} adopted code at {code_path}\n\n\
         Lifecycle-Slug: {slug}\n\
         Work-Item: {slug}\n\
         Lifecycle-Stage: Cb-Claim\n\
         Claim-Source: {code_path}\n\
         Claim-Type: cb-code"
    );
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
