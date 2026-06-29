// SPEC-MANAGED: .aw/tech-design/projects/jet/interfaces/cli/named-per-operation-request-response-types-xxxdata-xxxresponse-f.md#logic
// <HANDWRITE gap="standardize:projects-jet-src-codegen-client-emit-rs" tracker="standardize-gap-projects-jet-src-codegen-client-emit-rs" reason="Existing hand-written code in projects/jet/src/codegen/client_emit.rs requires tracked generator coverage.">
//! Emits `runtime.ts` (the fetch or axios base) and `client.ts` (a `createClient`
//! factory with one typed function per operation, taking a grouped `data` arg).

use crate::codegen::names::{self, is_ident};
use crate::codegen::plan::OperationPlan;
use crate::codegen::types_emit::HEADER;
use crate::codegen::{GenOptions, HttpClient};
use std::collections::BTreeSet;

/// Static request runtime shared by every generated client. The body depends
/// only on the chosen [`HttpClient`] backend — the `ClientConfig`/`request`
/// contract is the same, so `client.ts` and `hooks.ts` never change.
///
/// @spec .aw/tech-design/projects/jet/interfaces/cli/select-http-client-backend-fetch-axios-for-openapi-codegen.md#logic
pub fn emit_runtime(http_client: HttpClient) -> String {
    let body = match http_client {
        HttpClient::Fetch => FETCH_RUNTIME,
        HttpClient::Axios => AXIOS_RUNTIME,
    };
    format!("{HEADER}{body}")
}

const FETCH_RUNTIME: &str = r##"export interface ClientConfig {
  baseUrl: string;
  fetch?: typeof fetch;
  headers?: Record<string, string>;
}

export interface RequestArgs {
  method: string;
  path: string;
  query?: Record<string, unknown>;
  body?: unknown;
  headers?: Record<string, string>;
}

export async function request<T>(config: ClientConfig, args: RequestArgs): Promise<T> {
  const doFetch = config.fetch ?? fetch;
  const url = new URL(config.baseUrl + args.path);
  if (args.query) {
    for (const [key, value] of Object.entries(args.query)) {
      if (value !== undefined && value !== null) {
        url.searchParams.set(key, String(value));
      }
    }
  }
  const response = await doFetch(url.toString(), {
    method: args.method,
    headers: { "Content-Type": "application/json", ...config.headers, ...args.headers },
    body: args.body !== undefined ? JSON.stringify(args.body) : undefined,
  });
  if (!response.ok) {
    throw new Error(`HTTP ${response.status}`);
  }
  if (response.status === 204) {
    return undefined as T;
  }
  return (await response.json()) as T;
}
"##;

const AXIOS_RUNTIME: &str = r##"import axios from "axios";
import type { AxiosInstance } from "axios";

export interface ClientConfig {
  baseUrl: string;
  axios?: AxiosInstance;
  headers?: Record<string, string>;
}

export interface RequestArgs {
  method: string;
  path: string;
  query?: Record<string, unknown>;
  body?: unknown;
  headers?: Record<string, string>;
}

export async function request<T>(config: ClientConfig, args: RequestArgs): Promise<T> {
  const instance = config.axios ?? axios.create();
  const response = await instance.request<T>({
    method: args.method,
    baseURL: config.baseUrl,
    url: args.path,
    params: args.query,
    data: args.body,
    headers: { "Content-Type": "application/json", ...config.headers, ...args.headers },
  });
  return response.data;
}
"##;

/// Render `client.ts`.
///
/// @spec .aw/tech-design/projects/jet/interfaces/cli/named-per-operation-request-response-types-xxxdata-xxxresponse-f.md#logic
pub fn emit_client(plans: &[OperationPlan], opts: &GenOptions) -> String {
    let mut out = String::from(HEADER);
    out.push_str("import type { ClientConfig } from \"./runtime\";\n");
    out.push_str("import { request } from \"./runtime\";\n");
    out.push_str(&type_import(plans));
    out.push('\n');

    let factory = &opts.client_name;
    out.push_str(&format!(
        "export function {factory}(config: ClientConfig) {{\n"
    ));
    out.push_str("  return {\n");
    for p in plans {
        out.push_str(&emit_method(p));
    }
    out.push_str("  };\n");
    out.push_str("}\n\n");
    out.push_str(&format!(
        "export type ApiClient = ReturnType<typeof {factory}>;\n"
    ));
    out
}

fn emit_method(p: &OperationPlan) -> String {
    let sig = match &p.data_type_name {
        Some(name) => format!("(data: {name})"),
        None => "()".to_string(),
    };

    let mut args = vec![
        format!("method: \"{}\"", p.http_method),
        format!("path: {}", path_template(p)),
    ];
    if !p.query_params.is_empty() {
        let entries = p
            .query_params
            .iter()
            .map(|f| {
                format!(
                    "{}: {}",
                    names::prop_key(&f.name),
                    access("data.query", !p.query_required(), &f.name)
                )
            })
            .collect::<Vec<_>>()
            .join(", ");
        args.push(format!("query: {{ {entries} }}"));
    }
    if !p.header_params.is_empty() {
        let entries = p
            .header_params
            .iter()
            .map(|f| {
                format!(
                    "{}: String({})",
                    names::prop_key(&f.name),
                    access("data.headers", !p.headers_required(), &f.name)
                )
            })
            .collect::<Vec<_>>()
            .join(", ");
        args.push(format!("headers: {{ {entries} }}"));
    }
    if p.body.is_some() {
        args.push("body: data.body".to_string());
    }

    let resp = &p.response_type_name;
    format!(
        "    {name}{sig}: Promise<{resp}> {{\n      return request<{resp}>(config, {{ {args} }});\n    }},\n",
        name = p.fn_name,
        sig = sig,
        resp = resp,
        args = args.join(", "),
    )
}

/// `/pets/{petId}` → `` `/pets/${data.path.petId}` ``.
fn path_template(p: &OperationPlan) -> String {
    let mut out = String::from("`");
    let mut chars = p.path_raw.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '{' {
            let mut name = String::new();
            for c in chars.by_ref() {
                if c == '}' {
                    break;
                }
                name.push(c);
            }
            out.push_str("${");
            out.push_str(&access("data.path", false, &name));
            out.push('}');
        } else {
            out.push(c);
        }
    }
    out.push('`');
    out
}

/// Member access against a grouped sub-object, e.g. `data.query?.limit` or
/// `data.headers["X-Id"]`.
fn access(base: &str, optional: bool, name: &str) -> String {
    if is_ident(name) {
        if optional {
            format!("{base}?.{name}")
        } else {
            format!("{base}.{name}")
        }
    } else {
        let key = name.replace('\\', "\\\\").replace('"', "\\\"");
        if optional {
            format!("{base}?.[\"{key}\"]")
        } else {
            format!("{base}[\"{key}\"]")
        }
    }
}

/// `import type { ... } from "./types";` for the per-operation type names.
pub fn type_import(plans: &[OperationPlan]) -> String {
    let mut names: BTreeSet<String> = BTreeSet::new();
    for p in plans {
        if let Some(d) = &p.data_type_name {
            names.insert(d.clone());
        }
        names.insert(p.response_type_name.clone());
    }
    if names.is_empty() {
        return String::new();
    }
    let list = names.into_iter().collect::<Vec<_>>().join(", ");
    format!("import type {{ {list} }} from \"./types\";\n")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::codegen::openapi::Spec;
    use crate::codegen::plan;
    use crate::codegen::{build_type_map, GenOptions, HookRuntime};
    use std::path::PathBuf;

    fn opts() -> GenOptions {
        GenOptions {
            spec_path: PathBuf::new(),
            out_dir: PathBuf::new(),
            client_name: "createClient".to_string(),
            http_client: HttpClient::Fetch,
            emit_types: true,
            emit_client: true,
            emit_hooks: true,
            hooks_runtime: HookRuntime::ReactQuery,
        }
    }

    fn render(json: &str) -> String {
        let s: Spec = serde_json::from_str(json).unwrap();
        let tm = build_type_map(&s);
        let plans = plan::build(&s, &tm);
        emit_client(&plans, &opts())
    }

    #[test]
    fn client_method_takes_grouped_data() {
        let out = render(
            r##"{"components":{"schemas":{"Pet":{"type":"object","properties":{"id":{"type":"integer"}}}}},
            "paths":{"/pets/{petId}":{"get":{"operationId":"getPetById",
              "parameters":[{"name":"petId","in":"path","required":true,"schema":{"type":"integer"}}],
              "responses":{"200":{"content":{"application/json":{"schema":{"$ref":"#/components/schemas/Pet"}}}}}}}}}"##,
        );
        assert!(
            out.contains("import type { GetPetByIdData, GetPetByIdResponse } from \"./types\";")
        );
        assert!(out.contains("getPetById(data: GetPetByIdData): Promise<GetPetByIdResponse> {"));
        assert!(out.contains("return request<GetPetByIdResponse>(config, { method: \"GET\", path: `/pets/${data.path.petId}` });"));
    }

    #[test]
    fn client_query_and_body_access() {
        let out = render(
            r##"{"paths":{"/pets":{
              "get":{"operationId":"listPets","parameters":[{"name":"limit","in":"query","required":false,"schema":{"type":"integer"}}],
                "responses":{"200":{"content":{"application/json":{"schema":{"type":"array","items":{"type":"string"}}}}}}},
              "post":{"operationId":"createPet","requestBody":{"required":true,"content":{"application/json":{"schema":{"type":"object"}}}},
                "responses":{"201":{"content":{"application/json":{"schema":{"type":"string"}}}}}}}}}"##,
        );
        assert!(out.contains("listPets(data: ListPetsData): Promise<ListPetsResponse> {"));
        assert!(out.contains("query: { limit: data.query?.limit }"));
        assert!(out.contains("createPet(data: CreatePetData): Promise<CreatePetResponse> {"));
        assert!(out.contains("body: data.body"));
    }

    #[test]
    fn no_input_operation_takes_no_arg() {
        let out = render(
            r##"{"paths":{"/health":{"get":{"operationId":"health","responses":{"200":{"content":{"application/json":{"schema":{"type":"boolean"}}}}}}}}}"##,
        );
        assert!(out.contains("health(): Promise<HealthResponse> {"));
        assert!(out.contains("import type { HealthResponse } from \"./types\";"));
    }

    #[test]
    fn runtime_fetch_and_axios() {
        let fetch = emit_runtime(HttpClient::Fetch);
        assert!(fetch.contains("const doFetch = config.fetch ?? fetch;"));
        assert!(!fetch.contains("axios"));
        let axios = emit_runtime(HttpClient::Axios);
        assert!(axios.contains("import axios from \"axios\";"));
        assert!(axios.contains("config.axios ?? axios.create()"));
        assert!(axios.contains("return response.data;"));
    }
}
// </HANDWRITE>
