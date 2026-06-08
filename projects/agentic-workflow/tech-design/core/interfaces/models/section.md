---
id: sdd-models-section
fill_sections: [overview, schema, changes]
capability_refs:
  - id: aw-core-client-model-workitem-first-artifact-lifecycle
    role: primary
    gap: core-concept-model-and-invariants
    claim: core-concept-model-and-invariants
    coverage: full
    rationale: "Core model/parser TDs define AW Core domain nouns, invariants, and artifact structure."
---

# Section Metadata Model

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/agentic-workflow/src/models/section.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `RawSectionAnnotation` | projects/agentic-workflow/src/models/section.rs | struct | pub | 68 |  |
| `SectionMeta` | projects/agentic-workflow/src/models/section.rs | struct | pub | 24 |  |
| `effective_lang` | projects/agentic-workflow/src/models/section.rs | function | pub | 59 | effective_lang(&self) -> &str |
| `new` | projects/agentic-workflow/src/models/section.rs | function | pub | 36 | new(section_type: SectionType, lang: Option<String>) -> Self |
| `parse_all_section_annotations` | projects/agentic-workflow/src/models/section.rs | function | pub | 120 | parse_all_section_annotations(content: &str) -> Vec<(usize, SectionMeta)> |
| `parse_section_annotation` | projects/agentic-workflow/src/models/section.rs | function | pub | 90 | parse_section_annotation(content: &str) -> Option<SectionMeta> |
| `parse_section_annotation_parts` | projects/agentic-workflow/src/models/section.rs | function | pub | 108 | parse_section_annotation_parts(content: &str) -> Option<RawSectionAnnotation> |
| `with_attributes` | projects/agentic-workflow/src/models/section.rs | function | pub | 45 | with_attributes(         section_type: SectionType,         lang: Option<String>,         attributes: BTreeMap<String, String>,     ) -> Self |
## Schema
<!-- type: schema lang: yaml -->

```yaml
definitions:
  SectionMeta:
    type: object
    required: [section_type, attributes]
    description: >
      Metadata extracted from a section annotation comment of the form
      `<!-- type: <section_type> lang: <language> -->`.
    properties:
      section_type:
        type: string
        x-rust-type: "SectionType"
        description: "Section kind (one of the defined SectionType variants)."
      lang:
        type: string
        description: "Content language / format (e.g. 'markdown', 'yaml', 'mermaid', 'json'); None means use the SectionType's default_lang()."
      attributes:
        type: object
        x-rust-type: "BTreeMap<String, String>"
        additionalProperties:
          type: string
        default: {}
        description: "Optional attr-style metadata excluding core type/lang keys."
    x-rust-struct:
      derive: [Debug, Clone, PartialEq, Eq]
```

## Source
<!-- type: source lang: rust -->
<!-- source-from-target: strip-managed-markers -->

<!-- source-snapshot: path=projects/agentic-workflow/src/models/section.rs -->
~~~rust
// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/interfaces/models/section.md#source
// CODEGEN-BEGIN
//! Section annotation model and parser.
//!
//! Sections within change spec files are annotated with a type comment:
//!
//! ```markdown
//! ## Overview
//! <!-- type: overview lang: markdown -->
//!
//! Content here...
//! ```
//!
//! This module parses that annotation and exposes `SectionMeta`.

/// @spec projects/agentic-workflow/tech-design/core/interfaces/models/section.md#source
use crate::models::spec_rules::SectionType;
use std::collections::BTreeMap;
use std::str::FromStr;

/// Metadata extracted from a section annotation comment of the form `<!-- type: <section_type> lang: <language> -->`.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/models/section.md#schema
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SectionMeta {
    /// Section kind (one of the defined SectionType variants).
    pub section_type: SectionType,
    /// Content language / format (e.g. 'markdown', 'yaml', 'mermaid', 'json'); None means use the SectionType's default_lang().
    pub lang: Option<String>,
    /// Optional attr-style metadata excluding core type/lang keys.
    pub attributes: BTreeMap<String, String>,
}

/// @spec projects/agentic-workflow/tech-design/core/interfaces/models/section.md#source
impl SectionMeta {
    /// Create a new `SectionMeta`.
    pub fn new(section_type: SectionType, lang: Option<String>) -> Self {
        Self {
            section_type,
            lang,
            attributes: BTreeMap::new(),
        }
    }

    /// Create a new `SectionMeta` with optional attr-style attributes.
    pub fn with_attributes(
        section_type: SectionType,
        lang: Option<String>,
        attributes: BTreeMap<String, String>,
    ) -> Self {
        Self {
            section_type,
            lang,
            attributes,
        }
    }

    /// Return the effective language: explicit `lang` if set, otherwise
    /// the default for the section type.
    pub fn effective_lang(&self) -> &str {
        self.lang
            .as_deref()
            .unwrap_or_else(|| self.section_type.default_lang())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// @spec projects/agentic-workflow/tech-design/core/interfaces/models/section.md#source
pub struct RawSectionAnnotation {
    pub section_type: String,
    pub lang: Option<String>,
    pub attributes: BTreeMap<String, String>,
}

/// Parse the first `<!-- type: xxx lang: yyy -->` annotation found in `content`.
///
/// Returns `None` if no valid annotation is found.
///
/// # Examples
///
/// ```
/// use agentic_workflow::models::section::parse_section_annotation;
/// use agentic_workflow::models::spec_rules::SectionType;
///
/// let content = "<!-- type: overview lang: markdown -->\n\nSome prose.";
/// let meta = parse_section_annotation(content).unwrap();
/// assert_eq!(meta.section_type, SectionType::Overview);
/// assert_eq!(meta.lang.as_deref(), Some("markdown"));
/// ```
/// @spec projects/agentic-workflow/tech-design/core/interfaces/models/section.md#source
pub fn parse_section_annotation(content: &str) -> Option<SectionMeta> {
    content.lines().find_map(|line| {
        let raw = parse_section_annotation_parts(line)?;
        let section_type = SectionType::from_str(&raw.section_type).ok()?;
        Some(SectionMeta::with_attributes(
            section_type,
            raw.lang,
            raw.attributes,
        ))
    })
}

/// Parse the first supported section annotation comment into raw string parts.
///
/// Supported forms:
/// - `<!-- type: wireframe lang: yaml -->`
/// - `<!-- score-section type="wireframe" lang="yaml" workspace="studio" -->`
/// @spec projects/agentic-workflow/tech-design/core/interfaces/models/section.md#source
pub fn parse_section_annotation_parts(content: &str) -> Option<RawSectionAnnotation> {
    content.lines().find_map(parse_section_annotation_line)
}

/// Parse ALL section annotations in a markdown document.
///
/// Returns a list of `(heading_line_index, SectionMeta)` pairs, where
/// `heading_line_index` is the 0-based line number of the heading that
/// immediately precedes the annotation.
///
/// Only H2 (`##`) and H3 (`###`) headings are considered.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/models/section.md#source
pub fn parse_all_section_annotations(content: &str) -> Vec<(usize, SectionMeta)> {
    let lines: Vec<&str> = content.lines().collect();
    let mut results = Vec::new();
    let mut last_heading_idx: Option<usize> = None;
    let mut in_fence = false;

    for (i, line) in lines.iter().enumerate() {
        if is_markdown_fence(line) {
            in_fence = !in_fence;
            continue;
        }
        if in_fence {
            continue;
        }
        let trimmed = line.trim();
        // Track H2/H3 headings
        if is_markdown_heading(line) {
            last_heading_idx = Some(i);
            continue;
        }
        // Look for annotation comment
        if let Some(raw) = parse_section_annotation_line(trimmed) {
            if let Ok(section_type) = SectionType::from_str(&raw.section_type) {
                let meta = SectionMeta::with_attributes(section_type, raw.lang, raw.attributes);
                let heading_idx = last_heading_idx.unwrap_or(i);
                results.push((heading_idx, meta));
            }
        }
    }

    results
}

fn is_markdown_heading(line: &str) -> bool {
    let leading_spaces = line.chars().take_while(|c| *c == ' ').count();
    if leading_spaces > 3 {
        return false;
    }
    let trimmed = line.trim();
    trimmed.starts_with("## ") || trimmed.starts_with("### ")
}

fn is_markdown_fence(line: &str) -> bool {
    let leading_spaces = line.chars().take_while(|c| *c == ' ').count();
    leading_spaces <= 3
        && line
            .get(leading_spaces..)
            .is_some_and(|trimmed| trimmed.starts_with("```"))
}

fn parse_section_annotation_line(line: &str) -> Option<RawSectionAnnotation> {
    let trimmed = line.trim();
    if !trimmed.starts_with("<!--") || !trimmed.ends_with("-->") {
        return None;
    }
    let inner = trimmed.strip_prefix("<!--")?.strip_suffix("-->")?.trim();
    if inner.starts_with("score-section") {
        parse_attr_style_annotation(inner)
    } else {
        parse_legacy_annotation(inner)
    }
}

fn parse_legacy_annotation(inner: &str) -> Option<RawSectionAnnotation> {
    let parts: Vec<&str> = inner.split_whitespace().collect();
    let mut section_type = None;
    let mut lang = None;
    let mut idx = 0;
    while idx < parts.len() {
        match parts[idx] {
            "type:" if idx + 1 < parts.len() => {
                section_type = Some(parts[idx + 1].to_string());
                idx += 2;
            }
            "lang:" if idx + 1 < parts.len() => {
                lang = Some(parts[idx + 1].to_string());
                idx += 2;
            }
            token if token.starts_with("type:") && token.len() > "type:".len() => {
                section_type = Some(token["type:".len()..].to_string());
                idx += 1;
            }
            token if token.starts_with("lang:") && token.len() > "lang:".len() => {
                lang = Some(token["lang:".len()..].to_string());
                idx += 1;
            }
            _ => idx += 1,
        }
    }
    Some(RawSectionAnnotation {
        section_type: section_type?,
        lang,
        attributes: BTreeMap::new(),
    })
}

fn parse_attr_style_annotation(inner: &str) -> Option<RawSectionAnnotation> {
    let attr_src = inner.strip_prefix("score-section")?.trim();
    let mut attrs = BTreeMap::new();
    let re = regex::Regex::new(r#"([A-Za-z_][A-Za-z0-9_-]*)\s*=\s*"([^"]*)""#)
        .expect("attr-style annotation regex is valid");
    for cap in re.captures_iter(attr_src) {
        attrs.insert(cap[1].to_string(), cap[2].to_string());
    }
    let section_type = attrs.remove("type")?;
    let lang = attrs.remove("lang");
    Some(RawSectionAnnotation {
        section_type,
        lang,
        attributes: attrs,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::spec_rules::SectionType;

    #[test]
    fn test_parse_overview_with_lang() {
        let content = "<!-- type: overview lang: markdown -->";
        let meta = parse_section_annotation(content).unwrap();
        assert_eq!(meta.section_type, SectionType::Overview);
        assert_eq!(meta.lang.as_deref(), Some("markdown"));
    }

    #[test]
    fn test_parse_changes_with_lang() {
        let content = "<!-- type: changes lang: yaml -->";
        let meta = parse_section_annotation(content).unwrap();
        assert_eq!(meta.section_type, SectionType::Changes);
        assert_eq!(meta.lang.as_deref(), Some("yaml"));
    }

    #[test]
    fn test_parse_without_lang() {
        let content = "<!-- type: requirements -->";
        let meta = parse_section_annotation(content).unwrap();
        assert_eq!(meta.section_type, SectionType::Requirements);
        assert_eq!(meta.lang, None);
        assert!(meta.attributes.is_empty());
    }

    #[test]
    fn test_parse_attr_style_preserves_optional_attrs() {
        let content = r#"<!-- score-section type="wireframe" lang="yaml" workspace="cue-artifact-studio" surface="studio" role="owner" -->"#;
        let meta = parse_section_annotation(content).unwrap();
        assert_eq!(meta.section_type, SectionType::Wireframe);
        assert_eq!(meta.lang.as_deref(), Some("yaml"));
        assert_eq!(
            meta.attributes.get("workspace").map(String::as_str),
            Some("cue-artifact-studio")
        );
        assert_eq!(
            meta.attributes.get("surface").map(String::as_str),
            Some("studio")
        );
        assert_eq!(
            meta.attributes.get("role").map(String::as_str),
            Some("owner")
        );
    }

    #[test]
    fn test_effective_lang_explicit() {
        let meta = SectionMeta::new(SectionType::Overview, Some("html".to_string()));
        assert_eq!(meta.effective_lang(), "html");
    }

    #[test]
    fn test_effective_lang_default() {
        let meta = SectionMeta::new(SectionType::Changes, None);
        assert_eq!(meta.effective_lang(), "yaml");
    }

    #[test]
    fn test_effective_lang_mermaid_default() {
        let meta = SectionMeta::new(SectionType::Interaction, None);
        assert_eq!(meta.effective_lang(), "mermaid");
    }

    #[test]
    fn test_parse_none_for_invalid() {
        let content = "<!-- type: invalid-type lang: yaml -->";
        assert!(parse_section_annotation(content).is_none());
    }

    #[test]
    fn test_parse_none_for_no_annotation() {
        let content = "Some plain markdown content with no annotation.";
        assert!(parse_section_annotation(content).is_none());
    }

    #[test]
    fn test_parse_in_multiline_content() {
        let content = concat!(
            "## Overview\n",
            "<!-- type: overview lang: markdown -->\n\n",
            "This is the overview text.\n\n",
            "## Changes\n",
            "<!-- type: changes lang: yaml -->\n",
        );
        let meta = parse_section_annotation(content).unwrap();
        assert_eq!(meta.section_type, SectionType::Overview);
    }

    #[test]
    fn test_parse_all_annotations() {
        let content = concat!(
            "## Overview\n",
            "<!-- type: overview lang: markdown -->\n\n",
            "Content.\n\n",
            "## Changes\n",
            "<!-- type: changes lang: yaml -->\n\n",
            "files:\n",
            "  - path: foo.rs\n",
        );
        let annotations = parse_all_section_annotations(content);
        assert_eq!(annotations.len(), 2);
        assert_eq!(annotations[0].1.section_type, SectionType::Overview);
        assert_eq!(annotations[1].1.section_type, SectionType::Changes);
    }

    #[test]
    fn test_parse_all_annotations_attr_style() {
        let content = concat!(
            "## Wireframe\n",
            "<!-- score-section type=\"wireframe\" lang=\"yaml\" workspace=\"studio\" -->\n\n",
            "```yaml\n",
            "tasks: []\n",
            "```\n",
        );
        let annotations = parse_all_section_annotations(content);
        assert_eq!(annotations.len(), 1);
        assert_eq!(annotations[0].1.section_type, SectionType::Wireframe);
        assert_eq!(
            annotations[0]
                .1
                .attributes
                .get("workspace")
                .map(String::as_str),
            Some("studio")
        );
    }

    #[test]
    fn test_parse_all_annotations_ignores_indented_fixture_markdown_in_fence() {
        let content = concat!(
            "## Tests\n",
            "<!-- type: tests lang: yaml -->\n\n",
            "```yaml\n",
            "tests:\n",
            "  - name: embedded_fixture\n",
            "    body: |\n",
            "      ## Not A Section\n",
            "      <!-- type: logic lang: mermaid -->\n",
            "      ```mermaid\n",
            "      flowchart TD\n",
            "          A --> B\n",
            "      ```\n",
            "```\n\n",
            "## Changes\n",
            "<!-- type: changes lang: yaml -->\n",
        );
        let annotations = parse_all_section_annotations(content);
        assert_eq!(annotations.len(), 2);
        assert_eq!(annotations[0].1.section_type, SectionType::Tests);
        assert_eq!(annotations[1].1.section_type, SectionType::Changes);
    }

    #[test]
    fn test_parse_flexible_whitespace() {
        let content = "<!--  type:  test_plan  lang:  markdown  -->";
        let meta = parse_section_annotation(content).unwrap();
        assert_eq!(meta.section_type, SectionType::TestPlan);
    }

    #[test]
    fn test_all_section_types_parseable() {
        for st in SectionType::all_in_fill_order() {
            let annotation = format!("<!-- type: {} -->", st.as_str());
            let meta = parse_section_annotation(&annotation)
                .unwrap_or_else(|| panic!("Failed to parse annotation for {:?}", st));
            assert_eq!(meta.section_type, st);
        }
    }
}

// CODEGEN-END
~~~

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/models/section.rs
    action: modify
    section: source
    impl_mode: codegen
    description: |
      Source template owns the complete section annotation module.
  - action: annotate
    section: schema
    impl_mode: hand-written
    description: "Traceability metadata edge for the schema section."

```

# Reviews

## Review 1
<!-- type: doc lang: markdown -->
**Verdict:** approved

- [overview] Accurate: correctly identifies `SectionMeta` fields, derives, no-serde constraint, and the three codegen exercises (foreign type, non-serde emission, Option auto-wrap). Matches source file exactly.
- [schema] Correct and complete: `x-rust-type: "SectionType"` on `section_type`, `lang` absent from `required` for Option auto-wrap, `x-rust-struct.derive` list matches actual derives. No ambiguity for the generator.
- [changes] Clear codegen/hand-written split: `replaces: [SectionMeta]` scopes codegen to the struct declaration; all hand-written symbols are enumerated. Sufficient for `aw td gen-code` to place CODEGEN-BEGIN/END markers correctly.
