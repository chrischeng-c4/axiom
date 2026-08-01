// HANDWRITE-BEGIN gap="missing-generator:python-artifact-code-check" tracker="#2305" reason="The terminal graph verifier composes compiler and target manifests until the Python artifact protocol generator owns the closure."
//! Cold target verification for the Python-v1 terminal artifact graph.

use super::{
    project_registry,
    python_ec::{self, PythonEcInventory},
    python_td::{compile_python_td_project, PythonTdIr, NATIVE_TARGET_OWNER},
    python_td_rust_target::emit_python_td_rust_target,
    python_td_target::emit_python_td_target,
    python_td_typescript_target::emit_python_td_typescript_target,
};
use anyhow::{Context, Result};
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Component, Path, PathBuf},
    process::Command,
};
use walkdir::WalkDir;

use crate::models::project::ProjectArtifactModel;

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct PythonTargetBuildCheck {
    pub target: String,
    pub workspace_root: String,
    pub ownership: NativeTargetOwnership,
    pub td_semantic_digest: String,
    pub target_build_digest: String,
    pub clean: bool,
    pub drifted_paths: Vec<String>,
    pub handwrite_paths: Vec<String>,
}

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum NativeTargetOwnership {
    Generated,
    Handwrite,
    Unresolved,
}

/// Terminal graph result for an opt-in Python artifact project.  Every value
/// is derived from the current sources; a clean result therefore cannot be
/// reused after TD, generated-target, lock, or EC-inventory drift.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct PythonArtifactCodeCheck {
    pub project: String,
    pub td_semantic_digest: String,
    pub target_build_digest: String,
    pub td_lock_clean: bool,
    pub ec_lock_clean: bool,
    pub native_unit_clean: bool,
    pub clean: bool,
    pub artifact_ids: Vec<String>,
    pub native_targets: Vec<PythonTargetBuildCheck>,
    pub findings: Vec<String>,
    pub next_command: String,
}

/// Compile the TD into a fresh target directory, then compare only the files
/// owned by the emitter manifest. Unrelated product files are intentionally
/// outside this comparison and cannot create either a false red or a false
/// green for generated output.
pub fn verify_python_target_build(
    td_root: &Path,
    output_root: &Path,
) -> Result<PythonTargetBuildCheck> {
    verify_python_target_build_for_target(
        td_root,
        output_root,
        &[output_root.to_path_buf()],
        "python",
        false,
    )
}

fn verify_python_target_build_for_target(
    td_root: &Path,
    generated_output_root: &Path,
    handwrite_roots: &[PathBuf],
    target_name: &str,
    generated_by_lifecycle: bool,
) -> Result<PythonTargetBuildCheck> {
    let ir = compile_python_td_project(td_root)?;
    let td_modules = ir
        .modules
        .iter()
        .map(|module| module.path.as_str())
        .collect::<BTreeSet<_>>();
    let cold = tempfile::tempdir().context("create Python TD cold output directory")?;
    let handwrite_paths =
        bounded_handwrite_paths(td_root, handwrite_roots, target_name, &td_modules)
            .unwrap_or_default();
    let (files, target_build_digest) = match target_name {
        "python" => {
            let target = emit_python_td_target(&ir, cold.path())?;
            (
                target
                    .files
                    .into_iter()
                    .map(|file| file.path)
                    .collect::<Vec<_>>(),
                target.digest,
            )
        }
        "rust" => {
            let target = emit_python_td_rust_target(&ir, cold.path())?;
            (
                target
                    .files
                    .into_iter()
                    .map(|file| file.path)
                    .collect::<Vec<_>>(),
                target.digest,
            )
        }
        "typescript" => {
            let target = emit_python_td_typescript_target(&ir, cold.path())?;
            (
                target
                    .files
                    .into_iter()
                    .map(|file| file.path)
                    .collect::<Vec<_>>(),
                target.digest,
            )
        }
        _ => {
            let ownership = if handwrite_paths.is_empty() {
                NativeTargetOwnership::Unresolved
            } else {
                NativeTargetOwnership::Handwrite
            };
            return Ok(PythonTargetBuildCheck {
                target: target_name.to_string(),
                workspace_root: generated_output_root.display().to_string(),
                ownership,
                td_semantic_digest: ir.semantic_digest,
                target_build_digest: digest_bytes(
                    format!("{target_name}:unsupported-native-emitter").as_bytes(),
                ),
                clean: ownership == NativeTargetOwnership::Handwrite,
                drifted_paths: Vec::new(),
                handwrite_paths: handwrite_paths
                    .iter()
                    .map(|path| path.display().to_string())
                    .collect(),
            });
        }
    };
    let generated_owned = files.iter().any(|relative| {
        fs::read_to_string(generated_output_root.join(relative))
            .is_ok_and(|source| source.contains(NATIVE_TARGET_OWNER))
    });
    let ownership = if generated_by_lifecycle || generated_owned {
        NativeTargetOwnership::Generated
    } else if !handwrite_paths.is_empty() {
        NativeTargetOwnership::Handwrite
    } else {
        NativeTargetOwnership::Unresolved
    };
    let mut drifted_paths = Vec::new();
    if ownership == NativeTargetOwnership::Generated {
        for relative in &files {
            let expected = cold.path().join(relative);
            let actual = generated_output_root.join(relative);
            let matches = match (fs::read(&expected), fs::read(&actual)) {
                (Ok(expected), Ok(actual)) => expected == actual,
                _ => false,
            };
            if !matches {
                drifted_paths.push(relative.clone());
            }
        }
    }
    drifted_paths.sort();
    Ok(PythonTargetBuildCheck {
        target: target_name.to_string(),
        workspace_root: generated_output_root.display().to_string(),
        ownership,
        td_semantic_digest: ir.semantic_digest,
        target_build_digest,
        clean: ownership != NativeTargetOwnership::Unresolved && drifted_paths.is_empty(),
        drifted_paths,
        handwrite_paths: if ownership == NativeTargetOwnership::Handwrite {
            handwrite_paths
                .iter()
                .map(|path| path.display().to_string())
                .collect()
        } else {
            Vec::new()
        },
    })
}

fn bounded_handwrite_paths(
    td_root: &Path,
    workspace_roots: &[PathBuf],
    target_name: &str,
    td_modules: &BTreeSet<&str>,
) -> Option<Vec<PathBuf>> {
    let mut native_sources = Vec::new();
    for workspace_root in workspace_roots {
        for entry in WalkDir::new(workspace_root)
            .follow_links(false)
            .into_iter()
            .filter_entry(|entry| !ignored_native_source_directory(entry))
        {
            let Ok(entry) = entry else {
                return None;
            };
            // A build may follow source or directory symlinks even though this
            // ownership walk does not. Omitting them would let the remaining
            // regular files claim the whole workspace as bounded HANDWRITE.
            if entry.file_type().is_symlink() {
                return None;
            }
            if entry.file_type().is_file() && is_native_source_for_target(entry.path(), target_name)
            {
                native_sources.push(entry);
            }
        }
    }
    (!native_sources.is_empty()
        && native_sources.iter().all(|entry| {
            fs::read_to_string(entry.path()).is_ok_and(|source| {
                has_valid_whole_file_handwrite_ownership(&source, td_root, td_modules)
            })
        }))
    .then(|| {
        let mut paths = native_sources
            .into_iter()
            .map(|entry| entry.path().to_path_buf())
            .collect::<Vec<_>>();
        paths.sort();
        paths
    })
}

fn ignored_native_source_directory(entry: &walkdir::DirEntry) -> bool {
    entry.file_type().is_dir()
        && matches!(
            entry.file_name().to_str(),
            Some(
                ".aw"
                    | ".eggs"
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
                    | "external-contracts"
                    | "node_modules"
                    | "playwright-report"
                    | "target"
                    | "tech-design"
                    | "test-results"
                    | "venv"
            )
        )
}

fn has_valid_whole_file_handwrite_ownership(
    source: &str,
    td_root: &Path,
    td_modules: &BTreeSet<&str>,
) -> bool {
    let significant = source
        .lines()
        .enumerate()
        .filter(|(_, line)| !line.trim().is_empty())
        .collect::<Vec<_>>();
    let Some((_, first)) = significant.first() else {
        return false;
    };
    let Some((_, second)) = significant.get(1) else {
        return false;
    };
    let Some((_, last)) = significant.last() else {
        return false;
    };

    let Some(spec_ref) = ownership_comment_body(first)
        .and_then(|body| body.strip_prefix("SPEC-MANAGED:"))
        .map(str::trim)
        .filter(|value| !value.is_empty())
    else {
        return false;
    };
    if !python_td_spec_ref_resolves(td_root, spec_ref, td_modules) {
        return false;
    }

    let Some(begin) = ownership_comment_body(second) else {
        return false;
    };
    if !begin.starts_with("HANDWRITE-BEGIN")
        || !matches!(
            begin.as_bytes().get("HANDWRITE-BEGIN".len()),
            None | Some(b' ' | b':')
        )
        || !valid_handwrite_attribute_set(begin)
    {
        return false;
    }
    if ownership_comment_body(last) != Some("HANDWRITE-END") {
        return false;
    }

    significant
        .iter()
        .filter_map(|(_, line)| ownership_comment_body(line))
        .filter(|body| body.starts_with("HANDWRITE-BEGIN") || *body == "HANDWRITE-END")
        .count()
        == 2
}

fn ownership_comment_body(line: &str) -> Option<&str> {
    let trimmed = line.trim_start();
    for prefix in ["//", "#", "<!--", "/*"] {
        if let Some(body) = trimmed.strip_prefix(prefix) {
            return Some(
                body.trim_start()
                    .trim_end_matches("-->")
                    .trim_end_matches("*/")
                    .trim(),
            );
        }
    }
    None
}

fn valid_handwrite_attribute_set(body: &str) -> bool {
    let mut remainder = body["HANDWRITE-BEGIN".len()..]
        .trim_start_matches(':')
        .trim();
    let mut attributes = BTreeMap::new();
    while !remainder.is_empty() {
        let key_end = remainder
            .find(|character: char| character == '=' || character.is_whitespace())
            .unwrap_or(remainder.len());
        let key = &remainder[..key_end];
        remainder = remainder[key_end..].trim_start();
        let Some(after_equals) = remainder.strip_prefix('=') else {
            return false;
        };
        remainder = after_equals.trim_start();
        let Some(quote) = remainder.chars().next() else {
            return false;
        };
        if !matches!(quote, '"' | '\'') {
            return false;
        }
        let quoted = &remainder[quote.len_utf8()..];
        let Some(value_end) = quoted.find(quote) else {
            return false;
        };
        let value = &quoted[..value_end];
        if key.is_empty() || value.trim().is_empty() || attributes.insert(key, value).is_some() {
            return false;
        }
        remainder = quoted[(value_end + quote.len_utf8())..].trim_start();
    }
    ["gap", "tracker", "reason"]
        .into_iter()
        .all(|attribute| attributes.contains_key(attribute))
}

fn python_td_spec_ref_resolves(
    td_root: &Path,
    spec_ref: &str,
    td_modules: &BTreeSet<&str>,
) -> bool {
    let path = Path::new(spec_ref.split_once('#').map_or(spec_ref, |(path, _)| path));
    if path.is_absolute()
        || path
            .components()
            .any(|component| !matches!(component, Component::Normal(_)))
        || path.extension().and_then(|value| value.to_str()) != Some("py")
    {
        return false;
    }
    let Ok(canonical_root) = td_root.canonicalize() else {
        return false;
    };
    // SPEC-MANAGED accepts either a TD-root-relative reference (`src/...`)
    // or the checkout-relative path stored by the WI. Resolve both against
    // the configured root itself; the directory need not be named
    // `tech-design` (for example `examples/todo-app/td` is valid).
    std::iter::once(td_root.join(path))
        .chain(td_root.ancestors().map(|ancestor| ancestor.join(path)))
        .any(|candidate| {
            let Ok(candidate) = candidate.canonicalize() else {
                return false;
            };
            let Ok(relative) = candidate.strip_prefix(&canonical_root) else {
                return false;
            };
            candidate.is_file()
                && td_modules.contains(relative.to_string_lossy().replace('\\', "/").as_str())
        })
}

fn is_native_source_for_target(relative: &Path, target_name: &str) -> bool {
    let extension = relative.extension().and_then(|value| value.to_str());
    match target_name {
        "python" => extension == Some("py"),
        "rust" => extension == Some("rs"),
        "typescript" => matches!(extension, Some("ts" | "tsx" | "mts" | "cts")),
        "javascript" => matches!(extension, Some("js" | "jsx" | "mjs" | "cjs")),
        // Schema and future native targets have no single canonical extension.
        // Requiring every file in src/ to be bounded keeps unsupported emitters
        // fail-closed without guessing a permissive ownership rule.
        _ => true,
    }
}

fn digest_bytes(bytes: &[u8]) -> String {
    format!("sha256:{:x}", Sha256::digest(bytes))
}

/// Verify the Python-v1 terminal graph without mutating the project.  Legacy
/// projects return `None` and retain the established Markdown code-check
/// lifecycle.  Python projects must prove: clean TD and EC locks, explicit
/// DDD artifact identities, configured target applicability, and native
/// verification appropriate to the target's ownership. Generator-owned
/// targets require a cold byte comparison; bounded HANDWRITE targets retain
/// the shared CB ownership/evidence gates and run their configured workspace
/// tests. EC semantic review and the two-cell health contract own behavioral
/// completeness.
pub fn verify_python_artifact_code_check(
    project_root: &Path,
    project: &str,
    wi: Option<&str>,
) -> Result<Option<PythonArtifactCodeCheck>> {
    let row = project_registry::resolve_project_config_row(project_root, project)?;
    if row.effective_artifact_model() != ProjectArtifactModel::PythonV1 {
        return Ok(None);
    }

    let artifact_root = project_root.join(&row.path);
    let td_root = configured_td_root(project_root, &row.name)?;
    let ir = compile_python_td_project(&td_root)?;
    let td_lock = crate::cli::td_lock::check_project_td_lock_at_root(project_root, &row.name)?;
    let ec_lock = crate::cli::ec::project_ec_lock_status_at_root(project_root, &row.name)?;
    let inventory =
        python_ec::discover_python_ec_inventory(&artifact_root.join("external-contracts"))?;
    let configured_projects = project_registry::load_projects(project_root)?;
    let configured = configured_projects
        .iter()
        .find(|configured| configured.name == row.name)
        .ok_or_else(|| {
            anyhow::anyhow!(
                "project `{}` disappeared from the project registry",
                row.name
            )
        })?;
    let workspace_roots = configured_workspace_roots(project_root, &artifact_root, configured);
    let workspace_targets = workspace_roots.keys().map(String::as_str).collect();
    let native_targets = workspace_roots
        .iter()
        .flat_map(|(target, roots)| {
            roots
                .iter()
                .enumerate()
                .map(move |(index, workspace_root)| (target, index, workspace_root))
        })
        .map(|(target, _index, workspace_root)| {
            let generated_by_lifecycle = match wi {
                Some(slug) => {
                    lifecycle_target_generated(project_root, slug, target, workspace_root)?
                }
                None => false,
            };
            let mut check = verify_python_target_build_for_target(
                &td_root,
                workspace_root,
                std::slice::from_ref(workspace_root),
                target,
                generated_by_lifecycle,
            )?;
            check.workspace_root = repo_relative_path(project_root, workspace_root);
            check.handwrite_paths = check
                .handwrite_paths
                .iter()
                .map(|path| repo_relative_path(project_root, Path::new(path)))
                .collect();
            Ok(check)
        })
        .collect::<Result<Vec<_>>>()?;

    let inventory_clean = inventory.findings.is_empty();
    let mut findings = inventory.findings.clone();
    if !td_lock.clean {
        findings.push(format!("TD lock is not clean: {}", td_lock.message));
    }
    if !ec_lock.clean {
        findings.push(format!("EC lock is not clean: {}", ec_lock.message));
    }
    for target in &native_targets {
        match target.ownership {
            NativeTargetOwnership::Generated => {
                let label = display_target(&target.target);
                for path in &target.drifted_paths {
                    findings.push(format!(
                        "generated {label} target at `{}` drifted: {path}",
                        target.workspace_root
                    ));
                }
            }
            NativeTargetOwnership::Handwrite => {}
            NativeTargetOwnership::Unresolved => findings.push(format!(
                "{} target ownership at `{}` is unresolved: no generated `{NATIVE_TARGET_OWNER}` sentinel or bounded SPEC-MANAGED HANDWRITE block exists",
                display_target(&target.target),
                target.workspace_root,
            )),
        }
    }

    let artifact_ids = validate_identity_edges(&ir, &inventory, &workspace_targets, &mut findings);
    let targets_clean = native_targets.iter().all(|target| target.clean);
    let (native_unit_clean, native_test_next) = if targets_clean {
        let result = run_configured_native_tests(project_root, configured);
        findings.extend(result.findings);
        (result.clean, result.next_command)
    } else {
        findings.push(
            "native workspace tests were not run because target ownership or cold output is stale"
                .to_string(),
        );
        (false, None)
    };

    findings.sort();
    findings.dedup();
    let clean = findings.is_empty();
    let target_build_digest = digest_bytes(&serde_json::to_vec(&native_targets)?);
    let next_command = if !inventory_clean {
        format!("aw ec check --project {}", row.name)
    } else if !ec_lock.clean {
        format!("aw ec review --project {}", row.name)
    } else if let Some(command) = native_test_next {
        command
    } else {
        // Identity, TD-lock, or generated-target failures are repaired by the
        // TD/generation side. `cb` substitutes its root WI slug here.
        format!("aw cb gen --project {}", row.name)
    };
    Ok(Some(PythonArtifactCodeCheck {
        project: row.name,
        td_semantic_digest: ir.semantic_digest,
        target_build_digest,
        td_lock_clean: td_lock.clean,
        ec_lock_clean: ec_lock.clean,
        native_unit_clean,
        clean,
        artifact_ids,
        native_targets,
        findings,
        next_command,
    }))
}

pub(crate) fn project_has_bounded_native_handwrite(
    project_root: &Path,
    project: &str,
    target: &str,
) -> Result<bool> {
    Ok(project_bounded_native_handwrite_paths(project_root, project, target)?.is_some())
}

/// Resolve native HANDWRITE paths explicitly bound to one work item by a
/// Python TD. Unlike [`project_bounded_native_handwrite_paths`], this is not a
/// Python-v1 generated-target ownership query: acceptance harnesses and other
/// root-level native sources may be outside a configured target workspace and
/// intentionally carry no inline HANDWRITE marker.
pub(crate) fn project_declared_native_handwrite_paths(
    project_root: &Path,
    project: &str,
    wi: &str,
) -> Result<Option<Vec<String>>> {
    let row = project_registry::resolve_project_config_row(project_root, project)?;
    let td_root = configured_td_root(project_root, &row.name)?;
    let ir = compile_python_td_project(&td_root)?;
    let mut paths = ir
        .modules
        .iter()
        .filter(|module| module.work_item.as_deref() == Some(wi))
        .flat_map(|module| module.native_handwrite_targets.iter().cloned())
        .collect::<Vec<_>>();
    paths.sort();
    paths.dedup();
    if paths.is_empty() {
        return Ok(None);
    }

    let canonical_root = project_root
        .canonicalize()
        .with_context(|| format!("canonicalize repository root {}", project_root.display()))?;
    for path in &paths {
        let candidate = project_root.join(path);
        let canonical = candidate.canonicalize().with_context(|| {
            format!("declared native HANDWRITE target `{path}` for work item `{wi}` does not exist")
        })?;
        if !canonical.starts_with(&canonical_root) || !canonical.is_file() {
            anyhow::bail!(
                "declared native HANDWRITE target `{path}` for work item `{wi}` must resolve to a regular repository file"
            );
        }
    }
    Ok(Some(paths))
}

pub(crate) fn project_bounded_native_handwrite_paths(
    project_root: &Path,
    project: &str,
    target: &str,
) -> Result<Option<Vec<String>>> {
    let row = project_registry::resolve_project_config_row(project_root, project)?;
    let artifact_root = project_root.join(&row.path);
    let td_root = configured_td_root(project_root, &row.name)?;
    let ir = compile_python_td_project(&td_root)?;
    let td_modules = ir
        .modules
        .iter()
        .map(|module| module.path.as_str())
        .collect::<BTreeSet<_>>();
    let configured = project_registry::load_projects(project_root)?
        .into_iter()
        .find(|configured| configured.name == row.name)
        .ok_or_else(|| anyhow::anyhow!("project `{}` has no workspace config", row.name))?;
    let workspace_roots = configured_workspace_roots(project_root, &artifact_root, &configured);
    let Some(roots) = workspace_roots
        .get(target)
        .filter(|roots| !roots.is_empty())
    else {
        return Ok(None);
    };
    let mut paths = Vec::new();
    for root in roots {
        let Some(root_paths) =
            bounded_handwrite_paths(&td_root, std::slice::from_ref(root), target, &td_modules)
        else {
            return Ok(None);
        };
        paths.extend(
            root_paths
                .iter()
                .map(|path| repo_relative_path(project_root, path)),
        );
    }
    paths.sort();
    paths.dedup();
    Ok((!paths.is_empty()).then_some(paths))
}

fn repo_relative_path(project_root: &Path, path: &Path) -> String {
    path.strip_prefix(project_root)
        .unwrap_or(path)
        .to_string_lossy()
        .replace(std::path::MAIN_SEPARATOR, "/")
}

pub(crate) fn project_primary_native_workspace_root(
    project_root: &Path,
    project: &str,
    target: &str,
) -> Result<PathBuf> {
    let row = project_registry::resolve_project_config_row(project_root, project)?;
    let artifact_root = project_root.join(&row.path);
    let configured = project_registry::load_projects(project_root)?
        .into_iter()
        .find(|configured| configured.name == row.name)
        .ok_or_else(|| anyhow::anyhow!("project `{}` has no workspace config", row.name))?;
    configured_workspace_roots(project_root, &artifact_root, &configured)
        .remove(target)
        .and_then(|roots| roots.into_iter().next())
        .ok_or_else(|| {
            anyhow::anyhow!(
                "project `{}` has no configured `{target}` workspace root",
                row.name
            )
        })
}

pub(crate) fn project_native_generation_workspace_root(
    project_root: &Path,
    project: &str,
    target: &str,
    wi: Option<&str>,
) -> Result<PathBuf> {
    let row = project_registry::resolve_project_config_row(project_root, project)?;
    let artifact_root = project_root.join(&row.path);
    let td_root = configured_td_root(project_root, &row.name)?;
    let configured = project_registry::load_projects(project_root)?
        .into_iter()
        .find(|configured| configured.name == row.name)
        .ok_or_else(|| anyhow::anyhow!("project `{}` has no workspace config", row.name))?;
    let roots = configured_workspace_roots(project_root, &artifact_root, &configured)
        .remove(target)
        .ok_or_else(|| {
            anyhow::anyhow!(
                "project `{}` has no configured `{target}` workspace root",
                row.name
            )
        })?;

    let mut unresolved = Vec::new();
    let mut stale_generated = Vec::new();
    let mut clean_generated = Vec::new();
    for root in &roots {
        let generated_by_lifecycle = match wi {
            Some(slug) => lifecycle_target_generated(project_root, slug, target, root)?,
            None => false,
        };
        let check = verify_python_target_build_for_target(
            &td_root,
            root,
            std::slice::from_ref(root),
            target,
            generated_by_lifecycle,
        )?;
        match check.ownership {
            NativeTargetOwnership::Unresolved => unresolved.push(root.clone()),
            NativeTargetOwnership::Generated if check.clean => clean_generated.push(root.clone()),
            NativeTargetOwnership::Generated => stale_generated.push(root.clone()),
            NativeTargetOwnership::Handwrite => {}
        }
    }
    let mut candidates = unresolved;
    candidates.extend(stale_generated);
    if candidates.is_empty() {
        candidates = clean_generated;
    }
    match candidates.as_slice() {
        [root] => Ok(root.clone()),
        [] => anyhow::bail!(
            "project `{}` `{target}` workspaces are all bounded HANDWRITE; run `aw cb fill` instead of generation",
            row.name
        ),
        many => anyhow::bail!(
            "project `{}` has {} `{target}` workspace roots requiring generation ({}); one target command cannot mutate multiple roots safely",
            row.name,
            many.len(),
            many.iter()
                .map(|root| repo_relative_path(project_root, root))
                .collect::<Vec<_>>()
                .join(", ")
        ),
    }
}

fn configured_workspace_roots(
    project_root: &Path,
    artifact_root: &Path,
    project: &crate::models::project::Project,
) -> BTreeMap<String, Vec<PathBuf>> {
    let mut by_target = BTreeMap::<String, Vec<PathBuf>>::new();
    for workspace in &project.workspaces {
        let target = language_target(workspace.target).to_string();
        for pattern in &workspace.paths {
            if let Some(root) = workspace_root_from_pattern(project_root, artifact_root, pattern) {
                let roots = by_target.entry(target.clone()).or_default();
                if !roots.contains(&root) {
                    roots.push(root);
                }
            }
        }
    }
    by_target
}

fn workspace_root_from_pattern(
    project_root: &Path,
    artifact_root: &Path,
    pattern: &str,
) -> Option<PathBuf> {
    let pattern_path = Path::new(pattern);
    if pattern_path.is_absolute()
        || pattern_path
            .components()
            .any(|component| !matches!(component, Component::Normal(_)))
    {
        return None;
    }
    let prefix = pattern
        .split('/')
        .take_while(|component| !component.contains(['*', '?', '[', '{']))
        .filter(|component| !component.is_empty())
        .fold(PathBuf::new(), |mut path, component| {
            path.push(component);
            path
        });
    let mut candidate = if prefix.as_os_str().is_empty() {
        artifact_root.to_path_buf()
    } else {
        project_root.join(prefix)
    };
    if candidate.is_file() {
        candidate = candidate.parent()?.to_path_buf();
    }
    let mut current = candidate.as_path();
    loop {
        if current.join("Cargo.toml").is_file()
            || current.join("pyproject.toml").is_file()
            || current.join("package.json").is_file()
        {
            candidate = current.to_path_buf();
            break;
        }
        if current == artifact_root {
            break;
        }
        let parent = current.parent()?;
        if !parent.starts_with(artifact_root) {
            break;
        }
        current = parent;
    }
    candidate.starts_with(artifact_root).then_some(candidate)
}

fn lifecycle_target_generated(
    project_root: &Path,
    slug: &str,
    target: &str,
    workspace_root: &Path,
) -> Result<bool> {
    use crate::issues::types::lifecycle_trailer;

    let git = crate::git::find_git_bin()
        .ok_or_else(|| anyhow::anyhow!("git binary not found on PATH"))?;
    let slug_line = format!("Lifecycle-Slug: {slug}");
    let output = Command::new(git)
        .arg("-C")
        .arg(project_root)
        .args([
            "log",
            "--format=%B%x1e",
            "--fixed-strings",
            "--grep",
            &slug_line,
            "HEAD",
        ])
        .output()
        .context("git log failed while resolving native target ownership")?;
    if !output.status.success() {
        anyhow::bail!(
            "git log failed while resolving native target ownership: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }
    let workspace = repo_relative_path(project_root, workspace_root);
    Ok(String::from_utf8_lossy(&output.stdout)
        .split('\x1e')
        .any(|body| {
            lifecycle_trailer::body_has_slug_trailer(body, slug)
                && lifecycle_trailer::body_has_stage_trailer(body, lifecycle_trailer::CB_GEN)
                && body
                    .lines()
                    .any(|line| line.trim_end() == format!("Native-Target: {target}"))
                && body
                    .lines()
                    .any(|line| line.trim_end() == format!("Native-Workspace: {workspace}"))
        }))
}

fn configured_td_root(project_root: &Path, project: &str) -> Result<PathBuf> {
    crate::services::project_registry::resolve_td_root_from_config(project_root, project)
        .map(|resolved| PathBuf::from(resolved.root))
        .map_err(|error| {
            anyhow::anyhow!(
                "cannot resolve configured TD root for project `{project}` ({}: {})",
                error.kind,
                error.message
            )
        })
}

fn language_target(language: crate::models::tech_stack::Language) -> &'static str {
    match language {
        crate::models::tech_stack::Language::Python => "python",
        crate::models::tech_stack::Language::Rust => "rust",
        crate::models::tech_stack::Language::TypeScript => "typescript",
        crate::models::tech_stack::Language::JavaScript => "javascript",
        crate::models::tech_stack::Language::Schemas => "schemas",
    }
}

fn display_target(target: &str) -> &str {
    match target {
        "python" => "Python",
        "rust" => "Rust",
        "typescript" => "TypeScript",
        "javascript" => "JavaScript",
        "schemas" => "schemas",
        other => other,
    }
}

struct NativeTestResult {
    clean: bool,
    findings: Vec<String>,
    next_command: Option<String>,
}

fn run_configured_native_tests(
    project_root: &Path,
    project: &crate::models::project::Project,
) -> NativeTestResult {
    let mut findings = Vec::new();
    let mut next_command = None;
    if project.workspaces.is_empty() {
        findings.push(format!(
            "project `{}` has no configured native workspace",
            project.name
        ));
        next_command = Some("aw conf sync".to_string());
    }
    for workspace in &project.workspaces {
        let name = workspace.name.as_deref().unwrap_or(project.name.as_str());
        let Some(command) = workspace.test_cmd.as_deref() else {
            findings.push(format!(
                "native workspace `{name}` has no configured test_cmd"
            ));
            next_command.get_or_insert_with(|| "aw conf sync".to_string());
            continue;
        };
        if command.trim().is_empty() {
            findings.push(format!(
                "native workspace `{name}` has an empty configured test_cmd"
            ));
            next_command.get_or_insert_with(|| "aw conf sync".to_string());
            continue;
        }
        match Command::new("sh")
            .args(["-c", command])
            .current_dir(project_root)
            .output()
        {
            Ok(output) if output.status.success() => {}
            Ok(output) => {
                findings.push(format!(
                    "native workspace `{name}` test failed: command={command:?} exit={} stdout={} stderr={}",
                    output.status,
                    bounded_output(&output.stdout),
                    bounded_output(&output.stderr),
                ));
                next_command.get_or_insert_with(|| command.to_string());
            }
            Err(error) => {
                findings.push(format!(
                    "native workspace `{name}` test could not start: command={command:?}: {error}"
                ));
                next_command.get_or_insert_with(|| command.to_string());
            }
        }
    }
    NativeTestResult {
        clean: findings.is_empty(),
        findings,
        next_command,
    }
}

fn bounded_output(bytes: &[u8]) -> String {
    let text = String::from_utf8_lossy(bytes);
    let mut chars = text.trim().chars();
    let output = chars.by_ref().take(1_000).collect::<String>();
    if chars.next().is_some() {
        format!("{output}…")
    } else {
        output
    }
}

fn validate_identity_edges(
    ir: &PythonTdIr,
    inventory: &PythonEcInventory,
    workspace_targets: &BTreeSet<&str>,
    findings: &mut Vec<String>,
) -> Vec<String> {
    let mut artifacts = BTreeSet::new();
    for module in ir
        .modules
        .iter()
        .filter(|module| module.path.starts_with("src/"))
    {
        match module.artifact_id.as_deref() {
            Some(id) => {
                artifacts.insert(id.to_string());
            }
            None if !module.declarations.is_empty() => findings.push(format!(
                "Python TD module `{}` has no explicit artifact:<context>/<name> identity",
                module.path
            )),
            // Package markers and other declaration-free modules carry no
            // domain edge of their own, so forcing an artifact identity onto
            // them would create duplicate/fake DDD nodes.
            None => {}
        }
    }
    if artifacts.is_empty() {
        findings.push("Python TD declares no explicit artifact identities under src/*".to_string());
    }

    for case in &inventory.cases {
        if !artifacts.contains(&case.artifact_id) {
            findings.push(format!(
                "Python EC case `{}` references undeclared TD artifact `{}`",
                case.id, case.artifact_id
            ));
        }
        if !workspace_targets.contains(case.target.as_str()) {
            findings.push(format!(
                "Python EC case `{}` targets `{}`, which has no configured project workspace",
                case.id, case.target
            ));
        }
    }
    artifacts.into_iter().collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn write_graph_fixture(root: &Path) {
        fs::create_dir_all(root.join("projects/demo/tech-design/src/demo/domain")).unwrap();
        fs::create_dir_all(root.join("projects/demo/external-contracts/src")).unwrap();
        fs::create_dir_all(root.join("projects/demo/external-contracts/evidence")).unwrap();
        fs::write(
            root.join("aw.toml"),
            r#"
[[projects]]
name = "demo"
path = "projects/demo"
artifact_model = "python-v1"

[[projects.workspaces]]
name = "python"
paths = ["projects/demo/**"]
target = "python"
test_cmd = "python3 -m unittest discover -s projects/demo/tests/unit"
"#,
        )
        .unwrap();
        fs::write(
            root.join("projects/demo/tech-design/src/demo/domain/order.py"),
            "__aw_artifact_id__ = \"artifact:orders/create-order\"\n\nclass Order:\n    pass\n",
        )
        .unwrap();
        fs::write(
            root.join("projects/demo/tech-design/pyproject.toml"),
            "[project]\nname = \"demo-td\"\nversion = \"0.1.0\"\nrequires-python = \">=3.11\"\ndependencies = []\n",
        )
        .unwrap();
        fs::write(
            root.join("projects/demo/tech-design/uv.lock"),
            "version = 1\nrevision = 3\nrequires-python = \">=3.11\"\n",
        )
        .unwrap();
        fs::write(
            root.join("projects/demo/external-contracts/pyproject.toml"),
            r#"
[tool.aw.python-artifact]
protocol = "aw.python-artifact.v1"
entrypoint = "src/runner.py"
source_roots = ["src"]
dependency_files = ["pyproject.toml", "uv.lock"]
evidence_dir = "evidence"

[tool.aw.python-ec]
protocol = "aw.python-ec.v1"
author = "fixture-author"
efficiency_policy = "required"

[[tool.aw.python-ec.cases]]
id = "order-behavior"
artifact_id = "artifact:orders/create-order"
capability_id = "orders"
use_case_id = "create-order"
dimension = "behavior"
applicability = "td"
test_path = "src/behavior.py"
promise = "orders are created"
oracle = "fixture-target"
target = "python"
command = "true"
evidence_paths = ["evidence/behavior.json"]

[[tool.aw.python-ec.cases]]
id = "order-security"
artifact_id = "artifact:orders/create-order"
capability_id = "orders"
use_case_id = "create-order"
dimension = "security"
applicability = "td"
test_path = "src/security.py"
promise = "orders reject unauthorized changes"
oracle = "fixture-target"
target = "python"
command = "true"
evidence_paths = ["evidence/security.json"]

[[tool.aw.python-ec.cases]]
id = "order-stability"
artifact_id = "artifact:orders/create-order"
capability_id = "orders"
use_case_id = "restart"
dimension = "stability"
applicability = "post-gen"
test_path = "src/stability.py"
promise = "orders survive restart"
oracle = "fixture-target"
threshold = "5 seconds"
target = "python"
command = "true"
evidence_paths = ["evidence/stability.json"]

[[tool.aw.python-ec.cases]]
id = "order-efficiency"
artifact_id = "artifact:orders/create-order"
capability_id = "orders"
use_case_id = "latency"
dimension = "efficiency"
applicability = "post-gen"
test_path = "src/efficiency.py"
promise = "orders meet latency budget"
oracle = "fixture-target"
threshold = "p95 under 100ms"
target = "python"
command = "true"
evidence_paths = ["evidence/efficiency.json"]
"#,
        )
        .unwrap();
        fs::write(
            root.join("projects/demo/external-contracts/uv.lock"),
            "version = 1\nrevision = 3\n",
        )
        .unwrap();
        for name in ["runner", "behavior", "security", "stability", "efficiency"] {
            fs::write(
                root.join("projects/demo/external-contracts/src")
                    .join(format!("{name}.py")),
                "def contract() -> None:\n    pass\n",
            )
            .unwrap();
        }
        for name in ["behavior", "security", "stability", "efficiency"] {
            fs::write(
                root.join("projects/demo/external-contracts/evidence")
                    .join(format!("{name}.json")),
                "{\"ok\":true}\n",
            )
            .unwrap();
        }
        let td_root = root.join("projects/demo/tech-design");
        let artifact_root = root.join("projects/demo");
        let ir = compile_python_td_project(&td_root).unwrap();
        emit_python_td_target(&ir, &artifact_root).unwrap();
        assert!(
            crate::cli::td_lock::write_project_td_lock_snapshot_at_root(root, "demo")
                .unwrap()
                .clean
        );
        assert!(
            crate::cli::ec::write_project_ec_lock_snapshot_at_root(root, "demo")
                .unwrap()
                .clean
        );
    }

    fn convert_fixture_to_handwrite_rust(root: &Path, test_cmd: &str) {
        let project = root.join("projects/demo");
        fs::remove_dir_all(project.join("src")).unwrap();
        fs::remove_dir_all(project.join("tests")).unwrap();
        fs::remove_file(project.join("pyproject.toml")).unwrap();
        fs::create_dir_all(project.join("src")).unwrap();
        fs::write(
            project.join("Cargo.toml"),
            "[package]\nname = \"demo\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
        )
        .unwrap();
        fs::write(
            project.join("src/main.rs"),
            "// SPEC-MANAGED: projects/demo/tech-design/src/demo/domain/order.py\n\
             // HANDWRITE-BEGIN gap=\"python-td-rust-body\" tracker=\"#2874\" reason=\"native Rust body remains hand-written\"\n\
             fn main() {}\n\
             // HANDWRITE-END\n",
        )
        .unwrap();

        let config = fs::read_to_string(root.join("aw.toml")).unwrap();
        let config = config
            .replace("name = \"python\"", "name = \"rust\"")
            .replace("target = \"python\"", "target = \"rust\"")
            .replace(
                "test_cmd = \"python3 -m unittest discover -s projects/demo/tests/unit\"",
                &format!("test_cmd = {test_cmd:?}"),
            );
        fs::write(root.join("aw.toml"), config).unwrap();

        let inventory = project.join("external-contracts/pyproject.toml");
        let source = fs::read_to_string(&inventory)
            .unwrap()
            .replace("target = \"python\"", "target = \"rust\"");
        fs::write(inventory, source).unwrap();
        crate::cli::td_lock::write_project_td_lock_snapshot_at_root(root, "demo").unwrap();
        crate::cli::ec::write_project_ec_lock_snapshot_at_root(root, "demo").unwrap();
    }

    #[test]
    fn cold_python_target_build_detects_only_manifest_owned_drift() {
        let td = tempfile::tempdir().unwrap();
        let output = tempfile::tempdir().unwrap();
        let source = td.path().join("src/demo/domain/invoice.py");
        fs::create_dir_all(source.parent().unwrap()).unwrap();
        fs::write(
            &source,
            "__aw_artifact_id__ = \"artifact:billing/issue-invoice\"\n\ndef issue_invoice() -> None:\n    pass\n",
        )
        .unwrap();
        let ir = compile_python_td_project(td.path()).unwrap();
        emit_python_td_target(&ir, output.path()).unwrap();
        fs::write(output.path().join("notes.txt"), "user-owned\n").unwrap();

        let clean = verify_python_target_build(td.path(), output.path()).unwrap();
        assert!(clean.clean, "{clean:#?}");
        assert!(clean.drifted_paths.is_empty());

        fs::write(
            output.path().join("src/demo/domain/invoice.py"),
            "changed\n",
        )
        .unwrap();
        let drifted = verify_python_target_build(td.path(), output.path()).unwrap();
        assert!(!drifted.clean);
        assert_eq!(drifted.drifted_paths, vec!["src/demo/domain/invoice.py"]);
    }

    #[test]
    fn spec_tree_text_cannot_claim_native_handwrite_ownership() {
        let td = tempfile::tempdir().unwrap();
        let output = tempfile::tempdir().unwrap();
        let source = td.path().join("src/demo/domain/invoice.py");
        fs::create_dir_all(source.parent().unwrap()).unwrap();
        fs::write(
            &source,
            "__aw_artifact_id__ = \"artifact:billing/issue-invoice\"\n\nclass Invoice:\n    pass\n",
        )
        .unwrap();
        let spec_only = output.path().join("tech-design/src/demo.py");
        fs::create_dir_all(spec_only.parent().unwrap()).unwrap();
        fs::write(
            spec_only,
            "# SPEC-MANAGED: source\n# HANDWRITE-BEGIN: quoted example\n",
        )
        .unwrap();

        let check = verify_python_target_build(td.path(), output.path()).unwrap();
        assert_eq!(check.ownership, NativeTargetOwnership::Unresolved);
        assert!(!check.clean);
    }

    #[test]
    fn one_handwrite_file_cannot_claim_an_unbounded_native_target() {
        let td = tempfile::tempdir().unwrap();
        let output = tempfile::tempdir().unwrap();
        let source = td.path().join("src/demo/domain/invoice.py");
        fs::create_dir_all(source.parent().unwrap()).unwrap();
        fs::write(
            &source,
            "__aw_artifact_id__ = \"artifact:billing/issue-invoice\"\n\nclass Invoice:\n    pass\n",
        )
        .unwrap();
        fs::create_dir_all(output.path().join("src")).unwrap();
        fs::write(
            output.path().join("src/owned.rs"),
            "// SPEC-MANAGED: tech-design/src/demo/domain/invoice.py\n\
             // HANDWRITE-BEGIN gap=\"rust-body\" tracker=\"#2874\" reason=\"native Rust body remains hand-written\"\n\
             pub fn owned() {}\n\
             // HANDWRITE-END\n",
        )
        .unwrap();
        fs::write(
            output.path().join("src/unowned.rs"),
            "pub fn unowned() {}\n",
        )
        .unwrap();

        let check = verify_python_target_build_for_target(
            td.path(),
            output.path(),
            &[output.path().to_path_buf()],
            "rust",
            false,
        )
        .unwrap();
        assert_eq!(check.ownership, NativeTargetOwnership::Unresolved);
        assert!(!check.clean);
    }

    #[test]
    fn native_sources_outside_src_must_be_bounded_too() {
        let td = tempfile::tempdir().unwrap();
        let output = tempfile::tempdir().unwrap();
        let source = td.path().join("src/demo/domain/invoice.py");
        fs::create_dir_all(source.parent().unwrap()).unwrap();
        fs::write(
            &source,
            "__aw_artifact_id__ = \"artifact:billing/issue-invoice\"\n\nclass Invoice:\n    pass\n",
        )
        .unwrap();
        fs::create_dir_all(output.path().join("src")).unwrap();
        fs::write(
            output.path().join("src/invoice.rs"),
            "// SPEC-MANAGED: tech-design/src/demo/domain/invoice.py\n\
             // HANDWRITE-BEGIN gap=\"rust-body\" tracker=\"#2874\" reason=\"native Rust body remains hand-written\"\n\
             pub struct Invoice;\n\
             // HANDWRITE-END\n",
        )
        .unwrap();
        fs::write(output.path().join("build.rs"), "fn main() {}\n").unwrap();

        let check = verify_python_target_build_for_target(
            td.path(),
            output.path(),
            &[output.path().to_path_buf()],
            "rust",
            false,
        )
        .unwrap();
        assert_eq!(check.ownership, NativeTargetOwnership::Unresolved);
        assert!(!check.clean);
    }

    #[test]
    fn ownership_scan_prunes_standard_cache_and_build_trees() {
        let td = tempfile::tempdir().unwrap();
        let output = tempfile::tempdir().unwrap();
        let source = td.path().join("src/demo/domain/invoice.py");
        fs::create_dir_all(source.parent().unwrap()).unwrap();
        fs::write(
            &source,
            "__aw_artifact_id__ = \"artifact:billing/issue-invoice\"\n\nclass Invoice:\n    pass\n",
        )
        .unwrap();
        fs::create_dir_all(output.path().join("src")).unwrap();
        fs::write(
            output.path().join("src/manual.py"),
            "# SPEC-MANAGED: src/demo/domain/invoice.py\n\
             # HANDWRITE-BEGIN gap=\"python-body\" tracker=\"#2874\" reason=\"native Python body remains hand-written\"\n\
             value = 1\n\
             # HANDWRITE-END\n",
        )
        .unwrap();
        for cache in [
            ".pytest_cache",
            ".mypy_cache",
            ".ruff_cache",
            ".tox",
            "build",
            "dist",
        ] {
            fs::create_dir_all(output.path().join(cache)).unwrap();
            fs::write(output.path().join(cache).join("cached.py"), "value = 2\n").unwrap();
        }

        let check = verify_python_target_build_for_target(
            td.path(),
            output.path(),
            &[output.path().to_path_buf()],
            "python",
            false,
        )
        .unwrap();
        assert_eq!(check.ownership, NativeTargetOwnership::Handwrite);
        assert!(check.clean);
    }

    #[test]
    fn glob_only_workspace_pattern_resolves_to_artifact_root() {
        let root = tempfile::tempdir().unwrap();
        let artifact_root = root.path().join("apps/demo");
        fs::create_dir_all(&artifact_root).unwrap();
        assert_eq!(
            workspace_root_from_pattern(root.path(), &artifact_root, "**").unwrap(),
            artifact_root
        );
    }

    #[test]
    fn workspace_pattern_with_parent_component_cannot_escape_artifact_root() {
        let root = tempfile::tempdir().unwrap();
        let artifact_root = root.path().join("projects/demo");
        fs::create_dir_all(&artifact_root).unwrap();
        assert!(workspace_root_from_pattern(
            root.path(),
            &artifact_root,
            "projects/demo/../other/**"
        )
        .is_none());
    }

    #[cfg(unix)]
    #[test]
    fn symlinked_native_source_prevents_handwrite_ownership() {
        use std::os::unix::fs::symlink;

        let td = tempfile::tempdir().unwrap();
        let output = tempfile::tempdir().unwrap();
        let external = tempfile::tempdir().unwrap();
        let source = td.path().join("src/demo/domain/invoice.py");
        fs::create_dir_all(source.parent().unwrap()).unwrap();
        fs::write(
            &source,
            "__aw_artifact_id__ = \"artifact:billing/issue-invoice\"\n\nclass Invoice:\n    pass\n",
        )
        .unwrap();
        fs::create_dir_all(output.path().join("src")).unwrap();
        fs::write(
            output.path().join("src/manual.rs"),
            "// SPEC-MANAGED: src/demo/domain/invoice.py\n\
             // HANDWRITE-BEGIN gap=\"rust-body\" tracker=\"#2874\" reason=\"native Rust body remains hand-written\"\n\
             pub struct Invoice;\n\
             // HANDWRITE-END\n",
        )
        .unwrap();
        let outside = external.path().join("hidden.rs");
        fs::write(&outside, "pub struct Hidden;\n").unwrap();
        symlink(&outside, output.path().join("src/hidden.rs")).unwrap();

        let check = verify_python_target_build_for_target(
            td.path(),
            output.path(),
            &[output.path().to_path_buf()],
            "rust",
            false,
        )
        .unwrap();
        assert_eq!(check.ownership, NativeTargetOwnership::Unresolved);
        assert!(!check.clean);
    }

    #[test]
    fn spec_reference_resolves_against_configured_td_root_name() {
        let root = tempfile::tempdir().unwrap();
        let td_root = root.path().join("examples/demo/td");
        let module = td_root.join("src/demo/policy.py");
        fs::create_dir_all(module.parent().unwrap()).unwrap();
        fs::write(
            &module,
            "__aw_artifact_id__ = \"artifact:policy/evaluate\"\n\nclass Policy:\n    pass\n",
        )
        .unwrap();
        let modules = BTreeSet::from(["src/demo/policy.py"]);
        assert!(python_td_spec_ref_resolves(
            &td_root,
            "examples/demo/td/src/demo/policy.py",
            &modules,
        ));
        assert!(python_td_spec_ref_resolves(
            &td_root,
            "src/demo/policy.py",
            &modules,
        ));
    }

    #[test]
    fn generation_selects_unresolved_root_without_mutating_handwrite_root() {
        let root = tempfile::tempdir().unwrap();
        let td = root
            .path()
            .join("projects/demo/tech-design/src/demo/policy.py");
        fs::create_dir_all(td.parent().unwrap()).unwrap();
        fs::write(
            &td,
            "__aw_artifact_id__ = \"artifact:policy/evaluate\"\n\nclass Policy:\n    pass\n",
        )
        .unwrap();
        fs::write(
            root.path().join("aw.toml"),
            r#"
[[projects]]
name = "demo"
path = "projects/demo"
artifact_model = "python-v1"

[[projects.workspaces]]
name = "hand"
paths = ["projects/demo/hand/**"]
target = "rust"

[[projects.workspaces]]
name = "generated"
paths = ["projects/demo/generated/**"]
target = "rust"
"#,
        )
        .unwrap();
        let hand = root.path().join("projects/demo/hand");
        let generated = root.path().join("projects/demo/generated");
        fs::create_dir_all(hand.join("src")).unwrap();
        fs::create_dir_all(&generated).unwrap();
        fs::write(
            hand.join("Cargo.toml"),
            "[package]\nname = \"hand\"\nversion = \"0.1.0\"\n",
        )
        .unwrap();
        fs::write(
            generated.join("Cargo.toml"),
            "[package]\nname = \"generated\"\nversion = \"0.1.0\"\n",
        )
        .unwrap();
        fs::write(
            hand.join("src/policy.rs"),
            "// SPEC-MANAGED: projects/demo/tech-design/src/demo/policy.py\n\
             // HANDWRITE-BEGIN gap=\"rust-body\" tracker=\"#2874\" reason=\"native Rust body remains hand-written\"\n\
             pub struct Policy;\n\
             // HANDWRITE-END\n",
        )
        .unwrap();

        assert_eq!(
            project_native_generation_workspace_root(root.path(), "demo", "rust", None).unwrap(),
            generated
        );
    }

    #[test]
    fn committed_generation_cannot_downgrade_to_handwrite_after_sentinels_disappear() {
        let td = tempfile::tempdir().unwrap();
        let output = tempfile::tempdir().unwrap();
        let source = td.path().join("src/demo/domain/invoice.py");
        fs::create_dir_all(source.parent().unwrap()).unwrap();
        fs::write(
            &source,
            "__aw_artifact_id__ = \"artifact:billing/issue-invoice\"\n\nclass Invoice:\n    pass\n",
        )
        .unwrap();
        fs::create_dir_all(output.path().join("src")).unwrap();
        fs::write(
            output.path().join("src/manual.py"),
            format!(
                "# SPEC-MANAGED: tech-design/src/demo/domain/invoice.py\n\
                 # HANDWRITE-BEGIN gap=\"manual\" tracker=\"#2874\" reason=\"attempted ownership downgrade\"\n\
                 value = {NATIVE_TARGET_OWNER:?}\n\
                 # HANDWRITE-END\n"
            ),
        )
        .unwrap();

        let check = verify_python_target_build_for_target(
            td.path(),
            output.path(),
            &[output.path().to_path_buf()],
            "python",
            true,
        )
        .unwrap();
        assert_eq!(check.ownership, NativeTargetOwnership::Generated);
        assert!(!check.clean);
        assert!(!check.drifted_paths.is_empty());
    }

    #[test]
    fn quoted_reason_text_cannot_supply_missing_handwrite_attributes() {
        let td = tempfile::tempdir().unwrap();
        let output = tempfile::tempdir().unwrap();
        let source = td.path().join("src/demo/domain/invoice.py");
        fs::create_dir_all(source.parent().unwrap()).unwrap();
        fs::write(
            &source,
            "__aw_artifact_id__ = \"artifact:billing/issue-invoice\"\n\nclass Invoice:\n    pass\n",
        )
        .unwrap();
        fs::create_dir_all(output.path().join("src")).unwrap();
        fs::write(
            output.path().join("src/invoice.rs"),
            "// SPEC-MANAGED: tech-design/src/demo/domain/invoice.py\n\
             // HANDWRITE-BEGIN reason=\"description gap=fake tracker=#1\"\n\
             pub struct Invoice;\n\
             // HANDWRITE-END\n",
        )
        .unwrap();

        let check = verify_python_target_build_for_target(
            td.path(),
            output.path(),
            &[output.path().to_path_buf()],
            "rust",
            false,
        )
        .unwrap();
        assert_eq!(check.ownership, NativeTargetOwnership::Unresolved);
    }

    #[test]
    fn non_compiled_python_file_cannot_bind_native_handwrite_ownership() {
        let td = tempfile::tempdir().unwrap();
        let output = tempfile::tempdir().unwrap();
        let source = td.path().join("src/demo/domain/invoice.py");
        fs::create_dir_all(source.parent().unwrap()).unwrap();
        fs::write(
            &source,
            "__aw_artifact_id__ = \"artifact:billing/issue-invoice\"\n\nclass Invoice:\n    pass\n",
        )
        .unwrap();
        fs::create_dir_all(td.path().join("docs")).unwrap();
        fs::write(
            td.path().join("docs/example.py"),
            "class Example:\n    pass\n",
        )
        .unwrap();
        fs::create_dir_all(output.path().join("src")).unwrap();
        fs::write(
            output.path().join("src/invoice.rs"),
            "// SPEC-MANAGED: tech-design/docs/example.py\n\
             // HANDWRITE-BEGIN gap=\"manual\" tracker=\"#2874\" reason=\"invalid non-TD binding\"\n\
             pub struct Invoice;\n\
             // HANDWRITE-END\n",
        )
        .unwrap();

        let check = verify_python_target_build_for_target(
            td.path(),
            output.path(),
            &[output.path().to_path_buf()],
            "rust",
            false,
        )
        .unwrap();
        assert_eq!(check.ownership, NativeTargetOwnership::Unresolved);
    }

    #[test]
    fn cb_gen_history_binds_generated_ownership_to_one_native_target() {
        if crate::git::find_git_bin().is_none() {
            return;
        }
        let root = tempfile::tempdir().unwrap();
        let generated = root.path().join("projects/demo/generated");
        let handwrite = root.path().join("projects/demo/handwrite");
        fs::create_dir_all(&generated).unwrap();
        fs::create_dir_all(&handwrite).unwrap();
        for args in [
            vec!["init", "-q", "-b", "main"],
            vec!["config", "user.email", "test@example.com"],
            vec!["config", "user.name", "Test"],
            vec!["commit", "--allow-empty", "-m", "seed", "-q"],
            vec![
                "commit",
                "--allow-empty",
                "-m",
                "rust generated\n\nLifecycle-Slug: 42\nLifecycle-Stage: Cb-Gen\nNative-Target: rust\nNative-Workspace: projects/demo/generated",
                "-q",
            ],
        ] {
            assert!(Command::new("git")
                .arg("-C")
                .arg(root.path())
                .args(args)
                .status()
                .unwrap()
                .success());
        }

        assert!(lifecycle_target_generated(root.path(), "42", "rust", &generated).unwrap());
        assert!(!lifecycle_target_generated(root.path(), "42", "rust", &handwrite).unwrap());
        assert!(!lifecycle_target_generated(root.path(), "42", "typescript", &generated).unwrap());
    }

    #[test]
    fn python_td_canonical_routing_readiness_cb_and_code_check_share_one_ir() {
        let root = tempfile::tempdir().unwrap();
        write_graph_fixture(root.path());

        let readiness = crate::services::python_artifact_readiness::evaluate(root.path(), "demo")
            .unwrap()
            .unwrap();
        let clean = verify_python_artifact_code_check(root.path(), "demo", None)
            .unwrap()
            .unwrap();
        assert_eq!(
            readiness.td_semantic_digest.as_deref(),
            Some(clean.td_semantic_digest.as_str())
        );
        assert!(clean.clean, "{clean:#?}");
        assert_eq!(clean.artifact_ids, vec!["artifact:orders/create-order"]);
        assert!(clean.td_lock_clean && clean.ec_lock_clean && clean.native_unit_clean);

        fs::write(
            root.path().join("projects/demo/src/demo/domain/order.py"),
            "stale\n",
        )
        .unwrap();
        let stale = verify_python_artifact_code_check(root.path(), "demo", None)
            .unwrap()
            .unwrap();
        assert!(!stale.clean);
        assert!(stale.findings.iter().any(|finding| finding.contains(
            "generated Python target at `projects/demo` drifted: src/demo/domain/order.py"
        )));
    }

    #[test]
    fn every_configured_root_for_one_generated_target_is_verified() {
        let root = tempfile::tempdir().unwrap();
        write_graph_fixture(root.path());
        let secondary = root.path().join("projects/demo/secondary");
        fs::create_dir_all(secondary.join("src")).unwrap();
        fs::write(
            secondary.join("pyproject.toml"),
            "[project]\nname = \"demo-secondary\"\nversion = \"0.1.0\"\n",
        )
        .unwrap();
        fs::write(secondary.join("src/unowned.py"), "value = 1\n").unwrap();
        let config = fs::read_to_string(root.path().join("aw.toml")).unwrap();
        fs::write(
            root.path().join("aw.toml"),
            format!(
                "{config}\n[[projects.workspaces]]\nname = \"secondary\"\npaths = [\"projects/demo/secondary/**\"]\ntarget = \"python\"\ntest_cmd = \"true\"\n"
            ),
        )
        .unwrap();

        let report = verify_python_artifact_code_check(root.path(), "demo", None)
            .unwrap()
            .unwrap();
        assert!(!report.clean);
        assert_eq!(report.native_targets.len(), 2);
        assert!(report.native_targets.iter().any(|target| {
            target.workspace_root == "projects/demo/secondary"
                && target.ownership == NativeTargetOwnership::Unresolved
        }));
    }

    #[test]
    fn python_artifact_code_check_keeps_identity_valid_across_projection_move() {
        let root = tempfile::tempdir().unwrap();
        write_graph_fixture(root.path());
        let before = verify_python_artifact_code_check(root.path(), "demo", None)
            .unwrap()
            .unwrap();

        let td_root = root.path().join("projects/demo/tech-design");
        let old = td_root.join("src/demo/domain/order.py");
        let moved = td_root.join("src/demo/domain/create_order.py");
        fs::create_dir_all(moved.parent().unwrap()).unwrap();
        fs::rename(old, &moved).unwrap();
        let ir = compile_python_td_project(&td_root).unwrap();
        emit_python_td_target(&ir, &root.path().join("projects/demo")).unwrap();
        crate::cli::td_lock::write_project_td_lock_snapshot_at_root(root.path(), "demo").unwrap();

        let after = verify_python_artifact_code_check(root.path(), "demo", None)
            .unwrap()
            .unwrap();
        assert!(after.clean, "{after:#?}");
        assert_eq!(before.td_semantic_digest, after.td_semantic_digest);
        assert_eq!(before.artifact_ids, after.artifact_ids);
    }

    #[test]
    fn python_handwrite_cb_uses_native_workspace_test_without_generated_drift() {
        let root = tempfile::tempdir().unwrap();
        write_graph_fixture(root.path());
        convert_fixture_to_handwrite_rust(root.path(), "true");

        let report = verify_python_artifact_code_check(root.path(), "demo", None)
            .unwrap()
            .unwrap();
        assert!(report.clean, "{report:#?}");
        assert!(report.native_unit_clean);
        assert_eq!(report.native_targets.len(), 1);
        assert_eq!(report.native_targets[0].target, "rust");
        assert_eq!(
            report.native_targets[0].ownership,
            NativeTargetOwnership::Handwrite
        );
        assert!(report
            .findings
            .iter()
            .all(|finding| !finding.contains("generated Python target drifted")));
    }

    #[test]
    fn python_handwrite_cb_fails_closed_when_native_workspace_test_fails() {
        let root = tempfile::tempdir().unwrap();
        write_graph_fixture(root.path());
        convert_fixture_to_handwrite_rust(root.path(), "false");

        let report = verify_python_artifact_code_check(root.path(), "demo", None)
            .unwrap()
            .unwrap();
        assert!(!report.clean);
        assert!(!report.native_unit_clean);
        assert_eq!(report.next_command, "false");
        assert!(report
            .findings
            .iter()
            .any(|finding| finding.contains("native workspace `rust` test failed")));
    }

    #[test]
    fn python_handwrite_cb_rejects_an_empty_native_workspace_test() {
        let root = tempfile::tempdir().unwrap();
        write_graph_fixture(root.path());
        convert_fixture_to_handwrite_rust(root.path(), "   ");

        let report = verify_python_artifact_code_check(root.path(), "demo", None)
            .unwrap()
            .unwrap();
        assert!(!report.clean);
        assert!(!report.native_unit_clean);
        assert_eq!(report.next_command, "aw conf sync");
        assert!(report
            .findings
            .iter()
            .any(|finding| finding.contains("empty configured test_cmd")));
    }

    #[test]
    fn python_artifact_code_check_rejects_unknown_target_without_inventing_dimensions() {
        let root = tempfile::tempdir().unwrap();
        write_graph_fixture(root.path());
        let inventory = root
            .path()
            .join("projects/demo/external-contracts/pyproject.toml");
        let source = fs::read_to_string(&inventory).unwrap();
        let missing_security_and_bad_target = source
            .replace("dimension = \"security\"", "dimension = \"behavior\"")
            .replace(
            "target = \"python\"\ncommand = \"true\"\nevidence_paths = [\"evidence/security.json\"]\n\n[[tool.aw.python-ec.cases]]\nid = \"order-stability\"",
            "target = \"rust\"\ncommand = \"true\"\nevidence_paths = [\"evidence/security.json\"]\n\n[[tool.aw.python-ec.cases]]\nid = \"order-stability\"",
        );
        fs::write(&inventory, missing_security_and_bad_target).unwrap();

        let report = verify_python_artifact_code_check(root.path(), "demo", None)
            .unwrap()
            .unwrap();
        assert!(!report.clean);
        assert!(report
            .findings
            .iter()
            .any(|finding| finding
                .contains("targets `rust`, which has no configured project workspace")));
        assert!(!report
            .findings
            .iter()
            .any(|finding| finding.contains("missing required `security` EC coverage")));
    }
}
// HANDWRITE-END
