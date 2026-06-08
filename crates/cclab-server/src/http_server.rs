//! Unified HTTP Server - Dashboard + Plan Viewer
//!
//! This server hosts:
//! - Dashboard at `/` (project listing)
//! - Plan Viewer UI at `/view/*` (change review)
//! - Health check at `/health`
//!
//! MCP tools were removed — Claude Code invokes SDD tool logic via the CLI
//! directly. See `projects/agentic-workflow/` for the CLI that replaces the former
//! MCP tool surface.

use crate::lens_pool::LensHandlerPool;
use crate::registry::Registry;
use crate::Result;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{Html, IntoResponse, Response},
    routing::get,
    Json, Router,
};
use pulldown_cmark::{html, Options, Parser};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::RwLock;
use tower_http::cors::CorsLayer;

// =============================================================================
// Data Models
// =============================================================================

/// Dashboard state showing all registered projects and their changes
#[derive(Debug, Clone, Serialize)]
pub struct DashboardState {
    pub server_info: ServerInfoDto,
    pub projects: Vec<ProjectInfoDto>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ServerInfoDto {
    pub port: u16,
    pub pid: u32,
    pub started_at: String,
    pub uptime: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ProjectInfoDto {
    pub name: String,
    pub path: String,
    pub changes: Vec<ChangeInfoDto>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ChangeInfoDto {
    pub id: String,
    pub status: String,
}

// =============================================================================
// Application State
// =============================================================================

/// Unified application state shared across handlers
#[derive(Clone)]
pub struct UnifiedAppState {
    pub registry: Arc<RwLock<Registry>>,
    #[allow(dead_code)] // retained for LSP integration
    pub lens_pool: Arc<LensHandlerPool>,
}

impl UnifiedAppState {
    pub fn new(registry: Registry, lens_pool: LensHandlerPool) -> Self {
        Self {
            registry: Arc::new(RwLock::new(registry)),
            lens_pool: Arc::new(lens_pool),
        }
    }
}

// =============================================================================
// Server Startup
// =============================================================================

/// Start unified HTTP server
pub async fn start_server(port: u16, registry: Registry) -> Result<()> {
    let http_addr = SocketAddr::from(([127, 0, 0, 1], port));

    let lens_pool = LensHandlerPool::new();

    // Collect project paths for background initialization
    let project_paths: Vec<_> = registry
        .list_projects()
        .into_iter()
        .map(|(_, info)| info.path.clone())
        .collect();

    // Start background initialization for all registered projects (R2, R3)
    if !project_paths.is_empty() {
        eprintln!(
            "Starting background Lens initialization for {} projects...",
            project_paths.len()
        );
        lens_pool.initialize_projects_background(project_paths);
    }

    let state = UnifiedAppState::new(registry.clone(), lens_pool);

    // Build HTTP router
    let app = Router::new()
        // Dashboard at root
        .route("/", get(handle_dashboard))
        .route("/api/dashboard", get(api_dashboard))
        // Project root view (list changes)
        .route("/view/{project}", get(handle_project_root))
        .route("/view/{project}/", get(handle_project_root))
        .route("/view/{project}/api/changes", get(api_project_changes))
        // File viewers (must come before change viewer to avoid conflicts)
        .route("/view/{project}/file/spec/{spec}", get(handle_spec_viewer))
        .route(
            "/view/{project}/file/knowledge/{*path}",
            get(handle_knowledge_viewer),
        )
        .route(
            "/view/{project}/file/project.md",
            get(handle_project_md_viewer),
        )
        // Change viewer
        .route("/view/{project}/{change}", get(handle_change_viewer))
        .route("/view/{project}/{change}/", get(handle_change_viewer))
        // Project cclab Viewer
        .route("/{project}/cclab", get(handle_project_sdd))
        .route("/api/{project}/cclab/tree", get(api_sdd_tree))
        // Health check
        .route("/health", get(health_check))
        .layer(CorsLayer::permissive())
        .with_state(state);

    let listener = tokio::net::TcpListener::bind(&http_addr).await?;
    println!("Server listening on http://{}", http_addr);
    println!("  Dashboard: http://{}/", http_addr);

    axum::serve(listener, app).await?;
    Ok(())
}

/// Health check endpoint
async fn health_check() -> &'static str {
    "OK"
}

// =============================================================================
// Dashboard Handlers
// =============================================================================

/// Serve dashboard HTML page
async fn handle_dashboard() -> Html<&'static str> {
    Html(include_str!("dashboard.html"))
}

/// Dashboard API - returns project and change listing
async fn api_dashboard(State(state): State<UnifiedAppState>) -> Response {
    // Reload registry to get latest state
    let registry = match Registry::load() {
        Ok(reg) => reg,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": format!("Failed to load registry: {}", e) })),
            )
                .into_response();
        }
    };

    // Update shared state
    {
        let mut reg_lock = state.registry.write().await;
        *reg_lock = registry.clone();
    }

    // Build dashboard state
    let mut projects = Vec::new();
    for (name, info) in registry.list_projects() {
        let changes = scan_project_changes(&info.path);
        projects.push(ProjectInfoDto {
            name: name.clone(),
            path: info.path.display().to_string(),
            changes,
        });
    }

    let dashboard = DashboardState {
        server_info: ServerInfoDto {
            port: registry.server.port,
            pid: registry.server.pid,
            started_at: registry.server.started_at.to_rfc3339(),
            uptime: registry.server_uptime(),
        },
        projects,
    };

    Json(dashboard).into_response()
}

/// Scan a project directory for active changes
fn scan_project_changes(project_path: &std::path::Path) -> Vec<ChangeInfoDto> {
    let changes_dir = project_path.join(".aw/changes");
    let mut changes = Vec::new();

    if let Ok(entries) = std::fs::read_dir(&changes_dir) {
        for entry in entries.flatten() {
            if entry.path().is_dir() {
                let id = entry.file_name().to_string_lossy().to_string();
                let status = read_change_status(&entry.path());
                changes.push(ChangeInfoDto { id, status });
            }
        }
    }

    changes
}

/// Read status from STATE.yaml
fn read_change_status(change_dir: &std::path::Path) -> String {
    let state_file = change_dir.join("STATE.yaml");
    if state_file.exists() {
        if let Ok(content) = std::fs::read_to_string(&state_file) {
            for line in content.lines() {
                if line.starts_with("phase:") {
                    return line.trim_start_matches("phase:").trim().to_string();
                }
            }
        }
    }
    "unknown".to_string()
}

// =============================================================================
// Project Root View Handlers (/view/{project}/)
// =============================================================================

/// URL parameters for project root view
#[derive(Debug, Deserialize)]
struct ProjectPath {
    project: String,
}

/// URL parameters for change viewer
#[derive(Debug, Deserialize)]
struct ChangePath {
    project: String,
    change: String,
}

/// URL parameters for spec viewer
#[derive(Debug, Deserialize)]
struct SpecPath {
    project: String,
    spec: String,
}

/// URL parameters for knowledge viewer
#[derive(Debug, Deserialize)]
struct KnowledgePath {
    project: String,
    path: String,
}

/// Spec info for display
#[derive(Debug, Clone, Serialize)]
pub struct SpecInfoDto {
    pub id: String,
    pub title: Option<String>,
}

/// Knowledge category info
#[derive(Debug, Clone, Serialize)]
pub struct KnowledgeCategoryDto {
    pub name: String,
    pub files: Vec<String>,
}

/// Project content for the root page
#[derive(Debug, Clone)]
struct ProjectContent {
    changes: Vec<ChangeInfoDto>,
    specs: Vec<SpecInfoDto>,
    knowledge: Vec<KnowledgeCategoryDto>,
    has_project_md: bool,
}

/// Handle project root view - list all changes
async fn handle_project_root(
    State(state): State<UnifiedAppState>,
    Path(params): Path<ProjectPath>,
) -> Response {
    // Get project path from registry
    let registry = state.registry.read().await;
    let project_path = match registry.get_project_path(&params.project) {
        Some(path) => path.clone(),
        None => {
            return (
                StatusCode::NOT_FOUND,
                Html(format!(
                    r#"<!DOCTYPE html><html><head><title>Not Found</title></head>
                    <body><h1>Project '{}' not found</h1></body></html>"#,
                    params.project
                )),
            )
                .into_response();
        }
    };
    drop(registry);

    // Scan all project content
    let content = ProjectContent {
        changes: scan_project_changes(&project_path),
        specs: scan_main_specs(&project_path),
        knowledge: scan_knowledge(&project_path),
        has_project_md: project_path.join("cclab/project.md").exists(),
    };

    // Generate HTML
    let html = generate_project_root_html(&params.project, &content);
    Html(html).into_response()
}

/// Scan main specs directory
fn scan_main_specs(project_path: &std::path::Path) -> Vec<SpecInfoDto> {
    let specs_dir = project_path.join(".aw/tech-design");
    let mut specs = Vec::new();

    if let Ok(entries) = std::fs::read_dir(&specs_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() && path.extension().map_or(false, |e| e == "md") {
                let id = path
                    .file_stem()
                    .map(|s| s.to_string_lossy().to_string())
                    .unwrap_or_default();

                // Try to read title from frontmatter
                let title = std::fs::read_to_string(&path).ok().and_then(|content| {
                    content.lines().find(|l| l.starts_with("title:")).map(|l| {
                        l.trim_start_matches("title:")
                            .trim()
                            .trim_matches('"')
                            .to_string()
                    })
                });

                specs.push(SpecInfoDto { id, title });
            }
        }
    }

    specs.sort_by(|a, b| a.id.cmp(&b.id));
    specs
}

/// Scan knowledge directory
fn scan_knowledge(project_path: &std::path::Path) -> Vec<KnowledgeCategoryDto> {
    let knowledge_dir = project_path.join("cclab/knowledge");
    let mut categories = Vec::new();

    if let Ok(entries) = std::fs::read_dir(&knowledge_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                let name = path
                    .file_name()
                    .map(|s| s.to_string_lossy().to_string())
                    .unwrap_or_default();

                // Scan files in this category
                let mut files = Vec::new();
                if let Ok(sub_entries) = std::fs::read_dir(&path) {
                    for sub_entry in sub_entries.flatten() {
                        let sub_path = sub_entry.path();
                        if sub_path.is_file() && sub_path.extension().map_or(false, |e| e == "md") {
                            if let Some(fname) = sub_path.file_name() {
                                files.push(fname.to_string_lossy().to_string());
                            }
                        }
                    }
                }
                files.sort();

                if !files.is_empty() {
                    categories.push(KnowledgeCategoryDto { name, files });
                }
            }
        }
    }

    categories.sort_by(|a, b| a.name.cmp(&b.name));
    categories
}

/// API endpoint for project changes
async fn api_project_changes(
    State(state): State<UnifiedAppState>,
    Path(params): Path<ProjectPath>,
) -> Response {
    let registry = state.registry.read().await;
    let project_path = match registry.get_project_path(&params.project) {
        Some(path) => path.clone(),
        None => {
            return (
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({ "error": format!("Project '{}' not found", params.project) })),
            )
                .into_response();
        }
    };
    drop(registry);

    let changes = scan_project_changes(&project_path);
    Json(changes).into_response()
}

// =============================================================================
// File Viewer Handlers
// =============================================================================

/// Handle spec viewer
async fn handle_spec_viewer(
    State(state): State<UnifiedAppState>,
    Path(params): Path<SpecPath>,
) -> Response {
    let registry = state.registry.read().await;
    let project_path = match registry.get_project_path(&params.project) {
        Some(path) => path.clone(),
        None => {
            return (StatusCode::NOT_FOUND, Html("Project not found".to_string())).into_response();
        }
    };
    drop(registry);

    let file_path = project_path
        .join(".aw/tech-design")
        .join(format!("{}.md", params.spec));
    if !file_path.exists() {
        return (
            StatusCode::NOT_FOUND,
            Html(format!("Spec '{}' not found", params.spec)),
        )
            .into_response();
    }

    let content = match std::fs::read_to_string(&file_path) {
        Ok(c) => c,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Html(format!("Error reading file: {}", e)),
            )
                .into_response()
        }
    };

    let html = generate_file_viewer_html(
        &params.project,
        &format!("specs/{}.md", params.spec),
        &content,
        &format!("/view/{}/", params.project),
    );
    Html(html).into_response()
}

/// Handle knowledge viewer
async fn handle_knowledge_viewer(
    State(state): State<UnifiedAppState>,
    Path(params): Path<KnowledgePath>,
) -> Response {
    let registry = state.registry.read().await;
    let project_path = match registry.get_project_path(&params.project) {
        Some(path) => path.clone(),
        None => {
            return (StatusCode::NOT_FOUND, Html("Project not found".to_string())).into_response();
        }
    };
    drop(registry);

    let file_path = project_path.join("cclab/knowledge").join(&params.path);
    if !file_path.exists() {
        return (
            StatusCode::NOT_FOUND,
            Html(format!("Knowledge file '{}' not found", params.path)),
        )
            .into_response();
    }

    let content = match std::fs::read_to_string(&file_path) {
        Ok(c) => c,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Html(format!("Error reading file: {}", e)),
            )
                .into_response()
        }
    };

    let html = generate_file_viewer_html(
        &params.project,
        &format!("knowledge/{}", params.path),
        &content,
        &format!("/view/{}/", params.project),
    );
    Html(html).into_response()
}

/// Handle project.md viewer
async fn handle_project_md_viewer(
    State(state): State<UnifiedAppState>,
    Path(params): Path<ProjectPath>,
) -> Response {
    let registry = state.registry.read().await;
    let project_path = match registry.get_project_path(&params.project) {
        Some(path) => path.clone(),
        None => {
            return (StatusCode::NOT_FOUND, Html("Project not found".to_string())).into_response();
        }
    };
    drop(registry);

    let file_path = project_path.join("cclab/project.md");
    if !file_path.exists() {
        return (
            StatusCode::NOT_FOUND,
            Html("project.md not found".to_string()),
        )
            .into_response();
    }

    let content = match std::fs::read_to_string(&file_path) {
        Ok(c) => c,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Html(format!("Error reading file: {}", e)),
            )
                .into_response()
        }
    };

    let html = generate_file_viewer_html(
        &params.project,
        "project.md",
        &content,
        &format!("/view/{}/", params.project),
    );
    Html(html).into_response()
}

/// Convert markdown to HTML
fn render_markdown(content: &str) -> String {
    let mut options = Options::empty();
    options.insert(Options::ENABLE_TABLES);
    options.insert(Options::ENABLE_STRIKETHROUGH);
    options.insert(Options::ENABLE_TASKLISTS);

    let parser = Parser::new_ext(content, options);
    let mut html_output = String::new();
    html::push_html(&mut html_output, parser);
    html_output
}

/// Generate HTML for file viewer
fn generate_file_viewer_html(
    project: &str,
    file_path: &str,
    content: &str,
    back_url: &str,
) -> String {
    // Render markdown to HTML
    let rendered_content = render_markdown(content);

    format!(
        r##"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>{file_path} - {project}</title>
    <link rel="preconnect" href="https://fonts.googleapis.com">
    <link rel="preconnect" href="https://fonts.gstatic.com" crossorigin>
    <link href="https://fonts.googleapis.com/css2?family=Inter:wght@400;500;600&display=swap" rel="stylesheet">
    <style>
        :root {{
            --bg-primary: #0F172A;
            --bg-secondary: #1E293B;
            --bg-card: #334155;
            --text-primary: #F8FAFC;
            --text-secondary: #94A3B8;
            --accent: #22C55E;
            --border: #334155;
            --radius-sm: 4px;
            --radius-md: 8px;
        }}
        * {{ box-sizing: border-box; margin: 0; padding: 0; }}
        body {{
            font-family: 'Inter', -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif;
            background: var(--bg-primary);
            color: var(--text-primary);
            min-height: 100vh;
            line-height: 1.6;
        }}
        .container {{
            max-width: 900px;
            margin: 0 auto;
            padding: 2rem 1.5rem;
        }}
        header {{
            display: flex;
            align-items: center;
            gap: 1rem;
            margin-bottom: 1.5rem;
            padding-bottom: 1rem;
            border-bottom: 1px solid var(--border);
        }}
        .back-link {{
            display: inline-flex;
            align-items: center;
            gap: 0.5rem;
            color: var(--text-secondary);
            text-decoration: none;
            font-size: 0.875rem;
            font-weight: 500;
            padding: 0.5rem 0.75rem;
            border-radius: var(--radius-sm);
            transition: all 150ms ease;
            cursor: pointer;
        }}
        .back-link:hover {{
            color: var(--accent);
            background: rgba(34, 197, 94, 0.1);
        }}
        .file-path {{
            font-family: 'SF Mono', Monaco, Consolas, monospace;
            font-size: 0.875rem;
            color: var(--text-secondary);
            background: var(--bg-card);
            padding: 0.375rem 0.75rem;
            border-radius: var(--radius-sm);
        }}
        .icon {{
            width: 1rem;
            height: 1rem;
        }}
        /* Markdown content styles */
        .markdown-body {{
            padding: 1.5rem;
            background: var(--bg-secondary);
            border: 1px solid var(--border);
            border-radius: var(--radius-md);
        }}
        .markdown-body h1, .markdown-body h2, .markdown-body h3,
        .markdown-body h4, .markdown-body h5, .markdown-body h6 {{
            margin-top: 1.5rem;
            margin-bottom: 0.75rem;
            font-weight: 600;
            line-height: 1.3;
            color: var(--text-primary);
        }}
        .markdown-body h1 {{ font-size: 1.75rem; border-bottom: 1px solid var(--border); padding-bottom: 0.5rem; }}
        .markdown-body h2 {{ font-size: 1.5rem; border-bottom: 1px solid var(--border); padding-bottom: 0.375rem; }}
        .markdown-body h3 {{ font-size: 1.25rem; }}
        .markdown-body h4 {{ font-size: 1.125rem; }}
        .markdown-body h1:first-child, .markdown-body h2:first-child {{ margin-top: 0; }}
        .markdown-body p {{
            margin-bottom: 1rem;
            color: var(--text-primary);
        }}
        .markdown-body a {{
            color: #3B82F6;
            text-decoration: none;
        }}
        .markdown-body a:hover {{
            text-decoration: underline;
        }}
        .markdown-body ul, .markdown-body ol {{
            margin-bottom: 1rem;
            padding-left: 1.5rem;
        }}
        .markdown-body li {{
            margin-bottom: 0.25rem;
        }}
        .markdown-body li > ul, .markdown-body li > ol {{
            margin-bottom: 0;
            margin-top: 0.25rem;
        }}
        .markdown-body code {{
            font-family: 'SF Mono', Monaco, Consolas, monospace;
            font-size: 0.875em;
            background: #0D1117;
            padding: 0.125rem 0.375rem;
            border-radius: var(--radius-sm);
            color: #E06C75;
        }}
        .markdown-body pre {{
            background: #0D1117;
            padding: 1rem;
            border-radius: var(--radius-md);
            overflow-x: auto;
            margin-bottom: 1rem;
            border: 1px solid var(--border);
        }}
        .markdown-body pre code {{
            background: none;
            padding: 0;
            font-size: 0.8125rem;
            line-height: 1.6;
            color: var(--text-primary);
        }}
        .markdown-body blockquote {{
            border-left: 4px solid var(--accent);
            padding-left: 1rem;
            margin: 1rem 0;
            color: var(--text-secondary);
            font-style: italic;
        }}
        .markdown-body table {{
            width: 100%;
            border-collapse: collapse;
            margin-bottom: 1rem;
        }}
        .markdown-body th, .markdown-body td {{
            padding: 0.625rem 0.875rem;
            border: 1px solid var(--border);
            text-align: left;
        }}
        .markdown-body th {{
            background: var(--bg-card);
            font-weight: 600;
        }}
        .markdown-body tr:nth-child(even) {{
            background: rgba(255, 255, 255, 0.02);
        }}
        .markdown-body hr {{
            border: none;
            border-top: 1px solid var(--border);
            margin: 1.5rem 0;
        }}
        .markdown-body img {{
            max-width: 100%;
            border-radius: var(--radius-md);
        }}
        .markdown-body strong {{
            font-weight: 600;
            color: var(--text-primary);
        }}
        @media (max-width: 640px) {{
            .container {{ padding: 1rem; }}
            header {{ flex-direction: column; align-items: flex-start; }}
            .markdown-body {{ padding: 1rem; }}
            .markdown-body pre {{ font-size: 0.75rem; padding: 0.75rem; }}
        }}
    </style>
</head>
<body>
    <div class="container">
        <header>
            <a href="{back_url}" class="back-link">
                <svg class="icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                    <line x1="19" y1="12" x2="5" y2="12"></line>
                    <polyline points="12 19 5 12 12 5"></polyline>
                </svg>
                Back
            </a>
            <span class="file-path">{file_path}</span>
        </header>

        <div class="markdown-body">
            {content}
        </div>
    </div>
</body>
</html>"##,
        project = project,
        file_path = file_path,
        back_url = back_url,
        content = rendered_content
    )
}

/// Handle change viewer
async fn handle_change_viewer(
    State(state): State<UnifiedAppState>,
    Path(params): Path<ChangePath>,
) -> Response {
    let registry = state.registry.read().await;
    let project_path = match registry.get_project_path(&params.project) {
        Some(path) => path.clone(),
        None => {
            return (
                StatusCode::NOT_FOUND,
                Html(format!("Project '{}' not found", params.project)),
            )
                .into_response();
        }
    };
    drop(registry);

    // Check change exists
    let change_dir = project_path.join(".aw/changes").join(&params.change);
    if !change_dir.exists() {
        return (
            StatusCode::NOT_FOUND,
            Html(format!(
                "Change '{}' not found in project '{}'",
                params.change, params.project
            )),
        )
            .into_response();
    }

    // Read status
    let status = read_change_status(&change_dir);

    // Scan files in change directory
    let files = scan_change_files(&change_dir);

    let html = generate_change_viewer_html(
        &params.project,
        &params.change,
        &status,
        &change_dir,
        &files,
    );
    Html(html).into_response()
}

/// Scan files in a change directory
fn scan_change_files(change_dir: &std::path::Path) -> Vec<String> {
    let mut files = Vec::new();

    // Check for proposal.md
    if change_dir.join("proposal.md").exists() {
        files.push("proposal.md".to_string());
    }

    // Check for specs directory
    let specs_dir = change_dir.join("specs");
    if specs_dir.exists() {
        if let Ok(entries) = std::fs::read_dir(&specs_dir) {
            for entry in entries.flatten() {
                if let Some(name) = entry.file_name().to_str() {
                    if name.ends_with(".md") {
                        files.push(format!("specs/{}", name));
                    }
                }
            }
        }
    }

    // Check for tasks.md
    if change_dir.join("tasks.md").exists() {
        files.push("tasks.md".to_string());
    }

    // Check for clarifications.md
    if change_dir.join("clarifications.md").exists() {
        files.push("clarifications.md".to_string());
    }

    // Check for REVIEW.md
    if change_dir.join("REVIEW.md").exists() {
        files.push("REVIEW.md".to_string());
    }

    files
}

/// Generate HTML for change viewer page
fn generate_change_viewer_html(
    project: &str,
    change: &str,
    status: &str,
    change_dir: &std::path::Path,
    files: &[String],
) -> String {
    let status_class = format!("status-{}", status.to_lowercase().replace(' ', "-"));

    let files_html = if files.is_empty() {
        r#"<div class="empty-state">No files found</div>"#.to_string()
    } else {
        let items: Vec<String> = files
            .iter()
            .map(|f| {
                format!(
                    r#"<div class="file-item">
                        <svg class="icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                            <path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"></path>
                            <polyline points="14 2 14 8 20 8"></polyline>
                        </svg>
                        <span class="file-name">{}</span>
                    </div>"#,
                    f
                )
            })
            .collect();
        format!(r#"<div class="files-list">{}</div>"#, items.join(""))
    };

    format!(
        r##"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>{change} - {project}</title>
    <link rel="preconnect" href="https://fonts.googleapis.com">
    <link rel="preconnect" href="https://fonts.gstatic.com" crossorigin>
    <link href="https://fonts.googleapis.com/css2?family=Inter:wght@400;500;600&display=swap" rel="stylesheet">
    <style>
        :root {{
            --bg-primary: #0F172A;
            --bg-secondary: #1E293B;
            --bg-card: #334155;
            --bg-card-hover: #3E4C63;
            --text-primary: #F8FAFC;
            --text-secondary: #94A3B8;
            --accent: #22C55E;
            --border: #334155;
            --status-proposed: #F59E0B;
            --status-challenged: #3B82F6;
            --status-implementing: #E94560;
            --status-implemented: #22C55E;
            --status-archived: #6B7280;
            --radius-sm: 4px;
            --radius-md: 8px;
            --radius-lg: 12px;
        }}
        * {{ box-sizing: border-box; margin: 0; padding: 0; }}
        body {{
            font-family: 'Inter', -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif;
            background: var(--bg-primary);
            color: var(--text-primary);
            min-height: 100vh;
            line-height: 1.5;
        }}
        .container {{
            max-width: 800px;
            margin: 0 auto;
            padding: 2rem 1.5rem;
        }}
        header {{
            display: flex;
            align-items: center;
            justify-content: space-between;
            margin-bottom: 2rem;
            padding-bottom: 1.5rem;
            border-bottom: 1px solid var(--border);
        }}
        .back-link {{
            display: inline-flex;
            align-items: center;
            gap: 0.5rem;
            color: var(--text-secondary);
            text-decoration: none;
            font-size: 0.875rem;
            font-weight: 500;
            padding: 0.5rem 0.75rem;
            border-radius: var(--radius-sm);
            transition: all 150ms ease;
            cursor: pointer;
        }}
        .back-link:hover {{
            color: var(--accent);
            background: rgba(34, 197, 94, 0.1);
        }}
        .back-link:focus {{
            outline: 2px solid var(--accent);
            outline-offset: 2px;
        }}
        h1 {{
            font-size: 1.5rem;
            font-weight: 600;
            color: var(--text-primary);
        }}
        .icon {{
            width: 1rem;
            height: 1rem;
            flex-shrink: 0;
        }}
        .info-section {{
            background: var(--bg-secondary);
            border: 1px solid var(--border);
            border-radius: var(--radius-md);
            padding: 1.25rem;
            margin-bottom: 1.5rem;
        }}
        .info-row {{
            display: flex;
            align-items: center;
            gap: 0.75rem;
            margin-bottom: 0.75rem;
        }}
        .info-row:last-child {{
            margin-bottom: 0;
        }}
        .info-label {{
            color: var(--text-secondary);
            font-size: 0.875rem;
            min-width: 60px;
        }}
        .info-value {{
            color: var(--text-primary);
            font-size: 0.875rem;
            font-family: 'SF Mono', Monaco, Consolas, monospace;
        }}
        .status-badge {{
            display: inline-flex;
            align-items: center;
            font-size: 0.625rem;
            font-weight: 600;
            text-transform: uppercase;
            letter-spacing: 0.05em;
            padding: 0.25rem 0.5rem;
            border-radius: var(--radius-sm);
        }}
        .status-proposed {{ background: rgba(245, 158, 11, 0.15); color: var(--status-proposed); }}
        .status-challenged {{ background: rgba(59, 130, 246, 0.15); color: var(--status-challenged); }}
        .status-implementing {{ background: rgba(233, 69, 96, 0.15); color: var(--status-implementing); }}
        .status-implemented {{ background: rgba(34, 197, 94, 0.15); color: var(--status-implemented); }}
        .status-archived {{ background: rgba(107, 114, 128, 0.15); color: var(--status-archived); }}
        .status-unknown {{ background: rgba(148, 163, 184, 0.15); color: var(--text-secondary); }}
        .section-title {{
            font-size: 0.75rem;
            font-weight: 600;
            text-transform: uppercase;
            letter-spacing: 0.1em;
            color: var(--text-secondary);
            margin-bottom: 1rem;
        }}
        .files-list {{
            display: flex;
            flex-direction: column;
            gap: 0.25rem;
        }}
        .file-item {{
            display: flex;
            align-items: center;
            gap: 0.75rem;
            padding: 0.75rem 1rem;
            background: var(--bg-secondary);
            border: 1px solid var(--border);
            border-radius: var(--radius-sm);
            transition: all 150ms ease;
        }}
        .file-item:hover {{
            background: var(--bg-card);
        }}
        .file-item .icon {{
            color: var(--text-secondary);
        }}
        .file-name {{
            font-size: 0.875rem;
            font-family: 'SF Mono', Monaco, Consolas, monospace;
            color: var(--text-primary);
        }}
        .empty-state {{
            text-align: center;
            padding: 2rem;
            color: var(--text-secondary);
            font-size: 0.875rem;
        }}
        @media (max-width: 640px) {{
            .container {{ padding: 1rem; }}
            header {{ flex-direction: column; align-items: flex-start; gap: 1rem; }}
            h1 {{ font-size: 1.25rem; }}
        }}
    </style>
</head>
<body>
    <div class="container">
        <header>
            <a href="/view/{project}/" class="back-link">
                <svg class="icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                    <line x1="19" y1="12" x2="5" y2="12"></line>
                    <polyline points="12 19 5 12 12 5"></polyline>
                </svg>
                Back to {project}
            </a>
            <h1>{change}</h1>
        </header>

        <div class="info-section">
            <div class="info-row">
                <span class="info-label">Status</span>
                <span class="status-badge {status_class}">{status}</span>
            </div>
            <div class="info-row">
                <span class="info-label">Path</span>
                <span class="info-value">{path}</span>
            </div>
        </div>

        <div class="section-title">Files</div>
        {files_html}
    </div>
</body>
</html>"##,
        project = project,
        change = change,
        status = status.to_uppercase(),
        status_class = status_class,
        path = change_dir.display(),
        files_html = files_html
    )
}

/// Generate HTML for project root page
fn generate_project_root_html(project: &str, content: &ProjectContent) -> String {
    // Generate changes section
    let changes_html = if content.changes.is_empty() {
        r#"<div class="empty-section">No active changes</div>"#.to_string()
    } else {
        let items: Vec<String> = content.changes
            .iter()
            .map(|c| {
                let status_class = format!("status-{}", c.status.to_lowercase().replace(' ', "-"));
                format!(
                    r##"<a href="/view/{project}/{id}/" class="item-card">
                        <div class="card-left">
                            <svg class="icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                                <path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"></path>
                            </svg>
                            <span class="item-name">{id}</span>
                        </div>
                        <span class="status-badge {status_class}">{status}</span>
                    </a>"##,
                    project = project,
                    id = c.id,
                    status_class = status_class,
                    status = c.status.to_uppercase()
                )
            })
            .collect();
        format!(r#"<div class="items-list">{}</div>"#, items.join("\n"))
    };

    // Generate specs section
    let specs_html = if content.specs.is_empty() {
        r#"<div class="empty-section">No specs defined</div>"#.to_string()
    } else {
        let items: Vec<String> = content.specs
            .iter()
            .map(|s| {
                let display = s.title.as_ref().unwrap_or(&s.id);
                format!(
                    r##"<a href="/view/{project}/file/spec/{id}" class="item-card file-card">
                        <div class="card-left">
                            <svg class="icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                                <path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"></path>
                                <polyline points="14 2 14 8 20 8"></polyline>
                            </svg>
                            <span class="item-name">{display}</span>
                        </div>
                        <span class="item-meta">{id}.md</span>
                    </a>"##,
                    project = project,
                    display = display,
                    id = s.id
                )
            })
            .collect();
        format!(r#"<div class="items-list">{}</div>"#, items.join("\n"))
    };

    // Generate knowledge section
    let knowledge_html = if content.knowledge.is_empty() {
        r#"<div class="empty-section">No knowledge files</div>"#.to_string()
    } else {
        let categories: Vec<String> = content.knowledge
            .iter()
            .map(|cat| {
                let cat_name = &cat.name;
                let files_html: Vec<String> = cat.files.iter().map(|f| {
                    format!(
                        r##"<a href="/view/{project}/file/knowledge/{category}/{file}" class="knowledge-file">
                            <svg class="icon-sm" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                                <path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"></path>
                                <polyline points="14 2 14 8 20 8"></polyline>
                            </svg>
                            <span>{file}</span>
                        </a>"##,
                        project = project,
                        category = cat_name,
                        file = f
                    )
                }).collect();
                format!(
                    r##"<div class="knowledge-category">
                        <div class="category-header">
                            <svg class="icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                                <path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"></path>
                            </svg>
                            <span class="category-name">{}</span>
                            <span class="category-count">{} files</span>
                        </div>
                        <div class="category-files">{}</div>
                    </div>"##,
                    cat.name,
                    cat.files.len(),
                    files_html.join("\n")
                )
            })
            .collect();
        format!(
            r#"<div class="knowledge-list">{}</div>"#,
            categories.join("\n")
        )
    };

    // Project.md link
    let project_md_html = if content.has_project_md {
        format!(
            r##"<a href="/view/{project}/file/project.md" class="item-card file-card">
            <div class="card-left">
                <svg class="icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                    <path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"></path>
                    <polyline points="14 2 14 8 20 8"></polyline>
                    <line x1="16" y1="13" x2="8" y2="13"></line>
                    <line x1="16" y1="17" x2="8" y2="17"></line>
                    <polyline points="10 9 9 9 8 9"></polyline>
                </svg>
                <span class="item-name">project.md</span>
            </div>
            <span class="item-meta">Project configuration</span>
        </a>"##,
            project = project
        )
    } else {
        String::new()
    };

    format!(
        r##"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>{project} - SDD</title>
    <link rel="preconnect" href="https://fonts.googleapis.com">
    <link rel="preconnect" href="https://fonts.gstatic.com" crossorigin>
    <link href="https://fonts.googleapis.com/css2?family=Inter:wght@400;500;600&display=swap" rel="stylesheet">
    <style>
        :root {{
            --bg-primary: #0F172A;
            --bg-secondary: #1E293B;
            --bg-card: #334155;
            --bg-card-hover: #3E4C63;
            --text-primary: #F8FAFC;
            --text-secondary: #94A3B8;
            --accent: #22C55E;
            --accent-blue: #3B82F6;
            --accent-purple: #8B5CF6;
            --accent-amber: #F59E0B;
            --border: #334155;
            --status-proposed: #F59E0B;
            --status-challenged: #3B82F6;
            --status-implementing: #E94560;
            --status-implemented: #22C55E;
            --status-archived: #6B7280;
            --radius-sm: 4px;
            --radius-md: 8px;
            --radius-lg: 12px;
        }}
        * {{ box-sizing: border-box; margin: 0; padding: 0; }}
        body {{
            font-family: 'Inter', -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif;
            background: var(--bg-primary);
            color: var(--text-primary);
            min-height: 100vh;
            line-height: 1.5;
        }}
        .container {{
            max-width: 900px;
            margin: 0 auto;
            padding: 2rem 1.5rem;
        }}
        header {{
            display: flex;
            align-items: center;
            justify-content: space-between;
            margin-bottom: 2rem;
            padding-bottom: 1.5rem;
            border-bottom: 1px solid var(--border);
        }}
        .back-link {{
            display: inline-flex;
            align-items: center;
            gap: 0.5rem;
            color: var(--text-secondary);
            text-decoration: none;
            font-size: 0.875rem;
            font-weight: 500;
            padding: 0.5rem 0.75rem;
            border-radius: var(--radius-sm);
            transition: all 150ms ease;
            cursor: pointer;
        }}
        .back-link:hover {{
            color: var(--accent);
            background: rgba(34, 197, 94, 0.1);
        }}
        .header-title {{
            display: flex;
            align-items: center;
            gap: 0.5rem;
        }}
        .header-title span {{
            color: var(--text-secondary);
            font-size: 0.875rem;
        }}
        h1 {{
            font-size: 1.75rem;
            font-weight: 600;
            color: var(--text-primary);
        }}
        .icon {{
            width: 1.125rem;
            height: 1.125rem;
            flex-shrink: 0;
        }}
        .icon-sm {{
            width: 0.875rem;
            height: 0.875rem;
            flex-shrink: 0;
            color: var(--text-secondary);
        }}

        /* Sections */
        .section {{
            margin-bottom: 2rem;
        }}
        .section-header {{
            display: flex;
            align-items: center;
            gap: 0.5rem;
            margin-bottom: 1rem;
        }}
        .section-title {{
            font-size: 0.75rem;
            font-weight: 600;
            text-transform: uppercase;
            letter-spacing: 0.1em;
            color: var(--text-secondary);
        }}
        .section-count {{
            font-size: 0.625rem;
            background: var(--bg-card);
            color: var(--text-secondary);
            padding: 0.125rem 0.375rem;
            border-radius: var(--radius-sm);
        }}

        /* Items list */
        .items-list {{
            display: flex;
            flex-direction: column;
            gap: 0.5rem;
        }}
        .item-card {{
            display: flex;
            align-items: center;
            justify-content: space-between;
            padding: 0.875rem 1rem;
            background: var(--bg-secondary);
            border: 1px solid var(--border);
            border-radius: var(--radius-md);
            text-decoration: none;
            transition: all 150ms ease;
        }}
        a.item-card {{
            cursor: pointer;
        }}
        a.item-card:hover {{
            background: var(--bg-card);
            border-color: var(--accent);
            transform: translateY(-1px);
        }}
        .file-card {{
            background: var(--bg-secondary);
        }}
        .card-left {{
            display: flex;
            align-items: center;
            gap: 0.75rem;
        }}
        .card-left .icon {{
            color: var(--text-secondary);
        }}
        .item-name {{
            font-weight: 500;
            font-size: 0.9375rem;
            color: var(--text-primary);
        }}
        .item-meta {{
            font-size: 0.75rem;
            color: var(--text-secondary);
            font-family: 'SF Mono', Monaco, Consolas, monospace;
        }}

        /* Status badges */
        .status-badge {{
            font-size: 0.625rem;
            font-weight: 600;
            text-transform: uppercase;
            letter-spacing: 0.05em;
            padding: 0.25rem 0.5rem;
            border-radius: var(--radius-sm);
        }}
        .status-proposed {{ background: rgba(245, 158, 11, 0.15); color: var(--status-proposed); }}
        .status-challenged {{ background: rgba(59, 130, 246, 0.15); color: var(--status-challenged); }}
        .status-implementing {{ background: rgba(233, 69, 96, 0.15); color: var(--status-implementing); }}
        .status-implemented {{ background: rgba(34, 197, 94, 0.15); color: var(--status-implemented); }}
        .status-archived {{ background: rgba(107, 114, 128, 0.15); color: var(--status-archived); }}
        .status-complete {{ background: rgba(34, 197, 94, 0.15); color: var(--status-implemented); }}
        .status-unknown {{ background: rgba(148, 163, 184, 0.15); color: var(--text-secondary); }}

        /* Knowledge section */
        .knowledge-list {{
            display: flex;
            flex-direction: column;
            gap: 0.75rem;
        }}
        .knowledge-category {{
            background: var(--bg-secondary);
            border: 1px solid var(--border);
            border-radius: var(--radius-md);
            overflow: hidden;
        }}
        .category-header {{
            display: flex;
            align-items: center;
            gap: 0.5rem;
            padding: 0.75rem 1rem;
            background: var(--bg-card);
            border-bottom: 1px solid var(--border);
        }}
        .category-header .icon {{
            color: var(--accent-amber);
        }}
        .category-name {{
            font-weight: 500;
            font-size: 0.875rem;
            color: var(--text-primary);
        }}
        .category-count {{
            margin-left: auto;
            font-size: 0.6875rem;
            color: var(--text-secondary);
        }}
        .category-files {{
            padding: 0.5rem;
            display: flex;
            flex-direction: column;
            gap: 0.25rem;
        }}
        .knowledge-file {{
            display: flex;
            align-items: center;
            gap: 0.5rem;
            padding: 0.375rem 0.5rem;
            font-size: 0.8125rem;
            color: var(--text-secondary);
            font-family: 'SF Mono', Monaco, Consolas, monospace;
            border-radius: var(--radius-sm);
            text-decoration: none;
            cursor: pointer;
            transition: all 150ms ease;
        }}
        .knowledge-file:hover {{
            background: var(--bg-card);
            color: var(--accent);
        }}

        /* Empty state */
        .empty-section {{
            padding: 1.5rem;
            text-align: center;
            color: var(--text-secondary);
            font-size: 0.875rem;
            background: var(--bg-secondary);
            border: 1px dashed var(--border);
            border-radius: var(--radius-md);
        }}

        /* Section icons colored */
        .section-changes .section-header .icon {{ color: var(--accent); }}
        .section-specs .section-header .icon {{ color: var(--accent-blue); }}
        .section-knowledge .section-header .icon {{ color: var(--accent-amber); }}
        .section-config .section-header .icon {{ color: var(--accent-purple); }}

        @media (max-width: 640px) {{
            .container {{ padding: 1rem; }}
            header {{ flex-direction: column; align-items: flex-start; gap: 1rem; }}
            h1 {{ font-size: 1.5rem; }}
        }}
    </style>
</head>
<body>
    <div class="container">
        <header>
            <a href="/" class="back-link">
                <svg class="icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                    <line x1="19" y1="12" x2="5" y2="12"></line>
                    <polyline points="12 19 5 12 12 5"></polyline>
                </svg>
                Dashboard
            </a>
            <div class="header-title">
                <span>Project:</span>
                <h1>{project}</h1>
            </div>
        </header>

        <main>
            <!-- Changes Section -->
            <div class="section section-changes">
                <div class="section-header">
                    <svg class="icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                        <circle cx="12" cy="12" r="10"></circle>
                        <path d="M12 6v6l4 2"></path>
                    </svg>
                    <span class="section-title">Active Changes</span>
                    <span class="section-count">{changes_count}</span>
                </div>
                {changes_html}
            </div>

            <!-- Specs Section -->
            <div class="section section-specs">
                <div class="section-header">
                    <svg class="icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                        <path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"></path>
                        <polyline points="14 2 14 8 20 8"></polyline>
                        <line x1="16" y1="13" x2="8" y2="13"></line>
                        <line x1="16" y1="17" x2="8" y2="17"></line>
                    </svg>
                    <span class="section-title">Main Specs</span>
                    <span class="section-count">{specs_count}</span>
                </div>
                {specs_html}
            </div>

            <!-- Knowledge Section -->
            <div class="section section-knowledge">
                <div class="section-header">
                    <svg class="icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                        <path d="M4 19.5A2.5 2.5 0 0 1 6.5 17H20"></path>
                        <path d="M6.5 2H20v20H6.5A2.5 2.5 0 0 1 4 19.5v-15A2.5 2.5 0 0 1 6.5 2z"></path>
                    </svg>
                    <span class="section-title">Knowledge</span>
                    <span class="section-count">{knowledge_count} categories</span>
                </div>
                {knowledge_html}
            </div>

            <!-- Config Section -->
            {project_md_section}
        </main>
    </div>
</body>
</html>"##,
        project = project,
        changes_count = content.changes.len(),
        changes_html = changes_html,
        specs_count = content.specs.len(),
        specs_html = specs_html,
        knowledge_count = content.knowledge.len(),
        knowledge_html = knowledge_html,
        project_md_section = if content.has_project_md {
            format!(
                r##"<div class="section section-config">
                <div class="section-header">
                    <svg class="icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                        <circle cx="12" cy="12" r="3"></circle>
                        <path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1 0 2.83 2 2 0 0 1-2.83 0l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-2 2 2 2 0 0 1-2-2v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83 0 2 2 0 0 1 0-2.83l.06-.06a1.65 1.65 0 0 0 .33-1.82 1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1-2-2 2 2 0 0 1 2-2h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 0-2.83 2 2 0 0 1 2.83 0l.06.06a1.65 1.65 0 0 0 1.82.33H9a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 2-2 2 2 0 0 1 2 2v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 0 2 2 0 0 1 0 2.83l-.06.06a1.65 1.65 0 0 0-.33 1.82V9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 2 2 2 2 0 0 1-2 2h-.09a1.65 1.65 0 0 0-1.51 1z"></path>
                    </svg>
                    <span class="section-title">Configuration</span>
                </div>
                {}</div>"##,
                project_md_html
            )
        } else {
            String::new()
        }
    )
}

// =============================================================================
// SDD Viewer Handlers (Project-Level)
// =============================================================================

/// Serve project SDD Viewer HTML
async fn handle_project_sdd(Path(project): Path<String>) -> Html<String> {
    // Simple sdd viewer HTML template
    // In a real implementation, this would load from assets or template files
    let html = format!(
        r#"<!DOCTYPE html>
<html>
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>SDD Viewer - {}</title>
    <style>
        * {{ margin: 0; padding: 0; box-sizing: border-box; }}
        body {{ font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif; background: #f5f5f5; }}
        .container {{ display: flex; height: 100vh; }}
        .sidebar {{ width: 300px; background: white; border-right: 1px solid #ddd; overflow-y: auto; }}
        .main {{ flex: 1; display: flex; flex-direction: column; }}
        .tree-node {{ padding: 8px 12px; cursor: pointer; user-select: none; }}
        .tree-node:hover {{ background: #f0f0f0; }}
        .tree-node.directory {{ font-weight: 500; }}
        .content {{ flex: 1; overflow-y: auto; padding: 20px; }}
        .header {{ background: white; border-bottom: 1px solid #ddd; padding: 12px 20px; }}
        pre {{ background: #f5f5f5; padding: 12px; border-radius: 4px; overflow-x: auto; }}
        code {{ font-family: 'Monaco', 'Courier', monospace; font-size: 12px; }}
    </style>
</head>
<body>
    <div class="container">
        <div class="sidebar" id="sidebar">
            <div style="padding: 12px; font-weight: bold;">SDD</div>
            <div id="tree"></div>
        </div>
        <div class="main">
            <div class="header">Project: <strong>{}</strong></div>
            <div class="content" id="content">
                <p>Select a file to view...</p>
            </div>
        </div>
    </div>
    <script>
        const project = '{}';

        async function loadTree() {{
            const response = await fetch(`/api/${{project}}/cclab/tree`);
            const tree = await response.json();
            renderTree(tree, document.getElementById('tree'));
        }}

        function renderTree(node, container, level = 0) {{
            const div = document.createElement('div');
            div.className = `tree-node ${{node.is_directory ? 'directory' : 'file'}}`;
            div.style.paddingLeft = (12 + level * 12) + 'px';
            div.textContent = node.name;

            if (node.is_directory && node.children) {{
                div.style.cursor = 'pointer';
                let expanded = false;
                const childContainer = document.createElement('div');
                childContainer.style.display = 'none';

                div.addEventListener('click', () => {{
                    expanded = !expanded;
                    childContainer.style.display = expanded ? 'block' : 'none';
                }});

                container.appendChild(div);

                node.children.forEach(child => {{
                    renderTree(child, childContainer, level + 1);
                }});

                container.appendChild(childContainer);
            }} else {{
                container.appendChild(div);
            }}
        }}

        loadTree();
    </script>
</body>
</html>"#,
        project, project, project
    );
    Html(html)
}

/// API endpoint to get project SDD tree structure
async fn api_sdd_tree(Path(_project): Path<String>) -> Response {
    // Placeholder implementation - in production, would load from filesystem
    let tree = serde_json::json!({
        "id": "sdd-root",
        "name": "SDD",
        "path": ".",
        "is_directory": true,
        "children": [
            {
                "id": "dir-specs",
                "name": "specs",
                "path": "specs",
                "is_directory": true,
                "children": []
            },
            {
                "id": "dir-knowledge",
                "name": "knowledge",
                "path": "knowledge",
                "is_directory": true,
                "children": []
            }
        ]
    });

    Json(tree).into_response()
}
