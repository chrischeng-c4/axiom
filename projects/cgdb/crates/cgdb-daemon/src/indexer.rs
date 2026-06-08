// HANDWRITE-BEGIN gap="missing-generator:hand-written:fd044651" tracker="2087" reason="project.sync indexer — walks td_path + path, extracts Spec/Code/SpecRef, honors codegen-skip header."
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::time::Instant;

use anyhow::Result;
use cgdb_core::catalog::CatalogProject;
use cgdb_core::graph::{
    EdgeRecord, EdgeSource, EdgeType, GraphAppender, GraphRecord, NodeRecord, NodeType, RegionKind,
};
use regex::Regex;
use walkdir::WalkDir;

pub struct SyncOutcome {
    pub node_count: usize,
    pub edge_count: usize,
    pub duration_ms: u128,
}

pub fn run_sync(
    project: &CatalogProject,
    graph_path: &Path,
    _rebuild: bool,
) -> Result<SyncOutcome> {
    let start = Instant::now();
    // v0: every sync truncates for byte-stable output.
    GraphAppender::truncate(graph_path)?;
    let mut writer = GraphAppender::open(graph_path)?;

    let mut node_count = 0usize;
    let mut edge_count = 0usize;
    let mut spec_id_by_key: BTreeMap<String, String> = BTreeMap::new();

    let td_root = PathBuf::from(&project.td_path);
    let mut spec_nodes: Vec<NodeRecord> = Vec::new();
    if td_root.exists() {
        for entry in WalkDir::new(&td_root).into_iter().filter_map(|e| e.ok()) {
            if !entry.file_type().is_file() {
                continue;
            }
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) != Some("md") {
                continue;
            }
            let body = match std::fs::read_to_string(path) {
                Ok(b) => b,
                Err(_) => continue,
            };
            let rel = relativize(path, &td_root);
            for (heading, level) in headings(&body) {
                let anchor = slugify(&heading);
                let key = format!("{}#{}", rel, anchor);
                let id = stable_id("Spec", &rel, &anchor);
                spec_id_by_key.insert(key, id.clone());
                spec_nodes.push(NodeRecord {
                    id,
                    node_type: NodeType::Spec,
                    file: rel.clone(),
                    anchor,
                    symbol: None,
                    region_kind: None,
                    level,
                });
            }
        }
    }
    spec_nodes.sort_by(|a, b| a.id.cmp(&b.id));
    for n in spec_nodes {
        writer.append(&GraphRecord::node(n))?;
        node_count += 1;
    }

    let src_root = PathBuf::from(&project.source_path);
    let re_pub_item =
        Regex::new(r"(?m)^\s*pub\s+(fn|struct|enum|mod)\s+([A-Za-z_][A-Za-z0-9_]*)").unwrap();
    let re_spec_doc = Regex::new(r"//\s*@spec\s+([^\s#]+)#([^\s]+)").unwrap();
    let re_spec_ref = Regex::new(r"//\s*SPEC-REF:\s*([^\s#]+)#([^\s]+)").unwrap();
    let re_codegen_begin =
        Regex::new(r#"CODEGEN-BEGIN[^\n]*?([\S]+\.md)#([^\s\"']+)"#).unwrap();
    let re_handwrite_begin =
        Regex::new(r#"HANDWRITE-BEGIN[^\n]*?([\S]+\.md)#([^\s\"']+)"#).unwrap();

    let mut code_nodes: Vec<NodeRecord> = Vec::new();
    let mut edges: Vec<EdgeRecord> = Vec::new();

    if src_root.exists() {
        for entry in WalkDir::new(&src_root).into_iter().filter_map(|e| e.ok()) {
            if !entry.file_type().is_file() {
                continue;
            }
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) != Some("rs") {
                continue;
            }
            let body = match std::fs::read_to_string(path) {
                Ok(b) => b,
                Err(_) => continue,
            };
            if has_codegen_skip(&body) {
                continue;
            }
            let rel = relativize(path, &src_root);
            let region = classify_region(&body);
            let mut file_code_ids: Vec<String> = Vec::new();
            for cap in re_pub_item.captures_iter(&body) {
                let symbol = cap.get(2).unwrap().as_str().to_string();
                let id = stable_id("Code", &rel, &symbol);
                file_code_ids.push(id.clone());
                code_nodes.push(NodeRecord {
                    id,
                    node_type: NodeType::Code,
                    file: rel.clone(),
                    anchor: symbol.clone(),
                    symbol: Some(symbol),
                    region_kind: Some(region),
                    level: 3,
                });
            }
            if let Some(code_id) = file_code_ids.first().cloned() {
                extract_edges(&body, &re_spec_doc, EdgeSource::DocComment, &code_id, &mut edges);
                extract_edges(&body, &re_spec_ref, EdgeSource::SpecRefMarker, &code_id, &mut edges);
                extract_edges(
                    &body,
                    &re_codegen_begin,
                    EdgeSource::CodegenPayload,
                    &code_id,
                    &mut edges,
                );
                extract_edges(
                    &body,
                    &re_handwrite_begin,
                    EdgeSource::HandwritePayload,
                    &code_id,
                    &mut edges,
                );
            }
        }
    }

    let mut resolved_edges: Vec<EdgeRecord> = Vec::new();
    for e in edges {
        if let Some(id) = spec_id_by_key.get(&e.to) {
            resolved_edges.push(EdgeRecord { to: id.clone(), ..e });
        } else {
            resolved_edges.push(e);
        }
    }

    code_nodes.sort_by(|a, b| a.id.cmp(&b.id));
    for n in code_nodes {
        writer.append(&GraphRecord::node(n))?;
        node_count += 1;
    }
    resolved_edges.sort_by(|a, b| (a.from.clone(), a.to.clone()).cmp(&(b.from.clone(), b.to.clone())));
    for e in resolved_edges {
        writer.append(&GraphRecord::edge(e))?;
        edge_count += 1;
    }

    Ok(SyncOutcome {
        node_count,
        edge_count,
        duration_ms: start.elapsed().as_millis(),
    })
}

fn relativize(path: &Path, root: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .to_string_lossy()
        .into_owned()
}

fn headings(md: &str) -> Vec<(String, u8)> {
    let mut out = Vec::new();
    for line in md.lines() {
        let t = line.trim_start();
        if let Some(rest) = t.strip_prefix("## ") {
            out.push((rest.trim().to_string(), 3));
        } else if let Some(rest) = t.strip_prefix("### ") {
            out.push((rest.trim().to_string(), 4));
        }
    }
    out
}

fn slugify(s: &str) -> String {
    let mut out = String::new();
    let mut prev_dash = false;
    for c in s.chars() {
        if c.is_alphanumeric() {
            out.push(c.to_ascii_lowercase());
            prev_dash = false;
        } else if !prev_dash {
            out.push('-');
            prev_dash = true;
        }
    }
    out.trim_matches('-').to_string()
}

fn has_codegen_skip(body: &str) -> bool {
    for line in body.lines() {
        let t = line.trim_start();
        if t.is_empty() || t.starts_with("#!") {
            continue;
        }
        return t.starts_with("//! @codegen-skip:");
    }
    false
}

fn classify_region(body: &str) -> RegionKind {
    if body.contains("HANDWRITE-BEGIN") {
        RegionKind::Handwrite
    } else if body.contains("CODEGEN-BEGIN") {
        RegionKind::Codegen
    } else {
        RegionKind::Plain
    }
}

fn extract_edges(
    body: &str,
    re: &Regex,
    src: EdgeSource,
    code_id: &str,
    out: &mut Vec<EdgeRecord>,
) {
    for cap in re.captures_iter(body) {
        let file = cap.get(1).map(|m| m.as_str()).unwrap_or("");
        let section = cap.get(2).map(|m| m.as_str()).unwrap_or("");
        let to_key = format!("{}#{}", file, section);
        out.push(EdgeRecord {
            from: code_id.into(),
            to: to_key,
            edge_type: EdgeType::SpecRef,
            source: src,
        });
    }
}

fn stable_id(kind: &str, file: &str, anchor: &str) -> String {
    let key = format!("{}|{}|{}", kind, file, anchor);
    let mut hash: u64 = 0xcbf29ce484222325;
    for b in key.bytes() {
        hash ^= b as u64;
        hash = hash.wrapping_mul(0x100000001b3);
    }
    format!("{}-{:016x}", kind.to_lowercase(), hash)
}
// HANDWRITE-END
// SPEC-MANAGED: .score/tech_design/projects/cgdb/specs/cgdb-v0-1.md#changes
