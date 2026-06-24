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
pub struct ParamIR {
    pub name: String,
    pub schema: Option<RefOr<Schema>>,
    pub required: bool,
}

/// The JSON request body, if any.
#[derive(Debug, Clone)]
pub struct BodyIR {
    pub schema: RefOr<Schema>,
    pub required: bool,
}

/// One HTTP operation, fully structural (no language-specific names or types).
#[derive(Debug, Clone)]
pub struct OperationIR {
    /// `operationId` if present (emitters fall back to method+path otherwise).
    pub operation_id: Option<String>,
    /// Lowercase HTTP verb, e.g. `get`.
    pub method: String,
    /// Uppercase HTTP verb, e.g. `GET`.
    pub http_method: String,
    /// True for read operations (`GET`).
    pub is_query: bool,
    /// Raw path template, e.g. `/pets/{petId}`.
    pub path: String,
    pub path_params: Vec<ParamIR>,
    pub query_params: Vec<ParamIR>,
    pub header_params: Vec<ParamIR>,
    pub body: Option<BodyIR>,
    /// The success-response JSON schema, or `None` for a no-content response.
    pub response: Option<RefOr<Schema>>,
}

impl OperationIR {
    pub fn has_inputs(&self) -> bool {
        !self.path_params.is_empty()
            || !self.query_params.is_empty()
            || !self.header_params.is_empty()
            || self.body.is_some()
    }
}

const METHODS: &[&str] = &["get", "post", "put", "patch", "delete"];

/// Walk every path/method and produce a structural plan per operation, in a
/// deterministic order.
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
                _ => &None,
            };
            if let Some(op) = op {
                ops.push(build_one(method, path, op, path_level));
            }
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

    OperationIR {
        operation_id: op.operation_id.clone(),
        method: method.to_string(),
        http_method: method.to_uppercase(),
        is_query: method == "get",
        path: path.to_string(),
        path_params,
        query_params,
        header_params,
        body,
        response,
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
