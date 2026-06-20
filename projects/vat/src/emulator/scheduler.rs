// SPEC-MANAGED: projects/vat/tech-design/semantic/vat-src.md#schema
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

use anyhow::{Context, Result};
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
        .with_state(state);

    let listener = tokio::net::TcpListener::bind(host_port)
        .await
        .with_context(|| format!("bind cloud-scheduler emulator on {host_port}"))?;
    axum::serve(listener, app)
        .await
        .context("serve cloud-scheduler emulator")?;
    Ok(())
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
// CODEGEN-END
