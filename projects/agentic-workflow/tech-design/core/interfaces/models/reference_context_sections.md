---
id: projects-sdd-src-models-reference-context-sections-rs
fill_sections: [overview, source, changes]
capability_refs:
  - id: aw-core-client-model-workitem-first-artifact-lifecycle
    role: primary
    gap: core-concept-model-and-invariants
    claim: core-concept-model-and-invariants
    coverage: full
    rationale: "Core model/parser TDs define AW Core domain nouns, invariants, and artifact structure."
---

# Standardized projects/agentic-workflow/src/models/reference_context_sections.rs

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/agentic-workflow/src/models/reference_context_sections.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `REFERENCE_CONTEXT_SECTIONS` | projects/agentic-workflow/src/models/reference_context_sections.rs | constant | pub | 17 |  |
| `is_valid_section` | projects/agentic-workflow/src/models/reference_context_sections.rs | function | pub | 54 | is_valid_section(name: &str) -> bool |
| `section_to_heading` | projects/agentic-workflow/src/models/reference_context_sections.rs | function | pub | 27 | section_to_heading(section: &str) -> String |
## Source
<!-- type: source lang: rust -->
<!-- source-from-target: strip-handwrite -->

<!-- source-snapshot: path=projects/agentic-workflow/src/models/reference_context_sections.rs -->
```rust
//! Shared reference context section definitions.
//!
//! Defines the canonical section names used by both SDD change reference context
//! (`create_reference_context.rs`) and issue Reference Context
//! (`fill_issue_reference_context.rs`). Having a single source of truth ensures
//! cross-references between issues and changes resolve correctly.

// @spec projects/agentic-workflow/tech-design/core/logic/reference-context.md#R3

/// Default ordered list of reference context sections.
///
/// Both SDD change `reference_context.md` and issue `## Reference Context`
/// use this same list. Section names are kebab-case identifiers used in
/// frontmatter (`fill_sections` / `filled_sections`) and as heading anchors.
pub const REFERENCE_CONTEXT_SECTIONS: &[&str] = &[
    "source_refs",
    "related_specs",
    "reproductions",
    "related_issues",
    "first_fix",
];

/// Map a section name to its markdown heading (## level).
/// @spec projects/agentic-workflow/tech-design/core/interfaces/models/reference_context_sections.md#source
pub fn section_to_heading(section: &str) -> String {
    match section {
        "source_refs" => "## Source References".to_string(),
        "related_specs" => "## Related Specs".to_string(),
        "reproductions" => "## Reproductions".to_string(),
        "related_issues" => "## Related Issues".to_string(),
        "first_fix" => "## First Fix Analysis".to_string(),
        other => format!("## {}", title_case(other)),
    }
}

/// Convert a snake_case identifier to Title Case.
fn title_case(s: &str) -> String {
    s.split('_')
        .map(|w| {
            let mut c = w.chars();
            match c.next() {
                None => String::new(),
                Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

/// Check whether a section name is a valid reference context section.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/models/reference_context_sections.md#source
pub fn is_valid_section(name: &str) -> bool {
    REFERENCE_CONTEXT_SECTIONS.contains(&name)
}

#[cfg(test)]
mod tests {
    use super::*;

    // REQ: R3
    #[test]
    fn test_shared_section_enum_used_by_both_tools() {
        // T8: verify the shared list is non-empty and contains expected sections
        assert!(!REFERENCE_CONTEXT_SECTIONS.is_empty());
        assert!(REFERENCE_CONTEXT_SECTIONS.contains(&"source_refs"));
        assert!(REFERENCE_CONTEXT_SECTIONS.contains(&"related_specs"));
        assert!(REFERENCE_CONTEXT_SECTIONS.contains(&"reproductions"));
        assert!(REFERENCE_CONTEXT_SECTIONS.contains(&"related_issues"));
        assert!(REFERENCE_CONTEXT_SECTIONS.contains(&"first_fix"));
    }

    #[test]
    fn test_section_to_heading() {
        assert_eq!(section_to_heading("source_refs"), "## Source References");
        assert_eq!(section_to_heading("related_specs"), "## Related Specs");
        assert_eq!(section_to_heading("first_fix"), "## First Fix Analysis");
    }

    #[test]
    fn test_is_valid_section() {
        assert!(is_valid_section("source_refs"));
        assert!(is_valid_section("first_fix"));
        assert!(!is_valid_section("bogus_section"));
    }
}
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/models/reference_context_sections.rs
    action: modify
    section: source
    impl_mode: codegen
    description: |
      Source template owns the complete shared reference context sections module.
```
