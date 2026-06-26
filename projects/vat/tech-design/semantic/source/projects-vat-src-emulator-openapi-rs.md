---
id: projects-vat-src-emulator-openapi-rs
summary: >
  rust-source-unit TD AST payload for projects/vat/src/emulator/openapi.rs.
fill_sections: [overview, source, changes]
capability_refs:
  - id: agent-native-gpu-native-dev-containers
    role: primary
    claim: local-agent-test-runner-protocol
    coverage: partial
    rationale: "This rust-source-unit TD preserves vat source ownership while migrating #39 off group-level source replay."
---

# Standardized projects/vat/src/emulator/openapi.rs

## Overview
<!-- type: overview lang: markdown -->

Rust source-unit TD for `projects/vat/src/emulator/openapi.rs`, captured during #39 vat migration onto td_ast lossless source generation.

## Source
<!-- type: rust-source-unit lang: rust -->

````rust
//! OpenAPI-driven mock HTTP service — read a spec, serve its responses.
//!
//! One engine ([`OpenApiSpec`]) parses an OpenAPI 3.x (or, loosely, Swagger 2.0)
//! document as a generic JSON value and, for a request method+path, matches the
//! operation (with path templating like `/users/{id}`), selects a 2xx (else
//! `default`) response, and produces a body from the response `example`, then
//! `examples`, then a schema-synthesized example (`$ref` resolved, depth-guarded).
//! It backs two surfaces: the standalone `openapi` preset ([`serve`]) and the
//! http-mock proxy's `/__admin/openapi` source ([`SpecRegistry`]). No request
//! validation, no auth — enough to stand up a working fake. Never panics on bad
//! input: an unmatched path is `None` (404), a malformed spec degrades to `{}`.
//!
//! @spec projects/vat/tech-design/interfaces/rest/openapi-driven-mock-http-service.md#logic

use std::sync::{Arc, Mutex};

use anyhow::{Context, Result};
use axum::extract::{Request, State};
use axum::http::{header, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::Router;
use serde::Deserialize;
use serde_json::{json, Value};

/// A response generated from the spec for a matched operation.
pub struct MockResponse {
    pub status: u16,
    pub content_type: String,
    pub body: Vec<u8>,
}

/// A parsed OpenAPI document, walked as a generic value.
pub struct OpenApiSpec {
    doc: Value,
}

impl OpenApiSpec {
    /// Parse a spec from YAML or JSON text (YAML is a JSON superset, so one path).
    #[allow(clippy::should_implement_trait)]
    pub fn from_str(text: &str) -> Result<Self> {
        let doc: Value = serde_yaml::from_str(text).context("parse OpenAPI document")?;
        Ok(Self { doc })
    }

    /// Load a spec from a file path.
    pub fn load(path: &str) -> Result<Self> {
        let text =
            std::fs::read_to_string(path).with_context(|| format!("read OpenAPI spec {path}"))?;
        Self::from_str(&text)
    }

    /// Generate a response for `method`+`path`, or `None` if no operation matches.
    pub fn respond(&self, method: &str, path: &str) -> Option<MockResponse> {
        let paths = self.doc.get("paths")?.as_object()?;
        let op = self.match_operation(paths, method, path)?;
        let response = self.select_response(op)?;
        let (content_type, body) = self.response_body(response);
        let status = self.response_status(op);
        Some(MockResponse {
            status,
            content_type,
            body,
        })
    }

    /// Find the operation whose templated path + method match the request.
    fn match_operation<'a>(
        &'a self,
        paths: &'a serde_json::Map<String, Value>,
        method: &str,
        path: &str,
    ) -> Option<&'a Value> {
        let m = method.to_ascii_lowercase();
        for (tmpl, item) in paths {
            if path_matches(tmpl, path) {
                if let Some(op) = item.get(&m) {
                    return Some(op);
                }
            }
        }
        None
    }

    /// The status code we report: the lowest 2xx, else 200 (default/any).
    fn response_status(&self, op: &Value) -> u16 {
        if let Some(responses) = op.get("responses").and_then(|r| r.as_object()) {
            let mut best: Option<u16> = None;
            for code in responses.keys() {
                if let Ok(n) = code.parse::<u16>() {
                    if (200..300).contains(&n) && best.map(|b| n < b).unwrap_or(true) {
                        best = Some(n);
                    }
                }
            }
            if let Some(n) = best {
                return n;
            }
        }
        200
    }

    /// Select the response object: lowest 2xx, else `default`, else any.
    fn select_response<'a>(&'a self, op: &'a Value) -> Option<&'a Value> {
        let responses = op.get("responses")?.as_object()?;
        let mut best: Option<(u16, &Value)> = None;
        for (code, resp) in responses {
            if let Ok(n) = code.parse::<u16>() {
                if (200..300).contains(&n) && best.map(|(b, _)| n < b).unwrap_or(true) {
                    best = Some((n, resp));
                }
            }
        }
        if let Some((_, r)) = best {
            return Some(r);
        }
        if let Some(d) = responses.get("default") {
            return Some(d);
        }
        responses.values().next()
    }

    /// Pick a body for a response: example, then examples, then schema synthesis.
    /// OpenAPI 2.0 falls back to the response's `schema` directly.
    fn response_body(&self, response: &Value) -> (String, Vec<u8>) {
        if let Some(content) = response.get("content").and_then(|c| c.as_object()) {
            // Prefer application/json, else the first declared media type.
            let media_key = if content.contains_key("application/json") {
                "application/json".to_string()
            } else {
                content
                    .keys()
                    .next()
                    .cloned()
                    .unwrap_or_else(|| "application/json".to_string())
            };
            if let Some(m) = content.get(&media_key) {
                if let Some(ex) = m.get("example") {
                    return (media_key, serialize(ex));
                }
                if let Some(exs) = m.get("examples").and_then(|e| e.as_object()) {
                    if let Some(first) = exs.values().next() {
                        let v = first.get("value").unwrap_or(first);
                        return (media_key, serialize(v));
                    }
                }
                if let Some(schema) = m.get("schema") {
                    return (media_key, serialize(&self.example_from_schema(schema, 0)));
                }
            }
            return (media_key, b"{}".to_vec());
        }
        if let Some(schema) = response.get("schema") {
            return (
                "application/json".into(),
                serialize(&self.example_from_schema(schema, 0)),
            );
        }
        ("application/json".into(), b"{}".to_vec())
    }

    /// Synthesize an example value from a JSON schema (recursive, depth-guarded).
    fn example_from_schema(&self, schema: &Value, depth: u8) -> Value {
        if depth > 8 {
            return Value::Null;
        }
        if let Some(r) = schema.get("$ref").and_then(|r| r.as_str()) {
            return match self.resolve_ref(r) {
                Some(resolved) => self.example_from_schema(&resolved, depth + 1),
                None => Value::Null,
            };
        }
        if let Some(ex) = schema.get("example") {
            return ex.clone();
        }
        if let Some(d) = schema.get("default") {
            return d.clone();
        }
        if let Some(first) = schema
            .get("enum")
            .and_then(|e| e.as_array())
            .and_then(|a| a.first())
        {
            return first.clone();
        }
        // Composition: allOf merges objects; oneOf/anyOf take the first branch.
        if let Some(all) = schema.get("allOf").and_then(|a| a.as_array()) {
            let mut merged = serde_json::Map::new();
            for s in all {
                if let Value::Object(o) = self.example_from_schema(s, depth + 1) {
                    merged.extend(o);
                }
            }
            return Value::Object(merged);
        }
        for key in ["oneOf", "anyOf"] {
            if let Some(first) = schema
                .get(key)
                .and_then(|a| a.as_array())
                .and_then(|a| a.first())
            {
                return self.example_from_schema(first, depth + 1);
            }
        }
        let ty = schema.get("type").and_then(|t| t.as_str()).unwrap_or(
            if schema.get("properties").is_some() {
                "object"
            } else {
                "string"
            },
        );
        match ty {
            "object" => {
                let mut obj = serde_json::Map::new();
                if let Some(props) = schema.get("properties").and_then(|p| p.as_object()) {
                    for (k, v) in props {
                        obj.insert(k.clone(), self.example_from_schema(v, depth + 1));
                    }
                }
                Value::Object(obj)
            }
            "array" => {
                let item = schema
                    .get("items")
                    .map(|it| self.example_from_schema(it, depth + 1))
                    .unwrap_or(Value::Null);
                Value::Array(vec![item])
            }
            "integer" => json!(0),
            "number" => json!(0.0),
            "boolean" => json!(true),
            _ => Value::String("string".into()),
        }
    }

    /// Resolve a local JSON-pointer `$ref` (`#/components/schemas/X`, `#/definitions/X`).
    fn resolve_ref(&self, r: &str) -> Option<Value> {
        let pointer = r.strip_prefix("#/")?;
        let mut cur = &self.doc;
        for seg in pointer.split('/') {
            let seg = seg.replace("~1", "/").replace("~0", "~");
            cur = cur.get(&seg)?;
        }
        Some(cur.clone())
    }
}

/// Render a value as a body: a JSON string becomes a raw body, else JSON bytes.
fn serialize(v: &Value) -> Vec<u8> {
    match v {
        Value::String(s) => s.clone().into_bytes(),
        _ => serde_json::to_vec(v).unwrap_or_default(),
    }
}

/// Match a templated path (`/users/{id}`) against a concrete path. A `{...}`
/// segment matches any single non-empty segment; lengths must be equal.
fn path_matches(tmpl: &str, path: &str) -> bool {
    let t: Vec<&str> = tmpl.trim_matches('/').split('/').collect();
    let p: Vec<&str> = path.trim_matches('/').split('/').collect();
    if t.len() != p.len() {
        return false;
    }
    t.iter().zip(p.iter()).all(|(a, b)| {
        if a.starts_with('{') && a.ends_with('}') {
            !b.is_empty()
        } else {
            a == b
        }
    })
}

/// A set of registered specs, optionally host-bound, used by the http-mock proxy.
#[derive(Default)]
pub struct SpecRegistry {
    specs: Mutex<Vec<(Option<String>, OpenApiSpec)>>,
}

impl SpecRegistry {
    /// Register a spec, optionally bound to a host (else consulted for any host).
    pub fn add(&self, host: Option<String>, spec: OpenApiSpec) {
        if let Ok(mut g) = self.specs.lock() {
            g.push((host, spec));
        }
    }

    /// Drop all registered specs.
    pub fn clear(&self) {
        if let Ok(mut g) = self.specs.lock() {
            g.clear();
        }
    }

    /// First registered spec (host-bound match or global) that answers the request.
    pub fn respond(&self, host: &str, method: &str, path: &str) -> Option<MockResponse> {
        let g = self.specs.lock().ok()?;
        for (bound, spec) in g.iter() {
            if bound.as_deref().map(|h| h == host).unwrap_or(true) {
                if let Some(r) = spec.respond(method, path) {
                    return Some(r);
                }
            }
        }
        None
    }
}

/// Registration payload for the http-mock proxy's `/__admin/openapi` route.
#[derive(Deserialize)]
pub struct Registration {
    #[serde(default)]
    pub host: Option<String>,
    pub spec: String,
}

#[derive(Clone)]
struct AppState {
    spec: Arc<OpenApiSpec>,
}

/// Serve the standalone OpenAPI mock server until the process is killed.
pub async fn serve(host_port: &str, spec_path: &str) -> Result<()> {
    let spec = Arc::new(OpenApiSpec::load(spec_path)?);
    let app = Router::new()
        .fallback(handle_any)
        .with_state(AppState { spec });
    let listener = tokio::net::TcpListener::bind(host_port)
        .await
        .with_context(|| format!("bind openapi mock on {host_port}"))?;
    axum::serve(listener, app)
        .await
        .context("serve openapi mock")?;
    Ok(())
}

/// Answer any request from the spec, or 404 when the operation is undocumented.
async fn handle_any(State(st): State<AppState>, req: Request) -> Response {
    let method = req.method().to_string();
    let path = req.uri().path().to_string();
    match st.spec.respond(&method, &path) {
        Some(r) => (
            StatusCode::from_u16(r.status).unwrap_or(StatusCode::OK),
            [(header::CONTENT_TYPE, r.content_type)],
            r.body,
        )
            .into_response(),
        None => (
            StatusCode::NOT_FOUND,
            [(header::CONTENT_TYPE, "application/json".to_string())],
            br#"{"error":"not in spec"}"#.to_vec(),
        )
            .into_response(),
    }
}
// CODEGEN-END

#[cfg(test)]
mod tests {
    use super::*;

    const SPEC: &str = r##"
openapi: 3.0.0
info: { title: Pets, version: "1.0" }
paths:
  /pets/{id}:
    get:
      responses:
        "200":
          content:
            application/json:
              example: { id: 7, name: "Rex" }
  /widgets:
    get:
      responses:
        "200":
          content:
            application/json:
              schema:
                $ref: "#/components/schemas/Widget"
components:
  schemas:
    Widget:
      type: object
      properties:
        id: { type: integer }
        label: { type: string }
        tags:
          type: array
          items: { type: string }
"##;

    #[test]
    fn respond_uses_example_with_path_templating() {
        let spec = OpenApiSpec::from_str(SPEC).unwrap();
        let r = spec.respond("GET", "/pets/123").expect("matched");
        assert_eq!(r.status, 200);
        let v: Value = serde_json::from_slice(&r.body).unwrap();
        assert_eq!(v["id"], 7);
        assert_eq!(v["name"], "Rex");
    }

    #[test]
    fn respond_synthesizes_from_schema_via_ref() {
        let spec = OpenApiSpec::from_str(SPEC).unwrap();
        let r = spec.respond("get", "/widgets").expect("matched");
        let v: Value = serde_json::from_slice(&r.body).unwrap();
        assert_eq!(v["id"], 0);
        assert_eq!(v["label"], "string");
        assert_eq!(v["tags"], json!(["string"]));
    }

    #[test]
    fn unmatched_path_or_method_is_none() {
        let spec = OpenApiSpec::from_str(SPEC).unwrap();
        assert!(spec.respond("GET", "/pets").is_none());
        assert!(spec.respond("GET", "/nope/1").is_none());
        assert!(spec.respond("POST", "/pets/1").is_none());
    }

    #[test]
    fn self_referential_schema_is_depth_guarded() {
        let spec = OpenApiSpec::from_str(
            r##"
openapi: 3.0.0
paths:
  /node:
    get:
      responses:
        "200":
          content:
            application/json:
              schema: { $ref: "#/components/schemas/Node" }
components:
  schemas:
    Node:
      type: object
      properties:
        next: { $ref: "#/components/schemas/Node" }
"##,
        )
        .unwrap();
        // Must terminate (depth guard), not stack-overflow.
        let r = spec.respond("GET", "/node").expect("matched");
        assert!(serde_json::from_slice::<Value>(&r.body).is_ok());
    }

    #[test]
    fn malformed_spec_does_not_panic() {
        let spec = OpenApiSpec::from_str("not: [valid: openapi").err();
        assert!(spec.is_some());
        let empty = OpenApiSpec::from_str("openapi: 3.0.0").unwrap();
        assert!(empty.respond("GET", "/x").is_none());
    }

    #[test]
    fn registry_respects_host_binding() {
        let reg = SpecRegistry::default();
        reg.add(
            Some("api.test".into()),
            OpenApiSpec::from_str(SPEC).unwrap(),
        );
        assert!(reg.respond("api.test", "GET", "/pets/1").is_some());
        assert!(reg.respond("other.test", "GET", "/pets/1").is_none());
    }
}
````

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/vat/src/emulator/openapi.rs
    action: modify
    section: rust-source-unit
    impl_mode: codegen
    description: |
      rust-source-unit (td_ast) source for `projects/vat/src/emulator/openapi.rs` captured during #39 vat standardization.
```
