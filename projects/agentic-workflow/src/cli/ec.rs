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
use walkdir::WalkDir;

const EC_MANIFEST_VERSION: u8 = 1;
const EC_MANIFEST_REL: &str = "tests/aw-ec.toml";
const EC_BEGIN_MARKER: &str = "AW-EC-BEGIN";
const EC_END_MARKER: &str = "AW-EC-END";

#[derive(Debug, Args)]
/// External contract lifecycle: generate and check TD-derived E2E contract inventory.
/// @spec projects/agentic-workflow/tech-design/semantic/agentic-workflow-cli.md#schema
pub struct EcArgs {
    #[command(subcommand)]
    pub command: EcCommand,
}

/// @spec projects/agentic-workflow/tech-design/semantic/agentic-workflow-cli.md#schema
#[derive(Debug, Subcommand)]
pub enum EcCommand {
    /// Generate tests/aw-ec.toml and placeholder EC test files from TD e2e-test sections.
    Gen(EcGenArgs),
    /// Check EC manifest/list drift and generated test-file presence.
    Check(EcCheckArgs),
}

/// @spec projects/agentic-workflow/tech-design/semantic/agentic-workflow-cli.md#schema
#[derive(Debug, Args)]
pub struct EcGenArgs {
    /// Project name or alias from .aw/config.toml.
    pub project: String,
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
    /// Project name or alias from .aw/config.toml.
    pub project: String,
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
}

/// @spec projects/agentic-workflow/tech-design/semantic/agentic-workflow-cli.md#schema
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct EcManifestCase {
    pub id: String,
    pub capability_id: String,
    pub contract_id: String,
    pub category: String,
    pub td_ref: String,
    pub test_path: String,
    pub command: String,
    #[serde(default)]
    pub assertions: Vec<String>,
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
    pub stale: bool,
    pub missing_test_paths: Vec<String>,
    pub orphan_test_paths: Vec<String>,
    pub findings: Vec<String>,
}

/// @spec projects/agentic-workflow/tech-design/semantic/agentic-workflow-cli.md#schema
#[derive(Debug, Clone)]
pub struct EcProjectContext {
    pub project_root: PathBuf,
    pub project: String,
    pub source_root: PathBuf,
    pub td_root: PathBuf,
    pub tests_root: PathBuf,
    pub manifest_path: PathBuf,
    pub target: String,
    pub package_name: String,
}

#[derive(Debug, Deserialize, Default)]
struct EcConfigFile {
    #[serde(default)]
    projects: Vec<EcProjectConfig>,
}

#[derive(Debug, Deserialize, Default)]
struct EcProjectConfig {
    name: String,
    #[serde(default)]
    aliases: Vec<String>,
    path: PathBuf,
    #[serde(default)]
    workspaces: Vec<EcWorkspaceConfig>,
}

#[derive(Debug, Deserialize, Default)]
struct EcWorkspaceConfig {
    #[serde(default)]
    target: Option<String>,
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
    assertions: Option<StringOrList>,
    #[serde(default)]
    asserts: Option<StringOrList>,
    #[serde(default)]
    capability_id: Option<String>,
    #[serde(default)]
    contract_id: Option<String>,
    #[serde(default)]
    category: Option<String>,
}

#[derive(Debug, Deserialize)]
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
    match args.command {
        EcCommand::Gen(args) => run_gen(args),
        EcCommand::Check(args) => run_check(args),
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

fn run_gen(args: EcGenArgs) -> Result<()> {
    let project_root = crate::find_project_root()?;
    let ctx = resolve_ec_project_context(&project_root, &args.project)?;
    let manifest = build_expected_manifest(&ctx)?;
    let generated_files = generated_ec_test_files(&ctx, &manifest);

    if args.dry_run {
        let dry_run = serde_json::json!({
            "project": ctx.project,
            "manifest_path": relative_to(&ctx.project_root, &ctx.manifest_path),
            "case_count": manifest.cases.len(),
            "generated_from_td_digest": manifest.generated_from_td_digest,
            "test_paths": manifest.cases.iter().map(|case| &case.test_path).collect::<Vec<_>>(),
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
        }
    } else {
        write_ec_manifest(&ctx, &manifest)?;
        for (path, content) in generated_files {
            write_generated_ec_test(&path, &content)?;
        }
        if args.json {
            println!(
                "{}",
                serde_json::to_string_pretty(&serde_json::json!({
                    "project": ctx.project,
                    "manifest_path": relative_to(&ctx.project_root, &ctx.manifest_path),
                    "case_count": manifest.cases.len(),
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

fn run_check(args: EcCheckArgs) -> Result<()> {
    let project_root = crate::find_project_root()?;
    let ctx = resolve_ec_project_context(&project_root, &args.project)?;
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
                "ec check {}: clean, no TD e2e-test sections found",
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

fn resolve_ec_project_context(project_root: &Path, requested: &str) -> Result<EcProjectContext> {
    let config_path = project_root.join(".aw/config.toml");
    let content = fs::read_to_string(&config_path)
        .with_context(|| format!("read {}", config_path.display()))?;
    let config: EcConfigFile =
        toml::from_str(&content).with_context(|| format!("parse {}", config_path.display()))?;
    let Some(project) = config.projects.into_iter().find(|project| {
        project.name == requested || project.aliases.iter().any(|alias| alias == requested)
    }) else {
        bail!(
            "project `{requested}` not found in {}",
            config_path.display()
        );
    };

    let source_root = project_root.join(&project.path);
    let td_root =
        crate::services::project_registry::resolve_td_root_from_config(project_root, requested)
            .map(|resolved| PathBuf::from(resolved.root))
            .map_err(|err| anyhow::anyhow!("{}", err.message))?;
    let tests_root = source_root.join("tests");
    let manifest_path = source_root.join(EC_MANIFEST_REL);
    let target = project
        .workspaces
        .iter()
        .find_map(|workspace| workspace.target.clone())
        .unwrap_or_else(|| "rust".to_string());
    let package_name = package_name_for(&source_root).unwrap_or_else(|| project.name.clone());

    Ok(EcProjectContext {
        project_root: project_root.to_path_buf(),
        project: project.name,
        source_root,
        td_root,
        tests_root,
        manifest_path,
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

fn build_expected_manifest(ctx: &EcProjectContext) -> Result<EcManifest> {
    let mut cases = extract_td_e2e_cases(ctx)?;
    cases.sort_by(|left, right| left.id.cmp(&right.id));
    let digest = digest_cases(&cases);
    Ok(EcManifest {
        version: EC_MANIFEST_VERSION,
        project: ctx.project.clone(),
        generated_from_td_digest: digest,
        cases,
    })
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
                let category = raw
                    .category
                    .as_deref()
                    .map(slugify)
                    .filter(|value| !value.is_empty())
                    .unwrap_or_else(|| "behavior".to_string());
                let test_path = ec_test_path(ctx, &category, &id);
                let command = raw
                    .command
                    .map(|value| value.trim().to_string())
                    .filter(|value| !value.is_empty())
                    .unwrap_or_else(|| default_ec_command(ctx, &test_path));
                let assertions = raw
                    .assertions
                    .or(raw.asserts)
                    .map(StringOrList::into_vec)
                    .unwrap_or_default();
                out.push(EcManifestCase {
                    id: id.clone(),
                    capability_id: raw
                        .capability_id
                        .map(|value| value.trim().to_string())
                        .filter(|value| !value.is_empty())
                        .unwrap_or_else(|| "unmapped".to_string()),
                    contract_id: raw
                        .contract_id
                        .map(|value| value.trim().to_string())
                        .filter(|value| !value.is_empty())
                        .unwrap_or_else(|| id.clone()),
                    category,
                    td_ref: format!("{}#{}", relative_to(&ctx.project_root, path), id),
                    test_path: relative_to(&ctx.project_root, &test_path),
                    command,
                    assertions,
                });
            }
            idx = next_idx;
            continue;
        }
        idx += 1;
    }
    Ok(out)
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

fn digest_cases(cases: &[EcManifestCase]) -> String {
    let mut sorted = cases.to_vec();
    sorted.sort_by(|left, right| left.id.cmp(&right.id));
    let mut hasher = Sha256::new();
    for case in sorted {
        hash_field(&mut hasher, &case.id);
        hash_field(&mut hasher, &case.capability_id);
        hash_field(&mut hasher, &case.contract_id);
        hash_field(&mut hasher, &case.category);
        hash_field(&mut hasher, &case.td_ref);
        hash_field(&mut hasher, &case.test_path);
        hash_field(&mut hasher, &case.command);
        for assertion in &case.assertions {
            hash_field(&mut hasher, assertion);
        }
    }
    format!("sha256:{:x}", hasher.finalize())
}

fn hash_field(hasher: &mut Sha256, value: &str) {
    hasher.update(value.as_bytes());
    hasher.update([0]);
}

fn load_ec_manifest(ctx: &EcProjectContext) -> Result<Option<(PathBuf, EcManifest)>> {
    if !ctx.manifest_path.is_file() {
        return Ok(None);
    }
    let content = fs::read_to_string(&ctx.manifest_path)
        .with_context(|| format!("read {}", ctx.manifest_path.display()))?;
    let manifest: EcManifest = toml::from_str(&content)
        .with_context(|| format!("parse {}", ctx.manifest_path.display()))?;
    Ok(Some((ctx.manifest_path.clone(), manifest)))
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
    } else if !expected.cases.is_empty() {
        findings.push(format!(
            "EC manifest missing at {}; run `aw ec gen {}`",
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
        if expected_case.assertions != actual_case.assertions {
            findings.push(format!(
                "manifest case `{}` assertions drifted",
                expected_case.id
            ));
        }
    }

    for actual_case in &actual_cases {
        if !expected_by_id.contains_key(actual_case.id.as_str()) {
            findings.push(format!(
                "manifest has orphan EC case `{}` not present in TD e2e-test sections",
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
                        "EC test file {} generated content drifted for `{}`; run `aw ec gen {}`",
                        actual_case.test_path, actual_case.id, ctx.project
                    ));
                }
            }
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
    let stale = actual
        .map(|manifest| manifest.generated_from_td_digest != expected.generated_from_td_digest)
        .unwrap_or(!expected.cases.is_empty());

    Ok(EcCheckSummary {
        project: ctx.project.clone(),
        clean: findings.is_empty(),
        configured,
        manifest_path,
        generated_from_td_digest: expected.generated_from_td_digest.clone(),
        manifest_td_digest,
        expected_case_count: expected.cases.len(),
        case_count: actual_cases.len(),
        stale,
        missing_test_paths,
        orphan_test_paths,
        findings,
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
    if let Some(parent) = ctx.manifest_path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }
    let body = format!(
        "# SPEC-MANAGED: generated by `aw ec gen {}` from TD e2e-test sections.\n# CODEGEN-BEGIN\n{}# CODEGEN-END\n",
        ctx.project,
        toml::to_string_pretty(manifest)?
    );
    fs::write(&ctx.manifest_path, body)
        .with_context(|| format!("write {}", ctx.manifest_path.display()))
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

fn render_rust_ec_test(case: &EcManifestCase) -> String {
    let fn_name = rust_ident(&case.id);
    let placeholder = rust_string_literal(&format!("AW EC placeholder for {}", case.id));
    format!(
        "// SPEC-MANAGED: {}\n// CODEGEN-BEGIN\n// {EC_BEGIN_MARKER}\n// @ec {}\n// @capability {}\n// @contract {}\n// @category {}\n// @command {}\n// {EC_END_MARKER}\n\n#[test]\n#[ignore = \"AW EC placeholder: implement this external contract test or keep the manifest command authoritative\"]\nfn {fn_name}() {{\n    panic!({});\n}}\n// CODEGEN-END\n",
        case.td_ref,
        case.id,
        case.capability_id,
        case.contract_id,
        case.category,
        case.command,
        placeholder
    )
}

fn render_python_ec_test(case: &EcManifestCase) -> String {
    format!(
        "# SPEC-MANAGED: {}\n# CODEGEN-BEGIN\n# {EC_BEGIN_MARKER}\n# @ec {}\n# @capability {}\n# @contract {}\n# @category {}\n# @command {}\n# {EC_END_MARKER}\n\nimport pytest\n\n\n@pytest.mark.skip(reason=\"AW EC placeholder: implement this external contract test or keep the manifest command authoritative\")\ndef test_{}():\n    raise AssertionError(\"AW EC placeholder for {}\")\n# CODEGEN-END\n",
        case.td_ref,
        case.id,
        case.capability_id,
        case.contract_id,
        case.category,
        case.command,
        rust_ident(&case.id),
        escape_py_string(&case.id)
    )
}

fn render_ts_ec_test(case: &EcManifestCase) -> String {
    format!(
        "// SPEC-MANAGED: {}\n// CODEGEN-BEGIN\n// {EC_BEGIN_MARKER}\n// @ec {}\n// @capability {}\n// @contract {}\n// @category {}\n// @command {}\n// {EC_END_MARKER}\n\nimport {{ test }} from \"vitest\";\n\ntest.skip({}, () => {{\n  throw new Error({});\n}});\n// CODEGEN-END\n",
        case.td_ref,
        case.id,
        case.capability_id,
        case.contract_id,
        case.category,
        case.command,
        serde_json::to_string(&case.id).unwrap_or_else(|_| "\"aw-ec\"".to_string()),
        serde_json::to_string(&format!("AW EC placeholder for {}", case.id))
            .unwrap_or_else(|_| "\"AW EC placeholder\"".to_string())
    )
}

fn render_text_ec_test(case: &EcManifestCase) -> String {
    format!(
        "SPEC-MANAGED: {}\nCODEGEN-BEGIN\n{EC_BEGIN_MARKER}\n@ec {}\n@capability {}\n@contract {}\n@category {}\n@command {}\n{EC_END_MARKER}\nCODEGEN-END\n",
        case.td_ref, case.id, case.capability_id, case.contract_id, case.category, case.command
    )
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
```
"#,
        )
        .unwrap();
        let ctx = resolve_ec_project_context(tmp.path(), "d").unwrap();
        (tmp, ctx)
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
        assert!(case.td_ref.ends_with("contract.md#demo-happy-path"));
        assert_eq!(
            case.test_path,
            "projects/demo/tests/behavior_demo_happy_path.rs"
        );
        assert!(manifest.generated_from_td_digest.starts_with("sha256:"));
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
}
// CODEGEN-END
