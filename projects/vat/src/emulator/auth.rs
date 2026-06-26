// SPEC-MANAGED: projects/vat/tech-design/semantic/source/projects-vat-src-emulator-auth-rs.md#rust-source-unit
// CODEGEN-BEGIN
//! Built-in Firebase Auth (Identity Toolkit) emulator — a small axum REST server
//! that the Firebase client/admin SDKs hit when `FIREBASE_AUTH_EMULATOR_HOST` is
//! set. In-memory users; idTokens are HS256 JWTs (the SDKs skip signature
//! verification in emulator mode). Covers the common ops: signUp,
//! signInWithPassword, lookup, delete, secure-token refresh, plus the
//! `/emulator` config + accounts endpoints.
//!
//! @spec projects/vat/tech-design/logic/built-in-rust-emulators-pub-sub-firebase-auth.md#logic

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use anyhow::{Context, Result};
use axum::extract::{Path, State};
use axum::routing::{get, post};
use axum::{Json, Router};
use jsonwebtoken::{Algorithm, EncodingKey, Header};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

/// The project the emulator mints tokens for. Real Firebase emulator setups use
/// a `demo-*` project so no real credentials are involved.
const PROJECT: &str = "demo-vat";
const SECRET: &[u8] = b"vat-firebase-auth-emulator";

#[derive(Clone)]
struct AppState {
    inner: Arc<Mutex<Store>>,
}

#[derive(Default)]
struct Store {
    by_id: HashMap<String, User>,
    by_email: HashMap<String, String>,
    next: u64,
}

#[derive(Clone)]
struct User {
    local_id: String,
    email: String,
    password: String,
    refresh_token: String,
}

#[derive(Serialize, Deserialize)]
struct Claims {
    sub: String,
    user_id: String,
    email: String,
    email_verified: bool,
    aud: String,
    iss: String,
    auth_time: i64,
    iat: i64,
    exp: i64,
}

/// Serve the Firebase Auth emulator until the process is killed.
/// @spec projects/vat/tech-design/semantic/source/projects-vat-src-emulator-auth-rs.md#source
pub async fn serve(host_port: &str) -> Result<()> {
    let state = AppState {
        inner: Arc::new(Mutex::new(Store::default())),
    };
    let app = Router::new()
        .route(
            "/identitytoolkit.googleapis.com/v1/accounts:signUp",
            post(sign_up),
        )
        .route(
            "/identitytoolkit.googleapis.com/v1/accounts:signInWithPassword",
            post(sign_in),
        )
        .route(
            "/identitytoolkit.googleapis.com/v1/accounts:lookup",
            post(lookup),
        )
        .route(
            "/identitytoolkit.googleapis.com/v1/accounts:delete",
            post(delete_account),
        )
        .route("/securetoken.googleapis.com/v1/token", post(refresh_token))
        .route("/emulator/v1/projects/{project}/config", get(config))
        .route(
            "/emulator/v1/projects/{project}/accounts",
            get(list_accounts).delete(clear_accounts),
        )
        .route("/", get(banner))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind(host_port)
        .await
        .with_context(|| format!("bind firebase-auth emulator on {host_port}"))?;
    axum::serve(listener, app)
        .await
        .context("serve firebase-auth emulator")?;
    Ok(())
}

fn mint(user: &User) -> String {
    let now = chrono::Utc::now().timestamp();
    let claims = Claims {
        sub: user.local_id.clone(),
        user_id: user.local_id.clone(),
        email: user.email.clone(),
        email_verified: false,
        aud: PROJECT.to_string(),
        iss: format!("https://securetoken.google.com/{PROJECT}"),
        auth_time: now,
        iat: now,
        exp: now + 3600,
    };
    jsonwebtoken::encode(
        &Header::new(Algorithm::HS256),
        &claims,
        &EncodingKey::from_secret(SECRET),
    )
    .expect("mint id token")
}

fn local_id_of(id_token: &str) -> Option<String> {
    let mut validation = jsonwebtoken::Validation::new(Algorithm::HS256);
    validation.validate_aud = false;
    jsonwebtoken::decode::<Claims>(
        id_token,
        &jsonwebtoken::DecodingKey::from_secret(SECRET),
        &validation,
    )
    .ok()
    .map(|data| data.claims.sub)
}

#[derive(Deserialize)]
struct PasswordRequest {
    email: String,
    password: String,
}

fn token_response(user: &User) -> Json<Value> {
    Json(json!({
        "kind": "identitytoolkit#SignupNewUserResponse",
        "idToken": mint(user),
        "email": user.email,
        "refreshToken": user.refresh_token,
        "expiresIn": "3600",
        "localId": user.local_id,
        "registered": true,
    }))
}

async fn sign_up(State(state): State<AppState>, Json(req): Json<PasswordRequest>) -> Json<Value> {
    let mut store = state.inner.lock().unwrap();
    if let Some(local_id) = store.by_email.get(&req.email).cloned() {
        let user = store.by_id.get(&local_id).unwrap().clone();
        return token_response(&user);
    }
    store.next += 1;
    let local_id = format!("vat-uid-{}", store.next);
    let user = User {
        local_id: local_id.clone(),
        email: req.email.clone(),
        password: req.password,
        refresh_token: format!("vat-refresh-{}", store.next),
    };
    store.by_email.insert(req.email, local_id.clone());
    store.by_id.insert(local_id, user.clone());
    token_response(&user)
}

async fn sign_in(State(state): State<AppState>, Json(req): Json<PasswordRequest>) -> Json<Value> {
    let store = state.inner.lock().unwrap();
    match store.by_email.get(&req.email) {
        Some(local_id) => {
            let user = store.by_id.get(local_id).unwrap();
            if user.password == req.password {
                token_response(user)
            } else {
                Json(json!({ "error": { "code": 400, "message": "INVALID_PASSWORD" } }))
            }
        }
        None => Json(json!({ "error": { "code": 400, "message": "EMAIL_NOT_FOUND" } })),
    }
}

async fn lookup(State(state): State<AppState>, Json(req): Json<Value>) -> Json<Value> {
    let store = state.inner.lock().unwrap();
    let mut users = Vec::new();
    if let Some(token) = req.get("idToken").and_then(Value::as_str) {
        if let Some(local_id) = local_id_of(token) {
            if let Some(user) = store.by_id.get(&local_id) {
                users.push(user_json(user));
            }
        }
    }
    if let Some(ids) = req.get("localId").and_then(Value::as_array) {
        for id in ids.iter().filter_map(Value::as_str) {
            if let Some(user) = store.by_id.get(id) {
                users.push(user_json(user));
            }
        }
    }
    Json(json!({ "kind": "identitytoolkit#GetAccountInfoResponse", "users": users }))
}

fn user_json(user: &User) -> Value {
    json!({
        "localId": user.local_id,
        "email": user.email,
        "emailVerified": false,
        "passwordHash": "fakeHash",
    })
}

async fn delete_account(State(state): State<AppState>, Json(req): Json<Value>) -> Json<Value> {
    let mut store = state.inner.lock().unwrap();
    let local_id = req
        .get("localId")
        .and_then(Value::as_str)
        .map(str::to_string)
        .or_else(|| {
            req.get("idToken")
                .and_then(Value::as_str)
                .and_then(local_id_of)
        });
    if let Some(id) = local_id {
        if let Some(user) = store.by_id.remove(&id) {
            store.by_email.remove(&user.email);
        }
    }
    Json(json!({ "kind": "identitytoolkit#DeleteAccountResponse" }))
}

async fn refresh_token(State(state): State<AppState>, Json(req): Json<Value>) -> Json<Value> {
    let store = state.inner.lock().unwrap();
    let refresh = req.get("refresh_token").and_then(Value::as_str);
    let user = refresh.and_then(|token| {
        store
            .by_id
            .values()
            .find(|u| u.refresh_token == token)
            .cloned()
    });
    match user {
        Some(user) => Json(json!({
            "id_token": mint(&user),
            "access_token": mint(&user),
            "refresh_token": user.refresh_token,
            "expires_in": "3600",
            "user_id": user.local_id,
            "project_id": PROJECT,
        })),
        None => Json(json!({ "error": { "code": 400, "message": "INVALID_REFRESH_TOKEN" } })),
    }
}

async fn config(Path(_project): Path<String>) -> Json<Value> {
    Json(json!({ "signIn": { "allowDuplicateEmails": false } }))
}

async fn list_accounts(State(state): State<AppState>, Path(_project): Path<String>) -> Json<Value> {
    let store = state.inner.lock().unwrap();
    let users: Vec<Value> = store.by_id.values().map(user_json).collect();
    Json(json!({ "userInfo": users }))
}

async fn clear_accounts(
    State(state): State<AppState>,
    Path(_project): Path<String>,
) -> Json<Value> {
    let mut store = state.inner.lock().unwrap();
    *store = Store::default();
    Json(json!({}))
}

async fn banner() -> Json<Value> {
    Json(json!({ "authEmulator": { "ready": true } }))
}
// CODEGEN-END
// SPEC-MANAGED: projects/vat/tech-design/logic/vat-td-ast-promote-remaining-grouped-source-units.md#rust-source-unit
// CODEGEN-BEGIN

// CODEGEN-END
