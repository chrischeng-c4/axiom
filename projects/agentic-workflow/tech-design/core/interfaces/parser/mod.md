---
id: projects-sdd-src-parser-mod-rs
fill_sections: [overview, source, changes]
capability_refs:
  - id: aw-core-client-model-workitem-first-artifact-lifecycle
    role: primary
    gap: core-concept-model-and-invariants
    claim: core-concept-model-and-invariants
    coverage: full
    rationale: "Core model/parser TDs define AW Core domain nouns, invariants, and artifact structure."
---

# Standardized projects/agentic-workflow/src/parser/mod.rs

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/agentic-workflow/src/parser/mod.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `archive_review` | projects/agentic-workflow/src/parser/mod.rs | module | pub | 3 |  |
| `challenge` | projects/agentic-workflow/src/parser/mod.rs | module | pub | 4 |  |
| `frontmatter` | projects/agentic-workflow/src/parser/mod.rs | module | pub | 5 |  |
| `inline_yaml` | projects/agentic-workflow/src/parser/mod.rs | module | pub | 6 |  |
| `markdown` | projects/agentic-workflow/src/parser/mod.rs | module | pub | 7 |  |
| `requirement` | projects/agentic-workflow/src/parser/mod.rs | module | pub | 8 |  |
| `review` | projects/agentic-workflow/src/parser/mod.rs | module | pub | 9 |  |
| `scenario` | projects/agentic-workflow/src/parser/mod.rs | module | pub | 10 |  |
| `xml` | projects/agentic-workflow/src/parser/mod.rs | module | pub | 11 |  |
## Source
<!-- type: source lang: rust -->
<!-- source-from-target: strip-handwrite -->

<!-- source-snapshot: path=projects/agentic-workflow/src/parser/mod.rs -->
```rust
pub mod archive_review;
pub mod challenge;
pub mod frontmatter;
pub mod inline_yaml;
pub mod markdown;
pub mod requirement;
pub mod review;
pub mod scenario;
pub mod xml;

pub use archive_review::{get_review_path, parse_archive_review_verdict};
pub use challenge::{parse_challenge_verdict, ChallengeParser};
pub use frontmatter::{
    calculate_body_checksum, calculate_checksum, has_frontmatter, is_stale, normalize_content,
    parse_document, parse_frontmatter_value, split_frontmatter, ParsedDocument,
};
pub use inline_yaml::{
    extract_yaml_blocks, extract_yaml_blocks_with_lines, parse_issue_blocks,
    parse_requirement_blocks, parse_task_blocks, parse_typed_yaml_blocks, YamlBlock,
};
pub use markdown::extract_heading_section;
pub use requirement::RequirementParser;
pub use review::{parse_latest_review, parse_review_verdict, ReviewBlock};
pub use scenario::ScenarioParser;
pub use xml::{
    extract_xml_block, extract_xml_blocks, parse_xml_attributes, update_xml_blocks, wrap_in_xml,
    UpdateMode, XmlBlock,
};
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/parser/mod.rs
    action: modify
    section: source
    impl_mode: codegen
    description: |
      Source template owns the complete parser module facade.
```
