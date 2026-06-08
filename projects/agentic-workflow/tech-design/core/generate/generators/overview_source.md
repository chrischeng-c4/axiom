---
id: sdd-generate-generators-overview-source
fill_sections: [overview, source, changes]
capability_refs:
  - id: td-cb-lifecycle-automation
    role: primary
    gap: cb-lifecycle-dispatch
    claim: cb-lifecycle-dispatch
    coverage: full
    rationale: "Generator primitives are part of TD/CB lifecycle automation because they produce reviewable code artifacts from TD sections."
---

# OverviewGenerator Source

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/agentic-workflow/src/generators/overview.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `OverviewGenerator` | projects/agentic-workflow/src/generators/overview.rs | struct | pub | 86 |  |
## Source
<!-- type: source lang: rust -->
<!-- source-from-target: handwrite-gap standardize:fold-shadow -->

<!-- source-snapshot: path=projects/agentic-workflow/src/generators/overview.rs -->
```rust
/// @spec projects/agentic-workflow/tech-design/core/generate/generators/overview_source.md#source
impl Generator for OverviewGenerator {
    fn section_type(&self) -> SectionType {
        SectionType::Overview
    }

    fn generate(&self, args: &GeneratorArgs) -> String {
        let context_hint = match &args.sdd_id {
            Some(id) => format!("\nChange: `{}`\n", id),
            None => String::new(),
        };

        let refs_hint = if args.sdd_refs.is_empty() {
            String::new()
        } else {
            format!("\nRelated: {}\n", args.sdd_refs.join(", "))
        };

        format!(
            "{}{}\
[Describe what this spec covers and why it exists. \
Include the scope, motivation, and high-level approach. \
Minimum 50 characters.]",
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
        assert_eq!(OverviewGenerator {}.section_type(), SectionType::Overview);
    }

    #[test]
    fn test_generate_no_args() {
        let args = GeneratorArgs::new(SectionType::Overview);
        let output = OverviewGenerator {}.generate(&args);
        assert!(output.contains("[Describe what this spec covers"));
    }

    #[test]
    fn test_generate_with_sdd_id() {
        let args = GeneratorArgs::new(SectionType::Overview).with_sdd_id("my-feature");
        let output = OverviewGenerator {}.generate(&args);
        assert!(output.contains("my-feature"));
    }

    #[test]
    fn test_generate_with_refs() {
        let args = GeneratorArgs::new(SectionType::Overview)
            .with_sdd_refs(vec!["api-spec".to_string(), "data-model".to_string()]);
        let output = OverviewGenerator {}.generate(&args);
        assert!(output.contains("api-spec"));
        assert!(output.contains("data-model"));
    }

    #[test]
    fn test_with_annotation() {
        let args = GeneratorArgs::new(SectionType::Overview);
        let output = OverviewGenerator {}.generate_with_annotation(&args);
        assert!(output.starts_with("<!-- type: overview lang: markdown -->"));
        assert!(output.contains("[Describe what this spec covers"));
    }
}
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/generators/overview.rs
    action: modify
    section: source
    impl_mode: codegen
    replaces:
      - "<handwrite-gap:standardize:fold-shadow>"
    description: "Source template owns the overview section generator behavior and tests."
```
