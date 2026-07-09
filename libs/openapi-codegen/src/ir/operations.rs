// SPEC-MANAGED: libs/openapi-codegen/tech-design/semantic/source/libs-openapi-codegen-src-ir-operations-rs.md#rust-source-unit
// CODEGEN-BEGIN
//! Language-neutral per-operation plan.
//!
//! Holds the HTTP shape of each operation and the *schema references* of its
//! inputs/outputs — never a rendered, language-specific type. Each emitter under
//! `crate::emit::<lang>` maps an [`OperationIR`] to its own naming (camelCase /
//! snake_case) and type syntax (TS / pydantic / serde).

use crate::ir::openapi::{Operation, Parameter, RefOr, Response, Schema, Spec};

/// A parameter (path / query / header). `schema == None` means an untyped
/// parameter, which emitters default to a string.
#[derive(Debug, Clone)]
/// @spec libs/openapi-codegen/tech-design/semantic/source/libs-openapi-codegen-src-ir-operations-rs.md#source
pub struct ParamIR {
    pub name: String,
    pub schema: Option<RefOr<Schema>>,
    pub required: bool,
}

/// The JSON request body, if any.
#[derive(Debug, Clone)]
/// @spec libs/openapi-codegen/tech-design/semantic/source/libs-openapi-codegen-src-ir-operations-rs.md#source
pub struct BodyIR {
    pub schema: RefOr<Schema>,
    pub required: bool,
}

/// One HTTP operation, fully structural (no language-specific names or types).
#[derive(Debug, Clone)]
/// @spec libs/openapi-codegen/tech-design/semantic/source/libs-openapi-codegen-src-ir-operations-rs.md#source
pub struct OperationIR {
    /// `operationId` if present (emitters fall back to method+path otherwise).
    pub operation_id: Option<String>,
    /// Lowercase HTTP verb, e.g. `get`. `"query"` for OpenAPI 3.2's `QUERY`
    /// method keyword; any other lowercased `additionalOperations` key
    /// otherwise (e.g. `"purge"`).
    pub method: String,
    /// Uppercase HTTP verb, e.g. `GET`, `QUERY`, `PURGE`.
    pub http_method: String,
    /// True for read operations — `GET` and OpenAPI 3.2 `QUERY` both map to a
    /// TanStack Query *query* hook in the TS emitter (vs. a mutation hook for
    /// everything else). Unrelated to the HTTP `QUERY` method name itself.
    pub is_query: bool,
    /// Raw path template, e.g. `/pets/{petId}`.
    pub path: String,
    pub path_params: Vec<ParamIR>,
    pub query_params: Vec<ParamIR>,
    pub header_params: Vec<ParamIR>,
    pub body: Option<BodyIR>,
    /// The success-response JSON schema, or `None` for a no-content response.
    pub response: Option<RefOr<Schema>>,
    /// POST-twin fallback target for `QUERY` operations (epic #1296 policy:
    /// every QUERY endpoint has a POST twin). `Some` only when `method ==
    /// "query"`: the operation's `x-post-twin: <path>` vendor extension if
    /// present, else the documented default convention — the sibling `post`
    /// operation on this same path item (i.e. the same path template).
    /// `None` for every other method.
    pub post_twin_path: Option<String>,
}

/// @spec libs/openapi-codegen/tech-design/semantic/source/libs-openapi-codegen-src-ir-operations-rs.md#source
impl OperationIR {
    pub fn has_inputs(&self) -> bool {
        !self.path_params.is_empty()
            || !self.query_params.is_empty()
            || !self.header_params.is_empty()
            || self.body.is_some()
    }
}

const METHODS: &[&str] = &["get", "post", "put", "patch", "delete", "query"];

/// Walk every path/method and produce a structural plan per operation, in a
/// deterministic order. Standard keywords (including OpenAPI 3.2's `query`)
/// are visited first, then any OpenAPI 3.2 `additionalOperations` entries
/// whose method name isn't already one of [`METHODS`] — so a method present
/// under both a dedicated keyword and `additionalOperations` is only emitted
/// once.
/// @spec libs/openapi-codegen/tech-design/semantic/source/libs-openapi-codegen-src-ir-operations-rs.md#source
pub fn build(spec: &Spec) -> Vec<OperationIR> {
    let mut ops = Vec::new();
    for (path, item) in &spec.paths {
        let path_level = &item.parameters;
        for method in METHODS {
            let op = match *method {
                "get" => &item.get,
                "post" => &item.post,
                "put" => &item.put,
                "patch" => &item.patch,
                "delete" => &item.delete,
                "query" => &item.query,
                _ => &None,
            };
            if let Some(op) = op {
                ops.push(build_one(method, path, op, path_level));
            }
        }
        for (method_upper, op) in &item.additional_operations {
            let method_lower = method_upper.to_lowercase();
            if METHODS.contains(&method_lower.as_str()) {
                continue; // already covered by a dedicated keyword above
            }
            ops.push(build_one(&method_lower, path, op, path_level));
        }
    }
    ops
}

fn build_one(
    method: &str,
    path: &str,
    op: &Operation,
    path_level: &[RefOr<Parameter>],
) -> OperationIR {
    let mut path_params = Vec::new();
    let mut query_params = Vec::new();
    let mut header_params = Vec::new();

    for p in inline_params(path_level).chain(inline_params(&op.parameters)) {
        let mk = |required: bool| ParamIR {
            name: p.name.clone(),
            schema: p.schema.clone(),
            required,
        };
        match p.location.as_str() {
            // Path parameters are always required.
            "path" => path_params.push(mk(true)),
            "query" => query_params.push(mk(p.required)),
            "header" => header_params.push(mk(p.required)),
            _ => {} // cookie and unknown locations are out of subset
        }
    }

    let body = op.request_body.as_ref().and_then(|rb| match rb {
        RefOr::Item(rb) => rb
            .content
            .get("application/json")
            .and_then(|mt| mt.schema.as_ref())
            .map(|schema| BodyIR {
                schema: schema.clone(),
                required: rb.required,
            }),
        RefOr::Ref(_) => None,
    });

    let response = pick_response(op).and_then(|r| match r {
        RefOr::Item(resp) => resp
            .content
            .get("application/json")
            .and_then(|mt| mt.schema.clone()),
        RefOr::Ref(_) => None,
    });

    let post_twin_path = (method == "query").then(|| {
        op.extensions
            .get("x-post-twin")
            .and_then(|v| v.as_str())
            .map(str::to_string)
            .unwrap_or_else(|| path.to_string())
    });

    OperationIR {
        operation_id: op.operation_id.clone(),
        method: method.to_string(),
        http_method: method.to_uppercase(),
        is_query: method == "get" || method == "query",
        path: path.to_string(),
        path_params,
        query_params,
        header_params,
        body,
        response,
        post_twin_path,
    }
}

fn inline_params(params: &[RefOr<Parameter>]) -> impl Iterator<Item = &Parameter> {
    params.iter().filter_map(|p| match p {
        RefOr::Item(p) => Some(p.as_ref()),
        RefOr::Ref(_) => None, // component-parameter refs are out of subset
    })
}

/// Pick the success response: prefer 200/201/202/203, then any `2xx`, then the
/// `2XX` range or `default`.
fn pick_response(op: &Operation) -> Option<&RefOr<Response>> {
    for code in ["200", "201", "202", "203"] {
        if let Some(r) = op.responses.get(code) {
            return Some(r);
        }
    }
    if let Some(r) = op
        .responses
        .iter()
        .find(|(k, _)| k.starts_with('2'))
        .map(|(_, v)| v)
    {
        return Some(r);
    }
    op.responses
        .get("2XX")
        .or_else(|| op.responses.get("default"))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn spec(json: &str) -> Spec {
        serde_json::from_str(json).unwrap()
    }

    #[test]
    fn query_operation_defaults_post_twin_to_same_path() {
        let s = spec(
            r##"{"paths":{"/pets":{
              "query":{"operationId":"searchPets",
                "requestBody":{"required":true,"content":{"application/json":{"schema":{"type":"object"}}}},
                "responses":{"200":{"content":{"application/json":{"schema":{"type":"array","items":{"type":"string"}}}}}}},
              "post":{"operationId":"createPet",
                "responses":{"201":{"content":{"application/json":{"schema":{"type":"string"}}}}}}}}}"##,
        );
        let ops = build(&s);
        let query_op = ops.iter().find(|o| o.method == "query").unwrap();
        assert_eq!(query_op.http_method, "QUERY");
        assert!(query_op.is_query);
        assert_eq!(query_op.post_twin_path.as_deref(), Some("/pets"));
        assert!(query_op.body.is_some());

        let post_op = ops.iter().find(|o| o.method == "post").unwrap();
        assert!(!post_op.is_query);
        assert_eq!(post_op.post_twin_path, None);
    }

    #[test]
    fn query_operation_honors_x_post_twin_extension_override() {
        let s = spec(
            r##"{"paths":{"/pets/{petId}":{
              "query":{"operationId":"searchPetById","x-post-twin":"/pets/{petId}/search",
                "parameters":[{"name":"petId","in":"path","required":true,"schema":{"type":"integer"}}],
                "requestBody":{"required":true,"content":{"application/json":{"schema":{"type":"object"}}}},
                "responses":{"200":{"content":{"application/json":{"schema":{"type":"string"}}}}}}}}}"##,
        );
        let ops = build(&s);
        let query_op = &ops[0];
        assert_eq!(
            query_op.post_twin_path.as_deref(),
            Some("/pets/{petId}/search")
        );
    }

    #[test]
    fn additional_operations_pass_through_without_choking_and_avoid_keyword_duplicates() {
        let s = spec(
            r##"{"paths":{"/pets":{
              "get":{"operationId":"listPets","responses":{"200":{"description":"ok"}}},
              "additionalOperations":{
                "PURGE":{"operationId":"purgePets","responses":{"204":{"description":"no content"}}},
                "GET":{"operationId":"shouldNotDuplicateGet","responses":{"200":{"description":"ok"}}}
              }}}}"##,
        );
        let ops = build(&s);
        assert_eq!(ops.iter().filter(|o| o.method == "get").count(), 1);
        let purge = ops.iter().find(|o| o.method == "purge").unwrap();
        assert_eq!(purge.http_method, "PURGE");
        assert_eq!(purge.operation_id.as_deref(), Some("purgePets"));
        assert_eq!(purge.post_twin_path, None);
    }
}
// CODEGEN-END
