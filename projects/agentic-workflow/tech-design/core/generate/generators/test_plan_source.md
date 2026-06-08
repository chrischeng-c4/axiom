---
id: sdd-generate-generators-test-plan-source
fill_sections: [overview, source, changes]
capability_refs:
  - id: td-cb-lifecycle-automation
    role: primary
    gap: cb-lifecycle-dispatch
    claim: cb-lifecycle-dispatch
    coverage: full
    rationale: "Generator primitives are part of TD/CB lifecycle automation because they produce reviewable code artifacts from TD sections."
---

# TestPlanGenerator Source

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/agentic-workflow/src/generators/test_plan.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `TestPlanGenerator` | projects/agentic-workflow/src/generators/test_plan.rs | struct | pub | 107 |  |
## Source
<!-- type: source lang: rust -->
<!-- source-from-target: handwrite-gap standardize:fold-shadow -->

<!-- source-snapshot: path=projects/agentic-workflow/src/generators/test_plan.rs -->
````rust
/// @spec projects/agentic-workflow/tech-design/core/generate/generators/test_plan_source.md#source
impl Generator for TestPlanGenerator {
    fn section_type(&self) -> SectionType {
        SectionType::TestPlan
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
requirementDiagram\n\
\n\
    requirement R1 {{\n\
        id: 1\n\
        text: [Requirement description]\n\
        risk: medium\n\
        verifymethod: test\n\
    }}\n\
\n\
    functionalRequirement R2 {{\n\
        id: 2\n\
        text: [Requirement description]\n\
        risk: low\n\
        verifymethod: test\n\
    }}\n\
\n\
    element TestCase1 {{\n\
        type: test\n\
        docref: [test file path]\n\
    }}\n\
\n\
    TestCase1 - verifies -> R1\n\
    TestCase1 - verifies -> R2\n\
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
        assert_eq!(TestPlanGenerator {}.section_type(), SectionType::TestPlan);
    }

    #[test]
    fn test_generate_contains_mermaid_plus() {
        let args = GeneratorArgs::new(SectionType::TestPlan).with_sdd_id("my-plan");
        let output = TestPlanGenerator {}.generate(&args);
        assert!(output.contains("```mermaid"));
        assert!(output.contains("id: my-plan"));
        assert!(output.contains("requirementDiagram"));
        assert!(output.contains("verifies"));
    }

    #[test]
    fn test_generate_with_refs() {
        let args = GeneratorArgs::new(SectionType::TestPlan)
            .with_sdd_id("my-plan")
            .with_sdd_refs(vec!["req-spec".to_string()]);
        let output = TestPlanGenerator {}.generate(&args);
        assert!(output.contains("$ref: \"req-spec\""));
    }

    #[test]
    fn test_with_annotation() {
        let args = GeneratorArgs::new(SectionType::TestPlan);
        let output = TestPlanGenerator {}.generate_with_annotation(&args);
        assert!(output.starts_with("<!-- type: test-plan lang: mermaid -->"));
    }
}
````

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/generators/test_plan.rs
    action: modify
    section: source
    impl_mode: codegen
    replaces:
      - "<handwrite-gap:standardize:fold-shadow>"
    description: "Source template owns the test plan section generator behavior and tests."
```
