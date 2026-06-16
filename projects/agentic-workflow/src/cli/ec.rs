// SPEC-MANAGED: projects/agentic-workflow/tech-design/semantic/agentic-workflow-cli.md#schema
// CODEGEN-BEGIN
// generator-gap: aw-ec-cli-v1
// reason: EC inventory/check generation is a new workflow surface not yet covered by deterministic CLI codegen primitives.
use anyhow::{bail, Context, Result};
use clap::{Args, Subcommand};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use walkdir::WalkDir;

const EC_MANIFEST_VERSION: u8 = 1;
const EC_MANIFEST_REL: &str = "tests/aw-ec.toml";
const EC_DOC_REL: &str = "docs/aw-ec-manual.md";
const EC_SOURCE_REL: &str = "external-contracts";
const PROJECT_AW_REL: &str = "aw.toml";
const EC_AW_BEGIN_MARKER: &str = "AW-EC-BEGIN";
const EC_AW_END_MARKER: &str = "AW-EC-END";
const EC_BEGIN_MARKER: &str = "AW-EC-BEGIN";
const EC_END_MARKER: &str = "AW-EC-END";
const EC_TOOL_BEGIN_MARKER: &str = "AW-EC-TOOL-BEGIN";
const EC_TOOL_END_MARKER: &str = "AW-EC-TOOL-END";
const EC_DOC_BEGIN_MARKER: &str = "AW-EC-DOC-BEGIN";
const EC_DOC_END_MARKER: &str = "AW-EC-DOC-END";
const EC_CATEGORIES: [&str; 4] = ["behavior", "efficiency", "security", "stability"];

/// The canonical EC dimension/category names. The single source of truth shared
/// with capability-type required-dimension derivation.
/// @spec projects/agentic-workflow/tech-design/semantic/agentic-workflow-cli.md#schema
pub fn ec_categories() -> &'static [&'static str] {
    &EC_CATEGORIES
}

#[derive(Debug, Args)]
/// External-contract lifecycle: draft/fill EC markdown, then generate/check/verify artifacts.
/// @spec projects/agentic-workflow/tech-design/semantic/agentic-workflow-cli.md#schema
pub struct EcArgs {
    /// Project name or alias from .aw/config.toml.
    #[arg(long, global = true)]
    pub project: Option<String>,
    #[command(subcommand)]
    pub command: EcCommand,
}

/// @spec projects/agentic-workflow/tech-design/semantic/agentic-workflow-cli.md#schema
#[derive(Debug, Subcommand)]
pub enum EcCommand {
    /// Create a project-local EC markdown draft under external-contracts/.
    Draft(EcDraftArgs),
    /// Fill one section in an EC markdown draft.
    Fill(EcFillArgs),
    /// Generate manifest, tests, and tool configs from ec/ markdown.
    Gen(EcGenArgs),
    /// Check EC manifest/list drift and generated test-file presence.
    Check(EcCheckArgs),
    /// Run generated external-contract verification commands.
    Verify(EcVerifyArgs),
    /// Generate, check, or preview EC-derived product documentation.
    Doc(EcDocArgs),
}

/// @spec projects/agentic-workflow/tech-design/semantic/agentic-workflow-cli.md#schema
#[derive(Debug, Args)]
pub struct EcDraftArgs {
    /// Contract id / file slug.
    pub id: String,
    /// Contract category: behavior, efficiency, security, or stability.
    #[arg(long, default_value = "behavior")]
    pub category: String,
    /// Human title for the markdown heading.
    #[arg(long)]
    pub title: Option<String>,
    /// README capability id protected by this contract.
    #[arg(long)]
    pub capability_id: Option<String>,
    /// Capability claim id protected by this contract.
    #[arg(long)]
    pub claim_id: Option<String>,
    /// Contract id exposed in the generated EC manifest.
    #[arg(long)]
    pub contract_id: Option<String>,
    /// Verification command for the generated manifest case.
    #[arg(long)]
    pub command: Option<String>,
    /// Also scaffold a tool-contract section for this native integration tool.
    #[arg(long)]
    pub tool: Vec<String>,
    /// Overwrite an existing draft.
    #[arg(long)]
    pub force: bool,
    /// Emit JSON instead of human-readable output.
    #[arg(long)]
    pub json: bool,
}

/// @spec projects/agentic-workflow/tech-design/semantic/agentic-workflow-cli.md#schema
#[derive(Debug, Args)]
pub struct EcFillArgs {
    /// EC markdown file to update.
    pub path: PathBuf,
    /// Section type to replace or append, for example e2e-test or tool-contract.
    #[arg(long)]
    pub section: String,
    /// Markdown fragment containing the complete replacement section.
    #[arg(long)]
    pub body_file: PathBuf,
    /// Emit JSON instead of human-readable output.
    #[arg(long)]
    pub json: bool,
}

/// @spec projects/agentic-workflow/tech-design/semantic/agentic-workflow-cli.md#schema
#[derive(Debug, Args)]
pub struct EcGenArgs {
    /// Print the generated inventory without writing files.
    #[arg(long)]
    pub dry_run: bool,
    /// Run `aw ec check` after generation.
    #[arg(long)]
    pub verify: bool,
    /// Emit JSON instead of human-readable output.
    #[arg(long)]
    pub json: bool,
}

/// @spec projects/agentic-workflow/tech-design/semantic/agentic-workflow-cli.md#schema
#[derive(Debug, Args)]
pub struct EcCheckArgs {
    /// Emit JSON instead of human-readable output.
    #[arg(long)]
    pub json: bool,
}

/// @spec projects/agentic-workflow/tech-design/semantic/agentic-workflow-cli.md#schema
#[derive(Debug, Args)]
pub struct EcVerifyArgs {
    /// Emit JSON instead of human-readable output.
    #[arg(long)]
    pub json: bool,
}

/// @spec projects/agentic-workflow/tech-design/semantic/agentic-workflow-cli.md#schema
#[derive(Debug, Args)]
pub struct EcDocArgs {
    #[command(subcommand)]
    pub command: EcDocCommand,
}

/// @spec projects/agentic-workflow/tech-design/semantic/agentic-workflow-cli.md#schema
#[derive(Debug, Subcommand)]
pub enum EcDocCommand {
    /// Generate docs/aw-ec-manual.md from the EC manifest and evidence metadata.
    Gen(EcDocGenArgs),
    /// Check generated EC documentation for manifest drift.
    Check(EcDocCheckArgs),
    /// Print the generated EC documentation path for local preview.
    Preview(EcDocPreviewArgs),
}

/// @spec projects/agentic-workflow/tech-design/semantic/agentic-workflow-cli.md#schema
#[derive(Debug, Args)]
pub struct EcDocGenArgs {
    /// Print the generated documentation without writing files.
    #[arg(long)]
    pub dry_run: bool,
    /// Run `aw ec doc check` after generation.
    #[arg(long)]
    pub verify: bool,
    /// Emit JSON instead of human-readable output.
    #[arg(long)]
    pub json: bool,
}

/// @spec projects/agentic-workflow/tech-design/semantic/agentic-workflow-cli.md#schema
#[derive(Debug, Args)]
pub struct EcDocCheckArgs {
    /// Emit JSON instead of human-readable output.
    #[arg(long)]
    pub json: bool,
}

/// @spec projects/agentic-workflow/tech-design/semantic/agentic-workflow-cli.md#schema
#[derive(Debug, Args)]
pub struct EcDocPreviewArgs {
    /// Emit JSON instead of human-readable output.
    #[arg(long)]
    pub json: bool,
}

/// @spec projects/agentic-workflow/tech-design/semantic/agentic-workflow-cli.md#schema
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct EcManifest {
    pub version: u8,
    pub project: String,
    pub generated_from_td_digest: String,
    #[serde(default)]
    pub cases: Vec<EcManifestCase>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tool_manifests: Vec<EcToolManifest>,
}

/// @spec projects/agentic-workflow/tech-design/semantic/agentic-workflow-cli.md#schema
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct EcManifestCase {
    pub id: String,
    pub capability_id: String,
    #[serde(default)]
    pub claim_id: String,
    pub contract_id: String,
    pub category: String,
    pub td_ref: String,
    pub test_path: String,
    pub command: String,
    #[serde(default = "default_required_for_production")]
    pub required_for_production: bool,
    #[serde(default)]
    pub assertions: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub evidence: Vec<EcEvidenceArtifact>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub evaluators: Vec<EcEvaluator>,
}

/// @spec projects/agentic-workflow/tech-design/semantic/agentic-workflow-cli.md#schema
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct EcEvidenceArtifact {
    pub kind: String,
    pub path: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub label: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub locator: String,
}

/// @spec projects/agentic-workflow/tech-design/semantic/agentic-workflow-cli.md#schema
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct EcEvaluator {
    pub id: String,
    pub tool: String,
    pub command: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub report_path: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub prompt: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub rubric: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub pass_criteria: Vec<String>,
}

/// Native tool manifest generated from TD `tool-contract` sections.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct EcToolManifest {
    pub id: String,
    pub tool: String,
    pub path: String,
    pub td_ref: String,
    pub content_digest: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub command: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub category: String,
    #[serde(skip, default)]
    pub generated_toml: String,
}

/// @spec projects/agentic-workflow/tech-design/semantic/agentic-workflow-cli.md#schema
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct EcCheckSummary {
    pub project: String,
    pub clean: bool,
    pub configured: bool,
    pub manifest_path: String,
    pub generated_from_td_digest: String,
    pub manifest_td_digest: Option<String>,
    pub expected_case_count: usize,
    pub case_count: usize,
    pub expected_tool_manifest_count: usize,
    pub tool_manifest_count: usize,
    pub stale: bool,
    pub missing_test_paths: Vec<String>,
    pub orphan_test_paths: Vec<String>,
    pub missing_tool_manifest_paths: Vec<String>,
    pub findings: Vec<String>,
}

/// @spec projects/agentic-workflow/tech-design/semantic/agentic-workflow-cli.md#schema
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct EcDocCheckSummary {
    pub project: String,
    pub clean: bool,
    pub configured: bool,
    pub doc_path: String,
    pub manifest_path: String,
    pub manifest_digest: Option<String>,
    pub case_count: usize,
    pub findings: Vec<String>,
}

/// @spec projects/agentic-workflow/tech-design/semantic/agentic-workflow-cli.md#schema
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct EcDocPreviewSummary {
    pub project: String,
    pub doc_path: String,
    pub exists: bool,
}

/// @spec projects/agentic-workflow/tech-design/semantic/agentic-workflow-cli.md#schema
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct EcVerifySummary {
    pub project: String,
    pub manifest_path: String,
    pub clean: bool,
    pub command_count: usize,
    pub passed_count: usize,
    pub failed_count: usize,
    pub results: Vec<EcVerifyCommandResult>,
}

/// @spec projects/agentic-workflow/tech-design/semantic/agentic-workflow-cli.md#schema
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct EcVerifyCommandResult {
    pub case_id: String,
    pub capability_id: String,
    pub claim_id: String,
    pub category: String,
    pub command: String,
    pub status: String,
    pub exit_code: Option<i32>,
    pub stdout_tail: String,
    pub stderr_tail: String,
}

/// @spec projects/agentic-workflow/tech-design/semantic/agentic-workflow-cli.md#schema
#[derive(Debug, Clone)]
pub struct EcProjectContext {
    pub project_root: PathBuf,
    pub project: String,
    pub source_root: PathBuf,
    pub ec_root: PathBuf,
    pub td_root: PathBuf,
    pub tests_root: PathBuf,
    pub manifest_path: PathBuf,
    pub legacy_manifest_path: PathBuf,
    pub project_aw_path: PathBuf,
    pub doc_path: PathBuf,
    pub target: String,
    pub package_name: String,
}

#[derive(Debug, Deserialize, Default)]
struct E2eYaml {
    #[serde(default)]
    e2e_tests: Vec<E2eYamlCase>,
}

#[derive(Debug, Deserialize, Default)]
struct E2eYamlCase {
    #[serde(default)]
    id: Option<String>,
    #[serde(default)]
    name: Option<String>,
    #[serde(default)]
    command: Option<String>,
    #[serde(default)]
    test_path: Option<String>,
    #[serde(default)]
    assertions: Option<StringOrList>,
    #[serde(default)]
    asserts: Option<StringOrList>,
    #[serde(default)]
    capability_id: Option<String>,
    #[serde(default)]
    claim_id: Option<String>,
    #[serde(default)]
    contract_id: Option<String>,
    #[serde(default)]
    category: Option<String>,
    #[serde(default)]
    required_for_production: Option<bool>,
    #[serde(default)]
    evidence: Option<E2eEvidenceYaml>,
    #[serde(default, alias = "evals", alias = "agent_evals")]
    evaluators: Vec<E2eEvaluatorYaml>,
}

#[derive(Debug, Deserialize, Default)]
struct E2eEvidenceYaml {
    #[serde(default)]
    screenshots: Vec<E2eArtifactYaml>,
    #[serde(default)]
    reports: Vec<E2eArtifactYaml>,
    #[serde(default)]
    docs: Vec<E2eArtifactYaml>,
    #[serde(default)]
    eval: Option<E2eArtifactYaml>,
}

#[derive(Debug, Deserialize, Default)]
struct E2eArtifactYaml {
    #[serde(default)]
    id: Option<String>,
    #[serde(default)]
    kind: Option<String>,
    #[serde(default)]
    path: Option<String>,
    #[serde(default)]
    label: Option<String>,
    #[serde(default)]
    locator: Option<String>,
}

#[derive(Debug, Deserialize, Default)]
struct E2eEvaluatorYaml {
    #[serde(default)]
    id: Option<String>,
    #[serde(default)]
    tool: Option<String>,
    #[serde(default)]
    command: Option<String>,
    #[serde(default, alias = "report")]
    report_path: Option<String>,
    #[serde(default)]
    prompt: Option<String>,
    #[serde(default)]
    rubric: Option<StringOrList>,
    #[serde(default, alias = "pass", alias = "passes")]
    pass_criteria: Option<StringOrList>,
}

#[derive(Debug, Deserialize, Default)]
struct ToolContractYaml {
    #[serde(default)]
    tool_contracts: Vec<ToolContractYamlItem>,
}

#[derive(Debug, Deserialize, Default)]
struct ToolContractYamlItem {
    #[serde(default)]
    id: Option<String>,
    #[serde(default)]
    name: Option<String>,
    #[serde(default)]
    tool: Option<String>,
    #[serde(default)]
    manifest: Option<String>,
    #[serde(default)]
    command: Option<String>,
    #[serde(default)]
    category: Option<String>,
    #[serde(default)]
    native: Option<serde_yaml::Value>,
    #[serde(default, alias = "toml")]
    native_toml: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
enum StringOrList {
    List(Vec<String>),
    String(String),
}

/// @spec projects/agentic-workflow/tech-design/semantic/agentic-workflow-cli.md#schema
impl StringOrList {
    fn into_vec(self) -> Vec<String> {
        match self {
            Self::List(values) => values
                .into_iter()
                .map(|value| value.trim().to_string())
                .filter(|value| !value.is_empty())
                .collect(),
            Self::String(value) => value
                .lines()
                .map(|line| line.trim().to_string())
                .filter(|line| !line.is_empty())
                .collect(),
        }
    }
}

/// @spec projects/agentic-workflow/tech-design/semantic/agentic-workflow-cli.md#schema
pub fn run(args: EcArgs) -> Result<()> {
    let project = args
        .project
        .ok_or_else(|| anyhow::anyhow!("ec requires --project <project>"))?;
    match args.command {
        EcCommand::Draft(args) => run_draft(&project, args),
        EcCommand::Fill(args) => run_fill(&project, args),
        EcCommand::Gen(args) => run_gen(&project, args),
        EcCommand::Check(args) => run_check(&project, args),
        EcCommand::Verify(args) => run_verify(&project, args),
        EcCommand::Doc(args) => run_doc(&project, args),
    }
}

/// @spec projects/agentic-workflow/tech-design/semantic/agentic-workflow-cli.md#schema
pub fn project_ec_check_summary(project: &str) -> Result<EcCheckSummary> {
    let project_root = crate::find_project_root()?;
    let ctx = resolve_ec_project_context(&project_root, project)?;
    check_ec_context(&ctx)
}

/// @spec projects/agentic-workflow/tech-design/semantic/agentic-workflow-cli.md#schema
pub fn load_project_ec_manifest(project: &str) -> Result<Option<(PathBuf, EcManifest)>> {
    let project_root = crate::find_project_root()?;
    let ctx = resolve_ec_project_context(&project_root, project)?;
    load_ec_manifest(&ctx)
}

fn run_draft(project: &str, args: EcDraftArgs) -> Result<()> {
    let project_root = crate::find_project_root()?;
    let ctx = resolve_ec_project_context(&project_root, project)?;
    let id = slugify(&args.id);
    if id.is_empty() {
        bail!("EC draft id cannot be empty");
    }
    let category = normalize_external_category(Some(&args.category), "behavior")?;
    let path = ctx.ec_root.join(&category).join(format!("{id}.md"));
    if path.exists() && !args.force {
        bail!(
            "{} already exists; pass --force to overwrite",
            relative_to(&ctx.project_root, &path)
        );
    }
    let title = args.title.clone().unwrap_or_else(|| title_case(&id));
    let content = render_ec_draft(&ctx, &id, &category, &title, &args);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }
    fs::write(&path, content).with_context(|| format!("write {}", path.display()))?;
    let rel = relative_to(&ctx.project_root, &path);
    if args.json {
        println!(
            "{}",
            serde_json::to_string_pretty(&serde_json::json!({
                "project": ctx.project,
                "path": rel,
                "id": id,
                "category": category,
            }))?
        );
    } else {
        println!("ec draft {}: wrote {}", ctx.project, rel);
        println!(
            "next: aw ec fill --project {} {} --section e2e-test --body-file <file>",
            ctx.project, rel
        );
        println!("then: aw ec gen --project {} --verify", ctx.project);
    }
    Ok(())
}

fn run_fill(project: &str, args: EcFillArgs) -> Result<()> {
    let project_root = crate::find_project_root()?;
    let ctx = resolve_ec_project_context(&project_root, project)?;
    let path = if args.path.is_absolute() {
        args.path.clone()
    } else {
        ctx.project_root.join(&args.path)
    };
    if !path.starts_with(&ctx.ec_root) {
        bail!(
            "EC fill target must be under {}; got {}",
            relative_to(&ctx.project_root, &ctx.ec_root),
            relative_to(&ctx.project_root, &path)
        );
    }
    let existing = fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
    let payload = fs::read_to_string(&args.body_file)
        .with_context(|| format!("read {}", args.body_file.display()))?;
    let merged = merge_ec_section(&existing, &args.section, &payload)?;
    fs::write(&path, merged).with_context(|| format!("write {}", path.display()))?;
    let rel = relative_to(&ctx.project_root, &path);
    if args.json {
        println!(
            "{}",
            serde_json::to_string_pretty(&serde_json::json!({
                "project": ctx.project,
                "path": rel,
                "section": args.section,
                "action": "filled",
            }))?
        );
    } else {
        println!(
            "ec fill {}: filled {} in {}",
            ctx.project, args.section, rel
        );
        println!("next: aw ec gen --project {} --verify", ctx.project);
    }
    Ok(())
}

fn render_ec_draft(
    ctx: &EcProjectContext,
    id: &str,
    category: &str,
    title: &str,
    args: &EcDraftArgs,
) -> String {
    let capability_id = args.capability_id.as_deref().unwrap_or("unmapped");
    let claim_id = args.claim_id.as_deref().unwrap_or(id);
    let contract_id = args.contract_id.as_deref().unwrap_or(id);
    let command = args.command.as_deref().unwrap_or("");
    let mut fill_sections = vec!["e2e-test"];
    if !args.tool.is_empty() {
        fill_sections.push("tool-contract");
    }
    let mut out = String::new();
    out.push_str("---\n");
    out.push_str(&format!("id: {id}\n"));
    out.push_str(&format!("summary: External contract for {title}.\n"));
    out.push_str(&format!("fill_sections: [{}]\n", fill_sections.join(", ")));
    out.push_str("---\n\n");
    out.push_str(&format!("# EC: {title}\n\n"));
    out.push_str("## External Contract\n");
    out.push_str("<!-- type: e2e-test lang: yaml -->\n\n");
    out.push_str("```yaml\n");
    out.push_str("e2e_tests:\n");
    out.push_str(&format!("  - id: {id}\n"));
    out.push_str(&format!("    capability_id: {capability_id}\n"));
    out.push_str(&format!("    claim_id: {claim_id}\n"));
    out.push_str(&format!("    contract_id: {contract_id}\n"));
    out.push_str(&format!("    category: {category}\n"));
    if !command.is_empty() {
        out.push_str(&format!("    command: {command:?}\n"));
    }
    out.push_str("    assertions:\n");
    out.push_str("      - \"Describe the externally observable guarantee.\"\n");
    out.push_str("```\n");
    if !args.tool.is_empty() {
        out.push_str("\n## Tool Contracts\n");
        out.push_str("<!-- type: tool-contract lang: yaml -->\n\n");
        out.push_str("```yaml\n");
        out.push_str("tool_contracts:\n");
        for tool in &args.tool {
            let tool = slugify(tool);
            if tool.is_empty() {
                continue;
            }
            let manifest = default_tool_contract_manifest_rel(&tool);
            out.push_str(&format!("  - id: {id}-{tool}\n"));
            out.push_str(&format!("    tool: {tool}\n"));
            out.push_str(&format!("    manifest: {manifest}\n"));
            out.push_str(&format!("    category: {category}\n"));
            out.push_str(&format!(
                "    command: {}\n",
                default_tool_command(ctx, &tool, &ctx.project_root.join(&manifest), id)
            ));
            out.push_str("    native:\n");
            out.push_str("      version: 1\n");
            out.push_str(&format!("      id: {id:?}\n"));
            out.push_str(&format!("      project: {:?}\n", ctx.project));
        }
        out.push_str("```\n");
    }
    out
}

fn default_tool_contract_manifest_rel(tool: &str) -> String {
    match tool {
        "rig" => "rig.toml".to_string(),
        "meter" => "meter.toml".to_string(),
        "arena" => "arena.toml".to_string(),
        "guard" => "guard.toml".to_string(),
        "vat" => "vat.toml".to_string(),
        _ => format!("{tool}.toml"),
    }
}

fn merge_ec_section(base_body: &str, section_type: &str, payload_body: &str) -> Result<String> {
    let payload_norm = {
        let trimmed = payload_body.trim_end_matches('\n');
        format!("{trimmed}\n")
    };
    let lines: Vec<&str> = base_body.split_inclusive('\n').collect();
    let mut matches: Vec<(usize, usize)> = Vec::new();
    for i in 0..lines.len() {
        if !lines[i].starts_with("## ") {
            continue;
        }
        let Some(next) = lines.get(i + 1) else {
            continue;
        };
        let Some(ann) = parse_ec_annotation(next.trim_end()) else {
            continue;
        };
        if ann.section_type != section_type {
            continue;
        }
        let mut end = lines.len();
        for j in (i + 1)..lines.len() {
            if lines[j].starts_with("## ") {
                end = j;
                break;
            }
        }
        matches.push((i, end));
    }

    let merged = if let Some((first_start, first_end)) = matches.first().copied() {
        let mut out: String = lines[..first_start].concat();
        out.push_str(&payload_norm);
        let mut cursor = first_end;
        for (dup_start, dup_end) in matches.iter().skip(1).copied() {
            out.push_str(&lines[cursor..dup_start].concat());
            cursor = dup_end;
        }
        out.push_str(&lines[cursor..].concat());
        out
    } else {
        let mut out = ensure_ec_fill_sections_has_section(base_body, section_type);
        if !out.ends_with("\n\n") {
            if !out.ends_with('\n') {
                out.push('\n');
            }
            out.push('\n');
        }
        out.push_str(&payload_norm);
        out
    };
    Ok(ensure_ec_fill_sections_has_section(&merged, section_type))
}

fn ensure_ec_fill_sections_has_section(content: &str, section_type: &str) -> String {
    let mut lines: Vec<String> = content.lines().map(str::to_string).collect();
    if lines.first().map(|line| line.trim()) != Some("---") {
        return content.to_string();
    }
    let Some(frontmatter_end) = lines
        .iter()
        .enumerate()
        .skip(1)
        .find_map(|(idx, line)| (line.trim() == "---").then_some(idx))
    else {
        return content.to_string();
    };
    for idx in 1..frontmatter_end {
        let trimmed = lines[idx].trim_start();
        let Some(rest) = trimmed.strip_prefix("fill_sections:") else {
            continue;
        };
        let indent_len = lines[idx].len() - trimmed.len();
        let indent = " ".repeat(indent_len);
        let inside = rest.trim().trim_start_matches('[').trim_end_matches(']');
        let mut sections = inside
            .split(',')
            .map(|part| part.trim().trim_matches('"').to_string())
            .filter(|part| !part.is_empty())
            .collect::<Vec<_>>();
        if !sections.iter().any(|section| section == section_type) {
            sections.push(section_type.to_string());
            lines[idx] = format!("{indent}fill_sections: [{}]", sections.join(", "));
        }
        return ensure_trailing_newline(&lines.join("\n"));
    }
    lines.insert(frontmatter_end, format!("fill_sections: [{section_type}]"));
    ensure_trailing_newline(&lines.join("\n"))
}

struct EcSectionAnnotation {
    section_type: String,
}

fn parse_ec_annotation(line: &str) -> Option<EcSectionAnnotation> {
    let trimmed = line.trim();
    let inner = trimmed.strip_prefix("<!--")?.strip_suffix("-->")?.trim();
    let section_type = inner
        .split_whitespace()
        .collect::<Vec<_>>()
        .windows(2)
        .find_map(|window| (window[0] == "type:").then(|| window[1].to_string()))?;
    Some(EcSectionAnnotation { section_type })
}

fn run_gen(project: &str, args: EcGenArgs) -> Result<()> {
    let project_root = crate::find_project_root()?;
    let ctx = resolve_ec_project_context(&project_root, project)?;
    let manifest = build_expected_manifest(&ctx)?;
    let generated_files = generated_ec_test_files(&ctx, &manifest);

    if args.dry_run {
        let dry_run = serde_json::json!({
            "project": ctx.project,
            "manifest_path": relative_to(&ctx.project_root, &ctx.manifest_path),
            "case_count": manifest.cases.len(),
            "tool_manifest_count": manifest.tool_manifests.len(),
            "generated_from_td_digest": manifest.generated_from_td_digest,
            "test_paths": manifest.cases.iter().map(|case| &case.test_path).collect::<Vec<_>>(),
            "tool_manifest_paths": manifest.tool_manifests.iter().map(|item| &item.path).collect::<Vec<_>>(),
        });
        if args.json {
            println!("{}", serde_json::to_string_pretty(&dry_run)?);
        } else {
            println!(
                "ec gen {}: dry-run, {} case(s), manifest {}",
                ctx.project,
                manifest.cases.len(),
                relative_to(&ctx.project_root, &ctx.manifest_path)
            );
            for case in &manifest.cases {
                println!("  - {} -> {}", case.id, case.test_path);
            }
            for item in &manifest.tool_manifests {
                println!("  - tool {} -> {}", item.id, item.path);
            }
        }
    } else {
        write_ec_manifest(&ctx, &manifest)?;
        for (path, content) in generated_files {
            write_generated_ec_test(&path, &content)?;
        }
        write_generated_tool_manifests(&ctx, &manifest)?;
        generate_case_skeletons(&ctx, &manifest)?;
        if args.json {
            println!(
                "{}",
                serde_json::to_string_pretty(&serde_json::json!({
                    "project": ctx.project,
                    "manifest_path": relative_to(&ctx.project_root, &ctx.manifest_path),
                    "case_count": manifest.cases.len(),
                    "tool_manifest_count": manifest.tool_manifests.len(),
                    "generated_from_td_digest": manifest.generated_from_td_digest,
                }))?
            );
        } else {
            println!(
                "ec gen {}: wrote {} case(s) to {}",
                ctx.project,
                manifest.cases.len(),
                relative_to(&ctx.project_root, &ctx.manifest_path)
            );
            if !manifest.tool_manifests.is_empty() {
                println!(
                    "ec gen {}: wrote {} native tool manifest(s)",
                    ctx.project,
                    manifest.tool_manifests.len()
                );
            }
        }
    }

    if args.verify {
        let summary = if args.dry_run {
            check_manifest_against_expected(&ctx, &manifest, None)?
        } else {
            check_ec_context(&ctx)?
        };
        if args.json {
            println!("{}", serde_json::to_string_pretty(&summary)?);
        } else if summary.clean {
            println!(
                "ec check {}: clean ({} case(s))",
                summary.project, summary.case_count
            );
        } else {
            print_ec_findings(&summary);
            bail!("ec check {} failed", summary.project);
        }
    }

    Ok(())
}

fn run_check(project: &str, args: EcCheckArgs) -> Result<()> {
    let project_root = crate::find_project_root()?;
    let ctx = resolve_ec_project_context(&project_root, project)?;
    let summary = check_ec_context(&ctx)?;
    if args.json {
        println!("{}", serde_json::to_string_pretty(&summary)?);
    } else if summary.clean {
        if summary.configured {
            println!(
                "ec check {}: clean ({} case(s))",
                summary.project, summary.case_count
            );
        } else {
            println!(
                "ec check {}: clean, no TD e2e-test or tool-contract sections found",
                summary.project
            );
        }
    } else {
        print_ec_findings(&summary);
    }
    if !summary.clean {
        std::process::exit(1);
    }
    Ok(())
}

fn run_verify(project: &str, args: EcVerifyArgs) -> Result<()> {
    let project_root = crate::find_project_root()?;
    let ctx = resolve_ec_project_context(&project_root, project)?;
    let summary = verify_ec_context(&ctx)?;
    if args.json {
        println!("{}", serde_json::to_string_pretty(&summary)?);
    } else if summary.clean {
        println!(
            "ec verify {}: passed ({}/{} command(s))",
            summary.project, summary.passed_count, summary.command_count
        );
    } else {
        println!(
            "ec verify {}: failed ({} failed / {} command(s))",
            summary.project, summary.failed_count, summary.command_count
        );
        for result in &summary.results {
            if result.status != "passed" {
                println!("  - {}: {}", result.case_id, result.command);
                if !result.stderr_tail.is_empty() {
                    println!("    stderr: {}", result.stderr_tail);
                }
            }
        }
    }
    if !summary.clean {
        std::process::exit(1);
    }
    Ok(())
}

fn run_doc(project: &str, args: EcDocArgs) -> Result<()> {
    match args.command {
        EcDocCommand::Gen(args) => run_doc_gen(project, args),
        EcDocCommand::Check(args) => run_doc_check(project, args),
        EcDocCommand::Preview(args) => run_doc_preview(project, args),
    }
}

fn run_doc_gen(project: &str, args: EcDocGenArgs) -> Result<()> {
    let project_root = crate::find_project_root()?;
    let ctx = resolve_ec_project_context(&project_root, project)?;
    let Some((_manifest_path, manifest)) = load_ec_manifest(&ctx)? else {
        bail!(
            "EC manifest missing at {}; run `aw ec gen --project {}` first",
            relative_to(&ctx.project_root, &ctx.manifest_path),
            ctx.project
        );
    };
    let content = render_ec_doc(&ctx, &manifest);
    if args.dry_run {
        if args.json {
            println!(
                "{}",
                serde_json::to_string_pretty(&serde_json::json!({
                    "project": ctx.project,
                    "doc_path": relative_to(&ctx.project_root, &ctx.doc_path),
                    "case_count": manifest.cases.len(),
                    "manifest_digest": manifest.generated_from_td_digest,
                    "content": content,
                }))?
            );
        } else {
            print!("{content}");
        }
    } else {
        write_ec_doc(&ctx, &content)?;
        if args.json {
            println!(
                "{}",
                serde_json::to_string_pretty(&serde_json::json!({
                    "project": ctx.project,
                    "doc_path": relative_to(&ctx.project_root, &ctx.doc_path),
                    "case_count": manifest.cases.len(),
                    "manifest_digest": manifest.generated_from_td_digest,
                }))?
            );
        } else {
            println!(
                "ec doc gen {}: wrote {} from {} case(s)",
                ctx.project,
                relative_to(&ctx.project_root, &ctx.doc_path),
                manifest.cases.len()
            );
        }
    }

    if args.verify {
        let summary = check_ec_doc_context(&ctx)?;
        if args.json {
            println!("{}", serde_json::to_string_pretty(&summary)?);
        } else if summary.clean {
            println!(
                "ec doc check {}: clean ({})",
                summary.project, summary.doc_path
            );
        } else {
            print_ec_doc_findings(&summary);
            bail!("ec doc check {} failed", summary.project);
        }
    }

    Ok(())
}

fn run_doc_check(project: &str, args: EcDocCheckArgs) -> Result<()> {
    let project_root = crate::find_project_root()?;
    let ctx = resolve_ec_project_context(&project_root, project)?;
    let summary = check_ec_doc_context(&ctx)?;
    if args.json {
        println!("{}", serde_json::to_string_pretty(&summary)?);
    } else if summary.clean {
        if summary.configured {
            println!(
                "ec doc check {}: clean ({})",
                summary.project, summary.doc_path
            );
        } else {
            println!(
                "ec doc check {}: clean, no EC manifest configured",
                summary.project
            );
        }
    } else {
        print_ec_doc_findings(&summary);
    }
    if !summary.clean {
        std::process::exit(1);
    }
    Ok(())
}

fn run_doc_preview(project: &str, args: EcDocPreviewArgs) -> Result<()> {
    let project_root = crate::find_project_root()?;
    let ctx = resolve_ec_project_context(&project_root, project)?;
    let summary = EcDocPreviewSummary {
        project: ctx.project.clone(),
        doc_path: relative_to(&ctx.project_root, &ctx.doc_path),
        exists: ctx.doc_path.is_file(),
    };
    if args.json {
        println!("{}", serde_json::to_string_pretty(&summary)?);
    } else if summary.exists {
        println!("ec doc preview {}: {}", summary.project, summary.doc_path);
    } else {
        println!(
            "ec doc preview {}: {} missing; run `aw ec doc gen --project {}`",
            summary.project, summary.doc_path, summary.project
        );
    }
    Ok(())
}

fn print_ec_findings(summary: &EcCheckSummary) {
    println!(
        "ec check {}: blocked ({} finding(s))",
        summary.project,
        summary.findings.len()
    );
    for finding in &summary.findings {
        println!("  - {finding}");
    }
}

fn print_ec_doc_findings(summary: &EcDocCheckSummary) {
    println!(
        "ec doc check {}: blocked ({} finding(s))",
        summary.project,
        summary.findings.len()
    );
    for finding in &summary.findings {
        println!("  - {finding}");
    }
}

fn resolve_ec_project_context(project_root: &Path, requested: &str) -> Result<EcProjectContext> {
    let row =
        crate::services::project_registry::resolve_project_config_row(project_root, requested)
            .with_context(|| format!("resolve project `{requested}`"))?;
    let source_root = project_root.join(&row.path);
    let ec_root = source_root.join(EC_SOURCE_REL);
    let td_root =
        crate::services::project_registry::resolve_td_root_from_config(project_root, &row.name)
            .map(|resolved| PathBuf::from(resolved.root))
            .map_err(|err| anyhow::anyhow!("{}", err.message))?;
    let tests_root = source_root.join("tests");
    let legacy_manifest_path = source_root.join(EC_MANIFEST_REL);
    let project_aw_path = source_root.join(PROJECT_AW_REL);
    let manifest_path = legacy_manifest_path.clone();
    let doc_path = source_root.join(EC_DOC_REL);
    let project_model = crate::services::project_registry::load_projects(project_root)?
        .into_iter()
        .find(|project| project.name == row.name);
    let target = project_model
        .as_ref()
        .into_iter()
        .flat_map(|project| project.workspaces.iter())
        .next()
        .map(|workspace| language_target_name(workspace.target).to_string())
        .unwrap_or_else(|| "rust".to_string());
    let package_name = package_name_for(&source_root).unwrap_or_else(|| row.name.clone());

    Ok(EcProjectContext {
        project_root: project_root.to_path_buf(),
        project: row.name,
        source_root,
        ec_root,
        td_root,
        tests_root,
        manifest_path,
        legacy_manifest_path,
        project_aw_path,
        doc_path,
        target,
        package_name,
    })
}

fn package_name_for(source_root: &Path) -> Option<String> {
    let content = fs::read_to_string(source_root.join("Cargo.toml")).ok()?;
    let value: toml::Value = toml::from_str(&content).ok()?;
    value
        .get("package")?
        .get("name")?
        .as_str()
        .map(|name| name.to_string())
}

fn language_target_name(language: crate::models::tech_stack::Language) -> &'static str {
    match language {
        crate::models::tech_stack::Language::Rust => "rust",
        crate::models::tech_stack::Language::Python => "python",
        crate::models::tech_stack::Language::JavaScript => "javascript",
        crate::models::tech_stack::Language::TypeScript => "typescript",
        crate::models::tech_stack::Language::Schemas => "schemas",
    }
}

fn build_expected_manifest(ctx: &EcProjectContext) -> Result<EcManifest> {
    let (mut cases, mut tool_manifests) = extract_ec_markdown_contracts(ctx)?;
    if cases.is_empty() && tool_manifests.is_empty() {
        cases = extract_td_e2e_cases(ctx)?;
        tool_manifests = extract_td_tool_manifests(ctx)?;
    }
    derive_required_for_production(ctx, &mut cases)?;
    cases.sort_by(|left, right| left.id.cmp(&right.id));
    tool_manifests.sort_by(|left, right| left.id.cmp(&right.id));
    let digest = digest_manifest_inputs(&cases, &tool_manifests);
    Ok(EcManifest {
        version: EC_MANIFEST_VERSION,
        project: ctx.project.clone(),
        generated_from_td_digest: digest,
        cases,
        tool_manifests,
    })
}

/// Derive each case's `required_for_production` from its capability's *type*.
///
/// TYPE -> which EC dimensions are production-required (structural). When a case's
/// capability has a type assigned in `.aw/capability-types.toml`, the derived
/// value (`case.category` is in the type's required dimensions) wins. Otherwise
/// the value already parsed from the YAML flag (`required_for_production`,
/// defaulting to `true`) is left untouched so existing projects don't break.
///
/// The type binding is loaded ONCE per generation, not per case. Maturity/env
/// (vat) deliberately plays no part here: it gates whether a contract is
/// verified/runnable, never whether it is required for production.
fn derive_required_for_production(
    ctx: &EcProjectContext,
    cases: &mut [EcManifestCase],
) -> Result<()> {
    // The README Capability Index pillar grouping is the primary source of a
    // capability's type; `.aw/capability-types.toml`, if present, overrides it.
    let readme_path = ctx.source_root.join("README.md");
    let mut types = crate::cli::capability_type::load_capability_types_from_readme(&readme_path)?;
    for (id, ty) in crate::cli::capability_type::load_capability_types(&ctx.project_root)? {
        types.insert(id, ty);
    }
    if types.is_empty() {
        return Ok(());
    }
    for case in cases.iter_mut() {
        if let Some(capability_type) = types.get(&case.capability_id) {
            case.required_for_production =
                crate::cli::capability_type::category_is_required_for_type(
                    capability_type,
                    &case.category,
                );
        }
    }
    Ok(())
}

fn extract_ec_markdown_contracts(
    ctx: &EcProjectContext,
) -> Result<(Vec<EcManifestCase>, Vec<EcToolManifest>)> {
    if !ctx.ec_root.is_dir() {
        return Ok((Vec::new(), Vec::new()));
    }

    let mut cases = Vec::new();
    let mut tool_manifests = Vec::new();
    for entry in WalkDir::new(&ctx.ec_root)
        .into_iter()
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.file_type().is_file())
    {
        let path = entry.path();
        if path.extension().and_then(|ext| ext.to_str()) != Some("md") {
            continue;
        }
        let content =
            fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
        cases.extend(extract_e2e_cases_from_markdown(ctx, path, &content)?);
        tool_manifests.extend(extract_tool_manifests_from_markdown(ctx, path, &content)?);
    }
    Ok((cases, tool_manifests))
}

fn extract_td_e2e_cases(ctx: &EcProjectContext) -> Result<Vec<EcManifestCase>> {
    if !ctx.td_root.is_dir() {
        return Ok(Vec::new());
    }

    let mut cases = Vec::new();
    for entry in WalkDir::new(&ctx.td_root)
        .into_iter()
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.file_type().is_file())
    {
        let path = entry.path();
        if path.extension().and_then(|ext| ext.to_str()) != Some("md") {
            continue;
        }
        let content =
            fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
        cases.extend(extract_e2e_cases_from_markdown(ctx, path, &content)?);
    }
    Ok(cases)
}

fn extract_td_tool_manifests(ctx: &EcProjectContext) -> Result<Vec<EcToolManifest>> {
    if !ctx.td_root.is_dir() {
        return Ok(Vec::new());
    }

    let mut manifests = Vec::new();
    for entry in WalkDir::new(&ctx.td_root)
        .into_iter()
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.file_type().is_file())
    {
        let path = entry.path();
        if path.extension().and_then(|ext| ext.to_str()) != Some("md") {
            continue;
        }
        let content =
            fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
        manifests.extend(extract_tool_manifests_from_markdown(ctx, path, &content)?);
    }
    Ok(manifests)
}

fn extract_e2e_cases_from_markdown(
    ctx: &EcProjectContext,
    path: &Path,
    content: &str,
) -> Result<Vec<EcManifestCase>> {
    let lines: Vec<&str> = content.lines().collect();
    let mut out = Vec::new();
    let mut idx = 0usize;
    while idx < lines.len() {
        let line = lines[idx];
        if line.contains("type: e2e-test") {
            let Some((yaml, next_idx)) = fenced_yaml_after(&lines, idx + 1) else {
                idx += 1;
                continue;
            };
            let parsed: E2eYaml = serde_yaml::from_str(&yaml)
                .with_context(|| format!("parse e2e-test YAML in {}", path.display()))?;
            for (case_idx, raw) in parsed.e2e_tests.into_iter().enumerate() {
                let raw_id = raw
                    .id
                    .clone()
                    .or_else(|| raw.name.clone())
                    .unwrap_or_else(|| {
                        format!(
                            "{}-{}",
                            path.file_stem()
                                .and_then(|stem| stem.to_str())
                                .unwrap_or("ec"),
                            case_idx + 1
                        )
                    });
                let id = slugify(&raw_id);
                let category = if path.starts_with(&ctx.ec_root) {
                    let fallback_category = external_contract_category_from_path(ctx, path);
                    normalize_external_category(raw.category.as_deref(), &fallback_category)?
                } else {
                    raw.category
                        .as_deref()
                        .map(slugify)
                        .filter(|value| !value.is_empty())
                        .unwrap_or_else(|| "behavior".to_string())
                };
                let default_test_path = ec_test_path(ctx, &category, &id);
                let test_path = raw
                    .test_path
                    .as_deref()
                    .map(str::trim)
                    .filter(|value| !value.is_empty())
                    .map(|value| normalize_project_relative_path(ctx, value))
                    .unwrap_or_else(|| relative_to(&ctx.project_root, &default_test_path));
                let command = raw
                    .command
                    .map(|value| value.trim().to_string())
                    .filter(|value| !value.is_empty())
                    .unwrap_or_else(|| default_ec_command(ctx, &ctx.project_root.join(&test_path)));
                let assertions = raw
                    .assertions
                    .or(raw.asserts)
                    .map(StringOrList::into_vec)
                    .unwrap_or_default();
                let evidence = raw
                    .evidence
                    .map(evidence_artifacts_from_yaml)
                    .unwrap_or_default();
                let evaluators = raw
                    .evaluators
                    .into_iter()
                    .map(evaluator_from_yaml)
                    .filter(|evaluator| !evaluator.id.is_empty())
                    .collect::<Vec<_>>();
                out.push(EcManifestCase {
                    id: id.clone(),
                    capability_id: raw
                        .capability_id
                        .map(|value| value.trim().to_string())
                        .filter(|value| !value.is_empty())
                        .unwrap_or_else(|| "unmapped".to_string()),
                    claim_id: raw
                        .claim_id
                        .map(|value| value.trim().to_string())
                        .filter(|value| !value.is_empty())
                        .unwrap_or_else(|| id.clone()),
                    contract_id: raw
                        .contract_id
                        .map(|value| value.trim().to_string())
                        .filter(|value| !value.is_empty())
                        .unwrap_or_else(|| id.clone()),
                    category,
                    td_ref: format!("{}#{}", relative_to(&ctx.project_root, path), id),
                    test_path,
                    command,
                    required_for_production: raw.required_for_production.unwrap_or(true),
                    assertions,
                    evidence,
                    evaluators,
                });
            }
            idx = next_idx;
            continue;
        }
        idx += 1;
    }
    Ok(out)
}

fn extract_tool_manifests_from_markdown(
    ctx: &EcProjectContext,
    path: &Path,
    content: &str,
) -> Result<Vec<EcToolManifest>> {
    let lines: Vec<&str> = content.lines().collect();
    let mut out = Vec::new();
    let mut idx = 0usize;
    while idx < lines.len() {
        let line = lines[idx];
        if line.contains("type: tool-contract") {
            let Some((yaml, next_idx)) = fenced_yaml_after(&lines, idx + 1) else {
                idx += 1;
                continue;
            };
            let parsed: ToolContractYaml = serde_yaml::from_str(&yaml)
                .with_context(|| format!("parse tool-contract YAML in {}", path.display()))?;
            for (contract_idx, raw) in parsed.tool_contracts.into_iter().enumerate() {
                let raw_id = raw
                    .id
                    .clone()
                    .or_else(|| raw.name.clone())
                    .unwrap_or_else(|| {
                        format!(
                            "{}-tool-{}",
                            path.file_stem()
                                .and_then(|stem| stem.to_str())
                                .unwrap_or("ec"),
                            contract_idx + 1
                        )
                    });
                let id = slugify(&raw_id);
                let tool = raw
                    .tool
                    .map(|value| slugify(&value))
                    .filter(|value| !value.is_empty())
                    .unwrap_or_else(|| "tool".to_string());
                let manifest_rel = raw
                    .manifest
                    .map(|value| value.trim().to_string())
                    .filter(|value| !value.is_empty())
                    .unwrap_or_else(|| format!("{tool}.toml"));
                let path_abs = native_manifest_path(ctx, &manifest_rel);
                let generated_toml = render_tool_contract_toml(raw.native_toml, raw.native)
                    .with_context(|| format!("render native tool manifest `{id}`"))?;
                let td_ref = format!("{}#{}", relative_to(&ctx.project_root, path), raw_id);
                let generated_toml = wrap_generated_tool_manifest(&td_ref, &generated_toml);
                let content_digest = digest_string(&generated_toml);
                out.push(EcToolManifest {
                    id,
                    tool,
                    path: relative_to(&ctx.project_root, &path_abs),
                    td_ref,
                    content_digest,
                    command: raw
                        .command
                        .map(|value| value.trim().to_string())
                        .unwrap_or_default(),
                    category: raw
                        .category
                        .map(|value| slugify(&value))
                        .unwrap_or_default(),
                    generated_toml,
                });
            }
            idx = next_idx;
            continue;
        }
        idx += 1;
    }
    Ok(out)
}

fn native_manifest_path(ctx: &EcProjectContext, manifest_rel: &str) -> PathBuf {
    let manifest = Path::new(manifest_rel);
    if manifest.is_absolute()
        || manifest_rel.starts_with("projects/")
        || manifest_rel.starts_with("crates/")
        || manifest_rel.starts_with("packages/")
        || manifest_rel.starts_with(".")
    {
        ctx.project_root.join(manifest)
    } else {
        ctx.source_root.join(manifest)
    }
}

fn external_contract_category_from_path(ctx: &EcProjectContext, path: &Path) -> String {
    // Scan all path components for the dimension dir, so both the flat
    // `external-contracts/<dimension>/` and the capability-first
    // `external-contracts/<capability>/<dimension>/` layouts derive correctly.
    path.strip_prefix(&ctx.ec_root)
        .ok()
        .and_then(|relative| {
            relative
                .components()
                .filter_map(|component| component.as_os_str().to_str())
                .map(slugify)
                .find(|value| EC_CATEGORIES.contains(&value.as_str()))
        })
        .unwrap_or_else(|| "behavior".to_string())
}

fn normalize_external_category(raw: Option<&str>, fallback: &str) -> Result<String> {
    let category = raw
        .map(slugify)
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| fallback.to_string());
    if EC_CATEGORIES.contains(&category.as_str()) {
        Ok(category)
    } else {
        bail!(
            "external contract category `{category}` is unsupported; expected behavior|efficiency|security|stability"
        )
    }
}

fn normalize_project_relative_path(ctx: &EcProjectContext, value: &str) -> String {
    let trimmed = value.trim();
    if trimmed.starts_with("projects/")
        || trimmed.starts_with("crates/")
        || trimmed.starts_with("packages/")
        || trimmed.starts_with(".")
        || Path::new(trimmed).is_absolute()
    {
        trimmed.to_string()
    } else {
        relative_to(&ctx.project_root, &ctx.source_root.join(trimmed))
    }
}

fn default_tool_command(ctx: &EcProjectContext, tool: &str, path: &Path, id: &str) -> String {
    let rel = relative_to(&ctx.project_root, path);
    match tool {
        "arena" => format!("arena run --spec {rel}"),
        "rig" => path
            .parent()
            .map(|parent| format!("rig test --dir {}", relative_to(&ctx.project_root, parent)))
            .unwrap_or_else(|| format!("rig test --dir {rel}")),
        "meter" => format!("meter run --target {rel}"),
        "guard" => format!(
            "guard scan {} --compact --no-persist",
            relative_to(&ctx.project_root, &ctx.source_root)
        ),
        "vat" => format!("vat run {id}"),
        _ => format!("sh {rel}"),
    }
}

/// Which executable artifact `aw ec gen` should skeleton for a claim, dispatched
/// on the gate command: `rig test` -> a lifecycle case TOML (mode-1, rig DSL);
/// `cargo test` -> a native Rust `#[test]` body (mode-2); anything else -> none.
#[derive(Debug, PartialEq, Eq)]
enum CaseGenMode {
    Rig,
    NativeRust,
    Other,
}

fn case_gen_mode(case: &EcManifestCase) -> CaseGenMode {
    let cmd = case.command.trim_start();
    if cmd.starts_with("rig test") {
        CaseGenMode::Rig
    } else if cmd.starts_with("cargo test") {
        CaseGenMode::NativeRust
    } else {
        CaseGenMode::Other
    }
}

fn rig_dir_from_command(cmd: &str) -> Option<&str> {
    cmd.split("--dir ")
        .nth(1)
        .and_then(|s| s.split_whitespace().next())
}

fn cargo_test_target(cmd: &str) -> Option<&str> {
    cmd.split("--test ")
        .nth(1)
        .and_then(|s| s.split_whitespace().next())
}

fn sanitize_ident(s: &str) -> String {
    s.chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() {
                c.to_ascii_lowercase()
            } else {
                '_'
            }
        })
        .collect()
}

/// Mode-1: a lifecycle case-TOML skeleton from a rig-integrated claim. The
/// `[exercise]` is a placeholder for the author; `source_contract` back-links to
/// the EC claim (the bidirectional link).
fn render_case_toml_skeleton(
    case: &EcManifestCase,
    suite: &str,
    dimension: &str,
    stem: &str,
) -> String {
    let subject = case
        .assertions
        .first()
        .map(|s| s.as_str())
        .unwrap_or("fill: the behavior under test");
    format!(
        "# SPEC-MANAGED: generated by `aw ec gen` from EC claim `{contract}` — fill [exercise].\n\
[case]\n\
id = \"{stem}\"\n\
suite = \"{suite}\"\n\
dimension = \"{dimension}\"\n\
subject = \"{subject}\"\n\
expected = \"pass\"\n\
required = {required}\n\
source_contract = \"{contract}\"\n\
\n\
[prepare]\n\
needs = []\n\
\n\
[exercise]\n\
# n = 200   # uncomment + set for a load case\n\
[exercise.request]\n\
method = \"GET\"\n\
url = \"http://{{{{upstream}}}}/REPLACE_ME\"\n\
[exercise.request.expect]\n\
status = 200\n\
\n\
[clean]\n\
delegate = \"vat-cow\"\n",
        contract = case.contract_id,
        stem = stem,
        suite = suite,
        dimension = dimension,
        subject = subject,
        required = case.required_for_production,
    )
}

/// Mode-2: a native Rust `#[test]` body skeleton from a non-rig claim. The author
/// fills the in-process drive + assertions (goes beyond a gate wrapper).
fn render_native_rust_skeleton(case: &EcManifestCase, fn_name: &str) -> String {
    let asserts: String = case
        .assertions
        .iter()
        .map(|a| format!("    // contract: {a}\n"))
        .collect();
    format!(
        "// SPEC-MANAGED: generated by `aw ec gen` from EC claim `{contract}` — fill the body.\n\
// @ec {id}\n\
// @capability {cap}\n\
#[test]\n\
fn {fn_name}() {{\n\
{asserts}    // fill: drive the in-process target and assert the contract above.\n\
    todo!(\"implement EC claim {contract}\");\n\
}}\n",
        contract = case.contract_id,
        id = case.id,
        cap = case.capability_id,
        fn_name = fn_name,
        asserts = asserts,
    )
}

/// Generate an executable skeleton per claim: rig-integrated -> case TOML;
/// native rust -> a `#[test]` body. Skips any file that already exists (same
/// guard as gate tests) so hand-authored cases are never clobbered.
fn generate_case_skeletons(ctx: &EcProjectContext, manifest: &EcManifest) -> Result<()> {
    for case in &manifest.cases {
        match case_gen_mode(case) {
            CaseGenMode::Rig => {
                let Some(dir) = rig_dir_from_command(&case.command) else {
                    continue;
                };
                let dimension = Path::new(dir)
                    .file_name()
                    .and_then(|s| s.to_str())
                    .unwrap_or("behavior");
                let stem = sanitize_ident(&case.contract_id);
                let path = ctx.project_root.join(dir).join(format!("{stem}.toml"));
                let body = render_case_toml_skeleton(case, &ctx.project, dimension, &stem);
                write_skeleton_if_absent(&path, &body)?;
            }
            CaseGenMode::NativeRust => {
                let Some(target) = cargo_test_target(&case.command) else {
                    continue;
                };
                let test_dir = Path::new(&case.test_path)
                    .parent()
                    .unwrap_or_else(|| Path::new("tests"));
                let path = ctx.project_root.join(test_dir).join(format!("{target}.rs"));
                let fn_name = sanitize_ident(target);
                let body = render_native_rust_skeleton(case, &fn_name);
                write_skeleton_if_absent(&path, &body)?;
            }
            CaseGenMode::Other => {}
        }
    }
    Ok(())
}

fn write_skeleton_if_absent(path: &Path, body: &str) -> Result<()> {
    if path.exists() {
        return Ok(());
    }
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(path, body)?;
    Ok(())
}

fn render_tool_contract_toml(
    native_toml: Option<String>,
    native: Option<serde_yaml::Value>,
) -> Result<String> {
    if let Some(content) = native_toml {
        return Ok(ensure_trailing_newline(content.trim()));
    }
    let Some(native) = native else {
        return Ok(String::new());
    };
    let toml_value = yaml_value_to_toml(native)?;
    let content = toml::to_string_pretty(&toml_value).context("serializing native TOML")?;
    Ok(ensure_trailing_newline(content.trim()))
}

fn wrap_generated_tool_manifest(td_ref: &str, content: &str) -> String {
    format!(
        "# SPEC-MANAGED: {td_ref}\n# CODEGEN-BEGIN\n# {EC_TOOL_BEGIN_MARKER}\n{}# {EC_TOOL_END_MARKER}\n# CODEGEN-END\n",
        ensure_trailing_newline(content.trim())
    )
}

fn yaml_value_to_toml(value: serde_yaml::Value) -> Result<toml::Value> {
    Ok(match value {
        serde_yaml::Value::Null => bail!("native TOML payload cannot contain null"),
        serde_yaml::Value::Bool(value) => toml::Value::Boolean(value),
        serde_yaml::Value::Number(value) => {
            if let Some(i) = value.as_i64() {
                toml::Value::Integer(i)
            } else if let Some(f) = value.as_f64() {
                toml::Value::Float(f)
            } else {
                bail!("native TOML payload contains unsupported number")
            }
        }
        serde_yaml::Value::String(value) => toml::Value::String(value),
        serde_yaml::Value::Sequence(values) => toml::Value::Array(
            values
                .into_iter()
                .map(yaml_value_to_toml)
                .collect::<Result<Vec<_>>>()?,
        ),
        serde_yaml::Value::Mapping(mapping) => {
            let mut table = toml::map::Map::new();
            for (key, value) in mapping {
                let Some(key) = key.as_str() else {
                    bail!("native TOML payload contains non-string key")
                };
                table.insert(key.to_string(), yaml_value_to_toml(value)?);
            }
            toml::Value::Table(table)
        }
        serde_yaml::Value::Tagged(tagged) => yaml_value_to_toml(tagged.value)?,
    })
}

fn evaluator_from_yaml(raw: E2eEvaluatorYaml) -> EcEvaluator {
    let tool = raw
        .tool
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| "agent".to_string());
    let id = raw
        .id
        .map(|value| slugify(&value))
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| slugify(&tool));
    EcEvaluator {
        id,
        tool,
        command: raw
            .command
            .map(|value| value.trim().to_string())
            .unwrap_or_default(),
        report_path: raw
            .report_path
            .map(|value| value.trim().to_string())
            .unwrap_or_default(),
        prompt: raw
            .prompt
            .map(|value| value.trim().to_string())
            .unwrap_or_default(),
        rubric: raw.rubric.map(StringOrList::into_vec).unwrap_or_default(),
        pass_criteria: raw
            .pass_criteria
            .map(StringOrList::into_vec)
            .unwrap_or_default(),
    }
}

fn evidence_artifacts_from_yaml(evidence: E2eEvidenceYaml) -> Vec<EcEvidenceArtifact> {
    let mut artifacts = Vec::new();
    for item in evidence.screenshots {
        push_evidence_artifact(&mut artifacts, "screenshot", item);
    }
    for item in evidence.reports {
        push_evidence_artifact(&mut artifacts, "report", item);
    }
    for item in evidence.docs {
        push_evidence_artifact(&mut artifacts, "doc", item);
    }
    if let Some(item) = evidence.eval {
        push_evidence_artifact(&mut artifacts, "eval", item);
    }
    artifacts
}

fn push_evidence_artifact(
    artifacts: &mut Vec<EcEvidenceArtifact>,
    default_kind: &str,
    item: E2eArtifactYaml,
) {
    let Some(path) = item
        .path
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
    else {
        return;
    };
    let kind = item
        .kind
        .or(item.id)
        .map(|value| slugify(&value))
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| default_kind.to_string());
    artifacts.push(EcEvidenceArtifact {
        kind,
        path,
        label: item
            .label
            .map(|value| value.trim().to_string())
            .unwrap_or_default(),
        locator: item
            .locator
            .map(|value| value.trim().to_string())
            .unwrap_or_default(),
    });
}

fn markdown_heading_title(line: &str) -> Option<String> {
    let trimmed = line.trim_start();
    let hashes = trimmed.chars().take_while(|ch| *ch == '#').count();
    if hashes < 2 {
        return None;
    }
    let rest = trimmed[hashes..].trim();
    if rest.is_empty() {
        None
    } else {
        Some(rest.to_string())
    }
}

fn fenced_yaml_after(lines: &[&str], mut idx: usize) -> Option<(String, usize)> {
    while idx < lines.len() {
        let line = lines[idx].trim();
        if line.starts_with("```") {
            break;
        }
        if markdown_heading_title(lines[idx]).is_some() {
            return None;
        }
        idx += 1;
    }
    if idx >= lines.len() {
        return None;
    }
    idx += 1;
    let start = idx;
    while idx < lines.len() {
        if lines[idx].trim_start().starts_with("```") {
            return Some((lines[start..idx].join("\n"), idx + 1));
        }
        idx += 1;
    }
    None
}

fn ec_test_path(ctx: &EcProjectContext, category: &str, id: &str) -> PathBuf {
    match ctx.target.as_str() {
        "python" => ctx.tests_root.join(category).join(format!("test_{id}.py")),
        "typescript" | "javascript" | "ts" | "js" => {
            ctx.tests_root.join(category).join(format!("{id}.spec.ts"))
        }
        "rust" => ctx
            .tests_root
            .join(format!("{category}_{}.rs", rust_ident(id))),
        _ => ctx.tests_root.join(format!("{category}_{id}.txt")),
    }
}

fn default_ec_command(ctx: &EcProjectContext, test_path: &Path) -> String {
    match ctx.target.as_str() {
        "python" => format!("pytest {}", relative_to(&ctx.project_root, test_path)),
        "typescript" | "javascript" | "ts" | "js" => {
            format!(
                "npx vitest run {}",
                relative_to(&ctx.project_root, test_path)
            )
        }
        "rust" => {
            let stem = test_path
                .file_stem()
                .and_then(|stem| stem.to_str())
                .unwrap_or("aw_ec");
            format!(
                "cargo test -p {} --test {} -- --ignored",
                ctx.package_name, stem
            )
        }
        _ => format!(
            "# fill EC command for {}",
            relative_to(&ctx.project_root, test_path)
        ),
    }
}

fn digest_manifest_inputs(cases: &[EcManifestCase], tool_manifests: &[EcToolManifest]) -> String {
    let mut sorted = cases.to_vec();
    sorted.sort_by(|left, right| left.id.cmp(&right.id));
    let mut hasher = Sha256::new();
    for case in sorted {
        hash_field(&mut hasher, &case.id);
        hash_field(&mut hasher, &case.capability_id);
        hash_field(&mut hasher, &case.claim_id);
        hash_field(&mut hasher, &case.contract_id);
        hash_field(&mut hasher, &case.category);
        hash_field(&mut hasher, &case.td_ref);
        hash_field(&mut hasher, &case.test_path);
        hash_field(&mut hasher, &case.command);
        hash_field(
            &mut hasher,
            if case.required_for_production {
                "true"
            } else {
                "false"
            },
        );
        for assertion in &case.assertions {
            hash_field(&mut hasher, assertion);
        }
        for artifact in &case.evidence {
            hash_field(&mut hasher, &artifact.kind);
            hash_field(&mut hasher, &artifact.path);
            hash_field(&mut hasher, &artifact.label);
            hash_field(&mut hasher, &artifact.locator);
        }
        for evaluator in &case.evaluators {
            hash_field(&mut hasher, &evaluator.id);
            hash_field(&mut hasher, &evaluator.tool);
            hash_field(&mut hasher, &evaluator.command);
            hash_field(&mut hasher, &evaluator.report_path);
            hash_field(&mut hasher, &evaluator.prompt);
            for rubric in &evaluator.rubric {
                hash_field(&mut hasher, rubric);
            }
            for criterion in &evaluator.pass_criteria {
                hash_field(&mut hasher, criterion);
            }
        }
    }
    let mut sorted_tools = tool_manifests.to_vec();
    sorted_tools.sort_by(|left, right| left.id.cmp(&right.id));
    for manifest in sorted_tools {
        hash_field(&mut hasher, &manifest.id);
        hash_field(&mut hasher, &manifest.tool);
        hash_field(&mut hasher, &manifest.path);
        hash_field(&mut hasher, &manifest.td_ref);
        hash_field(&mut hasher, &manifest.content_digest);
        hash_field(&mut hasher, &manifest.command);
        hash_field(&mut hasher, &manifest.category);
    }
    format!("sha256:{:x}", hasher.finalize())
}

fn digest_string(value: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(value.as_bytes());
    format!("sha256:{:x}", hasher.finalize())
}

fn hash_field(hasher: &mut Sha256, value: &str) {
    hasher.update(value.as_bytes());
    hasher.update([0]);
}

fn ensure_trailing_newline(value: &str) -> String {
    if value.ends_with('\n') {
        value.to_string()
    } else {
        format!("{value}\n")
    }
}

fn default_required_for_production() -> bool {
    true
}

fn load_ec_manifest(ctx: &EcProjectContext) -> Result<Option<(PathBuf, EcManifest)>> {
    if !ctx.legacy_manifest_path.is_file() {
        return Ok(None);
    }
    let content = fs::read_to_string(&ctx.legacy_manifest_path)
        .with_context(|| format!("read {}", ctx.legacy_manifest_path.display()))?;
    let manifest: EcManifest = toml::from_str(&content)
        .with_context(|| format!("parse {}", ctx.legacy_manifest_path.display()))?;
    Ok(Some((ctx.legacy_manifest_path.clone(), manifest)))
}

fn check_ec_context(ctx: &EcProjectContext) -> Result<EcCheckSummary> {
    let expected = build_expected_manifest(ctx)?;
    let loaded = load_ec_manifest(ctx)?;
    check_manifest_against_expected(
        ctx,
        &expected,
        loaded.as_ref().map(|(_, manifest)| manifest),
    )
}

fn check_manifest_against_expected(
    ctx: &EcProjectContext,
    expected: &EcManifest,
    actual: Option<&EcManifest>,
) -> Result<EcCheckSummary> {
    let mut findings = Vec::new();
    let mut missing_test_paths = Vec::new();
    let mut orphan_test_paths = Vec::new();
    let mut missing_tool_manifest_paths = Vec::new();
    let manifest_path = relative_to(&ctx.project_root, &ctx.manifest_path);
    let configured = actual.is_some();
    let manifest_td_digest = actual.map(|manifest| manifest.generated_from_td_digest.clone());

    let actual_cases = actual
        .map(|manifest| manifest.cases.clone())
        .unwrap_or_default();

    if let Some(manifest) = actual {
        if manifest.version != EC_MANIFEST_VERSION {
            findings.push(format!(
                "{} has unsupported version {}; expected {}",
                manifest_path, manifest.version, EC_MANIFEST_VERSION
            ));
        }
        if manifest.project != ctx.project {
            findings.push(format!(
                "{} project is `{}`; expected `{}`",
                manifest_path, manifest.project, ctx.project
            ));
        }
        if manifest.generated_from_td_digest != expected.generated_from_td_digest {
            findings.push(format!(
                "{} is stale: manifest digest {}, current TD digest {}",
                manifest_path, manifest.generated_from_td_digest, expected.generated_from_td_digest
            ));
        }
    } else if !expected.cases.is_empty() || !expected.tool_manifests.is_empty() {
        findings.push(format!(
            "EC manifest missing at {}; run `aw ec gen --project {}`",
            manifest_path, ctx.project
        ));
    }

    let expected_by_id = expected
        .cases
        .iter()
        .map(|case| (case.id.as_str(), case))
        .collect::<BTreeMap<_, _>>();
    let actual_by_id = actual_cases
        .iter()
        .map(|case| (case.id.as_str(), case))
        .collect::<BTreeMap<_, _>>();

    for expected_case in &expected.cases {
        let Some(actual_case) = actual_by_id.get(expected_case.id.as_str()) else {
            if actual.is_some() {
                findings.push(format!(
                    "manifest missing EC case `{}` from {}",
                    expected_case.id, expected_case.td_ref
                ));
            }
            continue;
        };
        compare_case_field(
            &mut findings,
            &expected_case.id,
            "capability_id",
            &expected_case.capability_id,
            &actual_case.capability_id,
        );
        compare_case_field(
            &mut findings,
            &expected_case.id,
            "claim_id",
            &expected_case.claim_id,
            &actual_case.claim_id,
        );
        compare_case_field(
            &mut findings,
            &expected_case.id,
            "contract_id",
            &expected_case.contract_id,
            &actual_case.contract_id,
        );
        compare_case_field(
            &mut findings,
            &expected_case.id,
            "category",
            &expected_case.category,
            &actual_case.category,
        );
        compare_case_field(
            &mut findings,
            &expected_case.id,
            "td_ref",
            &expected_case.td_ref,
            &actual_case.td_ref,
        );
        compare_case_field(
            &mut findings,
            &expected_case.id,
            "test_path",
            &expected_case.test_path,
            &actual_case.test_path,
        );
        compare_case_field(
            &mut findings,
            &expected_case.id,
            "command",
            &expected_case.command,
            &actual_case.command,
        );
        if expected_case.required_for_production != actual_case.required_for_production {
            findings.push(format!(
                "manifest case `{}` required_for_production drifted",
                expected_case.id
            ));
        }
        if expected_case.assertions != actual_case.assertions {
            findings.push(format!(
                "manifest case `{}` assertions drifted",
                expected_case.id
            ));
        }
        if expected_case.evidence != actual_case.evidence {
            findings.push(format!(
                "manifest case `{}` evidence artifacts drifted",
                expected_case.id
            ));
        }
        if expected_case.evaluators != actual_case.evaluators {
            findings.push(format!(
                "manifest case `{}` evaluators drifted",
                expected_case.id
            ));
        }
    }

    for actual_case in &actual_cases {
        if !expected_by_id.contains_key(actual_case.id.as_str()) {
            findings.push(format!(
                "manifest has orphan EC case `{}` not present in ec/ markdown or legacy TD EC sections",
                actual_case.id
            ));
        }
        let test_path = ctx.project_root.join(&actual_case.test_path);
        if !test_path.is_file() {
            missing_test_paths.push(actual_case.test_path.clone());
            findings.push(format!(
                "EC test file missing for case `{}`: {}",
                actual_case.id, actual_case.test_path
            ));
        } else {
            let content = fs::read_to_string(&test_path)
                .with_context(|| format!("read {}", test_path.display()))?;
            if !content.contains(EC_BEGIN_MARKER) || !content.contains(&actual_case.id) {
                findings.push(format!(
                    "EC test file {} is missing generated EC marker metadata for `{}`",
                    actual_case.test_path, actual_case.id
                ));
            } else {
                let expected_content = render_ec_test(ctx, actual_case);
                if content != expected_content {
                    findings.push(format!(
                        "EC test file {} generated content drifted for `{}`; run `aw ec gen --project {}`",
                        actual_case.test_path, actual_case.id, ctx.project
                    ));
                }
            }
        }
    }

    let expected_tool_by_id = expected
        .tool_manifests
        .iter()
        .map(|manifest| (manifest.id.as_str(), manifest))
        .collect::<BTreeMap<_, _>>();
    let actual_tool_manifests = actual
        .map(|manifest| manifest.tool_manifests.clone())
        .unwrap_or_default();
    let actual_tool_by_id = actual_tool_manifests
        .iter()
        .map(|manifest| (manifest.id.as_str(), manifest))
        .collect::<BTreeMap<_, _>>();

    for expected_manifest in &expected.tool_manifests {
        let Some(actual_manifest) = actual_tool_by_id.get(expected_manifest.id.as_str()) else {
            if actual.is_some() {
                findings.push(format!(
                    "manifest missing tool contract `{}` from {}",
                    expected_manifest.id, expected_manifest.td_ref
                ));
            }
            continue;
        };
        compare_case_field(
            &mut findings,
            &expected_manifest.id,
            "tool",
            &expected_manifest.tool,
            &actual_manifest.tool,
        );
        compare_case_field(
            &mut findings,
            &expected_manifest.id,
            "path",
            &expected_manifest.path,
            &actual_manifest.path,
        );
        compare_case_field(
            &mut findings,
            &expected_manifest.id,
            "td_ref",
            &expected_manifest.td_ref,
            &actual_manifest.td_ref,
        );
        compare_case_field(
            &mut findings,
            &expected_manifest.id,
            "content_digest",
            &expected_manifest.content_digest,
            &actual_manifest.content_digest,
        );
        compare_case_field(
            &mut findings,
            &expected_manifest.id,
            "command",
            &expected_manifest.command,
            &actual_manifest.command,
        );
        compare_case_field(
            &mut findings,
            &expected_manifest.id,
            "category",
            &expected_manifest.category,
            &actual_manifest.category,
        );

        let path = ctx.project_root.join(&actual_manifest.path);
        if !path.is_file() {
            missing_tool_manifest_paths.push(actual_manifest.path.clone());
            findings.push(format!(
                "native tool manifest missing for `{}`: {}",
                actual_manifest.id, actual_manifest.path
            ));
        } else {
            let content =
                fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
            let digest = digest_string(&content);
            if digest != expected_manifest.content_digest {
                findings.push(format!(
                    "native tool manifest {} content drifted for `{}`; run `aw ec gen --project {}`",
                    actual_manifest.path, actual_manifest.id, ctx.project
                ));
            }
        }
    }

    for actual_manifest in &actual_tool_manifests {
        if !expected_tool_by_id.contains_key(actual_manifest.id.as_str()) {
            findings.push(format!(
                "manifest has orphan tool contract `{}` not present in ec/ markdown or legacy TD tool-contract sections",
                actual_manifest.id
            ));
        }
    }

    let manifest_paths = actual_cases
        .iter()
        .map(|case| case.test_path.as_str())
        .collect::<BTreeSet<_>>();
    for orphan_path in generated_ec_test_paths(ctx)? {
        if !manifest_paths.contains(orphan_path.as_str()) {
            findings.push(format!(
                "generated EC test file is not listed in manifest: {orphan_path}"
            ));
            orphan_test_paths.push(orphan_path);
        }
    }

    findings.sort();
    findings.dedup();
    missing_test_paths.sort();
    missing_test_paths.dedup();
    orphan_test_paths.sort();
    orphan_test_paths.dedup();
    missing_tool_manifest_paths.sort();
    missing_tool_manifest_paths.dedup();
    let stale = actual
        .map(|manifest| manifest.generated_from_td_digest != expected.generated_from_td_digest)
        .unwrap_or(!expected.cases.is_empty() || !expected.tool_manifests.is_empty());

    Ok(EcCheckSummary {
        project: ctx.project.clone(),
        clean: findings.is_empty(),
        configured,
        manifest_path,
        generated_from_td_digest: expected.generated_from_td_digest.clone(),
        manifest_td_digest,
        expected_case_count: expected.cases.len(),
        case_count: actual_cases.len(),
        expected_tool_manifest_count: expected.tool_manifests.len(),
        tool_manifest_count: actual_tool_manifests.len(),
        stale,
        missing_test_paths,
        orphan_test_paths,
        missing_tool_manifest_paths,
        findings,
    })
}

fn check_ec_doc_context(ctx: &EcProjectContext) -> Result<EcDocCheckSummary> {
    let mut findings = Vec::new();
    let doc_path = relative_to(&ctx.project_root, &ctx.doc_path);
    let manifest_path = relative_to(&ctx.project_root, &ctx.manifest_path);
    let loaded = load_ec_manifest(ctx)?;
    let configured = loaded.is_some();
    let manifest = loaded.as_ref().map(|(_, manifest)| manifest);

    if let Some(manifest) = manifest {
        let ec_summary = check_ec_context(ctx)?;
        for finding in ec_summary.findings {
            findings.push(format!("EC manifest is not clean: {finding}"));
        }

        let expected_content = render_ec_doc(ctx, manifest);
        if !ctx.doc_path.is_file() {
            findings.push(format!(
                "EC doc missing at {}; run `aw ec doc gen --project {}`",
                doc_path, ctx.project
            ));
        } else {
            let content = fs::read_to_string(&ctx.doc_path)
                .with_context(|| format!("read {}", ctx.doc_path.display()))?;
            if !content.contains(EC_DOC_BEGIN_MARKER) || !content.contains(EC_DOC_END_MARKER) {
                findings.push(format!(
                    "EC doc {} is missing generated EC doc markers",
                    doc_path
                ));
            } else if content != expected_content {
                findings.push(format!(
                    "EC doc {} generated content drifted; run `aw ec doc gen --project {}`",
                    doc_path, ctx.project
                ));
            }
        }
    } else {
        let expected = build_expected_manifest(ctx)?;
        if !expected.cases.is_empty() || !expected.tool_manifests.is_empty() {
            findings.push(format!(
                "EC manifest missing at {}; run `aw ec gen --project {}` before `aw ec doc gen --project {}`",
                manifest_path, ctx.project, ctx.project
            ));
        }
    }

    findings.sort();
    findings.dedup();
    Ok(EcDocCheckSummary {
        project: ctx.project.clone(),
        clean: findings.is_empty(),
        configured,
        doc_path,
        manifest_path,
        manifest_digest: manifest.map(|manifest| manifest.generated_from_td_digest.clone()),
        case_count: manifest
            .map(|manifest| manifest.cases.len())
            .unwrap_or_default(),
        findings,
    })
}

fn verify_ec_context(ctx: &EcProjectContext) -> Result<EcVerifySummary> {
    let Some((manifest_path, manifest)) = load_ec_manifest(ctx)? else {
        bail!(
            "EC manifest missing at {}; run `aw ec gen --project {}` first",
            relative_to(&ctx.project_root, &ctx.manifest_path),
            ctx.project
        );
    };
    let mut results = Vec::new();
    for case in &manifest.cases {
        let output = Command::new("sh")
            .arg("-c")
            .arg(&case.command)
            .current_dir(&ctx.project_root)
            .output();
        let result = match output {
            Ok(output) => EcVerifyCommandResult {
                case_id: case.id.clone(),
                capability_id: case.capability_id.clone(),
                claim_id: case.claim_id.clone(),
                category: case.category.clone(),
                command: case.command.clone(),
                status: if output.status.success() {
                    "passed".to_string()
                } else {
                    "failed".to_string()
                },
                exit_code: output.status.code(),
                stdout_tail: tail_lossy(&output.stdout, 4000),
                stderr_tail: tail_lossy(&output.stderr, 4000),
            },
            Err(err) => EcVerifyCommandResult {
                case_id: case.id.clone(),
                capability_id: case.capability_id.clone(),
                claim_id: case.claim_id.clone(),
                category: case.category.clone(),
                command: case.command.clone(),
                status: "failed".to_string(),
                exit_code: None,
                stdout_tail: String::new(),
                stderr_tail: err.to_string(),
            },
        };
        results.push(result);
    }
    let command_count = results.len();
    let passed_count = results
        .iter()
        .filter(|result| result.status == "passed")
        .count();
    let failed_count = command_count.saturating_sub(passed_count);
    Ok(EcVerifySummary {
        project: ctx.project.clone(),
        manifest_path: relative_to(&ctx.project_root, &manifest_path),
        clean: failed_count == 0,
        command_count,
        passed_count,
        failed_count,
        results,
    })
}

fn compare_case_field(
    findings: &mut Vec<String>,
    case_id: &str,
    field: &str,
    expected: &str,
    actual: &str,
) {
    if expected != actual {
        findings.push(format!(
            "manifest case `{case_id}` field `{field}` drifted: expected `{expected}`, found `{actual}`"
        ));
    }
}

fn tail_lossy(bytes: &[u8], max_chars: usize) -> String {
    let text = String::from_utf8_lossy(bytes).trim().to_string();
    if text.chars().count() <= max_chars {
        return text;
    }
    text.chars()
        .rev()
        .take(max_chars)
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .collect()
}

fn generated_ec_test_paths(ctx: &EcProjectContext) -> Result<Vec<String>> {
    if !ctx.tests_root.is_dir() {
        return Ok(Vec::new());
    }
    let mut paths = Vec::new();
    for entry in WalkDir::new(&ctx.tests_root)
        .into_iter()
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.file_type().is_file())
    {
        let path = entry.path();
        if path == ctx.manifest_path {
            continue;
        }
        let content =
            fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
        if content.contains(EC_BEGIN_MARKER) {
            paths.push(relative_to(&ctx.project_root, path));
        }
    }
    paths.sort();
    Ok(paths)
}

fn write_ec_manifest(ctx: &EcProjectContext, manifest: &EcManifest) -> Result<()> {
    write_ec_manifest_to_legacy_file(ctx, manifest)?;
    remove_aw_ec_generated_block(ctx)
}

fn write_ec_manifest_to_legacy_file(ctx: &EcProjectContext, manifest: &EcManifest) -> Result<()> {
    if let Some(parent) = ctx.legacy_manifest_path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }
    let body = format!(
        "# SPEC-MANAGED: generated by `aw ec gen --project {}` from ec/ markdown or legacy TD EC sections.\n# CODEGEN-BEGIN\n{}# CODEGEN-END\n",
        ctx.project,
        toml::to_string_pretty(manifest)?
    );
    fs::write(&ctx.legacy_manifest_path, body)
        .with_context(|| format!("write {}", ctx.legacy_manifest_path.display()))
}

fn remove_aw_ec_generated_block(ctx: &EcProjectContext) -> Result<()> {
    if !ctx.project_aw_path.is_file() {
        return Ok(());
    }
    let existing = fs::read_to_string(&ctx.project_aw_path)
        .with_context(|| format!("read {}", ctx.project_aw_path.display()))?;
    let Some(next) = strip_aw_ec_generated_block(&existing) else {
        return Ok(());
    };
    fs::write(&ctx.project_aw_path, next)
        .with_context(|| format!("write {}", ctx.project_aw_path.display()))
}

fn strip_aw_ec_generated_block(existing: &str) -> Option<String> {
    let begin_line = format!("# {EC_AW_BEGIN_MARKER}");
    let end_line = format!("# {EC_AW_END_MARKER}");
    let lines: Vec<&str> = existing.lines().collect();
    let begin_idx = lines.iter().position(|line| line.trim() == begin_line)?;
    let end_idx = lines.iter().position(|line| line.trim() == end_line)?;
    if begin_idx > end_idx {
        return None;
    }
    let mut out = String::new();
    for line in &lines[..begin_idx] {
        out.push_str(line);
        out.push('\n');
    }
    for line in &lines[(end_idx + 1)..] {
        out.push_str(line);
        out.push('\n');
    }
    Some(out)
}

fn write_ec_doc(ctx: &EcProjectContext, content: &str) -> Result<()> {
    if ctx.doc_path.exists() {
        let existing = fs::read_to_string(&ctx.doc_path)
            .with_context(|| format!("read {}", ctx.doc_path.display()))?;
        if existing == content {
            return Ok(());
        }
        if !existing.contains(EC_DOC_BEGIN_MARKER) || !existing.contains(EC_DOC_END_MARKER) {
            bail!(
                "refusing to overwrite non-EC doc file {}; move it or add AW-EC-DOC markers",
                ctx.doc_path.display()
            );
        }
    }
    if let Some(parent) = ctx.doc_path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }
    fs::write(&ctx.doc_path, content).with_context(|| format!("write {}", ctx.doc_path.display()))
}

fn render_ec_doc(ctx: &EcProjectContext, manifest: &EcManifest) -> String {
    let mut out = String::new();
    out.push_str(&format!("# {} EC Manual\n\n", title_case(&ctx.project)));
    out.push_str(&format!(
        "<!-- {EC_DOC_BEGIN_MARKER} project={} manifest={} digest={} -->\n\n",
        ctx.project,
        relative_to(&ctx.project_root, &ctx.manifest_path),
        manifest.generated_from_td_digest
    ));
    out.push_str("This document is generated from AW external-contract definitions. Do not edit the generated block directly; update `ec/` or rerun `aw ec doc gen`.\n\n");
    out.push_str("## Verification Summary\n\n");
    out.push_str(&format!("- Project: `{}`\n", ctx.project));
    out.push_str(&format!(
        "- Manifest: `{}`\n",
        relative_to(&ctx.project_root, &ctx.manifest_path)
    ));
    out.push_str(&format!(
        "- Manifest digest: `{}`\n",
        manifest.generated_from_td_digest
    ));
    out.push_str(&format!("- EC case count: `{}`\n\n", manifest.cases.len()));

    if manifest.cases.is_empty() {
        out.push_str("No EC cases are currently declared.\n\n");
    } else {
        out.push_str("## Product Journeys\n\n");
        for case in &manifest.cases {
            out.push_str(&format!("### {}\n\n", title_case(&case.id)));
            out.push_str(&format!("- Capability: `{}`\n", case.capability_id));
            out.push_str(&format!("- Claim: `{}`\n", case.claim_id));
            out.push_str(&format!("- Contract: `{}`\n", case.contract_id));
            out.push_str(&format!("- Category: `{}`\n", case.category));
            out.push_str(&format!("- Source: `{}`\n", case.td_ref));
            out.push_str(&format!(
                "- Required for production: `{}`\n",
                case.required_for_production
            ));
            out.push_str(&format!("- Test path: `{}`\n", case.test_path));
            out.push_str(&format!("- Verification command: `{}`\n", case.command));
            if !case.assertions.is_empty() {
                out.push_str("\nExpected evidence:\n\n");
                for assertion in &case.assertions {
                    out.push_str(&format!("- {}\n", assertion));
                }
            }
            if !case.evidence.is_empty() {
                out.push_str("\nEvidence artifacts:\n\n");
                for artifact in &case.evidence {
                    out.push_str(&format!(
                        "- `{}`: `{}`{}\n",
                        artifact.kind,
                        artifact.path,
                        render_artifact_suffix(artifact)
                    ));
                }
            }
            if !case.evaluators.is_empty() {
                out.push_str("\nAgent evaluators:\n\n");
                for evaluator in &case.evaluators {
                    out.push_str(&format!(
                        "- `{}` via `{}`{}\n",
                        evaluator.id,
                        evaluator.tool,
                        render_evaluator_suffix(evaluator)
                    ));
                    if !evaluator.rubric.is_empty() {
                        out.push_str("  - Rubric:\n");
                        for item in &evaluator.rubric {
                            out.push_str(&format!("    - {}\n", item));
                        }
                    }
                    if !evaluator.pass_criteria.is_empty() {
                        out.push_str("  - Pass criteria:\n");
                        for item in &evaluator.pass_criteria {
                            out.push_str(&format!("    - {}\n", item));
                        }
                    }
                }
            }
            out.push('\n');
        }
    }
    out.push_str(&format!("<!-- {EC_DOC_END_MARKER} -->\n"));
    out
}

fn render_artifact_suffix(artifact: &EcEvidenceArtifact) -> String {
    let mut parts = Vec::new();
    if !artifact.label.is_empty() {
        parts.push(format!("label: {}", artifact.label));
    }
    if !artifact.locator.is_empty() {
        parts.push(format!("locator: `{}`", artifact.locator));
    }
    if parts.is_empty() {
        String::new()
    } else {
        format!(" ({})", parts.join(", "))
    }
}

fn render_evaluator_suffix(evaluator: &EcEvaluator) -> String {
    let mut parts = Vec::new();
    if !evaluator.command.is_empty() {
        parts.push(format!("command: `{}`", evaluator.command));
    }
    if !evaluator.report_path.is_empty() {
        parts.push(format!("report: `{}`", evaluator.report_path));
    }
    if !evaluator.prompt.is_empty() {
        parts.push(format!("prompt: {}", evaluator.prompt));
    }
    if parts.is_empty() {
        String::new()
    } else {
        format!(" ({})", parts.join(", "))
    }
}

fn generated_ec_test_files(
    ctx: &EcProjectContext,
    manifest: &EcManifest,
) -> Vec<(PathBuf, String)> {
    manifest
        .cases
        .iter()
        .map(|case| {
            let path = ctx.project_root.join(&case.test_path);
            let content = render_ec_test(ctx, case);
            (path, content)
        })
        .collect()
}

fn render_ec_test(ctx: &EcProjectContext, case: &EcManifestCase) -> String {
    match ctx.target.as_str() {
        "rust" => render_rust_ec_test(case),
        "python" => render_python_ec_test(case),
        "typescript" | "javascript" | "ts" | "js" => render_ts_ec_test(case),
        _ => render_text_ec_test(case),
    }
}

fn write_generated_ec_test(path: &Path, content: &str) -> Result<()> {
    if path.exists() {
        let existing =
            fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
        if existing == content {
            return Ok(());
        }
        if !existing.contains(EC_BEGIN_MARKER) {
            bail!(
                "refusing to overwrite non-EC test file {}; move it or add AW-EC markers",
                path.display()
            );
        }
    }
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }
    fs::write(path, content).with_context(|| format!("write {}", path.display()))
}

fn write_generated_tool_manifests(ctx: &EcProjectContext, manifest: &EcManifest) -> Result<()> {
    for item in &manifest.tool_manifests {
        let path = ctx.project_root.join(&item.path);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
        }
        if path.is_file() {
            let existing =
                fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
            if existing != item.generated_toml && !existing.contains(EC_TOOL_BEGIN_MARKER) {
                bail!(
                    "refusing to overwrite non-generated tool manifest {}; move it or add AW-EC-TOOL markers",
                    path.display()
                );
            }
        }
        fs::write(&path, &item.generated_toml)
            .with_context(|| format!("write {}", path.display()))?;
    }
    Ok(())
}

// Real EC gate test: runs the contract `command` from the project root (the dir
// containing `.aw/`, where EC commands are defined to run) and asserts success.
// `#[ignore]` keeps it out of the default `cargo test` (EC commands are heavy and
// may themselves invoke cargo test); run via `cargo test -- --ignored` or
// `aw health --verify-ec`. `__FN__`/`__CMD__`/`__ID__` are substituted, not
// `format!`-interpolated, so the template's own `{...}` stay literal.
const EC_RUST_COMMAND_TEMPLATE: &str = r#"#[test]
#[ignore = "AW EC gate: run via `aw health --verify-ec` or `cargo test -- --ignored`"]
fn __FN__() {
    let command = __CMD__;
    let id = __ID__;
    let mut root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    while !root.join(".aw").is_dir() {
        assert!(
            root.pop(),
            "AW EC {id}: no .aw/ project root above {}",
            env!("CARGO_MANIFEST_DIR")
        );
    }
    let status = std::process::Command::new("sh")
        .arg("-c")
        .arg(command)
        .current_dir(&root)
        .status()
        .unwrap_or_else(|e| panic!("AW EC {id}: failed to spawn `{command}`: {e}"));
    assert!(
        status.success(),
        "AW EC {id} FAILED (exit {:?}): {command}",
        status.code()
    );
}
"#;

const EC_RUST_NO_COMMAND_TEMPLATE: &str = r#"#[test]
#[ignore = "AW EC placeholder: no command bound to this contract"]
fn __FN__() {
    panic!("AW EC {}: no command bound to this contract", __ID__);
}
"#;

fn render_rust_ec_test(case: &EcManifestCase) -> String {
    let fn_name = rust_ident(&case.id);
    let evaluator_markers = render_evaluator_marker_lines("//", case);
    let header = format!(
        "// SPEC-MANAGED: {}\n// CODEGEN-BEGIN\n// {EC_BEGIN_MARKER}\n// @ec {}\n// @capability {}\n// @claim {}\n// @contract {}\n// @category {}\n// @required_for_production {}\n// @command {}\n{}// {EC_END_MARKER}\n\n",
        case.td_ref,
        case.id,
        case.capability_id,
        case.claim_id,
        case.contract_id,
        case.category,
        case.required_for_production,
        case.command,
        evaluator_markers,
    );
    // Preserve the contract's English assertions as leading doc comments.
    let mut contract_doc = String::new();
    for assertion in &case.assertions {
        contract_doc.push_str("// Contract: ");
        contract_doc.push_str(assertion);
        contract_doc.push('\n');
    }
    let body = if case.command.trim().is_empty() {
        EC_RUST_NO_COMMAND_TEMPLATE
            .replace("__FN__", &fn_name)
            .replace("__ID__", &rust_string_literal(&case.id))
    } else {
        EC_RUST_COMMAND_TEMPLATE
            .replace("__FN__", &fn_name)
            .replace("__CMD__", &rust_string_literal(&case.command))
            .replace("__ID__", &rust_string_literal(&case.id))
    };
    format!("{header}{contract_doc}{body}// CODEGEN-END\n")
}

fn render_python_ec_test(case: &EcManifestCase) -> String {
    let evaluator_markers = render_evaluator_marker_lines("#", case);
    format!(
        "# SPEC-MANAGED: {}\n# CODEGEN-BEGIN\n# {EC_BEGIN_MARKER}\n# @ec {}\n# @capability {}\n# @claim {}\n# @contract {}\n# @category {}\n# @required_for_production {}\n# @command {}\n{}# {EC_END_MARKER}\n\nimport pytest\n\n\n@pytest.mark.skip(reason=\"AW EC placeholder: implement this external contract test or keep the manifest command authoritative\")\ndef test_{}():\n    raise AssertionError(\"AW EC placeholder for {}\")\n# CODEGEN-END\n",
        case.td_ref,
        case.id,
        case.capability_id,
        case.claim_id,
        case.contract_id,
        case.category,
        case.required_for_production,
        case.command,
        evaluator_markers,
        rust_ident(&case.id),
        escape_py_string(&case.id)
    )
}

fn render_ts_ec_test(case: &EcManifestCase) -> String {
    let evaluator_markers = render_evaluator_marker_lines("//", case);
    format!(
        "// SPEC-MANAGED: {}\n// CODEGEN-BEGIN\n// {EC_BEGIN_MARKER}\n// @ec {}\n// @capability {}\n// @claim {}\n// @contract {}\n// @category {}\n// @required_for_production {}\n// @command {}\n{}// {EC_END_MARKER}\n\nimport {{ test }} from \"vitest\";\n\ntest.skip({}, () => {{\n  throw new Error({});\n}});\n// CODEGEN-END\n",
        case.td_ref,
        case.id,
        case.capability_id,
        case.claim_id,
        case.contract_id,
        case.category,
        case.required_for_production,
        case.command,
        evaluator_markers,
        serde_json::to_string(&case.id).unwrap_or_else(|_| "\"aw-ec\"".to_string()),
        serde_json::to_string(&format!("AW EC placeholder for {}", case.id))
            .unwrap_or_else(|_| "\"AW EC placeholder\"".to_string())
    )
}

fn render_text_ec_test(case: &EcManifestCase) -> String {
    let evaluator_markers = render_evaluator_marker_lines("", case);
    format!(
        "SPEC-MANAGED: {}\nCODEGEN-BEGIN\n{EC_BEGIN_MARKER}\n@ec {}\n@capability {}\n@claim {}\n@contract {}\n@category {}\n@required_for_production {}\n@command {}\n{}{EC_END_MARKER}\nCODEGEN-END\n",
        case.td_ref,
        case.id,
        case.capability_id,
        case.claim_id,
        case.contract_id,
        case.category,
        case.required_for_production,
        case.command,
        evaluator_markers
    )
}

fn render_evaluator_marker_lines(prefix: &str, case: &EcManifestCase) -> String {
    let mut out = String::new();
    for evaluator in &case.evaluators {
        if prefix.is_empty() {
            out.push_str(&format!(
                "@evaluator {} tool={} command={} report={}\n",
                evaluator.id, evaluator.tool, evaluator.command, evaluator.report_path
            ));
        } else {
            out.push_str(&format!(
                "{prefix} @evaluator {} tool={} command={} report={}\n",
                evaluator.id, evaluator.tool, evaluator.command, evaluator.report_path
            ));
        }
    }
    out
}

fn relative_to(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .to_string_lossy()
        .replace('\\', "/")
}

fn slugify(value: &str) -> String {
    let mut out = String::new();
    let mut last_dash = false;
    for ch in value.chars() {
        if ch.is_ascii_alphanumeric() {
            out.push(ch.to_ascii_lowercase());
            last_dash = false;
        } else if !last_dash {
            out.push('-');
            last_dash = true;
        }
    }
    let out = out.trim_matches('-').to_string();
    if out.is_empty() {
        "ec".to_string()
    } else {
        out
    }
}

fn title_case(value: &str) -> String {
    value
        .split(|ch: char| ch == '-' || ch == '_' || ch.is_whitespace())
        .filter(|part| !part.is_empty())
        .map(|part| {
            let mut chars = part.chars();
            match chars.next() {
                Some(first) => {
                    let mut word = String::new();
                    word.push(first.to_ascii_uppercase());
                    word.push_str(chars.as_str());
                    word
                }
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn rust_ident(value: &str) -> String {
    let mut out = String::new();
    for ch in value.chars() {
        if ch.is_ascii_alphanumeric() {
            out.push(ch.to_ascii_lowercase());
        } else {
            out.push('_');
        }
    }
    if out.is_empty() || out.chars().next().is_some_and(|ch| ch.is_ascii_digit()) {
        out.insert_str(0, "ec_");
    }
    out
}

fn rust_string_literal(value: &str) -> String {
    format!("{value:?}")
}

fn escape_py_string(value: &str) -> String {
    value.replace('\\', "\\\\").replace('"', "\\\"")
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn write_demo_repo() -> (TempDir, EcProjectContext) {
        let tmp = tempfile::tempdir().unwrap();
        fs::create_dir_all(tmp.path().join(".aw/tech-design/projects/demo/specs")).unwrap();
        fs::create_dir_all(tmp.path().join("projects/demo")).unwrap();
        fs::write(
            tmp.path().join(".aw/config.toml"),
            r#"
[[projects]]
name = "demo"
aliases = ["d"]
path = "projects/demo"
td_path = ".aw/tech-design/projects/demo"

[[projects.workspaces]]
name = "demo"
paths = ["projects/demo/**"]
target = "rust"
test_cmd = "cargo test -p demo"
"#,
        )
        .unwrap();
        fs::write(
            tmp.path().join("projects/demo/Cargo.toml"),
            r#"[package]
name = "demo-crate"
version = "0.1.0"
edition = "2021"
"#,
        )
        .unwrap();
        fs::write(
            tmp.path()
                .join(".aw/tech-design/projects/demo/specs/contract.md"),
            r#"
## Contract E2E Test
<!-- type: e2e-test lang: yaml -->

```yaml
e2e_tests:
  - name: Demo happy path
    command: cargo test -p demo-crate demo_happy_path -- --nocapture
    asserts:
      - command exits zero
      - output is stable
    evidence:
      screenshots:
        - path: e2e-results/demo/happy-path.png
          label: Demo happy path
          locator: "[data-testid=demo-happy-path]"
      reports:
        - path: e2e-results/demo/report.json
          kind: agent-eval
          label: Agent eval report
    evaluators:
      - id: Demo agent judge
        tool: codex
        command: codex exec --json demo-eval
        report_path: e2e-results/demo/report.json
        rubric:
          - answer is grounded in project state
          - no blocking contradiction
        pass_criteria:
          - score >= 4
          - blocking_violations is empty
```
"#,
        )
        .unwrap();
        let ctx = resolve_ec_project_context(tmp.path(), "d").unwrap();
        (tmp, ctx)
    }

    fn ctx_with_root(project_root: &Path) -> EcProjectContext {
        EcProjectContext {
            project_root: project_root.to_path_buf(),
            project: "demo".to_string(),
            source_root: project_root.join("projects/demo"),
            ec_root: project_root.join("projects/demo/external-contracts"),
            td_root: project_root.join(".aw/tech-design/projects/demo"),
            tests_root: project_root.join("projects/demo/tests"),
            manifest_path: project_root.join("projects/demo/tests/aw-ec.toml"),
            legacy_manifest_path: project_root.join("projects/demo/tests/aw-ec.toml"),
            project_aw_path: project_root.join("projects/demo/aw.toml"),
            doc_path: project_root.join("projects/demo/docs/aw-ec-manual.md"),
            target: "rust".to_string(),
            package_name: "demo-crate".to_string(),
        }
    }

    fn case(id: &str, capability_id: &str, category: &str) -> EcManifestCase {
        EcManifestCase {
            id: id.to_string(),
            capability_id: capability_id.to_string(),
            claim_id: id.to_string(),
            contract_id: id.to_string(),
            category: category.to_string(),
            td_ref: format!("td#{id}"),
            test_path: format!("tests/{id}.rs"),
            command: "cargo test".to_string(),
            // Start from the YAML default (true) so we can observe derivation flip it.
            required_for_production: true,
            assertions: Vec::new(),
            evidence: Vec::new(),
            evaluators: Vec::new(),
        }
    }

    #[test]
    fn case_gen_mode_dispatches_on_command() {
        let mut c = case("x", "search", "stability");
        c.command = "rig test --dir cases/resilience".into();
        assert_eq!(case_gen_mode(&c), CaseGenMode::Rig);
        c.command = "cargo test -p lumen --test api_e2e".into();
        assert_eq!(case_gen_mode(&c), CaseGenMode::NativeRust);
        c.command = "pytest x".into();
        assert_eq!(case_gen_mode(&c), CaseGenMode::Other);
    }

    #[test]
    fn rig_skeleton_has_case_and_source_contract() {
        let mut c = case("search-stability-fault-resilience", "search", "stability");
        c.assertions = vec!["search p99 stays bounded".into()];
        let s = render_case_toml_skeleton(
            &c,
            "lumen",
            "resilience",
            "search_stability_fault_resilience",
        );
        assert!(s.contains("[case]"));
        assert!(s.contains("source_contract = \"search-stability-fault-resilience\""));
        assert!(s.contains("[exercise.request]"));
        assert!(s.contains("dimension = \"resilience\""));
        assert!(s.contains("subject = \"search p99 stays bounded\""));
    }

    #[test]
    fn native_skeleton_has_test_fn_and_ec_marker() {
        let c = case("lumen-x", "search", "behavior");
        let s = render_native_rust_skeleton(&c, "api_e2e");
        assert!(s.contains("#[test]"));
        assert!(s.contains("fn api_e2e()"));
        assert!(s.contains("@ec lumen-x"));
    }

    #[test]
    fn command_parsers_extract_dir_and_target() {
        assert_eq!(
            rig_dir_from_command("rig test --dir cases/load"),
            Some("cases/load")
        );
        assert_eq!(
            cargo_test_target("cargo test -p lumen --test api_e2e -- --ignored"),
            Some("api_e2e")
        );
    }

    #[test]
    fn derive_required_for_production_uses_capability_type() {
        let tmp = tempfile::tempdir().unwrap();
        fs::create_dir_all(tmp.path().join(".aw")).unwrap();
        fs::write(
            tmp.path().join(".aw/capability-types.toml"),
            "[capability_types]\nsvc-cap = \"Service\"\nagent-cap = \"AgentFirst\"\n",
        )
        .unwrap();
        let ctx = ctx_with_root(tmp.path());

        let mut cases = vec![
            // Service capability: security + stability are production-required.
            case("svc-sec", "svc-cap", "security"),
            case("svc-stab", "svc-cap", "stability"),
            // AgentFirst capability: only behavior is required; efficiency is not.
            case("agent-eff", "agent-cap", "efficiency"),
            case("agent-beh", "agent-cap", "behavior"),
            // Untyped capability: falls back to the YAML flag (default true).
            case("other", "no-type-cap", "security"),
        ];
        derive_required_for_production(&ctx, &mut cases).unwrap();

        let by_id = |id: &str| {
            cases
                .iter()
                .find(|c| c.id == id)
                .unwrap()
                .required_for_production
        };
        assert!(by_id("svc-sec"), "Service/security is production-required");
        assert!(
            by_id("svc-stab"),
            "Service/stability is production-required"
        );
        assert!(
            !by_id("agent-eff"),
            "AgentFirst/efficiency is NOT production-required"
        );
        assert!(
            by_id("agent-beh"),
            "AgentFirst/behavior is production-required"
        );
        assert!(
            by_id("other"),
            "untyped capability keeps the YAML fallback (true)"
        );
    }

    #[test]
    fn derive_required_for_production_no_binding_keeps_yaml_flag() {
        // No .aw/capability-types.toml at all -> values untouched.
        let tmp = tempfile::tempdir().unwrap();
        let ctx = ctx_with_root(tmp.path());
        let mut cases = vec![{
            let mut c = case("c1", "cap", "security");
            c.required_for_production = false; // simulate a YAML-set false
            c
        }];
        derive_required_for_production(&ctx, &mut cases).unwrap();
        assert!(
            !cases[0].required_for_production,
            "with no binding the YAML flag (false) is preserved"
        );
    }

    #[test]
    fn ec_context_defaults_td_root_to_project_tech_design() {
        let tmp = tempfile::tempdir().unwrap();
        fs::create_dir_all(tmp.path().join(".aw")).unwrap();
        fs::create_dir_all(tmp.path().join("projects/demo/tech-design/specs")).unwrap();
        fs::create_dir_all(tmp.path().join("projects/demo")).unwrap();
        fs::write(
            tmp.path().join(".aw/config.toml"),
            r#"
[[projects]]
name = "demo"
aliases = ["d"]
path = "projects/demo"

[[projects.workspaces]]
name = "demo"
paths = ["projects/demo/**"]
target = "rust"
"#,
        )
        .unwrap();
        fs::write(
            tmp.path().join("projects/demo/Cargo.toml"),
            r#"[package]
name = "demo-crate"
version = "0.1.0"
edition = "2021"
"#,
        )
        .unwrap();

        let ctx = resolve_ec_project_context(tmp.path(), "d").unwrap();

        assert_eq!(ctx.project, "demo");
        assert_eq!(ctx.td_root, tmp.path().join("projects/demo/tech-design"));
        assert_eq!(
            ctx.manifest_path,
            tmp.path().join("projects/demo/tests/aw-ec.toml")
        );
    }

    #[test]
    fn extracts_td_e2e_cases_into_manifest() {
        let (_tmp, ctx) = write_demo_repo();
        let manifest = build_expected_manifest(&ctx).unwrap();

        assert_eq!(manifest.project, "demo");
        assert_eq!(manifest.cases.len(), 1);
        let case = &manifest.cases[0];
        assert_eq!(case.id, "demo-happy-path");
        assert_eq!(case.capability_id, "unmapped");
        assert_eq!(case.contract_id, "demo-happy-path");
        assert_eq!(case.category, "behavior");
        assert_eq!(
            case.command,
            "cargo test -p demo-crate demo_happy_path -- --nocapture"
        );
        assert_eq!(case.assertions.len(), 2);
        assert_eq!(case.evidence.len(), 2);
        assert_eq!(case.evidence[0].kind, "screenshot");
        assert_eq!(case.evidence[0].path, "e2e-results/demo/happy-path.png");
        assert_eq!(case.evidence[0].label, "Demo happy path");
        assert_eq!(case.evidence[0].locator, "[data-testid=demo-happy-path]");
        assert_eq!(case.evidence[1].kind, "agent-eval");
        assert_eq!(case.evaluators.len(), 1);
        assert_eq!(case.evaluators[0].id, "demo-agent-judge");
        assert_eq!(case.evaluators[0].tool, "codex");
        assert_eq!(case.evaluators[0].command, "codex exec --json demo-eval");
        assert_eq!(
            case.evaluators[0].report_path,
            "e2e-results/demo/report.json"
        );
        assert_eq!(case.evaluators[0].rubric.len(), 2);
        assert_eq!(case.evaluators[0].pass_criteria.len(), 2);
        assert!(case.td_ref.ends_with("contract.md#demo-happy-path"));
        assert_eq!(
            case.test_path,
            "projects/demo/tests/behavior_demo_happy_path.rs"
        );
        assert!(manifest.generated_from_td_digest.starts_with("sha256:"));
    }

    #[test]
    fn ec_markdown_takes_priority_and_generates_tool_manifest() {
        let (tmp, ctx) = write_demo_repo();
        fs::create_dir_all(tmp.path().join("projects/demo/external-contracts/behavior")).unwrap();
        fs::write(
            tmp.path()
                .join("projects/demo/external-contracts/behavior/static.md"),
            r#"
## Static HTTP
<!-- type: e2e-test lang: yaml -->

```yaml
e2e_tests:
  - id: static http
    capability_id: serve
    claim_id: serve-static-http
    category: behavior
    command: "rig run --dir projects/demo/tests/rig/scenarios/behavior"
    assertions:
      - "GET /index.html returns 200"
```

## Static Tool
<!-- type: tool-contract lang: yaml -->

```yaml
tool_contracts:
  - id: static http rig
    tool: rig
    manifest: rig.toml
    category: behavior
    command: "rig run --dir projects/demo/tests/rig/scenarios/behavior"
    native:
      version: 1
      scenario: static-http
```
"#,
        )
        .unwrap();

        let manifest = build_expected_manifest(&ctx).unwrap();

        assert_eq!(manifest.cases.len(), 1);
        assert_eq!(manifest.cases[0].id, "static-http");
        assert_eq!(manifest.cases[0].capability_id, "serve");
        assert_eq!(manifest.cases[0].claim_id, "serve-static-http");
        assert_eq!(manifest.cases[0].category, "behavior");
        assert!(manifest.cases[0]
            .td_ref
            .ends_with("external-contracts/behavior/static.md#static-http"));
        assert_eq!(manifest.tool_manifests.len(), 1);
        assert_eq!(manifest.tool_manifests[0].tool, "rig");
        assert_eq!(manifest.tool_manifests[0].path, "projects/demo/rig.toml");
        assert_eq!(
            manifest.cases[0].command,
            "rig run --dir projects/demo/tests/rig/scenarios/behavior"
        );
    }

    #[test]
    fn ec_draft_fill_markdown_drives_manifest() {
        let (tmp, ctx) = write_demo_repo();
        let args = EcDraftArgs {
            id: "search-indexing".to_string(),
            category: "efficiency".to_string(),
            title: Some("Search Indexing".to_string()),
            capability_id: Some("search-indexing".to_string()),
            claim_id: Some("indexing-speed".to_string()),
            contract_id: None,
            command: None,
            tool: vec!["arena".to_string()],
            force: false,
            json: false,
        };
        let draft = render_ec_draft(
            &ctx,
            "search-indexing",
            "efficiency",
            "Search Indexing",
            &args,
        );
        let filled = merge_ec_section(
            &draft,
            "e2e-test",
            r#"## Indexing Speed
<!-- type: e2e-test lang: yaml -->

```yaml
e2e_tests:
  - id: indexing speed
    capability_id: search-indexing
    claim_id: indexing-speed
    category: efficiency
    test_path: projects/demo/tests/benchmark_indexing_speed.rs
    command: "cargo test -p demo-crate indexing_speed -- --nocapture"
    assertions:
      - "indexes under target latency"
```
"#,
        )
        .unwrap();
        let filled = merge_ec_section(
            &filled,
            "tool-contract",
            r#"## Arena Contract
<!-- type: tool-contract lang: yaml -->

```yaml
tool_contracts:
  - id: indexing speed arena
    tool: arena
    manifest: arena.toml
    category: efficiency
    command: "arena run --spec projects/demo/arena.toml"
    native:
      version: 1
      benchmark: indexing-speed
```
"#,
        )
        .unwrap();
        let path = tmp
            .path()
            .join("projects/demo/external-contracts/efficiency/search-indexing.md");
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(&path, filled).unwrap();

        let manifest = build_expected_manifest(&ctx).unwrap();

        assert_eq!(manifest.cases.len(), 1);
        let case = &manifest.cases[0];
        assert_eq!(case.id, "indexing-speed");
        assert_eq!(case.capability_id, "search-indexing");
        assert_eq!(case.claim_id, "indexing-speed");
        assert_eq!(case.category, "efficiency");
        assert_eq!(
            case.test_path,
            "projects/demo/tests/benchmark_indexing_speed.rs"
        );
        assert!(case
            .td_ref
            .ends_with("external-contracts/efficiency/search-indexing.md#indexing-speed"));
        assert_eq!(manifest.tool_manifests.len(), 1);
        assert_eq!(manifest.tool_manifests[0].tool, "arena");
        assert_eq!(manifest.tool_manifests[0].path, "projects/demo/arena.toml");
    }

    #[test]
    fn external_contracts_reject_unknown_category() {
        let (tmp, ctx) = write_demo_repo();
        fs::create_dir_all(tmp.path().join("projects/demo/external-contracts/behavior")).unwrap();
        fs::write(
            tmp.path()
                .join("projects/demo/external-contracts/behavior/static.md"),
            r#"
## Bad Contract
<!-- type: e2e-test lang: yaml -->

```yaml
e2e_tests:
  - id: bad
    category: usability
```
"#,
        )
        .unwrap();

        let err = build_expected_manifest(&ctx).unwrap_err().to_string();
        assert!(err.contains("expected behavior|efficiency|security|stability"));
    }

    #[test]
    fn ec_verify_runs_manifest_commands() {
        let (tmp, ctx) = write_demo_repo();
        fs::create_dir_all(tmp.path().join("projects/demo/external-contracts/behavior")).unwrap();
        fs::write(
            tmp.path()
                .join("projects/demo/external-contracts/behavior/smoke.md"),
            r#"
## Smoke
<!-- type: e2e-test lang: yaml -->

```yaml
e2e_tests:
  - id: smoke
    capability_id: demo
    claim_id: demo-smoke
    command: "true"
```
"#,
        )
        .unwrap();
        let manifest = build_expected_manifest(&ctx).unwrap();
        write_ec_manifest(&ctx, &manifest).unwrap();
        for (path, content) in generated_ec_test_files(&ctx, &manifest) {
            write_generated_ec_test(&path, &content).unwrap();
        }

        let summary = verify_ec_context(&ctx).unwrap();

        assert!(summary.clean, "{:?}", summary.results);
        assert_eq!(summary.command_count, 1);
        assert_eq!(summary.passed_count, 1);
        assert_eq!(summary.results[0].claim_id, "demo-smoke");
    }

    #[test]
    fn check_reports_missing_manifest_when_td_has_cases() {
        let (_tmp, ctx) = write_demo_repo();
        let summary = check_ec_context(&ctx).unwrap();

        assert!(!summary.clean);
        assert!(!summary.configured);
        assert_eq!(summary.expected_case_count, 1);
        assert!(summary
            .findings
            .iter()
            .any(|finding| finding.contains("EC manifest missing")));
    }

    #[test]
    fn generated_manifest_and_test_file_check_clean() {
        let (_tmp, ctx) = write_demo_repo();
        let manifest = build_expected_manifest(&ctx).unwrap();
        write_ec_manifest(&ctx, &manifest).unwrap();
        for (path, content) in generated_ec_test_files(&ctx, &manifest) {
            write_generated_ec_test(&path, &content).unwrap();
        }

        let summary = check_ec_context(&ctx).unwrap();
        assert!(summary.clean, "{:?}", summary.findings);
        assert!(summary.configured);
        assert_eq!(summary.case_count, 1);
        assert!(!summary.stale);
    }

    #[test]
    fn generated_manifest_lives_outside_project_aw_toml() {
        let (tmp, _ctx) = write_demo_repo();
        fs::write(
            tmp.path().join("projects/demo/aw.toml"),
            r#"
[project]
name = "demo"
aliases = ["d"]
td_path = ".aw/tech-design/projects/demo"

[[workspaces]]
name = "demo"
paths = ["**"]
target = "rust"
test_cmd = "cargo test -p demo"

# AW-EC-BEGIN
[aw.ec.generated]
version = 1
project = "demo"
generated_from_td_digest = "stale"
# AW-EC-END
"#,
        )
        .unwrap();
        let ctx = resolve_ec_project_context(tmp.path(), "d").unwrap();
        let manifest = build_expected_manifest(&ctx).unwrap();
        write_ec_manifest(&ctx, &manifest).unwrap();
        for (path, content) in generated_ec_test_files(&ctx, &manifest) {
            write_generated_ec_test(&path, &content).unwrap();
        }

        let aw_toml = fs::read_to_string(&ctx.project_aw_path).unwrap();
        assert!(!aw_toml.contains(EC_AW_BEGIN_MARKER));
        assert!(ctx.legacy_manifest_path.exists());

        let summary = check_ec_context(&ctx).unwrap();
        assert!(summary.clean, "{:?}", summary.findings);
        assert_eq!(summary.manifest_path, "projects/demo/tests/aw-ec.toml");
    }

    #[test]
    fn tool_contract_generates_native_toml_manifest() {
        let (tmp, ctx) = write_demo_repo();
        fs::write(
            tmp.path()
                .join(".aw/tech-design/projects/demo/specs/tool-contract.md"),
            r#"
## Guard Contract
<!-- type: tool-contract lang: yaml -->

```yaml
tool_contracts:
  - id: demo guard
    tool: guard
    manifest: guard.toml
    category: security
    command: guard scan projects/demo --compact --no-persist
    native:
      version: 1
      profile: baseline
      targets:
        - projects/demo
      ignore:
        generated: true
```
"#,
        )
        .unwrap();

        let manifest = build_expected_manifest(&ctx).unwrap();
        assert_eq!(manifest.tool_manifests.len(), 1);
        assert_eq!(manifest.tool_manifests[0].path, "projects/demo/guard.toml");
        assert!(manifest.tool_manifests[0]
            .generated_toml
            .contains("profile = \"baseline\""));
        assert!(manifest.tool_manifests[0]
            .generated_toml
            .contains("[ignore]"));

        write_ec_manifest(&ctx, &manifest).unwrap();
        write_generated_tool_manifests(&ctx, &manifest).unwrap();
        for (path, content) in generated_ec_test_files(&ctx, &manifest) {
            write_generated_ec_test(&path, &content).unwrap();
        }

        let summary = check_ec_context(&ctx).unwrap();
        assert!(summary.clean, "{:?}", summary.findings);
        assert_eq!(summary.tool_manifest_count, 1);
        let guard_toml = fs::read_to_string(tmp.path().join("projects/demo/guard.toml")).unwrap();
        assert!(guard_toml.contains("version = 1"));
    }

    #[test]
    fn check_detects_generated_ec_test_content_drift() {
        let (_tmp, ctx) = write_demo_repo();
        let manifest = build_expected_manifest(&ctx).unwrap();
        write_ec_manifest(&ctx, &manifest).unwrap();
        for (path, content) in generated_ec_test_files(&ctx, &manifest) {
            write_generated_ec_test(&path, &content).unwrap();
            fs::write(
                path,
                content.replace("@category behavior", "@category changed"),
            )
            .unwrap();
        }

        let summary = check_ec_context(&ctx).unwrap();
        assert!(!summary.clean);
        assert!(summary
            .findings
            .iter()
            .any(|finding| finding.contains("generated content drifted")));
    }

    #[test]
    fn check_detects_orphan_generated_ec_test() {
        let (_tmp, ctx) = write_demo_repo();
        let manifest = build_expected_manifest(&ctx).unwrap();
        write_ec_manifest(&ctx, &manifest).unwrap();
        for (path, content) in generated_ec_test_files(&ctx, &manifest) {
            write_generated_ec_test(&path, &content).unwrap();
        }
        let orphan = ctx.tests_root.join("behavior_orphan.rs");
        fs::write(
            &orphan,
            format!("// {EC_BEGIN_MARKER}\n// @ec orphan\n// {EC_END_MARKER}\n"),
        )
        .unwrap();

        let summary = check_ec_context(&ctx).unwrap();
        assert!(!summary.clean);
        assert_eq!(
            summary.orphan_test_paths,
            vec!["projects/demo/tests/behavior_orphan.rs"]
        );
    }

    #[test]
    fn ec_doc_gen_writes_manual_from_manifest() {
        let (_tmp, ctx) = write_demo_repo();
        let manifest = build_expected_manifest(&ctx).unwrap();
        write_ec_manifest(&ctx, &manifest).unwrap();
        for (path, content) in generated_ec_test_files(&ctx, &manifest) {
            write_generated_ec_test(&path, &content).unwrap();
        }

        let content = render_ec_doc(&ctx, &manifest);
        write_ec_doc(&ctx, &content).unwrap();

        let written = fs::read_to_string(&ctx.doc_path).unwrap();
        assert!(written.contains(EC_DOC_BEGIN_MARKER));
        assert!(written.contains("## Product Journeys"));
        assert!(written.contains("### Demo Happy Path"));
        assert!(written.contains("e2e-results/demo/happy-path.png"));
        assert!(written.contains("Agent eval report"));
        assert!(written.contains("Agent evaluators"));
        assert!(written.contains("demo-agent-judge"));
        assert!(written.contains("score >= 4"));
    }

    #[test]
    fn ec_doc_check_clean_after_doc_gen() {
        let (_tmp, ctx) = write_demo_repo();
        let manifest = build_expected_manifest(&ctx).unwrap();
        write_ec_manifest(&ctx, &manifest).unwrap();
        for (path, content) in generated_ec_test_files(&ctx, &manifest) {
            write_generated_ec_test(&path, &content).unwrap();
        }
        let content = render_ec_doc(&ctx, &manifest);
        write_ec_doc(&ctx, &content).unwrap();

        let summary = check_ec_doc_context(&ctx).unwrap();
        assert!(summary.clean, "{:?}", summary.findings);
        assert!(summary.configured);
        assert_eq!(summary.case_count, 1);
        assert_eq!(summary.doc_path, "projects/demo/docs/aw-ec-manual.md");
    }

    #[test]
    fn ec_doc_check_detects_doc_drift() {
        let (_tmp, ctx) = write_demo_repo();
        let manifest = build_expected_manifest(&ctx).unwrap();
        write_ec_manifest(&ctx, &manifest).unwrap();
        for (path, content) in generated_ec_test_files(&ctx, &manifest) {
            write_generated_ec_test(&path, &content).unwrap();
        }
        let content = render_ec_doc(&ctx, &manifest);
        write_ec_doc(&ctx, &content).unwrap();
        fs::write(&ctx.doc_path, content.replace("Demo Happy Path", "Changed")).unwrap();

        let summary = check_ec_doc_context(&ctx).unwrap();
        assert!(!summary.clean);
        assert!(summary
            .findings
            .iter()
            .any(|finding| finding.contains("generated content drifted")));
    }
}
// CODEGEN-END
