---
id: projects-vat-src-emulator-dispatch-rs
summary: >
  rust-source-unit TD AST payload for projects/vat/src/emulator/dispatch.rs.
fill_sections: [overview, source, changes]
capability_refs:
  - id: agent-native-gpu-native-dev-containers
    role: primary
    claim: local-agent-test-runner-protocol
    coverage: partial
    rationale: "This rust-source-unit TD preserves vat source ownership while migrating #39 off group-level source replay."
---

# Standardized projects/vat/src/emulator/dispatch.rs

## Overview
<!-- type: overview lang: markdown -->

Rust source-unit TD for `projects/vat/src/emulator/dispatch.rs`, captured during #39 vat migration onto td_ast lossless source generation.

## Source
<!-- type: rust-source-unit lang: rust -->

````rust
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
pub struct Target {
    pub uri: String,
    pub method: String,
    pub headers: BTreeMap<String, String>,
    pub body: Vec<u8>,
    pub oidc: Option<Oidc>,
}

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
pub struct DispatchResult {
    pub code: u16,
    pub body: String,
}

/// Deliver `target` over HTTP and collect the response status + body. Errors if
/// the request could not be made.
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
pub async fn dispatch_http(client: &reqwest::Client, target: &Target) -> anyhow::Result<u16> {
    Ok(dispatch_collect(client, target).await?.code)
}
````

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/vat/src/emulator/dispatch.rs
    action: modify
    section: rust-source-unit
    impl_mode: codegen
    description: |
      rust-source-unit (td_ast) source for `projects/vat/src/emulator/dispatch.rs` captured during #39 vat standardization.
```
