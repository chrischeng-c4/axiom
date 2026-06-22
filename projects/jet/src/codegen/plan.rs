// SPEC-MANAGED: .aw/tech-design/projects/jet/interfaces/cli/openapi-client-codegen-types-fetch-client-react-query-hooks.md#logic
// HANDWRITE-BEGIN
//! Per-operation generation plan, shared by the client and hooks emitters so
//! function names, parameter types, and return types stay consistent.

use crate::codegen::names::{self, to_camel, to_pascal, NameRegistry};
use crate::codegen::openapi::{Operation, Parameter, RefOr, Spec};
use crate::codegen::tsmap::{type_expr, TypeMap};

/// Fully-resolved plan for one HTTP operation.
///
/// @spec .aw/tech-design/projects/jet/interfaces/cli/openapi-client-codegen-types-fetch-client-react-query-hooks.md#logic
#[derive(Debug)]
pub struct OperationPlan {
    /// camelCase client function name (also the hook stem).
    pub fn_name: String,
    /// Uppercase HTTP verb for the request, e.g. `GET`.
    pub http_method: String,
    /// True for read operations (`GET`) → query hooks; else mutation hooks.
    pub is_query: bool,
    /// Path with `{x}` replaced by `${params.x}`, wrapped in backticks.
    pub path_template: String,
    /// Fields of the `params` object type (`petId: number`, `limit?: number`, ...).
    pub param_fields: Vec<String>,
    /// `(jsonKey, accessor)` pairs for query parameters.
    pub query_pairs: Vec<(String, String)>,
    /// `(headerName, accessor)` pairs for header parameters.
    pub header_pairs: Vec<(String, String)>,
    /// True when a JSON request body is sent.
    pub has_body: bool,
    /// TypeScript return type of the operation.
    pub return_type: String,
}

impl OperationPlan {
    /// `{ ... }` params object type, or `None` when the operation takes no input.
    pub fn params_type(&self) -> Option<String> {
        if self.param_fields.is_empty() {
            None
        } else {
            Some(format!("{{ {} }}", self.param_fields.join("; ")))
        }
    }
}

const METHODS: &[&str] = &["get", "post", "put", "patch", "delete"];

/// Build a deterministic plan for every operation in the spec.
pub fn build(spec: &Spec, tm: &TypeMap) -> Vec<OperationPlan> {
    let mut reg = NameRegistry::new();
    let mut plans = Vec::new();
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
                plans.push(build_one(method, path, op, path_level, tm, &mut reg));
            }
        }
    }
    plans
}

fn build_one(
    method: &str,
    path: &str,
    op: &Operation,
    path_level: &[RefOr<Parameter>],
    tm: &TypeMap,
    reg: &mut NameRegistry,
) -> OperationPlan {
    let fn_name = reg.unique(&op_base_name(method, path, op));

    let mut param_fields = Vec::new();
    let mut query_pairs = Vec::new();
    let mut header_pairs = Vec::new();

    for p in inline_params(path_level).chain(inline_params(&op.parameters)) {
        let ty = p
            .schema
            .as_ref()
            .map(|s| type_expr(s, tm))
            .unwrap_or_else(|| "string".to_string());
        match p.location.as_str() {
            "path" => {
                param_fields.push(format!("{}: {}", names::prop_key(&p.name), ty));
            }
            "query" => {
                param_fields.push(format!(
                    "{}{}: {}",
                    names::prop_key(&p.name),
                    if p.required { "" } else { "?" },
                    ty
                ));
                query_pairs.push((p.name.clone(), names::param_access(&p.name)));
            }
            "header" => {
                param_fields.push(format!(
                    "{}{}: {}",
                    names::prop_key(&p.name),
                    if p.required { "" } else { "?" },
                    ty
                ));
                header_pairs.push((p.name.clone(), names::param_access(&p.name)));
            }
            _ => {} // cookie and unknown locations are out of subset
        }
    }

    let mut has_body = false;
    if let Some(RefOr::Item(rb)) = &op.request_body {
        if let Some(mt) = rb.content.get("application/json") {
            if let Some(schema) = &mt.schema {
                has_body = true;
                param_fields.push(format!(
                    "body{}: {}",
                    if rb.required { "" } else { "?" },
                    type_expr(schema, tm)
                ));
            }
        }
    }

    OperationPlan {
        fn_name,
        http_method: method.to_uppercase(),
        is_query: method == "get",
        path_template: template_path(path),
        param_fields,
        query_pairs,
        header_pairs,
        has_body,
        return_type: return_type(op, tm),
    }
}

fn inline_params(params: &[RefOr<Parameter>]) -> impl Iterator<Item = &Parameter> {
    params.iter().filter_map(|p| match p {
        RefOr::Item(p) => Some(p.as_ref()),
        RefOr::Ref(_) => None, // component-parameter refs are out of subset
    })
}

fn op_base_name(method: &str, path: &str, op: &Operation) -> String {
    match &op.operation_id {
        Some(id) if !id.trim().is_empty() => to_camel(id),
        _ => fallback_name(method, path),
    }
}

/// Deterministic name for an operation with no `operationId`:
/// `GET /pets/{petId}` → `getPetsByPetId`.
fn fallback_name(method: &str, path: &str) -> String {
    let mut s = method.to_lowercase();
    for seg in path.split('/').filter(|s| !s.is_empty()) {
        if seg.starts_with('{') {
            let inner = seg.trim_start_matches('{').trim_end_matches('}');
            s.push_str("By");
            s.push_str(&to_pascal(inner));
        } else {
            s.push_str(&to_pascal(seg));
        }
    }
    to_camel(&s)
}

fn template_path(path: &str) -> String {
    let mut out = String::from("`");
    let mut chars = path.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '{' {
            let mut name = String::new();
            for c in chars.by_ref() {
                if c == '}' {
                    break;
                }
                name.push(c);
            }
            out.push_str("${params.");
            out.push_str(&name);
            out.push('}');
        } else {
            out.push(c);
        }
    }
    out.push('`');
    out
}

fn return_type(op: &Operation, tm: &TypeMap) -> String {
    let resp = pick_response(op);
    match resp {
        Some(RefOr::Item(resp)) => match resp.content.get("application/json") {
            Some(mt) => match &mt.schema {
                Some(schema) => type_expr(schema, tm),
                None => "void".to_string(),
            },
            None => "void".to_string(),
        },
        Some(RefOr::Ref(_)) => "unknown".to_string(),
        None => "void".to_string(),
    }
}

fn pick_response(op: &Operation) -> Option<&RefOr<crate::codegen::openapi::Response>> {
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
    use std::collections::BTreeMap;

    fn tm() -> TypeMap {
        TypeMap {
            names: BTreeMap::new(),
        }
    }

    fn spec(json: &str) -> Spec {
        serde_json::from_str(json).unwrap()
    }

    #[test]
    fn fallback_name_from_method_and_path() {
        assert_eq!(fallback_name("get", "/pets/{petId}"), "getPetsByPetId");
        assert_eq!(fallback_name("post", "/pets"), "postPets");
    }

    #[test]
    fn template_path_substitutes_params() {
        assert_eq!(template_path("/pets/{petId}"), "`/pets/${params.petId}`");
        assert_eq!(template_path("/pets"), "`/pets`");
    }

    #[test]
    fn plan_uses_operation_id_and_picks_2xx_json() {
        let s = spec(
            r##"{"paths":{"/pets/{petId}":{"get":{"operationId":"getPetById",
            "parameters":[{"name":"petId","in":"path","required":true,"schema":{"type":"integer"}}],
            "responses":{"200":{"content":{"application/json":{"schema":{"type":"string"}}}}}}}}}"##,
        );
        let plans = build(&s, &tm());
        assert_eq!(plans.len(), 1);
        let p = &plans[0];
        assert_eq!(p.fn_name, "getPetById");
        assert!(p.is_query);
        assert_eq!(p.return_type, "string");
        assert_eq!(p.params_type().unwrap(), "{ petId: number }");
        assert_eq!(p.path_template, "`/pets/${params.petId}`");
    }

    #[test]
    fn missing_operation_id_falls_back() {
        let s = spec(r##"{"paths":{"/pets":{"post":{"responses":{"204":{}}}}}}"##);
        let plans = build(&s, &tm());
        assert_eq!(plans[0].fn_name, "postPets");
        assert!(!plans[0].is_query);
        assert_eq!(plans[0].return_type, "void");
    }
}
// HANDWRITE-END
