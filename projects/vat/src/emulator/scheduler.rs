// SPEC-MANAGED: projects/vat/tech-design/semantic/source/projects-vat-src-emulator-scheduler-rs.md#rust-source-unit
// CODEGEN-BEGIN
//! Built-in Cloud Scheduler emulator — an axum REST server for the Cloud
//! Scheduler v1 API over in-memory state, plus a background cron ticker. Cloud
//! Scheduler has no official Google emulator, so this is vat's own. The client
//! points its base URL at `CLOUD_SCHEDULER_EMULATOR_HOST`. A job's `httpTarget`
//! fires on its cron `schedule` (a ~1s ticker evaluates each ENABLED job) and
//! immediately on `jobs/{j}:run`; `:pause` / `:resume` toggle the schedule.
//! `pubsubTarget` is a logged no-op in v1 (httpTarget is the local-test path).
//!
//! @spec projects/vat/tech-design/logic/built-in-cloud-tasks-cloud-scheduler-emulators.md#logic

use std::collections::{BTreeMap, HashMap};
use std::str::FromStr;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use anyhow::Result;
use axum::extract::{Path, State};
use axum::routing::{get, post};
use axum::{Json, Router};
use base64::Engine;
use cron::Schedule;
use serde_json::{json, Value};

use super::dispatch::{dispatch_http, Oidc, Target};

#[derive(Clone)]
struct Job {
    json: Value,
    enabled: bool,
    last_fire: i64,
}

#[derive(Clone)]
struct AppState {
    inner: Arc<Mutex<HashMap<String, Job>>>,
    client: reqwest::Client,
}

/// Serve the Cloud Scheduler emulator until the process is killed.
/// @spec projects/vat/tech-design/semantic/source/projects-vat-src-emulator-scheduler-rs.md#source
pub async fn serve(host_port: &str) -> Result<()> {
    let state = AppState {
        inner: Arc::new(Mutex::new(HashMap::new())),
        client: reqwest::Client::new(),
    };

    // Background cron ticker: fire each ENABLED job whose next scheduled time has
    // passed since the last fire.
    let ticker = state.clone();
    tokio::spawn(async move {
        loop {
            tokio::time::sleep(Duration::from_secs(1)).await;
            tick(&ticker).await;
        }
    });

    let app = Router::new()
        .route(
            "/v1/projects/{project}/locations/{location}/jobs",
            post(create_job).get(list_jobs),
        )
        .route(
            "/v1/projects/{project}/locations/{location}/jobs/{job}",
            get(get_job).delete(delete_job).post(job_action),
        )
        .with_state(state.clone());

    // Serve the Cloud Scheduler v1 gRPC service on the same port as the REST API
    // (the stock SDK clients default to gRPC). Both share `state`.
    let grpc = pb::cloud_scheduler_server::CloudSchedulerServer::new(SchedulerGrpc { state });
    super::grpc_mux::serve(host_port, app, grpc).await
}

fn parent(project: &str, location: &str) -> String {
    format!("projects/{project}/locations/{location}")
}

async fn create_job(
    State(state): State<AppState>,
    Path((project, location)): Path<(String, String)>,
    Json(req): Json<Value>,
) -> Json<Value> {
    let mut jobs = state.inner.lock().unwrap();
    let name = req
        .get("name")
        .and_then(Value::as_str)
        .map(str::to_string)
        .unwrap_or_else(|| {
            format!(
                "{}/jobs/job-{}",
                parent(&project, &location),
                jobs.len() + 1
            )
        });
    let mut job_json = req.clone();
    job_json["name"] = json!(name);
    job_json["state"] = json!("ENABLED");
    jobs.insert(
        name.clone(),
        Job {
            json: job_json.clone(),
            enabled: true,
            last_fire: chrono::Utc::now().timestamp(),
        },
    );
    Json(job_json)
}

async fn list_jobs(
    State(state): State<AppState>,
    Path((project, location)): Path<(String, String)>,
) -> Json<Value> {
    let prefix = format!("{}/jobs/", parent(&project, &location));
    let jobs = state.inner.lock().unwrap();
    let list: Vec<Value> = jobs
        .iter()
        .filter(|(name, _)| name.starts_with(&prefix))
        .map(|(_, j)| j.json.clone())
        .collect();
    Json(json!({ "jobs": list }))
}

async fn get_job(
    State(state): State<AppState>,
    Path((project, location, job)): Path<(String, String, String)>,
) -> Json<Value> {
    let name = format!("{}/jobs/{job}", parent(&project, &location));
    let jobs = state.inner.lock().unwrap();
    Json(
        jobs.get(&name)
            .map(|j| j.json.clone())
            .unwrap_or_else(|| json!({ "error": { "code": 404, "message": "job not found" } })),
    )
}

async fn delete_job(
    State(state): State<AppState>,
    Path((project, location, job)): Path<(String, String, String)>,
) -> Json<Value> {
    let name = format!("{}/jobs/{job}", parent(&project, &location));
    state.inner.lock().unwrap().remove(&name);
    Json(json!({}))
}

/// `{job}` may be `J:run` / `J:pause` / `J:resume`.
async fn job_action(
    State(state): State<AppState>,
    Path((project, location, job)): Path<(String, String, String)>,
) -> Json<Value> {
    let (job_id, action) = match job.rsplit_once(':') {
        Some((id, verb)) => (id.to_string(), verb.to_string()),
        None => (job.clone(), String::new()),
    };
    let name = format!("{}/jobs/{job_id}", parent(&project, &location));

    match action.as_str() {
        "pause" => {
            if let Some(j) = state.inner.lock().unwrap().get_mut(&name) {
                j.enabled = false;
                j.json["state"] = json!("PAUSED");
            }
        }
        "resume" => {
            if let Some(j) = state.inner.lock().unwrap().get_mut(&name) {
                j.enabled = true;
                j.json["state"] = json!("ENABLED");
            }
        }
        "run" => {
            // Force-fire now, regardless of schedule/state.
            let json = state
                .inner
                .lock()
                .unwrap()
                .get(&name)
                .map(|j| j.json.clone());
            if let Some(json) = json {
                fire(&state, &json).await;
            }
        }
        _ => {}
    }

    let jobs = state.inner.lock().unwrap();
    Json(jobs.get(&name).map(|j| j.json.clone()).unwrap_or(json!({})))
}

/// Build a dispatch Target from a job's httpTarget, if present.
fn job_target(job: &Value) -> Option<Target> {
    let http = job.get("httpTarget")?;
    let uri = http.get("uri").and_then(Value::as_str)?.to_string();
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
    let body = http
        .get("body")
        .and_then(Value::as_str)
        .and_then(|b| base64::engine::general_purpose::STANDARD.decode(b).ok())
        .unwrap_or_default();
    let oidc = http
        .get("oidcToken")
        .or_else(|| http.get("oauthToken"))
        .map(|o| Oidc {
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

async fn fire(state: &AppState, job: &Value) {
    match job_target(job) {
        Some(target) => {
            let _ = dispatch_http(&state.client, &target).await;
        }
        None => {
            // pubsubTarget / appEngineHttpTarget: logged no-op in v1.
            eprintln!(
                "vat cloud-scheduler: job {} has no httpTarget; skipping non-http target",
                job.get("name").and_then(Value::as_str).unwrap_or("?")
            );
        }
    }
}

/// One tick: fire each ENABLED job whose cron schedule came due since last fire.
async fn tick(state: &AppState) {
    let now = chrono::Utc::now();
    let due: Vec<Value> = {
        let mut jobs = state.inner.lock().unwrap();
        let mut due = Vec::new();
        for job in jobs.values_mut() {
            if !job.enabled {
                continue;
            }
            let Some(expr) = job.json.get("schedule").and_then(Value::as_str) else {
                continue;
            };
            // cron expects 6 fields (with seconds); accept a 5-field crontab by
            // prefixing a seconds wildcard.
            let normalized = if expr.split_whitespace().count() == 5 {
                format!("0 {expr}")
            } else {
                expr.to_string()
            };
            let Ok(schedule) = Schedule::from_str(&normalized) else {
                continue;
            };
            let last = chrono::DateTime::from_timestamp(job.last_fire, 0).unwrap_or(now);
            if let Some(next) = schedule.after(&last).next() {
                if next <= now {
                    job.last_fire = now.timestamp();
                    due.push(job.json.clone());
                }
            }
        }
        due
    };
    for job in due {
        fire(state, &job).await;
    }
}

// ---- gRPC front-end: google.cloud.scheduler.v1.CloudScheduler over the store ----

use super::googleapis::google::cloud::scheduler::v1 as pb;
use tonic::{Request, Response, Status};

/// gRPC service backed by the SAME in-memory store + cron dispatcher as the REST
/// surface.
#[derive(Clone)]
struct SchedulerGrpc {
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

/// Proto `Job` (httpTarget only) -> the JSON shape the shared store + `job_target`
/// expect (body base64-encoded, like the REST API).
fn job_proto_to_json(job: &pb::Job, name: &str) -> Value {
    let mut j = json!({ "name": name, "state": "ENABLED" });
    if !job.schedule.is_empty() {
        j["schedule"] = json!(job.schedule);
    }
    if !job.time_zone.is_empty() {
        j["timeZone"] = json!(job.time_zone);
    }
    if let Some(pb::job::Target::HttpTarget(h)) = &job.target {
        let mut http = json!({
            "uri": h.uri,
            "httpMethod": method_to_str(h.http_method),
        });
        if !h.headers.is_empty() {
            http["headers"] = json!(h.headers);
        }
        if !h.body.is_empty() {
            http["body"] = json!(base64::engine::general_purpose::STANDARD.encode(&h.body));
        }
        match &h.authorization_header {
            Some(pb::http_target::AuthorizationHeader::OidcToken(o)) => {
                http["oidcToken"] = json!({
                    "serviceAccountEmail": o.service_account_email,
                    "audience": o.audience,
                });
            }
            Some(pb::http_target::AuthorizationHeader::OauthToken(o)) => {
                http["oauthToken"] = json!({
                    "serviceAccountEmail": o.service_account_email,
                    "scope": o.scope,
                });
            }
            None => {}
        }
        j["httpTarget"] = http;
    }
    j
}

/// Stored JSON job -> proto `Job` for gRPC responses.
fn job_json_to_proto(j: &Value) -> pb::Job {
    let state = match j.get("state").and_then(Value::as_str) {
        Some("PAUSED") => pb::job::State::Paused,
        _ => pb::job::State::Enabled,
    } as i32;
    let mut job = pb::Job {
        name: j
            .get("name")
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_string(),
        schedule: j
            .get("schedule")
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_string(),
        time_zone: j
            .get("timeZone")
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_string(),
        state,
        ..Default::default()
    };
    if let Some(http) = j.get("httpTarget") {
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
            pb::http_target::AuthorizationHeader::OidcToken(pb::OidcToken {
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
        job.target = Some(pb::job::Target::HttpTarget(pb::HttpTarget {
            uri: http
                .get("uri")
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
    job
}

#[tonic::async_trait]
/// @spec projects/vat/tech-design/semantic/source/projects-vat-src-emulator-scheduler-rs.md#source
impl pb::cloud_scheduler_server::CloudScheduler for SchedulerGrpc {
    async fn create_job(
        &self,
        request: Request<pb::CreateJobRequest>,
    ) -> Result<Response<pb::Job>, Status> {
        let req = request.into_inner();
        let proto = req.job.unwrap_or_default();
        let name = if proto.name.is_empty() {
            let n = self.state.inner.lock().unwrap().len() + 1;
            format!("{}/jobs/job-{}", req.parent, n)
        } else {
            proto.name.clone()
        };
        let json = job_proto_to_json(&proto, &name);
        self.state.inner.lock().unwrap().insert(
            name.clone(),
            Job {
                json: json.clone(),
                enabled: true,
                last_fire: chrono::Utc::now().timestamp(),
            },
        );
        Ok(Response::new(job_json_to_proto(&json)))
    }

    async fn get_job(
        &self,
        request: Request<pb::GetJobRequest>,
    ) -> Result<Response<pb::Job>, Status> {
        let name = request.into_inner().name;
        let json = self
            .state
            .inner
            .lock()
            .unwrap()
            .get(&name)
            .map(|j| j.json.clone());
        match json {
            Some(j) => Ok(Response::new(job_json_to_proto(&j))),
            None => Err(Status::not_found(format!("job {name} not found"))),
        }
    }

    async fn list_jobs(
        &self,
        request: Request<pb::ListJobsRequest>,
    ) -> Result<Response<pb::ListJobsResponse>, Status> {
        let prefix = format!("{}/jobs/", request.into_inner().parent);
        let jobs = self
            .state
            .inner
            .lock()
            .unwrap()
            .iter()
            .filter(|(name, _)| name.starts_with(&prefix))
            .map(|(_, j)| job_json_to_proto(&j.json))
            .collect();
        Ok(Response::new(pb::ListJobsResponse {
            jobs,
            next_page_token: String::new(),
        }))
    }

    async fn delete_job(
        &self,
        request: Request<pb::DeleteJobRequest>,
    ) -> Result<Response<()>, Status> {
        let name = request.into_inner().name;
        self.state.inner.lock().unwrap().remove(&name);
        Ok(Response::new(()))
    }

    async fn run_job(
        &self,
        request: Request<pb::RunJobRequest>,
    ) -> Result<Response<pb::Job>, Status> {
        let name = request.into_inner().name;
        let json = self
            .state
            .inner
            .lock()
            .unwrap()
            .get(&name)
            .map(|j| j.json.clone());
        let Some(json) = json else {
            return Err(Status::not_found(format!("job {name} not found")));
        };
        fire(&self.state, &json).await;
        Ok(Response::new(job_json_to_proto(&json)))
    }

    async fn pause_job(
        &self,
        request: Request<pb::PauseJobRequest>,
    ) -> Result<Response<pb::Job>, Status> {
        let name = request.into_inner().name;
        let mut jobs = self.state.inner.lock().unwrap();
        match jobs.get_mut(&name) {
            Some(j) => {
                j.enabled = false;
                j.json["state"] = json!("PAUSED");
                Ok(Response::new(job_json_to_proto(&j.json)))
            }
            None => Err(Status::not_found(format!("job {name} not found"))),
        }
    }

    async fn resume_job(
        &self,
        request: Request<pb::ResumeJobRequest>,
    ) -> Result<Response<pb::Job>, Status> {
        let name = request.into_inner().name;
        let mut jobs = self.state.inner.lock().unwrap();
        match jobs.get_mut(&name) {
            Some(j) => {
                j.enabled = true;
                j.json["state"] = json!("ENABLED");
                Ok(Response::new(job_json_to_proto(&j.json)))
            }
            None => Err(Status::not_found(format!("job {name} not found"))),
        }
    }

    async fn update_job(
        &self,
        _request: Request<pb::UpdateJobRequest>,
    ) -> Result<Response<pb::Job>, Status> {
        Err(Status::unimplemented(
            "UpdateJob is not supported by the vat emulator",
        ))
    }
}
// CODEGEN-END
// SPEC-MANAGED: projects/vat/tech-design/logic/vat-td-ast-promote-remaining-grouped-source-units.md#rust-source-unit
// CODEGEN-BEGIN

// CODEGEN-END
