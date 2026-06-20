// SPEC-MANAGED: projects/vat/tech-design/semantic/vat-src.md#schema
// CODEGEN-BEGIN
//! Built-in Cloud Tasks emulator — an axum REST server for the Cloud Tasks v2
//! API over in-memory state. Cloud Tasks has no official Google emulator, so
//! this is vat's own. The client points its base URL at
//! `CLOUD_TASKS_EMULATOR_HOST`. On createTask the emulator schedules the task's
//! `httpRequest` to be delivered at its `scheduleTime` (immediate when absent),
//! and `tasks/{t}:run` forces delivery now — a faithful-enough local path for
//! testing task-queue producers/consumers.
//!
//! @spec projects/vat/tech-design/logic/built-in-cloud-tasks-cloud-scheduler-emulators.md#logic

use std::collections::{BTreeMap, HashMap};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use anyhow::{Context, Result};
use axum::extract::{Path, State};
use axum::routing::{get, post};
use axum::{Json, Router};
use base64::Engine;
use serde_json::{json, Value};

use super::dispatch::{dispatch_http, Oidc, Target};

#[derive(Clone)]
struct AppState {
    inner: Arc<Mutex<Store>>,
    client: reqwest::Client,
}

#[derive(Default)]
struct Store {
    queues: HashMap<String, Value>,
    tasks: HashMap<String, Value>,
    seq: u64,
}

/// Serve the Cloud Tasks emulator until the process is killed.
pub async fn serve(host_port: &str) -> Result<()> {
    let state = AppState {
        inner: Arc::new(Mutex::new(Store::default())),
        client: reqwest::Client::new(),
    };
    // Resource names carry slashes; route by the fixed v2 path depth, and treat
    // the `:run` custom verb as a literal trailing segment (split on ':').
    let app = Router::new()
        .route(
            "/v2/projects/{project}/locations/{location}/queues",
            post(create_queue).get(list_queues),
        )
        .route(
            "/v2/projects/{project}/locations/{location}/queues/{queue}",
            get(get_queue).delete(delete_queue),
        )
        .route(
            "/v2/projects/{project}/locations/{location}/queues/{queue}/tasks",
            post(create_task).get(list_tasks),
        )
        .route(
            "/v2/projects/{project}/locations/{location}/queues/{queue}/tasks/{task}",
            get(get_task).delete(delete_task).post(run_task),
        )
        .with_state(state);

    let listener = tokio::net::TcpListener::bind(host_port)
        .await
        .with_context(|| format!("bind cloud-tasks emulator on {host_port}"))?;
    axum::serve(listener, app)
        .await
        .context("serve cloud-tasks emulator")?;
    Ok(())
}

fn parent(project: &str, location: &str) -> String {
    format!("projects/{project}/locations/{location}")
}

async fn create_queue(
    State(state): State<AppState>,
    Path((project, location)): Path<(String, String)>,
    Json(req): Json<Value>,
) -> Json<Value> {
    let mut store = state.inner.lock().unwrap();
    let name = req
        .get("name")
        .and_then(Value::as_str)
        .map(str::to_string)
        .unwrap_or_else(|| {
            store.seq += 1;
            format!("{}/queues/q-{}", parent(&project, &location), store.seq)
        });
    let queue = json!({ "name": name, "state": "RUNNING" });
    store.queues.insert(name, queue.clone());
    Json(queue)
}

async fn list_queues(
    State(state): State<AppState>,
    Path((project, location)): Path<(String, String)>,
) -> Json<Value> {
    let store = state.inner.lock().unwrap();
    let prefix = format!("{}/queues/", parent(&project, &location));
    let queues: Vec<Value> = store
        .queues
        .iter()
        .filter(|(name, _)| name.starts_with(&prefix))
        .map(|(_, q)| q.clone())
        .collect();
    Json(json!({ "queues": queues }))
}

async fn get_queue(
    State(state): State<AppState>,
    Path((project, location, queue)): Path<(String, String, String)>,
) -> Json<Value> {
    let name = format!("{}/queues/{queue}", parent(&project, &location));
    let store = state.inner.lock().unwrap();
    Json(
        store
            .queues
            .get(&name)
            .cloned()
            .unwrap_or_else(|| json!({ "error": { "code": 404, "message": "queue not found" } })),
    )
}

async fn delete_queue(
    State(state): State<AppState>,
    Path((project, location, queue)): Path<(String, String, String)>,
) -> Json<Value> {
    let name = format!("{}/queues/{queue}", parent(&project, &location));
    let mut store = state.inner.lock().unwrap();
    store.queues.remove(&name);
    let task_prefix = format!("{name}/tasks/");
    store
        .tasks
        .retain(|task_name, _| !task_name.starts_with(&task_prefix));
    Json(json!({}))
}

async fn create_task(
    State(state): State<AppState>,
    Path((project, location, queue)): Path<(String, String, String)>,
    Json(req): Json<Value>,
) -> Json<Value> {
    let queue_name = format!("{}/queues/{queue}", parent(&project, &location));
    let task_in = req.get("task").cloned().unwrap_or(req);
    let (name, task) = {
        let mut store = state.inner.lock().unwrap();
        store.seq += 1;
        let name = task_in
            .get("name")
            .and_then(Value::as_str)
            .map(str::to_string)
            .unwrap_or_else(|| format!("{queue_name}/tasks/t-{}", store.seq));
        let mut task = task_in.clone();
        task["name"] = json!(name);
        store.tasks.insert(name.clone(), task.clone());
        (name, task)
    };

    // Schedule delivery: wait until scheduleTime (immediate when absent), then
    // dispatch and drop the task on a 2xx.
    let delay = schedule_delay(&task);
    let st = state.clone();
    let task_name = name.clone();
    tokio::spawn(async move {
        if !delay.is_zero() {
            tokio::time::sleep(delay).await;
        }
        deliver(&st, &task_name).await;
    });

    Json(task)
}

fn schedule_delay(task: &Value) -> Duration {
    task.get("scheduleTime")
        .and_then(Value::as_str)
        .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
        .map(|when| {
            let secs = when.timestamp() - chrono::Utc::now().timestamp();
            Duration::from_secs(secs.max(0) as u64)
        })
        .unwrap_or(Duration::ZERO)
}

/// Build a dispatch Target from a task's httpRequest, if present.
fn task_target(task: &Value) -> Option<Target> {
    let http = task.get("httpRequest")?;
    let uri = http.get("url").and_then(Value::as_str)?.to_string();
    let method = http
        .get("httpMethod")
        .and_then(Value::as_str)
        .unwrap_or("POST")
        .to_string();
    let mut headers = BTreeMap::new();
    if let Some(map) = http.get("headers").and_then(Value::as_object) {
        for (k, v) in map {
            if let Some(s) = v.as_str() {
                headers.insert(k.clone(), s.to_string());
            }
        }
    }
    // Cloud Tasks bodies are base64-encoded in the REST API.
    let body = http
        .get("body")
        .and_then(Value::as_str)
        .and_then(|b| base64::engine::general_purpose::STANDARD.decode(b).ok())
        .unwrap_or_default();
    let oidc = http.get("oidcToken").map(|o| Oidc {
        service_account_email: o
            .get("serviceAccountEmail")
            .and_then(Value::as_str)
            .unwrap_or("vat@emulator")
            .to_string(),
        audience: o
            .get("audience")
            .and_then(Value::as_str)
            .unwrap_or(&uri)
            .to_string(),
    });
    Some(Target {
        uri,
        method,
        headers,
        body,
        oidc,
    })
}

async fn deliver(state: &AppState, task_name: &str) {
    let task = { state.inner.lock().unwrap().tasks.get(task_name).cloned() };
    let Some(task) = task else { return };
    let Some(target) = task_target(&task) else {
        return;
    };
    if let Ok(code) = dispatch_http(&state.client, &target).await {
        if (200..300).contains(&code) {
            state.inner.lock().unwrap().tasks.remove(task_name);
        }
        // Local-test semantics: keep the task on a non-2xx (no retry policy).
    }
}

async fn list_tasks(
    State(state): State<AppState>,
    Path((project, location, queue)): Path<(String, String, String)>,
) -> Json<Value> {
    let prefix = format!("{}/queues/{queue}/tasks/", parent(&project, &location));
    let store = state.inner.lock().unwrap();
    let tasks: Vec<Value> = store
        .tasks
        .iter()
        .filter(|(name, _)| name.starts_with(&prefix))
        .map(|(_, t)| t.clone())
        .collect();
    Json(json!({ "tasks": tasks }))
}

/// `{task}` may be `T` (get/delete) or `T:run` (the custom run verb).
fn strip_run(task: &str) -> String {
    task.strip_suffix(":run").unwrap_or(task).to_string()
}

async fn get_task(
    State(state): State<AppState>,
    Path((project, location, queue, task)): Path<(String, String, String, String)>,
) -> Json<Value> {
    let name = format!(
        "{}/queues/{queue}/tasks/{task}",
        parent(&project, &location)
    );
    let store = state.inner.lock().unwrap();
    Json(
        store
            .tasks
            .get(&name)
            .cloned()
            .unwrap_or_else(|| json!({ "error": { "code": 404, "message": "task not found" } })),
    )
}

async fn delete_task(
    State(state): State<AppState>,
    Path((project, location, queue, task)): Path<(String, String, String, String)>,
) -> Json<Value> {
    let name = format!(
        "{}/queues/{queue}/tasks/{task}",
        parent(&project, &location)
    );
    state.inner.lock().unwrap().tasks.remove(&name);
    Json(json!({}))
}

async fn run_task(
    State(state): State<AppState>,
    Path((project, location, queue, task)): Path<(String, String, String, String)>,
) -> Json<Value> {
    let task_id = strip_run(&task);
    let name = format!(
        "{}/queues/{queue}/tasks/{task_id}",
        parent(&project, &location)
    );
    let task = state.inner.lock().unwrap().tasks.get(&name).cloned();
    deliver(&state, &name).await;
    Json(task.unwrap_or_else(|| json!({})))
}
// CODEGEN-END
