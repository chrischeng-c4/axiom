---
id: projects-vat-src-emulator-tasks-rs
summary: >
  rust-source-unit TD AST payload for projects/vat/src/emulator/tasks.rs.
fill_sections: [overview, source, changes]
capability_refs:
  - id: agent-native-gpu-native-dev-containers
    role: primary
    claim: local-agent-test-runner-protocol
    coverage: partial
    rationale: "This rust-source-unit TD preserves vat source ownership while migrating #39 off group-level source replay."
---

# Standardized projects/vat/src/emulator/tasks.rs

## Overview
<!-- type: overview lang: markdown -->

Rust source-unit TD for `projects/vat/src/emulator/tasks.rs`, captured during #39 vat migration onto td_ast lossless source generation.

## Source
<!-- type: rust-source-unit lang: rust -->

````rust
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

use anyhow::Result;
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
        .with_state(state.clone());

    // Serve the Cloud Tasks v2 gRPC service on the same port as the REST API
    // (the stock SDK clients default to gRPC). Both share `state`.
    let grpc = pb::cloud_tasks_server::CloudTasksServer::new(TasksGrpc { state });
    super::grpc_mux::serve(host_port, app, grpc).await
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

// ---- gRPC front-end: google.cloud.tasks.v2.CloudTasks over the same store ----

use super::googleapis::google::cloud::tasks::v2 as pb;
use tonic::{Request, Response, Status};

/// gRPC service backed by the SAME in-memory store + dispatcher as the REST
/// surface, so a task created over either protocol is stored and delivered
/// identically.
#[derive(Clone)]
struct TasksGrpc {
    state: AppState,
}

fn method_to_str(m: i32) -> &'static str {
    match pb::HttpMethod::try_from(m).unwrap_or(pb::HttpMethod::Post) {
        pb::HttpMethod::Get => "GET",
        pb::HttpMethod::Head => "HEAD",
        pb::HttpMethod::Put => "PUT",
        pb::HttpMethod::Delete => "DELETE",
        pb::HttpMethod::Patch => "PATCH",
        pb::HttpMethod::Options => "OPTIONS",
        _ => "POST",
    }
}

fn method_from_str(s: &str) -> i32 {
    let m = match s.to_ascii_uppercase().as_str() {
        "GET" => pb::HttpMethod::Get,
        "HEAD" => pb::HttpMethod::Head,
        "PUT" => pb::HttpMethod::Put,
        "DELETE" => pb::HttpMethod::Delete,
        "PATCH" => pb::HttpMethod::Patch,
        "OPTIONS" => pb::HttpMethod::Options,
        _ => pb::HttpMethod::Post,
    };
    m as i32
}

fn ts_to_rfc3339(ts: &prost_types::Timestamp) -> String {
    chrono::DateTime::from_timestamp(ts.seconds, ts.nanos.max(0) as u32)
        .unwrap_or_else(chrono::Utc::now)
        .to_rfc3339()
}

fn rfc3339_to_ts(s: &str) -> Option<prost_types::Timestamp> {
    let dt = chrono::DateTime::parse_from_rfc3339(s).ok()?;
    Some(prost_types::Timestamp {
        seconds: dt.timestamp(),
        nanos: dt.timestamp_subsec_nanos() as i32,
    })
}

/// Proto `Task` (httpRequest only) -> the JSON shape the shared store +
/// `task_target` expect (body base64-encoded, like the REST API).
fn task_proto_to_json(task: &pb::Task, name: &str) -> Value {
    let mut j = json!({ "name": name });
    if let Some(pb::task::MessageType::HttpRequest(h)) = &task.message_type {
        let mut http = json!({
            "url": h.url,
            "httpMethod": method_to_str(h.http_method),
        });
        if !h.headers.is_empty() {
            http["headers"] = json!(h.headers);
        }
        if !h.body.is_empty() {
            http["body"] = json!(base64::engine::general_purpose::STANDARD.encode(&h.body));
        }
        match &h.authorization_header {
            Some(pb::http_request::AuthorizationHeader::OidcToken(o)) => {
                http["oidcToken"] = json!({
                    "serviceAccountEmail": o.service_account_email,
                    "audience": o.audience,
                });
            }
            Some(pb::http_request::AuthorizationHeader::OauthToken(o)) => {
                http["oauthToken"] = json!({
                    "serviceAccountEmail": o.service_account_email,
                    "scope": o.scope,
                });
            }
            None => {}
        }
        j["httpRequest"] = http;
    }
    if let Some(ts) = &task.schedule_time {
        j["scheduleTime"] = json!(ts_to_rfc3339(ts));
    }
    j
}

/// Stored JSON task -> proto `Task` for gRPC responses.
fn task_json_to_proto(j: &Value) -> pb::Task {
    let mut task = pb::Task {
        name: j
            .get("name")
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_string(),
        ..Default::default()
    };
    if let Some(s) = j.get("scheduleTime").and_then(Value::as_str) {
        task.schedule_time = rfc3339_to_ts(s);
    }
    if let Some(http) = j.get("httpRequest") {
        let mut headers = std::collections::HashMap::new();
        if let Some(map) = http.get("headers").and_then(Value::as_object) {
            for (k, v) in map {
                if let Some(s) = v.as_str() {
                    headers.insert(k.clone(), s.to_string());
                }
            }
        }
        let body = http
            .get("body")
            .and_then(Value::as_str)
            .and_then(|b| base64::engine::general_purpose::STANDARD.decode(b).ok())
            .unwrap_or_default();
        let authorization_header = http.get("oidcToken").map(|o| {
            pb::http_request::AuthorizationHeader::OidcToken(pb::OidcToken {
                service_account_email: o
                    .get("serviceAccountEmail")
                    .and_then(Value::as_str)
                    .unwrap_or_default()
                    .to_string(),
                audience: o
                    .get("audience")
                    .and_then(Value::as_str)
                    .unwrap_or_default()
                    .to_string(),
            })
        });
        task.message_type = Some(pb::task::MessageType::HttpRequest(pb::HttpRequest {
            url: http
                .get("url")
                .and_then(Value::as_str)
                .unwrap_or("")
                .to_string(),
            http_method: http
                .get("httpMethod")
                .and_then(Value::as_str)
                .map(method_from_str)
                .unwrap_or(pb::HttpMethod::Post as i32),
            headers,
            body,
            authorization_header,
        }));
    }
    task
}

fn queue_json_to_proto(j: &Value) -> pb::Queue {
    pb::Queue {
        name: j
            .get("name")
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_string(),
        state: pb::queue::State::Running as i32,
        ..Default::default()
    }
}

#[tonic::async_trait]
impl pb::cloud_tasks_server::CloudTasks for TasksGrpc {
    async fn create_queue(
        &self,
        request: Request<pb::CreateQueueRequest>,
    ) -> Result<Response<pb::Queue>, Status> {
        let req = request.into_inner();
        let queue = req.queue.unwrap_or_default();
        let name = if queue.name.is_empty() {
            let mut store = self.state.inner.lock().unwrap();
            store.seq += 1;
            format!("{}/queues/q-{}", req.parent, store.seq)
        } else {
            queue.name.clone()
        };
        let json = json!({ "name": name, "state": "RUNNING" });
        self.state
            .inner
            .lock()
            .unwrap()
            .queues
            .insert(name.clone(), json.clone());
        Ok(Response::new(queue_json_to_proto(&json)))
    }

    async fn get_queue(
        &self,
        request: Request<pb::GetQueueRequest>,
    ) -> Result<Response<pb::Queue>, Status> {
        let name = request.into_inner().name;
        let json = self.state.inner.lock().unwrap().queues.get(&name).cloned();
        match json {
            Some(j) => Ok(Response::new(queue_json_to_proto(&j))),
            None => Err(Status::not_found(format!("queue {name} not found"))),
        }
    }

    async fn list_queues(
        &self,
        request: Request<pb::ListQueuesRequest>,
    ) -> Result<Response<pb::ListQueuesResponse>, Status> {
        let prefix = format!("{}/queues/", request.into_inner().parent);
        let queues = self
            .state
            .inner
            .lock()
            .unwrap()
            .queues
            .iter()
            .filter(|(name, _)| name.starts_with(&prefix))
            .map(|(_, j)| queue_json_to_proto(j))
            .collect();
        Ok(Response::new(pb::ListQueuesResponse {
            queues,
            next_page_token: String::new(),
        }))
    }

    async fn delete_queue(
        &self,
        request: Request<pb::DeleteQueueRequest>,
    ) -> Result<Response<()>, Status> {
        let name = request.into_inner().name;
        let mut store = self.state.inner.lock().unwrap();
        store.queues.remove(&name);
        let task_prefix = format!("{name}/tasks/");
        store.tasks.retain(|t, _| !t.starts_with(&task_prefix));
        Ok(Response::new(()))
    }

    async fn create_task(
        &self,
        request: Request<pb::CreateTaskRequest>,
    ) -> Result<Response<pb::Task>, Status> {
        let req = request.into_inner();
        let mut task = req.task.unwrap_or_default();
        let name = {
            let mut store = self.state.inner.lock().unwrap();
            store.seq += 1;
            if task.name.is_empty() {
                format!("{}/tasks/t-{}", req.parent, store.seq)
            } else {
                task.name.clone()
            }
        };
        task.name = name.clone();
        let json = task_proto_to_json(&task, &name);
        self.state
            .inner
            .lock()
            .unwrap()
            .tasks
            .insert(name.clone(), json.clone());

        // Same delivery path as REST createTask: wait until scheduleTime, deliver.
        let delay = schedule_delay(&json);
        let st = self.state.clone();
        let task_name = name.clone();
        tokio::spawn(async move {
            if !delay.is_zero() {
                tokio::time::sleep(delay).await;
            }
            deliver(&st, &task_name).await;
        });

        Ok(Response::new(task))
    }

    async fn get_task(
        &self,
        request: Request<pb::GetTaskRequest>,
    ) -> Result<Response<pb::Task>, Status> {
        let name = request.into_inner().name;
        let json = self.state.inner.lock().unwrap().tasks.get(&name).cloned();
        match json {
            Some(j) => Ok(Response::new(task_json_to_proto(&j))),
            None => Err(Status::not_found(format!("task {name} not found"))),
        }
    }

    async fn list_tasks(
        &self,
        request: Request<pb::ListTasksRequest>,
    ) -> Result<Response<pb::ListTasksResponse>, Status> {
        let prefix = format!("{}/tasks/", request.into_inner().parent);
        let tasks = self
            .state
            .inner
            .lock()
            .unwrap()
            .tasks
            .iter()
            .filter(|(name, _)| name.starts_with(&prefix))
            .map(|(_, j)| task_json_to_proto(j))
            .collect();
        Ok(Response::new(pb::ListTasksResponse {
            tasks,
            next_page_token: String::new(),
        }))
    }

    async fn delete_task(
        &self,
        request: Request<pb::DeleteTaskRequest>,
    ) -> Result<Response<()>, Status> {
        let name = request.into_inner().name;
        self.state.inner.lock().unwrap().tasks.remove(&name);
        Ok(Response::new(()))
    }

    async fn run_task(
        &self,
        request: Request<pb::RunTaskRequest>,
    ) -> Result<Response<pb::Task>, Status> {
        let name = request.into_inner().name;
        let json = self.state.inner.lock().unwrap().tasks.get(&name).cloned();
        let Some(json) = json else {
            return Err(Status::not_found(format!("task {name} not found")));
        };
        deliver(&self.state, &name).await;
        Ok(Response::new(task_json_to_proto(&json)))
    }

    async fn update_queue(
        &self,
        _request: Request<pb::UpdateQueueRequest>,
    ) -> Result<Response<pb::Queue>, Status> {
        Err(Status::unimplemented(
            "UpdateQueue is not supported by the vat emulator",
        ))
    }

    async fn purge_queue(
        &self,
        _request: Request<pb::PurgeQueueRequest>,
    ) -> Result<Response<pb::Queue>, Status> {
        Err(Status::unimplemented(
            "PurgeQueue is not supported by the vat emulator",
        ))
    }

    async fn pause_queue(
        &self,
        _request: Request<pb::PauseQueueRequest>,
    ) -> Result<Response<pb::Queue>, Status> {
        Err(Status::unimplemented(
            "PauseQueue is not supported by the vat emulator",
        ))
    }

    async fn resume_queue(
        &self,
        _request: Request<pb::ResumeQueueRequest>,
    ) -> Result<Response<pb::Queue>, Status> {
        Err(Status::unimplemented(
            "ResumeQueue is not supported by the vat emulator",
        ))
    }

    async fn get_iam_policy(
        &self,
        _request: Request<super::googleapis::google::iam::v1::GetIamPolicyRequest>,
    ) -> Result<Response<super::googleapis::google::iam::v1::Policy>, Status> {
        Err(Status::unimplemented(
            "IAM is not supported by the vat emulator",
        ))
    }

    async fn set_iam_policy(
        &self,
        _request: Request<super::googleapis::google::iam::v1::SetIamPolicyRequest>,
    ) -> Result<Response<super::googleapis::google::iam::v1::Policy>, Status> {
        Err(Status::unimplemented(
            "IAM is not supported by the vat emulator",
        ))
    }

    async fn test_iam_permissions(
        &self,
        _request: Request<super::googleapis::google::iam::v1::TestIamPermissionsRequest>,
    ) -> Result<Response<super::googleapis::google::iam::v1::TestIamPermissionsResponse>, Status>
    {
        Err(Status::unimplemented(
            "IAM is not supported by the vat emulator",
        ))
    }
}
````

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/vat/src/emulator/tasks.rs
    action: modify
    section: rust-source-unit
    impl_mode: codegen
    description: |
      rust-source-unit (td_ast) source for `projects/vat/src/emulator/tasks.rs` captured during #39 vat standardization.
```
