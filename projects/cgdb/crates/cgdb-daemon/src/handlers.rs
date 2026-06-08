// HANDWRITE-BEGIN gap="missing-generator:hand-written:6fe312b9" tracker="2087" reason="RPC method implementations (daemon.status, project.*, query.*, catalog.list_types)."
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::Instant;

use anyhow::{anyhow, Context, Result};
use cgdb_core::catalog::{Catalog, CatalogProject};
use cgdb_core::rpc::{RpcRequest, RpcResponse};
use serde_json::{json, Value};

use crate::indexer;
use crate::lens_service;
use crate::query;

pub struct DaemonState {
    pub data_root: PathBuf,
    pub uds_socket: String,
    pub tcp_listen: Option<String>,
    pub start: Instant,
}

impl DaemonState {
    pub fn catalog_path(&self) -> PathBuf {
        self.data_root.join("data").join("catalog.toml")
    }
    pub fn project_dir(&self, name: &str) -> PathBuf {
        self.data_root.join("data").join(name)
    }
    pub fn graph_path(&self, name: &str) -> PathBuf {
        self.project_dir(name).join("graph.jsonl")
    }
}

pub fn dispatch(state: &Arc<Mutex<DaemonState>>, req: RpcRequest) -> RpcResponse {
    let id = req.id.clone();
    let result: Result<Value> = match req.method.as_str() {
        "daemon.status" => daemon_status(state),
        "project.register" => project_register(state, &req.params),
        "project.unregister" => project_unregister(state, &req.params),
        "project.list" => project_list(state),
        "project.sync" => project_sync(state, &req.params),
        "query.coverage" => query_coverage(state, &req.params),
        "query.impact" => query_impact(state, &req.params),
        "lens.overview" => lens_overview(state, &req.params),
        "lens.zoom_in" => lens_zoom_in(state, &req.params),
        "lens.zoom_out" => lens_zoom_out(state, &req.params),
        "lens.focus" => lens_focus(state, &req.params),
        "lens.breadcrumb" => lens_breadcrumb(state, &req.params),
        "catalog.list_types" => Ok(json!({
            "node_types": ["Spec", "Code"],
            "edge_types": ["SpecRef", "SemanticSpecRef", "Contains"],
        })),
        other => Err(anyhow!("unknown method: {}", other)),
    };
    match result {
        Ok(v) => RpcResponse::ok(id, v),
        Err(e) => RpcResponse::err(id, -32000, e.to_string()),
    }
}

fn now_iso() -> String {
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    format!("@{}", secs)
}

fn daemon_status(state: &Arc<Mutex<DaemonState>>) -> Result<Value> {
    let s = state.lock().unwrap();
    let catalog = Catalog::load(&s.catalog_path())?;
    let projects: Vec<Value> = catalog
        .projects
        .iter()
        .map(|p| json!({ "name": p.name, "last_sync": p.last_sync }))
        .collect();
    Ok(json!({
        "pid": std::process::id(),
        "uds_socket": s.uds_socket,
        "tcp_listen": s.tcp_listen,
        "uptime_seconds": s.start.elapsed().as_secs(),
        "project_count": catalog.projects.len(),
        "rss_bytes": rss_bytes(),
        "projects": projects,
    }))
}

fn rss_bytes() -> u64 {
    if let Ok(s) = std::fs::read_to_string("/proc/self/statm") {
        if let Some(pages) = s.split_whitespace().nth(1).and_then(|v| v.parse::<u64>().ok()) {
            return pages * 4096;
        }
    }
    0
}

#[derive(serde::Deserialize)]
struct RegisterParams {
    name: String,
    repo_root: String,
}

fn read_score_project(repo_root: &Path, name: &str) -> Result<(String, String)> {
    let cfg_path = repo_root.join(".score").join("config.toml");
    let body = std::fs::read_to_string(&cfg_path)
        .with_context(|| format!("read {}", cfg_path.display()))?;
    let cfg: toml::Value = toml::from_str(&body)?;
    let arr = cfg
        .get("projects")
        .and_then(|v| v.as_array())
        .ok_or_else(|| anyhow!("no [[projects]] table in {}", cfg_path.display()))?;
    for p in arr {
        let n = p.get("name").and_then(|v| v.as_str()).unwrap_or("");
        if n == name {
            let path = p
                .get("path")
                .and_then(|v| v.as_str())
                .ok_or_else(|| anyhow!("project {} missing path", name))?;
            let td_path = p
                .get("td_path")
                .and_then(|v| v.as_str())
                .unwrap_or(".score/tech_design");
            return Ok((
                repo_root.join(path).to_string_lossy().into_owned(),
                repo_root.join(td_path).to_string_lossy().into_owned(),
            ));
        }
    }
    Err(anyhow!("unknown project: {}", name))
}

fn project_register(state: &Arc<Mutex<DaemonState>>, params: &Value) -> Result<Value> {
    let p: RegisterParams = serde_json::from_value(params.clone())?;
    let repo_root = PathBuf::from(&p.repo_root);
    let (source_path, td_path) = read_score_project(&repo_root, &p.name)?;
    let s = state.lock().unwrap();
    let mut catalog = Catalog::load(&s.catalog_path())?;
    catalog.upsert(CatalogProject {
        name: p.name.clone(),
        td_path: td_path.clone(),
        source_path: source_path.clone(),
        registered_at: now_iso(),
        last_sync: None,
    });
    catalog.save(&s.catalog_path())?;
    let data_dir = s.project_dir(&p.name);
    std::fs::create_dir_all(&data_dir)?;
    Ok(json!({
        "name": p.name,
        "td_path": td_path,
        "source_path": source_path,
        "data_dir": data_dir.to_string_lossy(),
    }))
}

#[derive(serde::Deserialize)]
struct NameParams {
    name: String,
}

fn project_unregister(state: &Arc<Mutex<DaemonState>>, params: &Value) -> Result<Value> {
    let p: NameParams = serde_json::from_value(params.clone())?;
    let s = state.lock().unwrap();
    let mut catalog = Catalog::load(&s.catalog_path())?;
    let removed = catalog
        .remove(&p.name)
        .ok_or_else(|| anyhow!("not registered: {}", p.name))?;
    catalog.save(&s.catalog_path())?;
    let dir = s.project_dir(&removed.name);
    if dir.exists() {
        let _ = std::fs::remove_dir_all(&dir);
    }
    Ok(json!({ "name": removed.name }))
}

fn project_list(state: &Arc<Mutex<DaemonState>>) -> Result<Value> {
    let s = state.lock().unwrap();
    let catalog = Catalog::load(&s.catalog_path())?;
    let projects: Vec<Value> = catalog
        .projects
        .iter()
        .map(|p| {
            json!({
                "name": p.name,
                "td_path": p.td_path,
                "source_path": p.source_path,
                "last_sync": p.last_sync,
            })
        })
        .collect();
    Ok(Value::Array(projects))
}

#[derive(serde::Deserialize)]
struct SyncParams {
    name: String,
    #[serde(default)]
    rebuild: bool,
}

fn project_sync(state: &Arc<Mutex<DaemonState>>, params: &Value) -> Result<Value> {
    let p: SyncParams = serde_json::from_value(params.clone())?;
    let (project, graph_path) = {
        let s = state.lock().unwrap();
        let catalog = Catalog::load(&s.catalog_path())?;
        let proj = catalog
            .find(&p.name)
            .ok_or_else(|| anyhow!("not registered: {}", p.name))?
            .clone();
        (proj, s.graph_path(&p.name))
    };
    let result = indexer::run_sync(&project, &graph_path, p.rebuild)?;
    {
        let s = state.lock().unwrap();
        let mut catalog = Catalog::load(&s.catalog_path())?;
        if let Some(existing) = catalog.projects.iter_mut().find(|x| x.name == p.name) {
            existing.last_sync = Some(now_iso());
        }
        catalog.save(&s.catalog_path())?;
    }
    Ok(json!({
        "project": p.name,
        "node_count": result.node_count,
        "edge_count": result.edge_count,
        "duration_ms": result.duration_ms,
    }))
}

fn query_coverage(state: &Arc<Mutex<DaemonState>>, params: &Value) -> Result<Value> {
    let p: NameParams = serde_json::from_value(params.clone())?;
    let graph_path = {
        let s = state.lock().unwrap();
        s.graph_path(&p.name)
    };
    let result = query::coverage(&graph_path)?;
    Ok(serde_json::to_value(result)?)
}

#[derive(serde::Deserialize)]
struct ImpactParams {
    name: String,
    spec_section: String,
}

fn query_impact(state: &Arc<Mutex<DaemonState>>, params: &Value) -> Result<Value> {
    let p: ImpactParams = serde_json::from_value(params.clone())?;
    let graph_path = {
        let s = state.lock().unwrap();
        s.graph_path(&p.name)
    };
    let result = query::impact(&graph_path, &p.spec_section)?;
    Ok(serde_json::to_value(result)?)
}

#[derive(serde::Deserialize)]
struct LensOverviewParams {
    project: String,
    #[serde(default = "default_format")]
    format: String,
}

#[derive(serde::Deserialize)]
struct LensNodeParams {
    project: String,
    node_id: String,
    #[serde(default = "default_format")]
    format: String,
}

#[derive(serde::Deserialize)]
struct LensFocusParams {
    project: String,
    node_id: String,
    #[serde(default = "default_depth")]
    depth: u8,
    #[serde(default)]
    include_semantic: bool,
    #[serde(default = "default_format")]
    format: String,
}

fn default_format() -> String { "json".to_string() }
fn default_depth() -> u8 { 1 }

fn render_lens(view: cgdb_core::LensView, format: &str) -> Value {
    match format {
        "mermaid" => json!({ "mermaid": view.to_mermaid() }),
        _ => view.to_json(),
    }
}

fn lens_overview(state: &Arc<Mutex<DaemonState>>, params: &Value) -> Result<Value> {
    let p: LensOverviewParams = serde_json::from_value(params.clone())?;
    let graph_path = state.lock().unwrap().graph_path(&p.project);
    let view = lens_service::overview(&graph_path, &p.project)?;
    Ok(render_lens(view, &p.format))
}

fn lens_zoom_in(state: &Arc<Mutex<DaemonState>>, params: &Value) -> Result<Value> {
    let p: LensNodeParams = serde_json::from_value(params.clone())?;
    let graph_path = state.lock().unwrap().graph_path(&p.project);
    let view = lens_service::zoom_in(&graph_path, &p.node_id)?;
    Ok(render_lens(view, &p.format))
}

fn lens_zoom_out(state: &Arc<Mutex<DaemonState>>, params: &Value) -> Result<Value> {
    let p: LensNodeParams = serde_json::from_value(params.clone())?;
    let graph_path = state.lock().unwrap().graph_path(&p.project);
    let view = lens_service::zoom_out(&graph_path, &p.node_id)?;
    Ok(render_lens(view, &p.format))
}

fn lens_focus(state: &Arc<Mutex<DaemonState>>, params: &Value) -> Result<Value> {
    let p: LensFocusParams = serde_json::from_value(params.clone())?;
    if !(1..=3).contains(&p.depth) {
        return Err(anyhow!("depth must be in [1,3], got {}", p.depth));
    }
    let graph_path = state.lock().unwrap().graph_path(&p.project);
    let view = lens_service::focus(&graph_path, &p.node_id, p.depth, p.include_semantic)?;
    Ok(render_lens(view, &p.format))
}

fn lens_breadcrumb(state: &Arc<Mutex<DaemonState>>, params: &Value) -> Result<Value> {
    let p: LensNodeParams = serde_json::from_value(params.clone())?;
    let graph_path = state.lock().unwrap().graph_path(&p.project);
    let crumb = lens_service::breadcrumb(&graph_path, &p.node_id)?;
    Ok(serde_json::to_value(crumb)?)
}
// HANDWRITE-END
// SPEC-MANAGED: .score/tech_design/projects/cgdb/specs/cgdb-v0-1.md#changes
