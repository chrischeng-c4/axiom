// SPEC-MANAGED: .aw/tech-design/projects/jet/interfaces/cli/openapi-client-codegen-types-fetch-client-react-query-hooks.md#logic
// HANDWRITE-BEGIN
//! Emits `hooks.ts`: TanStack Query (React Query) hooks bound to the client.
//!
//! `@tanstack/react-query` is a peer dependency of the *generated output*, not
//! of jet — only `import` statements reference it.

use crate::codegen::client_emit::type_import;
use crate::codegen::names::to_pascal;
use crate::codegen::plan::OperationPlan;
use crate::codegen::tsmap::TypeMap;
use crate::codegen::types_emit::HEADER;

/// Render `hooks.ts`.
///
/// @spec .aw/tech-design/projects/jet/interfaces/cli/openapi-client-codegen-types-fetch-client-react-query-hooks.md#logic
pub fn emit(plans: &[OperationPlan], tm: &TypeMap) -> String {
    let mut out = String::from(HEADER);
    out.push_str("import { useMutation, useQuery } from \"@tanstack/react-query\";\n");
    out.push_str(
        "import type { UseMutationOptions, UseQueryOptions } from \"@tanstack/react-query\";\n",
    );
    out.push_str("import type { ApiClient } from \"./client\";\n");
    out.push_str(&type_import(tm));
    out.push('\n');

    out.push_str("export function createHooks(client: ApiClient) {\n");
    out.push_str("  return {\n");
    for p in plans {
        out.push_str(&emit_hook(p));
    }
    out.push_str("  };\n");
    out.push_str("}\n");
    out
}

fn emit_hook(p: &OperationPlan) -> String {
    let hook_stem = to_pascal(&p.fn_name);
    let ret = &p.return_type;
    if p.is_query {
        let key = query_key(p);
        match p.params_type() {
            Some(params_ty) => format!(
                "    use{stem}Query(params: {params_ty}, options?: Omit<UseQueryOptions<{ret}>, \"queryKey\" | \"queryFn\">) {{\n\
                 \x20     return useQuery<{ret}>({{ queryKey: {key}, queryFn: () => client.{fn}(params), ...options }});\n\
                 \x20   }},\n",
                stem = hook_stem,
                params_ty = params_ty,
                ret = ret,
                key = key,
                fn = p.fn_name,
            ),
            None => format!(
                "    use{stem}Query(options?: Omit<UseQueryOptions<{ret}>, \"queryKey\" | \"queryFn\">) {{\n\
                 \x20     return useQuery<{ret}>({{ queryKey: {key}, queryFn: () => client.{fn}(), ...options }});\n\
                 \x20   }},\n",
                stem = hook_stem,
                ret = ret,
                key = key,
                fn = p.fn_name,
            ),
        }
    } else {
        match p.params_type() {
            Some(params_ty) => format!(
                "    use{stem}Mutation(options?: UseMutationOptions<{ret}, Error, {vars}>) {{\n\
                 \x20     return useMutation<{ret}, Error, {vars}>({{ mutationFn: (variables) => client.{fn}(variables), ...options }});\n\
                 \x20   }},\n",
                stem = hook_stem,
                ret = ret,
                vars = params_ty,
                fn = p.fn_name,
            ),
            None => format!(
                "    use{stem}Mutation(options?: UseMutationOptions<{ret}, Error, void>) {{\n\
                 \x20     return useMutation<{ret}, Error, void>({{ mutationFn: () => client.{fn}(), ...options }});\n\
                 \x20   }},\n",
                stem = hook_stem,
                ret = ret,
                fn = p.fn_name,
            ),
        }
    }
}

fn query_key(p: &OperationPlan) -> String {
    if p.param_fields.is_empty() {
        format!("[\"{}\"]", p.fn_name)
    } else {
        format!("[\"{}\", params]", p.fn_name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::codegen::openapi::Spec;
    use crate::codegen::{build_type_map, plan};

    fn render(json: &str) -> String {
        let s: Spec = serde_json::from_str(json).unwrap();
        let tm = build_type_map(&s);
        let plans = plan::build(&s, &tm);
        emit(&plans, &tm)
    }

    #[test]
    fn get_becomes_query_hook() {
        let out = render(
            r##"{"paths":{"/pets/{petId}":{"get":{"operationId":"getPetById",
            "parameters":[{"name":"petId","in":"path","required":true,"schema":{"type":"integer"}}],
            "responses":{"200":{"content":{"application/json":{"schema":{"type":"string"}}}}}}}}}"##,
        );
        assert!(out.contains("import { useMutation, useQuery } from \"@tanstack/react-query\";"));
        assert!(out.contains("useGetPetByIdQuery(params: { petId: number }"));
        assert!(out.contains("queryKey: [\"getPetById\", params]"));
        assert!(out.contains("queryFn: () => client.getPetById(params)"));
    }

    #[test]
    fn post_becomes_mutation_hook() {
        let out = render(
            r##"{"paths":{"/pets":{"post":{"operationId":"createPet",
            "requestBody":{"required":true,"content":{"application/json":{"schema":{"type":"object"}}}},
            "responses":{"201":{"content":{"application/json":{"schema":{"type":"object"}}}}}}}}}"##,
        );
        assert!(out.contains("useCreatePetMutation(options?: UseMutationOptions<"));
        assert!(out.contains("mutationFn: (variables) => client.createPet(variables)"));
    }
}
// HANDWRITE-END
