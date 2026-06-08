---
id: sdd-generate-generators-rest-api-source
fill_sections: [overview, source, changes]
capability_refs:
  - id: td-cb-lifecycle-automation
    role: primary
    gap: cb-lifecycle-dispatch
    claim: cb-lifecycle-dispatch
    coverage: full
    rationale: "Generator primitives are part of TD/CB lifecycle automation because they produce reviewable code artifacts from TD sections."
---

# RestApiGenerator Source

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/agentic-workflow/src/generators/rest_api.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `RestApiGenerator` | projects/agentic-workflow/src/generators/rest_api.rs | struct | pub | 128 |  |
## Source
<!-- type: source lang: rust -->
<!-- source-from-target: handwrite-gap standardize:fold-shadow -->

<!-- source-snapshot: path=projects/agentic-workflow/src/generators/rest_api.rs -->
````rust
/// @spec projects/agentic-workflow/tech-design/core/generate/generators/rest_api_source.md#source
impl Generator for RestApiGenerator {
    fn section_type(&self) -> SectionType {
        SectionType::RestApi
    }

    fn generate(&self, args: &GeneratorArgs) -> String {
        let sdd_id = args.sdd_id.as_deref().unwrap_or("TODO");
        let refs_yaml = if args.sdd_refs.is_empty() {
            String::new()
        } else {
            let refs: Vec<String> = args
                .sdd_refs
                .iter()
                .map(|r| format!("    - \"{}\"", r))
                .collect();
            format!("\n    refs:\n{}", refs.join("\n"))
        };

        format!(
            "```yaml\n\
openapi: \"3.1.0\"\n\
info:\n\
  title: \"[API Title]\"\n\
  version: \"1.0.0\"\n\
  x-sdd:\n\
    id: \"{sdd_id}\"{refs_yaml}\n\
paths:\n\
  /resource:\n\
    get:\n\
      summary: \"[Description]\"\n\
      operationId: getResource\n\
      responses:\n\
        \"200\":\n\
          description: Success\n\
          content:\n\
            application/json:\n\
              schema:\n\
                $ref: \"#/components/schemas/Resource\"\n\
    post:\n\
      summary: \"[Description]\"\n\
      operationId: createResource\n\
      requestBody:\n\
        required: true\n\
        content:\n\
          application/json:\n\
            schema:\n\
              $ref: \"#/components/schemas/CreateRequest\"\n\
      responses:\n\
        \"201\":\n\
          description: Created\n\
components:\n\
  schemas:\n\
    Resource:\n\
      type: object\n\
      properties:\n\
        id:\n\
          type: string\n\
        name:\n\
          type: string\n\
      required:\n\
        - id\n\
    CreateRequest:\n\
      type: object\n\
      properties:\n\
        name:\n\
          type: string\n\
      required:\n\
        - name\n\
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
        assert_eq!(RestApiGenerator {}.section_type(), SectionType::RestApi);
    }

    #[test]
    fn test_generate_contains_openapi() {
        let args = GeneratorArgs::new(SectionType::RestApi).with_sdd_id("my-api");
        let output = RestApiGenerator {}.generate(&args);
        assert!(output.contains("```yaml"));
        assert!(output.contains("openapi: \"3.1.0\""));
        assert!(output.contains("x-sdd:"));
        assert!(output.contains("id: \"my-api\""));
        assert!(output.contains("paths:"));
    }

    #[test]
    fn test_generate_with_refs() {
        let args = GeneratorArgs::new(SectionType::RestApi)
            .with_sdd_id("my-api")
            .with_sdd_refs(vec!["data-model".to_string()]);
        let output = RestApiGenerator {}.generate(&args);
        assert!(output.contains("\"data-model\""));
    }

    #[test]
    fn test_with_annotation() {
        let args = GeneratorArgs::new(SectionType::RestApi);
        let output = RestApiGenerator {}.generate_with_annotation(&args);
        assert!(output.starts_with("<!-- type: rest-api lang: yaml -->"));
    }
}
````

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/generators/rest_api.rs
    action: modify
    section: source
    impl_mode: codegen
    replaces:
      - "<handwrite-gap:standardize:fold-shadow>"
    description: "Source template owns the REST API section generator behavior and tests."
```
