use anyhow::{bail, Result};
use serde_json::{json, Value};

use crate::store::{get_task_context, PmStore};

pub async fn list_resources(store: &PmStore) -> Vec<Value> {
    store
        .read(|s| {
            let mut resources = Vec::new();

            for proj in s.projects.values() {
                resources.push(json!({
                    "uri": format!("pm://projects/{}", proj.id),
                    "name": format!("Project: {}", proj.name),
                    "mimeType": "application/json",
                    "description": proj.description,
                }));
            }

            for prd in s.prds.values() {
                resources.push(json!({
                    "uri": format!("pm://prds/{}", prd.id),
                    "name": format!("PRD: {} (v{})", prd.title, prd.version),
                    "mimeType": "text/markdown",
                    "description": format!("PRD for project {}", prd.project_id),
                }));
            }

            for td in s.tech_designs.values() {
                resources.push(json!({
                    "uri": format!("pm://tech-designs/{}", td.id),
                    "name": format!("Tech Design: {} (v{})", td.title, td.version),
                    "mimeType": "text/markdown",
                    "description": format!("Tech Design for project {}", td.project_id),
                }));
            }

            for task in s.tasks.values() {
                resources.push(json!({
                    "uri": format!("pm://tasks/{}", task.id),
                    "name": format!("Task: {} [{}]", task.title, task.status),
                    "mimeType": "text/markdown",
                    "description": format!("Task in project {}", task.project_id),
                }));
            }

            resources
        })
        .await
}

pub async fn read_resource(store: &PmStore, uri: &str) -> Result<Value> {
    let uri_parts: Vec<&str> = uri.trim_start_matches("pm://").split('/').collect();

    if uri_parts.len() < 2 {
        bail!("Invalid resource URI '{}'", uri);
    }

    let kind = uri_parts[0];
    let id = uri_parts[1];

    match kind {
        "projects" => {
            let proj = store.read(|s| s.get_project(id).cloned()).await;
            match proj {
                Some(p) => Ok(json!({
                    "contents": [{
                        "uri": uri,
                        "mimeType": "application/json",
                        "text": serde_json::to_string_pretty(&p)?,
                    }]
                })),
                None => bail!("Project '{}' not found", id),
            }
        }
        "prds" => {
            let prd = store.read(|s| s.get_prd(id).cloned()).await;
            match prd {
                Some(p) => Ok(json!({
                    "contents": [{
                        "uri": uri,
                        "mimeType": "text/markdown",
                        "text": p.content,
                    }]
                })),
                None => bail!("PRD '{}' not found", id),
            }
        }
        "tech-designs" => {
            let td = store.read(|s| s.get_tech_design(id).cloned()).await;
            match td {
                Some(t) => Ok(json!({
                    "contents": [{
                        "uri": uri,
                        "mimeType": "text/markdown",
                        "text": t.content,
                    }]
                })),
                None => bail!("Tech Design '{}' not found", id),
            }
        }
        "tasks" => {
            let context = store.read(|s| get_task_context(s, id)).await?;
            Ok(json!({
                "contents": [{
                    "uri": uri,
                    "mimeType": "text/markdown",
                    "text": context,
                }]
            }))
        }
        other => bail!("Unsupported resource kind '{}'", other),
    }
}
