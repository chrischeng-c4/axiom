// SPEC-MANAGED: apps/pgpool/tech-design/semantic/source/apps-pgpool-src-spec-rs.md#schema
// <HANDWRITE gap="missing-generator:logic:pgpool-bootstrap" tracker="#pgpool-bootstrap" reason="Initial offline route/spec/LLM contract before generated OpenAPI artifacts.">
use serde_json::{json, Value};

pub fn openapi_json() -> String {
    serde_json::to_string_pretty(&openapi()).expect("pgpool OpenAPI serializes")
}

pub fn openapi_yaml() -> String {
    serde_yaml::to_string(&openapi()).expect("pgpool OpenAPI serializes as YAML")
}

pub fn routes_json() -> String {
    serde_json::to_string_pretty(&json!({
        "project": "pgpool",
        "name_status": "working-name",
        "routes": [
            {"method": "GET", "path": "/healthz", "purpose": "liveness"},
            {"method": "GET", "path": "/readyz", "purpose": "readiness and drain state"},
            {"method": "GET", "path": "/metrics", "purpose": "Prometheus metrics"},
            {"method": "GET", "path": "/openapi.json", "purpose": "machine OpenAPI"},
            {"method": "GET", "path": "/docs", "purpose": "Swagger UI"},
            {"method": "GET", "path": "/pools", "purpose": "list backend pools"},
            {"method": "GET", "path": "/pools/{pool}/stats", "purpose": "pool saturation and reuse stats"},
            {"method": "POST", "path": "/drain", "purpose": "enter graceful drain before pod shutdown"}
        ],
        "tcp": [
            {"bind": "0.0.0.0:6432", "protocol": "postgresql-wire", "purpose": "frontend client admission"}
        ]
    }))
    .expect("pgpool route list serializes")
}

pub fn json_schema_json() -> String {
    serde_json::to_string_pretty(&json!({
        "components": {
            "schemas": schemas(),
        }
    }))
    .expect("pgpool schemas serialize")
}

pub const fn llm_workflow_md() -> &'static str {
    r#"# pgpool workflow

`pgpool` is a working app id, not the final product name. It is the initial
Kubernetes-native PostgreSQL pooler surface in `apps/pgpool`.

Start with:

```text
pgpool runtime-plan
pgpool spec --format routes
```

The current slice is intentionally an app scaffold and runtime plan. PostgreSQL
wire handling, backend pools, platform adapters, and k8s operator artifacts are
separate work roots.
"#
}

pub const fn llm_api_md() -> &'static str {
    r#"# pgpool API

The data plane is PostgreSQL wire protocol on TCP port 6432. The admin plane is
HTTP/1.1+h2c and carries standard service endpoints plus pool/drain operations:

- `/healthz`, `/readyz`, `/metrics`, `/openapi.json`, `/docs`
- `GET /pools`
- `GET /pools/{pool}/stats`
- `POST /drain`

Use `pgpool spec --format routes` for the current inventory.
"#
}

pub const fn llm_boundaries_md() -> &'static str {
    r#"# pgpool boundaries

- Core pooling does not embed Cloud SQL Proxy or AlloyDB discovery.
- Platform adapters may supply backend endpoints and auth material later.
- `server-core`, `tcp-server`, and `http-server` own reusable runtime mechanics.
- `pgpool` owns PostgreSQL pool semantics, connection reuse, and drain policy.
"#
}

fn openapi() -> Value {
    json!({
        "openapi": "3.1.0",
        "info": {
            "title": "pgpool working-name admin API",
            "version": env!("CARGO_PKG_VERSION"),
            "description": "Working-name PostgreSQL pooler admin API. The frontend data plane is PostgreSQL wire protocol over TCP."
        },
        "paths": {
            "/healthz": {"get": {"summary": "Liveness probe", "responses": ok_text()}},
            "/readyz": {"get": {"summary": "Readiness and drain probe", "responses": ok_json()}},
            "/metrics": {"get": {"summary": "Prometheus metrics", "responses": ok_text()}},
            "/openapi.json": {"get": {"summary": "OpenAPI document", "responses": ok_json()}},
            "/docs": {"get": {"summary": "Swagger UI", "responses": ok_text()}},
            "/pools": {"get": {"summary": "List backend pools", "responses": ok_schema("PoolList")}},
            "/pools/{pool}/stats": {
                "get": {
                    "summary": "Read pool saturation and reuse stats",
                    "parameters": [path_param("pool", "Backend pool name")],
                    "responses": ok_schema("PoolStats")
                }
            },
            "/drain": {"post": {"summary": "Enter graceful drain", "responses": ok_schema("DrainState")}}
        },
        "components": {"schemas": schemas()}
    })
}

fn schemas() -> Value {
    json!({
        "PoolList": {
            "type": "object",
            "required": ["pools"],
            "properties": {
                "pools": {"type": "array", "items": {"$ref": "#/components/schemas/PoolStats"}}
            }
        },
        "PoolStats": {
            "type": "object",
            "required": ["name", "mode", "frontend_active", "backend_active", "backend_idle"],
            "properties": {
                "name": {"type": "string"},
                "mode": {"type": "string", "enum": ["session", "transaction"]},
                "frontend_active": {"type": "integer", "minimum": 0},
                "backend_active": {"type": "integer", "minimum": 0},
                "backend_idle": {"type": "integer", "minimum": 0}
            }
        },
        "DrainState": {
            "type": "object",
            "required": ["draining"],
            "properties": {
                "draining": {"type": "boolean"}
            }
        }
    })
}

fn path_param(name: &str, description: &str) -> Value {
    json!({
        "name": name,
        "in": "path",
        "required": true,
        "description": description,
        "schema": {"type": "string"}
    })
}

fn ok_text() -> Value {
    json!({"200": {"description": "OK", "content": {"text/plain": {"schema": {"type": "string"}}}}})
}

fn ok_json() -> Value {
    json!({"200": {"description": "OK", "content": {"application/json": {"schema": {"type": "object"}}}}})
}

fn ok_schema(schema: &str) -> Value {
    json!({
        "200": {
            "description": "OK",
            "content": {
                "application/json": {
                    "schema": {"$ref": format!("#/components/schemas/{schema}")}
                }
            }
        }
    })
}
// </HANDWRITE>

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn routes_include_standard_endpoints_and_pg_frontend() {
        let routes = routes_json();
        assert!(routes.contains("/healthz"));
        assert!(routes.contains("/pools/{pool}/stats"));
        assert!(routes.contains("postgresql-wire"));
    }
}
