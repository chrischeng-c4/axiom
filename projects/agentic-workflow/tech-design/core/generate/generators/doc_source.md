---
id: sdd-generate-generators-doc-source
fill_sections: [overview, source, changes]
capability_refs:
  - id: td-cb-lifecycle-automation
    role: primary
    gap: cb-lifecycle-dispatch
    claim: cb-lifecycle-dispatch
    coverage: full
    rationale: "Generator primitives are part of TD/CB lifecycle automation because they produce reviewable code artifacts from TD sections."
---

# DocGenerator Source

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/agentic-workflow/src/generators/doc.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `DocGenerator` | projects/agentic-workflow/src/generators/doc.rs | struct | pub | 84 |  |
## Source
<!-- type: source lang: rust -->
<!-- source-from-target: handwrite-gap standardize:fold-shadow -->

<!-- source-snapshot: path=projects/agentic-workflow/src/generators/doc.rs -->
```rust
/// @spec projects/agentic-workflow/tech-design/core/generate/generators/doc_source.md#source
impl Generator for DocGenerator {
    fn section_type(&self) -> SectionType {
        SectionType::Doc
    }

    fn generate(&self, args: &GeneratorArgs) -> String {
        let context_hint = match &args.sdd_id {
            Some(id) => format!("<!-- sdd-id: {} -->\n\n", id),
            None => String::new(),
        };

        let refs_hint = if args.sdd_refs.is_empty() {
            String::new()
        } else {
            format!("<!-- sdd-refs: {} -->\n\n", args.sdd_refs.join(", "))
        };

        format!(
            "{}{}\
### What Changed\n\
[Describe the change from a user perspective]\n\
\n\
### How to Use\n\
[Usage instructions with examples]\n\
\n\
### Examples\n\
[Concrete usage examples]",
            context_hint, refs_hint
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
        assert_eq!(DocGenerator {}.section_type(), SectionType::Doc);
    }

    #[test]
    fn test_generate_contains_structure() {
        let args = GeneratorArgs::new(SectionType::Doc);
        let output = DocGenerator {}.generate(&args);
        assert!(output.contains("### What Changed"));
        assert!(output.contains("### How to Use"));
        assert!(output.contains("### Examples"));
    }

    #[test]
    fn test_generate_with_sdd_id() {
        let args = GeneratorArgs::new(SectionType::Doc).with_sdd_id("my-feature");
        let output = DocGenerator {}.generate(&args);
        assert!(output.contains("sdd-id: my-feature"));
    }

    #[test]
    fn test_with_annotation() {
        let args = GeneratorArgs::new(SectionType::Doc);
        let output = DocGenerator {}.generate_with_annotation(&args);
        assert!(output.starts_with("<!-- type: doc lang: markdown -->"));
    }
}
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/generators/doc.rs
    action: modify
    section: source
    impl_mode: codegen
    replaces:
      - "<handwrite-gap:standardize:fold-shadow>"
    description: "Source template owns the doc section generator behavior and tests."
```
