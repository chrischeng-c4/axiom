---
id: sdd-models-context
fill_sections: [overview, schema, changes]
capability_refs:
  - id: aw-core-client-model-workitem-first-artifact-lifecycle
    role: primary
    gap: core-concept-model-and-invariants
    claim: core-concept-model-and-invariants
    coverage: full
    rationale: "Core model/parser TDs define AW Core domain nouns, invariants, and artifact structure."
---

# Models: Context

## Overview
<!-- type: overview lang: markdown -->

Context artifact data models for the 3-stage exploration pipeline in `projects/agentic-workflow`.

- `SpecRef` — reference to an existing spec with relevance assessment (id, group, relevance, reason, key_sections).
- `DocRef` — reference to a knowledge document with summary (path, summary, relevant_sections).
- `PatternRef` — reference to a pattern found in the knowledge base (name, source, description).
- `FileRef` — reference to a codebase file with symbols and role (path, symbols, role).
- `LensResult` — result from a single Lens tool invocation (tool, query, summary).
- `SpecContext` — aggregated spec-exploration result (scanned_groups, specs, dependencies, gaps); legacy field, now part of unified reference_context.
- `KnowledgeContext` — aggregated knowledge-exploration result (scanned_categories, docs, patterns, pitfalls); legacy field, now part of unified reference_context.
- `CodebaseContext` — aggregated codebase-analysis result (lens_tools_used, files, lens_results, dependency_graph); legacy field, now part of unified reference_context.
- `ReviewVerdict` — 3-variant unit enum (Approved / Reviewed / Rejected) serialized as SCREAMING_SNAKE_CASE.
- `MissingItem` — missing checklist item in review feedback (checklist_item, details).
- `Inaccuracy` — inaccuracy found during review (location, expected, actual).
- `ReviewFeedback` — structured feedback record for a REVIEWED verdict (verdict, stage, iteration, artifact_file, missing_items, inaccuracies).
- `ContextType` — 3-variant unit enum (SpecContext / KnowledgeContext / CodebaseContext) serialized as snake_case; carries a codegen `filename()` dispatch method returning the canonical artifact filename for each variant.

This spec is the codegen migration target for `projects/agentic-workflow/src/models/context.rs` (281 LOC). All 11 structs and 2 enums are covered by the `## Schema` section with full `x-rust-struct` / `x-rust-enum` / `x-serde-default` / `x-methods` annotations so that `aw td gen-code` can regenerate the serde import, all type declarations, and the `ContextType::filename()` impl block within a CODEGEN-BEGIN/CODEGEN-END region. The existing `#[cfg(test)] mod tests` block (11 serialization/deserialization unit tests) is preserved hand-written outside any CODEGEN delimiters.
## Schema
<!-- type: schema lang: yaml -->

```yaml
definitions:
  SpecRef:
    type: object
    description: Reference to an existing spec with relevance assessment.
    required: [id, group, relevance, reason, key_sections]
    properties:
      id:
        type: string
        description: "Spec identifier."
      group:
        type: string
        description: "Spec group (defaults to empty string)."
        x-serde-default: true
      relevance:
        type: string
        description: "Relevance level (e.g. high, medium, low)."
      reason:
        type: string
        description: "Why this spec is relevant (defaults to empty string)."
        x-serde-default: true
      key_sections:
        type: array
        items:
          type: string
        description: "Key sections within the spec that are relevant."
        x-serde-default: true
    x-rust-struct:
      derive: [Debug, Clone, Serialize, Deserialize]

  DocRef:
    type: object
    description: Reference to a knowledge document with summary.
    required: [path, summary, relevant_sections]
    properties:
      path:
        type: string
        description: "Document file path."
      summary:
        type: string
        description: "Summary of the document's relevance."
      relevant_sections:
        type: array
        items:
          type: string
        description: "Specific sections within the document that are relevant."
        x-serde-default: true
    x-rust-struct:
      derive: [Debug, Clone, Serialize, Deserialize]

  PatternRef:
    type: object
    description: Reference to a pattern found in the knowledge base.
    required: [name, source, description]
    properties:
      name:
        type: string
        description: "Pattern name."
      source:
        type: string
        description: "Source document where the pattern is described."
      description:
        type: string
        description: "Description of the pattern."
    x-rust-struct:
      derive: [Debug, Clone, Serialize, Deserialize]

  FileRef:
    type: object
    description: Reference to a codebase file with symbols and role.
    required: [path, symbols, role]
    properties:
      path:
        type: string
        description: "File path relative to workspace root."
      symbols:
        type: array
        items:
          type: string
        description: "Symbols of interest within the file."
        x-serde-default: true
      role:
        type: string
        description: "Role of this file in the change (defaults to empty string)."
        x-serde-default: true
    x-rust-struct:
      derive: [Debug, Clone, Serialize, Deserialize]

  LensResult:
    type: object
    description: Result from a Lens tool invocation.
    required: [tool, query, summary]
    properties:
      tool:
        type: string
        description: "Name of the Lens tool invoked."
      query:
        type: string
        description: "Query string passed to the tool."
      summary:
        type: string
        description: "Summary of the tool result."
    x-rust-struct:
      derive: [Debug, Clone, Serialize, Deserialize]

  SpecContext:
    type: object
    description: |
      Spec context data (legacy — now part of unified reference_context).
      Captures analysis of existing main specs relevant to the change.
    required: [scanned_groups, specs, dependencies, gaps]
    properties:
      scanned_groups:
        type: array
        items:
          type: string
        description: "Spec groups that were scanned during exploration."
      specs:
        type: array
        items:
          $ref: "#/definitions/SpecRef"
        description: "Relevant spec references found."
      dependencies:
        type: array
        items:
          type: string
        description: "Spec dependency identifiers."
      gaps:
        type: array
        items:
          type: string
        description: "Identified gaps in existing specs."
    x-rust-struct:
      derive: [Debug, Clone, Serialize, Deserialize]

  KnowledgeContext:
    type: object
    description: |
      Knowledge context data (legacy — now part of unified reference_context).
      Captures analysis of knowledge base documents relevant to the change.
    required: [scanned_categories, docs, patterns, pitfalls]
    properties:
      scanned_categories:
        type: array
        items:
          type: string
        description: "Knowledge categories that were scanned."
      docs:
        type: array
        items:
          $ref: "#/definitions/DocRef"
        description: "Relevant knowledge documents found."
      patterns:
        type: array
        items:
          $ref: "#/definitions/PatternRef"
        description: "Relevant patterns found."
      pitfalls:
        type: array
        items:
          type: string
        description: "Pitfalls to be aware of (defaults to empty list)."
        x-serde-default: true
    x-rust-struct:
      derive: [Debug, Clone, Serialize, Deserialize]

  CodebaseContext:
    type: object
    description: |
      Codebase context data (legacy — now part of unified reference_context).
      Captures codebase analysis using Lens tools. lens_tools_used proves
      which analysis tools were invoked.
    required: [lens_tools_used, files, lens_results, dependency_graph]
    properties:
      lens_tools_used:
        type: array
        items:
          type: string
        description: "Lens tools that were used during codebase analysis."
      files:
        type: array
        items:
          $ref: "#/definitions/FileRef"
        description: "Relevant codebase files found."
      lens_results:
        type: array
        items:
          $ref: "#/definitions/LensResult"
        description: "Results from each Lens tool invocation."
      dependency_graph:
        type: array
        items:
          type: string
        description: "Dependency graph edges as strings (defaults to empty list)."
        x-serde-default: true
    x-rust-struct:
      derive: [Debug, Clone, Serialize, Deserialize]

  ReviewVerdict:
    type: string
    enum: [APPROVED, REVIEWED, REJECTED]
    description: Review verdict from agent self-review.
    x-rust-enum:
      derive: [Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize]
      serde_rename_all: SCREAMING_SNAKE_CASE
      variants:
        - name: Approved
          doc: "Artifact is approved with no required changes."
        - name: Reviewed
          doc: "Artifact is reviewed with feedback; revisions required."
        - name: Rejected
          doc: "Artifact is rejected; significant rework required."

  MissingItem:
    type: object
    description: A missing checklist item in review feedback.
    required: [checklist_item, details]
    properties:
      checklist_item:
        type: string
        description: "The checklist item that is missing."
      details:
        type: string
        description: "Details explaining what is missing and why it matters."
    x-rust-struct:
      derive: [Debug, Clone, Serialize, Deserialize]

  Inaccuracy:
    type: object
    description: An inaccuracy found during review.
    required: [location, expected, actual]
    properties:
      location:
        type: string
        description: "Location of the inaccuracy (e.g. section name or field path)."
      expected:
        type: string
        description: "What the correct value or content should be."
      actual:
        type: string
        description: "What was actually found."
    x-rust-struct:
      derive: [Debug, Clone, Serialize, Deserialize]

  ReviewFeedback:
    type: object
    description: Structured feedback for REVIEWED verdict.
    required: [verdict, stage, iteration, artifact_file, missing_items, inaccuracies]
    properties:
      verdict:
        $ref: "#/definitions/ReviewVerdict"
        description: "The review verdict."
      stage:
        type: string
        description: "The workflow stage being reviewed."
      iteration:
        type: integer
        description: "Iteration number of the review (u32)."
        x-rust-type: u32
      artifact_file:
        type: string
        description: "Path of the artifact file being reviewed."
      missing_items:
        type: array
        items:
          $ref: "#/definitions/MissingItem"
        description: "Missing checklist items found during review."
        x-serde-default: true
      inaccuracies:
        type: array
        items:
          $ref: "#/definitions/Inaccuracy"
        description: "Inaccuracies found during review."
        x-serde-default: true
    x-rust-struct:
      derive: [Debug, Clone, Serialize, Deserialize]

  ContextType:
    type: string
    enum: [spec_context, knowledge_context, codebase_context]
    description: Context artifact type identifier.
    x-rust-enum:
      derive: [Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize]
      serde_rename_all: snake_case
      variants:
        - name: SpecContext
          doc: "Spec context artifact (serialized as spec_context)."
        - name: KnowledgeContext
          doc: "Knowledge context artifact (serialized as knowledge_context)."
        - name: CodebaseContext
          doc: "Codebase context artifact (serialized as codebase_context)."
    x-methods:
      - name: filename
        returns: "&'static str"
        impl_mode: codegen
        doc: "Get the filename for this context type."
        dispatch:
          - variant: SpecContext
            value: "spec_context.md"
          - variant: KnowledgeContext
            value: "knowledge_context.md"
          - variant: CodebaseContext
            value: "codebase_context.md"
```
## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/models/context.rs
    action: modify
    section: schema
    impl_mode: codegen
    replaces:
      - SpecRef
      - DocRef
      - PatternRef
      - FileRef
      - LensResult
      - SpecContext
      - KnowledgeContext
      - CodebaseContext
      - ReviewVerdict
      - ReviewFeedback
      - MissingItem
      - Inaccuracy
      - ContextType
      - "impl ContextType"
    description: |
      Codegen replaces the serde import, 13 type declarations, and 1 impl block within a
      CODEGEN-BEGIN/CODEGEN-END region: SpecRef, DocRef, PatternRef, FileRef,
      LensResult, SpecContext, KnowledgeContext, CodebaseContext struct
      declarations; ReviewVerdict and ContextType enum declarations;
      ReviewFeedback, MissingItem, Inaccuracy struct declarations; and the
      ContextType::filename() dispatch impl block. All generated items carry
      @spec markers referencing this file's #schema anchor.
      ContextType's codegen methods (filename) are emitted as a standalone
      impl ContextType block inside CODEGEN-BEGIN/END.
  - path: projects/agentic-workflow/src/models/context.rs
    action: modify
    section: schema
    impl_mode: hand-written
    description: |
      Hand-written: the #[cfg(test)] mod tests block (11 serialization and
      deserialization unit tests) is preserved outside any CODEGEN-BEGIN/END
      block. aw td gen-code must not touch this block on an action: modify
      file.
```

# Reviews

## Review 1
<!-- type: doc lang: markdown -->
**Verdict:** approved

- [schema] All 11 structs and 2 enums from the 281-LOC source are faithfully represented; `x-serde-default`, `x-rust-struct.derive`, `x-rust-enum.serde_rename_all`, `x-rust-type`, and `x-methods` annotations are complete and consistent with the source.
- [changes] `replaces` list correctly enumerates all 13 type names plus `impl ContextType`; the dual-entry split (codegen region vs. hand-written test block) unambiguously scopes what `aw td gen-code` must not touch.
