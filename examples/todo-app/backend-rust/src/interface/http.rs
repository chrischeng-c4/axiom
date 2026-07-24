use axum::{
    extract::{Path, State}, http::{header, StatusCode}, response::{IntoResponse, Response}, routing::{get, patch}, Json, Router,
};
use serde::Serialize;

use crate::{application::todo_service::TodoService, domain::todo::{CreateTodo, Todo, TodoError, UpdateTodo}};

const INDEX_HTML: &[u8] = include_bytes!("../../../frontend/index.html");
const APP_JS: &[u8] = include_bytes!("../../../frontend/app.js");
const APP_CSS: &[u8] = include_bytes!("../../../frontend/app.css");

#[derive(Clone)]
pub struct AppState { pub todos: TodoService }

#[derive(Serialize)]
struct ErrorResponse<'a> { error: &'a str }

pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/api/health", get(health))
        .route("/api/todos", get(list_todos).post(create_todo))
        .route("/api/todos/:id", patch(update_todo).delete(delete_todo))
        .route("/", get(index))
        .route("/index.html", get(index))
        .route("/app.js", get(app_js))
        .route("/app.css", get(app_css))
        .fallback(not_found)
        .with_state(state)
}

async fn health() -> Json<serde_json::Value> { Json(serde_json::json!({"status": "ok"})) }
async fn list_todos(State(state): State<AppState>) -> Result<Json<serde_json::Value>, ApiError> {
    Ok(Json(serde_json::json!({"todos": state.todos.list()?})))
}
async fn create_todo(State(state): State<AppState>, Json(input): Json<CreateTodo>) -> Result<(StatusCode, Json<serde_json::Value>), ApiError> {
    Ok((StatusCode::CREATED, Json(serde_json::json!({"todo": state.todos.create(input)?}))))
}
async fn update_todo(Path(id): Path<i64>, State(state): State<AppState>, Json(input): Json<UpdateTodo>) -> Result<Json<serde_json::Value>, ApiError> {
    Ok(Json(serde_json::json!({"todo": state.todos.update(id, input)?})))
}
async fn delete_todo(Path(id): Path<i64>, State(state): State<AppState>) -> Result<StatusCode, ApiError> { state.todos.delete(id)?; Ok(StatusCode::NO_CONTENT) }
async fn index() -> Response { static_response("text/html; charset=utf-8", INDEX_HTML) }
async fn app_js() -> Response { static_response("text/javascript; charset=utf-8", APP_JS) }
async fn app_css() -> Response { static_response("text/css; charset=utf-8", APP_CSS) }
async fn not_found() -> Response { (StatusCode::NOT_FOUND, Json(ErrorResponse { error: "route not found" })).into_response() }
fn static_response(content_type: &'static str, bytes: &'static [u8]) -> Response { ([(header::CONTENT_TYPE, content_type)], bytes).into_response() }

struct ApiError(TodoError);
impl From<TodoError> for ApiError { fn from(error: TodoError) -> Self { Self(error) } }
impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, message) = match self.0 {
            TodoError::Validation(message) => (StatusCode::UNPROCESSABLE_ENTITY, message),
            TodoError::NotFound(id) => (StatusCode::NOT_FOUND, format!("todo {id} was not found")),
            TodoError::Storage(message) => (StatusCode::INTERNAL_SERVER_ERROR, message),
        };
        (status, Json(serde_json::json!({"error": message}))).into_response()
    }
}

#[allow(dead_code)]
fn _todo_shape(_: Todo) {}
