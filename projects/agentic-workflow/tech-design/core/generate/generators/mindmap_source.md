---
id: sdd-generate-generators-mindmap-source
fill_sections: [overview, source, changes]
capability_refs:
  - id: td-cb-lifecycle-automation
    role: primary
    gap: cb-lifecycle-dispatch
    claim: cb-lifecycle-dispatch
    coverage: full
    rationale: "Generator primitives are part of TD/CB lifecycle automation because they produce reviewable code artifacts from TD sections."
---

# MindmapGenerator Source

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/agentic-workflow/src/generators/mindmap.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `MindmapGenerator` | projects/agentic-workflow/src/generators/mindmap.rs | struct | pub | 90 |  |
## Source
<!-- type: source lang: rust -->
<!-- source-from-target: handwrite-gap standardize:fold-shadow -->

<!-- source-snapshot: path=projects/agentic-workflow/src/generators/mindmap.rs -->
````rust
/// @spec projects/agentic-workflow/tech-design/core/generate/generators/mindmap_source.md#source
impl Generator for MindmapGenerator {
    fn section_type(&self) -> SectionType {
        SectionType::Mindmap
    }

    fn generate(&self, args: &GeneratorArgs) -> String {
        let sdd_id = args.sdd_id.as_deref().unwrap_or("TODO");
        let refs_yaml = if args.sdd_refs.is_empty() {
            String::new()
        } else {
            let refs: Vec<String> = args
                .sdd_refs
                .iter()
                .map(|r| format!("  - $ref: \"{}\"", r))
                .collect();
            format!("\nrefs:\n{}", refs.join("\n"))
        };

        format!(
            "```mermaid\n\
---\n\
id: {sdd_id}{refs_yaml}\n\
---\n\
mindmap\n\
  root((Root Topic))\n\
    Branch A\n\
      Leaf A1\n\
      Leaf A2\n\
    Branch B\n\
      Leaf B1\n\
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
        assert_eq!(MindmapGenerator {}.section_type(), SectionType::Mindmap);
    }

    #[test]
    fn test_generate_contains_mermaid_plus() {
        let args = GeneratorArgs::new(SectionType::Mindmap).with_sdd_id("my-map");
        let output = MindmapGenerator {}.generate(&args);
        assert!(output.contains("```mermaid"));
        assert!(output.contains("id: my-map"));
        assert!(output.contains("mindmap"));
        assert!(output.contains("root"));
    }

    #[test]
    fn test_generate_with_refs() {
        let args = GeneratorArgs::new(SectionType::Mindmap)
            .with_sdd_id("my-map")
            .with_sdd_refs(vec!["scope-spec".to_string()]);
        let output = MindmapGenerator {}.generate(&args);
        assert!(output.contains("$ref: \"scope-spec\""));
    }

    #[test]
    fn test_with_annotation() {
        let args = GeneratorArgs::new(SectionType::Mindmap);
        let output = MindmapGenerator {}.generate_with_annotation(&args);
        assert!(output.starts_with("<!-- type: mindmap lang: mermaid -->"));
    }
}
````

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/generators/mindmap.rs
    action: modify
    section: source
    impl_mode: codegen
    replaces:
      - "<handwrite-gap:standardize:fold-shadow>"
    description: "Source template owns the mindmap section generator behavior and tests."
```
