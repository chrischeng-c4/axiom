// HANDWRITE-BEGIN gap="missing-generator:logic:defer-http-api" tracker="#766" reason="Shared service-http/auth shell around Defer's Raft-backed domain commands."
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use axum::extract::{Extension, Path, State};
use axum::http::{header, Method, StatusCode};
use axum::middleware::from_fn_with_state;
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::{Deserialize, Serialize};
use service_auth::{AuditedRoleMapPrincipal, ReloadableRoleMapVerifier, Role};
use service_http::{ApiErr, MetricsProvider};
use utoipa::ToSchema;

use crate::dispatch::DispatchDisposition;
use crate::metrics::DeferMetrics;
use crate::{
    auth, AuthConfig, CreateTask, DeferRaft, HttpDispatcher, QueueControlState, QueuePolicy,
    SchedulerError, TaskStatus,
};

#[derive(Clone)]
pub struct AppState {
    raft: Arc<DeferRaft>,
    dispatcher: Arc<HttpDispatcher>,
    verifier: Arc<ReloadableRoleMapVerifier>,
    metrics: Arc<DeferMetrics>,
    draining: Arc<AtomicBool>,
}

impl AppState {
    pub fn new(raft: Arc<DeferRaft>, dispatcher: HttpDispatcher, auth: AuthConfig) -> Self {
        Self {
            raft,
            dispatcher: Arc::new(dispatcher),
            verifier: Arc::new(auth.verifier()),
            metrics: Arc::new(DeferMetrics::default()),
            draining: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn start_drain(&self) {
        self.draining.store(true, Ordering::SeqCst);
    }

    pub fn is_draining(&self) -> bool {
        self.draining.load(Ordering::SeqCst)
    }

    pub fn raft(&self) -> Arc<DeferRaft> {
        self.raft.clone()
    }

    pub fn verifier(&self) -> Arc<ReloadableRoleMapVerifier> {
        self.verifier.clone()
    }

    /// Run one bounded dispatcher pass across the committed queue inventory.
    /// Each queue drains at most `max_per_queue` tasks so one hot queue cannot
    /// starve later queue names in the same tick.
    pub async fn dispatch_tick(
        &self,
        max_per_queue: usize,
        max_concurrency: usize,
    ) -> anyhow::Result<usize> {
        let queues = self.raft.scheduler().lock().unwrap().queue_names();
        let mut dispatched = 0;
        for queue in queues {
            let reports = self
                .dispatcher
                .dispatch_batch(
                    &self.raft,
                    &queue,
                    chrono::Utc::now(),
                    max_per_queue,
                    max_concurrency,
                )
                .await?;
            for report in reports {
                match report.disposition {
                    DispatchDisposition::Acked => self.metrics.dispatch_acked.incr(),
                    DispatchDisposition::Retried { .. } => self.metrics.dispatch_retried.incr(),
                    DispatchDisposition::DeadLettered => self.metrics.dispatch_dead_lettered.incr(),
                    DispatchDisposition::LostOwnership => {
                        self.metrics.dispatch_lost_ownership.incr()
                    }
                }
                dispatched += 1;
            }
        }
        Ok(dispatched)
    }
}

impl service_http::ReadinessHook for AppState {
    fn is_draining(&self) -> bool {
        self.draining.load(Ordering::SeqCst)
    }
}

impl MetricsProvider for AppState {
    fn render_metrics(&self) -> String {
        self.metrics.render()
    }
}

pub fn router(state: AppState) -> Router {
    router_with_admission(state, None)
}

pub fn router_without_raft_routes(state: AppState) -> Router {
    router_without_raft_routes_with_admission(state, None)
}

pub fn router_with_admission(
    state: AppState,
    admission: Option<service_http::AdmissionController>,
) -> Router {
    router_inner(state, true, admission)
}

pub fn router_without_raft_routes_with_admission(
    state: AppState,
    admission: Option<service_http::AdmissionController>,
) -> Router {
    router_inner(state, false, admission)
}

fn router_inner(
    state: AppState,
    include_raft_routes: bool,
    admission: Option<service_http::AdmissionController>,
) -> Router {
    let verifier = state.verifier.clone();
    let data = Router::new()
        .route("/v1/queues/{queue}", get(queue_get).put(queue_put))
        .route("/v1/queues/{queue}/control", post(queue_control))
        .route("/v1/queues/{queue}/tasks", post(task_create))
        .route("/v1/queues/{queue}/tasks:batch", post(task_create_batch))
        .route(
            "/v1/queues/{queue}/tasks/{task_id}",
            get(task_status).delete(task_cancel),
        )
        .route("/v1/queues/{queue}/dispatch", post(dispatch_one))
        .route("/admin/backup", get(admin_backup))
        .route_layer(from_fn_with_state(
            verifier,
            service_auth::auth_middleware::<ReloadableRoleMapVerifier>,
        ))
        .with_state(state.clone());
    let data = match admission {
        Some(controller) => data.route_layer(from_fn_with_state(
            service_http::AdmissionMiddleware::new(controller, |request| {
                let path = request.uri().path();
                let class = if path.starts_with("/admin/") || path.ends_with("/dispatch") {
                    "defer.admin"
                } else if *request.method() == Method::GET {
                    "defer.read"
                } else {
                    "defer.write"
                };
                let key = request
                    .headers()
                    .get(header::AUTHORIZATION)
                    .map(|value| value.as_bytes())
                    .unwrap_or(b"anonymous");
                Some(service_http::AdmissionInput::new(class, key))
            }),
            service_http::admission_middleware,
        )),
        None => data,
    };

    let probe_state = Arc::new(state.clone());
    let metrics: Arc<dyn MetricsProvider> = probe_state.clone();
    let app =
        service_http::standard_probe_routes(probe_state, Some(metrics), crate::openapi::openapi)
            .merge(data)
            .layer(service_http::trace_layer());
    if include_raft_routes {
        app.merge(state.raft.router())
    } else {
        app
    }
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct QueueControlRequest {
    pub state: QueueControlState,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateTasksRequest {
    pub tasks: Vec<CreateTask>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct CreateTasksResponse {
    pub created: usize,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct TaskStatusResponse {
    pub task_id: String,
    pub status: TaskStatus,
}

fn authorize(
    principal: &AuditedRoleMapPrincipal,
    queue: &str,
    role: Role,
) -> Result<(), service_auth::AuthError> {
    auth::authorize(principal, queue, role)
}

fn scheduler_error(error: SchedulerError) -> Response {
    let (status, kind) = match error {
        SchedulerError::QueueMissing(_) | SchedulerError::TaskMissing(_) => {
            (StatusCode::NOT_FOUND, "not_found")
        }
        SchedulerError::TaskExists(_) => (StatusCode::CONFLICT, "conflict"),
        SchedulerError::QueueDisabled(_) => (StatusCode::CONFLICT, "queue_disabled"),
        SchedulerError::AttemptMissing(_) => (StatusCode::NOT_FOUND, "not_found"),
    };
    ApiErr::new(status, kind, error.to_string()).into_response()
}

fn internal(error: impl std::fmt::Display) -> Response {
    ApiErr::new(
        StatusCode::INTERNAL_SERVER_ERROR,
        "internal",
        error.to_string(),
    )
    .into_response()
}

#[utoipa::path(
    get,
    path = "/v1/queues/{queue}",
    params(("queue" = String, Path, description = "Queue name")),
    responses((status = 200, body = QueueSnapshot), (status = 404, body = service_http::ErrorEnvelope))
)]
pub async fn queue_get(
    State(state): State<AppState>,
    Extension(principal): Extension<AuditedRoleMapPrincipal>,
    Path(queue): Path<String>,
) -> Response {
    if let Err(error) = authorize(&principal, &queue, Role::Read) {
        return error.into_response();
    }
    state.metrics.requests.incr();
    match state
        .raft
        .scheduler()
        .lock()
        .unwrap()
        .queue_snapshot(&queue)
    {
        Ok(snapshot) => Json(snapshot).into_response(),
        Err(error) => scheduler_error(error),
    }
}

#[utoipa::path(
    put,
    path = "/v1/queues/{queue}",
    params(("queue" = String, Path, description = "Queue name")),
    request_body = QueuePolicy,
    responses((status = 200, body = QueueSnapshot), (status = 400, body = service_http::ErrorEnvelope))
)]
pub async fn queue_put(
    State(state): State<AppState>,
    Extension(principal): Extension<AuditedRoleMapPrincipal>,
    Path(queue): Path<String>,
    Json(policy): Json<QueuePolicy>,
) -> Response {
    if let Err(error) = authorize(&principal, &queue, Role::Write) {
        return error.into_response();
    }
    state.metrics.requests.incr();
    match state.raft.configure_queue(queue, policy).await {
        Ok(snapshot) => Json(snapshot).into_response(),
        Err(error) => internal(error),
    }
}

#[utoipa::path(
    post,
    path = "/v1/queues/{queue}/control",
    params(("queue" = String, Path, description = "Queue name")),
    request_body = QueueControlRequest,
    responses((status = 200, body = QueueSnapshot))
)]
pub async fn queue_control(
    State(state): State<AppState>,
    Extension(principal): Extension<AuditedRoleMapPrincipal>,
    Path(queue): Path<String>,
    Json(request): Json<QueueControlRequest>,
) -> Response {
    if let Err(error) = authorize(&principal, &queue, Role::Write) {
        return error.into_response();
    }
    state.metrics.requests.incr();
    match state.raft.set_queue_control(queue, request.state).await {
        Ok(snapshot) => Json(snapshot).into_response(),
        Err(error) => internal(error),
    }
}

#[utoipa::path(
    post,
    path = "/v1/queues/{queue}/tasks",
    params(("queue" = String, Path, description = "Queue name")),
    request_body = CreateTask,
    responses((status = 201), (status = 409, body = service_http::ErrorEnvelope))
)]
pub async fn task_create(
    State(state): State<AppState>,
    Extension(principal): Extension<AuditedRoleMapPrincipal>,
    Path(queue): Path<String>,
    Json(task): Json<CreateTask>,
) -> Response {
    if let Err(error) = authorize(&principal, &queue, Role::Write) {
        return error.into_response();
    }
    state.metrics.requests.incr();
    match state.raft.create_task(queue, task).await {
        Ok(()) => StatusCode::CREATED.into_response(),
        Err(error) => internal(error),
    }
}

#[utoipa::path(
    post,
    path = "/v1/queues/{queue}/tasks:batch",
    params(("queue" = String, Path, description = "Queue name")),
    request_body = CreateTasksRequest,
    responses((status = 201, body = CreateTasksResponse), (status = 409, body = service_http::ErrorEnvelope))
)]
pub async fn task_create_batch(
    State(state): State<AppState>,
    Extension(principal): Extension<AuditedRoleMapPrincipal>,
    Path(queue): Path<String>,
    Json(request): Json<CreateTasksRequest>,
) -> Response {
    if let Err(error) = authorize(&principal, &queue, Role::Write) {
        return error.into_response();
    }
    state.metrics.requests.incr();
    match state.raft.create_tasks(queue, request.tasks).await {
        Ok(created) => (StatusCode::CREATED, Json(CreateTasksResponse { created })).into_response(),
        Err(error) => internal(error),
    }
}

#[utoipa::path(
    get,
    path = "/v1/queues/{queue}/tasks/{task_id}",
    params(
        ("queue" = String, Path, description = "Queue name"),
        ("task_id" = String, Path, description = "Task id")
    ),
    responses((status = 200, body = TaskStatusResponse), (status = 404, body = service_http::ErrorEnvelope))
)]
pub async fn task_status(
    State(state): State<AppState>,
    Extension(principal): Extension<AuditedRoleMapPrincipal>,
    Path((queue, task_id)): Path<(String, String)>,
) -> Response {
    if let Err(error) = authorize(&principal, &queue, Role::Read) {
        return error.into_response();
    }
    state.metrics.requests.incr();
    match state
        .raft
        .scheduler()
        .lock()
        .unwrap()
        .status(&queue, &task_id)
    {
        Ok(Some(status)) => Json(TaskStatusResponse { task_id, status }).into_response(),
        Ok(None) => {
            ApiErr::new(StatusCode::NOT_FOUND, "not_found", "task not found").into_response()
        }
        Err(error) => scheduler_error(error),
    }
}

#[utoipa::path(
    delete,
    path = "/v1/queues/{queue}/tasks/{task_id}",
    params(
        ("queue" = String, Path, description = "Queue name"),
        ("task_id" = String, Path, description = "Task id")
    ),
    responses((status = 204), (status = 404, body = service_http::ErrorEnvelope))
)]
pub async fn task_cancel(
    State(state): State<AppState>,
    Extension(principal): Extension<AuditedRoleMapPrincipal>,
    Path((queue, task_id)): Path<(String, String)>,
) -> Response {
    if let Err(error) = authorize(&principal, &queue, Role::Write) {
        return error.into_response();
    }
    state.metrics.requests.incr();
    match state.raft.cancel(queue, task_id).await {
        Ok(true) => StatusCode::NO_CONTENT.into_response(),
        Ok(false) => ApiErr::new(
            StatusCode::NOT_FOUND,
            "not_found",
            "task is missing or terminal",
        )
        .into_response(),
        Err(error) => internal(error),
    }
}

#[utoipa::path(
    post,
    path = "/v1/queues/{queue}/dispatch",
    params(("queue" = String, Path, description = "Queue name")),
    responses((status = 200, body = crate::DispatchReport), (status = 204))
)]
pub async fn dispatch_one(
    State(state): State<AppState>,
    Extension(principal): Extension<AuditedRoleMapPrincipal>,
    Path(queue): Path<String>,
) -> Response {
    if let Err(error) = authorize(&principal, &queue, Role::Admin) {
        return error.into_response();
    }
    state.metrics.requests.incr();
    match state
        .dispatcher
        .dispatch_one(&state.raft, &queue, chrono::Utc::now())
        .await
    {
        Ok(Some(report)) => {
            match report.disposition {
                DispatchDisposition::Acked => state.metrics.dispatch_acked.incr(),
                DispatchDisposition::Retried { .. } => state.metrics.dispatch_retried.incr(),
                DispatchDisposition::DeadLettered => state.metrics.dispatch_dead_lettered.incr(),
                DispatchDisposition::LostOwnership => state.metrics.dispatch_lost_ownership.incr(),
            }
            Json(report).into_response()
        }
        Ok(None) => StatusCode::NO_CONTENT.into_response(),
        Err(error) => internal(error),
    }
}

#[utoipa::path(
    get,
    path = "/admin/backup",
    responses((status = 200, description = "Exact Defer state-machine snapshot bytes"))
)]
pub async fn admin_backup(
    State(state): State<AppState>,
    Extension(principal): Extension<AuditedRoleMapPrincipal>,
) -> Response {
    if let Err(error) = authorize(&principal, "*", Role::Admin) {
        return error.into_response();
    }
    match state.raft.snapshot_bytes() {
        Ok(bytes) => ([(header::CONTENT_TYPE, "application/octet-stream")], bytes).into_response(),
        Err(error) => internal(error),
    }
}
// HANDWRITE-END
