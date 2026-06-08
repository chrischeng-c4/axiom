//! RPC API section generator.
//!
//! Produces an OpenRPC 1.3 YAML skeleton with method_name, params,
//! and result schema. Includes `x-sdd` metadata injection.

use super::{Generator, GeneratorArgs};
use crate::models::spec_rules::SectionType;

/// Generator for `<!-- type: rpc-api lang: yaml -->` sections.

// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/generate/generators/rpc_api_source.md#source
// CODEGEN-BEGIN
/// @spec projects/agentic-workflow/tech-design/core/generate/generators/rpc_api_source.md#source
impl Generator for RpcApiGenerator {
    fn section_type(&self) -> SectionType {
        SectionType::RpcApi
    }

    fn generate(&self, args: &GeneratorArgs) -> String {
        let sdd_id = args.sdd_id.as_deref().unwrap_or("TODO");
        let refs_yaml = if args.sdd_refs.is_empty() {
            String::new()
        } else {
            let refs: Vec<String> = args
                .sdd_refs
                .iter()
                .map(|r| format!("    - {}", r))
                .collect();
            format!("\n  refs:\n{}", refs.join("\n"))
        };

        format!(
            "```yaml\n\
openrpc: \"1.3.2\"\n\
info:\n\
  title: \"[API Title]\"\n\
  version: \"1.0.0\"\n\
  x-sdd:\n\
    id: \"{sdd_id}\"{refs_yaml}\n\
methods:\n\
  - name: \"[method_name]\"\n\
    summary: \"[Description]\"\n\
    params:\n\
      - name: param1\n\
        required: true\n\
        schema:\n\
          type: string\n\
    result:\n\
      name: result\n\
      schema:\n\
        type: object\n\
        properties:\n\
          success:\n\
            type: boolean\n\
```"
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::generators::{Generator, GeneratorArgs};
    use crate::models::spec_rules::SectionType;

    #[test]
    fn test_section_type() {
        assert_eq!(RpcApiGenerator {}.section_type(), SectionType::RpcApi);
    }

    #[test]
    fn test_generate_contains_openrpc() {
        let args = GeneratorArgs::new(SectionType::RpcApi).with_sdd_id("my-rpc");
        let output = RpcApiGenerator {}.generate(&args);
        assert!(output.contains("```yaml"));
        assert!(output.contains("openrpc: \"1.3.2\""));
        assert!(output.contains("x-sdd:"));
        assert!(output.contains("id: \"my-rpc\""));
        assert!(output.contains("methods:"));
    }

    #[test]
    fn test_generate_with_refs() {
        let args = GeneratorArgs::new(SectionType::RpcApi)
            .with_sdd_id("my-rpc")
            .with_sdd_refs(vec!["schema-spec".to_string()]);
        let output = RpcApiGenerator {}.generate(&args);
        assert!(output.contains("schema-spec"));
    }

    #[test]
    fn test_with_annotation() {
        let args = GeneratorArgs::new(SectionType::RpcApi);
        let output = RpcApiGenerator {}.generate_with_annotation(&args);
        assert!(output.starts_with("<!-- type: rpc-api lang: yaml -->"));
    }
}
// CODEGEN-END
// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/generate/generators/rpc_api_types.md#schema
// CODEGEN-BEGIN
/// RpcApiGenerator unit struct.
/// @spec projects/agentic-workflow/tech-design/core/generate/generators/rpc_api_types.md#schema
pub struct RpcApiGenerator {}
// CODEGEN-END
