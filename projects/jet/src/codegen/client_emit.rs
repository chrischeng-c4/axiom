// SPEC-MANAGED: .aw/tech-design/projects/jet/interfaces/cli/openapi-client-codegen-types-fetch-client-react-query-hooks.md#logic
// HANDWRITE-BEGIN
//! Emits `runtime.ts` (the fetch base) and `client.ts` (a `createClient`
//! factory with one typed function per operation).

use crate::codegen::plan::OperationPlan;
use crate::codegen::tsmap::TypeMap;
use crate::codegen::types_emit::HEADER;
use crate::codegen::{GenOptions, HttpClient};

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
/// @spec .aw/tech-design/projects/jet/interfaces/cli/openapi-client-codegen-types-fetch-client-react-query-hooks.md#logic
pub fn emit_client(plans: &[OperationPlan], tm: &TypeMap, opts: &GenOptions) -> String {
    let mut out = String::from(HEADER);
    out.push_str("import type { ClientConfig } from \"./runtime\";\n");
    out.push_str("import { request } from \"./runtime\";\n");
    out.push_str(&type_import(tm));
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
    let sig = match p.params_type() {
        Some(t) => format!("(params: {t})"),
        None => "()".to_string(),
    };

    let mut args = vec![
        format!("method: \"{}\"", p.http_method),
        format!("path: {}", p.path_template),
    ];
    if !p.query_pairs.is_empty() {
        let entries = p
            .query_pairs
            .iter()
            .map(|(k, access)| format!("{}: {}", crate::codegen::names::prop_key(k), access))
            .collect::<Vec<_>>()
            .join(", ");
        args.push(format!("query: {{ {entries} }}"));
    }
    if !p.header_pairs.is_empty() {
        let entries = p
            .header_pairs
            .iter()
            .map(|(k, access)| {
                format!("{}: String({})", crate::codegen::names::prop_key(k), access)
            })
            .collect::<Vec<_>>()
            .join(", ");
        args.push(format!("headers: {{ {entries} }}"));
    }
    if p.has_body {
        args.push("body: params.body".to_string());
    }

    format!(
        "    {name}{sig}: Promise<{ret}> {{\n      return request<{ret}>(config, {{ {args} }});\n    }},\n",
        name = p.fn_name,
        sig = sig,
        ret = p.return_type,
        args = args.join(", "),
    )
}

/// `import type { A, B } from "./types";` for all component type names.
pub fn type_import(tm: &TypeMap) -> String {
    if tm.names.is_empty() {
        return String::new();
    }
    let mut names: Vec<&String> = tm.names.values().collect();
    names.sort();
    names.dedup();
    let list = names
        .iter()
        .map(|n| n.as_str())
        .collect::<Vec<_>>()
        .join(", ");
    format!("import type {{ {list} }} from \"./types\";\n")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::codegen::openapi::Spec;
    use crate::codegen::plan;
    use crate::codegen::{build_type_map, GenOptions, HttpClient};
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
        }
    }

    #[test]
    fn client_method_with_path_param_and_body() {
        let s: Spec = serde_json::from_str(
            r##"{"components":{"schemas":{"Pet":{"type":"object","properties":{"id":{"type":"integer"}}}}},
            "paths":{"/pets":{"post":{"operationId":"createPet",
              "requestBody":{"required":true,"content":{"application/json":{"schema":{"$ref":"#/components/schemas/Pet"}}}},
              "responses":{"201":{"content":{"application/json":{"schema":{"$ref":"#/components/schemas/Pet"}}}}}}}}}"##,
        )
        .unwrap();
        let tm = build_type_map(&s);
        let plans = plan::build(&s, &tm);
        let out = emit_client(&plans, &tm, &opts());
        assert!(out.contains("import type { Pet } from \"./types\";"));
        assert!(out.contains("export function createClient(config: ClientConfig) {"));
        assert!(out.contains("createPet(params: { body: Pet }): Promise<Pet> {"));
        assert!(out.contains(
            "return request<Pet>(config, { method: \"POST\", path: `/pets`, body: params.body });"
        ));
        assert!(out.contains("export type ApiClient = ReturnType<typeof createClient>;"));
    }

    #[test]
    fn runtime_has_request_helper() {
        let rt = emit_runtime(HttpClient::Fetch);
        assert!(rt
            .contains("export async function request<T>(config: ClientConfig, args: RequestArgs)"));
        assert!(rt.contains("if (response.status === 204)"));
        assert!(rt.contains("const doFetch = config.fetch ?? fetch;"));
    }

    #[test]
    fn axios_runtime_uses_axios() {
        let rt = emit_runtime(HttpClient::Axios);
        assert!(rt.contains("import axios from \"axios\";"));
        assert!(rt.contains("import type { AxiosInstance } from \"axios\";"));
        assert!(rt.contains("axios?: AxiosInstance;"));
        assert!(rt.contains("config.axios ?? axios.create()"));
        assert!(rt.contains("return response.data;"));
        // Same request/ClientConfig contract as the fetch runtime.
        assert!(rt
            .contains("export async function request<T>(config: ClientConfig, args: RequestArgs)"));
    }
}
// HANDWRITE-END
