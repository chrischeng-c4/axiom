// SPEC-MANAGED: .aw/tech-design/projects/jet/interfaces/cli/named-per-operation-request-response-types-xxxdata-xxxresponse-f.md#logic
// HANDWRITE-BEGIN
//! Per-operation generation plan, shared by the type, client, and hooks
//! emitters so function names, the grouped `XxxData` request type, and the
//! `XxxResponse` type stay consistent (hey-api style).

use crate::codegen::names::{to_camel, to_pascal, NameRegistry};
use crate::codegen::openapi::{Operation, Parameter, RefOr, Spec};
use crate::codegen::tsmap::{type_expr, TypeMap};

/// One named field inside a grouped request sub-object (`path`/`query`/`headers`).
#[derive(Debug)]
pub struct ParamField {
    pub name: String,
    pub ts_type: String,
    pub required: bool,
}

/// The JSON request body, if any.
#[derive(Debug)]
pub struct BodyField {
    pub ts_type: String,
    pub required: bool,
}

/// Fully-resolved plan for one HTTP operation.
///
/// @spec .aw/tech-design/projects/jet/interfaces/cli/named-per-operation-request-response-types-xxxdata-xxxresponse-f.md#logic
#[derive(Debug)]
pub struct OperationPlan {
    /// camelCase client function name (also the hook stem source).
    pub fn_name: String,
    /// Uppercase HTTP verb for the request, e.g. `GET`.
    pub http_method: String,
    /// True for read operations (`GET`) → query hooks; else mutation hooks.
    pub is_query: bool,
    /// Raw path template, e.g. `/pets/{petId}`.
    pub path_raw: String,
    pub path_params: Vec<ParamField>,
    pub query_params: Vec<ParamField>,
    pub header_params: Vec<ParamField>,
    pub body: Option<BodyField>,
    /// TypeScript expression for the operation response (or `void`).
    pub response_type: String,
    /// Name of the grouped request type, e.g. `GetPetByIdData`. `None` when the
    /// operation has no inputs (the client function then takes no argument).
    pub data_type_name: Option<String>,
    /// Name of the response type alias, e.g. `GetPetByIdResponse`.
    pub response_type_name: String,
}

impl OperationPlan {
    pub fn has_inputs(&self) -> bool {
        self.data_type_name.is_some()
    }
    pub fn query_required(&self) -> bool {
        self.query_params.iter().any(|p| p.required)
    }
    pub fn headers_required(&self) -> bool {
        self.header_params.iter().any(|p| p.required)
    }
}

const METHODS: &[&str] = &["get", "post", "put", "patch", "delete"];

/// Build a deterministic plan for every operation in the spec. Per-operation
/// type names are collision-safe against component type names.
pub fn build(spec: &Spec, tm: &TypeMap) -> Vec<OperationPlan> {
    let mut fn_reg = NameRegistry::new();
    let mut type_reg = NameRegistry::new();
    // Reserve component type names so `XxxData`/`XxxResponse` never collide.
    for name in tm.names.values() {
        let _ = type_reg.unique(name);
    }
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
                plans.push(build_one(
                    method,
                    path,
                    op,
                    path_level,
                    tm,
                    &mut fn_reg,
                    &mut type_reg,
                ));
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
    fn_reg: &mut NameRegistry,
    type_reg: &mut NameRegistry,
) -> OperationPlan {
    let fn_name = fn_reg.unique(&op_base_name(method, path, op));

    let mut path_params = Vec::new();
    let mut query_params = Vec::new();
    let mut header_params = Vec::new();

    for p in inline_params(path_level).chain(inline_params(&op.parameters)) {
        let ts_type = p
            .schema
            .as_ref()
            .map(|s| type_expr(s, tm))
            .unwrap_or_else(|| "string".to_string());
        match p.location.as_str() {
            // Path parameters are always required.
            "path" => path_params.push(ParamField {
                name: p.name.clone(),
                ts_type,
                required: true,
            }),
            "query" => query_params.push(ParamField {
                name: p.name.clone(),
                ts_type,
                required: p.required,
            }),
            "header" => header_params.push(ParamField {
                name: p.name.clone(),
                ts_type,
                required: p.required,
            }),
            _ => {} // cookie and unknown locations are out of subset
        }
    }

    let body = op.request_body.as_ref().and_then(|rb| match rb {
        RefOr::Item(rb) => rb
            .content
            .get("application/json")
            .and_then(|mt| mt.schema.as_ref())
            .map(|schema| BodyField {
                ts_type: type_expr(schema, tm),
                required: rb.required,
            }),
        RefOr::Ref(_) => None,
    });

    let response_type = return_type(op, tm);
    let pascal = to_pascal(&fn_name);
    let has_inputs = !path_params.is_empty()
        || !query_params.is_empty()
        || !header_params.is_empty()
        || body.is_some();
    let data_type_name = if has_inputs {
        Some(type_reg.unique(&format!("{pascal}Data")))
    } else {
        None
    };
    let response_type_name = type_reg.unique(&format!("{pascal}Response"));

    OperationPlan {
        fn_name,
        http_method: method.to_uppercase(),
        is_query: method == "get",
        path_raw: path.to_string(),
        path_params,
        query_params,
        header_params,
        body,
        response_type,
        data_type_name,
        response_type_name,
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

fn return_type(op: &Operation, tm: &TypeMap) -> String {
    match pick_response(op) {
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
    use crate::codegen::build_type_map;

    fn spec(json: &str) -> Spec {
        serde_json::from_str(json).unwrap()
    }

    #[test]
    fn fallback_name_from_method_and_path() {
        assert_eq!(fallback_name("get", "/pets/{petId}"), "getPetsByPetId");
        assert_eq!(fallback_name("post", "/pets"), "postPets");
    }

    #[test]
    fn plan_groups_inputs_and_names_types() {
        let s = spec(
            r##"{"paths":{"/pets/{petId}":{"get":{"operationId":"getPetById",
            "parameters":[
              {"name":"petId","in":"path","required":true,"schema":{"type":"integer"}},
              {"name":"expand","in":"query","required":false,"schema":{"type":"boolean"}}],
            "responses":{"200":{"content":{"application/json":{"schema":{"type":"string"}}}}}}}}}"##,
        );
        let tm = build_type_map(&s);
        let plans = build(&s, &tm);
        let p = &plans[0];
        assert_eq!(p.fn_name, "getPetById");
        assert!(p.is_query);
        assert_eq!(p.path_params.len(), 1);
        assert_eq!(p.query_params.len(), 1);
        assert!(!p.query_required());
        assert_eq!(p.data_type_name.as_deref(), Some("GetPetByIdData"));
        assert_eq!(p.response_type_name, "GetPetByIdResponse");
        assert_eq!(p.response_type, "string");
    }

    #[test]
    fn no_input_operation_has_no_data_type() {
        let s = spec(
            r##"{"paths":{"/health":{"get":{"operationId":"health","responses":{"200":{"content":{"application/json":{"schema":{"type":"boolean"}}}}}}}}}"##,
        );
        let plans = build(&s, &build_type_map(&s));
        assert!(plans[0].data_type_name.is_none());
        assert!(!plans[0].has_inputs());
        assert_eq!(plans[0].response_type_name, "HealthResponse");
    }

    #[test]
    fn per_op_type_names_avoid_component_collision() {
        // A component literally named "GetThingData" must not clash with the
        // synthesized data type for operationId "getThing".
        let s = spec(
            r##"{"paths":{"/thing":{"post":{"operationId":"getThing",
              "requestBody":{"required":true,"content":{"application/json":{"schema":{"type":"object"}}}},
              "responses":{"200":{"content":{"application/json":{"schema":{"type":"string"}}}}}}}},
            "components":{"schemas":{"GetThingData":{"type":"object"}}}}"##,
        );
        let tm = build_type_map(&s);
        let plans = build(&s, &tm);
        assert_eq!(
            tm.names.get("GetThingData").map(String::as_str),
            Some("GetThingData")
        );
        assert_eq!(plans[0].data_type_name.as_deref(), Some("GetThingData_2"));
    }
}
// HANDWRITE-END
