// SPEC-MANAGED: .aw/tech-design/projects/jet/interfaces/cli/named-per-operation-request-response-types-xxxdata-xxxresponse-f.md#logic
// <HANDWRITE gap="standardize:projects-jet-src-codegen-hooks-emit-rs" tracker="standardize-gap-projects-jet-src-codegen-hooks-emit-rs" reason="Existing hand-written code in projects/jet/src/codegen/hooks_emit.rs requires tracked generator coverage.">
//! Emits `hooks.ts`: TanStack Query (React Query) hooks bound to the client,
//! using the per-operation `XxxData` / `XxxResponse` types.
//!
//! `@tanstack/react-query` is a peer dependency of the *generated output*, not
//! of jet — only `import` statements reference it.

use crate::codegen::client_emit::type_import;
use crate::codegen::names::to_pascal;
use crate::codegen::plan::OperationPlan;

/// Render `hooks.ts`.
///
/// @spec .aw/tech-design/projects/jet/interfaces/cli/named-per-operation-request-response-types-xxxdata-xxxresponse-f.md#logic
pub fn emit(plans: &[OperationPlan]) -> String {
    let mut out = String::from(crate::codegen::types_emit::HEADER);
    out.push_str("import { useMutation, useQuery } from \"@tanstack/react-query\";\n");
    out.push_str(
        "import type { UseMutationOptions, UseQueryOptions } from \"@tanstack/react-query\";\n",
    );
    out.push_str("import type { ApiClient } from \"./client\";\n");
    out.push_str(&type_import(plans));
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
    let stem = to_pascal(&p.fn_name);
    let resp = &p.response_type_name;
    let fn_name = &p.fn_name;
    if p.is_query {
        match &p.data_type_name {
            Some(data) => format!(
                "    use{stem}Query(data: {data}, options?: Omit<UseQueryOptions<{resp}>, \"queryKey\" | \"queryFn\">) {{\n\
                 \x20     return useQuery<{resp}>({{ queryKey: [\"{fn_name}\", data], queryFn: () => client.{fn_name}(data), ...options }});\n\
                 \x20   }},\n",
            ),
            None => format!(
                "    use{stem}Query(options?: Omit<UseQueryOptions<{resp}>, \"queryKey\" | \"queryFn\">) {{\n\
                 \x20     return useQuery<{resp}>({{ queryKey: [\"{fn_name}\"], queryFn: () => client.{fn_name}(), ...options }});\n\
                 \x20   }},\n",
            ),
        }
    } else {
        match &p.data_type_name {
            Some(data) => format!(
                "    use{stem}Mutation(options?: UseMutationOptions<{resp}, Error, {data}>) {{\n\
                 \x20     return useMutation<{resp}, Error, {data}>({{ mutationFn: (data) => client.{fn_name}(data), ...options }});\n\
                 \x20   }},\n",
            ),
            None => format!(
                "    use{stem}Mutation(options?: UseMutationOptions<{resp}, Error, void>) {{\n\
                 \x20     return useMutation<{resp}, Error, void>({{ mutationFn: () => client.{fn_name}(), ...options }});\n\
                 \x20   }},\n",
            ),
        }
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
        emit(&plans)
    }

    #[test]
    fn get_becomes_query_hook_with_data() {
        let out = render(
            r##"{"paths":{"/pets/{petId}":{"get":{"operationId":"getPetById",
            "parameters":[{"name":"petId","in":"path","required":true,"schema":{"type":"integer"}}],
            "responses":{"200":{"content":{"application/json":{"schema":{"type":"string"}}}}}}}}}"##,
        );
        assert!(
            out.contains("import type { GetPetByIdData, GetPetByIdResponse } from \"./types\";")
        );
        assert!(out.contains("useGetPetByIdQuery(data: GetPetByIdData, options?: Omit<UseQueryOptions<GetPetByIdResponse>"));
        assert!(out.contains("queryKey: [\"getPetById\", data]"));
        assert!(out.contains("queryFn: () => client.getPetById(data)"));
    }

    #[test]
    fn post_becomes_mutation_hook_with_data() {
        let out = render(
            r##"{"paths":{"/pets":{"post":{"operationId":"createPet",
            "requestBody":{"required":true,"content":{"application/json":{"schema":{"type":"object"}}}},
            "responses":{"201":{"content":{"application/json":{"schema":{"type":"object"}}}}}}}}}"##,
        );
        assert!(out.contains(
            "useCreatePetMutation(options?: UseMutationOptions<CreatePetResponse, Error, CreatePetData>)"
        ));
        assert!(out.contains("mutationFn: (data) => client.createPet(data)"));
    }

    #[test]
    fn no_input_query_hook_omits_data() {
        let out = render(
            r##"{"paths":{"/health":{"get":{"operationId":"health","responses":{"200":{"content":{"application/json":{"schema":{"type":"boolean"}}}}}}}}}"##,
        );
        assert!(out.contains("useHealthQuery(options?: Omit<UseQueryOptions<HealthResponse>"));
        assert!(out.contains("queryKey: [\"health\"]"));
        assert!(out.contains("queryFn: () => client.health()"));
    }
}
// </HANDWRITE>
