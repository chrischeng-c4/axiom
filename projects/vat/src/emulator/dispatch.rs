// SPEC-MANAGED: projects/vat/tech-design/semantic/source/projects-vat-src-emulator-dispatch-rs.md#rust-source-unit
// CODEGEN-BEGIN
//! Shared outbound dispatcher for the Cloud Tasks / Cloud Scheduler emulators.
//!
//! Both emulators *deliver* by making an HTTP request to a task/job target. This
//! module centralizes that: a reqwest call with the configured method, headers,
//! and body, and — when the target asks for an OIDC token — a minted HS256 JWT in
//! the `Authorization: Bearer` header (the same kind of fake token the Firebase
//! Auth emulator mints; receivers in emulator/test mode do not verify it).
//!
//! @spec projects/vat/tech-design/logic/built-in-cloud-tasks-cloud-scheduler-emulators.md#logic

use std::collections::BTreeMap;

use jsonwebtoken::{Algorithm, EncodingKey, Header};
use serde::Serialize;

const SECRET: &[u8] = b"vat-cloud-emulator-oidc";

/// A target to deliver to: an HTTP request, optionally carrying a minted OIDC
/// token for the given service account / audience.
/// @spec projects/vat/tech-design/semantic/source/projects-vat-src-emulator-dispatch-rs.md#source
pub struct Target {
    pub uri: String,
    pub method: String,
    pub headers: BTreeMap<String, String>,
    pub body: Vec<u8>,
    pub oidc: Option<Oidc>,
}

/// @spec projects/vat/tech-design/semantic/source/projects-vat-src-emulator-dispatch-rs.md#source
pub struct Oidc {
    pub service_account_email: String,
    pub audience: String,
}

#[derive(Serialize)]
struct OidcClaims {
    iss: String,
    sub: String,
    email: String,
    email_verified: bool,
    aud: String,
    iat: i64,
    exp: i64,
}

/// Mint a fake OIDC id token (HS256) for the dispatcher.
fn mint_oidc(oidc: &Oidc) -> String {
    let now = chrono::Utc::now().timestamp();
    let claims = OidcClaims {
        iss: "https://vat-emulator.local".to_string(),
        sub: oidc.service_account_email.clone(),
        email: oidc.service_account_email.clone(),
        email_verified: true,
        aud: oidc.audience.clone(),
        iat: now,
        exp: now + 3600,
    };
    jsonwebtoken::encode(
        &Header::new(Algorithm::HS256),
        &claims,
        &EncodingKey::from_secret(SECRET),
    )
    .expect("mint oidc token")
}

/// The result of a dispatch: the response status and body text.
/// @spec projects/vat/tech-design/semantic/source/projects-vat-src-emulator-dispatch-rs.md#source
pub struct DispatchResult {
    pub code: u16,
    pub body: String,
}

/// Deliver `target` over HTTP and collect the response status + body. Errors if
/// the request could not be made.
/// @spec projects/vat/tech-design/semantic/source/projects-vat-src-emulator-dispatch-rs.md#source
pub async fn dispatch_collect(
    client: &reqwest::Client,
    target: &Target,
) -> anyhow::Result<DispatchResult> {
    let method =
        reqwest::Method::from_bytes(target.method.as_bytes()).unwrap_or(reqwest::Method::POST);
    let mut req = client.request(method, &target.uri);
    for (key, value) in &target.headers {
        req = req.header(key, value);
    }
    if let Some(oidc) = &target.oidc {
        req = req.header("Authorization", format!("Bearer {}", mint_oidc(oidc)));
    }
    let resp = req.body(target.body.clone()).send().await?;
    let code = resp.status().as_u16();
    let body = resp.text().await.unwrap_or_default();
    Ok(DispatchResult { code, body })
}

/// Deliver `target` over HTTP. Returns the response status code, or an error if
/// the request could not be made.
/// @spec projects/vat/tech-design/semantic/source/projects-vat-src-emulator-dispatch-rs.md#source
pub async fn dispatch_http(client: &reqwest::Client, target: &Target) -> anyhow::Result<u16> {
    Ok(dispatch_collect(client, target).await?.code)
}
// CODEGEN-END
