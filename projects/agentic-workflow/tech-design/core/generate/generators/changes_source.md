---
id: sdd-generate-generators-changes-source
fill_sections: [overview, source, changes]
capability_refs:
  - id: td-cb-lifecycle-automation
    role: primary
    gap: cb-lifecycle-dispatch
    claim: cb-lifecycle-dispatch
    coverage: full
    rationale: "Generator primitives are part of TD/CB lifecycle automation because they produce reviewable code artifacts from TD sections."
---

# ChangesGenerator Source

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/agentic-workflow/src/generators/changes.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `ChangesGenerator` | projects/agentic-workflow/src/generators/changes.rs | struct | pub | 110 |  |
## Source
<!-- type: source lang: rust -->
<!-- source-from-target: handwrite-gap standardize:fold-shadow -->

<!-- source-snapshot: path=projects/agentic-workflow/src/generators/changes.rs -->
````rust
/// @spec projects/agentic-workflow/tech-design/core/generate/generators/changes_source.md#source
impl Generator for ChangesGenerator {
    fn section_type(&self) -> SectionType {
        SectionType::Changes
    }

    fn generate(&self, args: &GeneratorArgs) -> String {
        let context_comment = match &args.sdd_id {
            Some(id) => format!("# Change: {}\n", id),
            None => String::new(),
        };

        let refs_comment = if args.sdd_refs.is_empty() {
            String::new()
        } else {
            format!("# Refs: {}\n", args.sdd_refs.join(", "))
        };

        format!(
            "```yaml\n\
{}{}files:\n\
  - path: <file_path>\n\
    action: CREATE  # CREATE | MODIFY | DELETE\n\
    desc: <description>\n\
  - path: <file_path_2>\n\
    action: MODIFY\n\
    desc: <description>\n\
```",
            context_comment, refs_comment
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
        assert_eq!(ChangesGenerator {}.section_type(), SectionType::Changes);
    }

    #[test]
    fn test_generate_contains_yaml_structure() {
        let args = GeneratorArgs::new(SectionType::Changes);
        let output = ChangesGenerator {}.generate(&args);
        assert!(output.contains("```yaml"));
        assert!(output.contains("files:"));
        assert!(output.contains("action: CREATE"));
        assert!(output.contains("path: <file_path>"));
        assert!(output.contains("```"));
    }

    #[test]
    fn test_generate_with_sdd_id() {
        let args = GeneratorArgs::new(SectionType::Changes).with_sdd_id("my-feature");
        let output = ChangesGenerator {}.generate(&args);
        assert!(output.contains("my-feature"));
    }

    #[test]
    fn test_generate_with_refs() {
        let args =
            GeneratorArgs::new(SectionType::Changes).with_sdd_refs(vec!["api-spec".to_string()]);
        let output = ChangesGenerator {}.generate(&args);
        assert!(output.contains("api-spec"));
    }

    #[test]
    fn test_with_annotation() {
        let args = GeneratorArgs::new(SectionType::Changes);
        let output = ChangesGenerator {}.generate_with_annotation(&args);
        assert!(output.starts_with("<!-- type: changes lang: yaml -->"));
        assert!(output.contains("files:"));
    }

    #[test]
    fn test_yaml_has_all_three_actions_documented() {
        let args = GeneratorArgs::new(SectionType::Changes);
        let output = ChangesGenerator {}.generate(&args);
        // The template should document all three action values
        assert!(output.contains("CREATE"));
        assert!(output.contains("MODIFY"));
        assert!(output.contains("DELETE"));
    }
}
````

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/generators/changes.rs
    action: modify
    section: source
    impl_mode: codegen
    replaces:
      - "<handwrite-gap:standardize:fold-shadow>"
    description: "Source template owns the changes section generator behavior and tests."
```
