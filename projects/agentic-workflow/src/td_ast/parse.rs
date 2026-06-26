// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/interfaces/td_ast/parse.md#source
// CODEGEN-BEGIN
//! `parse_td(path)` — single entry point that converts a TD spec file on disk
//! into a structured `TDAst`.
//!
//! Reuses the existing primitives:
//!  - [`parse_all_section_annotations`] from `models::section`
//!  - [`extract_mermaid_plus_blocks`] from `generate::frontmatter`
//!
//! @spec projects/agentic-workflow/tech-design/core/interfaces/td_ast/types.md#logic-parse_td

use std::path::Path;

use serde_yaml::Value;

use crate::generate::frontmatter::{extract_mermaid_plus_blocks, MermaidPlusBlock};
use crate::models::section::parse_all_section_annotations;
use crate::models::spec_rules::SectionType;

use super::payloads::{
    AsyncApiPayload, CliManifestPayload, ConfigManifestPayload, JsonSchemaPayload, OpenApiPayload,
    OpenRpcPayload, TdParseErrorKind,
};
use super::types::{MermaidPlusPayload, SectionKind, TDAst, TDSection, TdParseError, TypedBody};

/// Construct a Stage 1B-shaped `TdParseError` with kind, source, and message
/// fields populated.
///
/// @spec projects/agentic-workflow/tech-design/core/interfaces/td_ast/payloads.md#schema
fn parse_error(
    kind: TdParseErrorKind,
    section_type: SectionType,
    line_start: usize,
    line_end: usize,
    message: impl Into<String>,
    source: Option<String>,
) -> TdParseError {
    TdParseError {
        kind,
        section_type,
        line_start,
        line_end,
        message: message.into(),
        source,
    }
}

/// Parse a TD spec file from disk into a [`TDAst`].
///
/// Pipeline (matches the spec's parse-td-entry state machine):
///   1. Read file bytes.
///   2. Parse YAML frontmatter (between leading `---` markers).
///   3. Run `parse_all_section_annotations` to discover section headings.
///   4. Run `extract_mermaid_plus_blocks` once for the whole file.
///   5. For each annotation: dispatch by [`SectionKind`], parse the body,
///      compute a content hash, and collect a [`TDSection`].
///
/// @spec projects/agentic-workflow/tech-design/core/interfaces/td_ast/types.md#logic-parse_td
pub fn parse_td(path: &Path) -> Result<TDAst, TdParseError> {
    let raw = std::fs::read_to_string(path).map_err(|e| {
        parse_error(
            TdParseErrorKind::Generic,
            SectionType::Overview,
            0,
            0,
            format!("failed to read {}: {}", path.display(), e),
            Some(e.to_string()),
        )
    })?;
    parse_td_str(&raw)
}

/// Same as [`parse_td`] but operates on an in-memory string — used by tests
/// and by callers that have already loaded the file.
///
/// @spec projects/agentic-workflow/tech-design/core/interfaces/td_ast/types.md#logic-parse_td
pub fn parse_td_str(raw: &str) -> Result<TDAst, TdParseError> {
    // 1. Split frontmatter.
    let (fm_value, body, body_offset) = split_frontmatter(raw)?;

    // 2. Discover annotations + mermaid blocks once.
    let annotations = parse_all_section_annotations(body);
    let mermaid_blocks = extract_mermaid_plus_blocks(body);
    let lines: Vec<&str> = body.lines().collect();

    // 3. Walk annotations in order, dispatch by SectionKind.
    let mut sections = Vec::with_capacity(annotations.len());
    for (idx, (heading_line0, meta)) in annotations.iter().enumerate() {
        let line_start = body_offset + heading_line0 + 1; // 1-based, file-absolute
        let line_end = if idx + 1 < annotations.len() {
            body_offset + annotations[idx + 1].0
        } else {
            body_offset + lines.len()
        };

        let kind = SectionKind::for_section_type(meta.section_type);
        let block = extract_section_body(body, *heading_line0, idx, &annotations);
        let typed = parse_typed_body(kind, &block, &mermaid_blocks, idx).map_err(
            |(err_kind, msg, source)| {
                parse_error(
                    err_kind,
                    meta.section_type,
                    line_start,
                    line_end,
                    format!(
                        "section '{}' at lines {}..{}: {}",
                        meta.section_type.as_str(),
                        line_start,
                        line_end,
                        msg
                    ),
                    source,
                )
            },
        )?;

        let content_hash = compute_hash(&typed);

        sections.push(TDSection {
            section_type: meta.section_type,
            lang: meta
                .lang
                .clone()
                .unwrap_or_else(|| meta.section_type.default_lang().to_string()),
            body: typed,
            line_start,
            line_end,
            content_hash,
        });
    }

    Ok(TDAst {
        frontmatter: fm_value,
        sections,
    })
}

/// Split a TD file's leading `---\n...\n---` YAML frontmatter from its body.
///
/// Returns `(parsed_value, body_str, body_line_offset)`.
/// If no frontmatter is present, returns `(Null, raw, 0)`.
///
/// @spec projects/agentic-workflow/tech-design/core/interfaces/td_ast/types.md#logic-parse_td
fn split_frontmatter(raw: &str) -> Result<(Value, &str, usize), TdParseError> {
    let lines: Vec<&str> = raw.lines().collect();
    if lines.first().map(|l| l.trim()) != Some("---") {
        return Ok((Value::Null, raw, 0));
    }
    let mut end = None;
    for (i, line) in lines.iter().enumerate().skip(1) {
        if line.trim() == "---" {
            end = Some(i);
            break;
        }
    }
    let end = end.ok_or_else(|| {
        parse_error(
            TdParseErrorKind::Frontmatter,
            SectionType::Overview,
            1,
            lines.len(),
            "frontmatter has no closing '---' marker",
            None,
        )
    })?;
    let fm_raw = lines[1..end].join("\n");
    let fm: Value = serde_yaml::from_str(&fm_raw).map_err(|e| {
        parse_error(
            TdParseErrorKind::Frontmatter,
            SectionType::Overview,
            1,
            end + 1,
            format!("frontmatter YAML parse error: {}", e),
            Some(e.to_string()),
        )
    })?;
    // Compute byte offset to start of body (after closing ---\n).
    let mut consumed = 0usize;
    for line in lines.iter().take(end + 1) {
        consumed += line.len() + 1; // include the \n we split on
    }
    let body = if consumed >= raw.len() {
        ""
    } else {
        &raw[consumed..]
    };
    Ok((fm, body, end + 1))
}

/// Extract the body slice between the heading at `annotations[idx]` and the
/// next heading (or EOF). Returns the joined lines, exclusive of the heading
/// itself.
///
/// @spec projects/agentic-workflow/tech-design/core/interfaces/td_ast/types.md#logic-parse_td
fn extract_section_body(
    body: &str,
    heading_line0: usize,
    idx: usize,
    annotations: &[(usize, crate::models::section::SectionMeta)],
) -> String {
    let lines: Vec<&str> = body.lines().collect();
    let start = (heading_line0 + 1).min(lines.len());
    let end = if idx + 1 < annotations.len() {
        annotations[idx + 1].0
    } else {
        lines.len()
    };
    let end = end.min(lines.len()).max(start);
    if start >= lines.len() {
        String::new()
    } else {
        lines[start..end].join("\n")
    }
}

/// Extract the first fenced code block (```lang ... ```) from a section body.
///
/// Returns `(lang, content)` if a fence is found.
///
/// @spec projects/agentic-workflow/tech-design/core/interfaces/td_ast/types.md#logic-parse_td
fn first_code_fence(section_body: &str) -> Option<(String, String)> {
    let lines: Vec<&str> = section_body.lines().collect();
    let mut i = 0;
    while i < lines.len() {
        if let Some((opener, lang)) = code_fence_open(lines[i]) {
            let content_start = i + 1;
            let mut content_end = lines.len();
            for (j, line) in lines.iter().enumerate().skip(content_start) {
                if code_fence_closes(line, &opener) {
                    content_end = j;
                    break;
                }
            }
            let content = if content_start <= content_end {
                lines[content_start..content_end].join("\n")
            } else {
                String::new()
            };
            return Some((lang, content));
        }
        i += 1;
    }
    None
}

fn code_fence_open(line: &str) -> Option<(String, String)> {
    let trimmed = line.trim_start();
    let first = trimmed.as_bytes().first().copied()?;
    if first != b'`' && first != b'~' {
        return None;
    }
    let count = trimmed
        .as_bytes()
        .iter()
        .take_while(|byte| **byte == first)
        .count();
    if count < 3 {
        return None;
    }
    let opener = trimmed[..count].to_string();
    let lang = trimmed[count..]
        .split_whitespace()
        .next()
        .unwrap_or("")
        .to_string();
    Some((opener, lang))
}

fn code_fence_closes(line: &str, opener: &str) -> bool {
    let Some((marker, _lang)) = code_fence_open(line) else {
        return false;
    };
    marker.as_bytes().first() == opener.as_bytes().first()
        && marker.len() >= opener.len()
        && line.trim_start()[marker.len()..].trim().is_empty()
}

/// Decide whether the section's fenced block is empty / placeholder.
///
/// @spec projects/agentic-workflow/tech-design/core/interfaces/td_ast/types.md#logic-parse_td
fn is_placeholder_block(content: &str) -> bool {
    content.trim().is_empty()
}

/// Dispatch by [`SectionKind`] and parse the section's body into the
/// appropriate [`TypedBody`] variant.
///
/// Stage 1B: errors are returned as `(TdParseErrorKind, message, source)`
/// tuples so the caller can preserve the discriminant and original serde
/// error text in [`TdParseError`].
///
/// @spec projects/agentic-workflow/tech-design/core/interfaces/td_ast/types.md#logic-parse_td
/// @spec projects/agentic-workflow/tech-design/core/interfaces/td_ast/payloads.md#schema
fn parse_typed_body(
    kind: SectionKind,
    section_body: &str,
    mermaid_blocks: &[MermaidPlusBlock],
    section_idx: usize,
) -> Result<TypedBody, (TdParseErrorKind, String, Option<String>)> {
    let fence = first_code_fence(section_body);

    // If the section has no fenced block at all, treat it as Markdown for
    // markdown-family sections and Placeholder elsewhere.
    let (lang, content) = match fence {
        Some(f) => f,
        None => {
            return Ok(match kind {
                SectionKind::MarkdownFamily => TypedBody::Markdown(section_body.trim().to_string()),
                _ => TypedBody::Placeholder,
            });
        }
    };

    if is_placeholder_block(&content) {
        return Ok(TypedBody::Placeholder);
    }

    let typed = match kind {
        SectionKind::MermaidFamily => {
            let block = mermaid_blocks
                .iter()
                .find(|b| {
                    section_body.contains(&b.body) || section_body.contains(&b.frontmatter_raw)
                })
                .cloned();
            if let Some(b) = block {
                TypedBody::MermaidPlus(MermaidPlusPayload::from(b))
            } else if lang == "mermaid" {
                TypedBody::Unsupported(content)
            } else {
                TypedBody::Placeholder
            }
        }
        SectionKind::JsonSchemaFamily => {
            let p: JsonSchemaPayload = serde_yaml::from_str(&content).map_err(|e| {
                (
                    TdParseErrorKind::TypedPayloadParse,
                    format!("JSON Schema typed-payload parse error: {}", e),
                    Some(e.to_string()),
                )
            })?;
            TypedBody::JsonSchema(p)
        }
        SectionKind::OpenRpcFamily => {
            let p = OpenRpcPayload::from_yaml_str(&content).map_err(|e| {
                (
                    TdParseErrorKind::TypedPayloadParse,
                    format!("OpenRPC typed-payload parse error: {}", e),
                    Some(e.to_string()),
                )
            })?;
            TypedBody::OpenRpc(p)
        }
        SectionKind::OpenApiFamily => {
            let p = OpenApiPayload::from_yaml_str(&content).map_err(|e| {
                (
                    TdParseErrorKind::TypedPayloadParse,
                    format!("OpenAPI typed-payload parse error: {}", e),
                    Some(e.to_string()),
                )
            })?;
            TypedBody::OpenApi(p)
        }
        SectionKind::AsyncApiFamily => {
            let p = AsyncApiPayload::from_yaml_str(&content).map_err(|e| {
                (
                    TdParseErrorKind::TypedPayloadParse,
                    format!("AsyncAPI typed-payload parse error: {}", e),
                    Some(e.to_string()),
                )
            })?;
            TypedBody::AsyncApi(p)
        }
        SectionKind::CliFamily => {
            let p: CliManifestPayload = serde_yaml::from_str(&content).map_err(|e| {
                (
                    TdParseErrorKind::TypedPayloadParse,
                    format!("CLI manifest typed-payload parse error: {}", e),
                    Some(e.to_string()),
                )
            })?;
            TypedBody::CliManifest(p)
        }
        SectionKind::ConfigFamily => {
            let p: ConfigManifestPayload = serde_yaml::from_str(&content).map_err(|e| {
                (
                    TdParseErrorKind::TypedPayloadParse,
                    format!("Config manifest typed-payload parse error: {}", e),
                    Some(e.to_string()),
                )
            })?;
            TypedBody::ConfigManifest(p)
        }
        SectionKind::ChangesFamily => {
            // Changes round-trips through JsonSchemaPayload's `extra` catch-all
            // — Changes itself has no dedicated TypedBody variant. The orphan-
            // changes-target validator walks the parsed Value via `extra`.
            let p: JsonSchemaPayload = serde_yaml::from_str(&content).map_err(|e| {
                (
                    TdParseErrorKind::TypedPayloadParse,
                    format!("Changes typed-payload parse error: {}", e),
                    Some(e.to_string()),
                )
            })?;
            TypedBody::JsonSchema(p)
        }
        SectionKind::RustSourceUnitFamily => {
            // @spec projects/agentic-workflow/tech-design/logic/aw-td-ast-parse-rust-source-unit-sections-as-typed-td-bodies.md#logic
            let unit = crate::generate::rust_source_unit::parse(&content).map_err(|e| {
                (
                    TdParseErrorKind::TypedPayloadParse,
                    format!("Rust source-unit typed-payload parse error: {}", e),
                    Some(e.to_string()),
                )
            })?;
            TypedBody::RustSourceUnit(unit)
        }
        SectionKind::MarkdownFamily => TypedBody::Markdown(content),
        SectionKind::Unsupported => TypedBody::Unsupported(content),
    };
    let _ = section_idx;
    Ok(typed)
}

/// Compute a deterministic content hash over the canonical-serialised body.
///
/// Hashing rules (R4, R8):
///  - `MermaidPlus`: hash the `frontmatter_raw` only; `rendered_body` is derived.
///  - `Placeholder` / `Unsupported`: no hash.
///  - All other variants: hash the canonical YAML serialisation of the payload.
///
/// We use the lower 64 bits of the blake3 256-bit digest to fit `Option<u64>`.
///
/// @spec projects/agentic-workflow/tech-design/core/interfaces/td_ast/types.md#logic-parse_td
fn compute_hash(body: &TypedBody) -> Option<u64> {
    let bytes: Vec<u8> = match body {
        TypedBody::MermaidPlus(p) => p.frontmatter_raw.as_bytes().to_vec(),
        TypedBody::JsonSchema(p) => serde_yaml::to_string(p).ok()?.into_bytes(),
        TypedBody::OpenRpc(p) => serde_yaml::to_string(p).ok()?.into_bytes(),
        TypedBody::OpenApi(p) => serde_yaml::to_string(p).ok()?.into_bytes(),
        TypedBody::AsyncApi(p) => serde_yaml::to_string(p).ok()?.into_bytes(),
        TypedBody::CliManifest(p) => serde_yaml::to_string(p).ok()?.into_bytes(),
        TypedBody::ConfigManifest(p) => serde_yaml::to_string(p).ok()?.into_bytes(),
        TypedBody::RustSourceUnit(p) => serde_yaml::to_string(p).ok()?.into_bytes(),
        TypedBody::Markdown(s) => s.as_bytes().to_vec(),
        TypedBody::Placeholder | TypedBody::Unsupported(_) => return None,
    };
    let digest = blake3::hash(&bytes);
    let arr = digest.as_bytes();
    Some(u64::from_le_bytes([
        arr[0], arr[1], arr[2], arr[3], arr[4], arr[5], arr[6], arr[7],
    ]))
}

#[cfg(test)]
mod tests {
    use super::*;

    const FIXTURE: &str = concat!(
        "---\n",
        "id: test-fixture\n",
        "fill_sections: [overview, schema]\n",
        "---\n\n",
        "# Test Fixture\n\n",
        "## Overview\n",
        "<!-- type: overview lang: markdown -->\n\n",
        "```markdown\n",
        "This is the overview prose.\n",
        "```\n\n",
        "## Schema\n",
        "<!-- type: schema lang: yaml -->\n\n",
        "```yaml\n",
        "$schema: \"https://json-schema.org/draft/2020-12/schema\"\n",
        "$id: test-fixture#schema\n",
        "definitions:\n",
        "  Foo:\n",
        "    type: object\n",
        "  Bar:\n",
        "    type: string\n",
        "```\n\n",
        "## State Machine\n",
        "<!-- type: state-machine lang: mermaid -->\n\n",
        "```mermaid\n",
        "---\n",
        "id: test-sm\n",
        "nodes:\n",
        "  start: { kind: initial }\n",
        "  done: { kind: terminal }\n",
        "---\n",
        "stateDiagram-v2\n",
        "    [*] --> start\n",
        "    start --> done\n",
        "```\n",
    );

    #[test]
    fn parse_td_str_basic() {
        let ast = parse_td_str(FIXTURE).expect("parse");
        assert_eq!(
            ast.frontmatter.get("id").and_then(Value::as_str),
            Some("test-fixture")
        );
        assert!(!ast.sections.is_empty());
    }

    #[test]
    fn parse_td_str_dispatches_by_kind() {
        let ast = parse_td_str(FIXTURE).expect("parse");
        let by_type: std::collections::HashMap<_, _> = ast
            .sections
            .iter()
            .map(|s| (s.section_type, &s.body))
            .collect();
        assert!(matches!(
            by_type.get(&SectionType::Overview).unwrap(),
            TypedBody::Markdown(_)
        ));
        assert!(matches!(
            by_type.get(&SectionType::Schema).unwrap(),
            TypedBody::JsonSchema(_)
        ));
        assert!(matches!(
            by_type.get(&SectionType::StateMachine).unwrap(),
            TypedBody::MermaidPlus(_)
        ));
    }

    #[test]
    fn parse_td_str_parses_rust_source_unit_body() {
        let raw = concat!(
            "---\n",
            "id: rust-source-unit-fixture\n",
            "---\n\n",
            "## Rust Source Unit\n",
            "<!-- type: rust-source-unit lang: rust -->\n\n",
            "```rust\n",
            "pub struct Demo {\n",
            "    pub value: i32,\n",
            "}\n",
            "```\n",
        );

        let ast = parse_td_str(raw).expect("parse rust-source-unit");
        let section = ast
            .sections
            .iter()
            .find(|s| s.section_type == SectionType::RustSourceUnit)
            .expect("rust-source-unit section");
        let TypedBody::RustSourceUnit(unit) = &section.body else {
            panic!("expected RustSourceUnit typed body");
        };

        assert_eq!(unit.emit(), "pub struct Demo {\n    pub value: i32,\n}");
        assert!(
            section.content_hash.is_some(),
            "rust-source-unit bodies must carry a content hash"
        );
    }

    #[test]
    fn parse_td_str_rejects_invalid_rust_source_unit() {
        let raw = concat!(
            "---\n",
            "id: bad-rust-source-unit\n",
            "---\n\n",
            "## Rust Source Unit\n",
            "<!-- type: rust-source-unit lang: rust -->\n\n",
            "```rust\n",
            "pub struct {\n",
            "```\n",
        );

        let err = parse_td_str(raw).unwrap_err();
        assert_eq!(
            err.kind,
            super::super::payloads::TdParseErrorKind::TypedPayloadParse
        );
        assert_eq!(err.section_type, SectionType::RustSourceUnit);
        assert!(err.message.contains("Rust source-unit"));
        assert!(err.source.is_some(), "parse error source must be carried");
    }

    #[test]
    fn parse_td_str_hash_is_deterministic() {
        let ast1 = parse_td_str(FIXTURE).expect("parse");
        let ast2 = parse_td_str(FIXTURE).expect("parse");
        let hashes1: Vec<_> = ast1.sections.iter().map(|s| s.content_hash).collect();
        let hashes2: Vec<_> = ast2.sections.iter().map(|s| s.content_hash).collect();
        assert_eq!(hashes1, hashes2);
        // At least one section should have a non-None hash.
        assert!(hashes1.iter().any(|h| h.is_some()));
    }

    #[test]
    fn parse_td_str_entities_listed() {
        use super::super::entities::SectionEntities;
        let ast = parse_td_str(FIXTURE).expect("parse");
        let schema = ast
            .sections
            .iter()
            .find(|s| s.section_type == SectionType::Schema)
            .expect("schema section");
        let names: Vec<_> = schema.body.entities().into_iter().map(|e| e.id).collect();
        assert!(names.contains(&"Foo".to_string()));
        assert!(names.contains(&"Bar".to_string()));
    }

    #[test]
    fn extract_section_body_handles_overlapping_or_empty_ranges() {
        let annotations = vec![(
            99,
            crate::models::section::SectionMeta::new(SectionType::Overview, None),
        )];

        assert_eq!(extract_section_body("## Overview", 99, 0, &annotations), "");
    }

    #[test]
    fn first_code_fence_respects_long_outer_fence() {
        let body = concat!(
            "<!-- type: source lang: rust -->\n\n",
            "```````rust\n",
            "fn fixture() {\n",
            "    let md = r#\"\n",
            "```yaml\n",
            "changes: []\n",
            "```\n",
            "\"#;\n",
            "}\n",
            "```````\n",
        );

        let (lang, content) = first_code_fence(body).expect("source fence");
        assert_eq!(lang, "rust");
        assert!(content.contains("```yaml\nchanges: []\n```"));
        assert!(content.contains("fn fixture()"));
    }

    #[test]
    fn parse_td_str_mermaid_hash_excludes_rendered_body() {
        // Two fixtures with identical frontmatter but different rendered
        // body should yield identical MermaidPlus hashes (R4).
        let a = concat!(
            "---\n",
            "id: a\n",
            "---\n\n",
            "## State Machine\n",
            "<!-- type: state-machine lang: mermaid -->\n\n",
            "```mermaid\n",
            "---\n",
            "id: same\n",
            "nodes:\n",
            "  start: { kind: initial }\n",
            "---\n",
            "stateDiagram-v2\n",
            "    [*] --> start\n",
            "```\n",
        );
        let b = concat!(
            "---\n",
            "id: b\n",
            "---\n\n",
            "## State Machine\n",
            "<!-- type: state-machine lang: mermaid -->\n\n",
            "```mermaid\n",
            "---\n",
            "id: same\n",
            "nodes:\n",
            "  start: { kind: initial }\n",
            "---\n",
            "stateDiagram-v2\n",
            "    [*] --> start\n",
            "    start --> done\n",
            "```\n",
        );
        let ast_a = parse_td_str(a).unwrap();
        let ast_b = parse_td_str(b).unwrap();
        let h_a = ast_a.sections[0].content_hash;
        let h_b = ast_b.sections[0].content_hash;
        assert_eq!(h_a, h_b, "MermaidPlus hash must cover frontmatter only");
    }

    #[test]
    fn parse_td_str_rejects_unterminated_frontmatter() {
        let bad = "---\nid: oops\nno_close: true\n\n## Body\n";
        let err = parse_td_str(bad).unwrap_err();
        assert!(err.message.contains("closing"));
        assert_eq!(
            err.kind,
            super::super::payloads::TdParseErrorKind::Frontmatter
        );
    }

    /// R2: typed-payload deserialisation failures surface
    /// `TdParseErrorKind::TypedPayloadParse` with `expected_type` set to
    /// the failing SectionType and `source` carrying the underlying serde
    /// error text.
    #[test]
    fn parse_td_str_typed_payload_parse_error_carries_kind_and_source() {
        // Schema section with a body that is invalid YAML for the
        // JsonSchemaPayload deserialiser (the `definitions` key has a
        // non-mapping value, which cannot be parsed into
        // `BTreeMap<String, PayloadTypeDef>`).
        let bad = "---\nid: bad-schema\n---\n\n## Schema\n<!-- type: schema lang: yaml -->\n\n```yaml\ndefinitions: \"this is a string, not a map\"\n```\n";
        let err = parse_td_str(bad).unwrap_err();
        assert_eq!(
            err.kind,
            super::super::payloads::TdParseErrorKind::TypedPayloadParse
        );
        assert_eq!(err.section_type, SectionType::Schema);
        assert!(err.source.is_some(), "source must be carried per R2");
        assert!(err.message.contains("Schema") || err.message.contains("schema"));
    }
}

// CODEGEN-END
