---
id: semantic-agentic-workflow-parser
summary: Semantic coverage for "projects/agentic-workflow/src/parser"
fill_sections: [schema, tests, changes]
capability_refs:
  - id: td-cb-lifecycle-automation
    role: primary
    gap: cb-lifecycle-dispatch
    claim: cb-lifecycle-dispatch
    coverage: full
    rationale: "This semantic TD covers TD/CB generation, parsing, validation, and code artifact lifecycle source behavior."
---

# Semantic TD: agentic-workflow/parser

## Schema
<!-- type: schema lang: yaml -->

```yaml
semantic_domain:
  key: "agentic-workflow/parser"
  source_group: "projects/agentic-workflow/src/parser"
  coverage_kind: semantic
  evidence:
    source_units:
      - path: "projects/agentic-workflow/src/parser/xml.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["data_model", "enum_model", "service_method"]
        symbols:
          - name: "XmlBlock"
            kind: "struct"
            public: true
          - name: "UpdateMode"
            kind: "enum"
            public: true
          - name: "extract_xml_blocks"
            kind: "function"
            public: true
          - name: "extract_xml_block"
            kind: "function"
            public: true
          - name: "parse_xml_attributes"
            kind: "function"
            public: true
          - name: "wrap_in_xml"
            kind: "function"
            public: true
          - name: "update_xml_blocks"
            kind: "function"
            public: true
          - name: "tests"
            kind: "module"
            public: false
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "source"
          section_type: "schema"
          domain: "projects/agentic-workflow/src/parser"
      - path: "projects/agentic-workflow/src/parser/review.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["data_model", "service_method"]
        symbols:
          - name: "parse_review_verdict"
            kind: "function"
            public: true
          - name: "normalize_verdict_section"
            kind: "function"
            public: false
          - name: "ReviewBlock"
            kind: "struct"
            public: true
          - name: "parse_latest_review"
            kind: "function"
            public: true
          - name: "tests"
            kind: "module"
            public: false
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "source"
          section_type: "schema"
          domain: "projects/agentic-workflow/src/parser"
      - path: "projects/agentic-workflow/src/parser/frontmatter.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["data_model", "service_method"]
        symbols:
          - name: "ParsedDocument"
            kind: "struct"
            public: true
          - name: "parse_document"
            kind: "function"
            public: true
          - name: "parse_frontmatter_value"
            kind: "function"
            public: true
          - name: "has_frontmatter"
            kind: "function"
            public: true
          - name: "normalize_content"
            kind: "function"
            public: true
          - name: "split_frontmatter"
            kind: "function"
            public: true
          - name: "normalize_for_checksum"
            kind: "function"
            public: false
          - name: "calculate_checksum"
            kind: "function"
            public: true
          - name: "is_stale"
            kind: "function"
            public: true
          - name: "calculate_body_checksum"
            kind: "function"
            public: true
          - name: "tests"
            kind: "module"
            public: false
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "source"
          section_type: "schema"
          domain: "projects/agentic-workflow/src/parser"
      - path: "projects/agentic-workflow/src/parser/requirement.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["data_model"]
        symbols:
          - name: "RequirementParser"
            kind: "struct"
            public: true
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "source"
          section_type: "schema"
          domain: "projects/agentic-workflow/src/parser"
      - path: "projects/agentic-workflow/src/parser/inline_yaml.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["data_model", "service_method"]
        symbols:
          - name: "YamlBlock"
            kind: "struct"
            public: true
          - name: "TaskBlockWrapper"
            kind: "struct"
            public: true
          - name: "RequirementBlockWrapper"
            kind: "struct"
            public: true
          - name: "IssueBlockWrapper"
            kind: "struct"
            public: true
          - name: "extract_yaml_blocks"
            kind: "function"
            public: true
          - name: "extract_yaml_blocks_with_lines"
            kind: "function"
            public: true
          - name: "parse_typed_yaml_blocks"
            kind: "function"
            public: true
          - name: "parse_task_blocks"
            kind: "function"
            public: true
          - name: "parse_requirement_blocks"
            kind: "function"
            public: true
          - name: "parse_issue_blocks"
            kind: "function"
            public: true
          - name: "tests"
            kind: "module"
            public: false
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "source"
          section_type: "schema"
          domain: "projects/agentic-workflow/src/parser"
      - path: "projects/agentic-workflow/src/parser/challenge.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["data_model", "service_method"]
        symbols:
          - name: "parse_challenge_verdict"
            kind: "function"
            public: true
          - name: "parse_verdict_from_proposal"
            kind: "function"
            public: false
          - name: "parse_verdict_from_review_xml"
            kind: "function"
            public: false
          - name: "parse_verdict_from_checkboxes"
            kind: "function"
            public: false
          - name: "ChallengeParser"
            kind: "struct"
            public: true
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "source"
          section_type: "schema"
          domain: "projects/agentic-workflow/src/parser"
      - path: "projects/agentic-workflow/src/parser/mod.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["source_unit"]
        symbols:
          - name: "archive_review"
            kind: "module"
            public: true
          - name: "challenge"
            kind: "module"
            public: true
          - name: "frontmatter"
            kind: "module"
            public: true
          - name: "inline_yaml"
            kind: "module"
            public: true
          - name: "markdown"
            kind: "module"
            public: true
          - name: "requirement"
            kind: "module"
            public: true
          - name: "review"
            kind: "module"
            public: true
          - name: "scenario"
            kind: "module"
            public: true
          - name: "xml"
            kind: "module"
            public: true
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "source"
          section_type: "schema"
          domain: "projects/agentic-workflow/src/parser"
      - path: "projects/agentic-workflow/src/parser/scenario.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["data_model"]
        symbols:
          - name: "ScenarioParser"
            kind: "struct"
            public: true
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "source"
          section_type: "schema"
          domain: "projects/agentic-workflow/src/parser"
      - path: "projects/agentic-workflow/src/parser/archive_review.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["service_method"]
        symbols:
          - name: "parse_archive_review_verdict"
            kind: "function"
            public: true
          - name: "get_review_path"
            kind: "function"
            public: true
          - name: "tests"
            kind: "module"
            public: false
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "source"
          section_type: "schema"
          domain: "projects/agentic-workflow/src/parser"
      - path: "projects/agentic-workflow/src/parser/markdown.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["service_method"]
        symbols:
          - name: "extract_heading_section"
            kind: "function"
            public: true
          - name: "tests"
            kind: "module"
            public: false
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "source"
          section_type: "schema"
          domain: "projects/agentic-workflow/src/parser"
```

## Tests
<!-- type: tests lang: yaml -->

```yaml
tests:
  coverage_kind: semantic
  strategy: preserve observed source behavior while semantic coverage is promoted toward generator primitives
  evidence:
    source_tests: []
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
coverage_kind: semantic
changes:
  - path: "projects/agentic-workflow/src/parser/xml.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/agentic-workflow/src/parser/review.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/agentic-workflow/src/parser/frontmatter.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/agentic-workflow/src/parser/requirement.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/agentic-workflow/src/parser/inline_yaml.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/agentic-workflow/src/parser/challenge.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/agentic-workflow/src/parser/mod.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/agentic-workflow/src/parser/scenario.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/agentic-workflow/src/parser/archive_review.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/agentic-workflow/src/parser/markdown.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - action: annotate
    section: unit-test
    impl_mode: hand-written
    description: "Traceability metadata edge for the unit-test section."

```
