---
id: sdd-generate-generators-async-api-source
fill_sections: [overview, source, changes]
capability_refs:
  - id: td-cb-lifecycle-automation
    role: primary
    gap: cb-lifecycle-dispatch
    claim: cb-lifecycle-dispatch
    coverage: full
    rationale: "Generator primitives are part of TD/CB lifecycle automation because they produce reviewable code artifacts from TD sections."
---

# AsyncApiGenerator Source

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/agentic-workflow/src/generators/async_api.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `AsyncApiGenerator` | projects/agentic-workflow/src/generators/async_api.rs | struct | pub | 110 |  |
## Source
<!-- type: source lang: rust -->
<!-- source-from-target: handwrite-gap standardize:fold-shadow -->

<!-- source-snapshot: path=projects/agentic-workflow/src/generators/async_api.rs -->
````rust
/// @spec projects/agentic-workflow/tech-design/core/generate/generators/async_api_source.md#source
impl Generator for AsyncApiGenerator {
    fn section_type(&self) -> SectionType {
        SectionType::AsyncApi
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
asyncapi: \"2.6.0\"\n\
info:\n\
  title: \"[API Title]\"\n\
  version: \"1.0.0\"\n\
  x-sdd:\n\
    id: \"{sdd_id}\"{refs_yaml}\n\
channels:\n\
  events/resource-created:\n\
    publish:\n\
      summary: \"[Description]\"\n\
      message:\n\
        payload:\n\
          type: object\n\
          properties:\n\
            id:\n\
              type: string\n\
            timestamp:\n\
              type: string\n\
              format: date-time\n\
  events/resource-updated:\n\
    publish:\n\
      summary: \"[Description]\"\n\
      message:\n\
        payload:\n\
          type: object\n\
          properties:\n\
            id:\n\
              type: string\n\
            changes:\n\
              type: object\n\
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
        assert_eq!(AsyncApiGenerator {}.section_type(), SectionType::AsyncApi);
    }

    #[test]
    fn test_generate_contains_asyncapi() {
        let args = GeneratorArgs::new(SectionType::AsyncApi).with_sdd_id("my-events");
        let output = AsyncApiGenerator {}.generate(&args);
        assert!(output.contains("```yaml"));
        assert!(output.contains("asyncapi: \"2.6.0\""));
        assert!(output.contains("x-sdd:"));
        assert!(output.contains("id: \"my-events\""));
        assert!(output.contains("channels:"));
    }

    #[test]
    fn test_generate_with_refs() {
        let args = GeneratorArgs::new(SectionType::AsyncApi)
            .with_sdd_id("my-events")
            .with_sdd_refs(vec!["event-schema".to_string()]);
        let output = AsyncApiGenerator {}.generate(&args);
        assert!(output.contains("\"event-schema\""));
    }

    #[test]
    fn test_with_annotation() {
        let args = GeneratorArgs::new(SectionType::AsyncApi);
        let output = AsyncApiGenerator {}.generate_with_annotation(&args);
        assert!(output.starts_with("<!-- type: async-api lang: yaml -->"));
    }
}
````

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/generators/async_api.rs
    action: modify
    section: source
    impl_mode: codegen
    replaces:
      - "<handwrite-gap:standardize:fold-shadow>"
    description: "Source template owns the AsyncAPI section generator behavior and tests."
```
