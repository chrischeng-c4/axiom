---
id: projects-sdd-src-spec-alignment-parser-rs
fill_sections: [overview, source, changes]
capability_refs:
  - id: existing-project-standardization
    role: primary
    gap: traceability-closure-gate
    claim: traceability-closure-gate
    coverage: full
    rationale: "Spec alignment interfaces implement TD/source annotation and coverage checks used by the traceability closure gate."
---

# Standardized projects/agentic-workflow/src/spec_alignment/parser.rs

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/agentic-workflow/src/spec_alignment/parser.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `parse` | projects/agentic-workflow/src/spec_alignment/parser.rs | function | pub | 19 | parse(path: &str, content: &str) -> SpecDocument |
## Source
<!-- type: source lang: rust -->
<!-- source-from-target: strip-handwrite -->

<!-- source-snapshot: path=projects/agentic-workflow/src/spec_alignment/parser.rs -->
````rust
//! SpecDocument parser.
//!
//! Parses spec `.md` files into structured `SpecDocument` representation:
//! - Extracts YAML frontmatter between `---` delimiters
//! - Splits sections by `## Heading` lines
//! - Parses `<!-- type: X lang: Y -->` annotations on the line after headings
//! - Collects fenced code blocks within each section
//! - Attempts JSON parsing for `json` code blocks

use super::models::{CodeBlock, SectionAnnotation, SpecDocument, SpecSection};

/// Parse a spec markdown file into a `SpecDocument`.
///
/// The `path` is stored as-is in the returned document (not resolved).
/// The `content` is the raw file content.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/spec_alignment/parser.md#source
pub fn parse(path: &str, content: &str) -> SpecDocument {
    let lines: Vec<&str> = content.lines().collect();
    let frontmatter = extract_frontmatter(&lines);
    let sections = extract_sections(&lines);

    SpecDocument {
        path: path.to_string(),
        frontmatter,
        sections,
    }
}

/// Extract YAML frontmatter between `---` delimiters.
///
/// Returns a `serde_json::Value` (object) if valid YAML frontmatter is found,
/// or `Value::Null` if absent or unparseable.
fn extract_frontmatter(lines: &[&str]) -> serde_json::Value {
    if lines.is_empty() || lines[0].trim() != "---" {
        return serde_json::Value::Null;
    }

    // Find closing ---
    let mut end = None;
    for (i, line) in lines.iter().enumerate().skip(1) {
        if line.trim() == "---" {
            end = Some(i);
            break;
        }
    }

    let end = match end {
        Some(e) => e,
        None => return serde_json::Value::Null,
    };

    let yaml_content: String = lines[1..end].join("\n");
    match serde_yaml::from_str::<serde_json::Value>(&yaml_content) {
        Ok(v) => v,
        Err(_) => serde_json::Value::Null,
    }
}

/// Extract sections from the document lines.
///
/// A section starts with a `## Heading` line. Everything between two `## Heading`
/// lines (or until EOF) belongs to the first heading's section.
/// Sub-headings (`###`, `####`) do NOT start new top-level sections — they are
/// part of the enclosing `##` section.
fn extract_sections(lines: &[&str]) -> Vec<SpecSection> {
    let mut sections = Vec::new();
    let len = lines.len();
    let (heading_indices, reviews_start) = section_heading_indices(lines);

    for (idx, i) in heading_indices.iter().enumerate() {
        let i = *i;
        if let Some(heading) = parse_heading(lines[i]) {
            let heading_line = i + 1; // 1-based

            // Check if next line is an annotation
            let annotation = if i + 1 < len {
                parse_annotation(lines[i + 1])
            } else {
                None
            };

            // Find the end of this section (next TD ## heading, # Reviews, or EOF)
            let section_start = i + 1;
            let section_end = heading_indices
                .get(idx + 1)
                .copied()
                .or(reviews_start)
                .unwrap_or(len);

            // Collect code blocks within this section
            let code_blocks =
                extract_code_blocks(&lines[section_start..section_end], section_start);

            // Collect body text, skipping the annotation line when present.
            let body_start = if annotation.is_some() {
                section_start + 1
            } else {
                section_start
            };
            let body = if body_start < section_end {
                lines[body_start..section_end].join("\n").trim().to_string()
            } else {
                String::new()
            };

            sections.push(SpecSection {
                heading,
                line: heading_line,
                annotation,
                code_blocks,
                body,
            });
        }
    }

    sections
}

fn section_heading_indices(lines: &[&str]) -> (Vec<usize>, Option<usize>) {
    let mut headings = Vec::new();
    let mut in_fence = false;

    for (i, line) in lines.iter().enumerate() {
        if is_markdown_fence(line) {
            in_fence = !in_fence;
            continue;
        }
        if in_fence {
            continue;
        }
        if line.trim() == "# Reviews" {
            return (headings, Some(i));
        }
        if parse_heading(line).is_some() {
            headings.push(i);
        }
    }

    (headings, None)
}

/// Parse a `## Heading` line, returning the heading text.
///
/// Only matches level-2 headings (`## `). Sub-headings (`### `, etc.) are not
/// matched as they don't start new top-level sections.
fn parse_heading(line: &str) -> Option<String> {
    let leading_spaces = line.chars().take_while(|c| *c == ' ').count();
    if leading_spaces > 3 {
        return None;
    }
    let trimmed = line.trim();
    if trimmed.starts_with("## ") && !trimmed.starts_with("### ") {
        Some(trimmed[3..].trim().to_string())
    } else {
        None
    }
}

/// Parse an annotation comment: `<!-- type: X lang: Y -->`.
///
/// Returns `Some(SectionAnnotation)` if the line matches the expected format.
fn parse_annotation(line: &str) -> Option<SectionAnnotation> {
    let raw = crate::models::section::parse_section_annotation_parts(line)?;
    Some(SectionAnnotation {
        section_type: raw.section_type,
        lang: raw.lang?,
        attributes: raw.attributes,
    })
}

/// Extract fenced code blocks from a slice of lines within a section.
///
/// `offset` is the 0-based index of `section_lines[0]` in the original document,
/// so that line numbers in `CodeBlock` are reported as 1-based global positions.
fn extract_code_blocks(section_lines: &[&str], offset: usize) -> Vec<CodeBlock> {
    let mut blocks = Vec::new();
    let mut i = 0;

    while i < section_lines.len() {
        if let Some(line) = markdown_fence_payload(section_lines[i]) {
            // Opening fence: ```lang
            let lang = line[3..].trim().to_string();
            let fence_line = offset + i + 1; // 1-based
            let mut content_lines = Vec::new();
            i += 1;

            // Collect content until closing fence
            while i < section_lines.len() {
                if markdown_fence_payload(section_lines[i]) == Some("```") {
                    break;
                }
                content_lines.push(section_lines[i]);
                i += 1;
            }

            let content = content_lines.join("\n");

            // Attempt JSON parsing for json blocks
            let parsed_json = if lang == "json" {
                serde_json::from_str::<serde_json::Value>(&content).ok()
            } else {
                None
            };

            if !lang.is_empty() {
                blocks.push(CodeBlock {
                    lang,
                    line: fence_line,
                    content,
                    parsed_json,
                });
            }
        }
        i += 1;
    }

    blocks
}

fn is_markdown_fence(line: &str) -> bool {
    markdown_fence_payload(line).is_some()
}

fn markdown_fence_payload(line: &str) -> Option<&str> {
    let leading_spaces = line.chars().take_while(|c| *c == ' ').count();
    if leading_spaces > 3 {
        return None;
    }
    line.get(leading_spaces..)?.strip_prefix("```").map(|_| {
        // Return from the fence marker so existing lang extraction can keep
        // slicing at index 3.
        &line[leading_spaces..]
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_ignores_h2_headings_under_reviews() {
        let doc = parse(
            "test.md",
            "## Logic\n\
             <!-- type: logic lang: mermaid -->\n\
             body\n\
             \n\
             # Reviews\n\
             \n\
             ## Review 1\n\
             **Verdict:** approved\n",
        );

        assert_eq!(doc.sections.len(), 1);
        assert_eq!(doc.sections[0].heading, "Logic");
        assert!(!doc.sections[0].body.contains("Review 1"));
    }

    #[test]
    fn parse_ignores_h2_inside_fenced_blocks() {
        let doc = parse(
            "test.md",
            "## Logic\n\
             <!-- type: logic lang: mermaid -->\n\
             ```markdown\n\
             ## Not A Section\n\
             ```\n\
             \n\
             ## Changes\n\
             <!-- type: changes lang: yaml -->\n",
        );

        let headings: Vec<&str> = doc.sections.iter().map(|s| s.heading.as_str()).collect();
        assert_eq!(headings, vec!["Logic", "Changes"]);
    }

    #[test]
    fn parse_ignores_indented_fixture_markdown_inside_yaml_fence() {
        let doc = parse(
            "test.md",
            "## Tests\n\
             <!-- type: tests lang: yaml -->\n\
             ```yaml\n\
             tests:\n\
               - name: embedded_fixture\n\
                 body: |\n\
                   ## Not A Section\n\
                   <!-- type: logic lang: mermaid -->\n\
                   ```mermaid\n\
                   flowchart TD\n\
                       A --> B\n\
                   ```\n\
             ```\n\
             \n\
             ## Changes\n\
             <!-- type: changes lang: yaml -->\n",
        );

        let headings: Vec<&str> = doc.sections.iter().map(|s| s.heading.as_str()).collect();
        assert_eq!(headings, vec!["Tests", "Changes"]);
        assert_eq!(doc.sections[0].code_blocks.len(), 1);
        assert_eq!(doc.sections[0].code_blocks[0].lang, "yaml");
    }

    #[test]
    fn parse_attr_style_annotation_preserves_attributes() {
        let doc = parse(
            "test.md",
            "## Wireframe\n\
             <!-- score-section type=\"wireframe\" lang=\"yaml\" workspace=\"studio\" role=\"owner\" -->\n\
             ```yaml\n\
             tasks: []\n\
             ```\n",
        );

        let annotation = doc.sections[0].annotation.as_ref().unwrap();
        assert_eq!(annotation.section_type, "wireframe");
        assert_eq!(annotation.lang, "yaml");
        assert_eq!(
            annotation.attributes.get("workspace").map(String::as_str),
            Some("studio")
        );
        assert_eq!(
            annotation.attributes.get("role").map(String::as_str),
            Some("owner")
        );
    }
}
````

             ## Review 1\n\
             **Verdict:** approved\n",
        );

        assert_eq!(doc.sections.len(), 1);
        assert_eq!(doc.sections[0].heading, "Logic");
        assert!(!doc.sections[0].body.contains("Review 1"));
    }

    #[test]
    fn parse_ignores_h2_inside_fenced_blocks() {
        let doc = parse(
            "test.md",
            "## Logic\n\
             <!-- type: logic lang: mermaid -->\n\
             ```markdown\n\
             ## Not A Section\n\
             ```\n\
             \n\
             ## Changes\n\
             <!-- type: changes lang: yaml -->\n",
        );

        let headings: Vec<&str> = doc.sections.iter().map(|s| s.heading.as_str()).collect();
        assert_eq!(headings, vec!["Logic", "Changes"]);
    }

    #[test]
    fn parse_ignores_indented_fixture_markdown_inside_yaml_fence() {
        let doc = parse(
            "test.md",
            "## Tests\n\
             <!-- type: tests lang: yaml -->\n\
             ```yaml\n\
             tests:\n\
               - name: embedded_fixture\n\
                 body: |\n\
                   ## Not A Section\n\
                   <!-- type: logic lang: mermaid -->\n\
                   ```mermaid\n\
                   flowchart TD\n\
                       A --> B\n\
                   ```\n\
             ```\n\
             \n\
             ## Changes\n\
             <!-- type: changes lang: yaml -->\n",
        );

        let headings: Vec<&str> = doc.sections.iter().map(|s| s.heading.as_str()).collect();
        assert_eq!(headings, vec!["Tests", "Changes"]);
        assert_eq!(doc.sections[0].code_blocks.len(), 1);
        assert_eq!(doc.sections[0].code_blocks[0].lang, "yaml");
    }

    #[test]
    fn parse_attr_style_annotation_preserves_attributes() {
        let doc = parse(
            "test.md",
            "## Wireframe\n\
             <!-- score-section type=\"wireframe\" lang=\"yaml\" workspace=\"studio\" role=\"owner\" -->\n\
             ```yaml\n\
             tasks: []\n\
             ```\n",
        );

        let annotation = doc.sections[0].annotation.as_ref().unwrap();
        assert_eq!(annotation.section_type, "wireframe");
        assert_eq!(annotation.lang, "yaml");
        assert_eq!(
            annotation.attributes.get("workspace").map(String::as_str),
            Some("studio")
        );
        assert_eq!(
            annotation.attributes.get("role").map(String::as_str),
            Some("owner")
        );
    }
}
````

             ## Review 1\n\
             **Verdict:** approved\n",
        );

        assert_eq!(doc.sections.len(), 1);
        assert_eq!(doc.sections[0].heading, "Logic");
        assert!(!doc.sections[0].body.contains("Review 1"));
    }

    #[test]
    fn parse_ignores_h2_inside_fenced_blocks() {
        let doc = parse(
            "test.md",
            "## Logic\n\
             <!-- type: logic lang: mermaid -->\n\
             ```markdown\n\
             ## Not A Section\n\
             ```\n\
             \n\
             ## Changes\n\
             <!-- type: changes lang: yaml -->\n",
        );

        let headings: Vec<&str> = doc.sections.iter().map(|s| s.heading.as_str()).collect();
        assert_eq!(headings, vec!["Logic", "Changes"]);
    }

    #[test]
    fn parse_ignores_indented_fixture_markdown_inside_yaml_fence() {
        let doc = parse(
            "test.md",
            "## Tests\n\
             <!-- type: tests lang: yaml -->\n\
             ```yaml\n\
             tests:\n\
               - name: embedded_fixture\n\
                 body: |\n\
                   ## Not A Section\n\
                   <!-- type: logic lang: mermaid -->\n\
                   ```mermaid\n\
                   flowchart TD\n\
                       A --> B\n\
                   ```\n\
             ```\n\
             \n\
             ## Changes\n\
             <!-- type: changes lang: yaml -->\n",
        );

        let headings: Vec<&str> = doc.sections.iter().map(|s| s.heading.as_str()).collect();
        assert_eq!(headings, vec!["Tests", "Changes"]);
        assert_eq!(doc.sections[0].code_blocks.len(), 1);
        assert_eq!(doc.sections[0].code_blocks[0].lang, "yaml");
    }

    #[test]
    fn parse_attr_style_annotation_preserves_attributes() {
        let doc = parse(
            "test.md",
            "## Wireframe\n\
             <!-- score-section type=\"wireframe\" lang=\"yaml\" workspace=\"studio\" role=\"owner\" -->\n\
             ```yaml\n\
             tasks: []\n\
             ```\n",
        );

        let annotation = doc.sections[0].annotation.as_ref().unwrap();
        assert_eq!(annotation.section_type, "wireframe");
        assert_eq!(annotation.lang, "yaml");
        assert_eq!(
            annotation.attributes.get("workspace").map(String::as_str),
            Some("studio")
        );
        assert_eq!(
            annotation.attributes.get("role").map(String::as_str),
            Some("owner")
        );
    }
}
````

             ## Review 1\n\
             **Verdict:** approved\n",
        );

        assert_eq!(doc.sections.len(), 1);
        assert_eq!(doc.sections[0].heading, "Logic");
        assert!(!doc.sections[0].body.contains("Review 1"));
    }

    #[test]
    fn parse_ignores_h2_inside_fenced_blocks() {
        let doc = parse(
            "test.md",
            "## Logic\n\
             <!-- type: logic lang: mermaid -->\n\
             ```markdown\n\
             ## Not A Section\n\
             ```\n\
             \n\
             ## Changes\n\
             <!-- type: changes lang: yaml -->\n",
        );

        let headings: Vec<&str> = doc.sections.iter().map(|s| s.heading.as_str()).collect();
        assert_eq!(headings, vec!["Logic", "Changes"]);
    }

    #[test]
    fn parse_ignores_indented_fixture_markdown_inside_yaml_fence() {
        let doc = parse(
            "test.md",
            "## Tests\n\
             <!-- type: tests lang: yaml -->\n\
             ```yaml\n\
             tests:\n\
               - name: embedded_fixture\n\
                 body: |\n\
                   ## Not A Section\n\
                   <!-- type: logic lang: mermaid -->\n\
                   ```mermaid\n\
                   flowchart TD\n\
                       A --> B\n\
                   ```\n\
             ```\n\
             \n\
             ## Changes\n\
             <!-- type: changes lang: yaml -->\n",
        );

        let headings: Vec<&str> = doc.sections.iter().map(|s| s.heading.as_str()).collect();
        assert_eq!(headings, vec!["Tests", "Changes"]);
        assert_eq!(doc.sections[0].code_blocks.len(), 1);
        assert_eq!(doc.sections[0].code_blocks[0].lang, "yaml");
    }

    #[test]
    fn parse_attr_style_annotation_preserves_attributes() {
        let doc = parse(
            "test.md",
            "## Wireframe\n\
             <!-- score-section type=\"wireframe\" lang=\"yaml\" workspace=\"studio\" role=\"owner\" -->\n\
             ```yaml\n\
             tasks: []\n\
             ```\n",
        );

        let annotation = doc.sections[0].annotation.as_ref().unwrap();
        assert_eq!(annotation.section_type, "wireframe");
        assert_eq!(annotation.lang, "yaml");
        assert_eq!(
            annotation.attributes.get("workspace").map(String::as_str),
            Some("studio")
        );
        assert_eq!(
            annotation.attributes.get("role").map(String::as_str),
            Some("owner")
        );
    }
}
````

             ## Review 1\n\
             **Verdict:** approved\n",
        );

        assert_eq!(doc.sections.len(), 1);
        assert_eq!(doc.sections[0].heading, "Logic");
        assert!(!doc.sections[0].body.contains("Review 1"));
    }

    #[test]
    fn parse_ignores_h2_inside_fenced_blocks() {
        let doc = parse(
            "test.md",
            "## Logic\n\
             <!-- type: logic lang: mermaid -->\n\
             ```markdown\n\
             ## Not A Section\n\
             ```\n\
             \n\
             ## Changes\n\
             <!-- type: changes lang: yaml -->\n",
        );

        let headings: Vec<&str> = doc.sections.iter().map(|s| s.heading.as_str()).collect();
        assert_eq!(headings, vec!["Logic", "Changes"]);
    }

    #[test]
    fn parse_ignores_indented_fixture_markdown_inside_yaml_fence() {
        let doc = parse(
            "test.md",
            "## Tests\n\
             <!-- type: tests lang: yaml -->\n\
             ```yaml\n\
             tests:\n\
               - name: embedded_fixture\n\
                 body: |\n\
                   ## Not A Section\n\
                   <!-- type: logic lang: mermaid -->\n\
                   ```mermaid\n\
                   flowchart TD\n\
                       A --> B\n\
                   ```\n\
             ```\n\
             \n\
             ## Changes\n\
             <!-- type: changes lang: yaml -->\n",
        );

        let headings: Vec<&str> = doc.sections.iter().map(|s| s.heading.as_str()).collect();
        assert_eq!(headings, vec!["Tests", "Changes"]);
        assert_eq!(doc.sections[0].code_blocks.len(), 1);
        assert_eq!(doc.sections[0].code_blocks[0].lang, "yaml");
    }

    #[test]
    fn parse_attr_style_annotation_preserves_attributes() {
        let doc = parse(
            "test.md",
            "## Wireframe\n\
             <!-- score-section type=\"wireframe\" lang=\"yaml\" workspace=\"studio\" role=\"owner\" -->\n\
             ```yaml\n\
             tasks: []\n\
             ```\n",
        );

        let annotation = doc.sections[0].annotation.as_ref().unwrap();
        assert_eq!(annotation.section_type, "wireframe");
        assert_eq!(annotation.lang, "yaml");
        assert_eq!(
            annotation.attributes.get("workspace").map(String::as_str),
            Some("studio")
        );
        assert_eq!(
            annotation.attributes.get("role").map(String::as_str),
            Some("owner")
        );
    }
}
````

             ## Review 1\n\
             **Verdict:** approved\n",
        );

        assert_eq!(doc.sections.len(), 1);
        assert_eq!(doc.sections[0].heading, "Logic");
        assert!(!doc.sections[0].body.contains("Review 1"));
    }

    #[test]
    fn parse_ignores_h2_inside_fenced_blocks() {
        let doc = parse(
            "test.md",
            "## Logic\n\
             <!-- type: logic lang: mermaid -->\n\
             ```markdown\n\
             ## Not A Section\n\
             ```\n\
             \n\
             ## Changes\n\
             <!-- type: changes lang: yaml -->\n",
        );

        let headings: Vec<&str> = doc.sections.iter().map(|s| s.heading.as_str()).collect();
        assert_eq!(headings, vec!["Logic", "Changes"]);
    }

    #[test]
    fn parse_ignores_indented_fixture_markdown_inside_yaml_fence() {
        let doc = parse(
            "test.md",
            "## Tests\n\
             <!-- type: tests lang: yaml -->\n\
             ```yaml\n\
             tests:\n\
               - name: embedded_fixture\n\
                 body: |\n\
                   ## Not A Section\n\
                   <!-- type: logic lang: mermaid -->\n\
                   ```mermaid\n\
                   flowchart TD\n\
                       A --> B\n\
                   ```\n\
             ```\n\
             \n\
             ## Changes\n\
             <!-- type: changes lang: yaml -->\n",
        );

        let headings: Vec<&str> = doc.sections.iter().map(|s| s.heading.as_str()).collect();
        assert_eq!(headings, vec!["Tests", "Changes"]);
        assert_eq!(doc.sections[0].code_blocks.len(), 1);
        assert_eq!(doc.sections[0].code_blocks[0].lang, "yaml");
    }

    #[test]
    fn parse_attr_style_annotation_preserves_attributes() {
        let doc = parse(
            "test.md",
            "## Wireframe\n\
             <!-- score-section type=\"wireframe\" lang=\"yaml\" workspace=\"studio\" role=\"owner\" -->\n\
             ```yaml\n\
             tasks: []\n\
             ```\n",
        );

        let annotation = doc.sections[0].annotation.as_ref().unwrap();
        assert_eq!(annotation.section_type, "wireframe");
        assert_eq!(annotation.lang, "yaml");
        assert_eq!(
            annotation.attributes.get("workspace").map(String::as_str),
            Some("studio")
        );
        assert_eq!(
            annotation.attributes.get("role").map(String::as_str),
            Some("owner")
        );
    }
}
````

             ## Review 1\n\
             **Verdict:** approved\n",
        );

        assert_eq!(doc.sections.len(), 1);
        assert_eq!(doc.sections[0].heading, "Logic");
        assert!(!doc.sections[0].body.contains("Review 1"));
    }

    #[test]
    fn parse_ignores_h2_inside_fenced_blocks() {
        let doc = parse(
            "test.md",
            "## Logic\n\
             <!-- type: logic lang: mermaid -->\n\
             ```markdown\n\
             ## Not A Section\n\
             ```\n\
             \n\
             ## Changes\n\
             <!-- type: changes lang: yaml -->\n",
        );

        let headings: Vec<&str> = doc.sections.iter().map(|s| s.heading.as_str()).collect();
        assert_eq!(headings, vec!["Logic", "Changes"]);
    }

    #[test]
    fn parse_ignores_indented_fixture_markdown_inside_yaml_fence() {
        let doc = parse(
            "test.md",
            "## Tests\n\
             <!-- type: tests lang: yaml -->\n\
             ```yaml\n\
             tests:\n\
               - name: embedded_fixture\n\
                 body: |\n\
                   ## Not A Section\n\
                   <!-- type: logic lang: mermaid -->\n\
                   ```mermaid\n\
                   flowchart TD\n\
                       A --> B\n\
                   ```\n\
             ```\n\
             \n\
             ## Changes\n\
             <!-- type: changes lang: yaml -->\n",
        );

        let headings: Vec<&str> = doc.sections.iter().map(|s| s.heading.as_str()).collect();
        assert_eq!(headings, vec!["Tests", "Changes"]);
        assert_eq!(doc.sections[0].code_blocks.len(), 1);
        assert_eq!(doc.sections[0].code_blocks[0].lang, "yaml");
    }

    #[test]
    fn parse_attr_style_annotation_preserves_attributes() {
        let doc = parse(
            "test.md",
            "## Wireframe\n\
             <!-- score-section type=\"wireframe\" lang=\"yaml\" workspace=\"studio\" role=\"owner\" -->\n\
             ```yaml\n\
             tasks: []\n\
             ```\n",
        );

        let annotation = doc.sections[0].annotation.as_ref().unwrap();
        assert_eq!(annotation.section_type, "wireframe");
        assert_eq!(annotation.lang, "yaml");
        assert_eq!(
            annotation.attributes.get("workspace").map(String::as_str),
            Some("studio")
        );
        assert_eq!(
            annotation.attributes.get("role").map(String::as_str),
            Some("owner")
        );
    }
}
````

             ## Review 1\n\
             **Verdict:** approved\n",
        );

        assert_eq!(doc.sections.len(), 1);
        assert_eq!(doc.sections[0].heading, "Logic");
        assert!(!doc.sections[0].body.contains("Review 1"));
    }

    #[test]
    fn parse_ignores_h2_inside_fenced_blocks() {
        let doc = parse(
            "test.md",
            "## Logic\n\
             <!-- type: logic lang: mermaid -->\n\
             ```markdown\n\
             ## Not A Section\n\
             ```\n\
             \n\
             ## Changes\n\
             <!-- type: changes lang: yaml -->\n",
        );

        let headings: Vec<&str> = doc.sections.iter().map(|s| s.heading.as_str()).collect();
        assert_eq!(headings, vec!["Logic", "Changes"]);
    }

    #[test]
    fn parse_ignores_indented_fixture_markdown_inside_yaml_fence() {
        let doc = parse(
            "test.md",
            "## Tests\n\
             <!-- type: tests lang: yaml -->\n\
             ```yaml\n\
             tests:\n\
               - name: embedded_fixture\n\
                 body: |\n\
                   ## Not A Section\n\
                   <!-- type: logic lang: mermaid -->\n\
                   ```mermaid\n\
                   flowchart TD\n\
                       A --> B\n\
                   ```\n\
             ```\n\
             \n\
             ## Changes\n\
             <!-- type: changes lang: yaml -->\n",
        );

        let headings: Vec<&str> = doc.sections.iter().map(|s| s.heading.as_str()).collect();
        assert_eq!(headings, vec!["Tests", "Changes"]);
        assert_eq!(doc.sections[0].code_blocks.len(), 1);
        assert_eq!(doc.sections[0].code_blocks[0].lang, "yaml");
    }

    #[test]
    fn parse_attr_style_annotation_preserves_attributes() {
        let doc = parse(
            "test.md",
            "## Wireframe\n\
             <!-- score-section type=\"wireframe\" lang=\"yaml\" workspace=\"studio\" role=\"owner\" -->\n\
             ```yaml\n\
             tasks: []\n\
             ```\n",
        );

        let annotation = doc.sections[0].annotation.as_ref().unwrap();
        assert_eq!(annotation.section_type, "wireframe");
        assert_eq!(annotation.lang, "yaml");
        assert_eq!(
            annotation.attributes.get("workspace").map(String::as_str),
            Some("studio")
        );
        assert_eq!(
            annotation.attributes.get("role").map(String::as_str),
            Some("owner")
        );
    }
}
````

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/spec_alignment/parser.rs
    action: modify
    section: source
    impl_mode: codegen
    description: |
      Source template owns the complete spec-alignment parser module and tests.
```
