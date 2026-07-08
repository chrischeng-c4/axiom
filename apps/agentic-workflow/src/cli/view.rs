// HANDWRITE-BEGIN gap="missing-generator:view-cli-snapshot" tracker="pending-tracker" reason="Read-only repo visual-reader snapshot and desktop app surface; codegen ownership follows after the repo-level contract stabilizes."
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::rc::Rc;

use anyhow::{Context, Result};
use clap::{Args, ValueEnum};
use serde::{Deserialize, Serialize};

use cclab_surface::{Callback, Component, Element, Props, SurfaceSnapshot};

use crate::cli::capability::{collect_td_capability_refs, parse_capability_document};
use crate::cli::ec::EcManifest;
use crate::services::project_registry::{load_project_config_rows, ProjectConfigRow};

#[derive(Debug, Clone, Args)]
pub struct ViewArgs {
    /// Project/lib token to select when the repo desktop app opens, e.g. agentic-workflow or jet.
    #[arg(long)]
    pub focus: Option<String>,

    /// Emit a stable JSON snapshot for tests, EC gates, and native desktop renderers.
    #[arg(long)]
    pub snapshot: bool,

    /// Run a headless contract check without opening the desktop window.
    #[arg(long)]
    pub check: bool,

    /// Render a browser-style app screenshot PNG from the renderer-neutral surface tree.
    #[arg(long, value_name = "PNG")]
    pub screenshot: Option<PathBuf>,

    /// Build a macOS .app launcher for the native desktop repo view.
    #[arg(long, value_name = "APP")]
    pub app: Option<PathBuf>,

    /// Layout for the terminal and project detail area to the right of the project list.
    #[arg(long, value_enum, default_value_t = ViewLayout::LeftRight)]
    pub layout: ViewLayout,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, ValueEnum)]
#[serde(rename_all = "kebab-case")]
pub enum ViewLayout {
    LeftRight,
    TopBottom,
}

impl std::fmt::Display for ViewLayout {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(match self {
            Self::LeftRight => "left-right",
            Self::TopBottom => "top-bottom",
        })
    }
}

pub fn toggled_view_layout(layout: ViewLayout) -> ViewLayout {
    match layout {
        ViewLayout::LeftRight => ViewLayout::TopBottom,
        ViewLayout::TopBottom => ViewLayout::LeftRight,
    }
}

pub fn layout_toggle_button_label(layout: ViewLayout) -> &'static str {
    match layout {
        ViewLayout::LeftRight => "Toggle: top-bottom",
        ViewLayout::TopBottom => "Toggle: left-right",
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct RepoViewSnapshot {
    pub schema_version: u32,
    pub layout: ViewLayout,
    pub repo: RepoViewRepo,
    pub repo_catalog: Vec<RepoCatalogItem>,
    pub selected_repo: Option<String>,
    pub terminal: TerminalSnapshot,
    pub catalog: Vec<ProjectCatalogItem>,
    pub selected: Option<String>,
    pub items: Vec<RepoViewItemSnapshot>,
    pub surface: SurfaceSnapshot,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct RepoViewRepo {
    pub name: String,
    pub root: String,
    pub item_count: usize,
    pub project_count: usize,
    pub library_count: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct RepoCatalogItem {
    pub name: String,
    pub path: String,
    pub current: bool,
    pub item_count: usize,
    pub project_count: usize,
    pub library_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct UserRepoRegistry {
    #[serde(default)]
    repos: Vec<UserRepoRegistryEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct UserRepoRegistryEntry {
    name: String,
    path: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct TerminalSnapshot {
    pub title: String,
    pub lines: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct RepoViewItemSnapshot {
    pub project: ProjectViewProject,
    pub readme: ReadmeSnapshot,
    pub capabilities: CapabilitySnapshot,
    pub ec: EcSnapshot,
    pub td: TdSnapshot,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ProjectViewProject {
    pub name: String,
    pub aliases: Vec<String>,
    pub kind: String,
    pub path: String,
    pub td_path: String,
    pub cap_path: String,
    pub label: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ProjectCatalogItem {
    pub name: String,
    pub aliases: Vec<String>,
    pub kind: String,
    pub path: String,
    pub cap_path: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ReadmeSnapshot {
    pub path: String,
    pub title: String,
    pub brief: String,
    pub format_version: u8,
    pub finding_count: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct CapabilitySnapshot {
    pub count: usize,
    pub items: Vec<CapabilitySnapshotItem>,
}

#[derive(Debug, Clone, Serialize)]
pub struct CapabilitySnapshotItem {
    pub id: String,
    pub title: String,
    pub status: String,
    pub capability_type: Option<String>,
    pub surface_count: usize,
    pub ec_dimension_count: usize,
    pub claim_count: usize,
    pub td_ref_count: usize,
    pub ec_case_count: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct EcSnapshot {
    pub inventory_path: String,
    pub present: bool,
    pub generated: bool,
    pub case_count: usize,
    pub production_case_count: usize,
    pub by_category: BTreeMap<String, usize>,
    pub cases: Vec<EcCaseSnapshot>,
}

#[derive(Debug, Clone, Serialize)]
pub struct EcCaseSnapshot {
    pub id: String,
    pub capability_id: String,
    pub claim_id: String,
    pub category: String,
    pub command: String,
    pub td_ref: String,
    pub required_for_production: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct TdSnapshot {
    pub root: String,
    pub markdown_file_count: usize,
    pub capability_ref_count: usize,
}

pub async fn run(args: ViewArgs) -> Result<()> {
    let root = crate::find_project_root()?;
    let headless_modes = usize::from(args.snapshot)
        + usize::from(args.check)
        + usize::from(args.screenshot.is_some())
        + usize::from(args.app.is_some());
    if headless_modes > 1 {
        anyhow::bail!("choose only one of --snapshot, --check, --screenshot, or --app");
    }
    if let Some(path) = args.app.as_deref() {
        build_desktop_app_bundle(&root, path, args.focus.as_deref(), args.layout)?;
        println!("view app: wrote {}", path.display());
        return Ok(());
    }
    let snapshot = build_repo_view_snapshot(&root, args.focus.as_deref(), args.layout)?;
    if args.snapshot {
        println!("{}", serde_json::to_string_pretty(&snapshot)?);
        return Ok(());
    }
    if args.check {
        println!("{}", headless_contract_check(&snapshot)?);
        return Ok(());
    }
    if let Some(path) = args.screenshot.as_deref() {
        render_app_screenshot(&snapshot, path)?;
        println!("view screenshot: wrote {}", path.display());
        return Ok(());
    }

    #[cfg(feature = "ui")]
    {
        crate::ui::native_view::run_native_repo_view(&snapshot)?;
        return Ok(());
    }
    #[cfg(not(feature = "ui"))]
    {
        anyhow::bail!(
            "aw view opens the native desktop app and requires the `ui` feature; use --snapshot or --check for headless output"
        );
    }
}

pub fn build_repo_view_snapshot(
    root: &Path,
    focus: Option<&str>,
    layout: ViewLayout,
) -> Result<RepoViewSnapshot> {
    build_repo_view_snapshot_with_repo_registry_path(root, focus, layout, user_repo_registry_path())
}

fn build_repo_view_snapshot_with_repo_registry_path(
    root: &Path,
    focus: Option<&str>,
    layout: ViewLayout,
    repo_registry_path: Option<PathBuf>,
) -> Result<RepoViewSnapshot> {
    let rows = load_project_config_rows(root)?;
    let catalog = project_catalog(rows.clone());
    let mut warnings = Vec::new();
    let selected = select_catalog_item(&catalog, focus)?;
    let mut items = Vec::new();
    for row in rows {
        match build_repo_view_item_snapshot(root, &row) {
            Ok(item) => items.push(item),
            Err(err) => {
                warnings.push(format!("{} unavailable: {err}", row.name));
                items.push(empty_repo_view_item_snapshot(&row, err.to_string()));
            }
        }
    }
    items.sort_by(|left, right| {
        left.project
            .kind
            .cmp(&right.project.kind)
            .then_with(|| left.project.name.cmp(&right.project.name))
    });
    let repo_root = canonical_repo_path(root);
    let repo = RepoViewRepo {
        name: root
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("repo")
            .to_string(),
        root: repo_root.display().to_string(),
        item_count: catalog.len(),
        project_count: catalog.iter().filter(|item| item.kind == "project").count(),
        library_count: catalog.iter().filter(|item| item.kind == "library").count(),
    };
    let repo_catalog = load_or_update_repo_catalog(root, &repo, repo_registry_path, &mut warnings);
    let selected_repo = Some(repo.root.clone());
    let terminal = build_terminal_snapshot(&root, &repo, selected.as_deref(), &items, layout);
    let surface = build_surface_snapshot(
        &repo,
        &repo_catalog,
        selected_repo.as_deref(),
        &terminal,
        &catalog,
        selected.as_deref(),
        &items,
        layout,
    );

    Ok(RepoViewSnapshot {
        schema_version: 1,
        layout,
        repo,
        repo_catalog,
        selected_repo,
        terminal,
        catalog,
        selected,
        items,
        surface,
        warnings,
    })
}

fn build_repo_view_item_snapshot(
    root: &Path,
    row: &ProjectConfigRow,
) -> Result<RepoViewItemSnapshot> {
    let project = project_view_project(row);
    let cap_path = root.join(&project.cap_path);
    let cap_body = std::fs::read_to_string(&cap_path)
        .with_context(|| format!("failed to read capability map {}", cap_path.display()))?;
    let document = parse_capability_document(&cap_body, &cap_path)
        .with_context(|| format!("failed to parse capability map {}", cap_path.display()))?;

    let mut warnings = document.findings.clone();
    let readme_rel = format!("{}/README.md", project.path);
    let readme_path = root.join(&readme_rel);
    let (readme_body, readme_snapshot_path) = if readme_path == cap_path {
        (cap_body.clone(), project.cap_path.clone())
    } else {
        match std::fs::read_to_string(&readme_path) {
            Ok(body) => (body, readme_rel),
            Err(err) => {
                warnings.push(format!("README unavailable: {err}"));
                (cap_body.clone(), project.cap_path.clone())
            }
        }
    };
    let td_refs = match collect_td_capability_refs(root, &row.name, &document) {
        Ok(refs) => refs,
        Err(err) => {
            warnings.push(format!("td capability refs unavailable: {err}"));
            Vec::new()
        }
    };
    let ec = load_ec_snapshot(root, &row)?;
    let td = td_snapshot(root, &project.td_path, td_refs.len())?;
    let readme = ReadmeSnapshot {
        path: readme_snapshot_path,
        title: extract_h1(&readme_body).unwrap_or_else(|| row.name.clone()),
        brief: extract_brief(&readme_body).unwrap_or_default(),
        format_version: document.format_version(),
        finding_count: document.findings.len(),
    };
    let capabilities = capability_snapshot(&document.capabilities, &td_refs, &ec.cases);

    Ok(RepoViewItemSnapshot {
        project,
        readme,
        capabilities,
        ec,
        td,
        warnings,
    })
}

fn empty_repo_view_item_snapshot(row: &ProjectConfigRow, warning: String) -> RepoViewItemSnapshot {
    let project = project_view_project(row);
    RepoViewItemSnapshot {
        readme: ReadmeSnapshot {
            path: project.cap_path.clone(),
            title: row.name.clone(),
            brief: String::new(),
            format_version: 0,
            finding_count: 0,
        },
        capabilities: CapabilitySnapshot {
            count: 0,
            items: Vec::new(),
        },
        ec: empty_ec_snapshot(format!("{}/aw.toml", project.path), false),
        td: TdSnapshot {
            root: project.td_path.clone(),
            markdown_file_count: 0,
            capability_ref_count: 0,
        },
        project,
        warnings: vec![warning],
    }
}

fn select_catalog_item(
    catalog: &[ProjectCatalogItem],
    focus: Option<&str>,
) -> Result<Option<String>> {
    if let Some(focus) = focus {
        let selected = catalog
            .iter()
            .find(|item| item.name == focus || item.aliases.iter().any(|alias| alias == focus))
            .map(|item| item.name.clone())
            .ok_or_else(|| anyhow::anyhow!("repo item `{focus}` has no AW project config row"))?;
        return Ok(Some(selected));
    }
    if let Some(item) = catalog.iter().find(|item| item.name == "agentic-workflow") {
        return Ok(Some(item.name.clone()));
    }
    Ok(catalog.first().map(|item| item.name.clone()))
}

fn project_view_project(row: &ProjectConfigRow) -> ProjectViewProject {
    let td_path = row
        .td_path
        .clone()
        .unwrap_or_else(|| format!("{}/tech-design", row.path));
    let cap_path = row
        .cap_path
        .clone()
        .unwrap_or_else(|| format!("{}/README.md", row.path));
    ProjectViewProject {
        name: row.name.clone(),
        aliases: row.aliases.clone(),
        kind: project_kind(&row.path).to_string(),
        path: row.path.clone(),
        td_path,
        cap_path,
        label: row.label_or_default(),
    }
}

fn project_catalog(mut rows: Vec<ProjectConfigRow>) -> Vec<ProjectCatalogItem> {
    rows.sort_by(|left, right| {
        project_kind(&left.path)
            .cmp(project_kind(&right.path))
            .then_with(|| left.name.cmp(&right.name))
    });
    rows.into_iter()
        .map(|row| ProjectCatalogItem {
            name: row.name,
            aliases: row.aliases,
            kind: project_kind(&row.path).to_string(),
            path: row.path,
            cap_path: row.cap_path,
        })
        .collect()
}

fn project_kind(path: &str) -> &'static str {
    if path.starts_with("libs/") {
        "library"
    } else {
        "project"
    }
}

fn user_repo_registry_path() -> Option<PathBuf> {
    dirs::home_dir().map(|home| home.join(".aw").join("repos.toml"))
}

fn load_or_update_repo_catalog(
    root: &Path,
    repo: &RepoViewRepo,
    registry_path: Option<PathBuf>,
    warnings: &mut Vec<String>,
) -> Vec<RepoCatalogItem> {
    let current_path = canonical_repo_path(root);
    let current_entry = UserRepoRegistryEntry {
        name: repo.name.clone(),
        path: current_path.display().to_string(),
    };
    let Some(registry_path) = registry_path else {
        return vec![repo_catalog_item_from_entry(
            &current_entry,
            true,
            Some(repo),
        )];
    };

    let mut registry = match read_user_repo_registry(&registry_path) {
        Ok(registry) => registry,
        Err(err) => {
            warnings.push(format!(
                "repo registry unavailable at {}: {err}",
                registry_path.display()
            ));
            return vec![repo_catalog_item_from_entry(
                &current_entry,
                true,
                Some(repo),
            )];
        }
    };
    upsert_user_repo_entry(&mut registry, current_entry.clone());
    if let Err(err) = write_user_repo_registry(&registry_path, &registry) {
        warnings.push(format!(
            "repo registry write failed at {}: {err}",
            registry_path.display()
        ));
    }

    let current_path_string = current_entry.path.clone();
    let mut catalog = registry
        .repos
        .iter()
        .map(|entry| {
            let current = entry.path == current_path_string;
            repo_catalog_item_from_entry(entry, current, current.then_some(repo))
        })
        .collect::<Vec<_>>();
    if !catalog.iter().any(|item| item.current) {
        catalog.push(repo_catalog_item_from_entry(
            &current_entry,
            true,
            Some(repo),
        ));
    }
    catalog.sort_by(|left, right| {
        (!left.current)
            .cmp(&(!right.current))
            .then_with(|| left.name.cmp(&right.name))
            .then_with(|| left.path.cmp(&right.path))
    });
    catalog
}

fn read_user_repo_registry(path: &Path) -> Result<UserRepoRegistry> {
    match std::fs::read_to_string(path) {
        Ok(body) => toml::from_str(&body)
            .with_context(|| format!("failed to parse repo registry {}", path.display())),
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(UserRepoRegistry::default()),
        Err(err) => Err(err).with_context(|| format!("failed to read {}", path.display())),
    }
}

fn write_user_repo_registry(path: &Path, registry: &UserRepoRegistry) -> Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("failed to create {}", parent.display()))?;
    }
    let body = toml::to_string_pretty(registry).context("failed to serialize repo registry")?;
    std::fs::write(path, body).with_context(|| format!("failed to write {}", path.display()))
}

fn upsert_user_repo_entry(registry: &mut UserRepoRegistry, entry: UserRepoRegistryEntry) {
    registry
        .repos
        .retain(|existing| existing.path != entry.path);
    registry.repos.push(entry);
}

fn repo_catalog_item_from_entry(
    entry: &UserRepoRegistryEntry,
    current: bool,
    current_repo: Option<&RepoViewRepo>,
) -> RepoCatalogItem {
    if let Some(repo) = current_repo {
        return RepoCatalogItem {
            name: repo.name.clone(),
            path: entry.path.clone(),
            current,
            item_count: repo.item_count,
            project_count: repo.project_count,
            library_count: repo.library_count,
        };
    }
    let (item_count, project_count, library_count) = repo_counts_for_path(Path::new(&entry.path));
    RepoCatalogItem {
        name: entry.name.clone(),
        path: entry.path.clone(),
        current,
        item_count,
        project_count,
        library_count,
    }
}

fn repo_counts_for_path(path: &Path) -> (usize, usize, usize) {
    let Ok(rows) = load_project_config_rows(path) else {
        return (0, 0, 0);
    };
    let item_count = rows.len();
    let project_count = rows
        .iter()
        .filter(|row| project_kind(&row.path) == "project")
        .count();
    let library_count = rows
        .iter()
        .filter(|row| project_kind(&row.path) == "library")
        .count();
    (item_count, project_count, library_count)
}

fn canonical_repo_path(path: &Path) -> PathBuf {
    std::fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf())
}

fn load_ec_snapshot(root: &Path, row: &ProjectConfigRow) -> Result<EcSnapshot> {
    let inventory_path = root.join(&row.path).join("aw.toml");
    let inventory_rel = relative_to(root, &inventory_path);
    if !inventory_path.exists() {
        return Ok(empty_ec_snapshot(inventory_rel, false));
    }
    let body = std::fs::read_to_string(&inventory_path)
        .with_context(|| format!("failed to read EC inventory {}", inventory_path.display()))?;
    let value: toml::Value = toml::from_str(&body)
        .with_context(|| format!("failed to parse EC inventory {}", inventory_path.display()))?;
    let Some(generated) = value
        .get("aw")
        .and_then(|aw| aw.get("ec"))
        .and_then(|ec| ec.get("generated"))
    else {
        return Ok(empty_ec_snapshot(inventory_rel, true));
    };
    let manifest: EcManifest = generated
        .clone()
        .try_into()
        .with_context(|| format!("failed to parse [aw.ec.generated] in {inventory_rel}"))?;
    let mut by_category = BTreeMap::new();
    let mut cases = Vec::new();
    let mut production_case_count = 0usize;
    for case in manifest.cases {
        *by_category.entry(case.category.clone()).or_insert(0) += 1;
        if case.required_for_production {
            production_case_count += 1;
        }
        cases.push(EcCaseSnapshot {
            id: case.id,
            capability_id: case.capability_id,
            claim_id: case.claim_id,
            category: case.category,
            command: case.command,
            td_ref: case.td_ref,
            required_for_production: case.required_for_production,
        });
    }
    Ok(EcSnapshot {
        inventory_path: inventory_rel,
        present: true,
        generated: true,
        case_count: cases.len(),
        production_case_count,
        by_category,
        cases,
    })
}

fn empty_ec_snapshot(inventory_path: String, present: bool) -> EcSnapshot {
    EcSnapshot {
        inventory_path,
        present,
        generated: false,
        case_count: 0,
        production_case_count: 0,
        by_category: BTreeMap::new(),
        cases: Vec::new(),
    }
}

fn td_snapshot(root: &Path, td_path: &str, capability_ref_count: usize) -> Result<TdSnapshot> {
    let td_root = root.join(td_path);
    let mut markdown_file_count = 0usize;
    if td_root.exists() {
        for entry in walkdir::WalkDir::new(&td_root)
            .into_iter()
            .filter_map(|entry| entry.ok())
            .filter(|entry| entry.file_type().is_file())
        {
            if entry.path().extension().and_then(|ext| ext.to_str()) == Some("md") {
                markdown_file_count += 1;
            }
        }
    }
    Ok(TdSnapshot {
        root: td_path.to_string(),
        markdown_file_count,
        capability_ref_count,
    })
}

fn capability_snapshot(
    capabilities: &[crate::cli::capability::CapabilitySection],
    td_refs: &[crate::cli::capability::TdCapabilityEvidence],
    ec_cases: &[EcCaseSnapshot],
) -> CapabilitySnapshot {
    let mut items = Vec::new();
    for cap in capabilities {
        let td_ref_count = td_refs
            .iter()
            .filter(|td_ref| td_ref.capability_id == cap.id)
            .count();
        let ec_case_count = ec_cases
            .iter()
            .filter(|case| case.capability_id == cap.id)
            .count();
        let claim_count = cap
            .verification_contract
            .as_ref()
            .map(|contract| contract.claims.len())
            .unwrap_or_else(|| cap.work_roots.len());
        items.push(CapabilitySnapshotItem {
            id: cap.id.clone(),
            title: cap.title.clone(),
            status: cap.status.as_str().to_string(),
            capability_type: cap
                .capability_type
                .map(|capability_type| capability_type.as_str().to_string()),
            surface_count: cap.surfaces.len(),
            ec_dimension_count: cap.ec_dimensions.len(),
            claim_count,
            td_ref_count,
            ec_case_count,
        });
    }
    CapabilitySnapshot {
        count: items.len(),
        items,
    }
}

fn build_terminal_snapshot(
    root: &Path,
    repo: &RepoViewRepo,
    selected: Option<&str>,
    items: &[RepoViewItemSnapshot],
    layout: ViewLayout,
) -> TerminalSnapshot {
    let selected_item = selected
        .and_then(|name| items.iter().find(|item| item.project.name == name))
        .or_else(|| items.first());
    let view_command = if layout == ViewLayout::LeftRight {
        "$ aw view".to_string()
    } else {
        format!("$ aw view --layout {layout}")
    };
    let mut lines = vec![
        view_command,
        format!("repo {}", repo.root),
        format!(
            "items {} | projects {} | libs {}",
            repo.item_count, repo.project_count, repo.library_count
        ),
        String::new(),
        "$ git status --short --branch".to_string(),
    ];
    match std::process::Command::new("git")
        .arg("-C")
        .arg(root)
        .args(["status", "--short", "--branch"])
        .output()
    {
        Ok(output) if output.status.success() => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let mut count = 0usize;
            for line in stdout.lines().take(10) {
                lines.push(line.to_string());
                count += 1;
            }
            if count == 0 {
                lines.push("clean".to_string());
            }
        }
        Ok(output) => {
            let stderr = String::from_utf8_lossy(&output.stderr);
            lines.push(format!("git status failed: {}", stderr.trim()));
        }
        Err(err) => lines.push(format!("git status unavailable: {err}")),
    }
    lines.push(String::new());
    lines.push("$ aw view --check".to_string());
    lines.push(
        "surface repo-layout-toggle repo-terminal repo-catalog repo-project-selector repo-readme-detail repo-capability-table repo-ec-detail repo-td-detail"
            .to_string(),
    );
    if let Some(item) = selected_item {
        lines.push(format!("selected {}", item.project.name));
        lines.push(format!(
            "capabilities {} | EC {} | TD files {}",
            item.capabilities.count, item.ec.case_count, item.td.markdown_file_count
        ));
        lines.push(format!("readme {}", item.readme.path));
        if item.ec.cases.is_empty() {
            lines.push("EC cases none".to_string());
        } else {
            lines.push("EC cases".to_string());
            for case in item.ec.cases.iter().take(6) {
                lines.push(format!("  {} [{}]", case.id, case.category));
            }
        }
    } else {
        lines.push("selected none".to_string());
    }
    TerminalSnapshot {
        title: "Terminal".to_string(),
        lines,
    }
}

fn build_surface_snapshot(
    repo: &RepoViewRepo,
    repo_catalog: &[RepoCatalogItem],
    selected_repo: Option<&str>,
    terminal: &TerminalSnapshot,
    catalog: &[ProjectCatalogItem],
    selected: Option<&str>,
    items: &[RepoViewItemSnapshot],
    layout: ViewLayout,
) -> SurfaceSnapshot {
    let props = RepoSurfaceProps {
        repo: repo.clone(),
        repo_catalog: repo_catalog.to_vec(),
        selected_repo: selected_repo.map(str::to_string),
        terminal: terminal.clone(),
        catalog: catalog.to_vec(),
        selected: selected.map(str::to_string),
        items: items.to_vec(),
        layout,
    };
    let component = Component {
        name: "AwRepoView",
        render: render_repo_surface,
        props: Rc::new(props),
    };
    cclab_ui_runtime::mount(component)
        .snapshot()
        .surface_snapshot()
}

#[derive(Clone)]
struct RepoSurfaceProps {
    repo: RepoViewRepo,
    repo_catalog: Vec<RepoCatalogItem>,
    selected_repo: Option<String>,
    terminal: TerminalSnapshot,
    catalog: Vec<ProjectCatalogItem>,
    selected: Option<String>,
    items: Vec<RepoViewItemSnapshot>,
    layout: ViewLayout,
}

fn render_repo_surface(props: &Rc<dyn std::any::Any>) -> Element {
    let props = props
        .downcast_ref::<RepoSurfaceProps>()
        .expect("AwRepoView props type mismatch");
    build_surface_element(
        &props.repo,
        &props.repo_catalog,
        props.selected_repo.as_deref(),
        &props.terminal,
        &props.catalog,
        props.selected.as_deref(),
        &props.items,
        props.layout,
    )
}

fn build_surface_element(
    repo: &RepoViewRepo,
    repo_catalog: &[RepoCatalogItem],
    selected_repo: Option<&str>,
    terminal: &TerminalSnapshot,
    catalog: &[ProjectCatalogItem],
    selected: Option<&str>,
    items: &[RepoViewItemSnapshot],
    layout: ViewLayout,
) -> Element {
    let nav_items = repo_catalog
        .iter()
        .map(|item| {
            Element::intrinsic(
                "button",
                Props {
                    id: Some(format!("repo-item-{}", item.name)),
                    aria_label: Some(format!("repo {} {}", item.name, item.path)),
                    ..Default::default()
                },
                vec![Element::text(format!(
                    "{} {}",
                    if selected_repo == Some(item.path.as_str()) {
                        "current"
                    } else {
                        "repo"
                    },
                    item.name
                ))],
            )
        })
        .collect::<Vec<_>>();
    let selected_item = selected
        .and_then(|name| items.iter().find(|item| item.project.name == name))
        .or_else(|| items.first());
    let capability_rows = selected_item
        .map(|item| {
            item.capabilities
                .items
                .iter()
                .map(|capability| {
                    Element::intrinsic(
                        "tr",
                        Props {
                            id: Some(format!("repo-capability-{}", capability.id)),
                            ..Default::default()
                        },
                        vec![
                            Element::intrinsic(
                                "td",
                                Props::default(),
                                vec![Element::text(&capability.title)],
                            ),
                            Element::intrinsic(
                                "td",
                                Props::default(),
                                vec![Element::text(&capability.status)],
                            ),
                        ],
                    )
                })
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    let readme_text = selected_item
        .map(|item| format!("{} {}", item.readme.title, item.readme.brief))
        .unwrap_or_else(|| "No repo item selected".to_string());
    let ec_text = selected_item
        .map(|item| format!("{} EC cases", item.ec.case_count))
        .unwrap_or_else(|| "0 EC cases".to_string());
    let td_text = selected_item
        .map(|item| format!("{} TD markdown files", item.td.markdown_file_count))
        .unwrap_or_else(|| "0 TD markdown files".to_string());
    let terminal_text = terminal.lines.join("\n");
    let project_selector_items = catalog
        .iter()
        .map(|item| {
            Element::intrinsic(
                "option",
                Props {
                    id: Some(format!("repo-project-option-{}", item.name)),
                    aria_label: Some(format!("{} {}", item.kind, item.name)),
                    ..Default::default()
                },
                vec![Element::text(format!("{} [{}]", item.name, item.kind))],
            )
        })
        .collect::<Vec<_>>();
    Element::intrinsic(
        "main",
        Props {
            id: Some("aw-view".to_string()),
            aria_label: Some(format!("AW view repo {}", repo.name)),
            ..Default::default()
        },
        vec![
            Element::intrinsic(
                "button",
                Props {
                    id: Some("repo-layout-toggle".to_string()),
                    aria_label: Some("Toggle terminal and detail layout".to_string()),
                    on_click: Some(Callback::new(|_| {})),
                    ..Default::default()
                },
                vec![Element::text(layout_toggle_button_label(layout))],
            ),
            Element::intrinsic(
                "section",
                Props {
                    id: Some("repo-terminal".to_string()),
                    aria_label: Some(terminal.title.clone()),
                    ..Default::default()
                },
                vec![Element::text(terminal_text)],
            ),
            Element::intrinsic(
                "nav",
                Props {
                    id: Some("repo-catalog".to_string()),
                    aria_label: Some("Repos".to_string()),
                    ..Default::default()
                },
                nav_items,
            ),
            Element::intrinsic(
                "select",
                Props {
                    id: Some("repo-project-selector".to_string()),
                    aria_label: Some("Project or library selector".to_string()),
                    ..Default::default()
                },
                project_selector_items,
            ),
            Element::intrinsic(
                "section",
                Props {
                    id: Some("repo-readme-detail".to_string()),
                    aria_label: Some("Project brief".to_string()),
                    ..Default::default()
                },
                vec![Element::text(readme_text)],
            ),
            Element::intrinsic(
                "table",
                Props {
                    id: Some("repo-capability-table".to_string()),
                    aria_label: Some("Capabilities".to_string()),
                    ..Default::default()
                },
                capability_rows,
            ),
            Element::intrinsic(
                "section",
                Props {
                    id: Some("repo-ec-detail".to_string()),
                    aria_label: Some("External contracts".to_string()),
                    ..Default::default()
                },
                vec![Element::text(ec_text)],
            ),
            Element::intrinsic(
                "section",
                Props {
                    id: Some("repo-td-detail".to_string()),
                    aria_label: Some("Tech designs".to_string()),
                    ..Default::default()
                },
                vec![Element::text(td_text)],
            ),
        ],
    )
}

pub fn headless_contract_check(snapshot: &RepoViewSnapshot) -> Result<String> {
    let selected = selected_item(snapshot);
    Ok(format!(
        "view check: repo={} repos={} items={} layout={} selected={} terminal_lines={} capabilities={} ec_cases={} td_files={} surface_ids=repo-layout-toggle,repo-terminal,repo-catalog,repo-project-selector,repo-readme-detail,repo-capability-table,repo-ec-detail,repo-td-detail",
        snapshot.repo.name,
        snapshot.repo_catalog.len(),
        snapshot.repo.item_count,
        snapshot.layout,
        selected
            .map(|item| item.project.name.as_str())
            .unwrap_or("none"),
        snapshot.terminal.lines.len(),
        selected.map(|item| item.capabilities.count).unwrap_or_default(),
        selected.map(|item| item.ec.case_count).unwrap_or_default(),
        selected
            .map(|item| item.td.markdown_file_count)
            .unwrap_or_default()
    ))
}

fn selected_item(snapshot: &RepoViewSnapshot) -> Option<&RepoViewItemSnapshot> {
    snapshot
        .selected
        .as_deref()
        .and_then(|selected| {
            snapshot
                .items
                .iter()
                .find(|item| item.project.name == selected)
        })
        .or_else(|| snapshot.items.first())
}

fn extract_h1(markdown: &str) -> Option<String> {
    markdown.lines().find_map(|line| {
        line.strip_prefix("# ")
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(str::to_string)
    })
}

fn extract_brief(markdown: &str) -> Option<String> {
    let mut in_brief = false;
    let mut lines = Vec::new();
    for line in markdown.lines() {
        if line.trim() == "## Brief" {
            in_brief = true;
            continue;
        }
        if in_brief && line.starts_with("## ") {
            break;
        }
        if in_brief {
            let trimmed = line.trim();
            if !trimmed.is_empty() {
                lines.push(trimmed);
            }
        }
    }
    if lines.is_empty() {
        None
    } else {
        Some(lines.join(" "))
    }
}

fn relative_to(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .display()
        .to_string()
}

pub const APP_SCREENSHOT_WIDTH: u32 = 1280;
pub const APP_SCREENSHOT_HEIGHT: u32 = 820;

#[cfg(not(target_os = "macos"))]
const APP_SCREENSHOT_FONT_CANDIDATES: &[&str] = &[
    "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf",
    "/usr/share/fonts/truetype/liberation2/LiberationSans-Regular.ttf",
    "/Library/Fonts/Arial.ttf",
];

pub fn render_app_screenshot_image(snapshot: &RepoViewSnapshot) -> Result<image::RgbaImage> {
    render_app_screenshot_image_at_scale(snapshot, 1.0)
}

pub fn render_app_screenshot_image_at_scale(
    snapshot: &RepoViewSnapshot,
    scale: f32,
) -> Result<image::RgbaImage> {
    render_app_screenshot_image_at_scale_with_options(
        snapshot,
        scale,
        AppScreenshotPaintOptions::full(),
    )
}

fn render_app_screenshot_image_at_scale_with_options(
    snapshot: &RepoViewSnapshot,
    scale: f32,
    options: AppScreenshotPaintOptions,
) -> Result<image::RgbaImage> {
    let scale = normalized_screenshot_scale(scale);
    let pixel_width = scaled_screenshot_dimension(APP_SCREENSHOT_WIDTH, scale);
    let pixel_height = scaled_screenshot_dimension(APP_SCREENSHOT_HEIGHT, scale);

    #[cfg(target_os = "macos")]
    {
        let mut painter =
            macos_screenshot::CoreTextScreenshotPainter::new(pixel_width, pixel_height)?;
        paint_repo_view_screenshot(
            snapshot,
            &mut ScaledScreenshotPainter::new(&mut painter, scale),
            options,
        );
        return painter.into_image();
    }

    #[cfg(not(target_os = "macos"))]
    {
        let mut painter = FontdueScreenshotPainter::new(pixel_width, pixel_height)?;
        paint_repo_view_screenshot(
            snapshot,
            &mut ScaledScreenshotPainter::new(&mut painter, scale),
            options,
        );
        return Ok(painter.into_image());
    }
}

#[derive(Clone, Copy)]
struct AppScreenshotPaintOptions {
    layout_toggle: bool,
    catalog_content: bool,
    terminal_content: bool,
    detail_content: bool,
}

impl AppScreenshotPaintOptions {
    fn full() -> Self {
        Self {
            layout_toggle: true,
            catalog_content: true,
            terminal_content: true,
            detail_content: true,
        }
    }

    fn native_backdrop() -> Self {
        Self {
            layout_toggle: false,
            catalog_content: false,
            terminal_content: false,
            detail_content: false,
        }
    }
}

fn normalized_screenshot_scale(scale: f32) -> f32 {
    if scale.is_finite() && scale >= 1.0 {
        scale.min(4.0)
    } else {
        1.0
    }
}

fn scaled_screenshot_dimension(dimension: u32, scale: f32) -> u32 {
    ((dimension as f32) * scale).round().max(1.0) as u32
}

#[cfg(not(target_os = "macos"))]
fn load_app_screenshot_font() -> Result<fontdue::Font> {
    let mut failures = Vec::new();
    for path in APP_SCREENSHOT_FONT_CANDIDATES {
        let bytes = match std::fs::read(path) {
            Ok(bytes) => bytes,
            Err(err) => {
                failures.push(format!("{path}: read failed: {err}"));
                continue;
            }
        };
        match fontdue::Font::from_bytes(bytes, fontdue::FontSettings::default()) {
            Ok(font) => return Ok(font),
            Err(err) => failures.push(format!("{path}: parse failed: {err}")),
        }
    }
    anyhow::bail!(
        "failed to load app screenshot font from system font candidates: {}",
        failures.join("; ")
    )
}

trait ScreenshotPainter {
    fn fill_rect(&mut self, x: i32, y: i32, width: i32, height: i32, color: image::Rgba<u8>);
    fn draw_text_line(&mut self, x: i32, y: i32, size: f32, color: image::Rgba<u8>, text: &str);
    fn measure_text_width(&mut self, text: &str, size: f32) -> f32;

    fn stroke_rect(&mut self, x: i32, y: i32, width: i32, height: i32, color: image::Rgba<u8>) {
        self.fill_rect(x, y, width, 1, color);
        self.fill_rect(x, y + height - 1, width, 1, color);
        self.fill_rect(x, y, 1, height, color);
        self.fill_rect(x + width - 1, y, 1, height, color);
    }
}

struct ScaledScreenshotPainter<'a> {
    inner: &'a mut dyn ScreenshotPainter,
    scale: f32,
}

impl<'a> ScaledScreenshotPainter<'a> {
    fn new(inner: &'a mut dyn ScreenshotPainter, scale: f32) -> Self {
        Self { inner, scale }
    }

    fn px(&self, value: i32) -> i32 {
        (value as f32 * self.scale).round() as i32
    }
}

impl ScreenshotPainter for ScaledScreenshotPainter<'_> {
    fn fill_rect(&mut self, x: i32, y: i32, width: i32, height: i32, color: image::Rgba<u8>) {
        self.inner.fill_rect(
            self.px(x),
            self.px(y),
            self.px(width).max(1),
            self.px(height).max(1),
            color,
        );
    }

    fn draw_text_line(&mut self, x: i32, y: i32, size: f32, color: image::Rgba<u8>, text: &str) {
        self.inner
            .draw_text_line(self.px(x), self.px(y), size * self.scale, color, text);
    }

    fn measure_text_width(&mut self, text: &str, size: f32) -> f32 {
        self.inner.measure_text_width(text, size * self.scale) / self.scale
    }
}

#[cfg(not(target_os = "macos"))]
struct FontdueScreenshotPainter {
    image: image::RgbaImage,
    font: fontdue::Font,
}

#[cfg(not(target_os = "macos"))]
impl FontdueScreenshotPainter {
    fn new(width: u32, height: u32) -> Result<Self> {
        Ok(Self {
            image: image::RgbaImage::from_pixel(width, height, rgba(244, 246, 248, 255)),
            font: load_app_screenshot_font()?,
        })
    }

    fn into_image(self) -> image::RgbaImage {
        self.image
    }
}

#[cfg(not(target_os = "macos"))]
impl ScreenshotPainter for FontdueScreenshotPainter {
    fn fill_rect(&mut self, x: i32, y: i32, width: i32, height: i32, color: image::Rgba<u8>) {
        fill_rect_pixels(&mut self.image, x, y, width, height, color);
    }

    fn draw_text_line(&mut self, x: i32, y: i32, size: f32, color: image::Rgba<u8>, text: &str) {
        draw_fontdue_text_line(&mut self.image, &self.font, x, y, size, color, text);
    }

    fn measure_text_width(&mut self, text: &str, size: f32) -> f32 {
        estimate_fontdue_text_width(text, size)
    }
}

#[cfg(target_os = "macos")]
mod macos_screenshot {
    use super::{rgba, ScreenshotPainter};
    use anyhow::{Context, Result};
    use std::ffi::c_void;
    use std::ptr;

    type CGFloat = f64;
    type CFIndex = isize;
    type CGContextRef = *mut c_void;
    type CGColorSpaceRef = *mut c_void;
    type CGColorRef = *const c_void;
    type CFStringRef = *const c_void;
    type CFDictionaryRef = *const c_void;
    type CFAttributedStringRef = *const c_void;
    type CTFontRef = *const c_void;
    type CTLineRef = *const c_void;
    type CFTypeRef = *const c_void;

    const K_CF_STRING_ENCODING_UTF8: u32 = 0x0800_0100;
    const K_CG_IMAGE_ALPHA_PREMULTIPLIED_LAST: u32 = 1;
    const K_CG_BITMAP_BYTE_ORDER_32_BIG: u32 = 4 << 12;
    const K_CT_FONT_UIFONT_SYSTEM: u32 = 2;

    #[repr(C)]
    #[derive(Clone, Copy)]
    struct CGPoint {
        x: CGFloat,
        y: CGFloat,
    }

    #[repr(C)]
    #[derive(Clone, Copy)]
    struct CGSize {
        width: CGFloat,
        height: CGFloat,
    }

    #[repr(C)]
    #[derive(Clone, Copy)]
    struct CGRect {
        origin: CGPoint,
        size: CGSize,
    }

    #[repr(C)]
    #[derive(Clone, Copy)]
    struct CGAffineTransform {
        a: CGFloat,
        b: CGFloat,
        c: CGFloat,
        d: CGFloat,
        tx: CGFloat,
        ty: CGFloat,
    }

    #[repr(C)]
    struct CFDictionaryKeyCallBacks {
        version: CFIndex,
        retain: *const c_void,
        release: *const c_void,
        copy_description: *const c_void,
        equal: *const c_void,
        hash: *const c_void,
    }

    #[repr(C)]
    struct CFDictionaryValueCallBacks {
        version: CFIndex,
        retain: *const c_void,
        release: *const c_void,
        copy_description: *const c_void,
        equal: *const c_void,
    }

    #[link(name = "CoreGraphics", kind = "framework")]
    extern "C" {
        fn CGColorSpaceCreateDeviceRGB() -> CGColorSpaceRef;
        fn CGColorSpaceRelease(space: CGColorSpaceRef);
        fn CGBitmapContextCreate(
            data: *mut c_void,
            width: usize,
            height: usize,
            bits_per_component: usize,
            bytes_per_row: usize,
            space: CGColorSpaceRef,
            bitmap_info: u32,
        ) -> CGContextRef;
        fn CGContextRelease(context: CGContextRef);
        fn CGContextFlush(context: CGContextRef);
        fn CGContextSetRGBFillColor(
            context: CGContextRef,
            red: CGFloat,
            green: CGFloat,
            blue: CGFloat,
            alpha: CGFloat,
        );
        fn CGContextFillRect(context: CGContextRef, rect: CGRect);
        fn CGContextSetShouldAntialias(context: CGContextRef, should_antialias: bool);
        fn CGContextSetAllowsAntialiasing(context: CGContextRef, allows_antialiasing: bool);
        fn CGContextSetAllowsFontSmoothing(context: CGContextRef, allows_font_smoothing: bool);
        fn CGContextSetShouldSmoothFonts(context: CGContextRef, should_smooth_fonts: bool);
        fn CGContextSetTextMatrix(context: CGContextRef, transform: CGAffineTransform);
        fn CGContextSetTextPosition(context: CGContextRef, x: CGFloat, y: CGFloat);
        fn CGColorCreateGenericRGB(
            red: CGFloat,
            green: CGFloat,
            blue: CGFloat,
            alpha: CGFloat,
        ) -> CGColorRef;
        fn CGColorRelease(color: CGColorRef);
    }

    #[link(name = "CoreFoundation", kind = "framework")]
    extern "C" {
        static kCFTypeDictionaryKeyCallBacks: CFDictionaryKeyCallBacks;
        static kCFTypeDictionaryValueCallBacks: CFDictionaryValueCallBacks;

        fn CFStringCreateWithBytes(
            allocator: *const c_void,
            bytes: *const u8,
            num_bytes: CFIndex,
            encoding: u32,
            is_external_representation: bool,
        ) -> CFStringRef;
        fn CFDictionaryCreate(
            allocator: *const c_void,
            keys: *const *const c_void,
            values: *const *const c_void,
            num_values: CFIndex,
            key_callbacks: *const CFDictionaryKeyCallBacks,
            value_callbacks: *const CFDictionaryValueCallBacks,
        ) -> CFDictionaryRef;
        fn CFAttributedStringCreate(
            allocator: *const c_void,
            string: CFStringRef,
            attributes: CFDictionaryRef,
        ) -> CFAttributedStringRef;
        fn CFRelease(object: CFTypeRef);
    }

    #[link(name = "CoreText", kind = "framework")]
    extern "C" {
        static kCTFontAttributeName: CFStringRef;
        static kCTForegroundColorAttributeName: CFStringRef;

        fn CTFontCreateUIFontForLanguage(
            ui_type: u32,
            size: CGFloat,
            language: CFStringRef,
        ) -> CTFontRef;
        fn CTLineCreateWithAttributedString(string: CFAttributedStringRef) -> CTLineRef;
        fn CTLineDraw(line: CTLineRef, context: CGContextRef);
        fn CTLineGetTypographicBounds(
            line: CTLineRef,
            ascent: *mut CGFloat,
            descent: *mut CGFloat,
            leading: *mut CGFloat,
        ) -> CGFloat;
    }

    pub(super) struct CoreTextScreenshotPainter {
        width: u32,
        height: u32,
        data: Vec<u8>,
        color_space: CGColorSpaceRef,
        context: CGContextRef,
    }

    impl CoreTextScreenshotPainter {
        pub(super) fn new(width: u32, height: u32) -> Result<Self> {
            let bytes_per_row = width as usize * 4;
            let mut data = vec![0; bytes_per_row * height as usize];
            let color_space = unsafe { CGColorSpaceCreateDeviceRGB() };
            if color_space.is_null() {
                anyhow::bail!("failed to create CoreGraphics device RGB color space");
            }
            let context = unsafe {
                CGBitmapContextCreate(
                    data.as_mut_ptr().cast(),
                    width as usize,
                    height as usize,
                    8,
                    bytes_per_row,
                    color_space,
                    K_CG_IMAGE_ALPHA_PREMULTIPLIED_LAST | K_CG_BITMAP_BYTE_ORDER_32_BIG,
                )
            };
            if context.is_null() {
                unsafe {
                    CGColorSpaceRelease(color_space);
                }
                anyhow::bail!("failed to create CoreGraphics bitmap context");
            }
            let mut painter = Self {
                width,
                height,
                data,
                color_space,
                context,
            };
            painter.configure_context();
            painter.fill_rect(0, 0, width as i32, height as i32, rgba(244, 246, 248, 255));
            Ok(painter)
        }

        pub(super) fn into_image(mut self) -> Result<image::RgbaImage> {
            self.flush();
            self.release_context();
            let data = std::mem::take(&mut self.data);
            image::RgbaImage::from_raw(self.width, self.height, data)
                .context("failed to construct PNG image from CoreGraphics bitmap")
        }

        fn configure_context(&mut self) {
            unsafe {
                CGContextSetAllowsAntialiasing(self.context, true);
                CGContextSetShouldAntialias(self.context, true);
                CGContextSetAllowsFontSmoothing(self.context, true);
                CGContextSetShouldSmoothFonts(self.context, true);
                CGContextSetTextMatrix(self.context, CGAffineTransform::identity());
            }
        }

        fn flush(&mut self) {
            if !self.context.is_null() {
                unsafe {
                    CGContextFlush(self.context);
                }
            }
        }

        fn release_context(&mut self) {
            unsafe {
                if !self.context.is_null() {
                    CGContextRelease(self.context);
                    self.context = ptr::null_mut();
                }
                if !self.color_space.is_null() {
                    CGColorSpaceRelease(self.color_space);
                    self.color_space = ptr::null_mut();
                }
            }
        }

        fn make_line(text: &str, size: f32, color: image::Rgba<u8>) -> Result<CTLineRef> {
            unsafe {
                let string = cf_string(text)?;
                let font = CTFontCreateUIFontForLanguage(
                    K_CT_FONT_UIFONT_SYSTEM,
                    size as CGFloat,
                    ptr::null(),
                );
                if font.is_null() {
                    CFRelease(string.cast());
                    anyhow::bail!("failed to create CoreText system UI font");
                }
                let [red, green, blue, alpha] = color.0;
                let foreground = CGColorCreateGenericRGB(
                    red as CGFloat / 255.0,
                    green as CGFloat / 255.0,
                    blue as CGFloat / 255.0,
                    alpha as CGFloat / 255.0,
                );
                if foreground.is_null() {
                    CFRelease(font.cast());
                    CFRelease(string.cast());
                    anyhow::bail!("failed to create CoreGraphics text color");
                }
                let keys = [
                    kCTFontAttributeName as *const c_void,
                    kCTForegroundColorAttributeName as *const c_void,
                ];
                let values = [font as *const c_void, foreground as *const c_void];
                let attributes = CFDictionaryCreate(
                    ptr::null(),
                    keys.as_ptr(),
                    values.as_ptr(),
                    2,
                    &kCFTypeDictionaryKeyCallBacks,
                    &kCFTypeDictionaryValueCallBacks,
                );
                if attributes.is_null() {
                    CGColorRelease(foreground);
                    CFRelease(font.cast());
                    CFRelease(string.cast());
                    anyhow::bail!("failed to create CoreText attribute dictionary");
                }
                let attributed = CFAttributedStringCreate(ptr::null(), string, attributes);
                if attributed.is_null() {
                    CFRelease(attributes.cast());
                    CGColorRelease(foreground);
                    CFRelease(font.cast());
                    CFRelease(string.cast());
                    anyhow::bail!("failed to create CoreText attributed string");
                }
                let line = CTLineCreateWithAttributedString(attributed);
                CFRelease(attributed.cast());
                CFRelease(attributes.cast());
                CGColorRelease(foreground);
                CFRelease(font.cast());
                CFRelease(string.cast());
                if line.is_null() {
                    anyhow::bail!("failed to create CoreText line");
                }
                Ok(line)
            }
        }

        fn top_left_rect(&self, x: i32, y: i32, width: i32, height: i32) -> CGRect {
            CGRect {
                origin: CGPoint {
                    x: x as CGFloat,
                    y: self.height as CGFloat - y as CGFloat - height as CGFloat,
                },
                size: CGSize {
                    width: width.max(0) as CGFloat,
                    height: height.max(0) as CGFloat,
                },
            }
        }

        fn text_baseline_y(&self, y: i32, size: f32) -> CGFloat {
            self.height as CGFloat - (y as CGFloat + size as CGFloat * 0.82)
        }
    }

    impl Drop for CoreTextScreenshotPainter {
        fn drop(&mut self) {
            self.release_context();
        }
    }

    impl ScreenshotPainter for CoreTextScreenshotPainter {
        fn fill_rect(&mut self, x: i32, y: i32, width: i32, height: i32, color: image::Rgba<u8>) {
            if width <= 0 || height <= 0 {
                return;
            }
            let [red, green, blue, alpha] = color.0;
            unsafe {
                CGContextSetRGBFillColor(
                    self.context,
                    red as CGFloat / 255.0,
                    green as CGFloat / 255.0,
                    blue as CGFloat / 255.0,
                    alpha as CGFloat / 255.0,
                );
                CGContextFillRect(self.context, self.top_left_rect(x, y, width, height));
            }
        }

        fn draw_text_line(
            &mut self,
            x: i32,
            y: i32,
            size: f32,
            color: image::Rgba<u8>,
            text: &str,
        ) {
            if text.is_empty() {
                return;
            }
            let line = Self::make_line(text, size, color)
                .expect("CoreText line creation failed after renderer initialization");
            let [red, green, blue, alpha] = color.0;
            unsafe {
                CGContextSetTextMatrix(self.context, CGAffineTransform::identity());
                CGContextSetRGBFillColor(
                    self.context,
                    red as CGFloat / 255.0,
                    green as CGFloat / 255.0,
                    blue as CGFloat / 255.0,
                    alpha as CGFloat / 255.0,
                );
                CGContextSetTextPosition(self.context, x as CGFloat, self.text_baseline_y(y, size));
                CTLineDraw(line, self.context);
                CFRelease(line.cast());
            }
        }

        fn measure_text_width(&mut self, text: &str, size: f32) -> f32 {
            if text.is_empty() {
                return 0.0;
            }
            let line = match Self::make_line(text, size, rgba(0, 0, 0, 255)) {
                Ok(line) => line,
                Err(_) => return text.chars().count() as f32 * size * 0.56,
            };
            let width = unsafe {
                let width = CTLineGetTypographicBounds(
                    line,
                    ptr::null_mut(),
                    ptr::null_mut(),
                    ptr::null_mut(),
                );
                CFRelease(line.cast());
                width
            };
            width as f32
        }
    }

    impl CGAffineTransform {
        fn identity() -> Self {
            Self {
                a: 1.0,
                b: 0.0,
                c: 0.0,
                d: 1.0,
                tx: 0.0,
                ty: 0.0,
            }
        }
    }

    unsafe fn cf_string(value: &str) -> Result<CFStringRef> {
        let string = CFStringCreateWithBytes(
            ptr::null(),
            value.as_ptr(),
            value.len() as CFIndex,
            K_CF_STRING_ENCODING_UTF8,
            false,
        );
        if string.is_null() {
            anyhow::bail!("failed to create CoreFoundation string");
        }
        Ok(string)
    }
}

pub fn render_app_screenshot_png(snapshot: &RepoViewSnapshot) -> Result<Vec<u8>> {
    render_app_screenshot_png_at_scale(snapshot, 1.0)
}

pub fn render_app_screenshot_png_at_scale(
    snapshot: &RepoViewSnapshot,
    scale: f32,
) -> Result<Vec<u8>> {
    let image = render_app_screenshot_image_at_scale(snapshot, scale)?;
    let mut cursor = std::io::Cursor::new(Vec::new());
    image::DynamicImage::ImageRgba8(image)
        .write_to(&mut cursor, image::ImageFormat::Png)
        .context("failed to encode app screenshot PNG")?;
    Ok(cursor.into_inner())
}

pub fn render_native_app_backdrop_png_at_scale(
    snapshot: &RepoViewSnapshot,
    scale: f32,
) -> Result<Vec<u8>> {
    let image = render_app_screenshot_image_at_scale_with_options(
        snapshot,
        scale,
        AppScreenshotPaintOptions::native_backdrop(),
    )?;
    let mut cursor = std::io::Cursor::new(Vec::new());
    image::DynamicImage::ImageRgba8(image)
        .write_to(&mut cursor, image::ImageFormat::Png)
        .context("failed to encode native app backdrop PNG")?;
    Ok(cursor.into_inner())
}

pub fn render_app_screenshot(snapshot: &RepoViewSnapshot, path: &Path) -> Result<()> {
    if let Some(parent) = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
    {
        std::fs::create_dir_all(parent).with_context(|| {
            format!("failed to create screenshot directory {}", parent.display())
        })?;
    }

    let image = render_app_screenshot_image(snapshot)?;
    image
        .save(path)
        .with_context(|| format!("failed to write app screenshot {}", path.display()))?;
    Ok(())
}

pub fn build_desktop_app_bundle(
    root: &Path,
    app_path: &Path,
    focus: Option<&str>,
    layout: ViewLayout,
) -> Result<()> {
    #[cfg(target_os = "macos")]
    {
        return build_macos_app_bundle(root, app_path, focus, layout);
    }

    #[cfg(not(target_os = "macos"))]
    {
        let _ = (root, app_path, focus, layout);
        anyhow::bail!("aw view --app currently builds macOS .app bundles only");
    }
}

#[cfg(target_os = "macos")]
fn build_macos_app_bundle(
    root: &Path,
    app_path: &Path,
    focus: Option<&str>,
    layout: ViewLayout,
) -> Result<()> {
    use std::os::unix::fs::PermissionsExt;

    if app_path.exists() && !app_path.is_dir() {
        anyhow::bail!(
            "app path exists but is not a directory: {}",
            app_path.display()
        );
    }
    let contents_dir = app_path.join("Contents");
    let macos_dir = contents_dir.join("MacOS");
    std::fs::create_dir_all(&macos_dir)
        .with_context(|| format!("failed to create app bundle {}", macos_dir.display()))?;
    std::fs::write(
        contents_dir.join("Info.plist"),
        macos_app_info_plist(app_path),
    )
    .with_context(|| {
        format!(
            "failed to write {}",
            contents_dir.join("Info.plist").display()
        )
    })?;
    std::fs::write(contents_dir.join("PkgInfo"), "APPL????\n")
        .with_context(|| format!("failed to write {}", contents_dir.join("PkgInfo").display()))?;

    let aw_binary = root.join("target/debug/aw");
    let mut script = format!(
        "#!/bin/sh\ncd {} || exit 1\nexec {} view",
        shell_quote(&root.display().to_string()),
        shell_quote(&aw_binary.display().to_string())
    );
    if let Some(focus) = focus {
        script.push_str(" --focus ");
        script.push_str(&shell_quote(focus));
    }
    script.push_str(" --layout ");
    script.push_str(&layout.to_string());
    script.push('\n');
    let launcher = macos_dir.join("aw-view");
    std::fs::write(&launcher, script)
        .with_context(|| format!("failed to write app launcher {}", launcher.display()))?;
    let mut permissions = std::fs::metadata(&launcher)
        .with_context(|| format!("failed to inspect app launcher {}", launcher.display()))?
        .permissions();
    permissions.set_mode(0o755);
    std::fs::set_permissions(&launcher, permissions)
        .with_context(|| format!("failed to chmod app launcher {}", launcher.display()))?;
    Ok(())
}

#[cfg(target_os = "macos")]
fn macos_app_info_plist(app_path: &Path) -> String {
    let display_name = app_path
        .file_stem()
        .and_then(|name| name.to_str())
        .filter(|name| !name.is_empty())
        .unwrap_or("AW Repo View");
    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>CFBundleDevelopmentRegion</key>
  <string>en</string>
  <key>CFBundleExecutable</key>
  <string>aw-view</string>
  <key>CFBundleIdentifier</key>
  <string>dev.cclab.aw.repo-view</string>
  <key>CFBundleName</key>
  <string>{display_name}</string>
  <key>CFBundlePackageType</key>
  <string>APPL</string>
  <key>CFBundleShortVersionString</key>
  <string>0.1.0</string>
  <key>CFBundleVersion</key>
  <string>1</string>
  <key>LSMinimumSystemVersion</key>
  <string>11.0</string>
  <key>NSHighResolutionCapable</key>
  <true/>
</dict>
</plist>
"#
    )
}

#[cfg(target_os = "macos")]
fn shell_quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\"'\"'"))
}

fn paint_repo_view_screenshot(
    snapshot: &RepoViewSnapshot,
    painter: &mut dyn ScreenshotPainter,
    options: AppScreenshotPaintOptions,
) {
    let selected = selected_item(snapshot);
    painter.fill_rect(0, 0, APP_SCREENSHOT_WIDTH as i32, 58, rgba(31, 42, 55, 255));
    painter.fill_rect(
        0,
        58,
        APP_SCREENSHOT_WIDTH as i32,
        1,
        rgba(96, 165, 250, 255),
    );
    painter.draw_text_line(28, 18, 22.0, rgba(248, 250, 252, 255), "AW Repo View");
    painter.draw_text_line(
        210,
        22,
        14.0,
        rgba(203, 213, 225, 255),
        &format!(
            "{} - {} repos / {} projects / {} libs",
            snapshot.repo.name,
            snapshot.repo_catalog.len(),
            snapshot.repo.project_count,
            snapshot.repo.library_count
        ),
    );
    if options.layout_toggle {
        paint_layout_toggle_button(snapshot, painter);
    }

    let catalog_x = 24;
    let catalog_y = 82;
    let catalog_w = 250;
    let catalog_h = 708;
    paint_catalog(
        snapshot,
        painter,
        catalog_x,
        catalog_y,
        catalog_w,
        catalog_h,
        options.catalog_content,
    );

    match snapshot.layout {
        ViewLayout::LeftRight => {
            let terminal_x = 290;
            let terminal_y = 82;
            let terminal_w = 376;
            let terminal_h = 708;
            let detail_x = 682;
            let detail_y = 82;
            let detail_w = 574;
            let detail_h = 708;
            paint_terminal(
                snapshot,
                painter,
                terminal_x,
                terminal_y,
                terminal_w,
                terminal_h,
                options.terminal_content,
            );
            paint_detail_panel(
                snapshot,
                selected,
                painter,
                detail_x,
                detail_y,
                detail_w,
                detail_h,
                DetailPaintDensity::standard(),
                options.detail_content,
            );
        }
        ViewLayout::TopBottom => {
            let right_x = 290;
            let right_y = 82;
            let right_w = 966;
            let terminal_h = 192;
            let gap = 16;
            let detail_y = right_y + terminal_h + gap;
            let detail_h = 708 - terminal_h - gap;
            paint_terminal(
                snapshot,
                painter,
                right_x,
                right_y,
                right_w,
                terminal_h,
                options.terminal_content,
            );
            paint_detail_panel(
                snapshot,
                selected,
                painter,
                right_x,
                detail_y,
                right_w,
                detail_h,
                DetailPaintDensity::compact(),
                options.detail_content,
            );
        }
    }
}

fn paint_layout_toggle_button(snapshot: &RepoViewSnapshot, painter: &mut dyn ScreenshotPainter) {
    let width = 190;
    let height = 30;
    let x = APP_SCREENSHOT_WIDTH as i32 - width - 28;
    let y = 14;
    painter.fill_rect(x, y, width, height, rgba(248, 250, 252, 255));
    painter.stroke_rect(x, y, width, height, rgba(148, 163, 184, 255));
    painter.draw_text_line(
        x + 14,
        y + 9,
        12.0,
        rgba(15, 23, 42, 255),
        layout_toggle_button_label(snapshot.layout),
    );
}

#[derive(Clone, Copy)]
struct DetailPaintDensity {
    brief_lines: usize,
    capability_rows: usize,
    list_panel_height: i32,
}

impl DetailPaintDensity {
    fn standard() -> Self {
        Self {
            brief_lines: 4,
            capability_rows: 9,
            list_panel_height: 158,
        }
    }

    fn compact() -> Self {
        Self {
            brief_lines: 2,
            capability_rows: 4,
            list_panel_height: 92,
        }
    }
}

fn paint_detail_panel(
    snapshot: &RepoViewSnapshot,
    selected: Option<&RepoViewItemSnapshot>,
    painter: &mut dyn ScreenshotPainter,
    x: i32,
    y: i32,
    width: i32,
    height: i32,
    density: DetailPaintDensity,
    paint_content: bool,
) {
    painter.fill_rect(x, y, width, height, rgba(255, 255, 255, 255));
    painter.stroke_rect(x, y, width, height, rgba(203, 213, 225, 255));
    painter.draw_text_line(x + 24, y + 18, 13.0, rgba(100, 116, 139, 255), "Caps / EC");
    paint_project_selector(snapshot, selected, painter, x + 220, y + 12, width - 244);
    if !paint_content {
        return;
    }
    paint_detail(
        snapshot,
        selected,
        painter,
        x + 24,
        y + 84,
        width - 48,
        density,
    );
}

fn paint_catalog(
    snapshot: &RepoViewSnapshot,
    painter: &mut dyn ScreenshotPainter,
    x: i32,
    y: i32,
    width: i32,
    height: i32,
    paint_content: bool,
) {
    painter.fill_rect(x, y, width, height, rgba(255, 255, 255, 255));
    painter.stroke_rect(x, y, width, height, rgba(203, 213, 225, 255));
    painter.fill_rect(x, y, width, 42, rgba(248, 250, 252, 255));
    painter.draw_text_line(x + 14, y + 13, 13.5, rgba(15, 23, 42, 255), "Repos");
    painter.draw_text_line(
        x + width - 72,
        y + 15,
        10.5,
        rgba(100, 116, 139, 255),
        &format!("{} total", snapshot.repo_catalog.len()),
    );
    if !paint_content {
        return;
    }

    let selected = snapshot.selected_repo.as_deref().unwrap_or_default();
    let mut cursor_y = y + 56;
    let row_h = 22;
    for item in snapshot.repo_catalog.iter() {
        if cursor_y > y + height - 22 {
            break;
        }
        let is_selected = item.path == selected;
        if is_selected {
            painter.fill_rect(
                x + 8,
                cursor_y - 4,
                width - 16,
                row_h,
                rgba(219, 234, 254, 255),
            );
            painter.fill_rect(x + 8, cursor_y - 4, 3, row_h, rgba(37, 99, 235, 255));
        }
        let label = truncate_for_width(
            painter,
            &format!(
                "{} {}",
                if item.current { "cur" } else { "repo" },
                item.name
            ),
            width - 34,
            10.5,
        );
        painter.draw_text_line(
            x + 18,
            cursor_y,
            10.5,
            if is_selected {
                rgba(30, 64, 175, 255)
            } else {
                rgba(51, 65, 85, 255)
            },
            &label,
        );
        cursor_y += row_h;
    }
}

fn paint_project_selector(
    snapshot: &RepoViewSnapshot,
    selected: Option<&RepoViewItemSnapshot>,
    painter: &mut dyn ScreenshotPainter,
    x: i32,
    y: i32,
    width: i32,
) {
    let label = selected
        .map(|item| format!("{} [{}]", item.project.name, item.project.kind))
        .unwrap_or_else(|| "No project selected".to_string());
    painter.draw_text_line(x, y - 2, 10.5, rgba(100, 116, 139, 255), "Project / lib");
    painter.fill_rect(x, y + 14, width, 28, rgba(248, 250, 252, 255));
    painter.stroke_rect(x, y + 14, width, 28, rgba(148, 163, 184, 255));
    let text = truncate_for_width(painter, &label, width - 34, 12.0);
    painter.draw_text_line(x + 10, y + 23, 12.0, rgba(15, 23, 42, 255), &text);
    painter.draw_text_line(x + width - 20, y + 23, 12.0, rgba(71, 85, 105, 255), "v");
    painter.draw_text_line(
        x,
        y + 55,
        10.5,
        rgba(100, 116, 139, 255),
        &format!("{} projects/libs", snapshot.catalog.len()),
    );
}

fn paint_terminal(
    snapshot: &RepoViewSnapshot,
    painter: &mut dyn ScreenshotPainter,
    x: i32,
    y: i32,
    width: i32,
    height: i32,
    paint_content: bool,
) {
    painter.fill_rect(x, y, width, height, rgba(15, 23, 42, 255));
    painter.stroke_rect(x, y, width, height, rgba(51, 65, 85, 255));
    painter.fill_rect(x, y, width, 42, rgba(30, 41, 59, 255));
    painter.draw_text_line(
        x + 16,
        y + 13,
        14.0,
        rgba(248, 250, 252, 255),
        &snapshot.terminal.title,
    );
    painter.draw_text_line(
        x + width - 126,
        y + 15,
        11.0,
        rgba(125, 211, 252, 255),
        "agent + repo",
    );
    if !paint_content {
        return;
    }

    let mut cursor_y = y + 58;
    for line in &snapshot.terminal.lines {
        if cursor_y > y + height - 24 {
            break;
        }
        if line.is_empty() {
            cursor_y += 12;
            continue;
        }
        let (color, size) = if line.starts_with('$') {
            (rgba(134, 239, 172, 255), 11.5)
        } else if line.starts_with("##") {
            (rgba(251, 191, 36, 255), 11.0)
        } else if line.starts_with(" M ") || line.starts_with("?? ") || line.starts_with(" A ") {
            (rgba(253, 186, 116, 255), 10.5)
        } else {
            (rgba(226, 232, 240, 255), 10.5)
        };
        let text = truncate_for_width(painter, line, width - 32, size);
        painter.draw_text_line(x + 16, cursor_y, size, color, &text);
        cursor_y += 18;
    }
}

fn paint_detail(
    snapshot: &RepoViewSnapshot,
    item: Option<&RepoViewItemSnapshot>,
    painter: &mut dyn ScreenshotPainter,
    x: i32,
    y: i32,
    width: i32,
    density: DetailPaintDensity,
) {
    let Some(item) = item else {
        painter.draw_text_line(x, y, 18.0, rgba(51, 65, 85, 255), "No repo item selected");
        return;
    };
    let title = truncate_for_width(painter, &item.readme.title, width, 23.0);
    painter.draw_text_line(x, y, 23.0, rgba(15, 23, 42, 255), &title);
    let path_label = truncate_for_width(
        painter,
        &format!("{} - {}", item.project.kind, item.project.path),
        width,
        13.0,
    );
    painter.draw_text_line(x, y + 34, 13.0, rgba(100, 116, 139, 255), &path_label);

    let mut next_y = draw_wrapped_text(
        painter,
        x,
        y + 66,
        width,
        16,
        13.0,
        rgba(51, 65, 85, 255),
        &item.readme.brief,
        density.brief_lines,
    );
    next_y += 18;

    let card_w = (width - 24) / 3;
    paint_stat_card(
        painter,
        x,
        next_y,
        card_w,
        "Capabilities",
        item.capabilities.count,
        rgba(37, 99, 235, 255),
    );
    paint_stat_card(
        painter,
        x + card_w + 12,
        next_y,
        card_w,
        "EC cases",
        item.ec.case_count,
        rgba(5, 150, 105, 255),
    );
    paint_stat_card(
        painter,
        x + (card_w + 12) * 2,
        next_y,
        card_w,
        "TD markdown",
        item.td.markdown_file_count,
        rgba(124, 58, 237, 255),
    );
    next_y += 94;

    painter.draw_text_line(x, next_y, 16.0, rgba(15, 23, 42, 255), "Capabilities");
    next_y += 26;
    for cap in item.capabilities.items.iter().take(density.capability_rows) {
        painter.fill_rect(x, next_y - 4, width, 23, rgba(248, 250, 252, 255));
        let title = truncate_for_width(painter, &cap.title, width - 190, 11.5);
        painter.draw_text_line(x + 10, next_y, 11.5, rgba(30, 41, 59, 255), &title);
        painter.draw_text_line(
            x + width - 170,
            next_y,
            11.5,
            rgba(22, 101, 52, 255),
            &cap.status,
        );
        painter.draw_text_line(
            x + width - 82,
            next_y,
            11.5,
            rgba(71, 85, 105, 255),
            &format!("EC {}", cap.ec_case_count),
        );
        next_y += 25;
    }

    next_y += 8;
    let half_w = (width - 14) / 2;
    paint_list_panel(
        painter,
        x,
        next_y,
        half_w,
        density.list_panel_height,
        "External contracts",
        item.ec
            .cases
            .iter()
            .take(5)
            .map(|case| format!("{} - {}", case.id, case.command))
            .collect(),
    );
    paint_list_panel(
        painter,
        x + half_w + 14,
        next_y,
        half_w,
        density.list_panel_height,
        "Tech designs",
        vec![
            format!("{} markdown files", item.td.markdown_file_count),
            format!("{} capability refs", item.td.capability_ref_count),
            item.td.root.clone(),
        ],
    );
    if !snapshot.warnings.is_empty() || !item.warnings.is_empty() {
        painter.draw_text_line(
            x,
            next_y + density.list_panel_height + 26,
            11.0,
            rgba(180, 83, 9, 255),
            "Warnings present; inspect --snapshot for full detail.",
        );
    }
}

fn paint_stat_card(
    painter: &mut dyn ScreenshotPainter,
    x: i32,
    y: i32,
    width: i32,
    label: &str,
    value: usize,
    accent: image::Rgba<u8>,
) {
    painter.fill_rect(x, y, width, 72, rgba(248, 250, 252, 255));
    painter.stroke_rect(x, y, width, 72, rgba(226, 232, 240, 255));
    painter.fill_rect(x, y, 5, 72, accent);
    painter.draw_text_line(x + 18, y + 13, 12.0, rgba(100, 116, 139, 255), label);
    painter.draw_text_line(
        x + 18,
        y + 34,
        23.0,
        rgba(15, 23, 42, 255),
        &value.to_string(),
    );
}

fn paint_list_panel(
    painter: &mut dyn ScreenshotPainter,
    x: i32,
    y: i32,
    width: i32,
    height: i32,
    title: &str,
    rows: Vec<String>,
) {
    painter.fill_rect(x, y, width, height, rgba(248, 250, 252, 255));
    painter.stroke_rect(x, y, width, height, rgba(226, 232, 240, 255));
    painter.draw_text_line(x + 12, y + 14, 13.0, rgba(15, 23, 42, 255), title);
    let mut row_y = y + 42;
    if rows.is_empty() {
        painter.draw_text_line(x + 12, row_y, 11.0, rgba(100, 116, 139, 255), "none");
        return;
    }
    for row in rows {
        let row = truncate_for_width(painter, &row, width - 24, 10.5);
        painter.draw_text_line(x + 12, row_y, 10.5, rgba(51, 65, 85, 255), &row);
        row_y += 22;
        if row_y > y + height - 18 {
            break;
        }
    }
}

fn draw_wrapped_text(
    painter: &mut dyn ScreenshotPainter,
    x: i32,
    y: i32,
    width: i32,
    line_height: i32,
    size: f32,
    color: image::Rgba<u8>,
    text: &str,
    max_lines: usize,
) -> i32 {
    let mut lines = Vec::new();
    let mut current = String::new();
    for word in text.split_whitespace() {
        let candidate = if current.is_empty() {
            word.to_string()
        } else {
            format!("{current} {word}")
        };
        if painter.measure_text_width(&candidate, size) <= width as f32 {
            current = candidate;
        } else {
            if !current.is_empty() {
                lines.push(current);
            }
            current = word.to_string();
        }
        if lines.len() >= max_lines {
            break;
        }
    }
    if !current.is_empty() && lines.len() < max_lines {
        lines.push(current);
    }
    let mut cursor_y = y;
    for mut line in lines {
        if cursor_y == y + line_height * (max_lines.saturating_sub(1) as i32)
            && painter.measure_text_width(&line, size) > width as f32
        {
            line = truncate_for_width(painter, &line, width, size);
        }
        painter.draw_text_line(x, cursor_y, size, color, &line);
        cursor_y += line_height;
    }
    cursor_y
}

fn truncate_for_width(
    painter: &mut dyn ScreenshotPainter,
    text: &str,
    width: i32,
    size: f32,
) -> String {
    if painter.measure_text_width(text, size) <= width as f32 {
        return text.to_string();
    }
    let mut out = String::new();
    for ch in text.chars() {
        let candidate = format!("{out}{ch}...");
        if painter.measure_text_width(&candidate, size) > width as f32 {
            break;
        }
        out.push(ch);
    }
    out.push_str("...");
    out
}

#[cfg(not(target_os = "macos"))]
fn estimate_fontdue_text_width(text: &str, size: f32) -> f32 {
    text.chars()
        .map(|ch| {
            if ch.is_whitespace() {
                size * 0.32
            } else {
                size * 0.56
            }
        })
        .sum()
}

#[cfg(not(target_os = "macos"))]
fn draw_fontdue_text_line(
    image: &mut image::RgbaImage,
    font: &fontdue::Font,
    x: i32,
    y: i32,
    size: f32,
    color: image::Rgba<u8>,
    text: &str,
) {
    let mut pen_x = x as f32;
    let baseline_y = y as f32 + size * 0.82;
    for ch in text.chars() {
        if ch == '\n' {
            break;
        }
        if ch == '\t' {
            pen_x += size;
            continue;
        }
        let (metrics, bitmap) = font.rasterize(ch, size);
        let glyph_x = pen_x + metrics.xmin as f32;
        let glyph_y = baseline_y - metrics.ymin as f32 - metrics.height as f32;
        for row in 0..metrics.height {
            for col in 0..metrics.width {
                let alpha = bitmap[row * metrics.width + col];
                if alpha == 0 {
                    continue;
                }
                blend_pixel(
                    image,
                    glyph_x.round() as i32 + col as i32,
                    glyph_y.round() as i32 + row as i32,
                    color,
                    alpha,
                );
            }
        }
        pen_x += metrics.advance_width.max(size * 0.33);
    }
}

#[cfg(not(target_os = "macos"))]
fn fill_rect_pixels(
    image: &mut image::RgbaImage,
    x: i32,
    y: i32,
    width: i32,
    height: i32,
    color: image::Rgba<u8>,
) {
    let x0 = x.max(0) as u32;
    let y0 = y.max(0) as u32;
    let x1 = (x + width).max(0).min(image.width() as i32) as u32;
    let y1 = (y + height).max(0).min(image.height() as i32) as u32;
    for yy in y0..y1 {
        for xx in x0..x1 {
            image.put_pixel(xx, yy, color);
        }
    }
}

#[cfg(not(target_os = "macos"))]
fn blend_pixel(image: &mut image::RgbaImage, x: i32, y: i32, color: image::Rgba<u8>, coverage: u8) {
    if x < 0 || y < 0 || x >= image.width() as i32 || y >= image.height() as i32 {
        return;
    }
    let pixel = image.get_pixel_mut(x as u32, y as u32);
    let alpha = (coverage as f32 / 255.0) * (color.0[3] as f32 / 255.0);
    let inv = 1.0 - alpha;
    for channel in 0..3 {
        pixel.0[channel] = ((color.0[channel] as f32 * alpha) + (pixel.0[channel] as f32 * inv))
            .round()
            .clamp(0.0, 255.0) as u8;
    }
    pixel.0[3] = 255;
}

fn rgba(r: u8, g: u8, b: u8, a: u8) -> image::Rgba<u8> {
    image::Rgba([r, g, b, a])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn view_repo_snapshot_includes_catalog_focus_detail_and_surface() {
        let tmp = tempfile::TempDir::new().unwrap();
        let root = tmp.path();
        std::fs::write(
            root.join("aw.toml"),
            r#"
version = "1"

[[projects]]
name = "demo"
aliases = ["d"]
path = "projects/demo"
td_path = "projects/demo/tech-design"
cap_path = "projects/demo/CAPABILITIES.md"
label = "project:demo"
"#,
        )
        .unwrap();
        let project_dir = root.join("projects/demo");
        std::fs::create_dir_all(project_dir.join("tech-design/specs")).unwrap();
        std::fs::write(
            project_dir.join("README.md"),
            r#"# Demo

## Brief

Demo project for the visual reader.
"#,
        )
        .unwrap();
        std::fs::write(
            project_dir.join("CAPABILITIES.md"),
            r#"# Demo Capabilities

## Capabilities

### Capability Index

| Capability | Root WI | Impl | Verification | Maturity | Production | Notes |
|---|---:|---|---|---|---|---|
| Demo Reader | - | implemented | verified | smoke | ready | demo |

### Demo Reader

ID: demo-reader
Type: DeveloperTool
Surfaces:
- CLI: `demo view` - demo reader.
EC Dimensions:
- behavior: `demo view --snapshot` - snapshot contract.
Root WI: -
Status: auditing
Required Verification: smoke
Promise:
Demo exposes a project reader.
Gate Inventory:
- `demo view --snapshot`

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| Demo snapshot | change | - | implemented | verified | smoke | `demo view --snapshot` |
"#,
        )
        .unwrap();
        std::fs::write(
            project_dir.join("aw.toml"),
            r#"[project]
name = "demo"

[aw.ec.generated]
version = 1
project = "demo"
generated_from_td_digest = "sha256:test"

[[aw.ec.generated.cases]]
id = "demo-view-snapshot"
capability_id = "demo-reader"
claim_id = "demo-snapshot"
contract_id = "demo-view-snapshot"
category = "behavior"
td_ref = "projects/demo/tech-design/specs/demo-view.md#demo-view-snapshot"
test_path = "projects/demo/tests/behavior_demo_view_snapshot.rs"
command = "demo view --snapshot"
required_for_production = true
assertions = ["snapshot includes demo"]
"#,
        )
        .unwrap();
        std::fs::write(
            project_dir.join("tech-design/specs/demo-view.md"),
            "# Demo View\n",
        )
        .unwrap();

        let repo_registry_path = root.join(".aw-user/repos.toml");
        let snapshot = build_repo_view_snapshot_with_repo_registry_path(
            root,
            Some("d"),
            ViewLayout::LeftRight,
            Some(repo_registry_path.clone()),
        )
        .unwrap();
        assert_eq!(
            snapshot.repo.name,
            root.file_name().unwrap().to_string_lossy()
        );
        assert_eq!(snapshot.repo_catalog.len(), 1);
        let canonical_root = canonical_repo_path(root).display().to_string();
        assert_eq!(snapshot.repo_catalog[0].path, canonical_root);
        assert_eq!(
            snapshot.selected_repo.as_deref(),
            Some(canonical_root.as_str())
        );
        assert!(repo_registry_path.exists());
        assert_eq!(snapshot.selected.as_deref(), Some("demo"));
        assert_eq!(snapshot.catalog.len(), 1);
        let item = snapshot
            .items
            .iter()
            .find(|item| item.project.name == "demo")
            .unwrap();
        assert_eq!(item.project.kind, "project");
        assert_eq!(item.project.cap_path, "projects/demo/CAPABILITIES.md");
        assert_eq!(item.readme.path, "projects/demo/README.md");
        assert_eq!(item.readme.title, "Demo");
        assert_eq!(item.capabilities.count, 1);
        assert_eq!(item.capabilities.items[0].ec_case_count, 1);
        assert_eq!(item.ec.case_count, 1);
        assert_eq!(item.td.markdown_file_count, 1);
        assert!(snapshot
            .surface
            .find_by_semantic_id("repo-catalog")
            .is_some());
        assert!(snapshot
            .surface
            .find_by_semantic_id("repo-project-selector")
            .is_some());
        let toggle = snapshot
            .surface
            .find_by_semantic_id("repo-layout-toggle")
            .expect("semantic layout toggle button");
        assert_eq!(toggle.role.as_deref(), Some("button"));
        assert!(toggle.props.has_on_click);
        assert!(snapshot
            .surface
            .text_content()
            .contains("Toggle: top-bottom"));
        assert!(snapshot
            .surface
            .find_by_semantic_id("repo-readme-detail")
            .is_some());
        assert!(snapshot
            .surface
            .find_by_semantic_id("repo-capability-table")
            .is_some());
        assert!(snapshot
            .surface
            .find_by_semantic_id("repo-ec-detail")
            .is_some());
        assert!(snapshot
            .surface
            .find_by_semantic_id("repo-td-detail")
            .is_some());

        let screenshot_path = root.join("aw-view.png");
        render_app_screenshot(&snapshot, &screenshot_path).unwrap();
        let screenshot = std::fs::read(&screenshot_path).unwrap();
        assert!(screenshot.len() > 1024);
        assert_eq!(&screenshot[..8], b"\x89PNG\r\n\x1a\n");

        let retina_image = render_app_screenshot_image_at_scale(&snapshot, 2.0).unwrap();
        assert_eq!(retina_image.width(), APP_SCREENSHOT_WIDTH * 2);
        assert_eq!(retina_image.height(), APP_SCREENSHOT_HEIGHT * 2);
    }
}
// HANDWRITE-END
