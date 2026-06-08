---
id: sdd-spec-alignment-models
fill_sections: [overview, schema, changes]
capability_refs:
  - id: existing-project-standardization
    role: primary
    gap: traceability-closure-gate
    claim: traceability-closure-gate
    coverage: full
    rationale: "Spec alignment interfaces implement TD/source annotation and coverage checks used by the traceability closure gate."
---

# Spec Alignment Models

## Overview
<!-- type: overview lang: markdown -->

Data types for spec alignment checking in
`projects/agentic-workflow/src/spec_alignment/models.rs`. Fourteen serde shapes:

- `SpecDocument` — parsed `.md` spec file (path, frontmatter, sections).
- `SpecSection` — single section (heading, line, optional annotation, code_blocks, body).
- `SectionAnnotation` — legacy `<!-- type: X lang: Y -->` or attr-style
  `<!-- score-section type="X" lang="Y" ... -->` parsed
  (section_type, lang, attributes).
- `CodeBlock` — fenced code block (lang, line, content, optional parsed_json).
- `ViolationKind` — 14-variant enum with `serde_rename_all: snake_case`.
- `Violation` — single violation (kind, message, lots of optional fields).
- `FileResult` — per-file result (path, status, violations Vec).
- `CheckResult` — aggregate result (files, total_violations, passed, optional coverage).
- `SpecAnnotation` — `@spec {path}#{id}` annotation in source code.
- `CoverageReport` — coverage analysis (covered, uncovered_requirements, unspecced_functions, stale_annotations, orphan_requirements, schema_struct_mismatches, coverage_ratio f64).
- `CoverageEntry` — single requirement coverage status.
- `OrphanRequirementEntry` — requirement not referenced by any scenario.
- `UnspeccedFunction` — public function without `@spec` annotation.
- `SchemaStructMismatchEntry` — schema/struct property mismatch.

Codegen replaces all 14 type declarations and the serde import they need.
Companion source templates own the module documentation and the
`ViolationKind` helper/display impls.

## Schema
<!-- type: schema lang: yaml -->

```yaml
definitions:
  ViolationKind:
    type: string
    enum: [MissingSectionAnnotation, DuplicateSection, FormatPriorityViolation, DuplicateDefinition, DefinitionConflictRequired, DefinitionConflictFieldName, DefinitionConflictSchema, RpcFieldConsistency, IoError, OrphanRequirement, NestedSchemaConflictRequired, NestedSchemaConflictSchema, NestedSchemaConflictFieldName, SchemaStructMismatch]
    description: Violation kinds emitted by spec alignment checking.
    x-rust-enum:
      derive: [Debug, Clone, PartialEq, Eq, Serialize, Deserialize]
      serde_rename_all: snake_case

  SpecDocument:
    type: object
    required: [path, frontmatter, sections]
    description: Parsed representation of a spec `.md` file.
    properties:
      path:
        type: string
        description: "File path (relative to project root)."
      frontmatter:
        type: object
        x-rust-type: "serde_json::Value"
        description: "Parsed YAML frontmatter."
      sections:
        type: array
        items: { $ref: "#/definitions/SpecSection" }
        x-rust-type: "Vec<SpecSection>"
        description: "Parsed sections."
    x-rust-struct:
      derive: [Debug, Clone, Serialize, Deserialize]

  SpecSection:
    type: object
    required: [heading, line, annotation, code_blocks, body]
    description: A single section parsed from heading + annotation + content.
    properties:
      heading:
        type: string
        description: "Heading text (without `##` prefix)."
      line:
        type: integer
        x-rust-type: "usize"
        description: "Line number of the `## Heading` (1-based)."
      annotation:
        type: object
        x-rust-type: "Option<SectionAnnotation>"
        description: "Section type annotation, if present."
      code_blocks:
        type: array
        items: { $ref: "#/definitions/CodeBlock" }
        x-rust-type: "Vec<CodeBlock>"
        description: "Fenced code blocks found within this section."
      body:
        type: string
        x-serde-default: true
        description: "Raw body text trimmed of surrounding whitespace."
    x-rust-struct:
      derive: [Debug, Clone, Serialize, Deserialize]

  SectionAnnotation:
    type: object
    required: [section_type, lang, attributes]
    description: "Section type annotation parsed from legacy or attr-style comments."
    properties:
      section_type:
        type: string
        description: "Declared section type (e.g. overview, config, logic)."
      lang:
        type: string
        description: "Declared lang (e.g. markdown, json, mermaid, yaml)."
      attributes:
        type: object
        x-rust-type: "std::collections::BTreeMap<String, String>"
        x-serde-default: true
        x-serde-skip-if: "std::collections::BTreeMap::is_empty"
        additionalProperties:
          type: string
        default: {}
        description: "Optional attr-style metadata excluding core type/lang keys."
    x-rust-struct:
      derive: [Debug, Clone, Serialize, Deserialize]

  CodeBlock:
    type: object
    required: [lang, line, content, parsed_json]
    description: A fenced code block within a section.
    properties:
      lang:
        type: string
        description: "Code fence language (json, yaml, mermaid, etc.)."
      line:
        type: integer
        x-rust-type: "usize"
        description: "Line number of opening fence (1-based)."
      content:
        type: string
        description: "Raw content between fences."
      parsed_json:
        type: object
        x-rust-type: "Option<serde_json::Value>"
        x-serde-skip-if: "Option::is_none"
        description: "Parsed JSON value if lang=json and content is valid JSON."
    x-rust-struct:
      derive: [Debug, Clone, Serialize, Deserialize]

  Violation:
    type: object
    required: [kind, message, heading, line, lines, name, expected_lang, field, details]
    description: A single validation violation.
    properties:
      kind:
        type: string
        x-rust-type: "ViolationKind"
        description: "Violation kind."
      message:
        type: string
        description: "Human-readable violation message."
      heading:
        type: string
        x-rust-type: "Option<String>"
        x-serde-skip-if: "Option::is_none"
        description: "Section heading (for format rules)."
      line:
        type: integer
        x-rust-type: "Option<usize>"
        x-serde-skip-if: "Option::is_none"
        description: "Primary line number."
      lines:
        type: array
        x-rust-type: "Option<Vec<usize>>"
        x-serde-skip-if: "Option::is_none"
        description: "Multiple line numbers (for duplicates)."
      name:
        type: string
        x-rust-type: "Option<String>"
        x-serde-skip-if: "Option::is_none"
        description: "Definition name (for logical rules)."
      expected_lang:
        type: string
        x-rust-type: "Option<String>"
        x-serde-skip-if: "Option::is_none"
        description: "Expected code fence lang."
      field:
        type: string
        x-rust-type: "Option<String>"
        x-serde-skip-if: "Option::is_none"
        description: "Field name (for schema/field conflicts)."
      details:
        type: object
        x-rust-type: "Option<serde_json::Value>"
        x-serde-skip-if: "Option::is_none"
        description: "Additional context."
    x-rust-struct:
      derive: [Debug, Clone, Serialize, Deserialize]

  FileResult:
    type: object
    required: [path, status, violations]
    description: Check result for a single file.
    properties:
      path:
        type: string
        description: "File path."
      status:
        type: string
        description: "Status: `ok` or `fail`."
      violations:
        type: array
        items: { $ref: "#/definitions/Violation" }
        x-rust-type: "Vec<Violation>"
        description: "Violations found."
    x-rust-struct:
      derive: [Debug, Clone, Serialize, Deserialize]

  CheckResult:
    type: object
    required: [files, total_violations, passed, coverage]
    description: Aggregate result from `spec_alignment::check()`.
    properties:
      files:
        type: array
        items: { $ref: "#/definitions/FileResult" }
        x-rust-type: "Vec<FileResult>"
        description: "Per-file results."
      total_violations:
        type: integer
        x-rust-type: "usize"
        description: "Total violation count across all files."
      passed:
        type: boolean
        description: "True if no violations and no uncovered requirements."
      coverage:
        type: object
        x-rust-type: "Option<CoverageReport>"
        x-serde-skip-if: "Option::is_none"
        description: "Coverage report (present when check_with_coverage is used)."
    x-rust-struct:
      derive: [Debug, Clone, Serialize, Deserialize]

  SpecAnnotation:
    type: object
    required: [spec_path, requirement_id, source_file, line, comment_syntax]
    description: A `@spec {path}#{id}` annotation found in source code.
    properties:
      spec_path:
        type: string
        description: "Spec file path referenced."
      requirement_id:
        type: string
        description: "Requirement ID (e.g. `R1`)."
      source_file:
        type: string
        description: "Source file where the annotation was found."
      line:
        type: integer
        x-rust-type: "usize"
        description: "Line number (1-based)."
      comment_syntax:
        type: string
        description: "Comment syntax (`//`, `#`, `--`, `<!--`, `/*`)."
    x-rust-struct:
      derive: [Debug, Clone, Serialize, Deserialize]

  CoverageReport:
    type: object
    required: [covered, uncovered_requirements, unspecced_functions, stale_annotations, orphan_requirements, schema_struct_mismatches, coverage_ratio]
    description: Coverage analysis report.
    properties:
      covered:
        type: array
        items: { $ref: "#/definitions/CoverageEntry" }
        x-rust-type: "Vec<CoverageEntry>"
        description: "Requirements with matching annotations."
      uncovered_requirements:
        type: array
        items: { $ref: "#/definitions/CoverageEntry" }
        x-rust-type: "Vec<CoverageEntry>"
        description: "Requirements with no matching annotations."
      unspecced_functions:
        type: array
        items: { $ref: "#/definitions/UnspeccedFunction" }
        x-rust-type: "Vec<UnspeccedFunction>"
        description: "Public fns without `@spec`."
      stale_annotations:
        type: array
        items: { $ref: "#/definitions/SpecAnnotation" }
        x-rust-type: "Vec<SpecAnnotation>"
        description: "Annotations pointing to non-existent paths."
      orphan_requirements:
        type: array
        items: { $ref: "#/definitions/OrphanRequirementEntry" }
        x-rust-type: "Vec<OrphanRequirementEntry>"
        description: "Requirements not referenced by scenarios."
      schema_struct_mismatches:
        type: array
        items: { $ref: "#/definitions/SchemaStructMismatchEntry" }
        x-rust-type: "Vec<SchemaStructMismatchEntry>"
        description: "Schema/struct property mismatches."
      coverage_ratio:
        type: number
        x-rust-type: "f64"
        description: "Ratio of covered requirements (0.0–1.0)."
    x-rust-struct:
      derive: [Debug, Clone, Serialize, Deserialize]

  CoverageEntry:
    type: object
    required: [requirement_id, spec_path, status, annotations]
    description: A single requirement's coverage status.
    properties:
      requirement_id:
        type: string
        description: "Requirement ID."
      spec_path:
        type: string
        description: "Spec file path."
      status:
        type: string
        description: "Coverage status: `covered` or `uncovered`."
      annotations:
        type: array
        items: { $ref: "#/definitions/SpecAnnotation" }
        x-rust-type: "Vec<SpecAnnotation>"
        description: "@spec annotations matching this requirement."
    x-rust-struct:
      derive: [Debug, Clone, Serialize, Deserialize]

  OrphanRequirementEntry:
    type: object
    required: [requirement_id, spec_path, description]
    description: A requirement in the Requirements table not referenced by any scenario.
    properties:
      requirement_id:
        type: string
        description: "Requirement ID."
      spec_path:
        type: string
        description: "Spec file path."
      description:
        type: string
        x-rust-type: "Option<String>"
        description: "Description from the requirements table."
    x-rust-struct:
      derive: [Debug, Clone, Serialize, Deserialize]

  UnspeccedFunction:
    type: object
    required: [name, kind, source_file, line]
    description: A public function without a `@spec` annotation.
    properties:
      name:
        type: string
        description: "Function name."
      kind:
        type: string
        description: "Symbol kind."
      source_file:
        type: string
        description: "Source file path."
      line:
        type: integer
        x-rust-type: "usize"
        description: "Line number (1-based)."
    x-rust-struct:
      derive: [Debug, Clone, Serialize, Deserialize]

  SchemaStructMismatchEntry:
    type: object
    required: [schema_name, field, kind, spec_path]
    description: A mismatch between JSON Schema properties and Rust struct fields.
    properties:
      schema_name:
        type: string
        description: "Schema/struct name."
      field:
        type: string
        description: "Field name."
      kind:
        type: string
        description: "Mismatch kind."
      spec_path:
        type: string
        description: "Spec file path."
    x-rust-struct:
      derive: [Debug, Clone, Serialize, Deserialize]
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/spec_alignment/models.rs
    action: modify
    section: schema
    impl_mode: codegen
    replaces:
      - SpecDocument
      - SpecSection
      - SectionAnnotation
      - CodeBlock
      - ViolationKind
      - Violation
      - FileResult
      - CheckResult
      - SpecAnnotation
      - CoverageReport
      - CoverageEntry
      - OrphanRequirementEntry
      - UnspeccedFunction
      - SchemaStructMismatchEntry
    description: |
      Codegen replaces all fourteen type declarations.
```

# Reviews

## Review 1
<!-- type: doc lang: markdown -->

**Verdict:** approved

- [overview] 14 types covering spec alignment Phase 1 and Phase 2 reporting; mix of structs and one large enum.
- [schema] All well-formed; foreign types via x-rust-type; ViolationKind enum with snake_case rename_all.
- [changes] All 14 in `replaces`; impls for ViolationKind preserved hand-written.
