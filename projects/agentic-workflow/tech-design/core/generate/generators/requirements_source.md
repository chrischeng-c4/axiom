---
id: sdd-generate-generators-requirements-source
fill_sections: [overview, source, changes]
capability_refs:
  - id: td-cb-lifecycle-automation
    role: primary
    gap: cb-lifecycle-dispatch
    claim: cb-lifecycle-dispatch
    coverage: full
    rationale: "Generator primitives are part of TD/CB lifecycle automation because they produce reviewable code artifacts from TD sections."
---

# RequirementsGenerator Source

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/agentic-workflow/src/generators/requirements.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `RequirementsGenerator` | projects/agentic-workflow/src/generators/requirements.rs | struct | pub | 97 |  |
## Source
<!-- type: source lang: rust -->
<!-- source-from-target: handwrite-gap standardize:fold-shadow -->

<!-- source-snapshot: path=projects/agentic-workflow/src/generators/requirements.rs -->
```rust
/// @spec projects/agentic-workflow/tech-design/core/generate/generators/requirements_source.md#source
impl Generator for RequirementsGenerator {
    fn section_type(&self) -> SectionType {
        SectionType::Requirements
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
### R1: [Requirement Title]\n\
[Description of what must be true]\n\
\n\
**Priority**: high\n\
\n\
### R2: [Requirement Title]\n\
[Description of what must be true]\n\
\n\
**Priority**: medium",
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
        assert_eq!(
            RequirementsGenerator {}.section_type(),
            SectionType::Requirements
        );
    }

    #[test]
    fn test_generate_contains_structure() {
        let args = GeneratorArgs::new(SectionType::Requirements);
        let output = RequirementsGenerator {}.generate(&args);
        assert!(output.contains("### R1:"));
        assert!(output.contains("### R2:"));
        assert!(output.contains("**Priority**:"));
    }

    #[test]
    fn test_generate_with_sdd_id() {
        let args = GeneratorArgs::new(SectionType::Requirements).with_sdd_id("my-feature");
        let output = RequirementsGenerator {}.generate(&args);
        assert!(output.contains("sdd-id: my-feature"));
    }

    #[test]
    fn test_generate_with_refs() {
        let args = GeneratorArgs::new(SectionType::Requirements)
            .with_sdd_refs(vec!["api-spec".to_string()]);
        let output = RequirementsGenerator {}.generate(&args);
        assert!(output.contains("sdd-refs: api-spec"));
    }

    #[test]
    fn test_with_annotation() {
        let args = GeneratorArgs::new(SectionType::Requirements);
        let output = RequirementsGenerator {}.generate_with_annotation(&args);
        assert!(output.starts_with("<!-- type: requirements lang: mermaid -->"));
    }
}
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/generators/requirements.rs
    action: modify
    section: source
    impl_mode: codegen
    replaces:
      - "<handwrite-gap:standardize:fold-shadow>"
    description: "Source template owns the requirements section generator behavior and tests."
```
