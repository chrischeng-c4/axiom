// SPEC-MANAGED: projects/vat/tech-design/semantic/source/projects-vat-src-emulator-workflows-mod-rs.md#rust-source-unit
// CODEGEN-BEGIN
//! Built-in Cloud Workflows emulator — an axum REST server for the Workflows v1
//! API over in-memory state, plus a subset interpreter ([`interp`]) that runs a
//! workflow to completion and drives `call: http.*` steps through the shared
//! dispatcher. Cloud Workflows has no official Google emulator, so this is vat's
//! own; its value is orchestrating vat's *other* local emulators (and any HTTP
//! endpoint) end to end. The client points its base URL at
//! `CLOUD_WORKFLOWS_EMULATOR_HOST`.
//!
//! @spec projects/vat/tech-design/logic/built-in-cloud-workflows-emulator.md#logic

pub mod expr;
pub mod interp;

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use anyhow::{Context, Result};
use axum::extract::{Path, State};
use axum::routing::{get, post, put};
use axum::{Json, Router};
use serde_json::{json, Value};

#[derive(Clone)]
struct AppState {
    inner: Arc<Mutex<Store>>,
    client: reqwest::Client,
}

#[derive(Default)]
struct Store {
    workflows: HashMap<String, String>, // name -> sourceContents
    executions: HashMap<String, Value>, // name -> Execution json
    seq: u64,
}

/// Serve the Cloud Workflows emulator until the process is killed.
/// @spec projects/vat/tech-design/semantic/source/projects-vat-src-emulator-workflows-mod-rs.md#source
pub async fn serve(host_port: &str) -> Result<()> {
    let state = AppState {
        inner: Arc::new(Mutex::new(Store::default())),
        client: reqwest::Client::new(),
    };
    let app = Router::new()
        .route(
            "/v1/projects/{project}/locations/{location}/workflows/{workflow}",
            put(create_workflow).post(create_workflow).get(get_workflow),
        )
        .route(
            "/v1/projects/{project}/locations/{location}/workflows/{workflow}/executions",
            post(create_execution),
        )
        .route(
            "/v1/projects/{project}/locations/{location}/workflows/{workflow}/executions/{execution}",
            get(get_execution),
        )
        .with_state(state);

    let listener = tokio::net::TcpListener::bind(host_port)
        .await
        .with_context(|| format!("bind cloud-workflows emulator on {host_port}"))?;
    axum::serve(listener, app)
        .await
        .context("serve cloud-workflows emulator")?;
    Ok(())
}

fn workflow_name(project: &str, location: &str, workflow: &str) -> String {
    format!("projects/{project}/locations/{location}/workflows/{workflow}")
}

async fn create_workflow(
    State(state): State<AppState>,
    Path((project, location, workflow)): Path<(String, String, String)>,
    Json(req): Json<Value>,
) -> Json<Value> {
    let name = workflow_name(&project, &location, &workflow);
    let source = req
        .get("sourceContents")
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_string();
    state
        .inner
        .lock()
        .unwrap()
        .workflows
        .insert(name.clone(), source.clone());
    Json(json!({ "name": name, "sourceContents": source, "state": "ACTIVE" }))
}

async fn get_workflow(
    State(state): State<AppState>,
    Path((project, location, workflow)): Path<(String, String, String)>,
) -> Json<Value> {
    let name = workflow_name(&project, &location, &workflow);
    let store = state.inner.lock().unwrap();
    match store.workflows.get(&name) {
        Some(source) => Json(json!({ "name": name, "sourceContents": source, "state": "ACTIVE" })),
        None => Json(json!({ "error": { "code": 404, "message": "workflow not found" } })),
    }
}

async fn create_execution(
    State(state): State<AppState>,
    Path((project, location, workflow)): Path<(String, String, String)>,
    Json(req): Json<Value>,
) -> Json<Value> {
    let wf_name = workflow_name(&project, &location, &workflow);
    let (source, exec_name) = {
        let mut store = state.inner.lock().unwrap();
        let Some(source) = store.workflows.get(&wf_name).cloned() else {
            return Json(json!({ "error": { "code": 404, "message": "workflow not found" } }));
        };
        store.seq += 1;
        (source, format!("{wf_name}/executions/exec-{}", store.seq))
    };

    // The execution argument is a JSON string (GCP) or an inline object.
    let argument = match req.get("argument") {
        Some(Value::String(s)) => serde_json::from_str(s).unwrap_or(Value::Null),
        Some(other) => other.clone(),
        None => Value::Null,
    };

    // Run to completion synchronously; the first GET already shows terminal state.
    let execution = match interp::run(&source, argument, &state.client).await {
        Ok(result) => json!({
            "name": exec_name,
            "state": "SUCCEEDED",
            "result": serde_json::to_string(&result).unwrap_or_default(),
            "error": null,
        }),
        Err(message) => json!({
            "name": exec_name,
            "state": "FAILED",
            "result": null,
            "error": { "message": message },
        }),
    };

    state
        .inner
        .lock()
        .unwrap()
        .executions
        .insert(exec_name, execution.clone());
    Json(execution)
}

async fn get_execution(
    State(state): State<AppState>,
    Path((project, location, workflow, execution)): Path<(String, String, String, String)>,
) -> Json<Value> {
    let name = format!(
        "{}/executions/{execution}",
        workflow_name(&project, &location, &workflow)
    );
    let store = state.inner.lock().unwrap();
    match store.executions.get(&name) {
        Some(exec) => Json(exec.clone()),
        None => Json(json!({ "error": { "code": 404, "message": "execution not found" } })),
    }
}
// CODEGEN-END
// SPEC-MANAGED: projects/vat/tech-design/logic/vat-td-ast-promote-remaining-grouped-source-units.md#rust-source-unit
// CODEGEN-BEGIN

// CODEGEN-END
